//! Domain logic that is independent from CLI presentation and system effects.

use nox_config::{Capability, NoxConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub machine: String,
    pub profile: String,
    pub target: String,
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

    Plan {
        machine: config.name.clone(),
        profile: config.profile.to_string(),
        target: config.target.to_string(),
        capabilities,
        actions: vec![
            "evaluate the NixOS module graph".to_owned(),
            format!("prepare the {} target", config.target),
            "show the resulting activation diff before applying changes".to_owned(),
        ],
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
        format!(
            "Machine\n  name: {}\n  profile: {}\n  target: {}\n  capabilities: {}\n\nActions\n{}\n",
            self.machine, self.profile, self.target, capabilities, actions
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nox_config::{example_config, Capability};

    #[test]
    fn plan_contains_sorted_capabilities() {
        let mut config = example_config();
        config.capabilities.insert(Capability::Storage);
        let result = plan(&config);
        assert_eq!(result.capabilities, vec!["apps", "remote-management", "storage"]);
    }
}
