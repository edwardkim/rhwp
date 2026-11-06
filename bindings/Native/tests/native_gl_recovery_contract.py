#!/usr/bin/env python3
"""Actual CPU/GL quality and explicit context lifecycle/credit contracts.

Context destruction below simulates host lifecycle events; it is not a
hardware-loss injection or a certification of driver VRAM reclamation.
"""

import argparse
import ctypes
import io
import json
from pathlib import Path
import sys
import xml.etree.ElementTree as ET
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_gl import NativeGlPresenter
from native_host import NativeHost
from native_worker import NativeWorker
from native_gl_contract import accepted, corrupt_image, disabled_contract, rejected
from native_gl_tile_contract import QtTarget, budget, placement
from native_presentation_contract import lease_bytes


class QualityTarget(QtTarget):
    def pixels(self):
        """Raw stored premul RGBA, with GL bottom-left rows made top-left.

        No Qt QImage alpha conversion is used as a color/alpha reference.
        """
        assert self.framebuffer.bind()
        store = ctypes.CFUNCTYPE(None, ctypes.c_uint, ctypes.c_int)(
            int(self.context.getProcAddress("glPixelStorei")))
        for name, value in [(0x0D05, 1), (0x0D02, 0), (0x0D03, 0), (0x0D04, 0)]:
            store(name, value)  # PACK alignment/row length/skips, owned span only.
        bind = ctypes.CFUNCTYPE(None, ctypes.c_uint, ctypes.c_uint)(
            int(self.context.getProcAddress("glBindBuffer")))
        bind(0x88EB, 0)  # No pixel-pack buffer: pointer below is host memory.
        address = self.context.getProcAddress("glReadPixels")
        assert address
        read = ctypes.CFUNCTYPE(None, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_int,
            ctypes.c_uint, ctypes.c_uint, ctypes.c_void_p)(int(address))
        width, height = self.target["width"], self.target["height"]
        storage = (ctypes.c_uint8 * (width * height * 4))()
        read(0, 0, width, height, 0x1908, 0x1401, storage)
        error = ctypes.CFUNCTYPE(ctypes.c_uint)(int(self.context.getProcAddress("glGetError")))()
        assert error == 0, {"verificationGlError": error}
        self.readbacks += 1
        data, stride = bytes(storage), width * 4
        return b"".join(data[row * stride:(row + 1) * stride] for row in reversed(range(height)))


def callbacks(target):
    return (lambda name: target.context.getProcAddress(name.decode("ascii")),
            lambda identity: identity == target.context_id
                and target.context_type.currentContext() == target.context)


def stamp_for(worker, info, view, generation=1):
    stamp = {"documentId": info["documentId"], "revision": info["revision"],
             "fontGeneration": info["fontGeneration"], "viewGeneration": view,
             "deviceGeneration": generation, "deviceScale": 1.5}
    worker.host.host_request(op="presentationConfigure", worker=worker.worker, stamp=stamp)
    return stamp


def snapshot(worker, info, page=0):
    return accepted(worker, {"op": "exportRenderSnapshot",
        "target": {"documentId": info["documentId"], "revision": info["revision"]},
        "fontGeneration": info["fontGeneration"], "page": page, "profile": "screen"})["snapshot"]


def cpu_pixels(host, stamp, target, pages, clear):
    result = host.presentation_request(op="renderCpuBatch", stamp=stamp,
        viewport={"width": target["width"], "height": target["height"]},
        placements=pages, clearColor=clear)
    frame, lease = result["frame"], None
    try:
        assert result["info"]["coordinateSpace"] == "viewportPixels"
        assert result["info"]["pageCount"] == len(pages)
        descriptor = result["info"]["descriptor"]
        assert descriptor["pixelFormat"] == "rgba8Premultiplied"
        assert descriptor["alpha"] == "premultiplied" and descriptor["colorSpace"] == "srgb"
        assert descriptor["origin"] == "topLeft" and descriptor["stride"] == target["width"] * 4
        lease = host.presentation_request(op="frameAcquire", frame=frame, stamp=stamp)["lease"]
        return lease_bytes(host, lease)
    finally:
        if lease is not None:
            host.presentation_request(op="leaseRelease", lease=lease)
        host.presentation_request(op="frameRelease", frame=frame)


def compare(cpu, gpu, width, height, opaque_pages):
    """Uniform interiors: <=1 encoded 8-bit unit. AA edges: <=1 pixel support.

    Aliased opaque page clips must have exactly equal alpha coverage. Glyph and
    image AA boundary color differences are reported separately, not used as
    a substitute for interior color/premul/clip acceptance.
    """
    assert len(cpu) == len(gpu) == width * height * 4
    count = width * height
    uniform = []
    boundary = set()
    for index in range(count):
        x, y = index % width, index // width
        color = cpu[index * 4:index * 4 + 4]
        neighbors = [(yy * width + xx) for yy in range(max(0, y - 1), min(height, y + 2))
                     for xx in range(max(0, x - 1), min(width, x + 2))]
        if all(cpu[other * 4:other * 4 + 4] == color for other in neighbors):
            uniform.append(index)
        else:
            boundary.add(index)
        assert max(gpu[index * 4:index * 4 + 3]) <= gpu[index * 4 + 3] + 1, "GPU output is not premultiplied RGBA"
    interior_max = max((abs(cpu[index * 4 + channel] - gpu[index * 4 + channel])
                        for index in uniform for channel in range(4)), default=0)
    assert interior_max <= 1, {"uniformInteriorMaxDifference": interior_max}
    if opaque_pages:
        assert cpu[3::4] == gpu[3::4], "opaque page alpha clip/origin coverage changed"
    # New GPU coverage/color boundaries must stay in the one-pixel AA support
    # of CPU boundaries; uniform interiors have already passed exact color rules.
    for index in range(count):
        if max(abs(cpu[index * 4 + channel] - gpu[index * 4 + channel]) for channel in range(4)) <= 1:
            continue
        x, y = index % width, index // width
        assert any(yy * width + xx in boundary
                   for yy in range(max(0, y - 1), min(height, y + 2))
                   for xx in range(max(0, x - 1), min(width, x + 2))), "GPU difference escaped CPU AA boundary support"
    return {"uniformPixels": len(uniform), "uniformInteriorMaxDifference": interior_max,
            "boundaryMaxDifference": max((abs(a - b) for a, b in zip(cpu, gpu)), default=0)}


def quality_document(document):
    """Keep the picture/table corpus and add an unambiguous editable tail.

    The source fixture has only a top-level paragraph containing an inline
    table. Its initial caret has object affinity, which scalar editing rejects.
    A separate ordinary paragraph lets the public editor create distinct text
    pages without changing or dropping that original rendering control.
    """
    output = io.BytesIO()
    with zipfile.ZipFile(io.BytesIO(document)) as source, zipfile.ZipFile(output, "w") as target:
        for entry in source.infolist():
            data = source.read(entry.filename)
            if entry.filename == "Contents/section0.xml":
                for _, (prefix, uri) in ET.iterparse(io.BytesIO(data), events=["start-ns"]):
                    ET.register_namespace(prefix, uri)
                root = ET.fromstring(data)
                namespace = "{http://www.hancom.co.kr/hwpml/2011/paragraph}"
                paragraphs = root.findall(namespace + "p")
                assert paragraphs, "quality corpus must contain a top-level paragraph"
                paragraph_index = len(paragraphs)
                identifiers = [int(node.get("id", "0")) for node in root.iter(namespace + "p")]
                attributes = dict(paragraphs[-1].attrib, id=str(max(identifiers) + 1),
                                  pageBreak="0", columnBreak="0")
                paragraph = ET.SubElement(root, namespace + "p", attributes)
                run = ET.SubElement(paragraph, namespace + "run", {"charPrIDRef": "0"})
                ET.SubElement(run, namespace + "t")
                data = ET.tostring(root, encoding="utf-8", xml_declaration=True)
            target.writestr(entry, data)
    return output.getvalue(), paragraph_index


def run(library, document):
    document, paragraph_index = quality_document(document)
    retained, verification_readbacks, metrics = [], 0, []
    reference_frames = 0
    with NativeWorker(library) as worker:
        opened = worker.wait(worker.open_bytes(document))
        assert opened["ok"], opened
        info = opened["info"]
        target_id = {"documentId": info["documentId"], "revision": info["revision"]}
        caret = {"section": 0, "paragraph": paragraph_index, "scalarOffset": 0, "cellPath": []}
        accepted(worker, {"op": "beginEdit", "target": target_id,
                          "selection": {"anchor": caret, "focus": caret}})
        accepted(worker, {"op": "pasteText", "target": target_id,
                          "text": "\n".join(f"native quality row {index:03d}" for index in range(120))})
        info = accepted(worker, {"op": "info"})["info"]
        assert info["pageCount"] >= 2, "quality corpus must have distinct native pages"
        stamp = stamp_for(worker, info, view=1)
        pages = [snapshot(worker, info, page) for page in range(2)]
        retained.extend(pages)
        target, owner = QualityTarget(context_id=101, generation=1), None
        view = 1
        try:
            # Empty loader probes create no native context and must
            # not consume any process context/orphan admission.
            original_credits = worker.host.call("capabilities")["gpuGl"]["contextCredits"]
            for _ in range(8):
                error = rejected(lambda: NativeGlPresenter(worker.host, 101, 1,
                    lambda _name: None, callbacks(target)[1]), "backendUnavailable")
                assert not error["deviceFailure"]
            assert worker.host.call("capabilities")["gpuGl"]["contextCredits"] == original_credits
            owner = target.presenter(worker.host)
            cases = [
                ("opaque tile frame", [placement(pages[0], -15.25, -31.5, 0.75)], 0xff112233),
                ("fractional negative", [placement(pages[0], -15.25, -31.5, 0.75)], 0x40112233),
                ("positive clip", [placement(pages[0], 32.5, 24.25, 0.125)], 0x00112233),
                ("transparent page", [placement(pages[0], 32.5, 24.25, 0.125, None)], 0),
                ("partial-alpha viewport", [placement(pages[0], 32.5, 24.25, 0.125, None)], 0x80557799),
                ("distinct overlapping pages", [placement(pages[0], -10.25, 8.5, 0.5),
                    placement(pages[1], 47.75, 31.25, 0.25)], 0x80112233),
            ]
            for label, placements, clear in cases:
                view += 1
                stamp = stamp_for(worker, info, view)
                reference = cpu_pixels(worker.host, stamp, target.target, placements, clear)
                reference_frames += 1
                frame = owner.render(stamp, target.target, placements, clear_color=clear)
                metrics.append({"case": label, **compare(reference, target.pixels(), 256, 192,
                    all(page["backgroundColor"] is not None for page in placements))})
                assert frame["frame"]["stamp"] == stamp
                assert frame["frame"]["timingScope"] == "hostWallClock"
                assert frame["info"]["wholeFrameReadbacks"] == frame["info"]["wholeFrameCpuUploads"] == 0
                budget(frame["info"])
            # Wrong context metadata cannot paint or become a fake device loss.
            before = target.pixels()
            error = rejected(lambda: owner.render(stamp, dict(target.target, contextId=999),
                [placement(pages[0])]), "contextMismatch")
            assert not error["deviceFailure"] and target.pixels() == before
            # Retire/recreate is repeated with actual context destruction, a
            # stable native handle, and no orphan credit growth.
            stable_handle = owner.presenter
            for generation in range(2, 10):
                retired = owner.retire()
                assert retired["state"] == "released"
                assert retired["contextCredits"] == original_credits
                verification_readbacks += target.readbacks
                target.close()
                target = QualityTarget(context_id=100 + generation, generation=generation)
                loader, current = callbacks(target)
                error = rejected(lambda: owner.recreate(target.context_id, generation - 1, loader, current), "recoveryRejected")
                assert not error["deviceFailure"]
                recreated = owner.recreate(target.context_id, generation, loader, current)
                assert owner.presenter == stable_handle
                assert recreated["deviceGeneration"] == generation and recreated["state"] == "live"
                assert recreated["recovery"]["failure"] is None
                assert recreated["managedTargets"]["targets"] == 0
                assert recreated["tileCache"]["cachedTiles"] == 0
                assert recreated["contextCredits"]["orphaned"] == original_credits["orphaned"]
                view += 1
                stamp = stamp_for(worker, info, view, generation)
                budget(owner.render(stamp, target.target, [placement(pages[0])])["info"])
            owner.retire()
            owner.release()
            owner = None
        finally:
            if owner is not None and not owner._closed:
                owner.abandon()
            if target.context is not None:
                verification_readbacks += target.readbacks
                target.close()
        # Independent CPU and GL strict failures publish no CPU producer and
        # leave the last host framebuffer intact, without a device-fault label.
        replacement = {"documentId": info["documentId"], "revision": info["revision"]}
        changed = worker.wait(worker.open_bytes(corrupt_image(document), replace=replacement))
        assert changed["ok"], changed
        bad_info = changed["info"]
        view += 1
        bad_stamp = stamp_for(worker, bad_info, view, 9)
        bad_page = snapshot(worker, bad_info)
        retained.append(bad_page)
        invalid = QualityTarget(context_id=300, generation=9)
        invalid_owner = invalid.presenter(worker.host)
        try:
            functions = invalid.context.functions()
            functions.glClearColor(1, 0, 1, 1)
            functions.glClear(0x00004000)
            before_pixels = invalid.pixels()
            before_stats = worker.host.presentation_request(op="stats")["stats"]
            error = rejected(lambda: cpu_pixels(worker.host, bad_stamp, invalid.target,
                [placement(bad_page)], 0xff112233), "documentError")
            assert "frame" not in error
            assert worker.host.presentation_request(op="stats")["stats"] == before_stats
            error = rejected(lambda: invalid_owner.render(bad_stamp, invalid.target,
                [placement(bad_page)]), "documentError")
            assert error["domain"] == "document" and not error["deviceFailure"]
            assert invalid.pixels() == before_pixels
            budget(invalid_owner.info())
        finally:
            invalid_owner.release()
            verification_readbacks += invalid.readbacks
            invalid.close()
        replacement = {"documentId": bad_info["documentId"], "revision": bad_info["revision"]}
        restored_document = worker.wait(worker.open_bytes(document, replace=replacement))
        assert restored_document["ok"], restored_document
        info = restored_document["info"]
        view += 1
        stamp = stamp_for(worker, info, view, 9)
        pages[0] = snapshot(worker, info)
        retained.append(pages[0])
        # A failed admitted transition leaves an explicit lost handle. Missing
        # loader symbols do not fabricate a physical GPU loss event.
        old = QualityTarget(context_id=400, generation=100)
        owner = old.presenter(worker.host)
        view += 1
        stamp = stamp_for(worker, info, view, 100)
        owner.render(stamp, old.target, [placement(pages[0])])
        old.close()
        failed = QualityTarget(context_id=401, generation=101)
        try:
            error = rejected(lambda: owner.recreate(401, 101, lambda _name: None,
                callbacks(failed)[1]), "recoveryFailed")
            assert not error["deviceFailure"]
            lost = owner.info()
            assert lost["state"] == "recoveryFailed" and lost["recovery"]["failure"] is not None
            assert lost["recovery"]["lastAttemptedDeviceGeneration"] == 101
            rejected(owner.clear_caches, "contextUnavailable")
            before = owner.info()
            rejected(lambda: owner.recreate(401, 101, *callbacks(failed)), "recoveryRejected")
            assert owner.info() == before
        finally:
            failed.close()
        restored = QualityTarget(context_id=402, generation=102)
        try:
            recovered = owner.recreate(402, 102, *callbacks(restored))
            assert recovered["state"] == "live" and recovered["recovery"]["failure"] is None
            view += 1
            stamp = stamp_for(worker, info, view, 102)
            owner.render(stamp, restored.target, [placement(pages[0])])
            owner.release()
        finally:
            if not owner._closed:
                owner.abandon()
            restored.close()
        # Unproven abandon never returns credit, even after host destruction.
        while True:
            credits = worker.host.call("capabilities")["gpuGl"]["contextCredits"]
            if credits["inUse"] == credits["limit"]:
                break
            generation = 1000 + credits["inUse"]
            target = QualityTarget(context_id=10000 + generation, generation=generation)
            owner = target.presenter(worker.host)
            view += 1
            stamp = stamp_for(worker, info, view, generation)
            owner.render(stamp, target.target, [placement(pages[0])])
            target.close()
            owner.abandon()
        exhausted = worker.host.call("capabilities")["gpuGl"]["contextCredits"]
        assert exhausted["orphaned"] == exhausted["inUse"] == exhausted["limit"] == 4
        target = QualityTarget(context_id=20000, generation=2000)
        try:
            error = rejected(lambda: target.presenter(worker.host), "recoveryBudgetExhausted")
            assert not error["deviceFailure"] and error["domain"] == "resource"
            cpu_pixels(worker.host, stamp, target.target, [placement(pages[0])], 0xff112233)
            reference_frames += 1
            assert worker.host.call("capabilities")["gpuGl"]["contextCredits"] == exhausted
        finally:
            target.close()
            for page in retained:
                worker.host.presentation_request(op="snapshotRelease", snapshot=page)
        print(json.dumps({"contract": "PASS", "quality": metrics,
            "hostVerificationReadbacks": verification_readbacks, "wholeFrameReadbacks": 0,
            "cpuReferenceFrames":reference_frames,
            "wholeFrameCpuUploads": 0, "normalRetireRecreateCycles": 8,
            "contextCredits": exhausted, "simulatedHostContextTransitions": True,
            "hardwareDeviceLossObserved": False}))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("library", type=Path)
    parser.add_argument("--require-gl", action="store_true")
    parser.add_argument("--document", type=Path, default=Path(__file__).resolve().parents[3]
        / "tests/fixtures/pr8061_cell_picture_layer/bordered-page-picture.hwpx")
    args = parser.parse_args()
    host = NativeHost(args.library)
    caps = host.call("capabilities")
    if not caps["gpuGl"]["available"]:
        disabled_contract(host)
        assert not args.require_gl, "actual GL recovery/quality backend is required"
        print("native GL recovery disabled-build contract PASS")
        return
    assert caps["gpuGl"]["typedFailures"] and caps["gpuGl"]["recreate"]
    assert caps["presentation"]["cpuBatchFrames"]
    run(args.library, args.document.read_bytes())


if __name__ == "__main__":
    main()
