#!/usr/bin/env python3
"""Adds or removes a non-Steam shortcut in a Steam installation that is NOT running, e.g. the Steam
Deck disk mounted in a PC (scripts/steamos-disk-test.sh).

  steam-shortcuts.py add    STEAM_ROOT NAME EXECUTABLE [--start-directory DIR] [--launch-options TEXT]
  steam-shortcuts.py remove STEAM_ROOT NAME

STEAM_ROOT is the directory that holds `userdata/` and `config/` (on a Deck:
`/home/deck/.local/share/Steam`). The shortcut goes to the account that logged in most recently.
The old `shortcuts.vdf` is copied to `shortcuts.vdf.before-opentboi` first (once). Never run it
while Steam is running there: Steam rewrites the file when it exits.
"""

import argparse
import re
import shutil
import sys
import zlib
from pathlib import Path

STEAM_ID_64_BASE = 76561197960265728
END, MAP, TEXT, NUMBER = 8, 0, 1, 2
BACKUP_SUFFIX = ".before-opentboi"
SHORTCUTS_KEY = "shortcuts"


def new_entry(name, executable, start_directory, launch_options):
    """A shortcut entry with the keys and key order Steam itself writes."""
    quoted_executable = f'"{executable}"'
    appid = zlib.crc32((quoted_executable + name).encode()) | 0x80000000
    return [
        ("appid", appid),
        ("appname", name),
        ("Exe", quoted_executable),
        ("StartDir", f'"{start_directory}"'),
        ("icon", ""),
        ("ShortcutPath", ""),
        ("LaunchOptions", launch_options),
        ("IsHidden", 0),
        ("AllowDesktopConfig", 1),
        ("AllowOverlay", 1),
        ("OpenVR", 0),
        ("Devkit", 0),
        ("DevkitGameID", ""),
        ("DevkitOverrideAppID", 0),
        ("LastPlayTime", 0),
        ("FlatpakAppID", ""),
        ("tags", []),
    ]


def read_text(data, position):
    end = data.index(b"\0", position)
    return data[position:end].decode("utf-8", "surrogateescape"), end + 1


def parse_map(data, position):
    """Binary VDF map starting at `position`; returns (list of (key, value), next position)."""
    items = []
    while True:
        kind = data[position]
        position += 1
        if kind == END:
            return items, position
        key, position = read_text(data, position)
        if kind == MAP:
            value, position = parse_map(data, position)
        elif kind == TEXT:
            value, position = read_text(data, position)
        elif kind == NUMBER:
            value, position = int.from_bytes(data[position : position + 4], "little"), position + 4
        else:
            sys.exit(f"Unknown value type {kind} at byte {position}; not touching the file.")
        items.append((key, value))


def write_text(text):
    return text.encode("utf-8", "surrogateescape") + b"\0"


def write_map(items):
    out = bytearray()
    for key, value in items:
        if isinstance(value, list):
            out += bytes([MAP]) + write_text(key) + write_map(value)
        elif isinstance(value, str):
            out += bytes([TEXT]) + write_text(key) + write_text(value)
        else:
            out += bytes([NUMBER]) + write_text(key) + value.to_bytes(4, "little")
    return bytes(out) + bytes([END])


def load(path):
    """The file's contents as a map; refuses a file that cannot be written back identically."""
    data = path.read_bytes()
    root, _ = parse_map(data, 0)
    if write_map(root) != data:
        sys.exit("Could not reproduce the file byte for byte; not touching it.")
    return root


def most_recent_account_directory(steam_root):
    text = (steam_root / "config" / "loginusers.vdf").read_text()
    for steam_id, body in re.findall(r'"(\d{17})"\s*\{(.*?)\}', text, re.S):
        if re.search(r'"MostRecent"\s*"1"', body):
            return steam_root / "userdata" / str(int(steam_id) - STEAM_ID_64_BASE)
    sys.exit("No account is marked as the most recent one in loginusers.vdf.")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("action", choices=["add", "remove"])
    parser.add_argument("steam_root", type=Path)
    parser.add_argument("name")
    parser.add_argument("executable", nargs="?")
    parser.add_argument("--start-directory", default="")
    parser.add_argument("--launch-options", default="")
    arguments = parser.parse_args()

    shortcuts_path = most_recent_account_directory(arguments.steam_root) / "config" / "shortcuts.vdf"
    root = [(SHORTCUTS_KEY, [])]
    if shortcuts_path.exists():
        backup = shortcuts_path.with_name(shortcuts_path.name + BACKUP_SUFFIX)
        if not backup.exists():
            shutil.copy2(shortcuts_path, backup)
        root = load(shortcuts_path)
    kept = [entry for _, entry in dict(root)[SHORTCUTS_KEY] if dict(entry).get("appname") != arguments.name]
    if arguments.action == "add":
        if not arguments.executable:
            sys.exit("add needs an executable.")
        start_directory = arguments.start_directory or f"{Path(arguments.executable).parent}/"
        kept.append(new_entry(arguments.name, arguments.executable, start_directory, arguments.launch_options))
    root = [(SHORTCUTS_KEY, [(str(index), entry) for index, entry in enumerate(kept)])]
    shortcuts_path.parent.mkdir(parents=True, exist_ok=True)
    shortcuts_path.write_bytes(write_map(root))
    print(f"{arguments.action}: {arguments.name!r}, {len(kept)} shortcut(s) now in {shortcuts_path}")


if __name__ == "__main__":
    main()
