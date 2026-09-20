//! Domain logic that is independent from CLI presentation and system effects.

use nox_config::{Capability, DesktopFlavour, NoxConfig, Target};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub machine: String,
    pub profile: String,
    pub target: String,
    pub flavour: String,
    pub graphics: String,
    pub bootloader: String,
    pub filesystem: String,
    pub install_disk: Option<String>,
    pub capabilities: Vec<String>,
    pub actions: Vec<String>,
}
pub fn plan(config: &NoxConfig) -> Plan {
    let mut capabilities = config
        .capabilities
        .iter()
        .map(|capability| match capability {
            Capability::Desktop => "desktop",
            Capability::Gaming => "gaming",
            Capability::Storage => "storage",
            Capability::Shares => "shares",
            Capability::Apps => "apps",
            Capability::Virtualization => "virtualization",
            Capability::Development => "development",
            Capability::RemoteManagement => "remote-management",
            Capability::Recovery => "recovery",
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();
    capabilities.sort();

    let flavour = config.desktop_flavour();
    let mut actions = vec!["evaluate the locked NixOS module graph".to_owned()];
    if flavour != DesktopFlavour::None {
        actions.push(format!("prepare the {flavour} Wayland session"));
        actions.push(format!("configure {} graphics support", config.hardware.graphics));
    }
    actions.push(format!("prepare the {} target", config.target));
    actions.push(
        match config.target {
            Target::Metal => match &config.install.disk {
                Some(disk) => format!(
                    "after exact confirmation, partition and format {disk} as {} and install with {}",
                    config.install.filesystem, config.boot.loader
                ),
                None => "build the system; installation requires an explicit stable disk declaration"
                    .to_owned(),
            },
            Target::Iso => {
                "build EFI/USB installer media; boot and disk installation are separate operations"
                    .to_owned()
            }
            Target::Qcow2 => {
                "build an EFI qcow2 disk; test it in a disposable virtual machine".to_owned()
            }
            Target::Wsl => {
                "build the WSL tarball builder; packaging requires root before Windows import"
                    .to_owned()
            }
            Target::Oci => {
                "build an OCI userspace archive; this does not boot a kernel or systemd".to_owned()
            }
        },
    );

    Plan {
        machine: config.name.clone(),
        profile: config.profile.to_string(),
        target: config.target.to_string(),
        flavour: flavour.to_string(),
        graphics: config.hardware.graphics.to_string(),
        bootloader: config.boot.loader.to_string(),
        filesystem: config.install.filesystem.to_string(),
        install_disk: config.install.disk.clone(),
        capabilities,
        actions,
    }
}
impl Plan {
    pub fn render_text(&self) -> String {
        let capabilities = if self.capabilities.is_empty() {
            "none".to_owned()
        } else {
            self.capabilities.join(", ")
        };
        let actions = self
            .actions
            .iter()
            .map(|action| format!("  - {action}"))
            .collect::<Vec<_>>()
            .join("\n");
        let disk = self.install_disk.as_deref().unwrap_or("not declared");
        format!(
            "Machine\n  name: {}\n  profile: {}\n  target: {}\n  flavour: {}\n  graphics: {}\n  bootloader: {}\n  filesystem: {}\n  install disk: {}\n  capabilities: {}\n\nActions\n{}\n",
            self.machine,
            self.profile,
            self.target,
            self.flavour,
            self.graphics,
            self.bootloader,
            self.filesystem,
            disk,
            capabilities,
            actions
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nox_config::{example_config, Capability};

    #[test]
    fn plan_explains_wsl_packaging() {
        let mut config = example_config();
        config.target = Target::Wsl;
        assert!(plan(&config).render_text().contains("packaging requires root"));
    }

    #[test]
    fn plan_contains_sorted_capabilities() {
        let mut config = example_config();
        config.capabilities.insert(Capability::Storage);
        let result = plan(&config);
        assert_eq!(result.capabilities, vec!["apps", "remote-management", "storage"]);
    }

    #[test]
    fn plan_names_destructive_disk_action() {
        let mut config = example_config();
        config.install.disk = Some("/dev/disk/by-id/virtio-test".to_owned());
        let text = plan(&config).render_text();
        assert!(text.contains("/dev/disk/by-id/virtio-test"));
        assert!(text.contains("exact confirmation"));
    }
}
