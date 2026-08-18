from __future__ import annotations
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent

def main() -> int:
    inv = json.loads((HERE / "cycle_units_inventory.json").read_text(encoding="utf-8"))
    sys.path.insert(0, str(HERE))
    total = 0
    for unit in inv["units"]:
        pkg = "u_" + unit["command"].replace("-", "_")
        mod = __import__(f"{pkg}.verify_corpus", fromlist=["verify"])
        got = mod.verify()
        total += got["rows"]
        print(unit["command"], got["rows"], got["byVerdict"])
    print("TOTAL", total)
    if total != inv["rowTotal"]:
        raise SystemExit("inventory drift")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
