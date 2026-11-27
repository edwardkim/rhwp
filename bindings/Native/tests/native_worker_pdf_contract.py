#!/usr/bin/env python3
"""Exercise live Print PDF transport using a real Native FFI library."""

import argparse
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_host import NativeCallError
from native_worker import NativeWorker


def run(library, native_skia):
    with NativeWorker(library, {"maxPendingBytes": 1024 * 1024}) as worker:
        capabilities = worker.host.call("capabilities")
        assert capabilities["abiVersion"] == 1
        assert capabilities["livePdfExport"] is native_skia
        info = worker.wait(worker.submit({"op": "new"}))["info"]
        target = {"documentId": info["documentId"], "revision": info["revision"]}
        command = {
            "op": "exportPdf",
            "target": target,
            "fontGeneration": info["fontGeneration"],
            "firstPage": 0,
            "pageCount": 1,
            "maxOutputBytes": 1024 * 1024,
        }
        assert capabilities["maxLivePdfPages"] >= 1
        assert capabilities["maxLivePdfBytes"] >= command["maxOutputBytes"]
        invalid = dict(command, pageCount=capabilities["maxLivePdfPages"] + 1)
        try:
            worker.submit(invalid)
        except NativeCallError as error:
            assert error.code == "invalidArgument"
        else:
            raise AssertionError("out-of-contract page count was accepted")

        if not native_skia:
            disabled = worker.wait(worker.submit(command))
            assert disabled["code"] == "featureDisabled" and "buffer" not in disabled
            return

        position = {"section": 0, "paragraph": 0, "scalarOffset": 0, "cellPath": []}
        selection = {"anchor": position, "focus": position}
        worker.wait(worker.submit({"op": "beginEdit", "target": target, "selection": selection}))
        replaced = worker.wait(worker.submit(
            {"op": "replace", "target": target, "text": "Worker PDF"}
        ))
        assert replaced["ok"], replaced
        live = worker.wait(worker.submit({"op": "info"}))["info"]
        assert live["isDirty"] and live["canUndo"]

        stale = worker.wait(worker.submit(command))
        assert stale["code"] == "staleRevision" and "buffer" not in stale
        command["target"] = {"documentId": live["documentId"], "revision": live["revision"]}
        stale_font = worker.wait(worker.submit(
            dict(command, fontGeneration=live["fontGeneration"] + 1)
        ))
        assert stale_font["code"] == "invalidArgument" and "buffer" not in stale_font
        short = worker.wait(worker.submit(dict(command, maxOutputBytes=1)))
        assert short["code"] == "limitExceeded" and "buffer" not in short

        request = worker.submit(command)
        try:
            worker.submit(command)
        except NativeCallError as error:
            assert error.code == "byteCapacity"
        else:
            raise AssertionError("an unread PDF completion did not retain its reservation")

        exported = worker.wait(request)
        assert exported["kind"] == "exportedPdf"
        assert exported["target"] == command["target"]
        assert exported["fontGeneration"] == command["fontGeneration"]
        assert exported["firstPage"] == 0 and exported["pageCount"] == 1
        assert worker.wait(worker.submit({"op": "info"}))["info"] == live

        # Once received, the bytes use the host's retained-buffer contract.
        worker.wait_closed()
        data = worker.copy_buffer(exported)
        assert data.startswith(b"%PDF-") and data.rstrip().endswith(b"%%EOF")
        try:
            worker.copy_buffer(exported)
        except NativeCallError:
            pass
        else:
            raise AssertionError("PDF buffer could be consumed twice")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--library", type=Path, required=True)
    parser.add_argument("--native-skia", choices=("enabled", "disabled"), required=True)
    args = parser.parse_args()
    run(args.library, args.native_skia == "enabled")
    print("Native live PDF worker contract passed")
