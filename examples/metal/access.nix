{ ... }:
{
  # Add your public SSH key. Password and keyboard-interactive SSH are disabled.
  users.users.root.openssh.authorizedKeys.keys = [ ];
  assertions = [{ assertion = false; message = "Configure SSH access and remove this template assertion"; }];
}
