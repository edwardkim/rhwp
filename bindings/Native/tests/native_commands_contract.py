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


def select_text_unit(unit, position):
    return {"command": "selectTextUnit", "unit": unit, "position": position}


def text_unit_selection(library):
    point = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}
    caret = {"anchor": point, "focus": point}
    text = "한글,  Cafe\u0301!  👩\u200d💻 끝."
    with NativeWorker(library) as worker:
        def send(**command):
            return worker.wait(worker.submit(command))

        def accepted(**command):
            result = send(**command)
            assert result["ok"], result
            return result

        def select(unit, offset, start, end, expected):
            before = accepted(op="info")["info"]
            command = select_text_unit(unit, {**point, "scalarOffset": offset})
            selected = accepted(op="executeCommand", target=target, command=command)
            selection = {"anchor": {**point, "scalarOffset": start},
                         "focus": {**point, "scalarOffset": end}}
            assert selected["kind"] == "command" and selected["preferredX"] is None, selected
            assert selected["outcome"] == {
                "revision": target["revision"], "selection": selection, "documentChanged": False,
                "historyChanged": False, "compositionId": None,
            }, selected
            assert accepted(op="imeContext", target=target)["context"]["selectedText"] == expected
            assert accepted(op="info")["info"] == {**before, "selection": selection}

        assert worker.host.call("capabilities")["textUnitSelection"] is True
        info = accepted(op="new", replace=None)["info"]
        target = {key: info[key] for key in ("documentId", "revision")}
        word = select_text_unit("word", point)
        assert send(op="executeCommand", target=target, command=word)["code"] == "unsupported"
        accepted(op="beginEdit", target=target, selection=caret)
        select("sentence", 0, 0, 0, "")
        for inserted in (text, "X"):
            target["revision"] = accepted(op="replace", target=target, text=inserted)["outcome"]["revision"]
        target["revision"] = accepted(op="undo", target=target)["outcome"]["revision"]
        info = accepted(op="info")["info"]
        assert info["canUndo"] and info["canRedo"] and info["isDirty"], info
        for unit, offset, start, end, selected in (
            ("word", 1, 0, 2, "한글"),
            ("word", 2, 2, 3, ","),
            ("word", 3, 2, 3, ","),
            ("word", 4, 4, 4, ""),
            ("word", 9, 5, 10, "Cafe\u0301"),
            ("word", 14, 13, 16, "👩\u200d💻"),
            ("word", 19, 18, 19, "."),
            ("sentence", 12, 0, 11, "한글,  Cafe\u0301!"),
            ("sentence", 13, 13, 19, "👩\u200d💻 끝."),
            ("sentence", 19, 13, 19, "👩\u200d💻 끝."),
        ):
            select(unit, offset, start, end, selected)

        stale = {**target, "revision": target["revision"] - 1}
        assert send(op="executeCommand", target=stale, command=word)["code"] == "staleRevision"
        before = accepted(op="info")["info"]
        invalid = select_text_unit("word", {**point, "scalarOffset": len(text) + 1})
        assert send(op="executeCommand", target=target, command=invalid)["code"] == "invalidArgument"
        assert accepted(op="info")["info"] == before
        composition = accepted(op="beginComposition", target=target)["outcome"]["compositionId"]
        before = accepted(op="info")["info"]
        assert send(op="executeCommand", target=target, command=word)["code"] == "invalidArgument"
        assert accepted(op="info")["info"] == before
        accepted(op="cancelComposition", target=target, compositionId=composition)

        handle = worker.host.open_bytes()
        try:
            revision = worker.host.request(handle, op="info")["revision"]
            worker.host.request(handle, op="beginEdit", expectedRevision=revision, selection=caret)
            edited = worker.host.request(handle, op="replaceSelection", expectedRevision=revision, text=text)
            revision = edited["outcome"]["revision"]
            selected = worker.host.request(handle, op="executeCommand", expectedRevision=revision,
                                           command=select_text_unit("sentence", point))
            assert selected["preferredX"] is None
            assert selected["outcome"]["revision"] == revision
            assert selected["outcome"]["selection"] == {
                "anchor": point, "focus": {**point, "scalarOffset": 11},
            }, selected
        finally:
            worker.host.call("close", handle)


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
        cell_family = "RHWP 표 글꼴"
        cell_color = "#34904d"
        cell_edit = apply(fontName=cell_family, textColor=cell_color)
        assert cell_edit["ok"] and cell_edit["outcome"]["historyChanged"], cell_edit
        cell_formatting = formatting(cell_selection)
        assert cell_formatting["fontNames"] == [uniform(cell_family)] * 7, cell_formatting
        assert cell_formatting["textColor"] == uniform(cell_color), cell_formatting


def page_setup(library):
    with NativeWorker(library) as worker:
        def send(**command):
            return worker.wait(worker.submit(command))

        def accepted(**command):
            result = send(**command)
            assert result["ok"], result
            return result

        def read(**location):
            response = accepted(op="pageSetup", target=target, **location)
            assert response["kind"] == "pageSetup" and response["target"] == target, response
            return response["pageSetup"]

        def apply(settings, section=0):
            return send(op="executeCommand", target=target, command={
                "command": "setPageSetup", "section": section, "settings": settings,
            })

        assert worker.host.call("capabilities")["pageSetup"]
        info = accepted(op="new", replace=None)["info"]
        target = {"documentId": info["documentId"], "revision": info["revision"]}
        initial = read(page=0)
        assert initial["section"] == 0 and initial == read(section=0), initial
        assert initial["settings"]["width"] == 59528 and initial["settings"]["height"] == 84186, initial
        assert accepted(op="info")["info"] == info
        start = initial["startPosition"]
        assert start == {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}, start
        for locator in ({}, {"page": 0, "section": 0}, {"page": 1}, {"section": 1}):
            error = send(op="pageSetup", target=target, **locator)
            assert not error["ok"] and error["code"] == "invalidArgument", error

        settings = {**initial["settings"], "width": 67500, "height": 90000, "landscape": True,
                    "marginLeft": 5000, "marginRight": 6000, "marginTop": 7000,
                    "marginBottom": 3500, "marginHeader": 2200, "marginFooter": 1800,
                    "marginGutter": 850, "binding": 1}
        selection = {"anchor": start, "focus": start}
        accepted(op="beginEdit", target=target, selection=selection)
        before = accepted(op="info")["info"]
        old_size = accepted(op="pageSize", target=target, page=0)["size"]
        for invalid in ({"width": 0}, {"height": 288001}, {"binding": 3},
                        {"marginGutter": 2**32 - 1}, {"marginFooter": settings["width"]}):
            rejected = apply({**settings, **invalid})
            assert not rejected["ok"] and rejected["code"] == "invalidArgument", rejected
        bad_section = apply(settings, section=1)
        assert not bad_section["ok"] and bad_section["code"] == "invalidArgument", bad_section
        unchanged = apply(initial["settings"])["outcome"]
        assert not unchanged["documentChanged"] and not unchanged["historyChanged"], unchanged
        assert accepted(op="info")["info"] == before and read(page=0) == initial

        stale = target.copy()
        changed = apply(settings)
        assert changed["ok"] and changed["kind"] == "command", changed
        outcome = changed["outcome"]
        assert outcome["selection"] == selection and outcome["revision"] == target["revision"] + 1, outcome
        assert outcome["documentChanged"] and outcome["historyChanged"], outcome
        target["revision"] = outcome["revision"]
        assert read(page=0)["settings"] == settings
        assert accepted(op="pageSize", target=target, page=0)["size"] == {"width": 1200.0, "height": 900.0}
        assert send(op="pageSetup", target=stale, page=0)["code"] == "staleRevision"

        for file_format in ("hwp", "hwpx"):
            prepared = accepted(op="prepareSave", target=target, format=file_format)
            handle = worker.host.open_bytes(worker.copy_buffer(prepared))
            try:
                direct = worker.host.request(handle, op="pageSetup", expectedRevision=0, page=0)
                assert direct["kind"] == "pageSetup" and direct["revision"] == 0, direct
                assert direct["pageSetup"]["settings"] == settings, direct
            finally:
                worker.host.call("close", handle)
            accepted(op="cancelSave", documentId=target["documentId"], token=prepared["token"])

        undone = accepted(op="undo", target=target)["outcome"]
        target["revision"] = undone["revision"]
        assert undone["selection"] == selection and read(section=0) == initial, undone
        assert not accepted(op="info")["info"]["canUndo"]
        assert accepted(op="pageSize", target=target, page=0)["size"] == old_size
        redone = accepted(op="redo", target=target)["outcome"]
        target["revision"] = redone["revision"]
        assert redone["selection"] == selection and read(section=0)["settings"] == settings, redone

        handle = worker.host.open_bytes()
        try:
            worker.host.request(handle, op="beginEdit", expectedRevision=0, selection=selection)
            direct = worker.host.request(handle, op="executeCommand", expectedRevision=0,
                                         command={"command": "setPageSetup", "section": 0, "settings": settings})
            revision = direct["outcome"]["revision"]
            read_back = worker.host.request(handle, op="pageSetup", expectedRevision=revision, section=0)
            assert read_back["pageSetup"]["settings"] == settings, read_back
        finally:
            worker.host.call("close", handle)

        # Duplicate the existing blank section in memory. Applying settings to
        # its page must keep the original section's paper and selection intact.
        fixture = Path(__file__).resolve().parents[3] / "saved/blank_hwpx.hwpx"
        package = BytesIO()
        with ZipFile(fixture) as source, ZipFile(package, "w") as result:
            second_section = source.read("Contents/section0.xml")
            for entry in source.infolist():
                data = source.read(entry)
                if entry.filename == "Contents/header.xml":
                    data = data.replace(b'secCnt="1"', b'secCnt="2"')
                if entry.filename == "Contents/content.hpf":
                    data = data.replace(b"</opf:manifest>", b'<opf:item id="section1" '
                                        b'href="Contents/section1.xml" media-type="application/xml"/>'
                                        b"</opf:manifest>")
                    data = data.replace(b"</opf:spine>", b'<opf:itemref idref="section1"/></opf:spine>')
                result.writestr(entry, data)
            result.writestr("Contents/section1.xml", second_section)

        opened = worker.wait(worker.open_bytes(package.getvalue(), replace=target))
        assert opened["ok"] and opened["info"]["pageCount"] == 2, opened
        info = opened["info"]
        target = {"documentId": info["documentId"], "revision": info["revision"]}
        first, second = read(page=0), read(page=1)
        assert first["section"] == 0 and second["section"] == 1
        assert second == read(section=1) and second["startPosition"]["section"] == 1
        accepted(op="beginEdit", target=target, selection=selection)
        changed = apply(settings, section=second["section"])
        assert changed["ok"] and changed["outcome"]["selection"] == selection, changed
        target["revision"] = changed["outcome"]["revision"]
        assert read(page=0) == first and read(page=1)["settings"] == settings


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
        for unit in ("word", "sentence"):
            selected = worker.wait(worker.submit({
                "op": "executeCommand", "target": target, "command": select_text_unit(unit, point(4, 2)),
            }))
            assert selected["ok"] and selected["outcome"]["selection"] == {
                "anchor": point(4), "focus": point(4, 4),
            }, selected


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
        caption = {**first, "cellPath": [{**first["cellPath"][0], "cell": 65534}]}
        before = accepted(op="info")["info"]
        rejected = send(op="executeCommand", target=target, command=select_text_unit("word", caption))
        assert not rejected["ok"] and rejected["code"] == "unsupported", rejected
        assert accepted(op="info")["info"] == before
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
            edited = worker.host.request(handle, op="replaceSelection", expectedRevision=revision,
                                         text="셀 내용! Next.")
            revision = edited["outcome"]["revision"]
            selected = worker.host.request(handle, op="executeCommand", expectedRevision=revision,
                                           command=select_text_unit("sentence", first))["outcome"]
            assert selected["selection"] == {
                "anchor": first, "focus": {**first, "scalarOffset": 5},
            }, selected
            assert selected["revision"] == revision and not selected["historyChanged"], selected
            removed = worker.host.request(handle, op="undo", expectedRevision=revision)["outcome"]
            revision = removed["revision"]
            direct_undo = worker.host.request(handle, op="undo", expectedRevision=revision)["outcome"]
            assert direct_undo["selection"] == collapsed(body), direct_undo
            assert not worker.host.request(handle, op="info")["canUndo"]
        finally:
            worker.host.call("close", handle)


def table_object_properties(library):
    def call(worker, **command):
        reply = worker.wait(worker.submit(command))
        assert reply["ok"], reply
        return reply

    def document_target(info):
        return {key: info[key] for key in ("documentId", "revision")}

    def locate_table(worker, target):
        snapshot = call(worker, op="accessibility", target=target, request={})["snapshot"]
        point, = [node["start"] for node in snapshot["nodes"] if node["text"] == "표😀"]
        return point

    def update_hwpx(data, edit):
        output = BytesIO()
        with ZipFile(BytesIO(data)) as source, ZipFile(output, "w") as document:
            for entry in source.infolist():
                contents = source.read(entry)
                if entry.filename == "Contents/section0.xml":
                    root = minidom.parseString(contents)
                    edit(root)
                    contents = root.toxml(encoding="UTF-8")
                document.writestr(entry, contents)
        return output.getvalue()

    with NativeWorker(library) as worker:
        assert worker.host.call("capabilities")["tableObjectProperties"]
        target = document_target(call(worker, op="new", replace=None)["info"])
        body = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}
        call(worker, op="beginEdit", target=target, selection={"anchor": body, "focus": body})
        created = call(worker, op="executeCommand", target=target, command={
            "command": "createTable", "rows": 2, "columns": 3,
        })["outcome"]
        target["revision"] = created["revision"]
        position = created["selection"]["focus"]
        typed = call(worker, op="replace", target=target, text="표😀")["outcome"]
        target["revision"] = typed["revision"]
        page = call(worker, op="caretRect", target=target, position=position)["caret"]["page"]

        def table():
            reply = call(worker, op="tableObject", target=target, position=position, page=page)
            assert reply["kind"] == "tableObject" and reply["target"] == target and reply["page"] == page
            return reply["table"]

        baseline = table()
        address = baseline["address"]
        assert address == {"section": position["section"], "paragraph": position["paragraph"],
                           "control": position["cellPath"][0]["control"]}
        assert baseline["editable"] and baseline["canResize"]
        assert (baseline["rowCount"], baseline["columnCount"]) == (2, 3)
        assert baseline["bounds"]["width"] > 0 and baseline["bounds"]["height"] > 0
        properties = baseline["properties"]

        def apply(command, **payload):
            outcome = call(worker, op="executeCommand", target=target,
                           command={"command": command, "table": address, **payload})["outcome"]
            target["revision"] = outcome["revision"]
            assert outcome["selection"] == typed["selection"], outcome
            return outcome

        unchanged = apply("setTableObjectProperties", properties={"paddingTop": properties["paddingTop"]})
        assert not unchanged["documentChanged"]
        changed_properties = {"outerLeft": 500, "paddingTop": 250, "cellSpacing": 100,
                              "repeatHeader": True, "pageBreak": 2}
        initial_revision = target["revision"]
        outcome = apply("setTableObjectProperties", properties=changed_properties)
        assert outcome["documentChanged"] and outcome["historyChanged"]
        assert target["revision"] == initial_revision + 1
        before_resize = {**properties, **changed_properties}
        assert table()["properties"] == before_resize
        width, height = properties["tableWidth"] - 5000, properties["tableHeight"] + 3500
        outcome = apply("resizeTableObject", width=width, height=height)
        assert outcome["documentChanged"] and target["revision"] == initial_revision + 2
        expected = {**before_resize, "tableWidth": width, "tableHeight": height}
        resized = table()
        assert resized["properties"] == expected, resized
        assert resized["bounds"]["width"] < baseline["bounds"]["width"], resized
        assert resized["bounds"]["height"] > baseline["bounds"]["height"], resized
        assert call(worker, op="imeContext", target=target)["context"]["text"] == "표😀"

        for command in (
            {"command": "resizeTableObject", "table": address, "width": 1},
            {"command": "setTableObjectProperties", "table": address, "properties": {"pageBreak": 3}},
            {"command": "setTableObjectProperties", "table": {**address, "control": address["control"] + 1},
             "properties": {"outerLeft": 0}},
        ):
            result = worker.wait(worker.submit({"op": "executeCommand", "target": target, "command": command}))
            assert not result["ok"] and result["code"] == "invalidArgument", result
            assert table()["properties"] == expected
            assert call(worker, op="info")["info"]["revision"] == target["revision"]

        target["revision"] = call(worker, op="undo", target=target)["outcome"]["revision"]
        assert table()["properties"] == before_resize
        target["revision"] = call(worker, op="undo", target=target)["outcome"]["revision"]
        assert table()["properties"] == properties
        for _ in range(2):
            target["revision"] = call(worker, op="redo", target=target)["outcome"]["revision"]
        assert table()["properties"] == expected

        for file_format in ("hwp", "hwpx"):
            saved = call(worker, op="prepareSave", target=target, format=file_format)
            assert saved["report"]["count"] == 0, saved
            data = worker.copy_buffer(saved)
            call(worker, op="cancelSave", documentId=target["documentId"], token=saved["token"])
            with NativeWorker(library) as verifier:
                opened = verifier.wait(verifier.open_bytes(data))
                assert opened["ok"], opened
                reopened = document_target(opened["info"])
                point = locate_table(verifier, reopened)
                assert call(verifier, op="tableObject", target=reopened, position=point,
                            page=page)["table"]["properties"] == expected
                handle = verifier.host.open_bytes(data)
                try:
                    direct = verifier.host.request(handle, op="tableObject", expectedRevision=0,
                                                   position=point, page=page)
                    assert direct["table"]["properties"] == expected, direct
                finally:
                    verifier.host.call("close", handle)

        with NativeWorker(library) as verifier:
            protected_data = update_hwpx(
                data, lambda root: root.getElementsByTagName("hp:tc")[1].setAttribute("protect", "1"))
            opened = verifier.wait(verifier.open_bytes(protected_data))
            assert opened["ok"], opened
            reopened = document_target(opened["info"])
            point = locate_table(verifier, reopened)
            protected = call(verifier, op="tableObject", target=reopened, position=point, page=page)["table"]
            assert not protected["editable"] and not protected["canResize"], protected
            call(verifier, op="beginEdit", target=reopened, selection={"anchor": point, "focus": point})
            for command in ({"command": "resizeTableObject", "width": width + 100},
                            {"command": "setTableObjectProperties", "properties": {"outerLeft": 0}}):
                result = verifier.wait(verifier.submit({
                    "op": "executeCommand", "target": reopened,
                    "command": {"table": protected["address"], **command},
                }))
                assert not result["ok"] and result["code"] == "unsupported", result
            assert call(verifier, op="info")["info"]["revision"] == reopened["revision"]

        def small_column(root):
            table_element = root.getElementsByTagName("hp:tbl")[0]
            table_element.getElementsByTagName("hp:sz")[0].setAttribute("width", "20200")
            for cell in table_element.getElementsByTagName("hp:tc"):
                column = cell.getElementsByTagName("hp:cellAddr")[0].getAttribute("colAddr")
                cell.getElementsByTagName("hp:cellSz")[0].setAttribute("width", "200" if column == "0" else "10000")

        with NativeWorker(library) as verifier:
            opened = verifier.wait(verifier.open_bytes(update_hwpx(data, small_column)))
            assert opened["ok"], opened
            reopened = document_target(opened["info"])
            point = locate_table(verifier, reopened)
            call(verifier, op="beginEdit", target=reopened, selection={"anchor": point, "focus": point})
            small = call(verifier, op="tableObject", target=reopened, position=point, page=page)["table"]
            assert small["canResize"] and small["properties"]["tableWidth"] == 20200, small
            changed = call(verifier, op="executeCommand", target=reopened, command={
                "command": "resizeTableObject", "table": small["address"], "width": 20100,
            })["outcome"]
            reopened["revision"] = changed["revision"]
            resized_small = call(verifier, op="tableObject", target=reopened, position=point, page=page)["table"]
            assert resized_small["properties"]["tableWidth"] == 20100

        def one_row(root):
            table_element = root.getElementsByTagName("hp:tbl")[0]
            table_element.setAttribute("rowCnt", "1")
            table_element.getElementsByTagName("hp:sz")[0].setAttribute("height", "5000")
            table_element.removeChild(table_element.getElementsByTagName("hp:tr")[1])
            for size in table_element.getElementsByTagName("hp:cellSz"):
                size.setAttribute("height", "2000")

        with NativeWorker(library) as verifier:
            opened = verifier.wait(verifier.open_bytes(update_hwpx(data, one_row)))
            assert opened["ok"], opened
            reopened = document_target(opened["info"])
            point = locate_table(verifier, reopened)
            call(verifier, op="beginEdit", target=reopened, selection={"anchor": point, "focus": point})
            single = call(verifier, op="tableObject", target=reopened, position=point, page=page)["table"]
            assert single["canResize"] and single["properties"]["tableHeight"] == 5000, single
            changed = call(verifier, op="executeCommand", target=reopened, command={
                "command": "resizeTableObject", "table": single["address"], "height": 2000,
            })["outcome"]
            reopened["revision"] = changed["revision"]
            resized_single = call(verifier, op="tableObject", target=reopened, position=point, page=page)["table"]
            assert resized_single["properties"]["tableHeight"] == 2000
    print("PASS: whole-table bounds, protected properties, resize, undo/redo and HWP/HWPX reopen")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    library = parser.parse_args().library
    run(library)
    text_unit_selection(library)
    selection_formatting(library)
    character_font_color_formatting(library)
    table_navigation(library)
    table_creation(library)
    page_setup(library)
    table_object_properties(library)
