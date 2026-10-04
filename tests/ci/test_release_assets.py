import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("release_assets", "scripts/prepare-release-assets.py")
assets = importlib.util.module_from_spec(spec)
spec.loader.exec_module(assets)


class ReleaseAssets(unittest.TestCase):
    def fixture(self, directory, data):
        image = Path(directory) / "image.iso"
        image.write_bytes(data)
        image.with_name(image.name + ".sha256").write_text(hashlib.sha256(data).hexdigest() + "  image.iso\n")
        return image

    def test_small_image_remains_direct_asset(self):
        with tempfile.TemporaryDirectory() as directory:
            image = self.fixture(directory, b"small")
            assets.prepare(image, asset_limit=10, part_size=6, upload=lambda _: self.fail("unexpected split"))
            self.assertTrue(image.exists())

    def test_parts_reassemble_exact_image_and_have_valid_hashes(self):
        with tempfile.TemporaryDirectory() as directory:
            data = b"abcdefghijklmnopqrst"
            image = self.fixture(directory, data)
            uploaded = {}
            assets.prepare(image, asset_limit=10, part_size=6, upload=lambda path: uploaded.update({path.name: path.read_bytes()}))
            manifest = json.loads(image.with_name(image.name + ".parts.json").read_text())
            self.assertEqual(b"".join(uploaded[part["name"]] for part in manifest["parts"]), data)
            for part in manifest["parts"]:
                self.assertLess(part["size"], 10)
                self.assertEqual(hashlib.sha256(uploaded[part["name"]]).hexdigest(), part["sha256"])
            self.assertFalse(image.exists())
            self.assertFalse(list(Path(directory).glob("*.part-*")))

    def test_failed_upload_preserves_original_and_does_not_write_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            image = self.fixture(directory, b"abcdefghijklmnopqrst")
            def fail(_):
                raise RuntimeError("upload failed")
            with self.assertRaises(RuntimeError):
                assets.prepare(image, asset_limit=10, part_size=6, upload=fail)
            self.assertTrue(image.exists())
            self.assertFalse(image.with_name(image.name + ".parts.json").exists())

    def test_changed_image_refuses_complete_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            image = self.fixture(directory, b"abcdefghijklmnopqrst")
            image.with_name(image.name + ".sha256").write_text("wrong  image.iso\n")
            with self.assertRaises(ValueError):
                assets.prepare(image, asset_limit=10, part_size=6, upload=lambda _: None)
            self.assertTrue(image.exists())
            self.assertFalse(image.with_name(image.name + ".parts.json").exists())


if __name__ == "__main__":
    unittest.main()
