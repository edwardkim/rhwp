#!/usr/bin/env python3
"""Decide one V-shadow pair from flags or a JSON object.

This is a verifier helper. It is not a new rhwp CLI command.
"""

from __future__ import annotations

import argparse
import json
import sys
from typing import Any

try:
    from .decide import decide, decide_row
except ImportError:
    from decide import decide, decide_row


def _bool(value: str) -> bool:
    folded = value.strip().lower()
    if folded in {"1", "true", "yes", "y"}:
        return True
    if folded in {"0", "false", "no", "n"}:
        return False
    raise argparse.ArgumentTypeError(value)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check-a")
    parser.add_argument("--check-b")
    parser.add_argument("--a-pass", type=_bool)
    parser.add_argument("--b-pass", type=_bool)
    parser.add_argument("--json-row", help="JSON object with check_a/check_b/a_pass/b_pass")
    args = parser.parse_args(argv)

    if args.json_row:
        blob: Any = json.loads(args.json_row)
        decision = decide_row(blob)
    else:
        missing = [
            name
            for name, value in (
                ("--check-a", args.check_a),
                ("--check-b", args.check_b),
                ("--a-pass", args.a_pass),
                ("--b-pass", args.b_pass),
            )
            if value is None
        ]
        if missing:
            parser.error("missing " + ", ".join(missing))
        decision = decide(args.check_a, args.check_b, args.a_pass, args.b_pass)
    json.dump(decision.to_json(), sys.stdout, ensure_ascii=False, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
