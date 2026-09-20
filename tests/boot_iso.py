"""Boot an exact Nox ISO with UEFI and require its live-system readiness marker."""

import argparse
from pathlib import Path
import selectors
import shutil
import subprocess
import tempfile
import time

parser = argparse.ArgumentParser()
parser.add_argument("image", type=Path)
parser.add_argument("firmware", type=Path)
parser.add_argument("--expect-flavour", choices=["hyprland", "niri"], required=True)
parser.add_argument("--timeout", type=int, default=600)
parser.add_argument("--log", type=Path, default=Path("iso-boot-console.log"))
args = parser.parse_args()
variables = args.firmware.with_name("OVMF_VARS.fd")
if (
    not args.image.is_file()
    or not args.firmware.is_file()
    or not variables.is_file()
    or "," in str(args.image)
    or "," in str(args.firmware)
):
    parser.error("image, OVMF code, and adjacent OVMF variables must be files without commas")
firmware_state = tempfile.TemporaryDirectory(prefix="nox-ovmf-")
writable_variables = Path(firmware_state.name) / "OVMF_VARS.fd"
shutil.copy2(variables, writable_variables)

marker = f"NOX_LIVE_READY flavour={args.expect_flavour}".encode()
command = [
    "qemu-system-x86_64",
    "-machine",
    "q35",
    "-accel",
    "tcg",
    "-m",
    "4096",
    "-smp",
    "2",
    "-drive",
    f"if=pflash,format=raw,readonly=on,file={args.firmware}",
    "-drive",
    f"if=pflash,format=raw,file={writable_variables}",
    "-boot",
    "d",
    "-cdrom",
    str(args.image),
    "-nic",
    "user,model=virtio-net-pci",
    "-display",
    "none",
    "-serial",
    "stdio",
    "-monitor",
    "none",
    "-vga",
    "virtio",
]
process = subprocess.Popen(
    command,
    stdin=subprocess.DEVNULL,
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
)
selector = selectors.DefaultSelector()
selector.register(process.stdout, selectors.EVENT_READ)
deadline = time.monotonic() + args.timeout
seen = b""
success = False
try:
    with args.log.open("wb") as log:
        while time.monotonic() < deadline:
            ready = selector.select(timeout=1)
            if ready:
                chunk = process.stdout.read1(8192)
                if not chunk:
                    break
                log.write(chunk)
                log.flush()
                seen = (seen + chunk)[-65536:]
                if marker in seen:
                    success = True
                    break
            elif process.poll() is not None:
                break
finally:
    selector.close()
    process.terminate()
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()
    firmware_state.cleanup()

if not success:
    console = args.log.read_text(errors="replace")[-65536:]
    raise SystemExit(
        f"Exact ISO did not emit {marker.decode()!r}; console tail follows:\n{console}"
    )
print(f"Exact {args.expect_flavour} ISO reached graphical target; console: {args.log}")
