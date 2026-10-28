#!/usr/bin/env python3
"""#8160 전 페이지 시각 회귀: 실제 Native/Canvas PNG를 독립 Print와 재비교한다.

--runs-json은 {"native": {"hwpx": "...", ...}, "wasm": {...}} 형식이다.
경로는 각 Visual Sweep의 silhouette_manifest.json·rhwp_png·pdf_png를 가진 폴더다.
점수 TSV나 저장된 통과 판정을 읽지 않고 384개 PNG의 정수 미대응을 계산한다.
"""

from __future__ import annotations

import argparse
from concurrent.futures import ProcessPoolExecutor
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageChops, ImageFilter


ROOT = Path(__file__).resolve().parent.parent
KEYS = ("hwpx", "hwp2024", "hwp2022")
BACKENDS = {"native": "native-skia", "wasm": "wasm-canvas2d"}


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compare_page(task):
    backend, key, page, candidate, reference = task
    with Image.open(candidate) as opened:
        left = opened.convert("RGB")
    with Image.open(reference) as opened:
        right = opened.convert("RGB")
    size = max(left.width, right.width), max(left.height, right.height)

    def mask(source):
        padded = Image.new("RGB", size, "white")
        padded.paste(source, (0, 0))
        red, green, blue = padded.split()
        return ImageChops.darker(ImageChops.darker(red, green), blue).point(
            lambda value: 255 if value < 232 else 0
        )

    left_mask, right_mask = mask(left), mask(right)
    counts = [
        ImageChops.subtract(a, b.filter(ImageFilter.MaxFilter(5))).histogram()[255]
        for a, b in ((left_mask, right_mask), (right_mask, left_mask))
    ]
    return {
        "backend": backend, "document": key, "page": page,
        "candidate_png_sha256": sha256(candidate),
        "reference_png_sha256": sha256(reference),
        "candidate_unmatched": counts[0], "reference_unmatched": counts[1],
        "raw100": counts == [0, 0],
    }


def collect_tasks(runs):
    if set(runs) != set(BACKENDS):
        raise ValueError("Native와 fresh WASM의 전쪽 실행이 모두 필요합니다")
    tasks, identities = [], {}
    expected_pages = list(range(1, 65))
    for backend, raster_backend in BACKENDS.items():
        if set(runs[backend]) != set(KEYS):
            raise ValueError(f"{backend}: 세 문서의 실행이 모두 필요합니다")
        for key in KEYS:
            contract = json.loads((ROOT / f"tests/fixtures/issue8160/{key}.json").read_text())
            for role in ("sample", "pdf"):
                if sha256(ROOT / contract[role]) != contract[f"{role}_sha256"]:
                    raise ValueError(f"원본 또는 독립 Print가 변경됐습니다: {contract[role]}")
            folder = Path(runs[backend][key]).resolve()
            manifest_path = folder / "silhouette_manifest.json"
            manifest = json.loads(manifest_path.read_text())
            provenance = manifest["provenance"]
            run = json.loads((folder / "run_manifest.json").read_text())
            if (manifest["run_state"] != "complete" or run["dpi"] != 96
                    or manifest["requested_pages"] != expected_pages
                    or manifest["completed_pages"] != expected_pages or manifest["missing_pages"]
                    or run["document_page_counts"] != {"rhwp": 64, "pdf": 64}
                    or provenance["raster_backend"] != raster_backend
                    or provenance["comparison_profile"] != "print"
                    or provenance["hwp"]["sha256"] != contract["sample_sha256"]
                    or provenance["pdf"]["sha256"] != contract["pdf_sha256"]):
                raise ValueError(f"실제 전쪽 Print 실행 신원이 다릅니다: {manifest_path}")
            for subdir, prefix in (("rhwp_png", "rhwp_"), ("pdf_png", "pdf-")):
                expected = {f"{prefix}{page:03}.png" for page in expected_pages}
                actual = {path.name for path in (folder / subdir).glob("*.png")}
                if expected != actual:
                    raise ValueError(f"쪽 누락·추가 또는 전체 쪽수 불일치: {folder / subdir}")
            identities[f"{backend}/{key}"] = {
                "manifest_sha256": sha256(manifest_path), "provenance": provenance,
            }
            tasks.extend((backend, key, page, folder / f"rhwp_png/rhwp_{page:03}.png",
                          folder / f"pdf_png/pdf-{page:03}.png") for page in expected_pages)
    return tasks, identities


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runs-json", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--workers", type=int, default=2)
    args = parser.parse_args()
    if not 1 <= args.workers <= 12:
        parser.error("workers는 1~12 사이여야 합니다")
    tasks, identities = collect_tasks(json.loads(args.runs_json.read_text()))
    with ProcessPoolExecutor(max_workers=args.workers) as pool:
        pages = list(pool.map(compare_page, tasks))
    failed = [page for page in pages if not page["raw100"]]
    report = {
        "contract": "issue8160/384-print-pages-raw100",
        "metric": {"threshold": 232, "symmetric_radius_pixels": 2, "white_pad": True},
        "pages_checked": len(pages), "raw100": len(pages) - len(failed),
        "failed": failed, "runs": identities, "pages": pages,
    }
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(f"raw 100%: {report['raw100']}/{len(pages)}, 실패: {len(failed)}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
