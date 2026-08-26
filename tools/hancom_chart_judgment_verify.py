#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""한컴 판정 원장(MANIFEST.json)을 제3자가 재계산한다 — #5447 B2 스파이크.

`samples/issue5447/MANIFEST.json` 은 "어떤 파일을 한컴 2022 에 넣어 어떤 PDF 를 돌려받았고,
그 그림이 대조군과 어떻게 달랐는가"를 파일별 SHA-256 으로 적어 둔 원장이다. 이 스크립트는
그 원장을 **다시 계산해서** 맞는지 본다. 보고서를 믿지 않아도 되게 만드는 것이 목적이다.

    python tools/hancom_chart_judgment_verify.py
    python tools/hancom_chart_judgment_verify.py --rasterizer pdftoppm
    python tools/hancom_chart_judgment_verify.py --rasterizer none   # 해시만, 렌더 없이

로컬 실행 원칙(`scripts/check_markdown_links.py` 와 같은 운용). CI 는 원본·PDF 의 SHA-256
대조만 `tests/issue_4100_chart_data_edit.rs` 에서 상시로 돈다 — 렌더러 의존성이 없어서다.

## 래스터 해시는 도구에 딸린 값이다

절대 해시는 렌더러(와 그 버전)마다 다르다. 그래서 원장은 두 축을 따로 적는다.

- `pymupdf_144dpi_rgb_sha256` — PyMuPDF 1쪽 144dpi RGB 픽스맵 raw 바이트
- `pdftoppm_144dpi_ppm_sha256` — poppler `pdftoppm -r 144 -f 1 -l 1` PPM (보고서 §2 의 축)

**도구를 넘어 성립해야 하는 것은 절대 해시가 아니라 `invariants` 의 동치 관계다.**
그래서 이 스크립트는 어떤 rasterizer 로 돌든 invariants 를 전건 재판정한다.

## 이 스크립트가 일부러 시끄러운 이유

보고서 §6-1 이 기록한 실제 사고 — 한컴을 왕복한 파일명이 NFD 로 바뀌어 대조군 매칭이
0건이 됐는데, 판정 스크립트는 예외 없이 끝까지 돌았고 표까지 그럴듯하게 출력했다.
대조군 해시가 `None` 인 채로 계속 진행한 것이 문제였다. 그래서 여기서는

- 파일명을 NFC 로 정규화한 뒤 맞추고,
- **비었거나 못 찾은 값은 전부 즉시 실패**로 낸다. 조용한 통과가 없다.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
import unicodedata
from pathlib import Path

# 판정 대상이 전부 한글 파일명이다. Windows 기본 콘솔(cp949)에서 표를 찍다 죽지 않게 한다.
for _stream in (sys.stdout, sys.stderr):
    if hasattr(_stream, "reconfigure"):
        _stream.reconfigure(encoding="utf-8", errors="replace")

DEFAULT_MANIFEST = Path("samples/issue5447/MANIFEST.json")
DOCKER_HINT = (
    "poppler 가 없으면 컨테이너로 같은 값을 낼 수 있다:\n"
    "  docker run --rm -v \"$PWD/pdf/issue5447:/w:ro\" minidocks/poppler sh -c \\\n"
    "    'cd /w; for f in *.pdf; do pdftoppm -r 144 -f 1 -l 1 \"$f\" /tmp/o; "
    "sha256sum /tmp/o-1.ppm; rm -f /tmp/o-1.ppm; done'"
)


class Report:
    """실패를 모아 뒀다가 마지막에 한꺼번에 낸다 — 첫 실패에서 멈추면 전모가 안 보인다."""

    def __init__(self) -> None:
        self.failures: list[str] = []
        self.checks = 0

    def check(self, ok: bool, message: str) -> bool:
        self.checks += 1
        if not ok:
            self.failures.append(message)
        return ok

    def fail(self, message: str) -> None:
        self.checks += 1
        self.failures.append(message)


def nfc(text: str) -> str:
    return unicodedata.normalize("NFC", text)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def raster_pymupdf(pdf: Path, dpi: int) -> tuple[str, int, int, int]:
    try:
        import fitz  # PyMuPDF
    except ImportError as exc:  # pragma: no cover - 환경 안내
        raise SystemExit(f"PyMuPDF 가 필요하다: pip install pymupdf ({exc})") from exc
    doc = fitz.open(pdf)
    try:
        pix = doc[0].get_pixmap(dpi=dpi, colorspace=fitz.csRGB, alpha=False)
        return hashlib.sha256(pix.samples).hexdigest(), pix.width, pix.height, doc.page_count
    finally:
        doc.close()


def raster_pdftoppm(pdf: Path, dpi: int) -> tuple[str, int, int, int]:
    exe = shutil.which("pdftoppm")
    if not exe:
        raise SystemExit(f"pdftoppm 을 찾지 못했다 (poppler-utils).\n{DOCKER_HINT}")
    with tempfile.TemporaryDirectory() as tmp:
        prefix = Path(tmp) / "page"
        subprocess.run(
            [exe, "-r", str(dpi), "-f", "1", "-l", "1", str(pdf), str(prefix)],
            check=True,
            capture_output=True,
        )
        produced = sorted(Path(tmp).glob("page*.ppm"))
        if len(produced) != 1:
            raise SystemExit(f"{pdf.name}: pdftoppm 산출이 1개가 아니다 ({len(produced)})")
        data = produced[0].read_bytes()
    # PPM 헤더: P6\n{w} {h}\n255\n
    header = data.split(b"\n", 3)
    width, height = (int(v) for v in header[1].split())
    return hashlib.sha256(data).hexdigest(), width, height, 1


RASTERIZERS = {
    "pymupdf": ("pymupdf_144dpi_rgb_sha256", raster_pymupdf),
    "pdftoppm": ("pdftoppm_144dpi_ppm_sha256", raster_pdftoppm),
}


def verify_files(entries: list[dict], root: Path, report: Report) -> None:
    """원본과 한컴 PDF 가 실재하고 원장의 SHA-256 과 같은가."""
    for entry in entries:
        for path_key, hash_key in (
            ("original_path", "original_sha256"),
            ("hancom_pdf_path", "hancom_pdf_sha256"),
        ):
            rel = entry[path_key]
            recorded = entry[hash_key]
            if not recorded:
                report.fail(f"{entry['name']}: 원장의 {hash_key} 가 비었다")
                continue
            path = root / rel
            if not path.is_file():
                report.fail(f"{entry['name']}: {rel} 이(가) 없다")
                continue
            actual = sha256_file(path)
            report.check(
                actual == recorded,
                f"{entry['name']}: {rel} SHA-256 불일치\n"
                f"    원장 {recorded}\n    실제 {actual}",
            )


def verify_raster(
    entries: list[dict], root: Path, report: Report, rasterizer: str, dpi: int
) -> dict[str, str]:
    """PDF 를 다시 렌더해 원장의 래스터 해시와 대조하고, 실제 해시를 돌려준다."""
    hash_key, render = RASTERIZERS[rasterizer]
    computed: dict[str, str] = {}
    for entry in entries:
        pdf = root / entry["hancom_pdf_path"]
        if not pdf.is_file():
            continue  # 파일 단계에서 이미 실패로 기록됐다
        digest, width, height, pages = render(pdf, dpi)
        computed[entry["name"]] = digest
        recorded = entry["raster"].get(hash_key)
        if recorded is None:
            report.fail(
                f"{entry['name']}: 원장에 {hash_key} 가 기록돼 있지 않다 "
                f"(--rasterizer 를 바꾸거나 원장을 채워라)"
            )
        else:
            report.check(
                digest == recorded,
                f"{entry['name']}: 래스터({rasterizer}) 불일치\n"
                f"    원장 {recorded}\n    실제 {digest}",
            )
        report.check(
            (width, height, pages) == (entry["raster"]["width"], entry["raster"]["height"], entry["raster"]["pages"]),
            f"{entry['name']}: 판형 불일치 — 원장 "
            f"{entry['raster']['width']}x{entry['raster']['height']} {entry['raster']['pages']}쪽, "
            f"실제 {width}x{height} {pages}쪽",
        )
    return computed


def verify_invariants(
    manifest: dict, computed: dict[str, str], report: Report
) -> None:
    """렌더러가 달라도 성립해야 하는 관계 — 판정의 진짜 계약."""
    entries = {e["name"]: e for e in manifest["entries"]}

    def digest(name: str) -> str | None:
        if name not in entries:
            report.fail(f"불변식이 원장에 없는 파일을 가리킨다: {name}")
            return None
        if computed:
            value = computed.get(name)
            if not value:
                report.fail(f"{name}: 래스터 해시를 계산하지 못했다 (조용한 통과 금지)")
            return value
        value = entries[name]["raster"]["pymupdf_144dpi_rgb_sha256"]
        if not value:
            report.fail(f"{name}: 원장의 래스터 해시가 비었다")
        return value

    for inv in manifest["invariants"]:
        kind = inv["kind"]
        if kind in ("raster_equal", "raster_differs"):
            left, right = digest(inv["a"]), digest(inv["b"])
            if not left or not right:
                continue
            same = left == right
            want_same = kind == "raster_equal"
            report.check(
                same == want_same,
                f"불변식 실패 [{kind}] {inv['a']} vs {inv['b']}\n"
                f"    기대: {'같아야' if want_same else '달라야'} 한다 — {inv['why']}",
            )
        elif kind == "page_geometry":
            for entry in manifest["entries"]:
                raster = entry["raster"]
                report.check(
                    (raster["width"], raster["height"], raster["pages"])
                    == (inv["width"], inv["height"], inv["pages"]),
                    f"불변식 실패 [page_geometry] {entry['name']}: "
                    f"{raster['width']}x{raster['height']} {raster['pages']}쪽",
                )
        elif kind == "counts":
            units: dict[str, set[str]] = {}
            for entry in manifest["entries"]:
                if entry["role"] == "control":
                    continue
                key = (
                    entry["name"].rsplit(".", 1)[0]
                    if entry["role"] == "conversion"
                    else f"{entry['base_document']}-{entry['variant']}"
                )
                units.setdefault(key, set()).add(entry["verdict"])
            for key, verdicts in sorted(units.items()):
                report.check(
                    len(verdicts) == 1,
                    f"불변식 실패 [counts] {key}: 포맷 간 판정이 갈린다 {sorted(verdicts)}",
                )
            tally: dict[str, int] = {}
            for verdicts in units.values():
                tally[next(iter(verdicts))] = tally.get(next(iter(verdicts)), 0) + 1
            report.check(
                len(units) == inv["judgment_units"],
                f"불변식 실패 [counts] 판정 단위 {len(units)} != 원장 {inv['judgment_units']}",
            )
            report.check(
                tally == inv["tally"],
                f"불변식 실패 [counts] 판정 분포 {tally} != 원장 {inv['tally']}",
            )
        else:
            report.fail(f"모르는 불변식 종류: {kind}")


def verify_verdicts(manifest: dict, computed: dict[str, str], report: Report) -> None:
    """판정 칸이 래스터 관계와 실제로 합치하는가 — 원장이 스스로를 증명해야 한다."""
    entries = {e["name"]: e for e in manifest["entries"]}
    source = computed or {
        n: e["raster"]["pymupdf_144dpi_rgb_sha256"] for n, e in entries.items()
    }
    for entry in manifest["entries"]:
        if entry["role"] == "control":
            report.check(entry["verdict"] == "대조군", f"{entry['name']}: 대조군 판정이 아니다")
            continue
        control = entry["control"]
        if control not in entries:
            report.fail(f"{entry['name']}: 대조군 {control} 을(를) 원장에서 찾지 못했다")
            continue
        mine, theirs = source.get(entry["name"]), source.get(control)
        if not mine or not theirs:
            report.fail(f"{entry['name']}: 대조군 대조에 쓸 해시가 비었다 (조용한 통과 금지)")
            continue
        changed = mine != theirs
        expected_changed = entry["verdict"] in ("반영", "반영_의미깨짐")
        report.check(
            changed == expected_changed,
            f"{entry['name']}: 판정 '{entry['verdict']}' 인데 대조군 대비 "
            f"{'변화 없음' if not changed else '변화 있음'}",
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--manifest", type=Path, default=None, help="판정 원장 JSON")
    parser.add_argument(
        "--rasterizer",
        choices=["pymupdf", "pdftoppm", "none"],
        default="pymupdf",
        help="래스터 재계산에 쓸 도구. none 이면 파일 SHA-256 과 원장 내부 정합만 본다",
    )
    parser.add_argument("--dpi", type=int, default=144)
    parser.add_argument("--quiet", action="store_true", help="표를 찍지 않는다")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    manifest_path = args.manifest or (repo_root / DEFAULT_MANIFEST)
    if not manifest_path.is_file():
        print(f"원장을 찾지 못했다: {manifest_path}", file=sys.stderr)
        return 2
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))

    for entry in manifest["entries"]:
        entry["name"] = nfc(entry["name"])
        if entry["control"]:
            entry["control"] = nfc(entry["control"])
    for inv in manifest["invariants"]:
        for key in ("a", "b"):
            if key in inv:
                inv[key] = nfc(inv[key])

    entries = manifest["entries"]
    report = Report()

    if not entries:
        print("원장이 비었다 — 검증할 것이 없다는 것은 실패다", file=sys.stderr)
        return 1

    verify_files(entries, repo_root, report)
    computed: dict[str, str] = {}
    if args.rasterizer != "none":
        if args.dpi != manifest["raster"]["dpi"]:
            report.fail(f"--dpi {args.dpi} 가 원장의 {manifest['raster']['dpi']} 와 다르다")
        computed = verify_raster(entries, repo_root, report, args.rasterizer, args.dpi)
    verify_verdicts(manifest, computed, report)
    verify_invariants(manifest, computed, report)

    if not args.quiet:
        print(f"# {manifest['title']}")
        print(f"  원장    : {manifest_path.relative_to(repo_root)}")
        print(f"  한컴    : {manifest['hancom']['version']} ({manifest['hancom']['converted_on']})")
        print(f"  래스터  : {args.rasterizer} @ {args.dpi}dpi")
        print()
        print(f"  {'파일':<52}{'판정':<14}{'래스터(앞 16)'}")
        for entry in entries:
            digest = (computed.get(entry["name"]) or entry["raster"]["pymupdf_144dpi_rgb_sha256"] or "")
            print(f"  {entry['name']:<52}{entry['verdict']:<14}{digest[:16]}")
        print()
        counts = manifest["counts"]
        print(f"  판정 단위 {counts['judgment_units']} — {counts['tally']}")

    print()
    if report.failures:
        print(f"실패 {len(report.failures)}건 / 검사 {report.checks}건", file=sys.stderr)
        for failure in report.failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print(f"통과 — 검사 {report.checks}건 전건 일치 ({len(entries)} 파일)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
