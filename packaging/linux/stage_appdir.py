#!/usr/bin/env python3
"""Copy an audited Nobara AppDir, omitting unrelated host schemas/typelibs.

Never modifies the input. Keep data owned by the already bundled native source
packages, plus GNOME desktop schemas used by the GTK theme hook. Unknown data
fails closed. No native library, GTK input module or accessibility code is pruned.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess


def stage(source, destination, inventory):
    source, destination = source.resolve(), destination.absolute()
    if destination.exists() or source in destination.parents:
        raise ValueError("Use a fresh destination outside the input AppDir")
    audited = json.loads(inventory.read_text())
    # Older inventories contain only ELF records; expanded inventories also have
    # data-only packages. Do not let those reintroduce unrelated host resources.
    elf_owners = {f["package"] for f in audited["files"] if f.get("kind", "elf") == "elf"}
    keep_sources = {audited["packages"][p]["source_rpm"] for p in elf_owners}
    decisions = []
    for relative, host in [
        ("usr/share/glib-2.0/schemas", Path("/usr/share/glib-2.0/schemas")),
        ("usr/lib/girepository-1.0", Path("/usr/lib64/girepository-1.0")),
    ]:
        for path in sorted((source / relative).iterdir()):
            if path.name == "gschemas.compiled":
                continue  # Regenerated from the retained source schemas below.
            original = host / path.name
            if path.is_symlink() or not path.is_file() or not original.is_file():
                raise ValueError(f"Unexpected resource: {path}")
            content = path.read_bytes()
            if content != original.read_bytes():
                raise ValueError(f"Resource differs from RPM-owned host copy: {path}")
            owners = subprocess.check_output([
                "rpm", "-qf", "--qf", "%{NAME}\t%{SOURCERPM}\n", str(original)
            ], text=True).splitlines()
            keep = any(
                srpm in keep_sources or name == "gsettings-desktop-schemas"
                for name, srpm in (owner.split("\t") for owner in owners)
            )
            decisions.append({"file": str(path.relative_to(source)),
                              "sha256": hashlib.sha256(content).hexdigest(),
                              "owners": owners, "retained": keep})
    shutil.copytree(source, destination, symlinks=True)
    for decision in decisions:
        if not decision["retained"]:
            (destination / decision["file"]).unlink()
    subprocess.run(["glib-compile-schemas", "--strict",
                    str(destination / "usr/share/glib-2.0/schemas")], check=True)
    report = destination.parent / (destination.name + "-resource-selection.json")
    report.write_text(json.dumps(decisions, indent=2) + "\n")
    print(f"Retained {sum(d['retained'] for d in decisions)} data files; "
          f"omitted {sum(not d['retained'] for d in decisions)} unrelated files")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path)
    parser.add_argument("inventory", type=Path)
    args = parser.parse_args()
    stage(args.source, args.destination, args.inventory)
