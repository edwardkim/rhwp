#!/usr/bin/env python3
"""Exercise native picture insertion, selection, resize, history, and file round trips."""

import argparse
from base64 import b64decode
from io import BytesIO
from xml.etree import ElementTree
from zipfile import ZipFile

from native_equation_contract import NativeWorker, PARAGRAPH_NS, caret, point
from native_host import NativeCallError


PNG = b64decode(
    "iVBORw0KGgoAAAANSUhEUgAAAAQAAAACCAIAAADwyuo0AAAAEklEQVR42mN86KbMAANMDEgA"
    "AB/8AU6Xru/FAAAAAElFTkSuQmCC"
)
JPEG = b64decode(
    "/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAgGBgcGBQgHBwcJCQgKDBQNDAsLDBkSEw8U"
    "HRofHh0aHBwgJC4nICIsIxwcKDcpLDAxNDQ0Hyc5PTgyPC4zNDL/2wBDAQkJCQwLDBgN"
    "DRgyIRwhMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIy"
    "MjIyMjL/wAARCAACAAQDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAX/xAAU"
    "EAEAAAAAAAAAAAAAAAAAAAAA/8QAFQEBAQAAAAAAAAAAAAAAAAAABgf/xAAUEQEAAAAA"
    "AAAAAAAAAAAAAAAA/9oADAMBAAIRAxEAPwCeAMKO/9k="
)


def inspect(data, paragraphs, image=None, size=None):
    with ZipFile(BytesIO(data)) as package:
        section = ElementTree.fromstring(package.read("Contents/section0.xml"))
        assets = [
            name for name in package.namelist() if name.lower().startswith("bindata/")
        ]
        nodes = section.findall(".//hp:pic", PARAGRAPH_NS)
        texts = [
            "".join(run.text or "" for run in p.findall(".//hp:t", PARAGRAPH_NS))
            for p in section.findall("hp:p", PARAGRAPH_NS)
        ]
        assert texts == paragraphs, texts
        if image is None:
            assert not nodes and not assets, (nodes, assets)
            return
        assert len(nodes) == len(assets) == 1, (nodes, assets)
        assert package.read(assets[0]) == image
        assert nodes[0] in section.findall("hp:p", PARAGRAPH_NS)[1].iter()
        position = nodes[0].find("hp:pos", PARAGRAPH_NS)
        assert position.attrib["treatAsChar"] == "1", position.attrib
        dimensions = nodes[0].find("hp:sz", PARAGRAPH_NS)
        assert tuple(int(dimensions.attrib[key]) for key in ("width", "height")) == size


def run(library):
    with NativeWorker(library) as worker:
        def call(**request):
            return worker.wait(worker.submit(request))

        def accepted(**request):
            result = call(**request)
            assert result["ok"], result
            return result

        def insert(image, stamp=None):
            queued = worker.host.host_request(
                op="workerInsertPicture", worker=worker.worker, target=stamp or target, data=image
            )
            return worker.wait(queued["requestId"])

        def save(format="hwpx"):
            prepared = accepted(op="prepareSave", target=target, format=format)
            data = worker.copy_buffer(prepared)
            accepted(op="cancelSave", documentId=target["documentId"], token=prepared["token"])
            return data

        def advance(outcome):
            target["revision"] = outcome["revision"]
            return outcome

        def query():
            result = accepted(op="pictureObject", target=target, page=0, address=address)
            assert result["kind"] == "pictureObject" and result["target"] == target
            assert result["page"] == 0 and "fontGeneration" in result
            return result["picture"]

        capabilities = worker.host.call("capabilities")
        assert capabilities["insertPicture"] and capabilities["pictureObjectProperties"]
        assert capabilities["maxPictureBytes"] == 16 * 1024 * 1024
        assert capabilities["maxPictureObjectSize"] == 200_000
        opened = accepted(op="new", replace=None)["info"]
        target = {"documentId": opened["documentId"], "revision": opened["revision"]}
        assert accepted(op="pictureObjectAt", target=target, page=0, x=0, y=0)["picture"] is None
        accepted(op="beginEdit", target=target, selection=caret(0))
        advance(accepted(op="replace", target=target, text="앞뒤")["outcome"])
        address = {"section": 0, "paragraph": 1, "control": 0}
        accepted(op="setSelection", target=target, selection={"anchor": point(0), "focus": point(0, 1)})
        assert insert(PNG)["code"] == "invalidArgument"

        insertion = caret(0, 1)
        accepted(op="setSelection", target=target, selection=insertion)
        for image in (b"not an image", PNG[:36]):
            assert insert(image)["code"] == "invalidArgument"
        try:
            insert(b"x" * (capabilities["maxPictureBytes"] + 1))
        except NativeCallError as error:
            assert error.code == "limitExceeded"
        else:
            raise AssertionError("host accepted oversized picture data")
        assert accepted(op="info")["info"]["revision"] == target["revision"]
        assert query() is None

        for image in (PNG, JPEG):
            stale = dict(target)
            inserted = insert(image)
            assert inserted["ok"] and inserted["kind"] == "edited", inserted
            outcome = advance(inserted["outcome"])
            assert outcome["documentChanged"] and outcome["historyChanged"]
            assert outcome["selection"] == caret(2)
            assert insert(image, stale)["code"] == "staleRevision"
            picture = query()
            assert picture["address"] == address and picture["canResize"], picture
            assert picture["properties"]["treatAsChar"]
            assert (picture["properties"]["width"], picture["properties"]["height"]) == (300, 150)
            bounds = picture["bounds"]
            hit = accepted(op="pictureObjectAt", target=target, page=0,
                           x=bounds["x"] + bounds["width"] / 2,
                           y=bounds["y"] + bounds["height"] / 2)
            assert hit["picture"] == picture, hit
            assert accepted(op="pictureObjectAt", target=target, page=0, x=0, y=0)["picture"] is None

            def resize(**size):
                return call(op="executeCommand", target=target,
                            command={"command": "resizePictureObject", "picture": address, **size})

            assert resize(width=0)["code"] == "invalidArgument"
            unchanged = resize(width=300)
            assert unchanged["ok"] and not unchanged["outcome"]["documentChanged"]
            assert unchanged["outcome"]["revision"] == target["revision"]
            resized = resize(width=7_500, height=3_750)
            assert resized["ok"] and resized["kind"] == "command", resized
            assert advance(resized["outcome"])["historyChanged"]
            resized_picture = query()
            assert (resized_picture["properties"]["width"], resized_picture["properties"]["height"]) == (7_500, 3_750)
            assert resized_picture["bounds"]["width"] > bounds["width"]
            assert resized_picture["bounds"]["height"] > bounds["height"]
            inspect(save(), ["앞", "", "뒤"], image, (7_500, 3_750))
            reopened = worker.host.open_bytes(save("hwp"))
            try:
                prepared = worker.host.request(reopened, op="prepareSave", format="hwpx")
                inspect(worker.host.copy_buffer(prepared), ["앞", "", "뒤"], image, (7_500, 3_750))
                worker.host.request(reopened, op="cancelSave", token=prepared["token"])
            finally:
                worker.host.call("close", reopened)

            advance(accepted(op="undo", target=target)["outcome"])
            inspect(save(), ["앞", "", "뒤"], image, (300, 150))
            outcome = advance(accepted(op="undo", target=target)["outcome"])
            assert outcome["selection"] == insertion
            assert query() is None
            inspect(save(), ["앞뒤"])


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    run(parser.parse_args().library)
