mod lifecycle;

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use clap::{Args, Parser, Subcommand, ValueEnum};
use nox_config::{example_config, load, Capability, Profile, Target};

#[derive(Debug, Parser)]
#[command(name = "noxctl", version, about = "Control a declarative Nox system")]
struct Cli {
    #[command(subcommand)]
    command: CommandKind,
}

#[derive(Debug, Subcommand)]
enum CommandKind {
    /// Write a starter nox.toml without overwriting an existing file.
    Init(InitArgs),
    /// Validate a machine configuration.
    Validate(PathArgs),
    /// Render the planned NixOS activation steps.
    Plan(PathArgs),
    /// Check host prerequisites and repository shape.
    Doctor,
    /// Build the machine artifact without activating it.
    Build(BuildArgs),
    /// Resolve and write the machine project lock file.
    Lock(PathArgs),
    /// Plan or explicitly execute a destructive SSH installation.
    Install(InstallArgs),
    /// Build the selected target image (WSL produces its tarball builder).
    Image(ImageArgs),
    /// Placeholder for safe activation.
    Apply(PathArgs),
    /// List local NixOS system generations.
    Generations,
    /// Placeholder for rollback.
    Rollback,
}

#[derive(Debug, Args)]
struct PathArgs {
    /// Configuration file to read.
    #[arg(short, long, default_value = "nox.toml")]
    config: PathBuf,
}

#[derive(Debug, Args)]
struct BuildArgs {
    #[arg(short, long, default_value = "nox.toml")]
    config: PathBuf,
    #[arg(long)]
    dry_run: bool,
    #[arg(long, default_value = "result")]
    out_link: PathBuf,
}

#[derive(Debug, Args)]
struct InstallArgs {
    #[arg(short, long, default_value = "nox.toml")]
    config: PathBuf,
    #[arg(long)]
    host: String,
    #[arg(long)]
    execute: bool,
    #[arg(long)]
    confirm_host: Option<String>,
    #[arg(long)]
    confirm_disk: Option<String>,
}

#[derive(Debug, Args)]
struct InitArgs {
    /// Nox flake source. Use path:/absolute/checkout when developing.
    #[arg(long, default_value = "github:swarnimarun/nox")]
    source: String,
    #[arg(long, default_value = "x86_64-linux")]
    system: String,
    /// Directory in which nox.toml should be created.
    #[arg(default_value = ".")]
    directory: PathBuf,
    #[arg(long, value_enum, default_value_t = ProfileName::Server)]
    profile: ProfileName,
    #[arg(long, value_enum, default_value_t = TargetName::Metal)]
    target: TargetName,
}

#[derive(Debug, Args)]
struct ImageArgs {
    #[command(subcommand)]
    command: ImageCommand,
}

#[derive(Debug, Subcommand)]
enum ImageCommand {
    Build(BuildArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ProfileName {
    Server,
    Desktop,
    Gaming,
    Workspace,
    Recovery,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TargetName {
    Metal,
    Iso,
    Qcow2,
    Wsl,
    Oci,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    match Cli::parse().command {
        CommandKind::Init(args) => init(args),
        CommandKind::Validate(args) => validate(&args.config),
        CommandKind::Plan(args) => plan(&args.config),
        CommandKind::Doctor => doctor(),
        CommandKind::Build(args) => lifecycle::build(&args.config, args.dry_run, &args.out_link),
        CommandKind::Lock(args) => lifecycle::lock(&args.config),
        CommandKind::Install(args) => lifecycle::install(
            &args.config,
            &args.host,
            args.confirm_host.as_deref(),
            args.confirm_disk.as_deref(),
            args.execute,
        ),
        CommandKind::Image(ImageArgs { command: ImageCommand::Build(args) }) => {
            lifecycle::build(&args.config, args.dry_run, &args.out_link)
        }
        CommandKind::Apply(args) => future_command("apply", &args.config),
        CommandKind::Generations => lifecycle::generations(),
        CommandKind::Rollback => future_command("rollback", Path::new(".")),
    }
}

fn init(args: InitArgs) -> Result<(), Box<dyn Error>> {
    let flake = lifecycle::machine_flake(&args.source, &args.system)?;
    let directory = args.directory;
    if directory.exists() && fs::read_dir(&directory)?.next().is_some() {
        return Err(
            "init requires an empty directory; existing files will not be overwritten".into()
        );
    }
    let path = directory.join("nox.toml");
    if path.exists() {
        return Err(format!("{} already exists; refusing to overwrite", path.display()).into());
    }
    let mut config = example_config();
    config.profile = args.profile.into();
    config.target = args.target.into();
    if config.profile == Profile::Gaming {
        config.capabilities.insert(Capability::Gaming);
    }
    if config.profile == Profile::Recovery {
        config.capabilities.insert(Capability::Recovery);
    }
    if config.profile == Profile::Workspace {
        config.capabilities.insert(Capability::Development);
    }
    config.name = directory
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("nox-machine")
        .to_owned();
    config.validate().map_err(|errors| errors.join("\n"))?;
    fs::create_dir_all(&directory)?;
    lifecycle::write_new(&path, &toml::to_string_pretty(&config)?)?;
    lifecycle::write_new(&directory.join("flake.nix"), &flake)?;
    println!("Next: noxctl lock --config {}", path.display());
    println!("created {}", path.display());
    Ok(())
}

fn validate(path: &Path) -> Result<(), Box<dyn Error>> {
    let config = load(path)?;
    println!("valid: {}", path.display());
    println!("  profile: {}\n  target: {}", config.profile, config.target);
    Ok(())
}

fn plan(path: &Path) -> Result<(), Box<dyn Error>> {
    let config = load(path)?;
    print!("{}", nox_core::plan(&config).render_text());
    Ok(())
}

fn doctor() -> Result<(), Box<dyn Error>> {
    match Command::new("nix").arg("--version").output() {
        Ok(output) if output.status.success() => {
            println!("ok: {}", String::from_utf8_lossy(&output.stdout).trim())
        }
        Ok(_) => println!("warning: nix is installed but did not report a version"),
        Err(_) => {
            println!("warning: nix was not found; install Nix with flakes support before building")
        }
    }
    if Path::new("flake.nix").exists() {
        println!("ok: flake.nix found");
    } else {
        println!("warning: flake.nix not found in the current directory");
    }
    Ok(())
}

fn future_command(command: &str, _path: &Path) -> Result<(), Box<dyn Error>> {
    Err(format!("`noxctl {command}` is intentionally reserved for the next milestone; use `validate` or `plan` today").into())
}

impl From<ProfileName> for Profile {
    fn from(value: ProfileName) -> Self {
        match value {
            ProfileName::Server => Self::Server,
            ProfileName::Desktop => Self::Desktop,
            ProfileName::Gaming => Self::Gaming,
            ProfileName::Workspace => Self::Workspace,
            ProfileName::Recovery => Self::Recovery,
        }
    }
}

impl From<TargetName> for Target {
    fn from(value: TargetName) -> Self {
        match value {
            TargetName::Metal => Self::Metal,
            TargetName::Iso => Self::Iso,
            TargetName::Qcow2 => Self::Qcow2,
            TargetName::Wsl => Self::Wsl,
            TargetName::Oci => Self::Oci,
        }
    }
}
