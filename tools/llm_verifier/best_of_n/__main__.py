"""CLI: rank a set JSON or verify the committed corpus."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    from .rank import expected_ranks_match, rank_mapping
except ImportError:
    from rank import expected_ranks_match, rank_mapping

HERE = Path(__file__).resolve().parent


def check_corpus(corpus_dir: Path) -> int:
    manifest = json.loads((corpus_dir / "manifest.json").read_text(encoding="utf-8"))
    errors = 0
    seen: set[str] = set()
    for shard in manifest["shards"]:
        path = HERE / shard["path"]
        if not path.is_file():
            # manifest paths are relative to package root
            path = HERE / Path(shard["path"]).name
            if not path.is_file():
                path = corpus_dir / Path(shard["path"]).name
        payload = json.loads(path.read_text(encoding="utf-8"))
        for blob in payload:
            sid = blob["setId"]
            if sid in seen:
                print(f"duplicate setId {sid}", file=sys.stderr)
                errors += 1
            seen.add(sid)
            if "process_steps" in blob or "processSteps" in blob:
                print(f"{sid}: process_steps is forbidden", file=sys.stderr)
                errors += 1
            mismatches = expected_ranks_match(blob)
            for item in mismatches:
                print(f"{sid}: {item}", file=sys.stderr)
                errors += 1
    print(json.dumps({"sets": len(seen), "errors": errors, "lineCount": manifest["lineCount"]}))
    return 0 if errors == 0 else 1


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--set-json", type=Path)
    parser.add_argument("--check-corpus", action="store_true")
    parser.add_argument("--corpus-dir", type=Path, default=HERE / "corpus")
    args = parser.parse_args(argv)
    if args.check_corpus:
        return check_corpus(args.corpus_dir)
    if args.set_json is None:
        parser.error("pass --set-json or --check-corpus")
    blob = json.loads(args.set_json.read_text(encoding="utf-8"))
    ranked = rank_mapping(blob)
    json.dump(ranked.to_json(), sys.stdout, ensure_ascii=False, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
