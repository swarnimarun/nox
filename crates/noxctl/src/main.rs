mod lifecycle;

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use clap::{Args, Parser, Subcommand, ValueEnum};
use nox_config::{
    example_config, load, Bootloader, Capability, DesktopFlavour, Filesystem, GraphicsDriver,
    Profile, Target,
};

#[derive(Debug, Parser)]
#[command(name = "noxctl", version, about = "Control a declarative Nox system")]
struct Cli {
    #[command(subcommand)]
    command: CommandKind,
}

#[derive(Debug, Subcommand)]
enum CommandKind {
    /// Write a starter project without overwriting existing files.
    Init(InitArgs),
    /// Create a starter project and lock all flake inputs.
    Setup(InitArgs),
    /// Inspect or update locked flake dependencies.
    Upgrade(UpgradeArgs),
    /// Register user-owned NixOS modules in nox.toml.
    Module(ModuleArgs),
    /// Create a Git-ready Home Manager dotfiles flake.
    Dotfiles(DotfilesArgs),
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
    /// Launch the graphical installer or install the local machine with explicit guards.
    Installer(InstallerArgs),
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
    /// Desktop compositor. Desktop and gaming projects default to Hyprland.
    #[arg(long, value_enum)]
    flavour: Option<FlavourName>,
    #[arg(long, value_enum, default_value_t = GraphicsName::Auto)]
    graphics: GraphicsName,
    #[arg(long, value_enum, default_value_t = BootloaderName::SystemdBoot)]
    bootloader: BootloaderName,
    #[arg(long, value_enum, default_value_t = FilesystemName::Btrfs)]
    filesystem: FilesystemName,
    /// Stable installation target. Only /dev/disk/by-id/... is accepted.
    #[arg(long)]
    disk: Option<String>,
    #[arg(long, default_value = "nox")]
    username: String,
    #[arg(long, default_value = "en_US.UTF-8")]
    locale: String,
    #[arg(long, default_value = "UTC")]
    timezone: String,
    #[arg(long, default_value = "us")]
    keymap: String,
    /// Flake exporting nixosModules.default, commonly backed by a Git repository.
    #[arg(long)]
    dotfiles_flake: Option<String>,
}

#[derive(Debug, Args)]
struct UpgradeArgs {
    #[command(subcommand)]
    command: UpgradeCommand,
}

#[derive(Debug, Subcommand)]
enum UpgradeCommand {
    /// Show dependency changes without writing flake.lock.
    List(PathArgs),
    /// Update flake.lock, then evaluate and build before keeping it.
    Apply(PathArgs),
}

#[derive(Debug, Args)]
struct ModuleArgs {
    #[command(subcommand)]
    command: ModuleCommand,
}

#[derive(Debug, Subcommand)]
enum ModuleCommand {
    /// List NixOS modules registered in nox.toml.
    List(PathArgs),
    /// Register an existing relative .nix file.
    Add(ModuleEditArgs),
    /// Stop importing a relative .nix file without deleting it.
    Remove(ModuleEditArgs),
}

#[derive(Debug, Args)]
struct ModuleEditArgs {
    #[arg(short, long, default_value = "nox.toml")]
    config: PathBuf,
    /// Relative path inside the machine project.
    module: String,
}

#[derive(Debug, Args)]
struct DotfilesArgs {
    #[command(subcommand)]
    command: DotfilesCommand,
}

#[derive(Debug, Subcommand)]
enum DotfilesCommand {
    /// Create and lock a Home Manager flake suitable for a Git repository.
    Init(DotfilesInitArgs),
}

#[derive(Debug, Args)]
struct DotfilesInitArgs {
    /// Empty directory in which the dotfiles flake is created.
    directory: PathBuf,
    #[arg(long, default_value = "nox")]
    username: String,
    /// Create the files without invoking Nix.
    #[arg(long)]
    no_lock: bool,
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

#[derive(Debug, Args)]
struct InstallerArgs {
    #[command(subcommand)]
    command: InstallerCommand,
}

#[derive(Debug, Subcommand)]
enum InstallerCommand {
    /// Start the GTK installer wizard.
    Gui {
        /// Exercise generation and validation without touching a disk.
        #[arg(long)]
        dry_run: bool,
    },
    /// Install the current live system onto a local disk.
    Local(LocalInstallArgs),
}

#[derive(Debug, Args)]
struct LocalInstallArgs {
    #[arg(short, long, default_value = "nox.toml")]
    config: PathBuf,
    /// Required acknowledgement that disk contents will be destroyed.
    #[arg(long)]
    execute: bool,
    /// Must exactly match install.disk in nox.toml.
    #[arg(long)]
    confirm_disk: Option<String>,
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

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FlavourName {
    None,
    Hyprland,
    Niri,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum GraphicsName {
    Auto,
    Amd,
    Intel,
    NvidiaOpen,
    NvidiaProprietary,
    Vm,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BootloaderName {
    SystemdBoot,
    GrubEfi,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FilesystemName {
    Btrfs,
    Ext4,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    match Cli::parse().command {
        CommandKind::Init(args) => initialize(args, false),
        CommandKind::Setup(args) => initialize(args, true),
        CommandKind::Upgrade(UpgradeArgs { command: UpgradeCommand::List(args) }) => {
            lifecycle::upgrade_list(&args.config)
        }
        CommandKind::Upgrade(UpgradeArgs { command: UpgradeCommand::Apply(args) }) => {
            lifecycle::upgrade_apply(&args.config)
        }
        CommandKind::Module(ModuleArgs { command }) => module_command(command),
        CommandKind::Dotfiles(DotfilesArgs { command: DotfilesCommand::Init(args) }) => {
            dotfiles_init(args)
        }
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
        CommandKind::Installer(InstallerArgs { command: InstallerCommand::Gui { dry_run } }) => {
            lifecycle::installer_gui(dry_run)
        }
        CommandKind::Installer(InstallerArgs { command: InstallerCommand::Local(args) }) => {
            lifecycle::install_local(&args.config, args.confirm_disk.as_deref(), args.execute)
        }
        CommandKind::Apply(args) => future_command("apply", &args.config),
        CommandKind::Generations => lifecycle::generations(),
        CommandKind::Rollback => future_command("rollback", Path::new(".")),
    }
}

fn initialize(args: InitArgs, lock_after: bool) -> Result<(), Box<dyn Error>> {
    let flake =
        lifecycle::machine_flake(&args.source, &args.system, args.dotfiles_flake.as_deref())?;
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
    config.capabilities = match config.profile {
        Profile::Server => [Capability::Apps, Capability::RemoteManagement].into_iter().collect(),
        Profile::Desktop => [Capability::Desktop, Capability::Development].into_iter().collect(),
        Profile::Gaming => [Capability::Desktop, Capability::Gaming].into_iter().collect(),
        Profile::Workspace => [Capability::Development].into_iter().collect(),
        Profile::Recovery => {
            [Capability::Recovery, Capability::RemoteManagement].into_iter().collect()
        }
    };
    config.desktop.flavour = Some(args.flavour.map(Into::into).unwrap_or_else(|| {
        if matches!(config.profile, Profile::Desktop | Profile::Gaming) {
            DesktopFlavour::Hyprland
        } else {
            DesktopFlavour::None
        }
    }));
    config.hardware.graphics = args.graphics.into();
    config.boot.loader = args.bootloader.into();
    config.install.filesystem = args.filesystem.into();
    config.install.disk = args.disk;
    config.user.name = args.username;
    config.locale.locale = args.locale;
    config.locale.timezone = args.timezone;
    config.locale.keymap = args.keymap;
    config.nix.extra_modules.push("modules/system.nix".to_owned());
    config.name = directory
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("nox-machine")
        .to_owned();
    config.validate().map_err(|errors| errors.join("\n"))?;
    fs::create_dir_all(directory.join("modules"))?;
    lifecycle::write_new(&path, &toml::to_string_pretty(&config)?)?;
    lifecycle::write_new(&directory.join("flake.nix"), &flake)?;
    lifecycle::write_new(&directory.join("modules/system.nix"), lifecycle::machine_module())?;
    println!("created {}", path.display());
    if lock_after {
        lifecycle::lock(&path)?;
        println!("setup complete; review and commit nox.toml, flake.nix, and flake.lock");
    } else {
        println!("Next: noxctl lock --config {}", path.display());
    }
    Ok(())
}

fn module_command(command: ModuleCommand) -> Result<(), Box<dyn Error>> {
    match command {
        ModuleCommand::List(args) => {
            let config = load(&args.config)?;
            if config.nix.extra_modules.is_empty() {
                println!("no custom modules");
            } else {
                for module in config.nix.extra_modules {
                    println!("{module}");
                }
            }
            Ok(())
        }
        ModuleCommand::Add(args) => edit_module(args, true),
        ModuleCommand::Remove(args) => edit_module(args, false),
    }
}

fn edit_module(args: ModuleEditArgs, add: bool) -> Result<(), Box<dyn Error>> {
    let root = lifecycle::project(&args.config)?;
    let mut config = load(&args.config)?;
    if add {
        if config.nix.extra_modules.iter().any(|item| item == &args.module) {
            return Err(format!("{} is already registered", args.module).into());
        }
        config.nix.extra_modules.push(args.module.clone());
        config.validate().map_err(|errors| errors.join("\n"))?;
        let module = root.join(&args.module).canonicalize().map_err(|_| {
            format!(
                "{} does not exist; create the user-owned module before registering it",
                args.module
            )
        })?;
        if !module.starts_with(&root) || !module.is_file() {
            return Err("module must resolve to a file inside the machine project".into());
        }
    } else {
        let before = config.nix.extra_modules.len();
        config.nix.extra_modules.retain(|item| item != &args.module);
        if config.nix.extra_modules.len() == before {
            return Err(format!("{} is not registered", args.module).into());
        }
    }
    lifecycle::write_config(&args.config, &config)?;
    println!("{} {}", if add { "registered" } else { "unregistered" }, args.module);
    Ok(())
}

fn dotfiles_init(args: DotfilesInitArgs) -> Result<(), Box<dyn Error>> {
    let mut validation = example_config();
    validation.user.name = args.username.clone();
    validation.validate().map_err(|errors| errors.join("\n"))?;
    if args.directory.exists() && fs::read_dir(&args.directory)?.next().is_some() {
        return Err("dotfiles init requires an empty directory".into());
    }
    fs::create_dir_all(&args.directory)?;
    lifecycle::write_new(
        &args.directory.join("flake.nix"),
        &lifecycle::dotfiles_flake(&args.username),
    )?;
    lifecycle::write_new(
        &args.directory.join("home.nix"),
        &lifecycle::home_module(&args.username),
    )?;
    lifecycle::write_new(&args.directory.join(".gitignore"), "/result\n")?;
    println!("created dotfiles flake at {}", args.directory.display());
    if !args.no_lock {
        lifecycle::lock_flake(&args.directory)?;
    }
    println!("Next: git init {}", args.directory.display());
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

impl From<FlavourName> for DesktopFlavour {
    fn from(value: FlavourName) -> Self {
        match value {
            FlavourName::None => Self::None,
            FlavourName::Hyprland => Self::Hyprland,
            FlavourName::Niri => Self::Niri,
        }
    }
}

impl From<GraphicsName> for GraphicsDriver {
    fn from(value: GraphicsName) -> Self {
        match value {
            GraphicsName::Auto => Self::Auto,
            GraphicsName::Amd => Self::Amd,
            GraphicsName::Intel => Self::Intel,
            GraphicsName::NvidiaOpen => Self::NvidiaOpen,
            GraphicsName::NvidiaProprietary => Self::NvidiaProprietary,
            GraphicsName::Vm => Self::Vm,
        }
    }
}

impl From<BootloaderName> for Bootloader {
    fn from(value: BootloaderName) -> Self {
        match value {
            BootloaderName::SystemdBoot => Self::SystemdBoot,
            BootloaderName::GrubEfi => Self::GrubEfi,
        }
    }
}

impl From<FilesystemName> for Filesystem {
    fn from(value: FilesystemName) -> Self {
        match value {
            FilesystemName::Btrfs => Self::Btrfs,
            FilesystemName::Ext4 => Self::Ext4,
        }
    }
}
