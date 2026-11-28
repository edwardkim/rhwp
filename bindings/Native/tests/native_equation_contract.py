#!/usr/bin/env python3
"""Exercise displayed equation insertion and file round trips through the native ABI."""

import argparse
from io import BytesIO
from pathlib import Path
import sys
from xml.etree import ElementTree
from zipfile import ZipFile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_worker import NativeWorker


PARAGRAPH_NS = {"hp": "http://www.hancom.co.kr/hwpml/2011/paragraph"}


def point(paragraph, offset=0):
    return {"section": 0, "paragraph": paragraph, "scalarOffset": offset, "cellPath": []}


def caret(paragraph, offset=0):
    return {"anchor": point(paragraph, offset), "focus": point(paragraph, offset)}


def inspect_hwpx(data, texts, script=None, equation_paragraph=None, font_size=None):
    with ZipFile(BytesIO(data)) as package:
        section = ElementTree.fromstring(package.read("Contents/section0.xml"))
    paragraphs = section.findall("hp:p", PARAGRAPH_NS)
    assert ["".join(t.text or "" for t in p.findall(".//hp:t", PARAGRAPH_NS)) for p in paragraphs] == texts
    equations = section.findall(".//hp:equation", PARAGRAPH_NS)
    if script is None:
        assert not equations, equations
        return
    assert len(equations) == 1 and equations[0] in paragraphs[equation_paragraph].iter(), equations
    equation = equations[0]
    assert equation.findtext("hp:script", namespaces=PARAGRAPH_NS) == script
    assert int(equation.attrib["baseUnit"]) == font_size
    size = equation.find("hp:sz", PARAGRAPH_NS)
    assert int(size.attrib["width"]) > 0 and int(size.attrib["height"]) > 0


def run(library):
    with NativeWorker(library) as worker:
        def send(**command):
            return worker.wait(worker.submit(command))

        def accepted(**command):
            result = send(**command)
            assert result["ok"], result
            return result

        def insert(script, **options):
            return send(op="executeCommand", target=target,
                        command={"command": "insertEquation", "script": script, **options})

        def worker_save(file_format):
            prepared = accepted(op="prepareSave", target=target, format=file_format)
            data = worker.copy_buffer(prepared)
            accepted(op="cancelSave", documentId=target["documentId"], token=prepared["token"])
            return data

        def session_save(handle):
            prepared = worker.host.request(handle, op="prepareSave", format="hwpx")
            data = worker.host.copy_buffer(prepared)
            worker.host.request(handle, op="cancelSave", token=prepared["token"])
            return data

        capabilities = worker.host.call("capabilities")
        assert capabilities["insertEquation"]
        assert capabilities["maxEquationScriptBytes"] == 4096
        assert capabilities["minEquationFontSize"] == 200
        assert capabilities["maxEquationFontSize"] == 40000

        opened = accepted(op="new", replace=None)["info"]
        target = {"documentId": opened["documentId"], "revision": opened["revision"]}
        accepted(op="beginEdit", target=target, selection=caret(0))
        prefix, tail = "앞i\u0301😀", "뒤"
        script = "1 over 2\n+ sqrt { x_2 }"
        outcome = accepted(op="replace", target=target, text=prefix + tail)["outcome"]
        target["revision"] = outcome["revision"]
        accepted(op="setSelection", target=target,
                 selection={"anchor": point(0, 1), "focus": point(0, 4)})
        assert insert(script)["code"] == "invalidArgument"
        accepted(op="setSelection", target=target, selection=caret(0, 2))  # inside i + combining acute
        assert insert(script)["code"] == "invalidArgument"

        insertion_point = caret(0, len(prefix))
        accepted(op="setSelection", target=target, selection=insertion_point)
        before = accepted(op="info")["info"]
        for invalid, options, code in (
            ("\t\n ", {}, "invalidArgument"),
            ("x\x00", {}, "invalidArgument"),
            ("가" * 1366, {}, "limitExceeded"),
            (script, {"fontSize": 199}, "invalidArgument"),
            (script, {"fontSize": 40001}, "invalidArgument"),
        ):
            result = insert(invalid, **options)
            assert not result["ok"] and result["code"] == code, result
        composing = accepted(op="beginComposition", target=target)["outcome"]
        assert insert(script)["code"] == "invalidArgument"
        accepted(op="cancelComposition", target=target, compositionId=composing["compositionId"])
        assert accepted(op="info")["info"] == before

        inserted = insert(script.replace("\n", "\r\n"), fontSize=1350)
        assert inserted["ok"] and inserted["kind"] == "command", inserted
        outcome = inserted["outcome"]
        assert outcome["documentChanged"] and outcome["historyChanged"], outcome
        assert outcome["revision"] == target["revision"] + 1 and outcome["selection"] == caret(2), outcome
        target["revision"] = outcome["revision"]
        assert accepted(op="imeContext", target=target)["context"]["text"] == tail
        rectangle = accepted(op="caretRect", target=target, position=point(2))["caret"]
        assert rectangle["height"] > 0, rectangle

        inspect_hwpx(worker_save("hwpx"), [prefix, "", tail], script, 1, 1350)
        hwp = worker_save("hwp")
        assert hwp.startswith(bytes.fromhex("d0cf11e0a1b11ae1"))
        handle = worker.host.open_bytes(hwp)
        try:
            inspect_hwpx(session_save(handle), [prefix, "", tail], script, 1, 1350)
        finally:
            worker.host.call("close", handle)

        undone = accepted(op="undo", target=target)["outcome"]
        assert undone["revision"] == target["revision"] + 1 and undone["selection"] == insertion_point, undone
        target["revision"] = undone["revision"]
        inspect_hwpx(worker_save("hwpx"), [prefix + tail])
        redone = accepted(op="redo", target=target)["outcome"]
        assert redone["revision"] == target["revision"] + 1 and redone["selection"] == caret(2), redone
        target["revision"] = redone["revision"]
        inspect_hwpx(worker_save("hwpx"), [prefix, "", tail], script, 1, 1350)
        changed = accepted(op="replace", target=target, text="끝")
        target["revision"] = changed["outcome"]["revision"]
        inspect_hwpx(worker_save("hwpx"), [prefix, "", "끝" + tail], script, 1, 1350)

        created = accepted(op="executeCommand", target=target,
                           command={"command": "createTable", "rows": 1, "columns": 1})
        target["revision"] = created["outcome"]["revision"]
        assert created["outcome"]["selection"]["focus"]["cellPath"]
        assert insert(script)["code"] == "unsupported"

        handle = worker.host.open_bytes()
        try:
            worker.host.request(handle, op="beginEdit", expectedRevision=0, selection=caret(0))
            direct = worker.host.request(handle, op="executeCommand", expectedRevision=0,
                                         command={"command": "insertEquation", "script": "x_2"})
            assert direct["outcome"]["selection"] == caret(1)
            inspect_hwpx(session_save(handle), ["", ""], "x_2", 0, 1000)
        finally:
            worker.host.call("close", handle)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    run(parser.parse_args().library)
