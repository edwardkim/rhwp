#!/usr/bin/env python3
"""Explicit scratch build/producer checks on a genuine Apple SDK target.

The experimental Metal feature is enabled only for this isolated Cargo build.
No repository feature defaults, plugin policy or consumer frame credits change.
"""

import argparse
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
import time

from qualify_desktop_texture import digest, write_json


def platform_required(system):
    if system != "darwin":
        raise RuntimeError("actual macOS and Apple SDK are required")


def command_plan(repository, output, sdk, clang, framework, architecture, toolchain):
    if architecture not in ("arm64", "x86_64"):
        raise ValueError("unsupported macOS probe architecture")
    if not re.fullmatch(r"[A-Za-z0-9_.-]+", toolchain):
        raise ValueError("invalid Rust toolchain name")
    header = repository / "bindings/swift/Sources/CRhwpNative"
    native = output / "native-target/debug/librhwp_native_ffi.dylib"
    plugin = output / "RhwpMetalPlugin.dylib"
    bridge = output / "rhwp_metal_sdk_probe.dylib"
    common = [str(clang), "-std=c++17", "-fobjc-arc", "-fblocks", "-Wall", "-Wextra", "-Werror",
              "-isysroot", str(sdk), "-arch", architecture, "-mmacosx-version-min=12.0", "-dynamiclib"]
    commands = [
        ["rustup", "run", toolchain, "rustc", "--version", "--verbose"],
        ["rustup", "run", toolchain, "cargo", "build", "--locked", "--manifest-path",
         str(repository / "bindings/Native/Cargo.toml"), "--features", "native-skia-metal",
         "--lib", "--target-dir", str(output / "native-target")],
        [*common, "-F", str(framework.parent), "-Wl,-rpath," + str(framework.parent),
         "-framework", "FlutterMacOS", "-framework", "Metal", "-framework", "CoreVideo",
         "-framework", "CoreGraphics", "-framework", "IOSurface", "-framework", "Foundation",
         "-framework", "AppKit", str(repository / "bindings/Flutter/macos/Classes/RhwpMetalPlugin.mm"),
         "-o", str(plugin)],
        [*common, "-I", str(header), "-framework", "Metal", "-framework", "Foundation",
         str(repository / "bindings/Flutter/macos/tests/native_metal_sdk_probe.mm"), "-o", str(bridge)],
        ["xcrun", "otool", "-L", str(native)],
        ["xcrun", "otool", "-L", str(plugin)],
        [sys.executable, "-B", str(repository / "bindings/Flutter/macos/tests/native_metal_sdk_contract.py"),
         "--native", str(native), "--bridge", str(bridge), "--source-head", "SOURCE_HEAD",
         "--output", str(output / "metal-producer.json")],
    ]
    return commands, {"native": native, "plugin": plugin, "bridge": bridge}


def source_hashes(repository):
    paths = subprocess.check_output(["git", "ls-files", "-z", "--", "Cargo.toml", "Cargo.lock", "build.rs",
                                    "src", "crates", "bindings/Native", "bindings/Flutter/macos",
                                    "bindings/Flutter/tool/check_macos_metal_sdk.py",
                                    "bindings/Flutter/tool/qualify_desktop_texture.py",
                                    "bindings/swift/Sources/CRhwpNative"], cwd=repository).split(b"\0")
    return {os.fsdecode(value): digest(repository / os.fsdecode(value)) for value in paths if value}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--flutter-sdk", type=Path, required=True)
    parser.add_argument("--flutter-framework", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--toolchain", default="1.93.1")
    parser.add_argument("--enable-metal-experiment", action="store_true")
    args = parser.parse_args()
    try:
        platform_required(sys.platform)
    except RuntimeError as error:
        parser.error(str(error))
    if not args.enable_metal_experiment:
        parser.error("explicit --enable-metal-experiment is required; production defaults stay disabled")
    repository = Path(__file__).resolve().parents[3]
    sdk_root = args.flutter_sdk.resolve(strict=True)
    framework = args.flutter_framework.resolve(strict=True)
    if framework.name != "FlutterMacOS.framework" or not (framework / "Headers/FlutterMacOS.h").is_file():
        raise ValueError("the actual FlutterMacOS framework and headers are required")
    framework.relative_to(sdk_root)
    framework_binary = (framework / "FlutterMacOS").resolve(strict=True)
    flutter_head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=sdk_root, text=True).strip()
    engine_head = (sdk_root / "bin/internal/engine.version").read_text().strip()
    if any(not re.fullmatch(r"[0-9a-f]{40}", value) for value in [flutter_head, engine_head]):
        raise ValueError("Flutter SDK/framework must have pinned framework and engine revisions")
    sdk = Path(subprocess.check_output(["xcrun", "--sdk", "macosx", "--show-sdk-path"], text=True).strip()).resolve(strict=True)
    clang = Path(subprocess.check_output(["xcrun", "--sdk", "macosx", "--find", "clang++"], text=True).strip()).resolve(strict=True)
    compiler = subprocess.check_output([str(clang), "--version"], text=True)
    if "Apple clang" not in compiler:
        raise ValueError("an Apple SDK compiler is required, not a portable stub")
    changed = subprocess.check_output(["git", "status", "--porcelain", "--", "Cargo.toml", "Cargo.lock",
                                      "build.rs", "src", "crates", "bindings/Native", "bindings/Flutter/macos",
                                      "bindings/Flutter/tool", "bindings/swift/Sources/CRhwpNative"],
                                     cwd=repository, text=True)
    if changed:
        raise ValueError("commit the source before starting the stable SDK verification")
    source_head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repository, text=True).strip()
    output = args.output.resolve()
    commands, artifacts = command_plan(repository, output, sdk, clang, framework, platform.machine(), args.toolchain)
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    (output / "tmp").mkdir(mode=0o700)
    before = source_hashes(repository)
    state = {"platform": "macos", "source_head": source_head, "sourceFiles": before,
             "flutterRevision": flutter_head, "engineRevision": engine_head,
             "frameworkSha256": digest(framework_binary), "sdk": str(sdk), "compiler": compiler,
             "experimentalFeatureExplicitlyEnabled": True, "productionDefaultsChanged": False,
             "consumerGpuCompletionVerified": False, "continuousPresentationSupported": False,
             "flutterConsumerSamplingVerified": False, "steps": [], "started": time.time(),
             "build": "pending", "pid": os.getpid()}
    receipt = output / "macos-sdk-receipt.json"
    env = dict(os.environ, CARGO_TARGET_DIR=str(output / "native-target"),
               CARGO_BUILD_BUILD_DIR=str(output / "native-build"), TMPDIR=str(output / "tmp"),
               CARGO_INCREMENTAL="0", CARGO_BUILD_JOBS="2")
    try:
        for command in commands:
            command = [source_head if value == "SOURCE_HEAD" else value for value in command]
            index = len(state["steps"])
            step = {"command": command, "started": time.time(), "exitCode": None,
                    "log": str(output / f"step-{index}.log")}
            state["steps"].append(step)
            write_json(receipt, state)
            with Path(step["log"]).open("wb") as log:
                process = subprocess.Popen(command, cwd=repository, env=env, stdin=subprocess.DEVNULL,
                                           stdout=log, stderr=subprocess.STDOUT)
                step["pid"] = process.pid
                write_json(receipt, state)
                code = process.wait()
            step.update(exitCode=code, finished=time.time())
            write_json(receipt, state)
            if code:
                raise RuntimeError("SDK step failed; inspect the completed log: " + step["log"])
            if source_hashes(repository) != before:
                raise ValueError("source changed during target SDK verification")
        proof = json.loads((output / "metal-producer.json").read_text())
        if proof.get("status") != "passed" or proof.get("source_head") != source_head:
            raise ValueError("actual Metal producer did not produce a matching successful receipt")
        state.update(build="passed", targetSdkCompileLinkVerified=True, metalProducerReadbackVerified=True,
                     artifacts={key: {"path": str(path), "sha256": digest(path), "bytes": path.stat().st_size}
                                for key, path in artifacts.items()})
    except BaseException as error:
        state.update(build="failed", error=f"{type(error).__name__}: {error}")
        raise
    finally:
        state["finished"] = time.time()
        write_json(receipt, state)
    print(receipt)


if __name__ == "__main__":
    main()
