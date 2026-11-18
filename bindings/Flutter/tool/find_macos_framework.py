#!/usr/bin/env python3
"""Locate a real pinned SDK framework slice; no fixed darwin-arm64 assumption."""
import json, pathlib, platform, subprocess, sys
sdk = pathlib.Path(sys.argv[1]).resolve(strict=True)
root = sdk / "bin/cache/artifacts/engine"
arch = platform.machine()
if sys.platform != "darwin" or arch not in ("arm64", "x86_64"):
    raise SystemExit("real Darwin arm64/x86_64 required")
choices = []
for item in root.glob("darwin*/FlutterMacOS.xcframework/macos-*/FlutterMacOS.framework"):
    item = item.resolve(strict=True)
    item.relative_to(sdk)
    binary = item / "FlutterMacOS"
    if not (item / "Headers/FlutterMacOS.h").is_file() or not binary.is_file():
        continue
    if subprocess.run(["xcrun", "lipo", str(binary), "-verify_arch", arch],
                      stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0:
        choices.append(item)
if not choices:
    raise SystemExit("no real FlutterMacOS framework with the host slice was precached")
choices.sort(key=lambda item: ("-release" in str(item) or "-profile" in str(item), str(item)))
print(choices[0])
