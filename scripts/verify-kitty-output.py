#!/usr/bin/env python3
"""Kitty APC 독립 복원과 전쪽 PNG/Sixel 무회귀 검사. 한컴 Visual Sweep과 구분한다."""

import argparse
import base64
import hashlib
import io
import json
import math
import re
import subprocess
import xml.etree.ElementTree as ET
from pathlib import Path

from PIL import Image


def decode_stream(stream):
    """제품 writer를 호출하지 않고 APC 문법과 PNG를 독립 검사한다."""
    if not stream.endswith(b"\r\n"):
        raise ValueError("missing final CRLF")
    rest = stream[:-2]
    payloads = []
    more = True
    while rest:
        if not more or not rest.startswith(b"\x1b_G"):
            raise ValueError("unexpected bytes outside Kitty APC")
        end = rest.find(b"\x1b\\", 3)
        if end == -1:
            raise ValueError("unterminated Kitty APC")
        header, payload = rest[3:end].split(b";", 1)
        items = [item.split(b"=", 1) for item in header.split(b",")]
        keys = dict(items)
        if len(keys) != len(items):
            raise ValueError("duplicate control key")
        expected = {b"m", b"q"} if payloads else {b"a", b"f", b"t", b"q", b"m"}
        if keys.keys() != expected or keys[b"q"] != b"2" or keys[b"m"] not in (b"0", b"1"):
            raise ValueError("invalid chunk controls")
        if not payloads and (keys[b"a"], keys[b"f"], keys[b"t"]) != (b"T", b"100", b"d"):
            raise ValueError("not direct PNG transmit-and-display")
        more = keys[b"m"] == b"1"
        if not 0 < len(payload) <= 4096 or len(payload) % 4 or (more and b"=" in payload):
            raise ValueError("invalid Base64 chunk boundary")
        # Validate every chunk as well as the assembled payload.
        base64.b64decode(payload, validate=True)
        payloads.append(payload)
        rest = rest[end + 2:]
    if more or not payloads:
        raise ValueError("missing final chunk")
    png = base64.b64decode(b"".join(payloads), validate=True)
    if not png.startswith(b"\x89PNG\r\n\x1a\n"):
        raise ValueError("payload is not PNG")
    image = Image.open(io.BytesIO(png)).convert("RGBA")
    image.load()
    return png, image, len(payloads)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(args, root):
    result = subprocess.run([str(arg) for arg in args], capture_output=True, check=False)
    with (root / "commands.jsonl").open("a", encoding="utf-8") as log:
        log.write(json.dumps({"args": [str(a) for a in args], "exit": result.returncode,
                              "stderr": result.stderr.decode(errors="replace")}, ensure_ascii=False) + "\n")
    if result.returncode:
        raise RuntimeError(f"exit {result.returncode}: {args}: {result.stderr.decode(errors='replace')}")
    return result.stdout


def png_at_width(binary, source, page, width, page_width, directory, root):
    scale = width / page_width
    if math.ceil(page_width * scale) != width:
        scale = math.nextafter(scale, 0)
    run([binary, "export-png", source, "-p", page, "--profile", "print",
         "--scale", repr(scale), "-o", directory], root)
    return Image.open(directory / f"{source.stem}.png").convert("RGBA")


def require_same(left, right, label):
    if left.size != right.size or left.tobytes() != right.tobytes():
        raise AssertionError(f"pixels differ: {label}: {left.size}/{right.size}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rhwp-bin", required=True, type=Path)
    parser.add_argument("--baseline-bin", required=True, type=Path)
    parser.add_argument("--input", required=True, action="append", type=Path)
    parser.add_argument("--width", type=int, action="append")
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    # Refuse stale output rather than mixing evidence from different producers.
    if (args.out / "commands.jsonl").exists():
        parser.error("--out already contains a run; select an empty output directory")
    manifest = {"candidate": {"path": str(args.rhwp_bin), "sha256": sha(args.rhwp_bin)},
                "base": {"path": str(args.baseline_bin), "sha256": sha(args.baseline_bin)},
                "documents": [], "pages": []}
    for source in args.input:
        root = args.out / source.stem
        root.mkdir()
        count = json.loads(run([args.rhwp_bin, "export-png", source, "--metadata-only"], args.out))["pageCount"]
        assert count > 0, source
        svg_dir = root / "svg"
        # Print uses the common physical paper viewport; default screen SVG
        # dimensions are a different contract and can differ by one raster row.
        run([args.rhwp_bin, "export-svg", source, "--backend", "layer", "--profile", "print",
             "-o", svg_dir], args.out)
        svgs = sorted(svg_dir.glob("*.svg"))
        assert len(svgs) == count, (source, count, len(svgs))
        for binary, label in [(args.baseline_bin, "base"), (args.rhwp_bin, "candidate")]:
            run([binary, "export-png", source, "--profile", "print", "-o", root / label], args.out)
            assert len(list((root / label).glob("*.png"))) == count
        for page in range(count):
            name = source.stem if count == 1 else f"{source.stem}_{page + 1:03}"
            require_same(Image.open(root / "base" / f"{name}.png").convert("RGBA"),
                         Image.open(root / "candidate" / f"{name}.png").convert("RGBA"), f"default PNG {source}/{page}")
        manifest["documents"].append({"input": str(source), "sha256": sha(source), "pages": count,
                                      "default_png_pixels_unchanged": True})
        for width in args.width or [360, 720]:
            folder = root / f"width-{width}"
            folder.mkdir()
            stdout = run([args.rhwp_bin, "export-kitty", source, "--width", width,
                          "--profile", "print", "-o", folder / "kitty"], args.out)
            assert not stdout, "file mode must have empty stdout"
            assert len(list((folder / "kitty").glob("*.kitty"))) == count
            for binary, label in [(args.baseline_bin, "base-sixel"), (args.rhwp_bin, "candidate-sixel")]:
                assert not run([binary, "export-sixel", source, "--width", width,
                                "--profile", "print", "-o", folder / label], args.out)
                assert len(list((folder / label).glob("*.six"))) == count
            for page in range(count):
                name = source.stem if count == 1 else f"{source.stem}_{page + 1:03}"
                stream_path = folder / "kitty" / f"{name}.kitty"
                stream = run([args.rhwp_bin, "export-kitty", source, "--width", width,
                              "--profile", "print", "-p", page], args.out)
                assert stream_path.read_bytes() == stream, "file and stdout differ"
                png, restored, chunks = decode_stream(stream)
                (folder / f"page-{page + 1:03}-decoded.png").write_bytes(png)
                svg = ET.parse(svgs[page]).getroot()
                page_width = float(re.sub(r"px$", "", svg.attrib["width"]))
                page_height = float(re.sub(r"px$", "", svg.attrib["height"]))
                expected_size = (width, math.ceil(page_height * (width / page_width)))
                assert restored.size == expected_size, (source, page, restored.size, expected_size)
                before = png_at_width(args.baseline_bin, source, page, width, page_width,
                                      folder / f"page-{page + 1:03}-base", args.out)
                after = png_at_width(args.rhwp_bin, source, page, width, page_width,
                                     folder / f"page-{page + 1:03}-candidate", args.out)
                require_same(before, after, f"scaled PNG {source}/{page}/{width}")
                white = Image.alpha_composite(Image.new("RGBA", after.size, "white"), after)
                require_same(white, restored, f"Kitty PNG {source}/{page}/{width}")
                old_sixel = folder / "base-sixel" / f"{name}.six"
                new_sixel = folder / "candidate-sixel" / f"{name}.six"
                assert old_sixel.read_bytes() == new_sixel.read_bytes(), "Sixel stream changed"
                manifest["pages"].append({"input": str(source), "page": page, "width": width,
                                          "height": restored.height, "chunks": chunks,
                                          "stream_bytes": len(stream), "png_sha256": hashlib.sha256(png).hexdigest(),
                                          "kitty_matches_white_png_pixels": True, "png_pixels_unchanged": True,
                                          "sixel_bytes_unchanged": True, "file_equals_stdout": True})
            last = count - 1
            selected = folder / "selected-last"
            assert not run([args.rhwp_bin, "export-kitty", source, "-p", last, "--width", width,
                            "--profile", "print", "-o", selected], args.out)
            last_name = source.stem if count == 1 else f"{source.stem}_{count:03}"
            assert (selected / f"{source.stem}.kitty").read_bytes() == (folder / "kitty" / f"{last_name}.kitty").read_bytes()
    (args.out / "results.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"PASS: {len(manifest['documents'])} documents, {len(manifest['pages'])} page/width cases")


if __name__ == "__main__":
    main()
