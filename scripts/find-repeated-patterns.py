#!/usr/bin/env python3
"""Find repeated declarations and normalized code blocks across the repo."""

from __future__ import annotations

import argparse
import hashlib
import os
import re
from collections import defaultdict
from pathlib import Path


DEFAULT_SUFFIXES = {
    ".rs",
    ".js",
    ".ts",
    ".svelte",
    ".mjs",
    ".cjs",
    ".json",
    ".toml",
    ".md",
}

SKIP_DIRS = {
    ".git",
    "target",
    "node_modules",
    "dist",
    "build",
    "out",
    ".svelte-kit",
    ".vercel",
    "vendor",
}

SKIP_NAMES = {
    "Cargo.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
}

DECLARATION_RE = re.compile(
    r"^\s*(?:pub\s+)?(?:enum|struct|type|interface|function|fn|const|class)\s+([A-Za-z_][A-Za-z0-9_]*)"
)


def iter_files(root: Path, suffixes: set[str], max_bytes: int) -> list[Path]:
    files: list[Path] = []
    for directory, dirnames, filenames in os.walk(root):
        dirnames[:] = [name for name in dirnames if name not in SKIP_DIRS]
        for filename in filenames:
            if filename in SKIP_NAMES:
                continue
            path = Path(directory) / filename
            if path.suffix not in suffixes:
                continue
            if path.stat().st_size > max_bytes:
                continue
            files.append(path)
    return sorted(files)


def normalized_blocks(path: Path, window: int) -> list[tuple[int, str]]:
    lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
    normalized = [normalize_line(line) for line in lines]
    blocks: list[tuple[int, str]] = []
    for index in range(0, max(len(normalized) - window + 1, 0)):
        block = "\n".join(normalized[index : index + window]).strip()
        if not block or block.count("\n") < window - 1:
            continue
        if len(block) < 80:
            continue
        digest = hashlib.sha256(block.encode()).hexdigest()
        blocks.append((index + 1, digest))
    return blocks


def normalize_line(line: str) -> str:
    line = re.sub(r"//.*$", "", line)
    line = re.sub(r"#.*$", "", line)
    line = re.sub(r'"(?:\\.|[^"\\])*"', '"..."', line)
    line = re.sub(r"\b\d+\b", "0", line)
    return re.sub(r"\s+", " ", line).strip()


def repeated_declarations(files: list[Path], root: Path) -> dict[str, list[str]]:
    declarations: dict[str, list[str]] = defaultdict(list)
    for path in files:
        for line_number, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), 1):
            match = DECLARATION_RE.match(line)
            if match:
                declarations[match.group(1)].append(f"{path.relative_to(root)}:{line_number}")
    return {name: hits for name, hits in declarations.items() if len(hits) > 1}


def repeated_blocks(files: list[Path], root: Path, window: int) -> dict[str, list[str]]:
    blocks: dict[str, list[str]] = defaultdict(list)
    for path in files:
        for line_number, digest in normalized_blocks(path, window):
            blocks[digest].append(f"{path.relative_to(root)}:{line_number}")
    return {digest: hits for digest, hits in blocks.items() if len(hits) > 1}


def print_group(title: str, groups: dict[str, list[str]], limit: int) -> None:
    print(f"\n{title}")
    if not groups:
        print("  none")
        return
    for name, hits in sorted(groups.items(), key=lambda item: (-len(item[1]), item[0]))[:limit]:
        print(f"  {name} ({len(hits)})")
        for hit in hits[:limit]:
            print(f"    {hit}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=".", help="repository root")
    parser.add_argument("--window", type=int, default=8, help="line count per repeated block")
    parser.add_argument("--limit", type=int, default=20, help="max groups and hits to print")
    parser.add_argument(
        "--max-file-bytes",
        type=int,
        default=256_000,
        help="skip files larger than this many bytes",
    )
    parser.add_argument("--suffix", action="append", help="file suffix to scan; repeatable")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    suffixes = set(args.suffix or DEFAULT_SUFFIXES)
    files = iter_files(root, suffixes, args.max_file_bytes)

    print(f"Scanned {len(files)} files under {root}")
    print_group("Repeated declarations", repeated_declarations(files, root), args.limit)
    print_group("Repeated normalized blocks", repeated_blocks(files, root, args.window), args.limit)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
