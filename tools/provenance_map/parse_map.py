#!/usr/bin/env python3
"""Parse crates/rhwp-contracts/src/provenance.rs MAP into structured entries.

The map is the single source for export-provenance-map. This parser does not
invent commands or paths; it only reads the rust table so fixtures can drift-
check against the live declaration.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path


@dataclass(frozen=True)
class UntrustedField:
    path: str
    origin: str


@dataclass
class CommandMap:
    command: str
    untrusted: list[UntrustedField] = field(default_factory=list)
    note: str = ""
    map_index: int = 0


def repo_root() -> Path:
    here = Path(__file__).resolve()
    for parent in here.parents:
        candidate = parent / "crates" / "rhwp-contracts" / "src" / "provenance.rs"
        if candidate.is_file():
            return parent
    raise FileNotFoundError("crates/rhwp-contracts/src/provenance.rs")


def provenance_rs_path() -> Path:
    return repo_root() / "crates" / "rhwp-contracts" / "src" / "provenance.rs"


_F_CALL = re.compile(
    r'f\(\s*"([^"]+)"\s*,\s*"((?:\\.|[^"\\])*)"\s*,?\s*\)',
    re.MULTILINE,
)
_CMD = re.compile(r'command:\s*"([^"]+)"')
_NOTE = re.compile(r'note:\s*"((?:\\.|[^"\\])*)"')


def _unescape(text: str) -> str:
    return (
        text.replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace('\\"', '"')
        .replace("\\\\", "\\")
    )


def parse_map(source: str | None = None) -> list[CommandMap]:
    """Parse MAP entries in source order, including duplicate command names."""
    text = source if source is not None else provenance_rs_path().read_text(encoding="utf-8")
    # Rust string line continuations: "foo \\\n    bar" -> "foo bar"
    text = re.sub(r"\\[ \t]*\n[ \t]*", "", text)
    marker = "pub const MAP: &[CommandProvenance] = &["
    start = text.find(marker)
    if start < 0:
        raise ValueError("MAP marker missing in provenance.rs")
    body = text[start + len(marker) :]
    end = body.find("\n];")
    if end < 0:
        raise ValueError("MAP terminator missing")
    body = body[:end]

    chunks = re.split(r"CommandProvenance\s*\{", body)
    entries: list[CommandMap] = []
    for raw in chunks[1:]:
        cmd_m = _CMD.search(raw)
        if not cmd_m:
            continue
        note_m = _NOTE.search(raw)
        note = _unescape(note_m.group(1)) if note_m else ""
        fields = [
            UntrustedField(path=m.group(1), origin=_unescape(m.group(2)))
            for m in _F_CALL.finditer(raw)
        ]
        entries.append(
            CommandMap(
                command=cmd_m.group(1),
                untrusted=fields,
                note=note,
                map_index=len(entries),
            )
        )
    if not entries:
        raise ValueError("MAP parsed zero commands")
    return entries


def unique_by_first(entries: list[CommandMap]) -> list[CommandMap]:
    """entry() in rust uses find(), so the first declaration wins."""
    seen: set[str] = set()
    out: list[CommandMap] = []
    for item in entries:
        if item.command in seen:
            continue
        seen.add(item.command)
        out.append(item)
    return out
