"""Thin owner-thread GL FBO adapter; the host creates/makes-current its context."""

import ctypes
import json


GET_PROC_ADDRESS = ctypes.CFUNCTYPE(ctypes.c_void_p, ctypes.c_void_p, ctypes.c_char_p)
IS_CURRENT = ctypes.CFUNCTYPE(ctypes.c_int32, ctypes.c_void_p, ctypes.c_uint64)


class NativeGlPresenter:
    def __init__(self, host, context_id, device_generation, get_proc_address, is_current, config=None):
        self.host = host  # Pins the CDLL while callbacks/handles can execute code.
        self.context_id = context_id
        self.device_generation = device_generation
        self._closed = False
        self.callback_error = None

        def loader(_user_data, name):
            try:
                address = get_proc_address(name)
                return int(address) if address else None
            except BaseException as error:
                self.callback_error = str(error)[:256]
                return None

        def current(_user_data, identity):
            try:
                return int(bool(is_current(identity)))
            except BaseException as error:
                self.callback_error = str(error)[:256]
                return 0

        # C retains current/user_data until release/abandon. Retain both Python
        # callback objects and their captured context/callables for that lifetime.
        self._callbacks = (GET_PROC_ADDRESS(loader), IS_CURRENT(current))
        create = host.library.rhwp_gl_presenter_create
        create.argtypes = [ctypes.c_uint64, ctypes.c_uint64, GET_PROC_ADDRESS, IS_CURRENT,
                           ctypes.c_void_p, ctypes.c_void_p, ctypes.c_size_t]
        create.restype = ctypes.c_void_p
        request = host.library.rhwp_gl_presenter_request
        request.argtypes = [ctypes.c_uint64, ctypes.c_void_p, ctypes.c_size_t]
        request.restype = ctypes.c_void_p
        encoded = json.dumps(config or {}, allow_nan=False).encode("utf-8")
        storage = ctypes.create_string_buffer(encoded)
        result = host._result(create(context_id, device_generation, *self._callbacks,
                                     None, storage, len(encoded)))
        self.presenter = result["presenter"]
        self.creation_info = result["info"]

    def _request(self, **request):
        if self._closed:
            raise RuntimeError("GL presenter is closed")
        encoded = json.dumps(request, allow_nan=False).encode("utf-8")
        storage = ctypes.create_string_buffer(encoded)
        return self.host._result(self.host.library.rhwp_gl_presenter_request(
            self.presenter, storage, len(encoded)))

    def render(self, stamp, target, placements, clear_color=0xFFEEEEEE):
        """Synchronous GPU replay/submit; host owns swap/present and GL state."""
        return self._request(op="render", stamp=stamp, target=target,
                             placements=placements, clearColor=clear_color)

    def info(self):
        return self._request(op="info")["info"]

    def release(self):
        """Call with matching context current, before context destruction."""
        if self._closed:
            return
        self._request(op="release", contextId=self.context_id,
                      deviceGeneration=self.device_generation)
        self._closed = True
        self._callbacks = None

    def abandon(self):
        """Context-lost fallback; no current-context callback or GL is called."""
        if self._closed:
            return
        self._request(op="abandon")
        self._closed = True
        self._callbacks = None
