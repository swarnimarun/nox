#!/usr/bin/env python3
"""GTK4 installer for a local Nox system.

The UI only gathers typed settings. noxctl owns project generation,
validation, immutable preflight, disk confirmation, and installation.
"""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import GLib, Gtk  # noqa: E402


def command(*args: str, input_text: str | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(args),
        check=True,
        input=input_text,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        env=os.environ,
    )


def stable_disks() -> list[tuple[str, str]]:
    """Return unmounted whole disks keyed by stable /dev/disk/by-id paths."""
    try:
        result = command(
            "lsblk",
            "--json",
            "--bytes",
            "--output",
            "PATH,SIZE,TYPE,MOUNTPOINTS,MODEL,SERIAL,WWN",
        )
        devices = json.loads(result.stdout).get("blockdevices", [])
    except (FileNotFoundError, subprocess.CalledProcessError, json.JSONDecodeError):
        return []

    eligible: dict[Path, dict[str, object]] = {}
    for device in devices:
        path = Path(str(device.get("path", "")))
        descendants = [device, *device.get("children", [])]
        mounted = any(any(item.get("mountpoints") or []) for item in descendants)
        if device.get("type") == "disk" and path.is_absolute() and not mounted:
            try:
                eligible[path.resolve()] = device
            except OSError:
                continue

    choices: list[tuple[str, str]] = []
    by_id = Path("/dev/disk/by-id")
    if not by_id.is_dir():
        return choices
    for link in sorted(by_id.iterdir()):
        if "-part" in link.name:
            continue
        try:
            device = eligible.get(link.resolve(strict=True))
        except OSError:
            continue
        if device is None:
            continue
        size = int(device.get("size") or 0) / (1024**3)
        model = str(device.get("model") or "").strip()
        label = f"{link} — {size:.1f} GiB"
        if model:
            label += f" — {model}"
        choices.append((str(link), label))
    return choices


class InstallerWindow(Gtk.ApplicationWindow):
    def __init__(self, application: Gtk.Application, dry_run: bool):
        super().__init__(application=application, title="Install Nox")
        self.dry_run = dry_run
        self.set_default_size(720, 760)
        self.project_parent = Path(tempfile.mkdtemp(prefix="nox-installer-"))
        self.project = self.project_parent / "machine"
        self.disks = stable_disks()
        if dry_run and not self.disks:
            self.disks = [
                (
                    "/dev/disk/by-id/nox-dry-run",
                    "/dev/disk/by-id/nox-dry-run — dry-run placeholder",
                )
            ]

        outer = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        outer.set_margin_top(24)
        outer.set_margin_bottom(24)
        outer.set_margin_start(32)
        outer.set_margin_end(32)

        title = Gtk.Label()
        title.set_markup("<span size='xx-large' weight='bold'>Install Nox</span>")
        title.set_xalign(0)
        outer.append(title)
        subtitle = Gtk.Label(
            label=(
                "Create a locked Niri or Hyprland system, review its plan, "
                "then explicitly confirm the disk that will be erased."
            )
        )
        subtitle.set_wrap(True)
        subtitle.set_xalign(0)
        outer.append(subtitle)

        grid = Gtk.Grid(column_spacing=18, row_spacing=12)
        self.hostname = self._entry("nox", 0, "Hostname", grid)
        self.username = self._entry("nox", 1, "User", grid)
        self.password = self._entry("", 2, "Password", grid, secret=True)
        self.password_again = self._entry("", 3, "Confirm password", grid, secret=True)
        self.flavour = self._dropdown(["niri", "hyprland"], 4, "Wayland flavour", grid)
        self.graphics = self._dropdown(
            ["auto", "amd", "intel", "nvidia-open", "nvidia-proprietary", "vm"],
            5,
            "Graphics",
            grid,
        )
        self.bootloader = self._dropdown(
            ["systemd-boot", "grub-efi"], 6, "Bootloader", grid
        )
        self.filesystem = self._dropdown(["btrfs", "ext4"], 7, "Filesystem", grid)
        self.locale = self._entry("en_US.UTF-8", 8, "Locale", grid)
        self.timezone = self._entry("UTC", 9, "Timezone", grid)
        self.keymap = self._entry("us", 10, "Keyboard layout", grid)
        disk_labels = [label for _, label in self.disks] or [
            "No unmounted /dev/disk/by-id device found"
        ]
        self.disk = self._dropdown(disk_labels, 11, "Destination disk", grid)
        outer.append(grid)

        warning = Gtk.Label(
            label=(
                "The selected disk will be repartitioned and all existing data on it "
                "will be destroyed. Use only a disposable or fully backed-up disk."
            )
        )
        warning.add_css_class("error")
        warning.set_wrap(True)
        warning.set_xalign(0)
        outer.append(warning)

        self.status = Gtk.Label(label="Ready")
        self.status.set_wrap(True)
        self.status.set_xalign(0)
        outer.append(self.status)

        self.review = Gtk.Button(label="Generate and review plan")
        self.review.add_css_class("suggested-action")
        self.review.set_sensitive(bool(self.disks))
        self.review.connect("clicked", self.on_review)
        outer.append(self.review)

        scroller = Gtk.ScrolledWindow()
        scroller.set_child(outer)
        self.set_child(scroller)

    @staticmethod
    def _label(text: str) -> Gtk.Label:
        label = Gtk.Label(label=text)
        label.set_xalign(1)
        return label

    def _entry(
        self,
        default: str,
        row: int,
        label: str,
        grid: Gtk.Grid,
        *,
        secret: bool = False,
    ) -> Gtk.Entry:
        entry = Gtk.Entry()
        entry.set_text(default)
        entry.set_hexpand(True)
        if secret:
            entry.set_visibility(False)
            entry.set_input_purpose(Gtk.InputPurpose.PASSWORD)
        grid.attach(self._label(label), 0, row, 1, 1)
        grid.attach(entry, 1, row, 1, 1)
        return entry

    def _dropdown(
        self, values: list[str], row: int, label: str, grid: Gtk.Grid
    ) -> Gtk.DropDown:
        dropdown = Gtk.DropDown.new_from_strings(values)
        dropdown.set_hexpand(True)
        grid.attach(self._label(label), 0, row, 1, 1)
        grid.attach(dropdown, 1, row, 1, 1)
        return dropdown

    @staticmethod
    def selected(dropdown: Gtk.DropDown) -> str:
        item = dropdown.get_selected_item()
        return item.get_string() if item is not None else ""

    def values(self) -> dict[str, str]:
        selected_disk = self.disk.get_selected()
        disk = self.disks[selected_disk][0] if selected_disk < len(self.disks) else ""
        return {
            "hostname": self.hostname.get_text(),
            "username": self.username.get_text(),
            "password": self.password.get_text(),
            "password_again": self.password_again.get_text(),
            "flavour": self.selected(self.flavour),
            "graphics": self.selected(self.graphics),
            "bootloader": self.selected(self.bootloader),
            "filesystem": self.selected(self.filesystem),
            "locale": self.locale.get_text(),
            "timezone": self.timezone.get_text(),
            "keymap": self.keymap.get_text(),
            "disk": disk,
        }

    def on_review(self, _button: Gtk.Button) -> None:
        values = self.values()
        if values["password"] != values["password_again"]:
            self.show_error("Passwords do not match.")
            return
        if not values["password"]:
            self.show_error("Enter a password for the installed user.")
            return
        if not values["disk"].startswith("/dev/disk/by-id/"):
            self.show_error("Select a stable /dev/disk/by-id destination.")
            return
        self.review.set_sensitive(False)
        self.status.set_text("Generating and validating the locked machine project…")
        threading.Thread(target=self.prepare, args=(values,), daemon=True).start()

    def prepare(self, values: dict[str, str]) -> None:
        try:
            if self.project.exists():
                shutil.rmtree(self.project)
            source = os.environ.get("NOX_SOURCE", "github:swarnimarun/nox")
            args = [
                "noxctl",
                "init",
                str(self.project),
                "--source",
                source,
                "--profile",
                "gaming",
                "--target",
                "metal",
                "--flavour",
                values["flavour"],
                "--graphics",
                values["graphics"],
                "--bootloader",
                values["bootloader"],
                "--filesystem",
                values["filesystem"],
                "--disk",
                values["disk"],
                "--username",
                values["username"],
                "--locale",
                values["locale"],
                "--timezone",
                values["timezone"],
                "--keymap",
                values["keymap"],
            ]
            command(*args)
            config = str(self.project / "nox.toml")
            command("noxctl", "validate", "--config", config)
            plan = command("noxctl", "plan", "--config", config).stdout
            if not self.dry_run:
                command("noxctl", "lock", "--config", config)
            GLib.idle_add(self.confirm_plan, values, plan)
        except (OSError, subprocess.CalledProcessError) as error:
            output = getattr(error, "stdout", None) or str(error)
            GLib.idle_add(self.show_error, output)

    def confirm_plan(self, values: dict[str, str], plan: str) -> bool:
        if self.dry_run:
            self.status.set_text("Dry run complete: project generated and validated; no disk changed.")
            self.review.set_sensitive(True)
            self.show_message("Dry run complete", plan)
            return False

        text = (
            f"{plan}\nTHIS WILL ERASE {values['disk']}\n\n"
            "Continue only if the stable disk identifier above is correct."
        )
        dialog = Gtk.MessageDialog(
            transient_for=self,
            modal=True,
            message_type=Gtk.MessageType.WARNING,
            buttons=Gtk.ButtonsType.NONE,
            text="Confirm destructive installation",
        )
        dialog.format_secondary_text(text)
        dialog.add_button("Cancel", Gtk.ResponseType.CANCEL)
        dialog.add_button("Erase disk and install", Gtk.ResponseType.ACCEPT)
        dialog.connect("response", self.on_confirmation, values)
        dialog.present()
        return False

    def on_confirmation(
        self,
        dialog: Gtk.MessageDialog,
        response: Gtk.ResponseType,
        values: dict[str, str],
    ) -> None:
        dialog.destroy()
        if response != Gtk.ResponseType.ACCEPT:
            self.status.set_text("Installation cancelled; no disk command was run.")
            self.review.set_sensitive(True)
            return
        self.status.set_text("Building, partitioning, and installing. Do not power off this machine…")
        threading.Thread(target=self.install, args=(values,), daemon=True).start()

    def install(self, values: dict[str, str]) -> None:
        try:
            output = command(
                "sudo",
                "--",
                "noxctl",
                "installer",
                "local",
                "--config",
                str(self.project / "nox.toml"),
                "--execute",
                "--confirm-disk",
                values["disk"],
                input_text=values["password"] + "\n",
            ).stdout
            GLib.idle_add(self.install_complete, output)
        except (OSError, subprocess.CalledProcessError) as error:
            output = getattr(error, "stdout", None) or str(error)
            GLib.idle_add(self.show_error, output)

    def install_complete(self, output: str) -> bool:
        self.status.set_text("Installation complete. Review the log, then reboot.")
        self.review.set_sensitive(True)
        self.show_message("Nox installed", output)
        return False

    def show_error(self, detail: str) -> bool:
        self.status.set_text("Stopped before completing the requested operation.")
        self.review.set_sensitive(bool(self.disks))
        self.show_message("Installer stopped", detail, Gtk.MessageType.ERROR)
        return False

    def show_message(
        self,
        title: str,
        detail: str,
        kind: Gtk.MessageType = Gtk.MessageType.INFO,
    ) -> None:
        dialog = Gtk.MessageDialog(
            transient_for=self,
            modal=True,
            message_type=kind,
            buttons=Gtk.ButtonsType.CLOSE,
            text=title,
        )
        dialog.format_secondary_text(detail[-8000:])
        dialog.connect("response", lambda window, _response: window.destroy())
        dialog.present()


class InstallerApplication(Gtk.Application):
    def __init__(self, dry_run: bool):
        super().__init__(application_id="dev.nox.Installer")
        self.dry_run = dry_run

    def do_activate(self) -> None:
        window = self.props.active_window
        if window is None:
            window = InstallerWindow(self, self.dry_run)
        window.present()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    return InstallerApplication(args.dry_run).run([])


if __name__ == "__main__":
    raise SystemExit(main())
