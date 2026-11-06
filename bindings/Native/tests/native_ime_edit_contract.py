#!/usr/bin/env python3
"""Actual worker ABI: atomic UTF-16 edits reject stale/surrogate deltas."""
import argparse
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_worker import NativeWorker


def run(worker, command):
    result = worker.wait(worker.submit(command))
    assert result["ok"], result
    return result


def target(info):
    return {name: info[name] for name in ("documentId", "revision")}


def context(worker, current):
    return run(worker, {"op": "imeEditContext", "target": current})["context"]


def main(library):
    with NativeWorker(library) as worker:
        info = run(worker, {"op": "new", "replace": None})["info"]
        point = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}
        current = target(info)
        run(worker, {"op": "beginEdit", "target": current,
                     "selection": {"anchor": point, "focus": point}})
        outcome = run(worker, {"op": "replace", "target": current, "text": "가😀나"})["outcome"]
        current["revision"] = outcome["revision"]
        expected = context(worker, current)
        original = run(worker, {"op": "info"})["info"]
        invalid = worker.wait(worker.submit({"op": "applyImeEdit", "target": current, "expected": expected,
            "edit": {"kind": "commit", "replacement": {"range": {"start": 2, "end": 3}, "text": "X"}}}))
        assert not invalid["ok"], invalid
        assert run(worker, {"op": "info"})["info"] == original
        assert context(worker, current) == expected
        outcome = run(worker, {"op": "applyImeEdit", "target": current, "expected": expected,
            "edit": {"kind": "commit", "replacement": {"range": {"start": 1, "end": 3}, "text": "한"},
                     "selection": {"anchor": 2, "focus": 2}}})["outcome"]
        current["revision"] = outcome["revision"]
        assert context(worker, current)["context"]["text"] == "가한나"
        stale = worker.wait(worker.submit({"op": "applyImeEdit", "target": current, "expected": expected,
            "edit": {"kind": "commit", "replacement": {"range": {"start": 0, "end": 1}, "text": "X"}}}))
        assert not stale["ok"], stale
        assert context(worker, current)["context"]["text"] == "가한나"
        outcome = run(worker, {"op": "undo", "target": current})["outcome"]
        current["revision"] = outcome["revision"]
        assert context(worker, current)["context"]["text"] == "가😀나"
        print("atomic IME ABI: non-BMP, stale identity, rollback and undo passed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    main(parser.parse_args().library)
