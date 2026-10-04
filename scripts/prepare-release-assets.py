"""Upload oversized images in ordered, checksummed parts below GitHub's limit."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ASSET_LIMIT = 2 * 1024**3
PART_SIZE = 1024**3


def prepare(image: Path, *, asset_limit=ASSET_LIMIT, part_size=PART_SIZE, upload=None):
    if image.stat().st_size < asset_limit:
        return
    if not 0 < part_size < asset_limit:
        raise ValueError("part_size must be positive and below asset_limit")
    if upload is None:
        tag = os.environ["RELEASE_TAG"]

        def upload(path):
            subprocess.run(["gh", "release", "upload", tag, str(path)], check=True)

    parts = []
    image_hash = hashlib.sha256()
    with image.open("rb") as source:
        while True:
            chunk = source.read(min(part_size, 8 * 1024**2))
            if not chunk:
                break
            part = image.with_name(f"{image.name}.part-{len(parts):03d}")
            digest = hashlib.sha256()
            size = 0
            with part.open("wb") as destination:
                while chunk:
                    destination.write(chunk)
                    digest.update(chunk)
                    image_hash.update(chunk)
                    size += len(chunk)
                    if size == part_size:
                        break
                    chunk = source.read(min(part_size - size, 8 * 1024**2))
            upload(part)
            parts.append({"name": part.name, "size": size, "sha256": digest.hexdigest()})
            part.unlink()
    expected = image.with_name(image.name + ".sha256").read_text().split()[0]
    if image_hash.hexdigest() != expected:
        raise ValueError("Image changed after validation; refusing publication")
    manifest = {"image": image.name, "sha256": expected, "parts": parts}
    image.with_name(image.name + ".parts.json").write_text(json.dumps(manifest, indent=2) + "\n")
    image.with_name(image.name + ".parts.sha256").write_text(
        "".join(f"{part['sha256']}  {part['name']}\n" for part in parts)
    )
    image.unlink()


if __name__ == "__main__":
    prepare(Path(sys.argv[1]))
