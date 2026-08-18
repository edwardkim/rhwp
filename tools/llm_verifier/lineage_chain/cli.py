#!/usr/bin/env python3
"""Classify one lineage-chain observation from flags or JSON."""

from __future__ import annotations

import argparse
import json
import sys
from typing import Any

try:
    from .decide import decide, decide_row
    from .envelopes import decide_envelope
    from .schema import parse_optional_bool
except ImportError:
    from decide import decide, decide_row
    from envelopes import decide_envelope
    from schema import parse_optional_bool


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--parent-out", default="")
    parser.add_argument("--child-in", default="")
    parser.add_argument("--parent-ok", default="")
    parser.add_argument("--lineage-ok", default="")
    parser.add_argument("--broken-at", default="")
    parser.add_argument("--source", default="lineage")
    parser.add_argument("--kind", default="lineage")
    parser.add_argument("--parent-state", default="ok")
    parser.add_argument("--reproduced", default="")
    parser.add_argument("--json-row", help="JSON object with axis fields")
    parser.add_argument("--envelope-json", help="lineage or workCapsule JSON object")
    args = parser.parse_args(argv)

    if args.envelope_json:
        blob: Any = json.loads(args.envelope_json)
        decision = decide_envelope(blob)
    elif args.json_row:
        decision = decide_row(json.loads(args.json_row))
    else:
        decision = decide(
            args.parent_out,
            args.child_in,
            parse_optional_bool(args.parent_ok) if args.parent_ok != "" else None,
            parse_optional_bool(args.lineage_ok) if args.lineage_ok != "" else None,
            args.broken_at,
            source=args.source,
            kind=args.kind,
            parent_state=args.parent_state,
            reproduced=parse_optional_bool(args.reproduced) if args.reproduced != "" else None,
        )
    json.dump(decision.to_json(), sys.stdout, ensure_ascii=False, indent=2)
    sys.stdout.write("\n")
    return 0 if decision.chain_accepted else int(decision.exit_class or "3")


if __name__ == "__main__":
    raise SystemExit(main())
