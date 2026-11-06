"""Qt borrows its current GL FBO; the worker exports immutable page snapshots.

The controller deliberately has no Qt/GL dependency. Neither painting nor the
presenter receives a document handle. Normal presentation never copies pixels.
"""
import ctypes
import math


class RendererUnavailable(RuntimeError):
    pass


def response_of(error):
    return getattr(error, "response", {})


def transient(response):
    return (response.get("code") == "staleFrame" or
            response.get("kind") == "skipped" and response.get("reason") == "staleFrame" or
            response.get("code") == "invalidArgument" and response.get("error") in (
                "stale presentation source document",
                "presentation generations must advance before changing inputs"))


def device_failure(response):
    # A strict replay/resource or budget failure is a document diagnostic. Do
    # not switch it to the CPU renderer's permissive placeholder policy.
    return (response.get("code") in (
                "unsupported", "featureDisabled", "backendUnavailable",
                "contextNotCurrent", "contextUnavailable", "contextCreationFailed",
                "targetUnavailable", "deviceLost", "submissionFailed", "recoveryFailed") or
            response.get("code") == "invalidArgument" and
            response.get("error") == "matching host GL context is not current")


class GpuViewportController:
    MAX_VISIBLE_PAGES = 16

    def __init__(self, window):
        self.window = window
        self.host = window.worker.host
        self.snapshots = {}
        self.source = None
        self.view_generation = 0
        self.device_generation = 1
        self.context_id = None
        self.target_key = None
        self.view_key = None
        self.configured = None
        self.failed_key = None
        self.closed = False
        self.last_frame = None
        self.last_info = {}
        self.exports = 0

    def source_key(self):
        info = self.window.info
        return (self.window.document_generation, info.get("documentId"),
                info.get("revision"), info.get("fontGeneration"))

    def visible_pages(self):
        pages = tuple(item["page"] for item in self.window.view.get("visible", []))
        if len(pages) > self.MAX_VISIBLE_PAGES or len(set(pages)) != len(pages):
            raise ValueError("GL viewport exceeds its 16-page snapshot budget")
        return pages

    def release(self, snapshot):
        self.host.presentation_request(op="snapshotRelease", snapshot=snapshot)

    def sync_source(self):
        source = self.source_key()
        if source != self.source:
            self.source = source
            self.last_frame = None
            previous, self.snapshots = self.snapshots, {}
            for snapshot in previous.values():
                self.release(snapshot)
            self.failed_key = None
            self.window.canvas.update()
        visible = set(self.visible_pages())
        for page in tuple(self.snapshots):
            if page not in visible:
                self.release(self.snapshots.pop(page))

    def stamp(self, target=None):
        self.sync_source()
        dpr = self.window.state["config"]["deviceScale"]
        if not math.isfinite(dpr) or not 0 < dpr <= 16:
            raise ValueError("invalid GL device scale")
        if target is not None:
            self.target_key = tuple(target[key] for key in (
                "contextId", "deviceGeneration", "fbo", "width", "height",
                "format", "origin", "sampleCount", "stencilBits"))
        key = (self.source, self.window.epoch, dpr, self.device_generation, self.target_key)
        if key != self.view_key:
            self.view_key = key
            self.view_generation += 1
            self.last_frame = None
        return {"documentId": self.source[1], "revision": self.source[2],
                "fontGeneration": self.source[3], "viewGeneration": self.view_generation,
                "deviceGeneration": self.device_generation, "deviceScale": dpr}

    def configure(self, target=None):
        stamp = self.stamp(target)
        if stamp != self.configured:
            self.host.host_request(op="presentationConfigure", worker=self.window.worker.worker, stamp=stamp)
            self.configured = stamp
        return stamp

    def diagnose(self, error):
        response = response_of(error)
        self.failed_key = self.view_key
        self.last_frame = None
        self.window.canvas.update()
        if transient(response):
            # The worker may already be ahead of the last GUI info completion.
            # One FIFO info barrier refreshes that source; no busy retry loop.
            self.window.refresh()
        elif device_failure(response) or isinstance(error, RendererUnavailable):
            self.window.gpu_failed(error)
        else:
            self.window.fail(error)

    def render_next(self):
        if self.closed:
            return
        try:
            stamp = self.stamp()
            if self.failed_key == self.view_key:
                return
            self.configure()
            page = next((page for page in self.visible_pages() if page not in self.snapshots), None)
            if page is None:
                return
            source = self.source
            self.exports += 1
            self.window.send({"op": "exportRenderSnapshot",
                "target": {"documentId": stamp["documentId"], "revision": stamp["revision"]},
                "fontGeneration": stamp["fontGeneration"], "page": page, "profile": "screen"},
                lambda result: self.received(source, page, result))
        except Exception as error:
            self.diagnose(error)

    def received(self, source, page, result):
        snapshot = result.get("snapshot")
        retained = False
        try:
            if self.closed or source != self.source_key() or page not in self.visible_pages():
                return
            if not result.get("ok") or result.get("kind") == "skipped":
                if transient(result):
                    self.failed_key = self.view_key
                    self.window.refresh()
                else:
                    raise RuntimeError(result.get("error", "render snapshot export failed"))
                return
            info = result.get("info", {})
            if info.get("current") is False:
                self.failed_key = self.view_key
                self.window.refresh()
                return
            if (not isinstance(snapshot, int) or isinstance(snapshot, bool) or snapshot <= 0 or
                    result.get("kind") != "renderSnapshot" or
                    result.get("documentId") != source[1] or
                    any(info.get(key) != value for key, value in (
                        ("documentId", source[1]), ("revision", source[2]),
                        ("fontGeneration", source[3]), ("page", page), ("profile", "screen"))) or
                    info.get("current") is not True):
                raise ValueError("render snapshot metadata does not match its requested source")
            if page in self.snapshots:
                self.release(self.snapshots.pop(page))
            self.snapshots[page] = snapshot
            retained = True
            self.window.update_raster_status()
            self.window.canvas.update()
        except Exception as error:
            self.diagnose(error)
        finally:
            if snapshot is not None and not retained:
                self.release(snapshot)

    def frame_arguments(self, target):
        if self.closed or self.window.loading_pages or not self.window.info or not self.window.view:
            self.last_frame = None
            return None
        stamp = self.stamp(target)
        if self.failed_key == self.view_key:
            self.last_frame = None
            return None
        self.configure(target)
        pages = self.visible_pages()
        if not pages or any(page not in self.snapshots for page in pages):
            self.last_frame = None
            return None
        scale = self.window.state["config"]["zoom"] * stamp["deviceScale"]
        return stamp, [{"snapshot": self.snapshots[item["page"]], "scale": scale,
            "originX": item["pageRect"]["x"] * stamp["deviceScale"],
            "originY": item["pageRect"]["y"] * stamp["deviceScale"],
            "backgroundColor": 0xffffff} for item in self.window.view.get("visible", [])]

    def close(self):
        if self.closed:
            return
        self.closed = True
        self.last_frame = None
        previous, self.snapshots = self.snapshots, {}
        for snapshot in previous.values():
            self.release(snapshot)


def create_gpu_canvas(window, input_mixin):
    """Import GL lazily so an old/CPU-only Native build remains usable."""
    from PySide6.QtGui import QColor, QGuiApplication, QOpenGLContext, QPainter, QSurfaceFormat
    from PySide6.QtOpenGLWidgets import QOpenGLWidget
    from PySide6.QtCore import QTimer

    if QGuiApplication.platformName() in ("offscreen", "minimal"):
        raise RendererUnavailable("Qt platform does not support QOpenGLWidget; use xcb or a desktop platform")
    caps = window.worker.host.call("capabilities")
    caps = caps.get("capabilities", caps)
    if not (caps.get("gpuGl", {}).get("hostFboRendering") and
            caps.get("presentation", {}).get("workerRenderSnapshots") and
            hasattr(window.worker.host, "presentation_request")):
        raise RendererUnavailable("Native GL/presentation capabilities are unavailable")
    try:
        from native_gl import NativeGlPresenter
    except ImportError as error:
        raise RendererUnavailable("native_gl Python adapter is unavailable") from error

    class GpuCanvas(input_mixin, QOpenGLWidget):
        def __init__(self, parent):
            # Qt may retain this dynamic Python class in its meta-type cache.
            # Its methods must not capture the particular window in a closure.
            super().__init__(parent)
            self.controller = GpuViewportController(self.window)
            self.presenter = None
            self.gl_context = None
            self.get_integer = None
            self.driver = {}
            self.disposed = False
            self.failed_device = False
            surface = QSurfaceFormat()
            surface.setVersion(3, 3)
            surface.setProfile(QSurfaceFormat.OpenGLContextProfile.CoreProfile)
            surface.setSamples(0)
            surface.setDepthBufferSize(0)
            surface.setStencilBufferSize(8)
            self.setFormat(surface)
            self.setTextureFormat(0x8058)  # GL_RGBA8: do not infer from Qt's default.
            self.setUpdateBehavior(QOpenGLWidget.UpdateBehavior.PartialUpdate)
            self.context_check = QTimer(self)
            self.context_check.setSingleShot(True)
            self.context_check.timeout.connect(self.check_context)
            self.window.gpu = self.controller

        def showEvent(self, event):
            super().showEvent(event)
            self.context_check.start(0)

        def check_context(self):
            if not self.disposed and not self.isValid():
                self.device_error(RendererUnavailable("Qt could not create a valid OpenGL context"))

        def proc(self, name):
            return int(self.gl_context.getProcAddress(name))

        def matching_current(self, identity):
            return (identity == self.controller.context_id and self.gl_context is not None and
                    QOpenGLContext.currentContext() == self.gl_context and self.gl_context.isValid())

        def initializeGL(self):
            if self.disposed:
                return
            try:
                if self.presenter is not None:
                    # A platform may recreate the widget context after loss.
                    # Never release the old handle against the new context.
                    self.presenter.abandon()
                    self.presenter = None
                self.gl_context = self.context()
                if self.gl_context is None or not self.gl_context.isValid():
                    raise RendererUnavailable("Qt OpenGL context is invalid")
                self.controller.device_generation += 1
                self.controller.context_id = id(self.gl_context)
                self.gl_context.aboutToBeDestroyed.connect(self.context_destroying)
                # NativeGlPresenter owns the CFUNCTYPE closures until release or
                # abandon. Its closures retain these bound methods/context too.
                self.presenter = NativeGlPresenter(self.window.worker.host, self.controller.context_id,
                    self.controller.device_generation, self.proc, self.matching_current)
                pointer = self.proc(b"glGetIntegerv")
                if not pointer:
                    raise RendererUnavailable("glGetIntegerv unavailable")
                self.get_integer = ctypes.CFUNCTYPE(None, ctypes.c_uint, ctypes.POINTER(ctypes.c_int))(pointer)
                pointer = self.proc(b"glGetString")
                if pointer:
                    get_string = ctypes.CFUNCTYPE(ctypes.c_char_p, ctypes.c_uint)(pointer)
                    self.driver = {name: (get_string(value) or b"").decode("utf-8", "replace")
                        for name, value in (("vendor", 0x1f00), ("renderer", 0x1f01), ("version", 0x1f02))}
                    renderer = self.driver["renderer"].lower()
                    self.driver["execution"] = ("software" if any(name in renderer for name in
                        ("llvmpipe", "softpipe", "swiftshader", "software")) else "unverified")
                self.failed_device = False
                self.update()
            except Exception as error:
                self.device_error(error)

        def integer(self, parameter):
            value = ctypes.c_int()
            self.get_integer(parameter, ctypes.byref(value))
            return value.value

        def paintGL(self):
            painter = QPainter(self)
            success = False
            try:
                if self.disposed or self.failed_device or self.presenter is None:
                    return
                painter.beginNativePainting()
                try:
                    if not self.matching_current(self.controller.context_id):
                        raise RendererUnavailable("Qt GL context is no longer current")
                    size = self.size() * self.devicePixelRatioF()
                    target = {"contextId": self.controller.context_id,
                        "deviceGeneration": self.controller.device_generation,
                        "fbo": self.defaultFramebufferObject(), "width": size.width(), "height": size.height(),
                        "format": "rgba8", "origin": "bottomLeft",
                        "sampleCount": self.integer(0x80a9), "stencilBits": self.integer(0x0d57)}
                    if target["sampleCount"] != 0 or target["stencilBits"] not in (0, 8):
                        raise RendererUnavailable("Qt FBO sample/stencil configuration is unsupported")
                    args = self.controller.frame_arguments(target)
                    if args is not None:
                        result = self.presenter.render(args[0], target, args[1], clear_color=0xff55575a)
                        self.controller.last_frame = result["frame"]
                        self.controller.last_info = result["info"]
                        self.window.frame_count += 1
                        success = True
                finally:
                    painter.endNativePainting()
                if success:
                    self.paint_overlays(painter)
            except Exception as error:
                response = response_of(error)
                if device_failure(response) or isinstance(error, RendererUnavailable):
                    self.device_error(error)
                else:
                    self.controller.diagnose(error)
            finally:
                if not success:
                    # Native errors preserve its target atomically. The Qt
                    # compositor must still hide that previous document/view
                    # while input is pending or this presentation has failed.
                    self.controller.last_frame = None
                    painter.fillRect(self.rect(), QColor("#55575a"))
                painter.end()
                self.window.paint_count += 1

        def device_error(self, error):
            self.failed_device = True
            self.window.gpu_failed(error)

        def release_presenter(self):
            presenter = self.presenter
            if presenter is None:
                return
            try:
                self.makeCurrent()
                if self.matching_current(self.controller.context_id):
                    presenter.release()
                else:
                    presenter.abandon()
            except Exception as error:
                # Never attempt GL cleanup after makeCurrent/context loss failed.
                presenter.abandon()
                self.window.fail(error)
            finally:
                self.doneCurrent()
            self.presenter = None
            self.get_integer = None
            self.gl_context = None

        def context_destroying(self):
            self.release_presenter()
            # Snapshots retain only immutable IR/resources, independent of this
            # context. A new initializeGL generation can replay those same IDs.

        def dispose_gpu(self):
            if self.disposed:
                return
            self.disposed = True
            self.context_check.stop()
            self.release_presenter()
            self.controller.close()

    return GpuCanvas(window)
