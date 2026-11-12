"""Checks for actual Metal cache-on/off readbacks; portable tests check rejection only."""

import base64


CACHE_CONFIG = {"tileSize": 128, "maxCachedTiles": 2, "maxTileBytes": 128 * 128 * 4 * 2,
                "maxManagedTargets": 4, "maxManagedTargetBytes": 2 * 1024 * 1024}
CONFIGS = {"cached": CACHE_CONFIG,
           "disabled": dict(CACHE_CONFIG, maxCachedTiles=0),
           "targetLimited": dict(CACHE_CONFIG, maxManagedTargets=1)}


def requests(stamp, snapshot):
    # The authoritative blank page gives a colored rectangle with fractional
    # edges. Cache-off strict replay is the pixel oracle for each placement.
    placements = [(0, 0, .25, 0xe0d0c0), (0, 0, .25, 0xe0d0c0),
                  (-16, -8, .25, 0xe0d0c0), (.25, .5, .25, 0xe0d0c0),
                  (.25, .5, .25, 0xe0d0c0), (0, 0, .5, 0xe0d0c0),
                  (0, 0, .5, 0x306080), (0, 0, .25, 0xe0d0c0)]
    return [{"stamp": dict(stamp),
             "target": {"deviceGeneration": stamp["deviceGeneration"], "width": 32, "height": 24},
             "placements": [{"snapshot": snapshot, "scale": scale, "originX": x, "originY": y,
                             "backgroundColor": color}], "clearColor": 0xff102030}
            for x, y, scale, color in placements]


def integer(value):
    if type(value) is not int or value < 0:
        raise ValueError("cache observation must be a nonnegative integer")
    return value


def validate_batch(value, stamp, config):
    if (value.get("ok") is not True or value.get("ownerReleased") is not True
            or value.get("consumerGpuCompletionVerified") is not False
            or value.get("afterRelease", {}).get("ok") is not False
            or value.get("afterRelease", {}).get("code") != "invalidArgument"):
        raise ValueError("cache probe did not release its producer owner")
    frames = value.get("results", [])
    if len(frames) != 8:
        raise ValueError("cache probe requires eight actual bounded readbacks")
    pixels = []
    for index, item in enumerate(frames):
        native = item.get("native", {})
        frame = native.get("frame", {})
        if (item.get("ok") is not True or native.get("ok") is not True
                or item.get("consumerGpuCompletionVerified") is not False
                or item.get("hostVerificationReadbacks") != 1
                or item.get("width") != 32 or item.get("height") != 24
                or frame.get("backend") != "metalGanesh" or frame.get("stamp") != stamp
                or frame.get("target") != {"deviceGeneration": stamp["deviceGeneration"], "width": 32, "height": 24}
                or frame.get("producerGpuComplete") is not True
                or frame.get("consumerGpuComplete") is not False
                or frame.get("wholeFrameReadbacks") != 0 or frame.get("submission") != index + 1):
            raise ValueError("cache frame lacks actual current producer completion")
        data = base64.b64decode(item.get("bgraBase64", ""), validate=True)
        if len(data) != 32 * 24 * 4:
            raise ValueError("cache frame readback dimensions differ")
        pixels.append(data)
        info = item.get("info", {})
        if (info.get("state") != "active" or info.get("scratchTargets") != 1
                or any(info.get(key) != 0 for key in ("workingTargets", "frameTilePins", "quarantinedSnapshots"))):
            raise ValueError("completed frame still retains working or quarantined inputs")
        targets = info.get("managedTargets", {})
        tiles = info.get("tiles", {})
        for key, maximum in [("targets", config["maxManagedTargets"]),
                             ("peakTargets", config["maxManagedTargets"]),
                             ("estimatedBytes", config["maxManagedTargetBytes"]),
                             ("peakEstimatedBytes", config["maxManagedTargetBytes"])]:
            if integer(targets.get(key)) > maximum:
                raise ValueError("managed target admission exceeded its configured bound")
        for key, maximum in [("cachedTiles", config["maxCachedTiles"]),
                             ("cachedBytes", config["maxTileBytes"])]:
            if integer(tiles.get(key)) > maximum:
                raise ValueError("retained tile cache exceeded its configured bound")
    return pixels


def validate_responses(values, stamp):
    pixels = {name: validate_batch(values[name], stamp, config) for name, config in CONFIGS.items()}
    maximum = max(abs(a - b) for name in ("cached", "targetLimited")
                  for result, reference in zip(pixels[name], pixels["disabled"])
                  for a, b in zip(result, reference))
    if maximum > 1:
        raise ValueError("cached/fallback Metal pixels differ from cache-disabled strict replay")
    observations = [frame["info"]["tiles"] for frame in values["cached"]["results"]]
    if [integer(item.get("canonicalReplays")) for item in observations] != [1, 1, 1, 2, 2, 3, 4, 5]:
        raise ValueError("integer scroll was not reused or phase/scale/background did not invalidate")
    if integer(observations[-1].get("hits")) < 3 or integer(observations[-1].get("evictions")) < 3:
        raise ValueError("cache hits and bounded LRU eviction were not exercised")
    for name in ("disabled", "targetLimited"):
        last = values[name]["results"][-1]["info"]["tiles"]
        if (integer(last.get("canonicalReplays")) != 0 or integer(last.get("cachedTiles")) != 0
                or integer(last.get("viewportFallbacks")) != 8):
            raise ValueError("disabled/exhausted cache did not preserve strict viewport fallback")
    if integer(values["targetLimited"]["results"][-1]["info"]["tiles"].get("targetAdmissionFallbacks")) != 8:
        raise ValueError("managed target admission limit was not exercised")
    return {"framesPerOwner": 8, "ownersReleased": 3, "maxChannelDifference": maximum,
            "integerScrollReuse": True, "phaseScaleBackgroundInvalidation": True,
            "boundedLruAndTargetFallback": True, "consumerGpuCompletionVerified": False}
