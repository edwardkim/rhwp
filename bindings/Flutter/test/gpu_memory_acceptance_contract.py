#!/usr/bin/env python3
"""Portable adversarial receipts, never actual GPU/RSS/OS execution evidence."""

import argparse
import copy
import hashlib
import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tool"))
import gpu_memory_acceptance as memory
import qualify_linux_texture as linux
import qualify_windows_continuous as windows
import qualify_linux_tile_group as linux_group
import linux_qualification_contract as linux_cases
import windows_continuous_qualification_contract as windows_cases

spec = importlib.util.spec_from_file_location("memory_mac", ROOT / "macos/tests/qualify_native_view.py")
mac = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mac)


def evidence(cycles=2, group=False, publication_profile="continuous32To64"):
    frames = 3 if publication_profile == "linuxThreeStageTileGroup" else 32
    identity = {"textureId": 4, "deviceGeneration": 1, "memberTextureIds": [4, 5] if group else []}
    stages = []
    for index in range(1, frames + 1):
        stamp = {"documentId": 1, "revision": 1, "fontGeneration": 0,
                 "deviceGeneration": 1, "deviceScale": 1.25, "viewGeneration": index}
        stages.append({"index": index, "stamp": stamp, "memoryOwner": copy.deepcopy(identity),
                       "nativeStatus": {"textureId": 4, "deviceGeneration": 1},
                       **({"tileTextureIds": [4, 5], "groupId": 4} if group else {})})

    def sample(after, ordinal):
        stamp = copy.deepcopy(stages[after - 1]["stamp"])
        result = {"owner": copy.deepcopy(identity), "stamp": stamp, "rssBytes": 120000000,
                "canDisplay": True, "producerDiagnostics": None,
                "traceCounts": {"captureStarted": after, "submitStarted": after,
                                "statusPolled": ordinal * 3, "closeRequested": 0},
                "native": {"deviceGeneration": 1, "renderedStamp": stamp,
                           "progress": {"acceptedSerial": after, "renderedSerial": after}}}
        if group:
            result["native"]["members"] = [{"textureId": texture, "deviceGeneration": 1,
                "renderedStamp": copy.deepcopy(stamp), "progress": copy.deepcopy(result["native"]["progress"])}
                for texture in identity["memberTextureIds"]]
        return result

    def trim(texture, ordinal):
        noop = ordinal % 2 == 0
        return {"supported": True, "textureId": texture, "deviceGeneration": 1, "requestId": ordinal,
                "targetBytes": 0, "scope": "managedGpuTargets", "before": 4096,
                "after": 4096 if noop else 0, "trimmed": 0 if noop else 4096,
                "pending": False, "busy": noop, "reason": "pinned" if noop else "trimmed"}

    observations = []
    for ordinal, after in enumerate(memory.memory_pressure_schedule(frames, cycles, publication_profile), 1):
        before = sample(after, ordinal)
        receipt = trim(4, ordinal)
        if group:
            members = [trim(texture, ordinal) for texture in [4, 5]]
            receipt.update(groupId=4, members=members,
                           **{key: sum(member[key] for member in members) for key in ("before", "after", "trimmed")})
        samples = [copy.deepcopy(before) for _ in range(3)]
        for value, elapsed in zip(samples, [1000, 221000, 441000]):
            value["elapsedMicros"] = elapsed
        samples[-1]["traceCounts"]["statusPolled"] += 1
        observations.append({"cycle": ordinal, "afterPublication": after,
                             "trigger": "flutter/system:memoryPressure",
                             "observer": "NativeGpuLifecycle.didHaveMemoryPressure",
                             "before": before, "trim": receipt,
                             "outcome": "observedNoop" if receipt["trimmed"] == 0 else "observedReclamation",
                             "idle": {"durationMicros": 442000, "probeStatusPolls": 3,
                                      "controllerStatusPolls": 1, "samples": samples}})
    value = {"publications": frames, "consumerGpuCompletionVerified": False,
             "closed": {"textureId": 4, "deviceGeneration": 1},
             "memoryAcceptance": {"schemaVersion": 1, "scope": memory.SCOPE, "pid": 42,
                "publicationProfile": publication_profile,
                "requestedCycles": cycles, "rssScope": "probeProcessRss", "rssStartBytes": 100000000,
                "physicalVramBytes": None, "consumerAndDriverMemoryIncluded": False,
                "consumerGpuCompletionVerified": False, "cycles": observations,
                "close": {"owner": copy.deepcopy(identity), "controllerPhase": "closed",
                          "retirementPending": False, "rssBytes": 110000000,
                          "traceCounts": {"captureStarted": frames, "submitStarted": frames,
                                          "statusPolled": cycles * 4, "closeRequested": 1}}}}
    if group:
        value["closed"]["members"] = [{"textureId": texture, "deviceGeneration": 1}
                                      for texture in identity["memberTextureIds"]]
    return value, stages


class MemoryAcceptanceContracts(unittest.TestCase):
    def validate(self, value, stages, cycles=2):
        return memory.validate_memory_acceptance(value, cycles=cycles, stages=stages, pid=42)

    def test_default_flag_preserves_existing_protocol(self):
        parser = argparse.ArgumentParser()
        memory.add_memory_arguments(parser)
        self.assertEqual(parser.parse_args([]).memory_pressure_cycles, 0)
        self.assertIsNone(self.validate({}, [], cycles=0))
        self.assertEqual(memory.memory_pressure_schedule(3, 0), [])
        value, stages = evidence()
        with self.assertRaises(ValueError):
            self.validate(value, stages, cycles=0)

    def test_cycles_have_bounded_placement_and_later_publications(self):
        for frames in (32, 64):
            for cycles in range(1, 5):
                schedule = memory.memory_pressure_schedule(frames, cycles)
                self.assertEqual(schedule[0], 3)
                self.assertEqual(len(set(schedule)), cycles)
                self.assertTrue(all(3 <= index < frames for index in schedule))
        for frames, cycles in [(31, 1), (65, 1), (32, -1), (32, 5), (32, True), (32, 1.0)]:
            with self.subTest(frames=frames, cycles=cycles), self.assertRaises(ValueError):
                memory.memory_pressure_schedule(frames, cycles)

    def test_actual_receipt_shape_retains_noop_and_process_rss_scope(self):
        value, stages = evidence()
        summary = self.validate(value, stages)
        self.assertEqual(summary["outcomes"], ["observedReclamation", "observedNoop"])
        self.assertEqual(summary["managedBytesTrimmed"], 4096)
        self.assertEqual(summary["resumedPublications"], [4, 18])
        self.assertIsNone(summary["physicalVramBytes"])

    def test_missing_unsupported_pending_foreign_or_invalid_trim_is_rejected(self):
        changes = [("supported", False), ("pending", True), ("textureId", 5),
                   ("deviceGeneration", 2), ("requestId", False), ("requestId", 0),
                   ("targetBytes", 1), ("scope", "driverVram"), ("before", True),
                   ("after", 4097), ("trimmed", 4095), ("busy", 1), ("reason", "x" * 129)]
        for key, replacement in changes:
            value, stages = evidence()
            value["memoryAcceptance"]["cycles"][0]["trim"][key] = replacement
            with self.subTest(key=key, replacement=replacement), self.assertRaises(ValueError):
                self.validate(value, stages)
        value, stages = evidence()
        del value["memoryAcceptance"]["cycles"][0]["trim"]
        with self.assertRaises(ValueError):
            self.validate(value, stages)

    def test_group_members_sums_request_order_and_busy_state_remain_authoritative(self):
        value, stages = evidence(group=True)
        self.assertEqual(self.validate(value, stages)["managedBytesTrimmed"], 8192)
        for change in ("missing", "order", "sum", "request", "busy", "nested"):
            value, stages = evidence(group=True)
            trim = value["memoryAcceptance"]["cycles"][0]["trim"]
            if change == "missing": trim["members"].pop()
            if change == "order": trim["members"].reverse()
            if change == "sum": trim["before"] += 1
            if change == "request": trim["members"][1]["requestId"] += 1
            if change == "busy": trim["busy"] = True
            if change == "nested": trim["members"][0]["members"] = []
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.validate(value, stages)

    def test_noop_cannot_be_claimed_as_reclamation(self):
        value, stages = evidence()
        value["memoryAcceptance"]["cycles"][1]["outcome"] = "observedReclamation"
        with self.assertRaisesRegex(ValueError, "noop"):
            self.validate(value, stages)

    def test_idle_trace_progress_rss_and_clock_are_not_invented(self):
        for change in ("capture", "submit", "missingTrace", "progress", "scope", "elapsed", "duration",
                       "rss", "pollDelta", "pollCount", "missing", "stale"):
            value, stages = evidence()
            idle = value["memoryAcceptance"]["cycles"][0]["idle"]
            last = idle["samples"][-1]
            if change == "capture": last["traceCounts"]["captureStarted"] += 1
            if change == "submit": last["traceCounts"]["submitStarted"] += 1
            if change == "missingTrace": last["traceCounts"]["captureStarted"] = 0
            if change == "progress": last["native"]["progress"]["acceptedSerial"] += 1
            if change == "scope": last["owner"]["deviceGeneration"] += 1
            if change == "elapsed": last["elapsedMicros"] = 0
            if change == "duration": idle["durationMicros"] = 399999
            if change == "rss": last["rssBytes"] = True
            if change == "pollDelta": idle["controllerStatusPolls"] = 0
            if change == "pollCount": idle["probeStatusPolls"] = 4
            if change == "missing": idle["samples"].pop()
            if change == "stale": last["stamp"]["viewGeneration"] += 1
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.validate(value, stages)

    def test_cycle_and_session_identity_must_cover_the_entire_run(self):
        for change in ("count", "order", "request", "index", "boolean", "owner", "resume", "close", "closedOwner", "trace", "publications"):
            value, stages = evidence()
            cycles = value["memoryAcceptance"]["cycles"]
            if change == "count": cycles.pop()
            if change == "order": cycles.reverse()
            if change == "request": cycles[1]["trim"]["requestId"] = 1
            if change == "index": cycles[0]["afterPublication"] = 32
            if change == "boolean": cycles[0]["cycle"] = True
            if change == "owner": stages[8]["memoryOwner"]["textureId"] = 5
            if change == "resume": stages[3]["stamp"]["viewGeneration"] = 3
            if change == "close": value["memoryAcceptance"]["close"]["retirementPending"] = True
            if change == "closedOwner": value["closed"]["deviceGeneration"] = 2
            if change == "trace": cycles[1]["before"]["traceCounts"]["statusPolled"] = 0
            if change == "publications": value["publications"] = 31
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.validate(value, stages)

    def test_consumer_completion_and_physical_vram_remain_unknown(self):
        for key, replacement in [("pid", 43), ("schemaVersion", True), ("requestedCycles", True),
                                 ("rssStartBytes", -1), ("scope", "globalVram"),
                                 ("physicalVramBytes", 0), ("consumerAndDriverMemoryIncluded", True),
                                 ("consumerGpuCompletionVerified", True)]:
            value, stages = evidence()
            value["memoryAcceptance"][key] = replacement
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.validate(value, stages)

    def test_published_frame_metrics_are_not_live_post_trim_measurements(self):
        value, stages = evidence()
        sample = value["memoryAcceptance"]["cycles"][0]["idle"]["samples"][0]
        sample["producerDiagnostics"] = {"schemaVersion": 1, "backend": "openGlGanesh",
            "scope": "publishedFramePostRenderSnapshot", "stamp": copy.deepcopy(sample["stamp"]),
            "members": 1, "counters": {"cachedBytes": 8192}}
        self.validate(value, stages)
        sample["producerDiagnostics"]["scope"] = "liveAfterTrim"
        with self.assertRaises(ValueError):
            self.validate(value, stages)

    def test_collectors_require_memory_evidence_only_when_explicitly_requested(self):
        value, stages = evidence()
        base_linux = linux_cases.result()
        base_windows = windows_cases.completion()
        base_mac = {"protocol": 1, "nonce": "owned", "ok": True, "publications": 32,
                    "consumerGpuCompletionVerified": False, "continuousPresentationSupported": True,
                    "closed": {"ownerFinished": True, "registered": False, "nativeViewLayerDetached": True,
                               "quarantined": False, "frames": 32},
                    "stats": dict.fromkeys(linux_cases.result()["stats"], 0)}
        validators = [(linux.validate_result, base_linux, {}),
                      (windows.validate_result, base_windows, {"last_stage": windows_cases.stage(32)}),
                      (mac.validate_result, base_mac, {})]
        for validate, base, extra in validators:
            with self.subTest(collector=validate.__module__):
                validate(base, nonce="owned", frames=32, **extra)
                with self.assertRaises(ValueError):
                    validate(base, nonce="owned", frames=32, memory_cycles=2, stages=stages, pid=42, **extra)
                base["closed"].update(textureId=4, deviceGeneration=1)
                base["memoryAcceptance"] = copy.deepcopy(value["memoryAcceptance"])
                self.assertEqual(validate(base, nonce="owned", frames=32, memory_cycles=2,
                                          stages=stages, pid=42, **extra)["cycles"], 2)

    def test_memory_collectors_are_pinned_to_the_actual_build_sources(self):
        with tempfile.TemporaryDirectory() as directory:
            collector = Path(directory) / "qualify_linux_texture.py"
            collector.write_bytes(b"portable identity control")
            helper = Path(memory.__file__)
            expected = {"bindings/Flutter/tool/gpu_memory_acceptance.py": hashlib.sha256(helper.read_bytes()).hexdigest(),
                        "bindings/Flutter/tool/qualify_linux_texture.py": hashlib.sha256(collector.read_bytes()).hexdigest()}
            self.assertEqual(memory.verify_memory_sources({"sourceFiles": expected}, collector), expected)
            collector.write_bytes(b"changed collector")
            with self.assertRaises(ValueError):
                memory.verify_memory_sources({"sourceFiles": expected}, collector)

    def test_linux_tile_profile_preserves_three_publications_and_later_captures(self):
        profile = "linuxThreeStageTileGroup"
        self.assertEqual(memory.memory_pressure_schedule(3, 1, profile), [1])
        self.assertEqual(memory.memory_pressure_schedule(3, 2, profile), [1, 2])
        for frames, cycles in [(32, 1), (3, 3), (3, True)]:
            with self.assertRaises(ValueError): memory.memory_pressure_schedule(frames, cycles, profile)
        with self.assertRaisesRegex(ValueError, '0[.][.]2'):
            memory.memory_pressure_schedule(3, 3, profile)
        value, stages = evidence(group=True, publication_profile=profile)
        summary = memory.validate_memory_acceptance(value, cycles=2, stages=stages, pid=42,
                                                   publication_profile=profile)
        self.assertEqual(summary["resumedPublications"], [2, 3])
        with self.assertRaises(ValueError): self.validate(value, stages)
        value.update(ok=True, nonce="owned", captures=3, callerReleases=0,
                     stats=dict.fromkeys(linux_cases.result()["stats"], 0))
        for member in value["closed"]["members"]:
            member.update(registered=False, cleanupComplete=True,
                **dict.fromkeys(("liveRasterSlots", "pendingRetireSlots", "orphanedRasterSlots", "abandonedTargets",
                                 "orphanedOwners", "snapshotReleaseFailures", "wholeFrameReadbacks"), 0))
        self.assertEqual(linux_group.validate_result(value, "owned", memory_cycles=2,
            stages=stages, pid=42)["resumedPublications"], [2, 3])
        del value["memoryAcceptance"]
        self.assertIsNone(linux_group.validate_result(value, "owned"))
        with self.assertRaises(ValueError): linux_group.validate_result(value, "owned",
            memory_cycles=2, stages=stages, pid=42)

    def test_idle_and_close_retain_both_actual_tile_owners(self):
        profile = "linuxThreeStageTileGroup"
        for change in ("missing", "order", "foreign", "stamp", "progress", "closedMissing", "closedOrder"):
            value, stages = evidence(group=True, publication_profile=profile)
            native = value["memoryAcceptance"]["cycles"][0]["idle"]["samples"][-1]["native"]
            if change == "missing": native["members"].pop()
            if change == "order": native["members"].reverse()
            if change == "foreign": native["members"][1]["deviceGeneration"] += 1
            if change == "stamp": native["members"][1]["renderedStamp"]["revision"] += 1
            if change == "progress": native["members"][1]["progress"]["renderedSerial"] += 1
            if change == "closedMissing": value["closed"]["members"].pop()
            if change == "closedOrder": value["closed"]["members"].reverse()
            with self.subTest(change=change), self.assertRaises(ValueError):
                memory.validate_memory_acceptance(value, cycles=2, stages=stages, pid=42,
                                                  publication_profile=profile)


if __name__ == "__main__":
    unittest.main()
