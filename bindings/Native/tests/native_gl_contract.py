#!/usr/bin/env python3
"""Actual native GL/FBO contracts; run under Xvfb/xcb, never Qt offscreen plugin.

Qt toImage readbacks below are host verification operations. They are counted
separately from the presenter's zero normal-path whole-frame CPU readbacks.
"""

import argparse
from concurrent.futures import ThreadPoolExecutor
import ctypes
import hashlib
import io
import json
from pathlib import Path
import struct
import sys
import zipfile
import zlib

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_gl import NativeGlPresenter
from native_host import NativeCallError, NativeHost
from native_worker import NativeWorker


def rejected(operation, code="invalidArgument"):
    try:
        operation()
    except NativeCallError as error:
        assert error.code == code, error.response
        return error.response
    raise AssertionError("invalid GL operation succeeded")


def accepted(worker, command):
    result = worker.wait(worker.submit(command))
    assert result["ok"], result
    return result


def configure(worker, info, view=1):
    stamp = {"documentId": info["documentId"], "revision": info["revision"],
             "fontGeneration": info["fontGeneration"], "viewGeneration": view,
             "deviceGeneration": 1, "deviceScale": 1.0}
    worker.host.host_request(op="presentationConfigure", worker=worker.worker, stamp=stamp)
    return stamp


def export(worker, info):
    return accepted(worker, {"op": "exportRenderSnapshot",
        "target": {"documentId": info["documentId"], "revision": info["revision"]},
        "fontGeneration": info["fontGeneration"], "page": 0, "profile": "screen"})["snapshot"]


def corrupt_image(document):
    output = io.BytesIO()
    changed = 0
    with zipfile.ZipFile(io.BytesIO(document)) as source, zipfile.ZipFile(output, "w") as target:
        for entry in source.infolist():
            data = source.read(entry.filename)
            if entry.filename.lower().startswith("bindata/") and entry.filename.lower().endswith(".png"):
                # Keep the valid PNG header so layout emits an actual image,
                # rather than the normal MissingPicture semantic placeholder.
                assert data.startswith(b"\x89PNG\r\n\x1a\n")
                data = bytearray(data)
                offset = data.index(b"IDAT")
                size = struct.unpack(">I", data[offset - 4:offset])[0]
                assert size > 0 and offset + size + 8 <= len(data)
                data[offset + 4] ^= 0xff  # Invalid zlib compression method.
                # A valid chunk CRC makes this a full-decode failure, rather
                # than merely a container/header integrity rejection.
                end = offset + size + 4
                data[end:end + 4] = struct.pack(">I", zlib.crc32(data[offset:end]))
                try:
                    zlib.decompress(data[offset + 4:end])
                except zlib.error:
                    pass
                else:
                    raise AssertionError("corrupt PNG must fail actual pixel decoding")
                changed += 1
            target.writestr(entry, data)
    assert changed, "contract fixture must contain a PNG resource"
    return output.getvalue()


def disabled_contract(host):
    function = host.library.rhwp_gl_presenter_create
    function.argtypes = [ctypes.c_uint64, ctypes.c_uint64, ctypes.c_void_p, ctypes.c_void_p,
                         ctypes.c_void_p, ctypes.c_void_p, ctypes.c_size_t]
    function.restype = ctypes.c_void_p
    rejected(lambda: host._result(function(0, 0, None, None, None, None, 0)), "featureDisabled")
    function = host.library.rhwp_gl_presenter_request
    function.argtypes = [ctypes.c_uint64, ctypes.c_void_p, ctypes.c_size_t]
    function.restype = ctypes.c_void_p
    rejected(lambda: host._result(function(0, None, 0)), "featureDisabled")
    function = host.library.rhwp_gl_presenter_recreate
    function.argtypes = [ctypes.c_uint64, ctypes.c_uint64, ctypes.c_uint64,
                         ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p,
                         ctypes.c_void_p, ctypes.c_size_t]
    function.restype = ctypes.c_void_p
    rejected(lambda: host._result(function(0, 0, 0, None, None, None, None, 0)), "featureDisabled")


def raw_request(host, handle, **request):
    encoded = json.dumps(request, allow_nan=False).encode("utf-8")
    storage = ctypes.create_string_buffer(encoded)
    return host._result(host.library.rhwp_gl_presenter_request(handle, storage, len(encoded)))


def gl_contract(library, document):
    from PySide6.QtGui import QGuiApplication, QImage, QOffscreenSurface, QOpenGLContext, QSurfaceFormat
    from PySide6.QtOpenGL import QOpenGLFramebufferObject, QOpenGLFramebufferObjectFormat

    application = QGuiApplication.instance() or QGuiApplication([])
    assert application.platformName() != "offscreen", "use Xvfb and QT_QPA_PLATFORM=xcb for actual GL"
    format = QSurfaceFormat()
    format.setRenderableType(QSurfaceFormat.RenderableType.OpenGL)
    format.setVersion(3, 3)
    format.setProfile(QSurfaceFormat.OpenGLContextProfile.CoreProfile)
    format.setDepthBufferSize(24)
    format.setStencilBufferSize(8)
    format.setSamples(0)
    context = QOpenGLContext()
    context.setFormat(format)
    assert context.create() and context.isValid(), "Qt could not create an actual GL context"
    surface = QOffscreenSurface()
    surface.setFormat(context.format())
    surface.create()
    assert surface.isValid() and context.makeCurrent(surface), "Qt GL context is not current"
    fbo_format = QOpenGLFramebufferObjectFormat()
    fbo_format.setAttachment(QOpenGLFramebufferObject.Attachment.CombinedDepthStencil)
    fbo_format.setInternalTextureFormat(0x8058)  # GL_RGBA8
    fbo_format.setSamples(0)
    framebuffer = QOpenGLFramebufferObject(96, 96, fbo_format)
    assert framebuffer.isValid() and framebuffer.bind(), "Qt framebuffer is unavailable"
    calls = {"loader": 0, "current": 0, "forbidCurrent": False, "hostReadbacks": 0}

    def proc(name):
        assert name.startswith(b"gl"), "native adapter requested a foreign API from the GL resolver"
        calls["loader"] += 1
        # Both Qt's QByteArray and str overloads resolve the same GL symbol.
        return context.getProcAddress(name.decode("ascii"))

    def current(identity):
        assert not calls["forbidCurrent"], "abandon unexpectedly called host isCurrent"
        calls["current"] += 1
        return identity == 1 and QOpenGLContext.currentContext() == context

    def readback():
        calls["hostReadbacks"] += 1
        image = framebuffer.toImage().convertToFormat(QImage.Format.Format_RGBA8888)
        assert not image.isNull() and image.width() == 96 and image.height() == 96
        return hashlib.sha256(bytes(image.constBits()[:image.sizeInBytes()])).hexdigest()

    with NativeWorker(library) as worker:
        cpu_fields = ("frameHandles", "retainedFrames", "frameBytes", "leases")
        initial_stats = worker.host.presentation_request(op="stats")["stats"]
        info = worker.wait(worker.open_bytes(document))["info"]
        stamp = configure(worker, info)
        snapshot = export(worker, info)
        bad_snapshot = None
        presenter = NativeGlPresenter(worker.host, 1, 1, proc, current)
        spare = None
        try:
            loader_calls = calls["loader"]
            target = {"contextId": 1, "deviceGeneration": 1, "fbo": framebuffer.handle(),
                      "width": 96, "height": 96, "format": "rgba8", "origin": "bottomLeft",
                      "sampleCount": 0, "stencilBits": 8}
            placements = [{"snapshot": snapshot, "scale": 0.08, "originX": 8,
                           "originY": 4, "backgroundColor": 0xffffff},
                          {"snapshot": snapshot, "scale": 0.06, "originX": 20,
                           "originY": 12, "backgroundColor": 0xffffff}]
            functions = context.functions()
            functions.glClearColor(1, 0, 1, 1)
            functions.glClear(0x00004000)  # GL_COLOR_BUFFER_BIT: sentinel target.
            sentinel = readback()
            result = presenter.render(stamp, target, placements)
            frame, state = result["frame"], result["info"]
            assert frame["pageCount"] == 2 and frame["stamp"] == stamp, frame
            assert frame["pixelFormat"] == "rgba8Premultiplied" and frame["colorSpace"] == "srgb"
            assert frame["alpha"] == "premultiplied" and frame["stride"] is None
            assert frame["target"] == target and frame["wholeFrameReadbacks"] == 0
            assert state["renderers"] == 1 and state["externalTextureExport"] is False, state
            valid = readback()
            assert valid != sentinel, "host target did not receive the GPU replay"
            assert calls["loader"] == loader_calls, "render reused the creation-only loader"
            before = presenter.info()
            rejected(lambda: presenter.render(stamp, dict(target, width=4097), placements))
            rejected(lambda: presenter.render(stamp, dict(target, deviceGeneration=2), placements), "contextMismatch")
            stale = rejected(lambda: presenter.render(dict(stamp, viewGeneration=2), target, placements), "staleFrame")
            assert stale["retryable"] is True
            assert presenter.info() == before and readback() == valid
            with ThreadPoolExecutor(max_workers=1) as other:
                other.submit(rejected, presenter.info).result(timeout=20)
            context.doneCurrent()
            rejected(presenter.release, "contextNotCurrent")
            rejected(lambda: presenter.render(stamp, target, placements), "contextNotCurrent")
            assert context.makeCurrent(surface)
            assert presenter.info()["framesSubmitted"] == before["framesSubmitted"]
            # Advance only the view; the same immutable page input remains valid.
            stamp = configure(worker, info, view=2)
            presenter.render(stamp, target, placements)
            valid = readback()
            # The existing authored PNG fixture now has an undecodable payload.
            # The document's earlier paints must remain isolated in scratch.
            replacement = {"documentId": info["documentId"], "revision": info["revision"]}
            opened = worker.wait(worker.open_bytes(corrupt_image(document), replace=replacement))
            assert opened["ok"], opened
            info = opened["info"]
            stamp = configure(worker, info, view=3)
            bad_snapshot = export(worker, info)
            invalid_pages = [dict(placements[0], snapshot=bad_snapshot)]
            before = presenter.info()["framesSubmitted"]
            rejected(lambda: presenter.render(stamp, target, invalid_pages, clear_color=0xFFFF0000), "documentError")
            assert presenter.info()["framesSubmitted"] == before
            assert readback() == valid, "late strict resource error modified visible host pixels"
            worker.close()
            rejected(lambda: presenter.render(stamp, target, invalid_pages), "staleFrame")
            assert readback() == valid
            # A second owner proves context-lost abandon needs no host callback.
            spare = NativeGlPresenter(worker.host, 1, 1, proc, current)
            context.doneCurrent()
            calls["forbidCurrent"] = True
            before_current = calls["current"]
            spare.abandon()
            assert calls["current"] == before_current
            spare = None
            calls["forbidCurrent"] = False
            assert context.makeCurrent(surface)
            final = presenter.info()
            assert final["wholeFrameReadbacks"] == 0
            assert final["scratchTargets"] <= final["config"]["maxScratchTargets"]
            assert final["scratchEstimatedBytes"] <= final["config"]["maxScratchBytes"]
            assert final["renderers"] <= final["config"]["maxRenderers"]
            assert final["rendererFontEstimatedBytes"] <= final["config"]["maxRendererFontBytes"]
            final_stats = worker.host.presentation_request(op="stats")["stats"]
            assert all(final_stats[key] == initial_stats[key] for key in cpu_fields), final_stats
            presenter.release()
            rejected(lambda: raw_request(worker.host, presenter.presenter, op="release",
                contextId=1, deviceGeneration=1))
            rejected(lambda: raw_request(worker.host, presenter.presenter, op="info"))
            print(json.dumps({"contract": "PASS", "device": final["device"],
                "framesSubmitted": final["framesSubmitted"], "wholeFrameReadbacks": 0,
                "hostVerificationReadbacks": calls["hostReadbacks"],
                "strictFailurePreservedTarget": True, "externalTextureExport": False}))
        finally:
            if spare is not None:
                spare.abandon()
            if not presenter._closed:
                if QOpenGLContext.currentContext() == context:
                    presenter.release()
                else:
                    presenter.abandon()
            if bad_snapshot is not None:
                worker.host.presentation_request(op="snapshotRelease", snapshot=bad_snapshot)
            worker.host.presentation_request(op="snapshotRelease", snapshot=snapshot)
    assert context.makeCurrent(surface)
    framebuffer.release()
    del framebuffer  # Destroy Qt-owned GL target while its context is current.
    context.doneCurrent()
    surface.destroy()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("library", type=Path)
    parser.add_argument("--require-gl", action="store_true")
    parser.add_argument("--document", type=Path, default=Path(__file__).resolve().parents[3]
                        / "tests/fixtures/pr8061_cell_picture_layer/bordered-page-picture.hwpx")
    args = parser.parse_args()
    host = NativeHost(args.library)
    caps = host.call("capabilities")
    assert caps["gpuGl"]["externalTextureExport"] is False
    assert caps["presentation"]["gpuFrames"] is False
    if not caps["gpuGl"]["available"]:
        disabled_contract(host)
        assert not args.require_gl, "this invocation requires the actual native-skia-gl backend"
        print("native GL disabled-build contract PASS")
        return
    assert caps["gpuGl"]["hostFboRendering"]
    gl_contract(args.library, args.document.read_bytes())


if __name__ == "__main__":
    main()
