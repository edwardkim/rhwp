#!/usr/bin/env python3
"""Check disabled exports in a full Native SO or dependency-free rustc shim."""
import argparse
import ctypes
import json
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--library", type=Path, help="fresh full Native library with disabled D3D exports")
    mode.add_argument("--output", type=Path, help="build and check the dependency-free rustc shim here")
    args = parser.parse_args()
    if args.library:
        library = args.library
        free_name = "rhwp_string_free"
    else:
        source = Path(__file__).resolve().parents[3] / "Native/src/gpu_d3d.rs"
        if not source.exists():
            raise RuntimeError("production gpu_d3d.rs was not found")
        args.output.mkdir(parents=True, exist_ok=True)
        harness = args.output / "disabled.rs"
        harness.write_text('#[path = ' + json.dumps(str(source)) + ']\nmod gpu_d3d;\n'
            '#[no_mangle]\npub unsafe extern "C" fn rhwp_test_free(value:*mut std::ffi::c_char) {'
            'if !value.is_null() { drop(unsafe { std::ffi::CString::from_raw(value) }); }}\n')
        library = args.output / "libd3d_disabled.so"
        subprocess.run(["rustc", "--edition=2021", "--crate-type", "cdylib", str(harness), "-o", str(library)], check=True, timeout=60)
        free_name = "rhwp_test_free"
    api = ctypes.CDLL(str(library.resolve()))
    free_string = getattr(api, free_name)
    free_string.argtypes = [ctypes.c_void_p]
    free_string.restype = None
    def call(name, types, *values):
        function = getattr(api, name)
        function.argtypes = types
        function.restype = ctypes.c_void_p
        pointer = function(*values)
        assert pointer
        try:
            return json.loads(ctypes.string_at(pointer))
        finally:
            free_string(pointer)
    caps = call("rhwp_d3d_capabilities", [])
    assert caps["ok"] and caps["abiVersion"] == 1 and caps["available"] is False
    assert caps["consumerGpuCompletion"] is False and caps["wholeFrameReadbacks"] == 0
    pointer, number, length = ctypes.c_void_p, ctypes.c_uint64, ctypes.c_size_t
    calls = [
        ("rhwp_d3d_presenter_create", [pointer, pointer, pointer, number, pointer, length], [1, 2, 3, 1, 1, 2**63]),
        ("rhwp_d3d_presenter_render", [number, pointer, pointer, length], [2**63, 1, 1, 2**63]),
        ("rhwp_d3d_presenter_request", [number, pointer, length], [2**63, 1, 2**63]),
        ("rhwp_d3d_frame_check", [pointer, length], [1, 2**63]),
    ]
    for name, types, values in calls:
        result = call(name, types, *values)
        assert result["ok"] is False and result["code"] == "featureDisabled", (name, result)
    print("production disabled D3D ABI: capability + 4 ignored-invalid-span calls passed")


if __name__ == "__main__":
    main()
