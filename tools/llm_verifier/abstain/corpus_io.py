"""Read committed V-abstain TSV shards."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Iterator

from .schema import CASE_COLUMNS, AbstainCase

HERE = Path(__file__).resolve().parent
CORPUS_DIR = HERE / "corpus"


def load_manifest(path: Path | None = None) -> dict:
    manifest_path = path or (CORPUS_DIR / "manifest.json")
    return json.loads(manifest_path.read_text(encoding="utf-8"))


def iter_tsv(path: Path) -> Iterator[AbstainCase]:
    text = path.read_text(encoding="utf-8")
    if text.startswith("\ufeff"):
        raise ValueError(f"BOM is not allowed: {path}")
    lines = text.splitlines()
    if not lines:
        raise ValueError(f"empty TSV: {path}")
    header = lines[0].split("\t")
    if header != list(CASE_COLUMNS):
        raise ValueError(f"unexpected columns in {path}: {header}")
    for lineno, line in enumerate(lines[1:], start=2):
        if not line:
            continue
        cells = line.split("\t")
        if len(cells) != len(CASE_COLUMNS):
            raise ValueError(f"{path}:{lineno} expected {len(CASE_COLUMNS)} cells")
        yield AbstainCase.from_mapping(dict(zip(CASE_COLUMNS, cells, strict=True)))


def iter_corpus(corpus_dir: Path | None = None) -> Iterator[AbstainCase]:
    root = corpus_dir or CORPUS_DIR
    manifest = load_manifest(root / "manifest.json")
    for shard in manifest["shards"]:
        rel = shard["path"]
        path = Path(rel)
        if not path.is_absolute():
            # manifest paths are "corpus/shard_XXXX.tsv"
            path = HERE / rel
        yield from iter_tsv(path)


def iter_axis_table(path: Path | None = None) -> Iterator[dict[str, str]]:
    axis = path or (HERE / "fixtures" / "axis_closed_set.tsv")
    text = axis.read_text(encoding="utf-8")
    lines = [line for line in text.splitlines() if line]
    header = lines[0].split("\t")
    for line in lines[1:]:
        yield dict(zip(header, line.split("\t"), strict=True))
