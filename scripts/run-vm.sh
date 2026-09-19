#!/usr/bin/env bash
set -euo pipefail
# EFI image: firmware must match the guest architecture. Never attach a host disk.
if [[ $# != 2 ]]; then
  echo 'Usage: bash scripts/run-vm.sh IMAGE.qcow2 OVMF_CODE.fd' >&2
  exit 2
fi
[[ -f "$1" && -f "$2" ]] || { echo 'Image and EFI firmware must be regular files.' >&2; exit 1; }
[[ "$1" != *,* ]] || { echo "Image path must not contain commas." >&2; exit 1; }
# snapshot=on leaves the source image unchanged; TCG also works without /dev/kvm.
exec qemu-system-x86_64 -machine q35 -accel tcg -m 4096 -smp 2 \
  -bios "$2" -drive "file=$1,format=qcow2,if=virtio,snapshot=on" \
  -nic user,model=virtio-net-pci -nographic
