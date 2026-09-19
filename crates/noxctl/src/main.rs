use std::{error::Error, fs, path::{Path, PathBuf}, process::Command};

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
    /// Placeholder for the Nix build integration.
    Build(PathArgs),
    /// Placeholder for target image generation.
    Image(ImageArgs),
    /// Placeholder for safe activation.
    Apply(PathArgs),
    /// Placeholder for generation inspection.
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
struct InitArgs {
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
    Build(PathArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ProfileName { Server, Desktop, Gaming, Workspace, Recovery }

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TargetName { Metal, Iso, Qcow2, Wsl, Oci }

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
        CommandKind::Build(args) => future_command("build", &args.config),
        CommandKind::Image(ImageArgs { command: ImageCommand::Build(args) }) => {
            future_command("image build", &args.config)
        }
        CommandKind::Apply(args) => future_command("apply", &args.config),
        CommandKind::Generations => future_command("generations", Path::new(".")),
        CommandKind::Rollback => future_command("rollback", Path::new(".")),
    }
}

fn init(args: InitArgs) -> Result<(), Box<dyn Error>> {
    let directory = args.directory;
    fs::create_dir_all(&directory)?;
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
    fs::write(&path, toml::to_string_pretty(&config)?)?;
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
        Err(_) => println!("warning: nix was not found; enter `nix develop` before building"),
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
        match value { ProfileName::Server => Self::Server, ProfileName::Desktop => Self::Desktop, ProfileName::Gaming => Self::Gaming, ProfileName::Workspace => Self::Workspace, ProfileName::Recovery => Self::Recovery }
    }
}

impl From<TargetName> for Target {
    fn from(value: TargetName) -> Self {
        match value { TargetName::Metal => Self::Metal, TargetName::Iso => Self::Iso, TargetName::Qcow2 => Self::Qcow2, TargetName::Wsl => Self::Wsl, TargetName::Oci => Self::Oci }
    }
}
