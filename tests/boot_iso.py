"""Boot an exact Nox ISO with UEFI and require its live-system readiness marker."""

import argparse
from pathlib import Path
import selectors
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument("image", type=Path)
parser.add_argument("firmware", type=Path)
parser.add_argument("--expect-flavour", choices=["hyprland", "niri"], required=True)
parser.add_argument("--timeout", type=int, default=600)
parser.add_argument("--log", type=Path, default=Path("iso-boot-console.log"))
args = parser.parse_args()
if not args.image.is_file() or not args.firmware.is_file() or "," in str(args.image):
    parser.error("image and firmware must be files; image path cannot contain commas")

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
    "-bios",
    str(args.firmware),
    "-boot",
    "d",
    "-cdrom",
    str(args.image),
    "-nic",
    "user,model=virtio-net-pci",
    "-nographic",
    "-monitor",
    "none",
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

if not success:
    raise SystemExit(f"Exact ISO did not emit {marker.decode()!r}; inspect {args.log}")
print(f"Exact {args.expect_flavour} ISO reached graphical target; console: {args.log}")
