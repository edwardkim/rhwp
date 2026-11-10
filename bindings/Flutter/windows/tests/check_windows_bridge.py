#!/usr/bin/env python3
"""Portable checks only; this does not compile/execute a Windows D3D backend."""
import argparse
from pathlib import Path
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--flutter-public", type=Path, help="actual pinned common/public Flutter headers")
    parser.add_argument("--sanitize", action="store_true")
    args = parser.parse_args()
    source = Path(__file__).resolve().parents[1]
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    command = ["g++", "-std=c++17", "-pthread", "-Wall", "-Wextra", "-Werror", "-I" + str(source)]
    if args.sanitize:
        command += ["-fsanitize=address,undefined", "-fno-omit-frame-pointer", "-g"]
    for name in ["frame_lifetime_test", "import_lifetime_test", "close_registry_test"]:
        executable = output / name
        subprocess.run(command + [str(source / "tests" / (name + ".cpp")), "-o", str(executable)], check=True, timeout=60)
        subprocess.run([str(executable)], check=True, timeout=30)
    tiles = output / "bounded-tile-group-test"
    subprocess.run(command + [str(source.parent / "common/bounded_tile_group_test.cc"), "-o", str(tiles)], check=True, timeout=60)
    subprocess.run([str(tiles)], check=True, timeout=30)
    subprocess.run([sys.executable, "-B", str(source / "tests/native_d3d_disabled.py"), "--output", str(output / "disabled")], check=True, timeout=60)
    subprocess.run([sys.executable, "-B", str(source / "tests/group_error_limits_contract.py"), "--output", str(output / "group-error-limits")], check=True, timeout=90)
    if args.flutter_public:
        header_test = output / "public-texture-contract"
        native = source.parents[1] / "swift/Sources/CRhwpNative"
        subprocess.run(command + ["-I" + str(source / "include"), "-I" + str(native), "-I" + str(args.flutter_public.resolve()),
            str(source / "tests/public_texture_contract.cpp"), "-o", str(header_test)], check=True, timeout=60)
        subprocess.run([str(header_test)], check=True, timeout=10)
        print("actual Flutter common-header descriptor + typed native C header contract passed")
    print("Windows SDK/backend compile and real OS GPU execution were not performed")


if __name__ == "__main__":
    main()
