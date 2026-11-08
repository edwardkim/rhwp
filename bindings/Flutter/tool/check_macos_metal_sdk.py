#!/usr/bin/env python3
"""Explicit scratch build/producer checks on a genuine Apple SDK target.

The experimental Metal feature is enabled only for this isolated Cargo build.
No repository feature defaults, plugin policy or consumer frame credits change.
"""

import argparse
import json
import math
import os
from pathlib import Path
import platform
import re
import shutil
import signal
import subprocess
import sys
import time

from qualify_desktop_texture import digest, write_json


def platform_required(system):
    if system != "darwin":
        raise RuntimeError("actual macOS and Apple SDK are required")


def find_apple_cxx():
    # The clang++ symlink selects C++ driver/linker behavior. Resolving it to
    # clang compiles .mm but omits libc++ and the C++ exception runtime at link.
    return Path(subprocess.check_output(
        ["xcrun", "--sdk", "macosx", "--find", "clang++"], text=True).strip()).absolute()


def command_plan(repository, output, sdk, clang, framework, architecture, toolchain, native_override=None):
    if architecture not in ("arm64", "x86_64"):
        raise ValueError("unsupported macOS probe architecture")
    if not re.fullmatch(r"[A-Za-z0-9_.-]+", toolchain):
        raise ValueError("invalid Rust toolchain name")
    header = repository / "bindings/swift/Sources/CRhwpNative"
    rust_target = {"arm64": "aarch64-apple-darwin", "x86_64": "x86_64-apple-darwin"}[architecture]
    native = native_override or output / "native-target" / rust_target / "debug/librhwp_native_ffi.dylib"
    plugin = output / "RhwpMetalPlugin.dylib"
    bridge = output / "rhwp_metal_sdk_probe.dylib"
    cv_status = output / "rhwp_cv_closed_status_test"
    animation_policy = output / "rhwp_metal_view_animation_test"
    common = [str(clang), "-std=c++17", "-fobjc-arc", "-fblocks", "-Wall", "-Wextra", "-Werror",
              "-isysroot", str(sdk), "-arch", architecture, "-mmacosx-version-min=12.0", "-dynamiclib"]
    flutter_links = ["-F", str(framework.parent), "-Wl,-rpath," + str(framework.parent),
         "-framework", "FlutterMacOS", "-framework", "Metal", "-framework", "CoreVideo",
         "-framework", "CoreGraphics", "-framework", "IOSurface", "-framework", "Foundation",
         "-framework", "AppKit", "-framework", "QuartzCore"]
    commands = [
        ["rustup", "run", toolchain, "rustc", "--version", "--verbose"],
        [*common, *flutter_links,
         str(repository / "bindings/Flutter/macos/Classes/RhwpMetalPlugin.mm"),
         "-o", str(plugin)],
        [*common, "-I", str(header), "-framework", "Metal", "-framework", "Foundation",
         str(repository / "bindings/Flutter/macos/tests/native_metal_sdk_probe.mm"), "-o", str(bridge)],
        [*[value for value in common if value != "-dynamiclib"], *flutter_links,
         str(repository / "bindings/Flutter/macos/tests/cv_closed_status_sdk_test.mm"), "-o", str(cv_status)],
        [*[value for value in common if value != "-dynamiclib"], *flutter_links,
         str(repository / "bindings/Flutter/macos/tests/metal_view_animation_sdk_test.mm"), "-o", str(animation_policy)],
        [str(cv_status)],
        [str(animation_policy)],
        *([] if native_override else [["rustup", "run", toolchain, "cargo", "build", "--locked", "--manifest-path",
         str(repository / "bindings/Native/Cargo.toml"), "--features", "native-skia-metal",
         "--lib", "--target", rust_target, "--target-dir", str(output / "native-target")]]),
        ["xcrun", "otool", "-L", str(native)],
        ["xcrun", "otool", "-L", str(plugin)],
        [sys.executable, "-B", str(repository / "bindings/Flutter/macos/tests/native_metal_sdk_contract.py"),
         "--native", str(native), "--bridge", str(bridge), "--source-head", "SOURCE_HEAD",
         "--native-source-head", "NATIVE_SOURCE_HEAD",
         "--output", str(output / "metal-producer.json")],
    ]
    return commands, {"native": native, "plugin": plugin, "bridge": bridge,
                      "cvCloseStatus": cv_status, "metalViewAnimationPolicy": animation_policy}


def source_hashes(repository):
    """Pin every tracked byte without repeatedly reading the large fixture corpus.

    Git blob identities cover Cargo path/vendor dependencies, all workspace
    manifests, embedded data/fonts and compile-time include_str documents too.
    A whole-tree clean check is mandatory on every invocation, so these are
    identities of the actual tracked working files, not merely of an old index.
    """
    changed = subprocess.check_output(
        ["git", "-c", "gc.auto=0", "status", "--porcelain=v1", "--untracked-files=all"],
        cwd=repository, text=True)
    if changed:
        raise ValueError("commit all tracked build inputs and remove untracked source before verification")
    entries = subprocess.check_output(
        ["git", "-c", "gc.auto=0", "ls-files", "--stage", "--full-name", "-z"], cwd=repository).split(b"\0")
    result = {}
    for entry in entries:
        if not entry:
            continue
        metadata, path = entry.split(b"\t", 1)
        mode, blob, stage = metadata.split()
        if stage != b"0":
            raise ValueError("unmerged source index cannot qualify an SDK build")
        result[os.fsdecode(path)] = mode.decode("ascii") + ":" + blob.decode("ascii")
    return result


def stop_process_tree(process, grace=5):
    """Only for children created with start_new_session=True in execute_step."""
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        process.wait(timeout=grace)
    except subprocess.TimeoutExpired:
        pass
    finally:
        # A parent can exit on TERM while a compiler/device helper ignores it.
        # Its descendants still belong to this owned group and must not survive.
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait()


def execute_step(command, *, repository, env, log_path, step, record, timeout, stop_grace=5):
    if not math.isfinite(timeout) or timeout <= 0 or timeout > 7200:
        raise ValueError("step timeout must be finite and within two hours")
    step["timeoutSeconds"] = timeout
    with Path(log_path).open("wb") as log:
        Path(log_path).chmod(0o600)
        process = subprocess.Popen(command, cwd=repository, env=env, stdin=subprocess.DEVNULL,
                                   stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        step["pid"] = process.pid
        try:
            record()
            return process.wait(timeout=timeout)
        except subprocess.TimeoutExpired as error:
            step["timedOut"] = True
            stop_process_tree(process, grace=stop_grace)
            raise TimeoutError("SDK step exceeded its deadline: " + str(log_path)) from error
        except BaseException:
            step["interrupted"] = True
            stop_process_tree(process, grace=stop_grace)
            raise
        finally:
            step.update(exitCode=process.returncode, finished=time.time())
            record()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--flutter-sdk", type=Path, required=True)
    parser.add_argument("--flutter-framework", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--native-artifact-root", type=Path)
    parser.add_argument("--native-source-head")
    parser.add_argument("--toolchain", default="1.93.1")
    parser.add_argument("--enable-metal-experiment", action="store_true")
    parser.add_argument("--step-timeout", type=float, default=3600,
                        help="maximum seconds per compile/link step (30..7200)")
    parser.add_argument("--producer-timeout", type=float, default=120,
                        help="maximum seconds for the disposable Metal producer (1..300)")
    args = parser.parse_args()
    if bool(args.native_artifact_root) != bool(args.native_source_head):
        parser.error("Native artifact reuse requires both its directory and exact source head")
    if not math.isfinite(args.step_timeout) or not 30 <= args.step_timeout <= 7200:
        parser.error("step timeout must be finite and between 30 and 7200 seconds")
    if not math.isfinite(args.producer_timeout) or not 1 <= args.producer_timeout <= 300:
        parser.error("producer timeout must be finite and between 1 and 300 seconds")
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
    clang = find_apple_cxx()
    compiler = subprocess.check_output([str(clang), "--version"], text=True)
    if "Apple clang" not in compiler:
        raise ValueError("an Apple SDK compiler is required, not a portable stub")
    before = source_hashes(repository)
    source_head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repository, text=True).strip()
    output = args.output.resolve()
    reused = None
    if args.native_artifact_root:
        from native_artifact_provenance import load_native_artifact
        reused = load_native_artifact(args.native_artifact_root, repository, "macos", args.native_source_head)
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    (output / "tmp").mkdir(mode=0o700)
    native_override = None
    if reused:
        directory = output / "native-artifact"
        directory.mkdir(mode=0o700)
        native_override = directory / "librhwp_native_ffi.dylib"
        shutil.copyfile(reused["native"], native_override)
        if digest(native_override) != digest(reused["native"]):
            raise ValueError("Native artifact changed while copying")
    commands, artifacts = command_plan(repository, output, sdk, clang, framework, platform.machine(), args.toolchain, native_override)
    native_source_head = args.native_source_head or source_head
    state = {"platform": "macos", "source_head": source_head, "sourceFiles": before,
             "sourceHashKind": "git-index-mode-and-blob-oid; whole tracked tree verified clean each step",
             "nativeSourceHead": native_source_head, "nativeRebuilt": reused is None,
             **({"nativeReuse": reused["proof"]} if reused else {}),
             "architecture": platform.machine(),
             "rustTarget": {"arm64": "aarch64-apple-darwin", "x86_64": "x86_64-apple-darwin"}[platform.machine()],
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
            command = [source_head if value == "SOURCE_HEAD" else native_source_head if value == "NATIVE_SOURCE_HEAD" else value for value in command]
            index = len(state["steps"])
            step = {"command": command, "started": time.time(), "exitCode": None,
                    "log": str(output / f"step-{index}.log")}
            state["steps"].append(step)
            write_json(receipt, state)
            if source_hashes(repository) != before:
                raise ValueError("source changed before target SDK verification step")
            timeout = args.producer_timeout if index == len(commands) - 1 else args.step_timeout
            code = execute_step(command, repository=repository, env=env, log_path=step["log"],
                                step=step, record=lambda: write_json(receipt, state), timeout=timeout)
            if code:
                raise RuntimeError("SDK step failed; inspect the completed log: " + step["log"])
            if source_hashes(repository) != before:
                raise ValueError("source changed during target SDK verification")
        proof = json.loads((output / "metal-producer.json").read_text())
        if proof.get("status") != "passed" or proof.get("source_head") != source_head:
            raise ValueError("actual Metal producer did not produce a matching successful receipt")
        state.update(build="passed", targetSdkCompileLinkVerified=True, metalProducerReadbackVerified=True,
                     cvTerminalStatusVerified=True, metalViewAnimationPolicyVerified=True,
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
