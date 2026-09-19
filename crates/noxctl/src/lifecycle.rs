use std::{error::Error, fs, path::{Path, PathBuf}, process::Command};
use nox_config::{load, Target};

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

pub fn reference(root: &Path) -> String { format!("path:{}", root.display()) }

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
    let args = vec!["build".into(), "--no-update-lock-file".into(),
        format!("{}#image", reference(&root)), "--out-link".into(), out_link.display().to_string()];
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

pub fn install(config: &Path, host: &str, confirm_host: Option<&str>, confirm_disk: Option<&str>, execute: bool) -> Result<()> {
    let c = load(config)?;
    if c.target != Target::Metal { return Err("install requires target = metal".into()); }
    // Restrict to unambiguous SSH destinations; never pass user text through a shell.
    if host.is_empty() || host.starts_with('-') || !host.bytes().all(|b| b.is_ascii_alphanumeric() || b"@.-_:[]".contains(&b)) {
        return Err("invalid SSH destination".into());
    }
    let root = project(config)?;
    require_lock(&root)?;
    let flake = reference(&root);
    println!("DESTRUCTIVE: nixos-anywhere will replace the selected destination's operating system and format its declared disks. Destination: {host}");
    let mut args = vec!["run".into(), "--no-update-lock-file".into(), format!("{flake}#installer"), "--".into(),
        "--flake".into(), format!("{flake}#nox"), "--target-host".into(), host.into()];
    if !execute { return invoke("nix", &args, false); }
    if confirm_host != Some(host) { return Err("--confirm-host must exactly match --host".into()); }
    let disk = confirm_disk.ok_or("--confirm-disk must match the single Disko disk device")?;
    if !disk.starts_with("/dev/") || disk.contains("REPLACE") { return Err("invalid confirmation disk".into()); }
    // Freeze the project and all flake inputs before checking disks or installing.
    let archived = Command::new("nix").args(["flake", "archive", "--json", "--no-update-lock-file", &flake]).output()?;
    if !archived.status.success() { return Err("could not snapshot the locked machine flake".into()); }
    let archived: serde_json::Value = serde_json::from_slice(&archived.stdout)?;
    let snapshot = archived.get("path").and_then(|p| p.as_str()).ok_or("archive returned no store path")?;
    if !snapshot.starts_with("/nix/store/") { return Err("archive path is outside the Nix store".into()); }
    let flake = format!("path:{snapshot}");
    args[2] = format!("{flake}#installer");
    args[5] = format!("{flake}#nox");
    let output = Command::new("nix").args(["eval", "--no-update-lock-file", "--json", &format!("{flake}#nixosConfigurations.nox.config.disko.devices.disk")]).output()?;
    if !output.status.success() { return Err(format!("disk evaluation failed: {}", String::from_utf8_lossy(&output.stderr)).into()); }
    let disks: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let disks = disks.as_object().ok_or("invalid Disko disk declaration")?;
    if disks.len() != 1 || disks.values().next().and_then(|d| d.get("device")).and_then(|v| v.as_str()) != Some(disk) {
        return Err("installer currently accepts exactly one disk, matching --confirm-disk; review the Disko declaration".into());
    }
    invoke("nix", &["build".into(), "--no-update-lock-file".into(), "--no-link".into(), format!("{flake}#nixosConfigurations.nox.config.system.build.toplevel")], true)?;
    invoke("nix", &args, true)
}

pub fn generations() -> Result<()> {
    if !Path::new("/run/current-system").exists() { return Err("generations requires a running NixOS system".into()); }
    invoke("nix-env", &["--profile".into(), "/nix/var/nix/profiles/system".into(), "--list-generations".into()], true)
}

pub fn write_new(path: &Path, text: &str) -> Result<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

pub fn machine_flake(source: &str, system: &str) -> Result<String> {
    if source.contains(['"', '\\', '\n', '\r', '$']) { return Err("invalid Nox source reference".into()); }
    if !["x86_64-linux", "aarch64-linux"].contains(&system) { return Err("unsupported system".into()); }
    Ok(format!(r#"{{
  inputs.nox.url = "{source}";
  outputs = {{ self, nox }}:
    let machine = nox.lib.mkSystem {{ configFile = ./nox.toml; system = "{system}"; }};
    in {{
      nixosConfigurations.nox = machine;
      packages.{system} = {{
        image = nox.lib.artifact machine;
        default = nox.lib.artifact machine;
        installer = nox.packages.{system}.installer;
      }};
    }};
}}
"#))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_cannot_inject_nix() {
        for source in ["x\"; builtins.abort", "${builtins.abort}", "x\ny", "a\\b"] {
            assert!(machine_flake(source, "x86_64-linux").is_err());
        }
        assert!(machine_flake("github:swarnimarun/nox", "x86_64-linux").unwrap().contains("./nox.toml"));
        assert!(machine_flake("github:swarnimarun/nox", "unknown").is_err());
    }
}
