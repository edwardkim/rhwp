#!/usr/bin/env python3
"""Classify one third-party replay observation from flags or JSON."""

from __future__ import annotations

import argparse
import json
import sys
from typing import Any

try:
    from .decide import decide, decide_row
    from .envelopes import decide_envelope
    from .schema import parse_reproduced
except ImportError:
    from decide import decide, decide_row
    from envelopes import decide_envelope
    from schema import parse_reproduced


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", default="")
    parser.add_argument("--expect-sha", default="")
    parser.add_argument("--reproduced", default="")
    parser.add_argument("--tool-version", default="")
    parser.add_argument("--mode", default="verify")
    parser.add_argument("--source", default="replay")
    parser.add_argument("--expected-tool-version", default="")
    parser.add_argument("--json-row", help="JSON object with axis fields")
    parser.add_argument("--envelope-json", help="replay or workCapsule JSON object")
    args = parser.parse_args(argv)

    if args.envelope_json:
        blob: Any = json.loads(args.envelope_json)
        decision = decide_envelope(blob, expected_tool_version=args.expected_tool_version)
    elif args.json_row:
        decision = decide_row(json.loads(args.json_row))
    else:
        decision = decide(
            args.plan,
            args.expect_sha,
            parse_reproduced(args.reproduced) if args.reproduced != "" else None,
            args.tool_version,
            mode=args.mode,
            source=args.source,
            expected_tool_version=args.expected_tool_version,
        )
    json.dump(decision.to_json(), sys.stdout, ensure_ascii=False, indent=2)
    sys.stdout.write("\n")
    return 0 if decision.labor_accepted else 3


if __name__ == "__main__":
    raise SystemExit(main())
