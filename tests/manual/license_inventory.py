#!/usr/bin/env python3
"""Bundle declared dependency licenses/notices for the diagnostic binaries.

Includes unmodified registry source for MPL-2.0 crates. This is diagnostic
distribution support, not an installer or full release packaging system.
"""
import json
import pathlib
import shutil
import subprocess
import sys

destination = pathlib.Path(sys.argv[1])
destination.mkdir(parents=True, exist_ok=False)
host = next(line.split(": ", 1)[1] for line in subprocess.check_output(["rustc", "-vV"], text=True, encoding="utf-8").splitlines() if line.startswith("host: "))
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", host], text=True, encoding="utf-8"))
nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
reachable = set()
pending = list(metadata["workspace_members"])
while pending:
    package_id = pending.pop()
    if package_id not in reachable:
        reachable.add(package_id)
        pending.extend(nodes[package_id]["dependencies"])
inventory = ["VeekPanel diagnostic dependency notices", "", f"Registry dependencies reachable for {host}, including build tools, are listed.",
             "License texts remain under their original terms. Registry packages are unmodified.", ""]
for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
    if not package["source"] or package["id"] not in reachable:
        continue
    name = package["name"] + "-" + package["version"]
    source = pathlib.Path(package["manifest_path"]).parent
    target = destination / name
    target.mkdir()
    inventory.append(f"{name}: {package.get('license') or 'UNSPECIFIED'}")
    inventory.append(f"Source: https://crates.io/api/v1/crates/{package['name']}/{package['version']}/download")
    notices = [f for f in source.rglob("*") if f.is_file() and f.name.lower().startswith(("license", "licence", "copying", "notice"))]
    if not notices:
        # Exact-version license omitted from the published cookie-factory archive.
        # The checked-in upstream text and provenance are reviewed, never guessed.
        supplement = pathlib.Path(__file__).resolve().parents[2] / "packaging" / "third-party" / name
        if supplement.is_dir():
            shutil.copytree(supplement, target, dirs_exist_ok=True)
            inventory.append("License source: bundled PROVENANCE.txt")
            notices = []
        else:
            raise RuntimeError(f"No license/notice file found for {name}; inspect before distributing")
    for notice in notices:
        output = target / notice.relative_to(source)
        output.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(notice, output)
    if "MPL-2.0" in (package.get("license") or ""):
        shutil.copytree(source, target / "unmodified-source")
    inventory.append("")
(destination / "README.txt").write_text("\n".join(inventory), encoding="utf-8")
print(f"Dependency notices written to {destination}")
