# -*- coding: utf-8 -*-
"""쪽 내 PI 시작위치 오라클 — 저장 사다리 vs 렌더 y 의 쪽-내부 편차 검출 (COM 불필요).

10k PI 오라클의 사각(같은 쪽 안의 위치 이동 — #4490 91px 겹침·#4491 21px 밀림이 쪽수·
PI쪽·픽셀평균 모두 침묵)을 메운다. `dump-extents` 의 TextLine 이 렌더 y 와 저장 lineseg
vpos 라벨을 함께 내므로, 쪽별로 (렌더 y − vpos/75) 오프셋의 **중앙값 대비 편차**를 재면
쪽 기준선·여백·전역 오프셋이 소거되고 "같은 쪽 안에서 일부 블록만 밀린" 신호만 남는다.

함정 가드(실측 이력):
- 셀 안 TextLine 제외 — 셀 vpos 는 별도 좌표계(사다리 프로브 함정 '셀 pi 충돌')
- vpos 전부 0(합성 lineseg·HWPX 계산 경로) → 오프셋이 상수라 자연 침묵
- 쪽당 줄 4개 미만이면 중앙값이 무의미 → 스킵
- 같은 vpos 재사용(좌우분할·중복)은 개별 줄 판정이 아니라 문단 첫 줄만 대표로 씀

사용:
  python tools/verify_ladder_drift.py <file.hwp> [--exe rhwp.exe] [--threshold 15]
  python tools/verify_ladder_drift.py --list files.txt -o out.tsv [--exe ...]

출력(TSV): sample, verdict(OK/DRIFT/SKIP/ERR), pages_checked, worst_px, flagged_lines, detail
판정: 어느 쪽에서든 |편차| > threshold 인 줄이 있으면 DRIFT.
"""
from __future__ import annotations

import argparse
import re
import statistics
import subprocess
import sys
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8", errors="backslashreplace")

REPO = Path(__file__).resolve().parents[1]
LINE_RE = re.compile(
    r"^(\s*)TextLine\s+y=\s*([0-9.]+)\.\.\s*[0-9.]+\s+h=\s*([0-9.]+)\s.*?pi=(\d+)\s+line=(\d+)\s+vpos=(-?\d+)"
)
PAGE_RE = re.compile(r"^Page\s")
NODE_RE = re.compile(r"^(\s*)([A-Za-z]\w*)\s")
# 이 컨테이너 아래 줄은 본문 흐름 좌표계가 아니다 — 셀·글상자·도형·머리/꼬리말·각주.
FOREIGN = {
    "Table", "TableCell", "Header", "Footer", "FootnoteArea", "TextBox",
    "Shape", "Group", "Rect", "Ellipse", "Path", "Equation", "MasterPage", "Image",
}
HU_PER_PX = 75.0


def analyze(exe: Path, path: Path, threshold: float):
    proc = subprocess.run(
        [str(exe), "dump-extents", str(path)],
        capture_output=True,
        timeout=180,
        check=False,
    )
    if proc.returncode != 0:
        return "ERR", 0, 0.0, 0, proc.stderr.decode("utf-8", "replace")[:120]
    pages: list[list[tuple[float, int, int, int, int]]] = []
    stack: list[tuple[int, str, int]] = []  # (indent, node_type, col) — 조상 체인
    for raw in proc.stdout.decode("utf-8", "replace").splitlines():
        if PAGE_RE.match(raw):
            pages.append([])
            stack = []
            continue
        if not pages:
            continue
        node = NODE_RE.match(raw)
        if not node:
            continue
        indent, ntype = len(node.group(1)), node.group(2)
        while stack and stack[-1][0] >= indent:
            stack.pop()
        m = LINE_RE.match(raw)
        if m:
            # 조상에 외부 좌표계 컨테이너가 있으면 본문 흐름 줄이 아니다.
            if not any(t in FOREIGN for _, t, _c in stack):
                y, h = float(m.group(2)), float(m.group(3))
                pi, line, vpos = int(m.group(4)), int(m.group(5)), int(m.group(6))
                # 높이 0 의 빈 줄은 아무것도 안 그린다 — 표류해도 시각 무영향(위양성 실측:
                # 병무청 3143955 pi3, h=0 빈 줄이 이웃 위치로 접혀 +21.7px).
                if h > 0.5:
                    # 다단이면 단마다 vpos 가 리셋된다 — 중앙값은 (쪽, 단) 단위.
                    col = next((c for _, t, c in reversed(stack) if t == "Column"), 0)
                    pages[-1].append((y, pi, line, vpos, col))
        col = 0
        if ntype == "Column":
            col_m = re.search(r"col=(\d+)", raw)
            col = int(col_m.group(1)) if col_m else 0
        stack.append((indent, ntype, col))

    worst = 0.0
    flagged = 0
    checked = 0
    details = []
    for pno, rows in enumerate(pages, 1):
        cols: dict[int, dict[int, tuple[float, int]]] = {}
        # 문단 첫 줄만 대표로 — 같은 vpos 재사용(좌우분할·중복) 판정 오염 방지.
        for y, pi, line, vpos, col in rows:
            if line == 0 and pi not in cols.setdefault(col, {}):
                cols[col][pi] = (y, vpos)
        for col, first in cols.items():
            ordered = [(pi, y, v) for pi, (y, v) in sorted(first.items()) if v > 0]
            # vpos 되돌아감 = 쪽 중간 리베이스(구역/리스트 재시작) 신호 — 분절을 갈라
            # 각자의 중앙값으로 판정한다. 리베이스된 블록은 자기 기준으로 자기일관이라
            # 조용해지고, 같은 분절 안의 상대 이동(진짜 결함)만 남는다.
            segments = []
            cur = []
            prev_v = None
            for pi, y, v in ordered:
                if prev_v is not None and v < prev_v - 100:
                    segments.append(cur)
                    cur = []
                cur.append((pi, y, v))
                prev_v = v
            segments.append(cur)
            for seg in segments:
                if len(seg) < 4:
                    continue
                med = statistics.median(y - v / HU_PER_PX for _, y, v in seg)
                checked += 1
                for pi, y, v in seg:
                    dev = (y - v / HU_PER_PX) - med
                    if abs(dev) > worst:
                        worst = abs(dev)
                    if abs(dev) > threshold:
                        flagged += 1
                        if len(details) < 8:
                            details.append(f"p{pno}c{col} pi{pi} dev={dev:+.1f}px")
    verdict = "SKIP" if checked == 0 else ("DRIFT" if flagged else "OK")
    return verdict, checked, worst, flagged, " ; ".join(details)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("target", nargs="?", help="단일 파일")
    ap.add_argument("--list", help="파일 목록(줄당 1경로)")
    ap.add_argument("--exe", default=str(REPO / "target" / "debug" / "rhwp.exe"))
    ap.add_argument("--threshold", type=float, default=15.0)
    ap.add_argument("-o", "--out", help="TSV 출력 경로(--list 모드)")
    args = ap.parse_args()

    exe = Path(args.exe)
    if args.target:
        v = analyze(exe, Path(args.target), args.threshold)
        print(f"{Path(args.target).name}\t{v[0]}\tpages={v[1]}\tworst={v[2]:.1f}px\tflagged={v[3]}\t{v[4]}")
        return 0
    if not args.list:
        ap.error("target 또는 --list 필요")
    out = open(args.out, "w", encoding="utf-8", newline="\n") if args.out else sys.stdout
    out.write("sample\tverdict\tpages_checked\tworst_px\tflagged_lines\tdetail\n")
    n = 0
    for line in Path(args.list).read_text(encoding="utf-8").splitlines():
        p = line.strip()
        if not p:
            continue
        try:
            v = analyze(exe, Path(p), args.threshold)
        except subprocess.TimeoutExpired:
            v = ("ERR", 0, 0.0, 0, "timeout")
        out.write(f"{p}\t{v[0]}\t{v[1]}\t{v[2]:.1f}\t{v[3]}\t{v[4]}\n")
        n += 1
        if n % 100 == 0:
            out.flush()
            print(f"# 진행 {n}", file=sys.stderr)
    if args.out:
        out.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
