"""Same-thread client for the native document worker and its nonblocking queue."""

from collections import deque
import math
from pathlib import Path
import sys
import time

try:
    from native_host import NativeHost
except ModuleNotFoundError:
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from native_host import NativeHost


class NativeWorker:
    def __init__(self, library, config=None):
        self.host = NativeHost(library)
        self.worker = self.host.host_request(op="workerCreate", config=config or {})["worker"]
        self._deferred = deque()
        self._closed = False
        self._released = False

    def _check_open(self):
        if self._closed:
            raise RuntimeError("native worker is closed")

    def submit(self, command):
        self._check_open()
        return self.host.host_request(op="workerSubmit", worker=self.worker,
                                      command=command)["requestId"]

    def open_bytes(self, data, replace=None):
        self._check_open()
        return self.host.host_request(op="workerOpen", worker=self.worker,
                                      replace=replace, data=data)["requestId"]

    def cancel(self, request_id):
        self._check_open()
        return self.host.host_request(op="workerCancel", worker=self.worker,
                                      requestId=request_id)["outcome"]

    def _poll_native(self):
        return self.host.host_request(op="workerPoll", worker=self.worker)["completion"]

    def poll(self):
        self._check_open()
        return self._deferred.popleft() if self._deferred else self._poll_native()

    def copy_buffer(self, response):
        return self.host.copy_buffer(response)

    def wait(self, request_id, timeout=30):
        """Blocking CLI/test helper. GUI code uses poll from its event loop.

        A timeout does not mean that an accepted edit was cancelled. The caller
        must continue receiving its result or close the owning document worker.
        """
        self._check_open()
        if not math.isfinite(timeout) or timeout <= 0:
            raise ValueError("timeout must be positive")
        for result in tuple(self._deferred):
            if result["requestId"] == request_id:
                self._deferred.remove(result)
                return result
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            result = self._poll_native()
            if result is not None:
                if result["requestId"] == request_id:
                    return result
                self._deferred.append(result)
            time.sleep(0.001)
        raise TimeoutError("native request is still pending or running")

    def close(self):
        if not self._closed:
            # Deferred buffers have transferred out of the runtime and require
            # explicit release even though the document worker is closing.
            for result in self._deferred:
                if "buffer" in result:
                    self.host.call("buffer_release", result["buffer"])
            self._deferred.clear()
            result = self.host.host_request(op="workerClose", worker=self.worker)
            self._closed = True
            if result["isFinished"]:
                self.release()

    def is_finished(self):
        return self._released or self.host.host_request(
            op="workerStatus", worker=self.worker)["isFinished"]

    def release(self):
        """Release only after the native call has ended; then library unload is safe."""
        if self._released:
            return
        if not self._closed or not self.is_finished():
            raise RuntimeError("close and finish the worker before releasing it")
        try:
            self.host.host_request(op="workerRelease", worker=self.worker)
        finally:
            self._released = True

    def wait_closed(self, timeout=30):
        """Blocking CLI/test shutdown helper; GUI code checks is_finished later."""
        if not math.isfinite(timeout) or timeout <= 0:
            raise ValueError("timeout must be finite and positive")
        self.close()
        deadline = time.monotonic() + timeout
        while not self.is_finished():
            if time.monotonic() >= deadline:
                raise TimeoutError("native call is still finishing; keep the library loaded")
            time.sleep(0.001)
        self.release()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.wait_closed()
