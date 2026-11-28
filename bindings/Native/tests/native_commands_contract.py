#!/usr/bin/env python3
"""Exercise command and IME-context transport against the actual native worker."""

import argparse
from io import BytesIO
from pathlib import Path
import sys
from xml.dom import minidom
from zipfile import ZipFile

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
        unchanged = {name: empty["formatting"][name] for name in ("fontNames", "textColor")}
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
            **unchanged, "bold": uniform(True), "fontSize": uniform(1900), "alignment": "right"}

        whole = {"anchor": end, "focus": head}
        accepted(op="setSelection", target=target, selection=whole)
        assert send(**{**query, "selection": rest})["code"] == "invalidArgument"
        assert send(**{**query, "selection": whole, "fontGeneration": font + 1})["code"] == "invalidArgument"
        stale = {**target, "revision": target["revision"] - 1}
        assert send(**{**query, "selection": whole, "target": stale})["code"] == "staleRevision"
        before = accepted(op="info")["info"]
        assert accepted(**{**query, "selection": whole})["formatting"] == {
            **unchanged, "bold": uniform(True), "fontSize": mixed, "alignment": "right"}
        assert accepted(op="info")["info"] == before
        accepted(op="setSelection", target=target, selection=first)
        result = accepted(op="executeCommand", target=target, command={
            "command": "formatCharacters", "bold": False})
        target["revision"] = result["outcome"]["revision"]
        accepted(op="setSelection", target=target, selection=whole)
        assert accepted(**{**query, "selection": whole})["formatting"] == {
            **unchanged, "bold": mixed, "fontSize": mixed, "alignment": "right"}

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


def character_font_color_formatting(library):
    def uniform(value):
        return {"kind": "uniform", "value": value}

    def point(offset):
        return {"section": 0, "paragraph": 0, "scalarOffset": offset, "cellPath": []}

    start = {"anchor": point(0), "focus": point(0)}
    prefix = {"anchor": point(0), "focus": point(2)}
    whole = {"anchor": point(3), "focus": point(0)}
    family = "RHWP 검증 글꼴"
    color = "#126abc"
    expected_fonts = [uniform(family)] * 7

    with NativeWorker(library) as worker:
        def send(**command):
            return worker.wait(worker.submit(command))

        def accepted(**command):
            result = send(**command)
            assert result["ok"], result
            return result

        capabilities = worker.host.call("capabilities")
        assert capabilities["characterFontColorFormatting"]
        info = accepted(op="new", replace=None)["info"]
        target = {key: info[key] for key in ("documentId", "revision")}
        if capabilities["systemFontFamilies"]:
            fonts = accepted(op="systemFontFamilies", target=target)
            assert fonts["kind"] == "systemFontFamilies" and fonts["target"] == target
            assert fonts["families"] == sorted(set(fonts["families"]))
            assert all(isinstance(name, str) and name for name in fonts["families"])
            assert send(op="systemFontFamilies", target={**target, "revision": 1})["code"] == "staleRevision"
        assert accepted(op="info")["info"] == info
        accepted(op="beginEdit", target=target, selection=start)
        outcome = accepted(op="replace", target=target, text="한Aa")["outcome"]
        target["revision"] = outcome["revision"]

        def formatting(selection):
            accepted(op="setSelection", target=target, selection=selection)
            return accepted(op="selectionFormatting", target=target, selection=selection,
                            fontGeneration=info["fontGeneration"])["formatting"]

        def apply(**properties):
            result = send(op="executeCommand", target=target,
                          command={"command": "formatCharacters", **properties})
            if result["ok"]:
                target["revision"] = result["outcome"]["revision"]
            return result

        before = formatting(prefix)
        unchanged = accepted(op="info")["info"]
        for invalid_color in ("#12abé", "#+1ff00", "transparent", "#123"):
            assert apply(textColor=invalid_color)["code"] == "invalidArgument"
        assert apply(fontName="\x00Bad")["code"] == "invalidArgument"
        assert apply(fontName="f" * 65)["code"] == "invalidArgument"
        assert accepted(op="info")["info"] == unchanged
        edited = apply(fontName=family, textColor=color.upper())
        assert edited["ok"] and edited["outcome"]["historyChanged"], edited
        changed = formatting(prefix)
        assert changed == {**before, "fontNames": expected_fonts, "textColor": uniform(color)}

        no_op = apply(fontName=family.lower(), textColor=color)
        assert no_op["ok"] and not no_op["outcome"]["historyChanged"], no_op
        assert changed == formatting(prefix)
        mixed = formatting(whole)
        assert mixed["fontNames"] == [{"kind": "mixed"}] * 7, mixed
        assert mixed["textColor"] == {"kind": "mixed"}, mixed
        target["revision"] = accepted(op="undo", target=target)["outcome"]["revision"]
        assert formatting(prefix) == before
        target["revision"] = accepted(op="redo", target=target)["outcome"]["revision"]
        assert formatting(prefix) == changed

        for save_format in ("hwp", "hwpx"):
            prepared = accepted(op="prepareSave", target=target, format=save_format)
            saved = worker.host.copy_buffer(prepared)
            accepted(op="cancelSave", documentId=target["documentId"], token=prepared["token"])
            reopened = worker.host.open_bytes(saved)
            try:
                reopened_info = worker.host.request(reopened, op="info")
                revision = reopened_info["revision"]
                worker.host.request(reopened, op="beginEdit", expectedRevision=revision,
                                    selection=prefix)
                direct = worker.host.request(reopened, op="selectionFormatting", selection=prefix,
                                             expectedRevision=revision,
                                             fontGeneration=reopened_info["fontGeneration"])["formatting"]
                assert direct["fontNames"] == expected_fonts and direct["textColor"] == uniform(color), direct
                if capabilities["systemFontFamilies"]:
                    listed = worker.host.request(reopened, op="systemFontFamilies", expectedRevision=revision)
                    assert listed["families"] == fonts["families"], listed
            finally:
                worker.host.call("close", reopened)

        # The same ID means a different name in different HWP language tables.
        split_family = "RHWP Latin 검증"
        split_fonts = [*expected_fonts]
        split_fonts[1] = uniform(split_family)
        fixture = BytesIO()
        with ZipFile(BytesIO(saved)) as original, ZipFile(fixture, "w") as variant:
            for entry in original.infolist():
                data = original.read(entry)
                if entry.filename == "Contents/header.xml":
                    xml = minidom.parseString(data)
                    slot = next(element for element in xml.getElementsByTagNameNS("*", "fontface")
                                if element.getAttribute("lang") == "LATIN")
                    face = next(element for element in slot.getElementsByTagNameNS("*", "font")
                                if element.getAttribute("face") == family)
                    face.setAttribute("face", split_family)
                    data = xml.toxml(encoding="utf-8")
                variant.writestr(entry, data)
        reopened = worker.host.open_bytes(fixture.getvalue())
        try:
            reopened_info = worker.host.request(reopened, op="info")
            revision = reopened_info["revision"]
            worker.host.request(reopened, op="beginEdit", expectedRevision=revision, selection=prefix)
            direct = worker.host.request(reopened, op="selectionFormatting", selection=prefix,
                                         expectedRevision=revision,
                                         fontGeneration=reopened_info["fontGeneration"])["formatting"]
            assert direct["fontNames"] == split_fonts, direct
        finally:
            worker.host.call("close", reopened)

        accepted(op="setSelection", target=target, selection=start)
        table = accepted(op="executeCommand", target=target, command={
            "command": "createTable", "rows": 1, "columns": 1})["outcome"]
        target["revision"] = table["revision"]
        cell_start = table["selection"]["focus"]
        assert cell_start["cellPath"]
        cell_text = accepted(op="replace", target=target, text="셀 B")["outcome"]
        target["revision"] = cell_text["revision"]
        cell_selection = {"anchor": cell_start, "focus": cell_text["selection"]["focus"]}
        accepted(op="setSelection", target=target, selection=cell_selection)
        cell_edit = apply(fontName=family, textColor=color)
        assert cell_edit["ok"] and cell_edit["outcome"]["historyChanged"], cell_edit
        cell_formatting = formatting(cell_selection)
        assert cell_formatting["fontNames"] == expected_fonts, cell_formatting
        assert cell_formatting["textColor"] == uniform(color), cell_formatting


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


def table_creation(library):
    body = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}

    def collapsed(point):
        return {"anchor": point, "focus": point}

    def check_first_cell(outcome, revision):
        assert outcome["documentChanged"] and outcome["historyChanged"], outcome
        assert outcome["revision"] == revision + 1, outcome
        selection = outcome["selection"]
        cell = selection["focus"]
        assert selection == collapsed(cell), selection
        assert cell["section"] == 0 and cell["scalarOffset"] == 0, cell
        assert len(cell["cellPath"]) == 1, cell
        assert cell["cellPath"][0]["cell"] == cell["cellPath"][0]["paragraph"] == 0, cell
        return cell

    with NativeWorker(library) as worker:
        def send(**command):
            return worker.wait(worker.submit(command))

        def accepted(**command):
            result = send(**command)
            assert result["ok"], result
            return result

        def create(rows, columns):
            return send(op="executeCommand", target=target,
                        command={"command": "createTable", "rows": rows, "columns": columns})

        caps = worker.host.call("capabilities")
        assert caps["createTable"] is True, caps
        assert (caps["maxCreateTableRows"], caps["maxCreateTableColumns"],
                caps["maxCreateTableCells"]) == (256, 256, 4096), caps
        info = accepted(op="new", replace=None)["info"]
        target = {"documentId": info["documentId"], "revision": info["revision"]}
        accepted(op="beginEdit", target=target, selection=collapsed(body))
        before = accepted(op="info")["info"]
        for rows, columns, code in ((0, 2, "invalidArgument"),
                                    (2, 257, "invalidArgument"),
                                    (256, 256, "limitExceeded")):
            rejected = create(rows, columns)
            assert not rejected["ok"] and rejected["code"] == code, rejected
        assert accepted(op="info")["info"] == before

        created = create(2, 3)
        assert created["ok"] and created["kind"] == "command", created
        assert created["preferredX"] is None, created
        outcome = created["outcome"]
        first = check_first_cell(outcome, target["revision"])
        target["revision"] = outcome["revision"]
        # A new document's first paragraph also owns its existing section controls.
        assert first["paragraph"] == 0 and first["cellPath"][0]["control"] > 0, first
        caret = accepted(op="caretRect", target=target, position=first)["caret"]
        assert caret["height"] > 0 and caret["cellBounds"] is not None, caret
        rejected = create(1, 1)
        assert not rejected["ok"] and rejected["code"] == "unsupported", rejected
        undone = accepted(op="undo", target=target)["outcome"]
        target["revision"] = undone["revision"]
        assert undone["selection"] == collapsed(body), undone
        assert not accepted(op="info")["info"]["canUndo"]
        assert accepted(op="imeContext", target=target)["context"]["text"] == ""

        text = "왼😀오른"
        typed = accepted(op="replace", target=target, text=text)["outcome"]
        target["revision"] = typed["revision"]
        middle = {**body, "scalarOffset": 2}
        range_selection = {"anchor": body, "focus": middle}
        accepted(op="setSelection", target=target, selection=range_selection)
        rejected = create(2, 2)
        assert not rejected["ok"] and rejected["code"] == "invalidArgument", rejected
        assert accepted(op="info")["info"]["revision"] == target["revision"]
        accepted(op="setSelection", target=target, selection=collapsed(middle))

        split = create(2, 2)
        assert split["ok"], split
        outcome = split["outcome"]
        first = check_first_cell(outcome, target["revision"])
        target["revision"] = outcome["revision"]
        assert first["paragraph"] == body["paragraph"] + 1, first
        right = {**body, "paragraph": first["paragraph"] + 2}
        for point, expected in ((body, "왼😀"), (right, "오른")):
            accepted(op="setSelection", target=target, selection=collapsed(point))
            assert accepted(op="imeContext", target=target)["context"]["text"] == expected
        undone = accepted(op="undo", target=target)["outcome"]
        target["revision"] = undone["revision"]
        assert undone["selection"] == collapsed(middle), undone
        assert accepted(op="imeContext", target=target)["context"]["text"] == text

        handle = worker.host.open_bytes()
        try:
            revision = worker.host.request(handle, op="info")["revision"]
            worker.host.request(handle, op="beginEdit", expectedRevision=revision,
                                selection=collapsed(body))
            direct = worker.host.request(handle, op="executeCommand", expectedRevision=revision,
                                         command={"command": "createTable", "rows": 1, "columns": 1})
            first = check_first_cell(direct["outcome"], revision)
            revision = direct["outcome"]["revision"]
            caret = worker.host.request(handle, op="caret", position=first)["caret"]
            assert caret["height"] > 0 and caret["cellBounds"] is not None, caret
            direct_undo = worker.host.request(handle, op="undo", expectedRevision=revision)["outcome"]
            assert direct_undo["selection"] == collapsed(body), direct_undo
            assert not worker.host.request(handle, op="info")["canUndo"]
        finally:
            worker.host.call("close", handle)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    library = parser.parse_args().library
    run(library)
    selection_formatting(library)
    character_font_color_formatting(library)
    table_navigation(library)
    table_creation(library)
