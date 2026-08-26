#!/usr/bin/env python3
"""Decide one V-oracle case from flags or a JSON object."""

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
    parser.add_argument("--has-hangul-pdf", type=_bool)
    parser.add_argument("--versions", default="none")
    parser.add_argument("--page-count-match", type=_bool)
    parser.add_argument("--render-self-pass", type=_bool)
    parser.add_argument("--cheap-ok", type=_bool)
    parser.add_argument("--json-row", help="JSON object with the five input fields")
    args = parser.parse_args(argv)

    if args.json_row:
        blob: Any = json.loads(args.json_row)
        decision = decide_row(blob)
    else:
        missing = [
            name
            for name, value in (
                ("--has-hangul-pdf", args.has_hangul_pdf),
                ("--page-count-match", args.page_count_match),
                ("--render-self-pass", args.render_self_pass),
                ("--cheap-ok", args.cheap_ok),
            )
            if value is None
        ]
        if missing:
            parser.error("missing " + ", ".join(missing))
        decision = decide(
            args.has_hangul_pdf,
            args.versions,
            args.page_count_match,
            args.render_self_pass,
            args.cheap_ok,
        )
    json.dump(decision.to_json(), sys.stdout, ensure_ascii=False, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
