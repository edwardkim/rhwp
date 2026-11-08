#!/usr/bin/env python3
"""Actual-library accessibility transport contracts; no OS accessibility claims."""

import argparse
import json
from pathlib import Path
import sys
import unittest


sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from native_host import NativeCallError, NativeHost, body_position, collapsed
from native_worker import NativeWorker


TEXT = "가😀a\u0301"
ACCESSIBILITY_RESERVATION = 8 * 1024 * 1024


class NativeAccessibilityContracts(unittest.TestCase):
    library = None

    def setUp(self):
        self.host = NativeHost(self.library)
        capabilities = self.host.call("capabilities")
        self.assertEqual(capabilities["abiVersion"], 1)
        self.assertIs(capabilities.get("accessibilitySnapshots"), True)
        self.handle = self.host.open_bytes()
        self.addCleanup(self.host.call, "close", self.handle)

    def snapshot(self, request=None, **arguments):
        return self.host.request(
            self.handle, op="accessibility", request=request or {}, **arguments
        )["snapshot"]

    def edit(self):
        outcome = self.host.request(
            self.handle, op="beginEdit", expectedRevision=0, selection=collapsed(body_position())
        )["outcome"]
        return self.host.request(
            self.handle, op="replaceSelection", expectedRevision=outcome["revision"], text=TEXT
        )["outcome"]

    def worker_call(self, worker, command):
        response = worker.wait(worker.submit(command))
        self.assertIs(response.get("ok"), True, response)
        return response

    def test_sync_viewer_editor_and_explicit_selection_are_read_only(self):
        self.assertIsNone(self.snapshot()["selection"])
        self.edit()
        before = self.host.request(self.handle, op="info")
        current = self.snapshot()
        self.assertEqual(current["scope"], "bodyAndTableCellText")
        self.assertEqual(current["revision"], before["revision"])
        self.assertEqual(current["fontGeneration"], before["fontGeneration"])
        self.assertEqual(current["nodes"][0]["text"], TEXT)
        self.assertEqual(current["nodes"][0]["end"]["scalarOffset"], len(TEXT))
        self.assertEqual(current["selection"], collapsed(body_position(len(TEXT))))
        self.assertIsNone(current["selectionGeometry"])
        self.assertTrue(all(node["geometry"] is None for node in current["nodes"]))
        explicit = {"anchor": body_position(), "focus": body_position(2)}
        selected = self.snapshot(selection=explicit)
        self.assertEqual(selected["selection"], explicit)
        self.assertEqual(self.snapshot(), current)
        self.assertEqual(self.host.request(self.handle, op="info"), before)

    def test_sync_limits_range_geometry_and_invalid_inputs(self):
        self.edit()
        before = self.host.request(self.handle, op="info")
        limited = self.snapshot({"maxTextBytes": 4})
        self.assertTrue(limited["truncated"])
        self.assertFalse(limited["complete"])
        self.assertIn("textLimit", limited["limitations"])
        self.assertLessEqual(sum(len(node["text"].encode()) for node in limited["nodes"]), 4)
        empty = self.snapshot({"range": {"section": 0, "startParagraph": 0, "endParagraph": 0}})
        self.assertEqual(empty["nodes"], [])
        geometry = self.snapshot({"geometry": True})
        self.assertIsNotNone(geometry["selectionGeometry"])
        self.assertIn(geometry["selectionGeometry"]["status"],
                      ("complete", "unsupported", "unavailable", "limited"))
        self.assertTrue(all(node["geometry"] is not None for node in geometry["nodes"]))
        for request, code in (({"maxNodes": 1001}, "limitExceeded"),
                              ({"unknownField": True}, "invalidArgument")):
            with self.subTest(request=request), self.assertRaises(NativeCallError) as raised:
                self.snapshot(request)
            self.assertEqual(raised.exception.code, code)
        with self.assertRaises(NativeCallError) as raised:
            self.snapshot(selection=collapsed(body_position(100)))
        self.assertEqual(raised.exception.code, "invalidArgument")
        self.assertEqual(self.host.request(self.handle, op="info"), before)

    def test_worker_matches_sync_and_rejects_stale_revision(self):
        self.edit()
        expected = self.snapshot()
        with NativeWorker(self.library) as worker:
            info = self.worker_call(worker, {"op": "new"})["info"]
            initial_target = {"documentId": info["documentId"], "revision": info["revision"]}
            viewer = self.worker_call(worker, {"op": "accessibility", "target": initial_target,
                                               "request": {}})
            self.assertEqual(viewer["kind"], "accessibility")
            self.assertIsNone(viewer["snapshot"]["selection"])
            begun = self.worker_call(worker, {"op": "beginEdit", "target": initial_target,
                                              "selection": collapsed(body_position())})
            self.assertEqual(begun["kind"], "info")
            info = begun["info"]
            target = {"documentId": info["documentId"], "revision": info["revision"]}
            edited = self.worker_call(worker, {"op": "replace", "target": target,
                                               "text": TEXT})["outcome"]
            target["revision"] = edited["revision"]
            before = self.worker_call(worker, {"op": "info"})["info"]
            actual = self.worker_call(worker, {"op": "accessibility", "target": target, "request": {}})
            self.assertEqual(actual["target"], target)
            self.assertEqual(actual["snapshot"], expected)
            stale = worker.wait(worker.submit({"op": "accessibility", "target": initial_target,
                                                "request": {}}))
            self.assertIs(stale.get("ok"), False)
            self.assertEqual(stale["code"], "staleRevision")
            self.assertEqual(self.worker_call(worker, {"op": "info"})["info"], before)

    def test_worker_reserves_snapshot_bytes_until_completion_is_received(self):
        with NativeWorker(self.library, {"maxPendingBytes": ACCESSIBILITY_RESERVATION}) as worker:
            info = self.worker_call(worker, {"op": "new"})["info"]
            target = {"documentId": info["documentId"], "revision": info["revision"]}
            command = {"op": "accessibility", "target": target, "request": {}}
            first = worker.submit(command)
            # Completed-but-unread results still consume their reservation.
            with self.assertRaises(NativeCallError) as raised:
                worker.submit(command)
            self.assertEqual(raised.exception.code, "byteCapacity")
            response = worker.wait(first)
            self.assertIs(response.get("ok"), True, response)
            self.assertEqual(self.worker_call(worker, command)["snapshot"], response["snapshot"])


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument("--library", type=Path, required=True)
    args = parser.parse_args()
    NativeAccessibilityContracts.library = args.library
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(NativeAccessibilityContracts)
    )
    print(json.dumps({"ok": result.wasSuccessful(), "contract": "native-accessibility-transport",
                      "tests": result.testsRun, "osAccessibilityRegistered": False}))
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    raise SystemExit(main())
