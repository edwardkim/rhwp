#!/usr/bin/env python3
"""Exercise command and IME-context transport against the actual native worker."""

import argparse
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_worker import NativeWorker


def run(library):
    with NativeWorker(library) as worker:
        def accepted(command):
            result = worker.wait(worker.submit(command))
            assert result["ok"], result
            return result

        info = accepted({"op": "new", "replace": None})["info"]
        target = {"documentId": info["documentId"], "revision": 0}
        point = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}
        accepted({"op": "beginEdit", "target": target,
                  "selection": {"anchor": point, "focus": point}})
        edited = accepted({"op": "replace", "target": target, "text": "가👩\u200d💻"})
        target["revision"] = edited["outcome"]["revision"]
        context = accepted({"op": "imeContext", "target": target})["context"]
        assert context["text"] == "가👩\u200d💻" and context["cursorScalar"] == 4, context
        moved = accepted({"op": "executeCommand", "target": target,
                          "command": {"command": "moveLeft"}})
        assert moved["kind"] == "command", moved
        assert moved["outcome"]["selection"]["focus"]["scalarOffset"] == 1, moved
        assert moved["outcome"]["revision"] == target["revision"], moved
        deleted = accepted({"op": "executeCommand", "target": target,
                            "command": {"command": "deleteForward"}})
        target["revision"] = deleted["outcome"]["revision"]
        assert accepted({"op": "imeContext", "target": target})["context"]["text"] == "가"
        undone = accepted({"op": "undo", "target": target})
        target["revision"] = undone["outcome"]["revision"]
        assert accepted({"op": "imeContext", "target": target})["context"]["text"] == "가👩\u200d💻"


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    run(parser.parse_args().library)
