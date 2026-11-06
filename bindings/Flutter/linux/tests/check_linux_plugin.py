#!/usr/bin/env python3
"""Bounded C++ checks; optional real SDK headers/link and GDK GL state test."""
import argparse
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--flutter-engine", type=Path)
    parser.add_argument("--dev-root", type=Path)
    parser.add_argument("--native-include", type=Path)
    parser.add_argument("--gl-state", action="store_true", help="requires a running X/Wayland display")
    parser.add_argument("--gles-state", action="store_true", help="check direct EGL GLES 3 state without a GDK context")
    args = parser.parse_args()
    source = Path(__file__).resolve().parents[1]
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    common = ["g++", "-std=c++17", "-Wall", "-Wextra", "-Werror", "-pthread", "-I" + str(source)]
    def run(command):
        subprocess.run(command, check=True, timeout=60)
    core = output / "texture-state-test"
    run(common + [str(source / "texture_state.cc"), str(source / "tests/texture_state_test.cc"), "-o", str(core)])
    run([str(core)])
    if not args.flutter_engine:
        return
    includes = ["-I" + str(args.flutter_engine.resolve())]
    if args.dev_root:
        root = args.dev_root.resolve()
        for path in ("usr/include", "usr/include/gtk-3.0", "usr/include/glib-2.0",
                     "usr/lib/x86_64-linux-gnu/glib-2.0/include", "usr/include/pango-1.0",
                     "usr/include/harfbuzz", "usr/include/cairo", "usr/include/gdk-pixbuf-2.0",
                     "usr/include/atk-1.0"):
            includes.append("-I" + str(root / path))
    else:
        includes += subprocess.check_output(["pkg-config", "--cflags", "gtk+-3.0", "epoxy"], text=True).split()
    native = args.native_include or source.parents[1] / "swift/Sources/CRhwpNative"
    includes.append("-I" + str(native.resolve()))
    libraries = ["-L" + str(args.flutter_engine.resolve()), "-Wl,-rpath," + str(args.flutter_engine.resolve()),
                 "-l:libflutter_linux_gtk.so", "-l:libgtk-3.so.0", "-l:libgdk-3.so.0",
                 "-l:libglib-2.0.so.0", "-l:libgobject-2.0.so.0", "-l:libgmodule-2.0.so.0",
                 "-l:libepoxy.so.0", "-ldl"]
    plugin = output / "librhwp_native_reference_plugin.so"
    run(common + includes + ["-shared", "-fPIC", "-Wl,--no-undefined",
        str(source / "rhwp_texture_plugin.cc"), str(source / "texture_state.cc"),
        str(source / "gl_state_guard.cc"), "-o", str(plugin)] + libraries)
    print("real Flutter public-header compile/link passed:", plugin)
    if args.gl_state or args.gles_state:
        check = output / "gl-state-test"
        run(common + includes + [str(source / "gl_state_guard.cc"), str(source / "texture_state.cc"),
            str(source / "tests/gl_state_guard_test.cc"), "-o", str(check)] + libraries)
        if args.gl_state:
            run([str(check)])
        if args.gles_state:
            run([str(check), "--gles"])


if __name__ == "__main__":
    main()
