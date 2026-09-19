{ pkgs, ... }:
{
  name = "nox-server-boot";
  nodes.machine = { ... }: {
    imports = [
      ../../nix/profiles/server.nix
      ../../nix/capabilities
    ];
    nox.capabilities = [
      "development"
      "apps"
    ];
  };
  testScript = ''
    machine.start()
    machine.wait_for_unit("multi-user.target")
    machine.wait_for_unit("sshd.service")
    machine.succeed("git --version")
    machine.succeed("podman --version")
    machine.succeed("test -L /run/current-system")
  '';
}
