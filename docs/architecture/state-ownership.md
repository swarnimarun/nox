# State ownership

| State | Canonical owner | Examples |
| --- | --- | --- |
| Desired configuration | Nix + versioned TOML | profile, target, services, storage declarations |
| Generated plan | `nox-core` | activation actions and diffs |
| Runtime observation | future `noxd` | VM health, disk health, operation progress |
| Secrets | sops-nix or an external secret manager | keys, credentials, tokens |

`noxctl` can edit the stable TOML document. It must never parse and rewrite
arbitrary user Nix, and `noxd` must never turn runtime observations into
canonical configuration without an explicit user action.

