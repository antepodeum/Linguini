#!/usr/bin/env python3
"""Bump and publish all Linguini Rust and npm packages together."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUST_MANIFESTS = sorted(ROOT.glob("crates/*/Cargo.toml"))
NPM_MANIFESTS = [
    ROOT / "packages/linguini-web/package.json",
    ROOT / "packages/linguini-sveltekit/package.json",
    ROOT / "plugins/vite/package.json",
    ROOT / "editors/vscode/package.json",
]
VERSION_RE = re.compile(r'^version = "([^"]+)"$', re.MULTILINE)
PATH_DEP_VERSION_RE = re.compile(r'(path = "\.\./[^"]+", version = )"[^"]+"')
SEMVER_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)(?:-alpha\.(\d+))?$")


def next_alpha(version: str) -> str:
    match = SEMVER_RE.match(version)
    if not match:
        raise ValueError(f"unsupported version: {version}")
    major, minor, patch, alpha = match.groups()
    if alpha is None:
        return f"{major}.{minor}.{patch}-alpha.0"
    return f"{major}.{minor}.{patch}-alpha.{int(alpha) + 1}"


def rust_version() -> str:
    for manifest in RUST_MANIFESTS:
        text = manifest.read_text()
        match = VERSION_RE.search(text)
        if match:
            return match.group(1)
    raise RuntimeError("no Rust package versions found")


def target_version(args: argparse.Namespace) -> str:
    if args.version:
        return args.version
    current = rust_version()
    if args.alpha:
        return next_alpha(current)
    raise SystemExit("pass --version X.Y.Z[-alpha.N] or --alpha")


def bump_rust(version: str) -> None:
    for manifest in RUST_MANIFESTS:
        text = manifest.read_text()
        text = VERSION_RE.sub(f'version = "{version}"', text, count=1)
        text = PATH_DEP_VERSION_RE.sub(rf'\1"{version}"', text)
        manifest.write_text(text)


def bump_npm(version: str) -> None:
    for manifest in NPM_MANIFESTS:
        if not manifest.exists():
            continue
        data = json.loads(manifest.read_text())
        data["version"] = version
        manifest.write_text(json.dumps(data, indent=2) + "\n")

    for lockfile in [ROOT / "editors/vscode/package-lock.json", ROOT / "plugins/vite/pnpm-lock.yaml"]:
        if lockfile.exists():
            print(f"warning: update lockfile manually or via package manager: {lockfile.relative_to(ROOT)}")


def run(command: list[str], dry_run: bool, cwd: Path = ROOT) -> None:
    print("+", " ".join(command), f"(cwd: {cwd.relative_to(ROOT) if cwd != ROOT else '.'})")
    if not dry_run:
        subprocess.run(command, cwd=cwd, check=True)


def publish_rust(dry_run: bool) -> None:
    for manifest in RUST_MANIFESTS:
        text = manifest.read_text()
        if "publish = false" in text:
            continue
        package = manifest.parent.name
        run(["cargo", "publish", "-p", package], dry_run)


def publish_npm(dry_run: bool) -> None:
    for manifest in NPM_MANIFESTS:
        if not manifest.exists():
            continue
        package_dir = manifest.parent
        command = ["npm", "publish"]
        if "-alpha." in json.loads(manifest.read_text())["version"]:
            command.extend(["--tag", "alpha"])
        run(command, dry_run=dry_run, cwd=package_dir)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", help="set exact semver")
    parser.add_argument("--alpha", action="store_true", help="increment alpha suffix")
    parser.add_argument("--publish", action="store_true", help="publish Rust and npm packages")
    parser.add_argument("--dry-run", action="store_true", help="print publish commands only")
    args = parser.parse_args()

    version = target_version(args)
    bump_rust(version)
    bump_npm(version)
    print(f"bumped packages to {version}")

    if args.publish:
        publish_rust(args.dry_run)
        publish_npm(args.dry_run)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
