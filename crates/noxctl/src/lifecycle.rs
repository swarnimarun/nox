use nox_config::{load, Target};
use std::{
    env,
    error::Error,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn project(config: &Path) -> Result<PathBuf> {
    load(config)?;
    if config.file_name().and_then(|s| s.to_str()) != Some("nox.toml") {
        return Err("machine projects require a file named nox.toml".into());
    }
    let root = config.canonicalize()?.parent().ok_or("missing parent")?.to_path_buf();
    if !root.join("flake.nix").is_file() {
        return Err("missing machine flake.nix; create a project with noxctl init".into());
    }
    if root.to_string_lossy().contains(['#', '?']) {
        return Err("machine directory must not contain # or ?".into());
    }
    Ok(root)
}

pub fn reference(root: &Path) -> String {
    format!("path:{}", root.display())
}

pub fn invoke(program: &str, args: &[String], execute: bool) -> Result<()> {
    println!("{program} {}", args.iter().map(|a| format!("{a:?}")).collect::<Vec<_>>().join(" "));
    if execute && !Command::new(program).args(args).status()?.success() {
        return Err(format!("{program} failed; no subsequent operation was attempted").into());
    }
    Ok(())
}

pub fn require_lock(root: &Path) -> Result<()> {
    if !root.join("flake.lock").is_file() {
        return Err("flake.lock is missing; run noxctl lock --config PATH/nox.toml and review/commit the lock before building".into());
    }
    Ok(())
}

pub fn build(config: &Path, dry_run: bool, out_link: &Path) -> Result<()> {
    let c = load(config)?;
    let root = project(config)?;
    require_lock(&root)?;
    let args = vec![
        "build".into(),
        "--no-update-lock-file".into(),
        format!("{}#image", reference(&root)),
        "--out-link".into(),
        out_link.display().to_string(),
    ];
    invoke("nix", &args, !dry_run)?;
    if c.target == Target::Wsl {
        println!("WSL output is a tarball builder. Run sudo {}/bin/nixos-wsl-tarball-builder ./nox.wsl, then import on Windows.", out_link.display());
    }
    Ok(())
}

pub fn lock(config: &Path) -> Result<()> {
    let root = project(config)?;
    invoke("nix", &["flake".into(), "lock".into(), reference(&root)], true)
}

pub fn install(
    config: &Path,
    host: &str,
    confirm_host: Option<&str>,
    confirm_disk: Option<&str>,
    execute: bool,
) -> Result<()> {
    let c = load(config)?;
    if c.target != Target::Metal {
        return Err("install requires target = metal".into());
    }
    // Restrict to unambiguous SSH destinations; never pass user text through a shell.
    if host.is_empty()
        || host.starts_with('-')
        || !host.bytes().all(|b| b.is_ascii_alphanumeric() || b"@.-_:[]".contains(&b))
    {
        return Err("invalid SSH destination".into());
    }
    let root = project(config)?;
    require_lock(&root)?;
    let flake = reference(&root);
    println!("DESTRUCTIVE: nixos-anywhere will replace the selected destination's operating system and format its declared disks. Destination: {host}");
    let mut args = vec![
        "run".into(),
        "--no-update-lock-file".into(),
        format!("{flake}#installer"),
        "--".into(),
        "--flake".into(),
        format!("{flake}#nox"),
        "--target-host".into(),
        host.into(),
    ];
    if !execute {
        return invoke("nix", &args, false);
    }
    if confirm_host != Some(host) {
        return Err("--confirm-host must exactly match --host".into());
    }
    let disk = confirm_disk.ok_or("--confirm-disk must match the single Disko disk device")?;
    if !stable_disk(disk) {
        return Err("confirmation disk must be a concrete /dev/disk/by-id/... device".into());
    }
    // Freeze the project and all flake inputs before checking disks or installing.
    let flake = archive_flake(&flake)?;
    args[2] = format!("{flake}#installer");
    args[5] = format!("{flake}#nox");
    verify_single_disk(&flake, disk)?;
    invoke(
        "nix",
        &[
            "build".into(),
            "--no-update-lock-file".into(),
            "--no-link".into(),
            format!("{flake}#nixosConfigurations.nox.config.system.build.toplevel"),
        ],
        true,
    )?;
    invoke("nix", &args, true)
}

pub fn installer_gui(dry_run: bool) -> Result<()> {
    let mut args = Vec::new();
    if dry_run {
        args.push("--dry-run".to_owned());
    }
    invoke("nox-installer", &args, true)
}

pub fn install_local(config: &Path, confirm_disk: Option<&str>, execute: bool) -> Result<()> {
    let c = load(config)?;
    if c.target != Target::Metal {
        return Err("local installation requires target = metal".into());
    }
    let disk =
        c.install.disk.as_deref().ok_or("local installation requires install.disk in nox.toml")?;
    if !stable_disk(disk) {
        return Err("install.disk must be a concrete /dev/disk/by-id/... device".into());
    }
    let root = project(config)?;
    require_lock(&root)?;
    println!(
        "DESTRUCTIVE: local installation will erase {disk}, mount the new system at /mnt, and install Nox"
    );
    if !execute {
        println!(
            "Re-run with --execute --confirm-disk {disk} and provide the user password on stdin"
        );
        return Ok(());
    }
    if confirm_disk != Some(disk) {
        return Err("--confirm-disk must exactly match install.disk".into());
    }

    let flake = archive_flake(&reference(&root))?;
    verify_single_disk(&flake, disk)?;

    // A broken system must fail before password input or disk mutation.
    invoke(
        "nix",
        &[
            "build".into(),
            "--no-update-lock-file".into(),
            "--no-link".into(),
            format!("{flake}#nixosConfigurations.nox.config.system.build.toplevel"),
        ],
        true,
    )?;

    let password = read_password()?;
    invoke(
        "nix",
        &[
            "run".into(),
            "--no-update-lock-file".into(),
            format!("{flake}#disko"),
            "--".into(),
            "--mode".into(),
            "destroy,format,mount".into(),
            "--yes-wipe-all-disks".into(),
            "--flake".into(),
            format!("{flake}#nox"),
        ],
        true,
    )?;

    let target_root = install_root();
    let project_target = target_root.join("etc/nox");
    fs::create_dir_all(&project_target)?;
    invoke(
        "cp",
        &[
            "-a".into(),
            format!("{}/.", flake.trim_start_matches("path:")),
            project_target.display().to_string(),
        ],
        true,
    )?;
    invoke(
        "nixos-install",
        &[
            "--no-root-password".into(),
            "--root".into(),
            target_root.display().to_string(),
            "--flake".into(),
            format!("path:{}#nox", project_target.display()),
        ],
        true,
    )?;
    set_installed_password(&target_root, &c.user.name, &password)?;
    println!("installation complete; reboot only after reviewing the installer output");
    Ok(())
}

fn archive_flake(flake: &str) -> Result<String> {
    let archived = Command::new("nix")
        .args(["flake", "archive", "--json", "--no-update-lock-file", flake])
        .output()?;
    if !archived.status.success() {
        return Err("could not snapshot the locked machine flake".into());
    }
    let archived: serde_json::Value = serde_json::from_slice(&archived.stdout)?;
    let snapshot =
        archived.get("path").and_then(|p| p.as_str()).ok_or("archive returned no store path")?;
    if !snapshot.starts_with("/nix/store/") {
        return Err("archive path is outside the Nix store".into());
    }
    Ok(format!("path:{snapshot}"))
}

fn verify_single_disk(flake: &str, disk: &str) -> Result<()> {
    let output = Command::new("nix")
        .args([
            "eval",
            "--no-update-lock-file",
            "--json",
            &format!("{flake}#nixosConfigurations.nox.config.disko.devices.disk"),
        ])
        .output()?;
    if !output.status.success() {
        return Err(
            format!("disk evaluation failed: {}", String::from_utf8_lossy(&output.stderr)).into()
        );
    }
    let disks: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let disks = disks.as_object().ok_or("invalid Disko disk declaration")?;
    if disks.len() != 1
        || disks.values().next().and_then(|d| d.get("device")).and_then(|v| v.as_str())
            != Some(disk)
    {
        return Err("installer currently accepts exactly one disk, matching --confirm-disk; review the Disko declaration".into());
    }
    Ok(())
}

fn stable_disk(disk: &str) -> bool {
    disk.starts_with("/dev/disk/by-id/")
        && disk.len() > "/dev/disk/by-id/".len()
        && !disk.contains("..")
        && !disk.contains("REPLACE")
        && !disk.bytes().any(|byte| byte.is_ascii_whitespace())
}

fn read_password() -> Result<String> {
    let mut password = String::new();
    std::io::stdin().read_to_string(&mut password)?;
    while password.ends_with('\n') || password.ends_with('\r') {
        password.pop();
    }
    if password.is_empty()
        || password.chars().any(|character| matches!(character, '\n' | '\r' | ':' | '\0'))
    {
        return Err(
            "stdin must contain one non-empty password without colon or embedded newline".into()
        );
    }
    Ok(password)
}

fn set_installed_password(root: &Path, username: &str, password: &str) -> Result<()> {
    let root = root.display().to_string();
    let mut child = Command::new("nixos-enter")
        .args(["--root", &root, "-c", "chpasswd"])
        .stdin(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("could not open nixos-enter stdin")?
        .write_all(format!("{username}:{password}\n").as_bytes())?;
    if !child.wait()?.success() {
        return Err("nixos-enter failed while setting the installed user password".into());
    }
    Ok(())
}

fn install_root() -> PathBuf {
    if cfg!(debug_assertions) {
        if let Some(path) = env::var_os("NOX_TEST_INSTALL_ROOT") {
            return PathBuf::from(path);
        }
    }
    PathBuf::from("/mnt")
}

pub fn generations() -> Result<()> {
    if !Path::new("/run/current-system").exists() {
        return Err("generations requires a running NixOS system".into());
    }
    invoke(
        "nix-env",
        &["--profile".into(), "/nix/var/nix/profiles/system".into(), "--list-generations".into()],
        true,
    )
}

pub fn write_new(path: &Path, text: &str) -> Result<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

pub fn machine_flake(source: &str, system: &str) -> Result<String> {
    if source.contains(['"', '\\', '\n', '\r', '$']) {
        return Err("invalid Nox source reference".into());
    }
    if !["x86_64-linux", "aarch64-linux"].contains(&system) {
        return Err("unsupported system".into());
    }
    Ok(format!(
        r#"{{
  inputs.nox.url = "{source}";
  outputs = {{ self, nox }}:
    let machine = nox.lib.mkSystem {{ configFile = ./nox.toml; system = "{system}"; }};
    in {{
      nixosConfigurations.nox = machine;
      packages.{system} = {{
        image = nox.lib.artifact machine;
        default = nox.lib.artifact machine;
        installer = nox.packages.{system}.installer;
        disko = nox.packages.{system}.disko;
      }};
    }};
}}
"#
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_cannot_inject_nix() {
        for source in ["x\"; builtins.abort", "${builtins.abort}", "x\ny", "a\\b"] {
            assert!(machine_flake(source, "x86_64-linux").is_err());
        }
        assert!(machine_flake("github:swarnimarun/nox", "x86_64-linux")
            .unwrap()
            .contains("./nox.toml"));
        assert!(machine_flake("github:swarnimarun/nox", "unknown").is_err());
    }
}
