#!/usr/bin/env python3
"""Build a disposable genuine GTK/Flutter runner from pinned reference sources."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time

from build_desktop_probe import artifact, checked_sha, sources, verify_copied_sources
from check_macos_metal_sdk import execute_step
from qualify_desktop_texture import MAX_BUILD_RECEIPT_BYTES, digest, runtime_bundle_manifest, write_json


def require_linux():
    if sys.platform != "linux":
        raise RuntimeError("actual Linux GTK/GL execution is required")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--flutter-sdk", type=Path, required=True)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--native-source-head", required=True,
                        help="declared source SHA of the supplied library; this does not rebuild Native")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--step-timeout", type=float, default=600)
    args = parser.parse_args()
    require_linux()
    if not 30 <= args.step_timeout <= 1800:
        parser.error("step timeout must be between 30 and 1800 seconds")
    repository = Path(__file__).resolve().parents[3]
    changed = subprocess.check_output(
        ["git", "-c", "gc.auto=0", "status", "--porcelain", "--", "bindings/Flutter",
         "bindings/swift/Sources/CRhwpNative"], cwd=repository, text=True)
    if changed:
        raise ValueError("commit all qualification inputs before building")
    source_head = checked_sha(subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=repository, text=True).strip(), "source head")
    native_head = checked_sha(args.native_source_head, "declared Native source head")
    native = args.native.resolve(strict=True)
    sdk = args.flutter_sdk.resolve(strict=True)
    flutter = [str(sdk / "bin/cache/dart-sdk/bin/dart"), "--disable-dart-dev",
               str(sdk / "bin/cache/flutter_tools.snapshot")]
    output = args.output.resolve()
    try:
        output.relative_to(repository)
    except ValueError:
        pass
    else:
        raise ValueError("build output must be outside the source checkout")
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    copied = output / "source"
    original = sources(repository)
    native_hash = digest(native)
    for relative in original:
        destination = copied / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(repository / relative, destination)
    env = dict(os.environ, CI="true", FLUTTER_SUPPRESS_ANALYTICS="true",
               DART_SUPPRESS_ANALYTICS="true", FLUTTER_ROOT=str(sdk))
    receipt = {"platform": "linux", "sourceHead": source_head,
               "nativeSourceHead": native_head, "nativeBuildVerified": False,
               "nativeProvenance": "supplied-library hash and caller-declared source; no Native rebuild",
               "sourceFiles": original, "steps": [], "buildExitCode": None,
               "started": time.time(), "pid": os.getpid()}
    receipt_path = output / "build-receipt.json"

    def run(command, cwd):
        index = len(receipt["steps"])
        step = {"command": command, "cwd": str(cwd), "started": time.time(),
                "exitCode": None, "log": str(output / f"build-{index}.log")}
        receipt["steps"].append(step)
        write_json(receipt_path, receipt)
        code = execute_step(command, repository=cwd, env=env, log_path=step["log"],
                            step=step, record=lambda: write_json(receipt_path, receipt),
                            timeout=args.step_timeout)
        if code:
            raise RuntimeError("build failed; inspect completed log: " + step["log"])
        return Path(step["log"]).read_text(encoding="utf-8", errors="replace")

    try:
        version = json.loads(run([*flutter, "--version", "--machine"], output))
        receipt["flutterRevision"] = checked_sha(version.get("frameworkRevision"), "Flutter revision")
        receipt["engineRevision"] = checked_sha(version.get("engineRevision"), "engine revision")
        receipt["flutterVersion"] = version.get("frameworkVersion")
        headers = sdk / "bin/cache/artifacts/engine/linux-x64/flutter_linux"
        receipt["flutterEngineHeaders"] = {p.name: digest(p) for p in sorted(headers.glob("*.h"))}
        if not receipt["flutterEngineHeaders"]:
            raise ValueError("prepared real Linux Flutter public headers are required")
        run([*flutter, "create", "--platforms=linux", "--project-name=rhwp_texture_qualification",
             "runner"], output)
        runner = output / "runner"
        (runner / "pubspec.yaml").write_text(
            "name: rhwp_texture_qualification\npublish_to: none\n"
            "environment:\n  sdk: '>=3.13.0 <4.0.0'\n"
            "dependencies:\n  flutter:\n    sdk: flutter\n"
            "  rhwp_native_reference:\n    path: ../source/bindings/Flutter\n", encoding="utf-8")
        shutil.copyfile(copied / "bindings/Flutter/test/support/desktop_consumer_probe.dart",
                        runner / "lib/main.dart")
        # A disposable undecorated GTK client gives the external X11 collector
        # the same physical origin as Flutter's view. Product example is intact.
        application = runner / "linux/runner/my_application.cc"
        source = application.read_text()
        source, count = re.subn(r"gtk_window_set_default_size\(window, [0-9]+, [0-9]+\);",
                               "gtk_window_set_titlebar(window, nullptr);\n"
                               "  gtk_window_set_decorated(window, FALSE);\n"
                               "  gtk_window_set_default_size(window, 640, 480);", source)
        if count != 1:
            raise ValueError("unexpected pinned GTK runner template")
        application.write_text(source)
        receipt["generatedRunner"] = {"linux/runner/my_application.cc": digest(application)}
        run([*flutter, "pub", "get", "--offline"], runner)
        run([*flutter, "build", "linux", "--debug"], runner)
        executable = runner / "build/linux/x64/debug/bundle/rhwp_texture_qualification"
        dispatcher = output / "librhwp_flutter_owner.so"
        run([sys.executable, str(copied / "bindings/Flutter/tool/build_dispatcher.py"),
             "--output", str(dispatcher)], output)
        artifacts = {"runner": executable,
                     "plugin": executable.parent / "lib/librhwp_native_reference_plugin.so",
                     "engineRuntime": executable.parent / "lib/libflutter_linux_gtk.so",
                     "dispatcher": dispatcher, "native": native,
                     "fixture": copied / "bindings/Flutter/test/fixtures/desktop-consumer.hwpx"}
        receipt["artifacts"] = {key: artifact(path) for key, path in artifacts.items()}
        receipt["runtimeBundle"] = runtime_bundle_manifest(executable.parent)
        if sources(repository) != original or digest(native) != native_hash:
            raise ValueError("source or supplied Native library changed during build")
        receipt["copiedRuntimeSources"] = verify_copied_sources(copied, original)
        if len((json.dumps(receipt, indent=2) + "\n").encode()) > MAX_BUILD_RECEIPT_BYTES - 1024:
            raise ValueError("build receipt exceeds its runtime manifest bound")
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
