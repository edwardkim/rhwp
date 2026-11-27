#!/usr/bin/env python3
"""Exercise nested cell paragraph split, join and undo on the actual C ABI."""

import argparse
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_worker import NativeWorker


def call(worker, command):
    result = worker.wait(worker.submit(command))
    assert result["ok"], result
    return result


def contents(worker, target):
    snapshot = call(worker, {"op": "accessibility", "target": target,
                             "request": {"maxNodes": 128}})["snapshot"]
    assert not snapshot["truncated"], snapshot["limitations"]
    assert all(node["textComplete"] for node in snapshot["nodes"])
    return [(node["role"], node["start"], node["end"], node["text"])
            for node in snapshot["nodes"]]


def check_split_round_trips(library, worker, target, selection, expected):
    for file_format, signature in (("hwp", bytes.fromhex("d0cf11e0a1b11ae1")),
                                   ("hwpx", b"PK")):
        def save(host, current):
            prepared = call(host, {"op": "prepareSave", "target": current, "format": file_format})
            data = host.copy_buffer(prepared)
            assert data.startswith(signature) and prepared["report"]["count"] == 0, prepared
            saved = call(host, {"op": "acknowledgeSave", "documentId": current["documentId"],
                                "token": prepared["token"]})
            assert not saved["isDirty"], saved
            return data

        data = save(worker, target)
        with NativeWorker(library) as verifier:
            for pass_number in range(2):
                reopened = verifier.wait(verifier.open_bytes(data))
                assert reopened["ok"] and not reopened["info"]["isDirty"], reopened
                current = {key: reopened["info"][key] for key in target}
                assert contents(verifier, current) == expected
                if pass_number == 0:
                    call(verifier, {"op": "beginEdit", "target": current, "selection": selection})
                    data = save(verifier, current)
                    call(verifier, {"op": "closeDocument", "target": current})


def run(library):
    form = (Path(__file__).resolve().parents[3]
            / "samples/issue5599_oracle/3191107_leave_request_form.hwpx")
    with NativeWorker(library) as worker:
        opened = worker.wait(worker.open_bytes(form.read_bytes()))
        assert opened["ok"], opened
        info = opened["info"]
        target = {"documentId": info["documentId"], "revision": info["revision"]}

        def accepted(op, **payload):
            result = call(worker, {"op": op, "target": target, **payload})
            if "outcome" in result:
                target["revision"] = result["outcome"]["revision"]
            return result

        def point(paragraph, offset=0):
            return {"section": 0, "paragraph": 1, "scalarOffset": offset, "cellPath": [
                {"control": 0, "cell": 1, "paragraph": 1},
                {"control": 0, "cell": 5, "paragraph": paragraph},
            ]}

        def caret(paragraph, offset=0):
            return {"anchor": point(paragraph, offset), "focus": point(paragraph, offset)}

        original = contents(worker, target)
        accepted("beginEdit", selection=caret(0))
        accepted("replace", text="가😀나다")
        selection = {"anchor": point(0, 3), "focus": point(0, 1)}
        accepted("setSelection", selection=selection)
        revision = target["revision"]
        split = accepted("executeCommand", command={"command": "splitParagraph"})["outcome"]
        assert split["selection"] == caret(1) and split["revision"] == revision + 1, split
        assert accepted("imeContext")["context"]["text"] == "다"
        split_index, = (i for i, (_, start, _, _) in enumerate(original) if start == point(0))
        expected = original[:split_index] + [
            ("tableCellParagraph", point(0), point(0, 1), "가"),
            ("tableCellParagraph", point(1), point(1, 1), "다"),
        ] + original[split_index + 1:]
        # Compare the entire structural snapshot, including the untouched birth-date
        # cell in the next inner table, to its original paragraph addresses and text.
        assert contents(worker, target) == expected
        check_split_round_trips(library, worker, target, split["selection"], expected)

        undone = accepted("undo")["outcome"]
        assert undone["selection"] == selection, undone
        assert accepted("imeContext")["context"]["text"] == "가😀나다"
        accepted("redo")
        revision = target["revision"]
        joined = accepted("executeCommand", command={"command": "joinPreviousParagraph"})["outcome"]
        assert joined["selection"] == caret(0, 1) and joined["revision"] == revision + 1, joined
        assert accepted("imeContext")["context"]["text"] == "가다"
        assert worker.host.call("capabilities")["tableCellParagraphCommands"]


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    run(parser.parse_args().library)
