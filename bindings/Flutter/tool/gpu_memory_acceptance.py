"""Bounded receipt checks shared by the actual desktop consumer collectors.

Portable callers test rejection only. Runtime evidence comes from the real
probe, platform trim method and later externally sampled publications.
"""

import hashlib
from pathlib import Path

MAX_INTEGER = 2**63 - 1
SCOPE = "managedGpuTargetsAndProcessRss"
TRACE_KEYS = ("captureStarted", "submitStarted", "statusPolled", "closeRequested")
COUNTERS = {"cacheHits", "cacheMisses", "cacheEvictions", "cachedTiles", "cachedBytes",
            "managedTargets", "managedBytes", "canonicalReplays", "viewportFallbacks",
            "targetAdmissionFallbacks"}


def integer(value, minimum=0):
    if type(value) is not int or not minimum <= value <= MAX_INTEGER:
        raise ValueError("memory acceptance contains an invalid integer")
    return value


def memory_pressure_schedule(frames, cycles, publication_profile="continuous32To64"):
    integer(cycles)
    if publication_profile not in ("continuous32To64", "linuxThreeStageTileGroup"):
        raise ValueError("unknown memory-pressure publication profile")
    maximum = 2 if publication_profile == "linuxThreeStageTileGroup" else 4
    if cycles > maximum:
        raise ValueError(f"memory-pressure cycles must be 0..{maximum}")
    if not cycles:
        return []
    if publication_profile == "linuxThreeStageTileGroup":
        if type(frames) is not int or frames != 3:
            raise ValueError("Linux tile memory acceptance retains exactly three publications")
        return list(range(1, cycles + 1))
    if type(frames) is not int or not 32 <= frames <= 64:
        raise ValueError("memory-pressure cycles require 32..64 publications")
    return [3 + (frames - 3) * index // cycles for index in range(cycles)]


def add_memory_arguments(parser, publication_profile="continuous32To64"):
    parser.add_argument("--memory-pressure-cycles", type=int, default=0,
                        help="0 (default) or " + ("1..2 cycles between the existing three Linux tile publications"
                            if publication_profile == "linuxThreeStageTileGroup" else
                            "1..4 actual binding/trim/idle cycles within 32..64 frames"))


def verify_memory_sources(build, collector):
    """An opt-in collector must match the source inputs of its actual runner."""
    expected = build.get("sourceFiles")
    paths = {"bindings/Flutter/tool/gpu_memory_acceptance.py": Path(__file__),
             "bindings/Flutter/" + ("macos/tests/" if collector.name == "qualify_native_view.py" else "tool/") +
             collector.name: collector}
    actual = {}
    for relative, path in paths.items():
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if not isinstance(expected, dict) or expected.get(relative) != digest:
            raise ValueError("memory collector differs from the actual runner build: " + relative)
        actual[relative] = digest
    return actual


def owner(value, stage):
    if not isinstance(value, dict) or value != stage.get("memoryOwner"):
        raise ValueError("memory owner differs from its externally sampled publication")
    integer(value.get("textureId"), 1)
    if integer(value.get("deviceGeneration"), 1) != stage["stamp"]["deviceGeneration"]:
        raise ValueError("memory owner device generation changed")
    members = value.get("memberTextureIds")
    if not isinstance(members, list) or len(members) not in (0, 2):
        raise ValueError("memory owner must identify zero or two tile members")
    for identity in members:
        integer(identity, 1)
    if members and (members[0] == members[1] or members != stage.get("tileTextureIds") or
                    value["textureId"] != stage.get("groupId")):
        raise ValueError("memory tile owner order or group changed")
    native = stage.get("nativeStatus", {})
    if ("textureId" in native and native["textureId"] != value["textureId"]) or \
            integer(native.get("deviceGeneration"), 1) != value["deviceGeneration"]:
        raise ValueError("memory owner differs from its actual native status")
    return value


def trim_receipt(value, identity, *, request=None):
    if not isinstance(value, dict) or value.get("supported") is not True:
        raise ValueError("memory pressure lacks a supported actual trim receipt")
    if integer(value.get("textureId"), 1) != identity["textureId"] or \
            integer(value.get("deviceGeneration"), 1) != identity["deviceGeneration"] or \
            value.get("scope") != "managedGpuTargets" or value.get("pending") is not False:
        raise ValueError("trim owner/scope changed or the receipt remains pending")
    current_request = integer(value.get("requestId"), 1)
    if request is not None and current_request != request:
        raise ValueError("trim member request identity changed")
    if integer(value.get("targetBytes")) != 0:
        raise ValueError("memory pressure must use the lifecycle's zero-byte target")
    before, after, trimmed = (integer(value.get(key)) for key in ("before", "after", "trimmed"))
    if after > before or trimmed != before - after or type(value.get("busy")) is not bool:
        raise ValueError("trim byte totals or busy state are invalid")
    reason = value.get("reason")
    if not isinstance(reason, str) or not reason or len(reason) > 128 or "\0" in reason:
        raise ValueError("trim reason is missing or unbounded")
    ids = identity["memberTextureIds"]
    if ids:
        members = value.get("members")
        if value.get("groupId") != identity["textureId"] or not isinstance(members, list) or len(members) != 2:
            raise ValueError("group trim must retain both native member receipts")
        for member, texture in zip(members, ids):
            trim_receipt(member, {**identity, "textureId": texture, "memberTextureIds": []},
                         request=current_request)
        if any(value[key] != sum(member[key] for member in members) for key in ("before", "after", "trimmed")) or \
                value["busy"] != any(member["busy"] for member in members):
            raise ValueError("group trim totals or busy state differ from its members")
    elif "members" in value or "groupId" in value or (not value["busy"] and after):
        raise ValueError("single-owner trim cannot invent group members or an achieved target")
    return current_request


def trace_counts(value):
    if not isinstance(value, dict) or not all(key in value for key in TRACE_KEYS):
        raise ValueError("idle controller trace counts are missing")
    for key, count in value.items():
        if not isinstance(key, str) or len(key) > 64:
            raise ValueError("idle trace key is unbounded")
        integer(count)
    return value


def sample(value, held, identity):
    if not isinstance(value, dict) or value.get("owner") != identity or \
            value.get("stamp") != held["stamp"] or value.get("canDisplay") is not True:
        raise ValueError("idle sample lost the displayed source or owner")
    integer(value.get("rssBytes"), 1)
    counts = trace_counts(value.get("traceCounts"))
    if any(counts[key] < held["index"] for key in ("captureStarted", "submitStarted")):
        raise ValueError("idle trace did not observe the preceding real publications")
    native = value.get("native")
    if not isinstance(native, dict) or integer(native.get("deviceGeneration"), 1) != identity["deviceGeneration"] or \
            native.get("renderedStamp") != held["stamp"]:
        raise ValueError("idle native sample is stale or from another device")
    progress = native.get("progress")
    if not isinstance(progress, dict) or not progress or \
            not set(progress) <= {"acceptedSerial", "renderedSerial", "framesSubmitted", "frames"}:
        raise ValueError("idle native publication counters are missing")
    for count in progress.values():
        integer(count, 1)
    members = native.get("members")
    if identity["memberTextureIds"]:
        if not isinstance(members, list) or len(members) != 2:
            raise ValueError("idle observation must retain both native tile owners")
        for member, texture in zip(members, identity["memberTextureIds"]):
            if not isinstance(member, dict) or integer(member.get("textureId"), 1) != texture or \
                    integer(member.get("deviceGeneration"), 1) != identity["deviceGeneration"] or \
                    member.get("renderedStamp") != held["stamp"] or not isinstance(member.get("progress"), dict) or \
                    not member["progress"] or not set(member["progress"]) <= set(progress):
                raise ValueError("idle native tile identity/source/progress changed")
            for count in member["progress"].values():
                integer(count, 1)
    elif members is not None:
        raise ValueError("single-owner idle sample cannot invent native members")
    diagnostic = value.get("producerDiagnostics")
    if diagnostic is not None:
        if not isinstance(diagnostic, dict) or type(diagnostic.get("schemaVersion")) is not int or diagnostic["schemaVersion"] != 1 or \
                diagnostic.get("scope") != "publishedFramePostRenderSnapshot" or \
                diagnostic.get("stamp") != held["stamp"] or \
                diagnostic.get("backend") not in ("openGlGanesh", "metalGanesh", "d3d12Ganesh") or \
                integer(diagnostic.get("members"), 1) != (2 if identity["memberTextureIds"] else 1):
            raise ValueError("producer diagnostics must remain a published-frame snapshot")
        counters = diagnostic.get("counters")
        if not isinstance(counters, dict) or not counters or not set(counters) <= COUNTERS:
            raise ValueError("producer diagnostic counters are missing or unknown")
        for counter in counters.values():
            integer(counter)
    return value


def validate_memory_acceptance(result, *, cycles, stages, pid,
                               publication_profile="continuous32To64"):
    """Require pressure -> final trim -> held idle -> later sampled frame -> close."""
    schedule = memory_pressure_schedule(len(stages), cycles, publication_profile)
    memory = result.get("memoryAcceptance")
    if not cycles:
        if memory is not None:
            raise ValueError("memory acceptance was not requested by the collector")
        return None
    if not isinstance(memory, dict) or type(memory.get("schemaVersion")) is not int or memory["schemaVersion"] != 1 or \
            memory.get("scope") != SCOPE or integer(memory.get("pid"), 1) != pid or \
            memory.get("publicationProfile") != publication_profile or \
            integer(memory.get("requestedCycles"), 1) != cycles or memory.get("rssScope") != "probeProcessRss" or \
            "physicalVramBytes" not in memory or memory["physicalVramBytes"] is not None or \
            memory.get("consumerAndDriverMemoryIncluded") is not False or \
            memory.get("consumerGpuCompletionVerified") is not False:
        raise ValueError("memory acceptance identity/scope overstates its actual measurements")
    integer(memory.get("rssStartBytes"), 1)
    if type(result.get("publications")) is not int or result["publications"] != len(stages) or \
            result.get("consumerGpuCompletionVerified") is not False:
        raise ValueError("memory evidence does not cover all externally sampled publications")
    session_owner = owner(stages[0].get("memoryOwner"), stages[0])
    if any(owner(stage.get("memoryOwner"), stage) != session_owner for stage in stages):
        raise ValueError("memory session replaced its native owner")
    observations = memory.get("cycles")
    if not isinstance(observations, list) or len(observations) != cycles:
        raise ValueError("memory pressure cycle evidence is incomplete")
    last_request = 0
    last_counts = None
    trimmed_total = 0
    for index, (cycle, after) in enumerate(zip(observations, schedule), 1):
        if not isinstance(cycle, dict) or integer(cycle.get("cycle"), 1) != index or integer(cycle.get("afterPublication"), 1) != after or \
                cycle.get("trigger") != "flutter/system:memoryPressure" or \
                cycle.get("observer") != "NativeGpuLifecycle.didHaveMemoryPressure":
            raise ValueError("memory cycle order or real binding route changed")
        held, resumed = stages[after - 1], stages[after]
        identity = owner(held.get("memoryOwner"), held)
        if owner(resumed.get("memoryOwner"), resumed) != identity or \
                resumed["stamp"]["viewGeneration"] <= held["stamp"]["viewGeneration"]:
            raise ValueError("no later compositor publication proved the same owner could continue")
        before = sample(cycle.get("before"), held, identity)
        if last_counts is not None and any(before["traceCounts"].get(key, -1) < count for key, count in last_counts.items()):
            raise ValueError("memory cycle lifetime trace counts went backwards")
        request = trim_receipt(cycle.get("trim"), identity)
        if request <= last_request:
            raise ValueError("memory cycles reused or reordered a trim request")
        last_request = request
        trimmed = cycle["trim"]["trimmed"]
        if cycle.get("outcome") != ("observedNoop" if trimmed == 0 else "observedReclamation"):
            raise ValueError("a zero-byte trim must remain an observed noop")
        trimmed_total += trimmed
        idle = cycle.get("idle")
        if not isinstance(idle, dict) or idle.get("probeStatusPolls") != 3:
            raise ValueError("idle observation did not preserve its bounded three status polls")
        duration = integer(idle.get("durationMicros"))
        if not 400000 <= duration <= 30000000:
            raise ValueError("idle observation duration is missing or unbounded")
        samples = idle.get("samples")
        if not isinstance(samples, list) or len(samples) != 3:
            raise ValueError("idle samples are incomplete")
        elapsed = -1
        previous_counts = before["traceCounts"]
        for observed in samples:
            sample(observed, held, identity)
            current = integer(observed.get("elapsedMicros"))
            counts = observed["traceCounts"]
            if not elapsed <= current <= duration or \
                    any(counts.get(key, -1) < count for key, count in previous_counts.items()) or \
                    any(counts[key] != before["traceCounts"][key] for key in ("captureStarted", "submitStarted")) or \
                    observed["native"] != before["native"]:
                raise ValueError("idle source/capture/submission progressed or trace time went backwards")
            elapsed, previous_counts = current, counts
        if samples[-1]["elapsedMicros"] - samples[0]["elapsedMicros"] < 400000 or \
                integer(idle.get("controllerStatusPolls")) != samples[-1]["traceCounts"]["statusPolled"] - samples[0]["traceCounts"]["statusPolled"]:
            raise ValueError("idle status-poll delta does not match the observed trace window")
        last_counts = samples[-1]["traceCounts"]
    closed = memory.get("close")
    if not isinstance(closed, dict) or owner(closed.get("owner"), stages[-1]) != observations[-1]["before"]["owner"] or \
            closed.get("controllerPhase") != "closed" or closed.get("retirementPending") is not False:
        raise ValueError("memory acceptance lacks the final current-owner close receipt")
    integer(closed.get("rssBytes"), 1)
    native_closed = result.get("closed")
    if not isinstance(native_closed, dict) or integer(native_closed.get("deviceGeneration"), 1) != session_owner["deviceGeneration"] or \
            ("textureId" in native_closed and native_closed["textureId"] != session_owner["textureId"]):
        raise ValueError("memory close differs from the actual native close owner")
    if session_owner["memberTextureIds"]:
        members = native_closed.get("members", result.get("nativeTileMembers"))
        if not isinstance(members, list) or len(members) != 2:
            raise ValueError("memory close must retain both actual native tile receipts")
        for member, texture in zip(members, session_owner["memberTextureIds"]):
            if not isinstance(member, dict) or integer(member.get("textureId"), 1) != texture or \
                    integer(member.get("deviceGeneration"), 1) != session_owner["deviceGeneration"]:
                raise ValueError("memory close changed a native tile owner")
    counts = trace_counts(closed.get("traceCounts"))
    if counts["closeRequested"] < 1 or any(counts[key] < len(stages) for key in ("captureStarted", "submitStarted")) or \
            any(counts.get(key, -1) < count for key, count in last_counts.items()):
        raise ValueError("memory acceptance lost lifetime trace/close evidence")
    return {"cycles": cycles, "managedBytesTrimmed": trimmed_total,
            "outcomes": [cycle["outcome"] for cycle in observations],
            "resumedPublications": [index + 1 for index in schedule],
            "physicalVramBytes": None, "consumerGpuCompletionVerified": False}
