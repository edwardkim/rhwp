from __future__ import annotations

import sys
from pathlib import Path

PKG = Path(__file__).resolve().parents[1]
ROOT = PKG.parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
if str(PKG) not in sys.path:
    sys.path.insert(0, str(PKG))
