from __future__ import annotations

import sys
from pathlib import Path

PKG = Path(__file__).resolve().parents[1]
PARENT = PKG.parent
REPO = PKG.parents[2]
TESTS = Path(__file__).resolve().parent
for path in (str(TESTS), str(PARENT)):
    if path not in sys.path:
        sys.path.insert(0, path)
