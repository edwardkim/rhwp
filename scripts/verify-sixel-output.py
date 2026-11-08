#!/usr/bin/env python3
"""PNG 무회귀와 독립 libsixel Sixel 복원 검사. 한컴 Visual Sweep과 구분한다."""

import argparse
import hashlib
import json
import math
import re
import subprocess
import xml.etree.ElementTree as ET
from pathlib import Path

from PIL import Image, ImageChops, ImageStat


def execute(args, log):
    result = subprocess.run([str(arg) for arg in args], capture_output=True, check=False)
    log.with_suffix(".stderr").write_bytes(result.stderr)
    if result.returncode:
        raise RuntimeError(f"exit {result.returncode}: {args}: {result.stderr.decode(errors='replace')}")
    return result.stdout


def export_png(binary, source, output, page, scale=None):
    args = [binary, "export-png", source, "-p", page, "--profile", "print", "-o", output]
    if scale is not None:
        args.extend(["--scale", str(scale)])
    output.mkdir(parents=True, exist_ok=True)
    execute(args, output / "export")
    pngs = list(output.glob("*.png"))
    if len(pngs) != 1:
        raise RuntimeError(f"expected one PNG: {output}")
    return pngs[0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rhwp-bin", required=True, type=Path)
    parser.add_argument("--baseline-bin", required=True, type=Path)
    parser.add_argument("--decoder", required=True, type=Path)
    parser.add_argument("--input", required=True, action="append", type=Path)
    parser.add_argument("--width", type=int, action="append")
    parser.add_argument("--page", type=int, default=0, help="0부터 시작")
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    records = []
    for source in args.input:
        root = args.out / source.stem
        root.mkdir(parents=True, exist_ok=True)
        before = export_png(args.baseline_bin, source, root / "before", args.page)
        after = export_png(args.rhwp_bin, source, root / "after", args.page)
        before_rgba, after_rgba = Image.open(before).convert("RGBA"), Image.open(after).convert("RGBA")
        if before_rgba.size != after_rgba.size or before_rgba.tobytes() != after_rgba.tobytes():
            raise RuntimeError(f"PNG decoded pixels changed: {source}")
        svg_dir = root / "svg"
        execute([args.rhwp_bin, "export-svg", source, "-p", args.page, "-o", svg_dir], root / "svg-export")
        svg = ET.parse(next(svg_dir.glob("*.svg"))).getroot()
        page_width = float(re.sub(r"px$", "", svg.attrib["width"]))
        page_height = float(re.sub(r"px$", "", svg.attrib["height"]))
        for width in args.width or [360, 720]:
            destination = root / f"width-{width}"
            destination.mkdir(exist_ok=True)
            scale = width / page_width
            # 부동소수점의 ceil 오차로 1px 늘어나지 않는 동일 배율을 지정한다.
            if math.ceil(page_width * scale) != width:
                scale = math.nextafter(scale, 0)
            expected_png = export_png(args.rhwp_bin, source, destination / "png", args.page, scale)
            old_png = export_png(args.baseline_bin, source, destination / "old-png", args.page, scale)
            expected_rgba = Image.open(expected_png).convert("RGBA")
            old_rgba = Image.open(old_png).convert("RGBA")
            if old_rgba.size != expected_rgba.size or old_rgba.tobytes() != expected_rgba.tobytes():
                raise RuntimeError(f"scaled PNG pixels changed: {source}, {width}")
            expected = Image.alpha_composite(Image.new("RGBA", expected_rgba.size, "white"), expected_rgba).convert("RGB")
            stream = execute([args.rhwp_bin, "export-sixel", source, "--page", args.page,
                              "--width", width, "--profile", "print"], destination / "sixel-export")
            if not stream.startswith(b"\x1bP") or not stream.endswith(b"\x1b\\") or stream.count(b"\x1b") != 2:
                raise RuntimeError("stdout is not exactly one Sixel DCS stream")
            six = destination / "page.six"
            six.write_bytes(stream)
            restored_png = destination / "decoded.png"
            execute([args.decoder, "-i", six, "-o", restored_png], destination / "decode")
            restored = Image.open(restored_png).convert("RGB")
            expected_size = (width, math.ceil(page_height * (width / page_width)))
            if restored.size != expected_size or restored.size != expected.size:
                raise RuntimeError(f"Sixel dimensions differ: {restored.size}, expected {expected_size}/{expected.size}")
            difference = ImageChops.difference(expected, restored)
            histogram = difference.histogram()
            combined = [sum(histogram[channel * 256 + value] for channel in range(3)) for value in range(256)]
            total = sum(combined)
            cumulative = 0
            p99 = 255
            for value, count in enumerate(combined):
                cumulative += count
                if cumulative >= total * 0.99:
                    p99 = value
                    break
            mae = sum(ImageStat.Stat(difference).mean) / 3
            # 사전 품질 예산: 평균 채널 오차 4/255 이하, 99% 채널 오차 24/255 이하.
            # 이는 팔레트 색 손실의 예산이며 조판 누락·변형 허용치가 아니다.
            passed = mae <= 4.0 and p99 <= 24
            expected.save(destination / "reference-white.png")
            comparison = Image.new("RGB", (width * 2, restored.height), "white")
            comparison.paste(expected, (0, 0))
            comparison.paste(restored, (width, 0))
            comparison.save(destination / "compare.png")
            records.append({"input": str(source), "input_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                            "page": args.page, "width": width, "height": restored.height,
                            "png_decoded_pixels_unchanged": True, "stream_bytes": len(stream),
                            "rgb_mean_absolute_error": mae, "rgb_channel_error_p99": p99,
                            "quantization_budget_pass": passed, "comparison": str(destination / "compare.png")})
    (args.out / "results.json").write_text(json.dumps(records, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps(records, ensure_ascii=False, indent=2))
    if any(not record["quantization_budget_pass"] for record in records):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
