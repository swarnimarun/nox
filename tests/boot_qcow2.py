"""Boot the actual x86_64 EFI qcow2 artifact and require its serial login prompt."""
import argparse
from pathlib import Path
import selectors
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument('image', type=Path)
parser.add_argument('firmware', type=Path)
parser.add_argument('--timeout', type=int, default=300)
parser.add_argument('--log', type=Path, default=Path('boot-console.log'))
args = parser.parse_args()
if not args.image.is_file() or not args.firmware.is_file() or ',' in str(args.image):
    parser.error('image and firmware must be files; image path cannot contain commas')
command = ['qemu-system-x86_64', '-machine', 'q35', '-accel', 'tcg', '-m', '2048', '-smp', '2',
           '-bios', str(args.firmware), '-drive', f'file={args.image},format=qcow2,if=virtio,snapshot=on',
           '-nic', 'none', '-nographic', '-monitor', 'none']
process = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
selector = selectors.DefaultSelector()
selector.register(process.stdout, selectors.EVENT_READ)
deadline = time.monotonic() + args.timeout
seen = b''
success = False
try:
    with args.log.open('wb') as log:
        while time.monotonic() < deadline:
            ready = selector.select(timeout=1)
            if ready:
                chunk = process.stdout.read1(8192)
                if not chunk:
                    break
                log.write(chunk)
                log.flush()
                seen = (seen + chunk)[-65536:]
                if b'nox-vm login:' in seen:
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
    raise SystemExit(f'Exact image did not reach nox-vm login; inspect {args.log}')
print(f'Exact qcow2 artifact reached the login prompt; console: {args.log}')
