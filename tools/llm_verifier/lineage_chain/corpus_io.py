"""TSV shard IO for the committed V-lineage corpus."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Iterable

try:
    from .schema import CASE_COLUMNS, LineageCase
except ImportError:  # python generate_corpus.py
    from schema import CASE_COLUMNS, LineageCase

HERE = Path(__file__).resolve().parent
CORPUS_DIR = HERE / "corpus"


def parse_tsv(text: str) -> list[dict[str, str]]:
    lines = [line for line in text.splitlines() if line.strip()]
    if not lines:
        return []
    header = lines[0].split("\t")
    rows: list[dict[str, str]] = []
    for line in lines[1:]:
        cells = line.split("\t")
        row = {header[i]: (cells[i] if i < len(cells) else "") for i in range(len(header))}
        rows.append(row)
    return rows


def tsv_line(case: LineageCase) -> str:
    row = case.to_row()
    return "\t".join(row[col] for col in CASE_COLUMNS)


def write_shard(path: Path, cases: Iterable[LineageCase]) -> int:
    path.parent.mkdir(parents=True, exist_ok=True)
    count = 0
    with path.open("w", encoding="utf-8", newline="\n") as handle:
        handle.write("\t".join(CASE_COLUMNS) + "\n")
        for case in cases:
            handle.write(tsv_line(case) + "\n")
            count += 1
    return count


def load_shard(path: Path) -> list[LineageCase]:
    rows = parse_tsv(path.read_text(encoding="utf-8"))
    return [LineageCase.from_mapping(row) for row in rows]


def load_manifest(dir_path: Path | None = None) -> dict[str, Any]:
    root = dir_path or CORPUS_DIR
    return json.loads((root / "manifest.json").read_text(encoding="utf-8"))


def load_corpus(dir_path: Path | None = None) -> list[LineageCase]:
    root = dir_path or CORPUS_DIR
    manifest = load_manifest(root)
    cases: list[LineageCase] = []
    for shard in manifest["shards"]:
        rel = shard["path"]
        path = root / Path(rel).name if not (root / rel).exists() else root / rel
        if not path.exists():
            path = HERE / rel
        cases.extend(load_shard(path))
    return cases
