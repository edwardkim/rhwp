#!/usr/bin/env python3
"""Actual GL tile reuse, direct-frame parity, pin/admission and failure contracts.

Qt framebuffer readbacks are verification only. Native render stays GPU/FBO
based and advertises no external texture consumer/fence completion contract.
"""

import argparse
import io
from pathlib import Path
import struct
import sys
import zipfile
import zlib

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_gl import NativeGlPresenter
from native_host import NativeHost
from native_worker import NativeWorker
from native_gl_contract import accepted, configure, corrupt_image, disabled_contract, export, rejected


class QtTarget:
    def __init__(self, context_id=1, generation=1):
        from PySide6.QtGui import QGuiApplication, QOffscreenSurface, QOpenGLContext, QSurfaceFormat
        from PySide6.QtOpenGL import QOpenGLFramebufferObject, QOpenGLFramebufferObjectFormat

        self.application = QGuiApplication.instance() or QGuiApplication([])
        assert self.application.platformName() != "offscreen", "use Xvfb/xcb for actual GL"
        self.context_type = QOpenGLContext
        self.context_id, self.generation = context_id, generation
        format = QSurfaceFormat()
        format.setRenderableType(QSurfaceFormat.RenderableType.OpenGL)
        format.setVersion(3, 3)
        format.setProfile(QSurfaceFormat.OpenGLContextProfile.CoreProfile)
        format.setDepthBufferSize(24)
        format.setStencilBufferSize(8)
        format.setSamples(0)
        self.context = QOpenGLContext()
        self.context.setFormat(format)
        assert self.context.create() and self.context.isValid()
        self.surface = QOffscreenSurface()
        self.surface.setFormat(self.context.format())
        self.surface.create()
        assert self.surface.isValid() and self.context.makeCurrent(self.surface)
        format = QOpenGLFramebufferObjectFormat()
        format.setAttachment(QOpenGLFramebufferObject.Attachment.CombinedDepthStencil)
        format.setInternalTextureFormat(0x8058)
        format.setSamples(0)
        self.framebuffer = QOpenGLFramebufferObject(256, 192, format)
        assert self.framebuffer.isValid() and self.framebuffer.bind()
        self.readbacks = 0
        self.comparisons = []

    @property
    def target(self):
        return {"contextId": self.context_id, "deviceGeneration": self.generation,
                "fbo": self.framebuffer.handle(), "width": 256, "height": 192,
                "format": "rgba8", "origin": "bottomLeft", "sampleCount": 0, "stencilBits": 8}

    def presenter(self, host, config=None):
        return NativeGlPresenter(host, self.context_id, self.generation,
            lambda name: self.context.getProcAddress(name.decode("ascii")),
            lambda identity: identity == self.context_id
                and self.context_type.currentContext() == self.context,
            config=config)

    def pixels(self):
        from PySide6.QtGui import QImage
        self.readbacks += 1
        image = self.framebuffer.toImage().convertToFormat(QImage.Format.Format_RGBA8888)
        assert not image.isNull()
        return bytes(image.constBits()[:image.sizeInBytes()])

    def make_current(self):
        assert self.context.makeCurrent(self.surface)

    def close(self):
        self.make_current()
        self.framebuffer.release()
        self.framebuffer = None
        self.context.doneCurrent()
        self.surface.destroy()
        self.context = None
        self.surface = None


def budget(info):
    config, targets, tiles = info["config"], info["managedTargets"], info["tileCache"]
    assert targets["targets"] <= config["maxManagedTargets"], info
    assert targets["estimatedBytes"] <= config["maxManagedTargetBytes"], info
    assert targets["peakTargets"] <= config["maxManagedTargets"], info
    assert targets["peakEstimatedBytes"] <= config["maxManagedTargetBytes"], info
    assert tiles["cachedTiles"] <= config["maxCachedTiles"], info
    assert tiles["cachedBytes"] <= config["maxTileBytes"], info
    assert info["renderers"] <= config["maxRenderers"], info
    assert info["rendererFontEstimatedBytes"] <= config["maxRendererFontBytes"], info
    assert info["rendererCacheEstimatedBytes"] <= config["maxRenderers"] * config["rendererCacheBytes"], info
    assert info["wholeFrameReadbacks"] == 0, info
    # After synchronous submit returns, no canonical/pending tile target remains.
    assert targets["estimatedBytes"] == info["scratchEstimatedBytes"] + tiles["cachedBytes"], info
    assert targets["targets"] == info["scratchTargets"] + tiles["cachedTiles"], info


def same_pixels(reference, actual, label):
    assert len(reference) == len(actual)
    if reference != actual:
        differences = [abs(left - right) for left, right in zip(reference, actual) if left != right]
        raise AssertionError({"case": label, "mismatchedChannels": len(differences),
                              "maxChannelDifference": max(differences)})


def same_tile_pixels(reference, actual, label):
    # Ganesh rasterizes the canonical page into a differently sized target.
    # Antialias coverage and image interpolation can round RGB by one RGBA8
    # step. Tile copies themselves are exact; alpha, geometric support, integer
    # overlap, fallback, repeats and failed publication stay exact below.
    assert len(reference) == len(actual)
    rgb_differences = []
    for index, (left, right) in enumerate(zip(reference, actual)):
        difference = abs(left - right)
        if index % 4 == 3:
            assert difference == 0, {"case": label, "alphaByte": index,
                                     "reference": left, "actual": right}
        elif difference:
            rgb_differences.append(difference)
    maximum = max(rgb_differences, default=0)
    assert maximum <= 1, {"case": label, "maxRgbDifference": maximum,
                           "mismatchedRgbChannels": len(rgb_differences)}
    return {"case": label, "mismatchedRgbChannels": len(rgb_differences),
            "maxRgbDifference": maximum, "alphaDifferences": 0}


def same_geometry_pixels(reference, actual, label):
    comparison = same_tile_pixels(reference, actual, label)
    outline_pixels = 0
    for index in range(0, len(reference), 4):
        left, right = reference[index:index + 4], actual[index:index + 4]
        # The fixture's blue picture outline is antialiased over white paper.
        # Its exact pixel support must survive, even where AA intensity rounds.
        left_outline = left[0] == left[1] and left[2] > left[0]
        right_outline = right[0] == right[1] and right[2] > right[0]
        assert left_outline == right_outline, {"case": label, "displacedOutlinePixel": index // 4}
        if left_outline:
            outline_pixels += 1
        else:
            assert left == right, {"case": label, "geometryPixel": index // 4,
                                   "reference": list(left), "actual": list(right)}
    assert outline_pixels, "geometry control requires the fixture's blue picture outline"
    return dict(comparison, exactOutlineSupport=True, exactOtherGeometry=True)


def pixel_region(pixels, x, y, width, height):
    return b"".join(pixels[((row * 256 + x) * 4):((row * 256 + x + width) * 4)]
                    for row in range(y, y + height))


def transparent_images(document):
    """Keep fixture geometry and PNG dimensions; remove filtered image color."""
    def chunk(kind, data):
        return (struct.pack(">I", len(data)) + kind + data
                + struct.pack(">I", zlib.crc32(kind + data)))

    source, output = io.BytesIO(document), io.BytesIO()
    replaced = 0
    with zipfile.ZipFile(source) as archive, zipfile.ZipFile(output, "w") as result:
        for entry in archive.infolist():
            data = archive.read(entry)
            if entry.filename.lower().startswith("bindata/") and entry.filename.lower().endswith(".png"):
                assert data[:8] == b"\x89PNG\r\n\x1a\n" and data[12:16] == b"IHDR"
                width, height = struct.unpack(">II", data[16:24])
                assert 0 < width <= 1024 and 0 < height <= 1024
                header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
                rows = (b"\0" + bytes(width * 4)) * height
                data = (data[:8] + chunk(b"IHDR", header)
                        + chunk(b"IDAT", zlib.compress(rows)) + chunk(b"IEND", b""))
                replaced += 1
            result.writestr(entry, data)
    assert replaced, "geometry control requires a fixture PNG"
    return output.getvalue()


def placement(snapshot, x=0, y=0, scale=0.5, background=0xffffff):
    return {"snapshot": snapshot, "originX": x, "originY": y,
            "scale": scale, "backgroundColor": background}


def transformed_pages(snapshot):
    return [
        ("vertical scroll", [placement(snapshot, 0, -32)]),
        ("negative placement", [placement(snapshot, -16, -64)]),
        ("fractional negative", [placement(snapshot, -15.25, -31.5)]),
        ("fractional positive", [placement(snapshot, 8.5, 9.25)]),
        ("overlapping pages", [placement(snapshot, -10.25, 8.5),
            placement(snapshot, 47.75, 31.25, 0.25)]),
    ]


def render_pair(target, direct, cached, stamp, pages, label, clear=0xFF55575A,
                exact=False, geometry=False):
    direct.render(stamp, target.target, pages, clear_color=clear)
    reference = target.pixels()
    result = cached.render(stamp, target.target, pages, clear_color=clear)
    actual = target.pixels()
    if geometry:
        target.comparisons.append(same_geometry_pixels(reference, actual, label))
    elif exact:
        same_pixels(reference, actual, label)
        target.comparisons.append({"case": label, "exact": True})
    else:
        target.comparisons.append(same_tile_pixels(reference, actual, label))
    budget(result["info"])
    repeated = cached.render(stamp, target.target, pages, clear_color=clear)
    same_pixels(actual, target.pixels(), label + " cached repeat")
    budget(repeated["info"])
    return result["info"]


def run(library, document):
    retained = []
    with NativeWorker(library) as worker:
        opened = worker.wait(worker.open_bytes(document))
        assert opened["ok"], opened
        info = opened["info"]
        stamp = configure(worker, info)
        snapshot = export(worker, info)
        retained.append(snapshot)
        target = QtTarget()
        owners = []
        try:
            direct = target.presenter(worker.host, {"maxCachedTiles": 0, "maxTileBytes": 0})
            owners.append(direct)
            cached = target.presenter(worker.host, {"tileSize": 128, "maxCachedTiles": 16,
                                                    "maxTileBytes": 1024 * 1024})
            owners.append(cached)
            view = 1
            pages = [placement(snapshot)]
            first = render_pair(target, direct, cached, stamp, pages, "cold page")
            cold_pixels = target.pixels()
            assert first["tileCache"]["misses"] > 0 and first["tileCache"]["canonicalReplays"] > 0
            assert first["tileCache"]["cachedTiles"] > 0
            repeated = render_pair(target, direct, cached, stamp, pages, "same page hit")
            same_pixels(cold_pixels, target.pixels(), "cold versus warm cache")
            assert repeated["tileCache"]["hits"] > first["tileCache"]["hits"]
            assert repeated["tileCache"]["canonicalReplays"] == first["tileCache"]["canonicalReplays"]
            for label, pages in transformed_pages(snapshot):
                view += 1
                stamp = configure(worker, info, view=view)
                previous = cached.info()["tileCache"]
                state = render_pair(target, direct, cached, stamp, pages, label)
                if label == "vertical scroll":
                    assert state["tileCache"]["hits"] > previous["hits"], state
                    same_pixels(pixel_region(cold_pixels, 0, 32, 256, 160),
                                pixel_region(target.pixels(), 0, 0, 256, 160),
                                "integer scroll overlap including tile seams")
                elif label == "negative placement":
                    same_pixels(pixel_region(cold_pixels, 16, 64, 240, 128),
                                pixel_region(target.pixels(), 0, 0, 240, 128),
                                "negative placement overlap including tile seams")
            # Transparent/partial-alpha backgrounds preserve the original path.
            before = cached.info()["tileCache"]
            state = render_pair(target, direct, cached, stamp,
                [placement(snapshot, 8, 4, background=None)], "no page background", exact=True)
            assert state["tileCache"]["viewportFallbacks"] > before["viewportFallbacks"]
            before = state["tileCache"]
            state = render_pair(target, direct, cached, stamp, [placement(snapshot)],
                                "partial-alpha clear", clear=0x8055575A, exact=True)
            assert state["tileCache"]["viewportFallbacks"] > before["viewportFallbacks"]
            # Four tiles pinned by the first page cannot be evicted to admit a
            # second scale in the same frame. It uses visible P90 replay instead.
            pinned = target.presenter(worker.host, {"tileSize": 128, "maxCachedTiles": 4,
                                                   "maxTileBytes": 4 * 128 * 128 * 4})
            owners.append(pinned)
            pages = [placement(snapshot), placement(snapshot, scale=0.25)]
            pinned_state = render_pair(target, direct, pinned, stamp, pages, "busy tile admission")
            assert pinned_state["tileCache"]["pinnedAdmissionFallbacks"] > 0, pinned_state
            assert pinned_state["tileCache"]["cachedTiles"] == 4, pinned_state
            evicted_bytes = pinned_state["tileCache"]["cachedBytes"]
            next_state = render_pair(target, direct, pinned, stamp,
                                    [placement(snapshot, scale=0.25)], "idle tile eviction")
            assert next_state["tileCache"]["evictions"] >= 4, next_state
            assert next_state["tileCache"]["evictedBytes"] >= evicted_bytes, next_state
            # A framebuffer-only budget still renders through P90 and counts
            # that actual scratch allocation, with no uncharged canonical target.
            tight = target.presenter(worker.host, {"maxManagedTargets": 1,
                "maxManagedTargetBytes": 256 * 192 * 4, "tileSize": 128})
            owners.append(tight)
            rejected(lambda: target.presenter(worker.host), "limitExceeded")
            tight_state = render_pair(target, direct, tight, stamp,
                                     [placement(snapshot)], "framebuffer-only budget", exact=True)
            assert tight_state["managedTargets"]["targets"] == 1, tight_state
            assert tight_state["managedTargets"]["estimatedBytes"] == 256 * 192 * 4, tight_state
            assert tight_state["tileCache"]["cachedTiles"] == 0, tight_state
            assert tight_state["tileCache"]["viewportFallbacks"] > 0, tight_state
            before = tight.info()
            valid = target.pixels()
            rejected(lambda: tight.render(stamp, dict(target.target, width=4097), [placement(snapshot)]))
            assert tight.info() == before and target.pixels() == valid
            # Late strict failure may evict idle entries, but publishes neither
            # incomplete new tiles nor partially changed visible pixels.
            replacement = {"documentId": info["documentId"], "revision": info["revision"]}
            changed = worker.wait(worker.open_bytes(corrupt_image(document), replace=replacement))
            assert changed["ok"], changed
            bad_info = changed["info"]
            view += 1
            bad_stamp = configure(worker, bad_info, view=view)
            bad = export(worker, bad_info)
            retained.append(bad)
            before = cached.info()
            rejected(lambda: cached.render(bad_stamp, target.target, [placement(bad)]), "documentError")
            after = cached.info()
            assert after["tileCache"]["canonicalFailures"] > before["tileCache"]["canonicalFailures"], after
            assert after["tileCache"]["cachedTiles"] <= before["tileCache"]["cachedTiles"], after
            assert after["framesSubmitted"] == before["framesSubmitted"], after
            budget(after)
            same_pixels(valid, target.pixels(), "failed strict tile staging")
            for _ in range(4):
                rejected(lambda: cached.render(bad_stamp, target.target, [placement(bad)]), "documentError")
                repeated_failure = cached.info()
                budget(repeated_failure)
                assert repeated_failure["managedTargets"]["estimatedBytes"] == after["managedTargets"]["estimatedBytes"]
                assert repeated_failure["framesSubmitted"] == before["framesSubmitted"]
            same_pixels(valid, target.pixels(), "repeated failed staging cleanup")
            # Restore the same layout with transparent PNG payloads: filtered
            # color can no longer hide a displaced border or clip. AA outline
            # support and all other geometry must remain exact.
            replacement = {"documentId": bad_info["documentId"], "revision": bad_info["revision"]}
            restored = worker.wait(worker.open_bytes(transparent_images(document), replace=replacement))
            assert restored["ok"], restored
            info = restored["info"]
            view += 1
            stamp = configure(worker, info, view=view)
            snapshot = export(worker, info)
            retained.append(snapshot)
            for label, pages in [("cold page", [placement(snapshot)]), *transformed_pages(snapshot)]:
                view += 1
                stamp = configure(worker, info, view=view)
                render_pair(target, direct, cached, stamp, pages, "geometry " + label, geometry=True)
                if label == "cold page":
                    assert target.pixels() != cold_pixels, "transparent image control did not change pixels"
            # Keep the original colored image in the context recreation check.
            replacement = {"documentId": info["documentId"], "revision": info["revision"]}
            restored = worker.wait(worker.open_bytes(document, replace=replacement))
            assert restored["ok"], restored
            info = restored["info"]
            view += 1
            stamp = configure(worker, info, view=view)
            snapshot = export(worker, info)
            retained.append(snapshot)
            cached.render(stamp, target.target, [placement(snapshot)])
            target.context.doneCurrent()
            before = cached.info()
            rejected(cached.clear_caches, "contextNotCurrent")
            assert cached.info() == before
            target.make_current()
            cleared = cached.clear_caches()
            budget(cleared)
            assert cleared["managedTargets"]["targets"] == 0 and cleared["renderers"] == 0, cleared
            assert cleared["tileCache"]["cachedTiles"] == 0, cleared
            for owner in reversed(owners):
                owner.release()
            owners.clear()
            first_readbacks = target.readbacks
        finally:
            target.make_current()
            for owner in reversed(owners):
                if not owner._closed:
                    owner.release()
            target.close()
        second = QtTarget(context_id=2, generation=2)
        owner = None
        try:
            view += 1
            stamp = dict(stamp, viewGeneration=view, deviceGeneration=2)
            worker.host.host_request(op="presentationConfigure", worker=worker.worker, stamp=stamp)
            owner = second.presenter(worker.host)
            assert owner.info()["tileCache"]["cachedTiles"] == 0
            rejected(lambda: owner.render(stamp, dict(second.target, contextId=1), [placement(snapshot)]), "contextMismatch")
            result = owner.render(stamp, second.target, [placement(snapshot)])
            budget(result["info"])
            assert result["info"]["tileCache"]["misses"] > 0
            assert result["info"]["tileCache"]["hits"] == 0
            assert result["info"]["deviceGeneration"] == 2
            owner.release()
            print({"contract": "PASS", "device": result["info"]["device"],
                   "tileCache": result["info"]["tileCache"],
                   "managedTargets": result["info"]["managedTargets"],
                   "directTileComparisons": target.comparisons,
                   "hostVerificationReadbacks": first_readbacks + second.readbacks,
                   "wholeFrameReadbacks": 0})
        finally:
            if owner is not None and not owner._closed:
                second.make_current()
                owner.release()
            second.close()
            for snapshot in retained:
                worker.host.presentation_request(op="snapshotRelease", snapshot=snapshot)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("library", type=Path)
    parser.add_argument("--require-gl", action="store_true")
    parser.add_argument("--document", type=Path, default=Path(__file__).resolve().parents[3]
        / "tests/fixtures/pr8061_cell_picture_layer/bordered-page-picture.hwpx")
    args = parser.parse_args()
    host = NativeHost(args.library)
    caps = host.call("capabilities")["gpuGl"]
    if not caps["available"]:
        disabled_contract(host)
        assert not args.require_gl, "actual GL tile backend is required"
        print("native GL tile disabled-build contract PASS")
        return
    assert caps["tileCache"] and caps["clearCaches"]
    run(args.library, args.document.read_bytes())


if __name__ == "__main__":
    main()
