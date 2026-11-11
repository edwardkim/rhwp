#!/usr/bin/env python3
"""Portable rejection controls, never proof of actual Windows execution.

Receipt fixtures describe Native ownership contracts. Pixel counterexamples use
synthetic RGB shapes and independent high-resolution shapes, not layout goldens.
"""
import copy
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
from types import SimpleNamespace

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tool"))
import qualify_windows_tile_group as qualification
from windows_continuous_qualification_contract import stage as single_stage
import desktop_qualification_contract as desktop_contracts
from qualify_desktop_texture import runtime_bundle_manifest


def stage(index=1, dpr=1.25):
    width, height = (40, 30) if index < 3 else (44, 32)
    generation = 7 if index < 4 else 8
    ids = (11, 12) if index < 4 else (21, 22)
    serial = index if index < 4 else 1
    snapshot = 19 + index
    stamp = {"documentId": 1, "revision": 2 if index == 1 else 3, "fontGeneration": 0,
        "viewGeneration": index + 2, "deviceGeneration": generation, "deviceScale": dpr}
    placement = {"snapshot": snapshot, "scale": .55, "originX": 3.25, "originY": .5}
    rects = [{"left": 0, "top": 0, "width": width // 2, "height": height},
             {"left": width // 2, "top": 0, "width": width // 2, "height": height}]
    members = []
    for ident in ids:
        member = copy.deepcopy(single_stage(index)["nativeStatus"])
        member.update(textureId=ident, deviceGeneration=generation, acceptedStamp=copy.deepcopy(stamp),
            renderedStamp=copy.deepcopy(stamp), acceptedSnapshots=[snapshot], acceptedSerial=serial,
            renderedSerial=serial, descriptorOpenedSerial=serial, registered=True)
        member["limits"].update(producerOwnedFrames=2, outstandingFrames=2, pendingRetireFrames=2,
            producerOwnedBytes=2 * 1024 * 1024, reservedTargetBytes=2 * 1024 * 1024,
            handedOffFrames=2 * index - 2, descriptorOpenedNotifications=2 * index)
        members.append(member)

    def proof(factor):
        return {"snapshot": snapshot, "stamp": copy.deepcopy(stamp), "coordinateSpace": "viewportPixels",
            "viewport": {"width": width * factor, "height": height * factor},
            "placement": {"scale": .55 * factor, "originX": 3.25 * factor, "originY": .5 * factor},
            "descriptor": {"width": width * factor, "height": height * factor,
                "stride": width * factor * 4, "pixelFormat": "rgba8Premultiplied", "colorSpace": "srgb", "origin": "topLeft"}}
    result = {"protocol": 1, "nonce": "owned", "index": index, "pid": 42,
        "platform": "windows", "stage": qualification.STAGES[index - 1], "width": width, "height": height,
        "deviceScale": dpr, "physicalOffset": [0, 0], "stamp": stamp, "snapshotIds": [snapshot],
        "flutterView": {"width": 1280 if index < 3 else 1312, "height": 720 if index < 3 else 752, "deviceScale": dpr},
        "tileRects": rects, "group": {"groupId": ids[0], "deviceGeneration": generation, "members": members},
        "submission": {"groupId": ids[0], "deviceGeneration": generation, "stamp": copy.deepcopy(stamp),
            "tiles": [{"textureId": ident, "width": width // 2, "height": height,
                "placements": [{**placement, "originX": placement["originX"] - rect["left"]}]} for ident, rect in zip(ids, rects)]},
        "acceptance": {"accepted": True, "acceptedStamp": copy.deepcopy(stamp), "acceptedSnapshots": [snapshot]},
        "strips": [{"left": 0, "width": width, "cpu": proof(1), "reference": proof(2)}]}
    if index == 4:
        result["recoveryRetirement"] = closed(stage(3, dpr))
    return result


def closed(value):
    group = copy.deepcopy(value["group"])
    for member in group["members"]:
        member.update(closed=True, ownerFinished=True, unregistered=True, registered=False,
            ready=False, inFlight=False, hasFrame=False, renderedStamp=None, renderedSerial=0, descriptorOpenedSerial=0)
        for key in ("producerOwnedFrames", "producerOwnedBytes", "outstandingFrames", "pendingRetireFrames",
                    "reservedTargetBytes", "descriptorTickets", "descriptorCpuPendingTickets"):
            member["limits"][key] = 0
    return group


def completion(dpr=1.25):
    return {"protocol": 1, "nonce": "owned", "ok": True, "platform": "windows", "publications": 4,
        "captures": 4, "callerReleases": 0, "consumerGpuCompletionVerified": False,
        "hardwareAccelerationVerified": False, "closed": closed(stage(4, dpr)),
        "recoveryRetirement": closed(stage(3, dpr)), "stats": dict.fromkeys(qualification.NATIVE_STATS, 0)}


def capture_evidence(index, dpr):
    value = stage(index, dpr)
    total = value["width"] * value["height"]
    return {"stage": value, "window": value["flutterView"], "compositorSha256": "c" * 64,
        "comparison": {"seam": value["tileRects"][1]["left"], "cpuSha256": "a" * 64, "referenceSha256": "b" * 64,
            "comparison": {"comparison": "opaque-interiors-1-source-palette13-edge1px-zero-integer-shift",
                "sourceReferenceScale": 2, "sourceReferenceBytes": total * 16,
                "uniformInteriorPixels": total // 2, "aaEdgeDifferentPixels": 0, "maxChannelDifference": 0}}}


def receipt(dpr=1.25):
    return {"qualification": "windows-two-tile-consumer-passed", "exitCode": 0, "platform": "windows",
        "pid": 42, "nonce": "owned", "capture": "visible-win32-desktop", "consumerGpuCompletionVerified": False,
        "hardwareAccelerationVerified": False, "targetReuse": False, "consumerAndDriverMemoryIncluded": False,
        "windowResize": {"before": stage(2, dpr)["flutterView"], "after": stage(3, dpr)["flutterView"]},
        "stages": [capture_evidence(i, dpr) for i in range(1, 5)],
        "collectorSourceFiles": qualification.collector_sources(),
        "result": completion(dpr), "build": {"sourceHead": "1" * 40, "nativeSourceHead": "2" * 40,
            "flutterRevision": "3" * 40, "engineRevision": "4" * 40, "runtimeBundle": {"files": {}},
            "sourceFiles": qualification.collector_sources(),
            "artifacts": {key: {"sha256": "a" * 64} for key in
                ("runner", "native", "dispatcher", "plugin", "engineRuntime", "fixture")}}}


def pixels(index=1, width=40, height=30, factor=1):
    # Two independently specified geometric shapes crossing the actual tile seam.
    # This has no relationship to rhwp document-layout expected coordinates.
    data = bytearray([255] * (width * height * factor * factor * 4))
    for y in range(height * factor):
        for x in range(width * factor):
            color = None
            if 8 * factor <= x < 28 * factor and 8 * factor <= y < 24 * factor:
                color = [40 if index == 1 else 60] * 3 + [255]
            if 17 * factor <= x < 20 * factor and 2 * factor <= y < 28 * factor:
                color = [0, 0, 255, 255]
            if color:
                offset = (y * width * factor + x) * 4
                data[offset:offset + 4] = bytes(color)
    return bytes(data)


class WindowsTileContracts(unittest.TestCase):
    def validate(self, value, index=1):
        return qualification.validate_stage(value, nonce="owned", index=index, pid=42)

    def test_four_actual_publication_contracts_include_same_owner_recovery(self):
        stages = [stage(i) for i in range(1, 5)]
        for index, current in enumerate(stages, 1):
            self.assertEqual(self.validate(current, index), (current["width"], current["height"], (0, 0)))
            if index > 1:
                qualification.validate_transition(stages[index - 2], current)
        qualification.validate_result(completion(), nonce="owned", stages=stages)

    def test_wrong_process_platform_nonce_or_stage_is_rejected(self):
        for key, replacement in (("protocol", True), ("protocol", 2), ("index", True),
                                  ("pid", 43), ("nonce", "foreign"), ("platform", "linux"), ("stage", "resize")):
            value = stage(); value[key] = replacement
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.validate(value)

    def test_malformed_or_unbounded_metadata_is_rejected(self):
        for mutate in (lambda v: v.update(width=True), lambda v: v.update(height=513),
            lambda v: v.update(width=2049), lambda v: v.update(physicalOffset=[.5, 0]),
            lambda v: v["stamp"].update(fontGeneration=False), lambda v: v["stamp"].update(deviceScale=float("nan")),
            lambda v: v.update(group={}), lambda v: v.update(strips=None), lambda v: v["group"].update(members=None),
            lambda v: v["flutterView"].update(width=1), lambda v: v["flutterView"].update(deviceScale=2)):
            value = stage(); mutate(value)
            with self.subTest(value=value), self.assertRaises(ValueError): self.validate(value)

    def test_missing_aliasing_or_stale_member_is_rejected(self):
        for mutate in (lambda v: v["group"]["members"].pop(),
            lambda v: v["group"]["members"][1].update(textureId=11),
            lambda v: v["group"]["members"][1].update(deviceGeneration=8),
            lambda v: v["group"]["members"][1]["renderedStamp"].update(revision=0),
            lambda v: v["group"]["members"][1]["renderedStamp"].update(fontGeneration=False),
            lambda v: v["group"]["members"][1].update(descriptorOpenedSerial=0)):
            value = stage(); mutate(value)
            with self.assertRaises(ValueError): self.validate(value)

    def test_missing_or_partial_native_acceptance_receipt_is_rejected(self):
        for mutate in (lambda v: v.pop("acceptance"), lambda v: v["acceptance"].update(accepted=False),
            lambda v: v["acceptance"].update(acceptedSnapshots=[]), lambda v: v["acceptance"].update(acceptedSnapshots=[20, 20]),
            lambda v: v["acceptance"]["acceptedStamp"].update(revision=4),
            lambda v: v["group"]["members"][1].update(acceptedSnapshots=[99])):
            value = stage(); mutate(value)
            with self.assertRaises(ValueError): self.validate(value)

    def test_gap_overlap_and_different_source_transform_are_rejected(self):
        for mutate in (lambda v: v["tileRects"][1].update(left=21),
            lambda v: v["tileRects"][1].update(left=19), lambda v: v["tileRects"][1].update(height=29),
            lambda v: v["submission"]["tiles"][1]["placements"][0].update(originX=0),
            lambda v: v["submission"]["tiles"][1]["placements"][0].update(snapshot=99)):
            value = stage(); mutate(value)
            with self.assertRaises(ValueError): self.validate(value)

    def test_oracle_must_cross_seam_and_use_same_immutable_source(self):
        for mutate in (lambda v: v["strips"][0].update(left=1), lambda v: v["strips"][0].update(width=20),
            lambda v: v["strips"][0]["reference"].update(snapshot=99),
            lambda v: v["strips"][0]["reference"]["placement"].update(originX=0),
            lambda v: v["strips"][0]["reference"]["descriptor"].update(stride=3200)):
            value = stage(); mutate(value)
            with self.assertRaises(ValueError): self.validate(value)

    def test_actual_opened_members_preserve_existing_ownership_caps(self):
        for key, replacement in (("maxFrames", 5), ("maxTargetBytes", 128 * 1024 * 1024),
            ("maxDescriptorTickets", 9), ("consumerGpuCompletionVerified", True), ("targetReuse", True),
            ("producerOwnedFrames", 1), ("descriptorOpenedNotifications", 1), ("wholeFrameReadbacks", 1)):
            value = stage(); value["group"]["members"][1]["limits"][key] = replacement
            with self.subTest(key=key), self.assertRaises(ValueError): self.validate(value)

    def test_revision_resize_and_recovery_keep_document_and_font_owner(self):
        for index in (2, 3, 4):
            for key, replacement in (("documentId", 2), ("fontGeneration", 1), ("deviceScale", 2), ("viewGeneration", 1)):
                value = stage(index); value["stamp"][key] = replacement
                with self.subTest(index=index, key=key), self.assertRaises(ValueError):
                    qualification.validate_transition(stage(index - 1), value)
        value = stage(3); value.update(width=40, height=30)
        with self.assertRaises(ValueError): qualification.validate_transition(stage(2), value)

    def test_recovery_requires_confirmed_old_retirement_before_fresh_generation(self):
        for mutate in (lambda v: v.pop("recoveryRetirement"),
            lambda v: v["recoveryRetirement"]["members"][0].update(unregistered=False),
            lambda v: v["recoveryRetirement"]["members"][1]["limits"].update(descriptorTickets=1),
            lambda v: v["recoveryRetirement"].update(groupId=99), lambda v: v["stamp"].update(deviceGeneration=7)):
            value = stage(4); mutate(value)
            with self.assertRaises(ValueError): qualification.validate_transition(stage(3), value)

    def test_reused_flutter_texture_numbers_are_allowed_only_with_new_native_generation(self):
        value = stage(4)
        value["group"]["groupId"] = 11
        for member, ident in zip(value["group"]["members"], (11, 12)):
            member["textureId"] = ident
        qualification.validate_transition(stage(3), value)
        value["stamp"]["deviceGeneration"] = 7
        with self.assertRaises(ValueError): qualification.validate_transition(stage(3), value)

    def test_final_result_rejects_retained_native_objects_or_gpu_fence_claim(self):
        for mutate in (lambda v: v["stats"].update(leases=1), lambda v: v.update(consumerGpuCompletionVerified=True),
            lambda v: v.update(callerReleases=1), lambda v: v.update(captures=3),
            lambda v: v["closed"]["members"][1].update(ownerFinished=False),
            lambda v: v["closed"]["members"][0]["limits"].update(producerOwnedFrames=1)):
            value = completion(); mutate(value)
            with self.assertRaises(ValueError):
                qualification.validate_result(value, nonce="owned", stages=[stage(i) for i in range(1, 5)])

    def test_real_dpr_pair_requires_two_complete_runs_of_identical_built_inputs(self):
        qualification.validate_dpr_pair(receipt(1), receipt(1.25))
        with self.assertRaises(ValueError): qualification.validate_dpr_pair(receipt(1), receipt(1))
        for mutate in (lambda v: v.update(qualification="pending"), lambda v: v["stages"].pop(),
            lambda v: v["build"].update(sourceHead="5" * 40), lambda v: v.update(capture="producer-readback"),
            lambda v: v["stages"][0]["window"].update(deviceScale=2), lambda v: v.pop("windowResize"),
            lambda v: v.update(consumerGpuCompletionVerified=True),
            lambda v: v["stages"][0].pop("comparison"), lambda v: v["stages"][1].pop("compositorSha256"),
            lambda v: v["collectorSourceFiles"].pop("bindings/Flutter/tool/qualify_desktop_texture.py"),
            lambda v: v["build"]["artifacts"]["plugin"].update(sha256="b" * 64),
            lambda v: v["result"]["stats"].update(snapshotHandles=1)):
            value = receipt(1.25); mutate(value)
            with self.assertRaises(ValueError): qualification.validate_dpr_pair(receipt(1), value)

    def test_linux_cannot_invoke_actual_windows_entry_point(self):
        if sys.platform == "win32":
            self.skipTest("negative Linux entry-point control")
        tool = Path(qualification.__file__)
        result = subprocess.run([sys.executable, "-B", str(tool), *[argument for name in
            ("runner", "native", "dispatcher", "fixture", "build-receipt", "output") for argument in ("--" + name, "missing")]],
            capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("actual Windows desktop is required", result.stderr)

    def test_real_comparison_rejects_one_column_seam_gap_shift_and_cover(self):
        value = stage()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            expected = pixels()
            (root / "cpu-1-0.rgba").write_bytes(expected)
            (root / "reference-1-0.rgba").write_bytes(pixels(factor=2))
            result = qualification.compare_stage(value, root, expected, 1)
            self.assertEqual(result["seam"], 20)
            gap = bytearray(expected)
            for y in range(30):
                offset = (y * 40 + 20) * 4
                gap[offset:offset + 4] = b"\xff" * 4
            shifted = b"".join(b"\xff" * 4 + expected[y * 160:(y + 1) * 160 - 4] for y in range(30))
            for captured in (gap, shifted, bytes([85, 85, 85, 255]) * 1200):
                with self.assertRaises(ValueError): qualification.compare_stage(value, root, captured, 1)

    def test_cpu_input_cannot_grow_past_its_declared_bound(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "source.rgba"; path.write_bytes(b"a" * 65)
            with self.assertRaises(ValueError): qualification.read_pixels(path, 64)


class WindowsTileCollectorControls(unittest.TestCase):
    """Simulated process/Win32 boundaries; these are not qualification evidence."""
    def run_collector(self, expire=None, mutate_build=None):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); runtime = root / "runtime"; runtime.mkdir()
            desktop_contracts.RuntimeBundleContracts.bundle(self, runtime)
            _, supplied, build = desktop_contracts.BuildAndRuntimeIdentityContracts.artifacts(self, runtime)
            build.update(platform="windows", probeKind="twoTileGroup", buildExitCode=0,
                runtimeBundle=runtime_bundle_manifest(runtime),
                sourceHead="1" * 40, nativeSourceHead="2" * 40, flutterRevision="3" * 40, engineRevision="4" * 40,
                sourceFiles=qualification.collector_sources())
            if mutate_build: mutate_build(build)
            qualification.write_json(root / "build.json", build)
            output = root / "capture"
            clock = SimpleNamespace(now=0.0, reads=0, bundle_checks=0)
            case = SimpleNamespace(captures=0, config=None, killed=False)
            process = SimpleNamespace(pid=42, returncode=None)
            process.poll = lambda: process.returncode
            def kill():
                case.killed = True; process.returncode = -9
            process.kill = kill
            def wait(**kwargs):
                process.returncode = 0
                if expire == "close": clock.now = 31
                return 0
            process.wait = wait
            def launch(*args, **kwargs):
                case.config = qualification.read_json(kwargs["env"]["RHWP_CONSUMER_PROBE_CONFIG"])
                for index in range(1, 5):
                    value = stage(index); value["nonce"] = case.config["nonce"]
                    qualification.write_json(output / f"stage-{index}.json", value)
                    for factor, name in ((1, "cpu"), (2, "reference")):
                        (output / f"{name}-{index}-0.rgba").write_bytes(pixels(index, value["width"], value["height"], factor))
                result = completion(); result["nonce"] = case.config["nonce"]
                qualification.write_json(output / "result.json", result)
                return process
            def capture(width, height, offset):
                index = case.captures // 2 + 1
                self.assertEqual((width, height, offset), (stage(index)["width"], stage(index)["height"], (0, 0)))
                case.captures += 1
                if expire == "held" and case.captures == 2: clock.now = 31
                return pixels(index, width, height)
            compositor = SimpleNamespace(capture=capture,
                metrics=lambda: stage(case.captures // 2 + 1)["flutterView"],
                resize_client=lambda: {"before": stage(2)["flutterView"], "after": stage(3)["flutterView"]})
            actual_compare = qualification.compare_stage
            def compare(*args, **kwargs):
                result = actual_compare(*args, **kwargs)
                if expire == "comparison": clock.now = 31
                return result
            actual_bundle = qualification.verify_runtime_bundle
            def bundle(*args, **kwargs):
                result = actual_bundle(*args, **kwargs)
                clock.bundle_checks += 1
                if expire == "identity" and clock.bundle_checks == 2: clock.now = 31
                return result
            def monotonic():
                clock.reads += 1
                if expire == "ready" and clock.reads > 1: clock.now = 31
                return clock.now
            argv = ["qualify", *[argument for key, value in supplied.items() for argument in ("--" + key, str(value))],
                "--build-receipt", str(root / "build.json"), "--output", str(output), "--timeout", "30"]
            with mock.patch.object(sys, "argv", argv), mock.patch.object(qualification, "require_actual_windows"), \
                 mock.patch.object(qualification.subprocess, "Popen", launch), \
                 mock.patch.object(qualification, "TileCompositor", return_value=compositor), \
                 mock.patch.object(qualification, "compare_stage", compare), \
                 mock.patch.object(qualification, "verify_runtime_bundle", bundle), \
                 mock.patch.object(qualification.time, "monotonic", monotonic), \
                 mock.patch.object(qualification.time, "sleep"):
                if expire or mutate_build:
                    with self.assertRaises(ValueError): qualification.main()
                else:
                    qualification.main()
            if mutate_build:
                self.assertFalse(output.exists())
                self.assertIsNone(case.config)
                return
            result = qualification.read_json(output / "receipt.json", limit=2 * qualification.MAX_BUILD_RECEIPT_BYTES)
            self.assertEqual(case.config["backend"], "windowsTwoTileGroup")
            if expire:
                self.assertEqual(result["qualification"], "failed")
                self.assertIn("deadline", result["error"])
                if expire in ("ready", "comparison", "held"):
                    self.assertFalse((output / "ack-1.json").exists())
                    self.assertTrue(case.killed)
            else:
                self.assertEqual(result["qualification"], "windows-two-tile-consumer-passed")
                self.assertEqual(case.captures, 8)
                self.assertFalse(result["actualDprComparison"])
                self.assertEqual(result["windowResize"]["after"], stage(3)["flutterView"])

    def test_all_four_stages_capture_twice_and_close_without_claiming_multiple_dprs(self):
        self.run_collector()

    def test_expired_work_never_acknowledges_or_reports_success(self):
        for boundary in ("ready", "comparison", "held", "close", "identity"):
            with self.subTest(boundary=boundary): self.run_collector(expire=boundary)

    def test_legacy_build_or_changed_collector_cannot_launch_as_two_tile_qualification(self):
        for mutate in (lambda b: b.update(probeKind="singleTexture"),
                       lambda b: b["sourceFiles"].update({"bindings/Flutter/tool/qualify_windows_tile_group.py": "0" * 64}),
                       lambda b: b["sourceFiles"].update({"bindings/Flutter/tool/qualify_desktop_texture.py": "0" * 64}),
                       lambda b: b["sourceFiles"].update({"bindings/Flutter/tool/qualify_windows_continuous.py": "0" * 64})):
            self.run_collector(mutate_build=mutate)


if __name__ == "__main__":
    unittest.main()
