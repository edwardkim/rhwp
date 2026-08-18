from __future__ import annotations
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent

def main() -> int:
    inv = json.loads((HERE / "cycle_units_inventory.json").read_text(encoding="utf-8"))
    sys.path.insert(0, str(HERE))
    total_rules = 0
    for unit in inv["units"]:
        pkg = unit["package"]
        mod = __import__(f"{pkg}.decide", fromlist=["decide", "RULES"])
        if len(mod.RULES) != unit["rules"]:
            raise SystemExit(f"{pkg} rule count drift")
        env = {"declaredCount": 2, "arrayLen": 2, "pageCount": 2, "paraCount": 2, "exitCode": 0}
        verdict = mod.decide(env)
        if not isinstance(verdict, str) or not verdict:
            raise SystemExit(f"{pkg} empty verdict")
        total_rules += len(mod.RULES)
        print(unit["command"], len(mod.RULES), verdict)
    print("RULES", total_rules)
    if total_rules != inv["ruleTotal"]:
        raise SystemExit("inventory drift")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
