"""Load the skill-router catalog (stdlib only)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

CATALOG_RELPATH = Path("tools") / "skill_router" / "catalog.json"


def load_catalog(repo_root: str | Path) -> dict[str, Any]:
    """Return the parsed catalog.json for *repo_root*."""
    path = Path(repo_root) / CATALOG_RELPATH
    with path.open(encoding="utf-8") as fh:
        return json.load(fh)
