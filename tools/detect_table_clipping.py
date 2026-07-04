"""#1658 시각 검증 인프라 — 본문 영역 클리핑(overflow) 검출.

페이지네이션 용량 정합(per-page capacity) 작업의 핵심 리스크는 **클리핑**: 표/텍스트가 본문
영역(body) 아래(footer/여백)로 그려져 body-clip 에 의해 잘려 보이지 않는(데이터 손실) 상태.
페이지수 게이트(render_page_gate)로는 못 잡는다. 본 도구는 rhwp SVG 를 파싱해, body-clip 그룹
안에서 본문 하단(body_bottom)을 초과하는 콘텐츠를 검출한다.

원리: rhwp SVG 는 transform 없는 절대좌표. `<clipPath id="body-clip-N"><rect .../></clipPath>` 가
본문 영역. 같은 N 의 `<g clip-path="url(#body-clip-N)">` 안의 요소(text/rect/line) 하단 Y 가
body_bottom + EPS 를 넘으면 클리핑(잘림)으로 판정한다.

사용:
    python tools/detect_table_clipping.py <file.hwp|hwpx> [--exe target/release/rhwp.exe] [--eps 1.0]
    python tools/detect_table_clipping.py --batch <폴더> [--sample N --seed S]
종료코드: 클리핑 1건↑ → 1.
"""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET
from pathlib import Path

SVG_NS = "{http://www.w3.org/2000/svg}"


def norm_path(p) -> Path:
    """MSYS/Git-Bash 경로(/c/Users/..)를 Windows 경로(C:\\Users\\..)로 정규화.

    bash↔python 혼용 시 `/c/...` 가 Windows python 에 그대로 넘어와 glob/open 이
    무음 실패(0 파일)하던 재현성 버그 방지. 그 외 경로는 그대로 둔다.
    """
    s = str(p)
    if sys.platform == "win32":
        m = re.match(r"^/([a-zA-Z])/(.*)$", s)
        if m:
            return Path(f"{m.group(1).upper()}:/{m.group(2)}")
    return Path(s)


def _body_regions(root: ET.Element) -> dict[str, float]:
    """clipPath id=body-clip-N → body_bottom(y+height)."""
    out: dict[str, float] = {}
    for cp in root.iter(f"{SVG_NS}clipPath"):
        cid = cp.get("id", "")
        if not cid.startswith("body-clip"):
            continue
        rect = cp.find(f"{SVG_NS}rect")
        if rect is None:
            continue
        y = float(rect.get("y", "0"))
        h = float(rect.get("height", "0"))
        out[cid] = y + h
    return out


def _elem_bottom(el: ET.Element) -> float | None:
    """요소의 하단 Y 추정 (text=baseline y, rect=y+h, line=max(y1,y2))."""
    tag = el.tag.replace(SVG_NS, "")
    try:
        if tag == "text":
            return float(el.get("y")) if el.get("y") is not None else None
        if tag == "rect":
            return float(el.get("y", "0")) + float(el.get("height", "0"))
        if tag == "line":
            return max(float(el.get("y1", "0")), float(el.get("y2", "0")))
    except (TypeError, ValueError):
        return None
    return None


def detect_in_svg(svg_path: Path, eps: float) -> list[tuple[str, float, float]]:
    """body-clip 그룹 내 body_bottom 초과 콘텐츠 → [(clip_id, max_bottom, body_bottom)]."""
    root = ET.parse(svg_path).getroot()
    bottoms = _body_regions(root)
    if not bottoms:
        return []
    findings: list[tuple[str, float, float]] = []
    for g in root.iter(f"{SVG_NS}g"):
        cp = g.get("clip-path", "")
        m = re.match(r"url\(#(body-clip[\w-]*)\)", cp)
        if not m:
            continue
        cid = m.group(1)
        body_bottom = bottoms.get(cid)
        if body_bottom is None:
            continue
        max_bottom = 0.0
        for el in g.iter():
            b = _elem_bottom(el)
            if b is not None and b > max_bottom:
                max_bottom = b
        if max_bottom > body_bottom + eps:
            findings.append((cid, max_bottom, body_bottom))
    return findings


def check_file(path: Path, exe: str, eps: float) -> tuple[int, int, float]:
    """(클리핑 페이지수, 전체 페이지수, 최대 overflow px)."""
    path = norm_path(path)
    exe = str(norm_path(exe))
    with tempfile.TemporaryDirectory() as td:
        r = subprocess.run(
            [exe, "export-svg", str(path), "-o", td],
            capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=180,
        )
        if r.returncode != 0:
            return (-1, 0, 0.0)
        svgs = sorted(Path(td).glob("*.svg"))
        clipped = 0
        max_ov = 0.0
        for s in svgs:
            f = detect_in_svg(s, eps)
            if f:
                clipped += 1
                max_ov = max(max_ov, max(b - bb for _, b, bb in f))
        return (clipped, len(svgs), max_ov)


def main() -> int:
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("files", nargs="*", default=[], type=Path)
    g.add_argument("--batch", type=Path)
    ap.add_argument("--sample", type=int, default=0)
    ap.add_argument("--seed", type=int, default=1658)
    ap.add_argument("--exe", default="target/release/rhwp.exe" if sys.platform == "win32"
                    else "target/release/rhwp")
    ap.add_argument("--eps", type=float, default=1.0, help="허용 overflow(px)")
    a = ap.parse_args()

    if a.batch:
        files = [p for p in sorted(a.batch.rglob("*"))
                 if p.suffix.lower() in (".hwp", ".hwpx")]
        if a.sample and len(files) > a.sample:
            import random
            files = sorted(random.Random(a.seed).sample(files, a.sample))
    else:
        files = a.files
    if not files:
        print("대상 파일 없음", file=sys.stderr)
        return 2

    n_clip = n_err = 0
    for f in files:
        clipped, pages, max_ov = check_file(f, a.exe, a.eps)
        if clipped < 0:
            n_err += 1
            print(f"ERR  {f.name}")
            continue
        if clipped > 0:
            n_clip += 1
            print(f"CLIP {clipped}/{pages}p  max_overflow={max_ov:.1f}px  {f.name}")
    print(f"\n[clipping] 파일={len(files)} 클리핑={n_clip} ERR={n_err}")
    return 1 if n_clip > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
