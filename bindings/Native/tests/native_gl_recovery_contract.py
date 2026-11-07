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
import struct
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


def premul_argb(color):
    alpha = color >> 24 & 255
    return tuple(((color >> shift & 255) * alpha + 127) // 255
                 for shift in (16, 8, 0)) + (alpha,)


def page_rectangles(placements, sizes):
    # The API consumes f32 Skia geometry. No pixel masks come from either
    # rendered output; dimensions are queried on the worker before replay.
    def f32(value):
        return struct.unpack("f", struct.pack("f", value))[0]

    rectangles = []
    for page in placements:
        size, scale = sizes[page["snapshot"]], f32(page["scale"])
        left, top = f32(page["originX"]), f32(page["originY"])
        right = f32(left + f32(f32(size["width"]) * scale))
        bottom = f32(top + f32(f32(size["height"]) * scale))
        color = page["backgroundColor"]
        assert color in (None, 0xFFFFFF), "quality corpus has white Screen paper"
        # layout::build_page_background emits white paper for this corpus even
        # when the caller omits the additional host page preclear.
        rgba = (255, 255, 255, 255)
        rectangles.append((left, top, right, bottom, rgba))
    return rectangles


def cell_colors(x, y, rectangles, clear):
    """Possible blank-page endpoints from positive-area pixel-cell geometry."""
    colors = [clear]
    intersects = False
    for left, top, right, bottom, color in rectangles:
        if x + 1 <= left or x >= right or y + 1 <= top or y >= bottom:
            continue
        intersects = True
        if color is None:
            continue
        if left <= x and x + 1 <= right and top <= y and y + 1 <= bottom:
            colors = [color]
        elif color not in colors:
            colors.append(color)
    # The controls use nested pages: at most one unresolved edge remains after
    # any fully covering page. Fail explicitly if a future corpus changes that.
    assert len(colors) <= 2, "clip control requires at most one unresolved page edge"
    return colors, intersects


def endpoint_color(pixel, colors):
    if len(colors) == 1:
        return tuple(pixel) == colors[0]
    start, end = colors
    axis = max(range(4), key=lambda channel: abs(end[channel] - start[channel]))
    delta = end[axis] - start[axis]
    assert delta
    amount = min(1, max(0, (pixel[axis] - start[axis]) / delta))
    return all(abs(pixel[channel] - (start[channel] + amount * (end[channel] - start[channel]))) <= 1
               for channel in range(4))


def check_clip(cpu, gpu, width, height, rectangles, clear, blank=False):
    opaque = all(rectangle[4] is not None for rectangle in rectangles)
    ambiguous, alpha_differences = 0, 0
    clip_max, non_clip_max = 0, 0
    for index in range(width * height):
        colors, intersects = cell_colors(index % width, index // width, rectangles, clear)
        left, right = cpu[index * 4:index * 4 + 4], gpu[index * 4:index * 4 + 4]
        if blank or (opaque and len(colors) > 1) or not intersects:
            assert endpoint_color(left, colors) and endpoint_color(right, colors), {
                "clipPixel": [index % width, index // width], "cpu": list(left),
                "gpu": list(right), "expectedEndpoints": colors}
        if opaque and len(colors) == 1:
            assert left[3] == right[3] == colors[0][3], {
                "exactAlphaPixel": [index % width, index // width], "cpu": left[3],
                "gpu": right[3], "expectedAlpha": colors[0][3]}
        ambiguous += len(colors) > 1
        alpha_differences += left[3] != right[3]
        difference = max(abs(a - b) for a, b in zip(left, right))
        if len(colors) > 1:
            clip_max = max(clip_max, difference)
        else:
            non_clip_max = max(non_clip_max, difference)
    return {"ambiguousBackgroundCells": ambiguous, "alphaDifferentPixels": alpha_differences,
            "clipBoundaryMaxDifference": clip_max, "nonClipMaxDifference": non_clip_max}


def compare(cpu, gpu, width, height, rectangles, clear):
    """Uniform interiors: <=1 encoded 8-bit unit. AA edges: <=1 pixel support.

    Fractional page-edge cells use independently derived clip geometry and
    premul endpoints. All fully inside/outside opaque coverage stays exact.
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
    clipping = check_clip(cpu, gpu, width, height, rectangles, clear)
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
            "boundaryMaxDifference": max((abs(a - b) for a, b in zip(cpu, gpu)), default=0),
            **clipping}


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


def blank_document(document):
    """Remove visible content; preserve an authored section and query its size."""
    output = io.BytesIO()
    with zipfile.ZipFile(io.BytesIO(document)) as source, zipfile.ZipFile(output, "w") as target:
        for entry in source.infolist():
            data = source.read(entry)
            if entry.filename == "Contents/section0.xml":
                for _, (prefix, uri) in ET.iterparse(io.BytesIO(data), events=["start-ns"]):
                    ET.register_namespace(prefix, uri)
                root = ET.fromstring(data)
                namespace = "{http://www.hancom.co.kr/hwpml/2011/paragraph}"
                section = root.find(".//" + namespace + "secPr")
                assert section is not None
                root.clear()
                paragraph = ET.SubElement(root, namespace + "p",
                    {"id": "1", "paraPrIDRef": "0", "styleIDRef": "0"})
                run = ET.SubElement(paragraph, namespace + "run", {"charPrIDRef": "0"})
                run.append(section)
                ET.SubElement(run, namespace + "t")
                data = ET.tostring(root, encoding="utf-8", xml_declaration=True)
            target.writestr(entry, data)
    return output.getvalue()


def oracle_negative_controls():
    clear = premul_argb(0x40112233)
    rectangles = [(1.75, 1.75, 5.75, 5.75, (255, 255, 255, 255))]
    pixels = b"".join(bytes((255, 255, 255, 255) if 2 <= x < 6 and 2 <= y < 6 else clear)
                      for y in range(8) for x in range(8))
    check_clip(pixels, pixels, 8, 8, rectangles, clear, blank=True)
    shifted = b"".join(bytes(clear) + pixels[row * 32:row * 32 + 28] for row in range(8))
    wrong_alpha, outside = bytearray(pixels), bytearray(pixels)
    wrong_alpha[(3 * 8 + 3) * 4 + 3] = 128
    outside[:4] = bytes((255, 255, 255, 255))
    for label, invalid in [("one pixel shift", shifted), ("uniform alpha", wrong_alpha),
                            ("outside pixel", outside)]:
        try:
            check_clip(pixels, invalid, 8, 8, rectangles, clear, blank=True)
        except AssertionError:
            continue
        raise AssertionError("analytic clip oracle accepted " + label)


def blank_clip_controls(library, document):
    oracle_negative_controls()
    metrics = []
    with NativeWorker(library) as worker:
        opened = worker.wait(worker.open_bytes(blank_document(document)))
        assert opened["ok"], opened
        info = opened["info"]
        stamp_for(worker, info, 1)
        page = snapshot(worker, info)
        size = accepted(worker, {"op": "pageSize", "target": {
            "documentId": info["documentId"], "revision": info["revision"]}, "page": 0})["size"]
        sizes = {page: size}
        target, owner = QualityTarget(context_id=90), None
        try:
            owner = target.presenter(worker.host, {"maxCachedTiles": 0, "maxTileBytes": 0})
            cases = [(label + " top-left", [placement(page, 32 + phase, 24 + phase)], 0x40112233)
                     for label, phase in [("integer", 0), ("quarter", 0.25), ("half", 0.5),
                         ("three-quarter", 0.75), ("before-half", 0.499), ("after-half", 0.501)]]
            raster = page_rectangles([placement(page)], sizes)[0]
            for label, phase in [("integer", 0), ("quarter", 0.25), ("half", 0.5),
                                 ("three-quarter", 0.75)]:
                cases.append((label + " bottom-right", [placement(page,
                    128 + phase - raster[2], 120 + phase - raster[3])], 0x40112233))
            cases += [
                ("transparent clear", [placement(page, 32.5, 24.25, 0.125)], 0),
                ("opaque clear", [placement(page, 32.75, 24.25, 0.125)], 0xFF112233),
                ("no host preclear, transparent viewport", [placement(page, 32.75, 24.25, 0.125, None)], 0),
                ("no host preclear, partial-alpha viewport", [placement(page, 32.75, 24.25, 0.125, None)], 0x80557799),
                ("nested pages", [placement(page, 32.5, 24.25, 0.125),
                    placement(page, 64.25, 58.75, 0.0625)], 0x40112233),
            ]
            for view, (label, placements, clear) in enumerate(cases, start=2):
                stamp = stamp_for(worker, info, view)
                cpu = cpu_pixels(worker.host, stamp, target.target, placements, clear)
                result = owner.render(stamp, target.target, placements, clear_color=clear)
                gpu = target.pixels()
                rectangles = page_rectangles(placements, sizes)
                clipping = check_clip(cpu, gpu, 256, 192, rectangles, premul_argb(clear), blank=True)
                if label.startswith("integer"):
                    assert cpu == gpu, {"exactBlankControl": label}
                metrics.append({"case": label, "rectangles": rectangles, **clipping})
                budget(result["info"])
            return {"cases": metrics, "cpuReferenceFrames": len(cases),
                    "hostVerificationReadbacks": target.readbacks, "oracleNegativeControls": 3}
        finally:
            if owner is not None and not owner._closed:
                owner.release()
            target.close()
            worker.host.presentation_request(op="snapshotRelease", snapshot=page)


def run(library, document):
    clip_controls = blank_clip_controls(library, document)
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
        sizes = {handle: accepted(worker, {"op": "pageSize", "target": {
            "documentId": info["documentId"], "revision": info["revision"]}, "page": index})["size"]
            for index, handle in enumerate(pages)}
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
                ("no host page preclear", [placement(pages[0], 32.5, 24.25, 0.125, None)], 0),
                ("partial-alpha clear behind white paper", [placement(pages[0], 32.5, 24.25, 0.125, None)], 0x80557799),
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
                    page_rectangles(placements, sizes), premul_argb(clear))})
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
        print(json.dumps({"contract": "PASS", "quality": metrics, "blankClipControls": clip_controls,
            "hostVerificationReadbacks": verification_readbacks + clip_controls["hostVerificationReadbacks"],
            "wholeFrameReadbacks": 0,
            "cpuReferenceFrames": reference_frames + clip_controls["cpuReferenceFrames"],
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
