#!/usr/bin/env python3
"""Exercise paragraph styles through both Native request surfaces."""

import argparse
from io import BytesIO
from pathlib import Path
import sys
from xml.etree import ElementTree
from zipfile import ZipFile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_host import NativeCallError
from native_worker import NativeWorker


NS = {"hp": "http://www.hancom.co.kr/hwpml/2011/paragraph"}


def selection_at(offset=0, anchor_offset=None):
    position = {"section": 0, "paragraph": 0, "scalarOffset": offset, "cellPath": []}
    anchor = dict(position, scalarOffset=anchor_offset) if anchor_offset is not None else position
    return {"anchor": anchor, "focus": position}


def paragraph_snapshot(hwpx, cell=False):
    with ZipFile(BytesIO(hwpx)) as package:
        section = ElementTree.fromstring(package.read("Contents/section0.xml"))
    path = ".//hp:tc/hp:subList/hp:p" if cell else "hp:p"
    paragraph = section.find(path, NS)
    runs = [
        (run.attrib["charPrIDRef"], text.text)
        for run in paragraph.findall("hp:run", NS)
        for text in run.findall("hp:t", NS)
        if text.text
    ]
    return paragraph.attrib["styleIDRef"], paragraph.attrib["paraPrIDRef"], runs


def run(library):
    with NativeWorker(library) as worker:
        def send(**command):
            return worker.wait(worker.submit(command))

        def accepted(**command):
            result = send(**command)
            assert result["ok"], result
            return result

        def edit(**command):
            try:
                return send(op="executeCommand", target=target, command=command)
            except NativeCallError as error:
                return error.response

        def advance(result):
            assert result["ok"], result
            outcome = result["outcome"]
            assert outcome["documentChanged"] and outcome["historyChanged"], outcome
            assert outcome["revision"] == target["revision"] + 1, outcome
            target["revision"] = outcome["revision"]
            return outcome["selection"]

        def styles(selection):
            result = accepted(op="paragraphStyles", target=target, selection=selection)
            assert result["kind"] == "paragraphStyles" and result["target"] == target, result
            assert result["selection"] == selection, result
            return result

        def save(file_format="hwpx"):
            prepared = accepted(op="prepareSave", target=target, format=file_format)
            content = worker.copy_buffer(prepared)
            accepted(op="cancelSave", documentId=target["documentId"], token=prepared["token"])
            return content

        opened = accepted(op="new", replace=None)["info"]
        target = {"documentId": opened["documentId"], "revision": opened["revision"]}
        accepted(op="beginEdit", target=target, selection=selection_at())
        advance(accepted(op="replace", target=target, text="앞😀뒤"))
        accepted(op="setSelection", target=target, selection=selection_at(0, 1))
        advance(edit(command="formatCharacters", bold=True))

        backwards = selection_at(0, 3)
        accepted(op="setSelection", target=target, selection=backwards)
        before = accepted(op="info")["info"]
        catalog = styles(backwards)
        assert worker.host.call("capabilities")["paragraphStyles"]
        assert catalog["currentStyleId"] == 0, catalog
        names = {style["id"]: style["name"] for style in catalog["styles"]}
        assert names[0] == "바탕글" and names[17] == "차례 제목" and 12 not in names, names
        assert list(names) == sorted(names), names
        assert send(op="paragraphStyles", target=target, selection=selection_at())["code"] == "invalidArgument"
        stale = dict(target, revision=target["revision"] - 1)
        assert send(op="paragraphStyles", target=stale, selection=backwards)["code"] == "staleRevision"
        assert send(op="executeCommand", target=stale,
                    command={"command": "applyParagraphStyle", "styleId": 17})["code"] == "staleRevision"
        composition = accepted(op="beginComposition", target=target)["outcome"]["compositionId"]
        assert send(op="paragraphStyles", target=target, selection=backwards)["code"] == "invalidArgument"
        assert edit(command="applyParagraphStyle", styleId=17)["code"] == "invalidArgument"
        accepted(op="cancelComposition", target=target, compositionId=composition)
        for invalid in (12, 255, 256, -1):
            result = edit(command="applyParagraphStyle", styleId=invalid)
            assert not result["ok"] and result["code"] == "invalidArgument", result
        unchanged = edit(command="applyParagraphStyle", styleId=0)["outcome"]
        assert not unchanged["documentChanged"] and not unchanged["historyChanged"], unchanged
        assert unchanged["selection"] == backwards and accepted(op="info")["info"] == before

        original_style, original_shape, original_runs = paragraph_snapshot(save())
        assert (original_style, original_shape) == ("0", "0")
        assert [text for _, text in original_runs] == ["앞", "😀뒤"], original_runs
        assert original_runs[0][0] != original_runs[1][0], original_runs
        assert advance(edit(command="applyParagraphStyle", styleId=17)) == backwards
        assert styles(backwards)["currentStyleId"] == 17
        formatted = ("17", "12", [original_runs[0], ("5", "😀뒤")])
        assert paragraph_snapshot(save()) == formatted

        handle = worker.host.open_bytes(save("hwp"))
        try:
            worker.host.request(handle, op="beginEdit", expectedRevision=0, selection=backwards)
            result = worker.host.request(handle, op="paragraphStyles", expectedRevision=0, selection=backwards)
            assert result["revision"] == 0 and result["currentStyleId"] == 17, result
            assert result["selection"] == backwards and result["styles"] == catalog["styles"], result
            prepared = worker.host.request(handle, op="prepareSave", format="hwpx")
            assert paragraph_snapshot(worker.host.copy_buffer(prepared)) == formatted
            worker.host.request(handle, op="cancelSave", token=prepared["token"])
            direct = worker.host.request(handle, op="executeCommand", expectedRevision=0,
                                         command={"command": "applyParagraphStyle", "styleId": 0})
            assert direct["outcome"]["revision"] == 1 and direct["outcome"]["selection"] == backwards, direct
        finally:
            worker.host.call("close", handle)

        assert advance(accepted(op="undo", target=target)) == backwards
        assert paragraph_snapshot(save()) == (original_style, original_shape, original_runs)
        assert advance(accepted(op="redo", target=target)) == backwards
        assert paragraph_snapshot(save()) == formatted

        accepted(op="setSelection", target=target, selection=selection_at(3))
        cell = advance(edit(command="createTable", rows=1, columns=1))
        assert len(cell["focus"]["cellPath"]) == 1, cell
        cell = advance(accepted(op="replace", target=target, text="셀"))
        assert advance(edit(command="setParagraphAlignment", alignment="right")) == cell
        cell_style, cell_shape, cell_runs = paragraph_snapshot(save(), cell=True)
        assert str(styles(cell)["currentStyleId"]) == cell_style
        assert cell_style != "1" and advance(edit(command="applyParagraphStyle", styleId=1)) == cell
        assert styles(cell)["currentStyleId"] == 1
        assert paragraph_snapshot(save(), cell=True) == ("1", cell_shape, cell_runs)
        assert advance(accepted(op="undo", target=target)) == cell
        assert paragraph_snapshot(save(), cell=True) == (cell_style, cell_shape, cell_runs)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    run(parser.parse_args().library)
