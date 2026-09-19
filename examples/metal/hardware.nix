{ ... }:
{
  # Replace with reviewed hardware configuration from the destination machine.
  assertions = [{ assertion = false; message = "Replace the metal hardware template before installation"; }];
}
