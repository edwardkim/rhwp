#!/usr/bin/env python3
"""expected-fail 카탈로그 연산. 침묵 스킵 금지, 고친 이슈는 목록에서 뺀다.

M05-6 는 #4882 를 닫는다. #4056 #5128 은 남긴다.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

from harness import CatalogEntry, ROUTES, norm_rel

CATALOG_KIND = "pageRoundtripCatalog"
HELD_ISSUES = {4056, 5128}
RESOLVED_ISSUES = {4882}
FOREIGN_OPEN = {3518, 3521, 3737, 4056, 5128}


@dataclass(frozen=True)
class CatalogDiff:
    added: tuple[tuple[str, str], ...]
    removed: tuple[tuple[str, str], ...]
    kept: tuple[tuple[str, str], ...]

    def to_json(self) -> dict[str, Any]:
        fmt = lambda keys: [{"doc": d, "route": r} for d, r in keys]
        return {
            "added": fmt(self.added),
            "removed": fmt(self.removed),
            "kept": fmt(self.kept),
            "addedCount": len(self.added),
            "removedCount": len(self.removed),
            "keptCount": len(self.kept),
        }


def entry_key(entry: CatalogEntry) -> tuple[str, str]:
    return entry.key


def dump_catalog(
    entries: Iterable[CatalogEntry],
    *,
    notes: Iterable[str] | None = None,
) -> dict[str, Any]:
    return {
        "schemaVersion": 1,
        "kind": CATALOG_KIND,
        "notes": list(notes or ()),
        "entries": [
            {
                "doc": e.doc,
                "route": e.route,
                "issue": e.issue,
                "reason": e.reason,
            }
            for e in entries
        ],
    }


def write_catalog(path: Path, entries: Iterable[CatalogEntry], notes: Iterable[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = dump_catalog(entries, notes=notes)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def drop_resolved(entries: Iterable[CatalogEntry], resolved: Iterable[int] = RESOLVED_ISSUES) -> list[CatalogEntry]:
    resolved_set = set(resolved)
    return [e for e in entries if e.issue not in resolved_set]


def require_held(entries: Iterable[CatalogEntry], held: Iterable[int] = HELD_ISSUES) -> list[int]:
    present = {e.issue for e in entries if e.issue is not None}
    return sorted(set(held) - present)


def assert_m05_6_scope(entries: Iterable[CatalogEntry]) -> list[str]:
    """M05-6 계약: #4882 는 빠지고 #4056 #5128 은 남는다."""
    items = list(entries)
    errors: list[str] = []
    issues = {e.issue for e in items}
    if 4882 in issues:
        errors.append("#4882 는 고쳤으므로 카탈로그에서 빼야 한다")
    for issue in HELD_ISSUES:
        if issue not in issues:
            errors.append(f"#{issue} 는 이 PR 에서 고치지 않는다 — 카탈로그에 남겨야 한다")
    for e in items:
        if e.route not in ROUTES:
            errors.append(f"잘못된 route: {e.route} ({e.doc})")
        if e.issue in RESOLVED_ISSUES:
            errors.append(f"해결된 이슈가 남아 있다: {e.doc} #{e.issue}")
    return errors


def diff_catalog(old: Iterable[CatalogEntry], new: Iterable[CatalogEntry]) -> CatalogDiff:
    old_keys = {entry_key(e) for e in old}
    new_keys = {entry_key(e) for e in new}
    return CatalogDiff(
        added=tuple(sorted(new_keys - old_keys)),
        removed=tuple(sorted(old_keys - new_keys)),
        kept=tuple(sorted(old_keys & new_keys)),
    )


def sample_inventory_entry(path: Path, repo: Path) -> dict[str, Any]:
    rel = path.resolve().relative_to(repo.resolve()).as_posix() if path.is_absolute() else norm_rel(str(path))
    stat = path.stat() if path.is_file() else None
    return {
        "doc": rel.replace("\\", "/"),
        "suffix": path.suffix.lower(),
        "bytes": stat.st_size if stat else None,
        "exists": path.is_file(),
    }
