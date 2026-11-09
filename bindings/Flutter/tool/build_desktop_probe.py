#!/usr/bin/env python3
"""Build a disposable actual Windows runner from a clean, pinned source copy.

Native rhwp is supplied with an existing build receipt; this helper neither
downloads a native DLL nor treats portable checks as Windows execution.
"""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

from qualify_desktop_texture import (digest, read_json, write_json, require_actual_windows,
                                     runtime_bundle_manifest, MAX_BUILD_RECEIPT_BYTES)


def checked_sha(value, name):
    if not re.fullmatch(r"[0-9a-f]{40}", str(value)):
        raise ValueError("expected pinned 40-character " + name)
    return value


def sources(repository):
    paths = subprocess.check_output(
        ["git", "ls-files", "-z", "--", "bindings/Flutter", "bindings/swift/Sources/CRhwpNative"],
        cwd=repository).split(b"\0")
    return {os.fsdecode(item): digest(repository / os.fsdecode(item)) for item in paths if item}


def artifact(path):
    return {"path": str(path), "sha256": digest(path), "bytes": path.stat().st_size}


def verify_copied_sources(copied, expected):
    """Check every copied tracked input, including native plugin and headers.

    Generated runner files are separate and are not passed as package inputs.
    """
    actual = {relative: digest(copied / relative) for relative in expected}
    if actual != expected:
        changed = [relative for relative, value in actual.items() if value != expected[relative]]
        raise ValueError("build changed copied tracked input: " + ", ".join(changed))
    return actual


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--flutter-sdk", type=Path, required=True)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--native-receipt", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        require_actual_windows()
    except RuntimeError as error:
        parser.error(str(error))
    repository = Path(__file__).resolve().parents[3]
    status = subprocess.check_output(
        ["git", "status", "--porcelain", "--", "bindings/Flutter", "bindings/swift/Sources/CRhwpNative"],
        cwd=repository, text=True)
    if status:
        raise ValueError("commit the qualification source before building its receipt")
    source_head = checked_sha(subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=repository, text=True).strip(), "source head")
    native = args.native.resolve(strict=True)
    native_receipt = read_json(args.native_receipt)
    native_head = checked_sha(native_receipt.get("source_head"), "native source head")
    native_hash = native_receipt.get("artifacts", {}).get("rhwp_native_ffi.dll")
    if isinstance(native_hash, dict):
        native_hash = native_hash.get("sha256")
    if native_receipt.get("build") != "passed" or digest(native) != native_hash:
        raise ValueError("native DLL must match a successful actual Windows build receipt")
    sdk = args.flutter_sdk.resolve(strict=True)
    dart = sdk / "bin/cache/dart-sdk/bin/dart.exe"
    snapshot = sdk / "bin/cache/flutter_tools.snapshot"
    if not dart.is_file() or not snapshot.is_file():
        raise ValueError("prepare the Windows Flutter SDK/tool snapshot before qualification")
    # Calling the tool snapshot avoids implicit cmd.exe parsing of flutter.bat
    # paths. Every child command is an argument vector, never a shell string.
    flutter = [str(dart), "--disable-dart-dev", str(snapshot)]
    output = args.output.resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    original_sources = sources(repository)
    copied = output / "source"
    for relative in original_sources:
        destination = copied / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(repository / relative, destination)
    env = dict(os.environ, CI="true", FLUTTER_SUPPRESS_ANALYTICS="true",
               DART_SUPPRESS_ANALYTICS="true", FLUTTER_ROOT=str(sdk))
    receipt = {"platform": "windows", "sourceHead": source_head, "nativeSourceHead": native_head,
               "sourceFiles": original_sources, "steps": [], "buildExitCode": None,
               "started": time.time(), "pid": os.getpid()}
    receipt_path = output / "build-receipt.json"

    def run(command, cwd):
        index = len(receipt["steps"])
        step = {"command": command, "cwd": str(cwd), "started": time.time(), "exitCode": None,
                "log": str(output / f"build-{index}.log")}
        receipt["steps"].append(step)
        write_json(receipt_path, receipt)
        with Path(step["log"]).open("wb") as log:
            process = subprocess.Popen(command, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                       stdout=log, stderr=subprocess.STDOUT)
            step["pid"] = process.pid
            write_json(receipt_path, receipt)
            code = process.wait()
        step.update(exitCode=code, finished=time.time())
        write_json(receipt_path, receipt)
        if code:
            raise RuntimeError("build step failed; inspect completed log: " + step["log"])
        return Path(step["log"]).read_text(encoding="utf-8", errors="replace")

    try:
        version_text = run([*flutter, "--version", "--machine"], output)
        # --machine emits one JSON object; unsolicited output is an error, not
        # evidence that the desired compiler/engine was used.
        version = json.loads(version_text)
        receipt["flutterRevision"] = checked_sha(version.get("frameworkRevision"), "Flutter revision")
        receipt["engineRevision"] = checked_sha(version.get("engineRevision"), "engine revision")
        receipt["flutterVersion"] = version.get("frameworkVersion")
        run([*flutter, "create", "--platforms=windows", "--project-name=rhwp_texture_qualification", "runner"], output)
        runner = output / "runner"
        (runner / "pubspec.yaml").write_text(
            "name: rhwp_texture_qualification\npublish_to: none\n"
            "environment:\n  sdk: '>=3.13.0 <4.0.0'\n"
            "dependencies:\n  flutter:\n    sdk: flutter\n"
            "  rhwp_native_reference:\n    path: ../source/bindings/Flutter\n",
            encoding="utf-8")
        shutil.copyfile(copied / "bindings/Flutter/test/support/desktop_consumer_probe.dart", runner / "lib/main.dart")
        run([*flutter, "pub", "get"], runner)
        run([*flutter, "build", "windows", "--debug"], runner)
        executable = runner / "build/windows/x64/runner/Debug/rhwp_texture_qualification.exe"
        plugin = executable.parent / "rhwp_native_reference_plugin.dll"
        engine_runtime = executable.parent / "flutter_windows.dll"
        dispatcher = output / "rhwp_flutter_owner.dll"
        run([sys.executable, str(copied / "bindings/Flutter/tool/build_dispatcher.py"),
             "--output", str(dispatcher)], output)
        fixture = copied / "bindings/Flutter/test/fixtures/desktop-consumer.hwpx"
        artifacts = {"runner": executable, "plugin": plugin, "engineRuntime": engine_runtime,
                     "dispatcher": dispatcher, "native": native, "fixture": fixture}
        receipt["artifacts"] = {key: artifact(value) for key, value in artifacts.items()}
        receipt["runtimeBundle"] = runtime_bundle_manifest(executable.parent)
        if sources(repository) != original_sources:
            raise ValueError("source checkout changed while building")
        receipt["copiedRuntimeSources"] = verify_copied_sources(copied, original_sources)
        if len((json.dumps(receipt, indent=2) + "\n").encode("utf-8")) > MAX_BUILD_RECEIPT_BYTES - 1024:
            raise ValueError("build receipt exceeds its bounded runtime manifest size")
        receipt["buildExitCode"] = 0
    except BaseException as error:
        receipt.update(buildExitCode=1, error=f"{type(error).__name__}: {error}")
        raise
    finally:
        receipt["finished"] = time.time()
        write_json(receipt_path, receipt)
    print(receipt_path)


if __name__ == "__main__":
    main()
