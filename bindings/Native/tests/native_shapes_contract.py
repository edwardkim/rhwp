#!/usr/bin/env python3
"""Verify native shape controls and undo with HWP and HWPX persistence."""

import argparse
from io import BytesIO
from xml.etree import ElementTree
from zipfile import ZipFile

from native_equation_contract import NativeWorker, PARAGRAPH_NS, caret


def inspect(data, expected_shape, expected_text):
    with ZipFile(BytesIO(data)) as package:
        section = ElementTree.fromstring(package.read("Contents/section0.xml"))
    paragraphs = section.findall("hp:p", PARAGRAPH_NS)
    text = ["".join(run.text or "" for run in p.findall(".//hp:t", PARAGRAPH_NS)) for p in paragraphs]
    assert text == expected_text, text
    shapes = [
        element.tag.rsplit("}", 1)[-1]
        for tag in ("rect", "ellipse", "line")
        for element in section.findall(f".//hp:{tag}", PARAGRAPH_NS)
    ]
    assert shapes == ([expected_shape] if expected_shape else []), shapes


def run(library):
    with NativeWorker(library) as worker:
        def call(**request):
            return worker.wait(worker.submit(request))

        def accepted(**request):
            result = call(**request)
            assert result["ok"], result
            return result

        def save(format="hwpx"):
            prepared = accepted(op="prepareSave", target=target, format=format)
            data = worker.copy_buffer(prepared)
            accepted(op="cancelSave", documentId=target["documentId"], token=prepared["token"])
            return data

        capabilities = worker.host.call("capabilities")
        assert capabilities["insertShape"]
        assert (capabilities["minShapeSize"], capabilities["maxShapeSize"]) == (200, 200_000)
        opened = accepted(op="new", replace=None)["info"]
        target = {"documentId": opened["documentId"], "revision": opened["revision"]}
        accepted(op="beginEdit", target=target, selection=caret(0))
        target["revision"] = accepted(op="replace", target=target, text="앞뒤")["outcome"]["revision"]
        insertion = caret(0, 1)
        accepted(op="setSelection", target=target, selection=insertion)
        for values in ({"width": 199}, {"height": 200_001}, {"shape": "unknown"}):
            invalid = {"command": "insertShape", "shape": "rectangle", "width": 7200, "height": 3600}
            invalid.update(values)
            result = call(op="executeCommand", target=target, command=invalid)
            assert not result["ok"], result
            assert accepted(op="info")["info"]["revision"] == target["revision"]

        for kind, tag in (("rectangle", "rect"), ("ellipse", "ellipse"), ("line", "line")):
            edit = accepted(op="executeCommand", target=target,
                            command={"command": "insertShape", "shape": kind, "width": 7200, "height": 3600})
            outcome = edit["outcome"]
            assert outcome["documentChanged"] and outcome["historyChanged"] and outcome["selection"] == caret(2)
            target["revision"] = outcome["revision"]
            inspect(save(), tag, ["앞", "", "뒤"])
            reopened = worker.host.open_bytes(save("hwp"))
            try:
                prepared = worker.host.request(reopened, op="prepareSave", format="hwpx")
                inspect(worker.host.copy_buffer(prepared), tag, ["앞", "", "뒤"])
                worker.host.request(reopened, op="cancelSave", token=prepared["token"])
            finally:
                worker.host.call("close", reopened)
            undone = accepted(op="undo", target=target)["outcome"]
            target["revision"] = undone["revision"]
            assert undone["selection"] == insertion
            inspect(save(), None, ["앞뒤"])


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    run(parser.parse_args().library)
