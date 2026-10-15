#!/usr/bin/env python3
"""실제 native library의 문서/버퍼 수명을 검증한다 (WASM/DOM 불필요)."""
import argparse
import ctypes
import json
import threading


class Host:
    def __init__(self, library):
        self.lib = ctypes.CDLL(str(library))
        signatures = {
            "capabilities": [],
            "open": [ctypes.c_void_p, ctypes.c_size_t, ctypes.c_void_p, ctypes.c_size_t],
            "request": [ctypes.c_uint64, ctypes.c_void_p, ctypes.c_size_t],
            "close": [ctypes.c_uint64],
            "buffer_copy": [ctypes.c_uint64, ctypes.c_void_p, ctypes.c_size_t],
            "buffer_release": [ctypes.c_uint64],
        }
        for name, arguments in signatures.items():
            fn = getattr(self.lib, "rhwp_session_" + name)
            fn.argtypes = arguments
            fn.restype = ctypes.c_void_p
        self.lib.rhwp_string_free.argtypes = [ctypes.c_void_p]
        self.lib.rhwp_string_free.restype = None

    def call(self, name, *arguments):
        pointer = getattr(self.lib, "rhwp_session_" + name)(*arguments)
        assert pointer, name
        try:
            return json.loads(ctypes.string_at(pointer))
        finally:
            self.lib.rhwp_string_free(pointer)

    def request(self, handle, **request):
        data = json.dumps(request).encode()
        return self.call("request", handle, data, len(data))


def run(host):
    caps = host.call("capabilities")
    assert caps["ok"] and caps["abiVersion"] == 1
    assert not host.call("open", None, 1, None, 0)["ok"]
    assert not host.call("open", None, caps["maxDocumentBytes"] + 1, None, 0)["ok"]
    a = host.call("open", None, 0, None, 0)["handle"]
    b = host.call("open", None, 0, None, 0)["handle"]
    assert a != b
    assert host.request(a, op="info")["pageCount"] >= 1
    assert host.request(a, op="pageSize", page=0)["size"]["width"] > 0
    assert not host.request(a, op="pageSize", page=2**32 - 1)["ok"]
    for operation in ("info", "fontState", "clearCaches", "cacheStats"):
        assert not host.request(a, op=operation, unexpected=True)["ok"]
    assert not host.request(a, op="render", page=0, scale=-1)["ok"]
    foreign = []
    thread = threading.Thread(target=lambda: foreign.append(host.request(a, op="info")))
    thread.start()
    thread.join()
    assert foreign and not foreign[0]["ok"]
    frame = host.request(a, op="render", page=0, scale=0.25)
    if caps["nativeSkia"]:
        assert frame["ok"] and frame["pixelFormat"] == "rgba8Premultiplied"
        length = frame["byteLength"]
        assert length == frame["stride"] * frame["height"]
        output = (ctypes.c_uint8 * length)()
        output[0] = 17
        assert not host.call("buffer_copy", frame["buffer"], output, length - 1)["ok"]
        assert output[0] == 17
        assert host.call("close", a)["ok"]
        assert host.call("buffer_copy", frame["buffer"], output, length)["ok"]
        assert all(output[i] == 255 for i in range(3, length, 4))
        assert host.call("buffer_release", frame["buffer"])["ok"]
        assert not host.call("buffer_release", frame["buffer"])["ok"]
    else:
        assert frame["code"] == "featureDisabled"
        assert host.call("close", a)["ok"]
    assert not host.request(a, op="info")["ok"]
    assert not host.call("close", a)["ok"]
    assert host.request(b, op="info")["ok"]
    assert host.call("close", b)["ok"]
    print(json.dumps({"ok": True, "nativeSkia": caps["nativeSkia"], "contract": "native-session"}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("library")
    run(Host(parser.parse_args().library))
