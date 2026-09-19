//! Versioned, user-editable configuration for Nox.
//!
//! This crate deliberately owns TOML only. It does not parse or rewrite Nix
//! expressions; advanced Nix remains an explicit user-owned escape hatch.

use std::{collections::BTreeSet, fmt, fs, path::Path, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    Server,
    Desktop,
    Gaming,
    Workspace,
    Recovery,
}

impl Profile {
    pub const ALL: [&'static str; 5] = ["server", "desktop", "gaming", "workspace", "recovery"];
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Server => "server",
            Self::Desktop => "desktop",
            Self::Gaming => "gaming",
            Self::Workspace => "workspace",
            Self::Recovery => "recovery",
        })
    }
}

impl FromStr for Profile {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "server" => Ok(Self::Server),
            "desktop" => Ok(Self::Desktop),
            "gaming" => Ok(Self::Gaming),
            "workspace" => Ok(Self::Workspace),
            "recovery" => Ok(Self::Recovery),
            other => Err(ConfigError::InvalidValue {
                field: "profile",
                value: other.to_owned(),
                expected: Profile::ALL.join(", "),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    Metal,
    Iso,
    Qcow2,
    Wsl,
    Oci,
}

impl Target {
    pub const ALL: [&'static str; 5] = ["metal", "iso", "qcow2", "wsl", "oci"];
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Metal => "metal",
            Self::Iso => "iso",
            Self::Qcow2 => "qcow2",
            Self::Wsl => "wsl",
            Self::Oci => "oci",
        })
    }
}

impl FromStr for Target {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "metal" => Ok(Self::Metal),
            "iso" => Ok(Self::Iso),
            "qcow2" => Ok(Self::Qcow2),
            "wsl" => Ok(Self::Wsl),
            "oci" => Ok(Self::Oci),
            other => Err(ConfigError::InvalidValue {
                field: "target",
                value: other.to_owned(),
                expected: Target::ALL.join(", "),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    Desktop,
    Gaming,
    Storage,
    Shares,
    Apps,
    Virtualization,
    Development,
    RemoteManagement,
    Recovery,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NixSettings {
    #[serde(default = "default_nixpkgs_channel")]
    pub channel: String,
    #[serde(default)]
    pub extra_modules: Vec<String>,
}

impl Default for NixSettings {
    fn default() -> Self {
        Self { channel: default_nixpkgs_channel(), extra_modules: Vec::new() }
    }
}

fn default_nixpkgs_channel() -> String {
    "nixos-26.05".to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NoxConfig {
    pub schema_version: u32,
    pub name: String,
    pub profile: Profile,
    pub target: Target,
    #[serde(default)]
    pub capabilities: BTreeSet<Capability>,
    #[serde(default)]
    pub nix: NixSettings,
}

impl NoxConfig {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            errors.push(format!(
                "schema_version {} is unsupported; expected {}",
                self.schema_version, CURRENT_SCHEMA_VERSION
            ));
        }
        if self.name.trim().is_empty() {
            errors.push("name must not be empty".to_owned());
        }
        if self.name.len() > 63
            || !self.name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || self.name.starts_with('-')
            || self.name.ends_with('-')
        {
            errors.push(
                "name must be a hostname: 1-63 ASCII letters/digits/hyphens, no edge hyphens"
                    .to_owned(),
            );
        }
        if self.nix.channel != "nixos-26.05" {
            errors.push(
                "only nixos-26.05 is supported; the flake lock selects its revision".to_owned(),
            );
        }
        for module in &self.nix.extra_modules {
            let path = Path::new(module);
            if module.is_empty()
                || path.is_absolute()
                || !module.ends_with(".nix")
                || path.components().any(|c| !matches!(c, std::path::Component::Normal(_)))
            {
                errors.push(format!("extra module must be a relative .nix path inside the machine directory: {module}"));
            }
        }
        if self.target == Target::Oci && self.profile != Profile::Workspace {
            errors.push("oci target requires the workspace profile".to_owned());
        }
        if self.profile == Profile::Gaming && !self.capabilities.contains(&Capability::Gaming) {
            errors.push("gaming profile requires the gaming capability".to_owned());
        }
        if self.profile == Profile::Recovery && !self.capabilities.contains(&Capability::Recovery) {
            errors.push("recovery profile requires the recovery capability".to_owned());
        }
        if self.target == Target::Wsl && self.profile != Profile::Workspace {
            errors.push("wsl target currently requires the workspace profile".to_owned());
        }
        if self.target == Target::Oci && !self.capabilities.contains(&Capability::Development) {
            errors.push("oci target requires the development capability".to_owned());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not read {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid {field} value {value:?}; expected one of: {expected}")]
    InvalidValue { field: &'static str, value: String, expected: String },
    #[error("configuration is invalid:\n{0}")]
    Invalid(String),
}

pub fn load(path: impl AsRef<Path>) -> Result<NoxConfig, ConfigError> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .map_err(|source| ConfigError::Read { path: path.display().to_string(), source })?;
    parse(&contents)
}

pub fn parse(contents: &str) -> Result<NoxConfig, ConfigError> {
    let config: NoxConfig = toml::from_str(contents)?;
    if let Err(errors) = config.validate() {
        return Err(ConfigError::Invalid(errors.join("\n")));
    }
    Ok(config)
}

pub fn example_config() -> NoxConfig {
    NoxConfig {
        schema_version: CURRENT_SCHEMA_VERSION,
        name: "nox-machine".to_owned(),
        profile: Profile::Server,
        target: Target::Metal,
        capabilities: [Capability::Apps, Capability::RemoteManagement].into_iter().collect(),
        nix: NixSettings::default(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_validates_example() {
        let text = toml::to_string(&example_config()).expect("example serializes");
        assert_eq!(parse(&text).expect("example parses").name, "nox-machine");
    }

    #[test]
    fn rejects_invalid_hostname_and_unknown_fields() {
        for name in ["bad name", "-bad", "bad-", "a.b", "$(id)", ""] {
            let mut c = example_config();
            c.name = name.to_owned();
            assert!(c.validate().is_err(), "{name}");
        }
        let text = format!("unexpected = true\n{}", toml::to_string(&example_config()).unwrap());
        assert!(parse(&text).is_err());
    }

    #[test]
    fn rejects_module_escape_and_unsupported_channel() {
        for module in ["../secret.nix", "/tmp/a.nix", "", "a.txt"] {
            let mut c = example_config();
            c.nix.extra_modules.push(module.to_owned());
            assert!(c.validate().is_err());
        }
        let mut c = example_config();
        c.nix.channel = "unstable".to_owned();
        assert!(c.validate().is_err());
    }

    #[test]
    fn rejects_incompatible_target() {
        let config = NoxConfig {
            schema_version: CURRENT_SCHEMA_VERSION,
            name: "bad".to_owned(),
            profile: Profile::Server,
            target: Target::Wsl,
            capabilities: BTreeSet::new(),
            nix: NixSettings::default(),
        };
        assert!(config.validate().is_err());
    }
}
