#!/usr/bin/env python3
"""Exercise command and IME-context transport against the actual native worker."""

import argparse
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_worker import NativeWorker


def selection_formatting(library):
    def uniform(value):
        return {"kind": "uniform", "value": value}

    mixed = {"kind": "mixed"}
    with NativeWorker(library) as worker:
        def send(**command):
            return worker.wait(worker.submit(command))

        def accepted(**command):
            result = send(**command)
            assert result["ok"], result
            return result

        assert worker.host.call("capabilities")["selectionFormatting"]
        info = accepted(op="new", replace=None)["info"]
        target = {"documentId": info["documentId"], "revision": info["revision"]}
        font = info["fontGeneration"]
        head = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}
        caret = {"anchor": head, "focus": head}
        query = {"op": "selectionFormatting", "target": target, "fontGeneration": font,
                 "selection": caret}
        assert send(**query)["code"] == "unsupported"
        accepted(op="beginEdit", target=target, selection=caret)
        before = accepted(op="info")["info"]
        empty = accepted(**query)
        assert empty["kind"] == "selectionFormatting", empty
        assert empty["target"] == target and empty["fontGeneration"] == font, empty
        assert empty["selection"] == caret, empty
        assert empty["formatting"]["bold"]["kind"] == "uniform", empty
        assert empty["formatting"]["fontSize"]["kind"] == "uniform", empty
        assert accepted(op="info")["info"] == before
        composing = accepted(op="beginComposition", target=target)["outcome"]
        assert send(**query)["code"] == "invalidArgument"
        accepted(op="cancelComposition", target=target, compositionId=composing["compositionId"])
        assert accepted(op="info")["info"] == before

        edited = accepted(op="replace", target=target, text="가😀나")["outcome"]
        target["revision"] = edited["revision"]
        mid = {**head, "scalarOffset": 1}
        end = {**head, "scalarOffset": 3}
        first = {"anchor": mid, "focus": head}
        rest = {"anchor": end, "focus": mid}
        for selection, size in ((first, 1500), (rest, 1900)):
            accepted(op="setSelection", target=target, selection=selection)
            result = accepted(op="executeCommand", target=target, command={
                "command": "formatCharacters", "bold": True, "fontSize": size})
            target["revision"] = result["outcome"]["revision"]
        result = accepted(op="executeCommand", target=target, command={
            "command": "setParagraphAlignment", "alignment": "right"})
        target["revision"] = result["outcome"]["revision"]
        assert accepted(**{**query, "selection": rest})["formatting"] == {
            "bold": uniform(True), "fontSize": uniform(1900), "alignment": "right"}

        whole = {"anchor": end, "focus": head}
        accepted(op="setSelection", target=target, selection=whole)
        assert send(**{**query, "selection": rest})["code"] == "invalidArgument"
        assert send(**{**query, "selection": whole, "fontGeneration": font + 1})["code"] == "invalidArgument"
        stale = {**target, "revision": target["revision"] - 1}
        assert send(**{**query, "selection": whole, "target": stale})["code"] == "staleRevision"
        before = accepted(op="info")["info"]
        assert accepted(**{**query, "selection": whole})["formatting"] == {
            "bold": uniform(True), "fontSize": mixed, "alignment": "right"}
        assert accepted(op="info")["info"] == before
        accepted(op="setSelection", target=target, selection=first)
        result = accepted(op="executeCommand", target=target, command={
            "command": "formatCharacters", "bold": False})
        target["revision"] = result["outcome"]["revision"]
        accepted(op="setSelection", target=target, selection=whole)
        assert accepted(**{**query, "selection": whole})["formatting"] == {
            "bold": mixed, "fontSize": mixed, "alignment": "right"}

        handle = worker.host.open_bytes()
        try:
            session = worker.host.request(handle, op="info")
            worker.host.request(handle, op="beginEdit", expectedRevision=session["revision"],
                                selection=caret)
            direct = worker.host.request(handle, op="selectionFormatting",
                                         expectedRevision=session["revision"],
                                         fontGeneration=session["fontGeneration"], selection=caret)
            assert direct["formatting"] == empty["formatting"], direct
        finally:
            worker.host.call("close", handle)


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

        end = {**point, "scalarOffset": 1}
        selection = {"anchor": end, "focus": point}
        accepted({"op": "setSelection", "target": target, "selection": selection})
        assert worker.host.call("capabilities")["basicTextFormatting"]
        characters = {"command": "formatCharacters", "bold": True, "fontSize": 1500}
        alignment = {"command": "setParagraphAlignment", "alignment": "right"}
        for command in (characters, alignment):
            formatted = accepted({"op": "executeCommand", "target": target, "command": command})
            outcome = formatted["outcome"]
            assert formatted["kind"] == "command" and formatted["preferredX"] is None, formatted
            assert outcome["documentChanged"] and outcome["historyChanged"], outcome
            assert outcome["selection"] == selection, outcome
            assert outcome["revision"] == target["revision"] + 1, outcome
            target["revision"] = outcome["revision"]
        invalid = worker.wait(worker.submit({
            "op": "executeCommand", "target": target,
            "command": {"command": "formatCharacters", "fontSize": 0},
        }))
        assert not invalid["ok"] and invalid["code"] == "invalidArgument", invalid
        unchanged = accepted({"op": "executeCommand", "target": target, "command": alignment})
        assert not unchanged["outcome"]["documentChanged"], unchanged
        assert unchanged["outcome"]["revision"] == target["revision"], unchanged
        assert accepted({"op": "imeContext", "target": target})["context"]["selectedText"] == "가"

def table_navigation(library):
    form = (Path(__file__).resolve().parents[3]
            / "samples/issue5599_oracle/3191107_leave_request_form.hwpx")
    with NativeWorker(library) as worker:
        opened = worker.wait(worker.open_bytes(form.read_bytes()))
        assert opened["ok"], opened
        info = opened["info"]
        target = {"documentId": info["documentId"], "revision": info["revision"]}

        def point(cell, offset=0):
            return {"section": 0, "paragraph": 1, "scalarOffset": offset, "cellPath": [
                {"control": 0, "cell": 1, "paragraph": 1},
                {"control": 0, "cell": cell, "paragraph": 0},
            ]}

        start = point(0, 1)
        began = worker.wait(worker.submit({"op": "beginEdit", "target": target,
                                          "selection": {"anchor": start, "focus": start}}))
        assert began["ok"], began
        for direction, expected in [("previous", start), ("next", point(1)), ("next", point(2)),
                                    ("previous", point(1)), ("previous", point(0))]:
            moved = worker.wait(worker.submit({"op": "executeCommand", "target": target,
                                              "command": {"command": "navigateTableCell",
                                                          "direction": direction}}))
            assert moved["ok"] and moved["kind"] == "command", moved
            assert moved["outcome"]["selection"] == {"anchor": expected, "focus": expected}, moved
            assert moved["outcome"]["revision"] == target["revision"], moved
            assert not moved["outcome"]["documentChanged"], moved
            assert not moved["outcome"]["historyChanged"], moved
        assert worker.host.call("capabilities")["tableCellNavigation"]
        label = {"anchor": point(4, 4), "focus": point(4)}
        selection = worker.wait(worker.submit({"op": "setSelection", "target": target, "selection": label}))
        assert selection["ok"], selection
        formatting = worker.wait(worker.submit({"op": "selectionFormatting", "target": target,
                                                 "fontGeneration": info["fontGeneration"],
                                                 "selection": label}))
        assert formatting["ok"] and formatting["selection"] == label, formatting
        assert formatting["formatting"]["fontSize"]["kind"] == "uniform", formatting


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    library = parser.parse_args().library
    run(library)
    selection_formatting(library)
    table_navigation(library)
