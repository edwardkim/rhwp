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
NODE_RE = re.compile(r"^(\s*)(\S+)\s")
# 이 컨테이너 아래 줄은 본문 흐름 좌표계가 아니다 — 셀·글상자·도형·머리/꼬리말·각주.
FOREIGN = {
    "Table", "TableCell", "Header", "Footer", "FootnoteArea", "TextBox",
    "Shape", "Group", "Rect", "Ellipse", "Path", "Equation", "MasterPage", "Image",
    # dump-extents 는 무명 컨테이너(도형·캡션 등)를 "기타" 로 찍는다 — 옛 NODE_RE
    # ([A-Za-z]만)가 이를 스택에서 투명하게 만들어 글상자 내부 줄이 본문으로
    # 샜다(누리과정 7240000 FP 실측). 노드 토큰을 \S+ 로 넓히고 여기서 거른다.
    "기타",
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
            # 조우(렌더 트리) 순서를 유지한다 — pi 로 정렬하면 컬럼 직속으로 샌
            # 지역-사다리 줄(글상자 pi0, vpos 500대)이 맨 앞으로 끌려와 되돌아감
            # 분할이 무력화되고 본문 중앙값을 오염시킨다(환경위성 156489219 FP).
            # 조우순이면 그런 줄은 vpos 되돌아감으로 갈라져 4줄 미만 분절 스킵에
            # 걸린다.
            ordered = [(pi, y, v) for pi, (y, v) in first.items() if v > 0]
            # 샌드위치 외래 줄 드롭: pi 가 역행했는데 **나중에 원 수열이 재개**
            # 되면(같은 쪽에서 running-max 이상 pi 재등장) 그 역행 줄은 본문이
            # 아니라 컬럼 직속으로 샌 글상자/도형 줄이다(세종 p20: ..pi382,
            # pi0(글상자), pi383.. 실측). 구역 재시작은 복귀가 없어 보존된다.
            if ordered:
                runmax = []
                m = -1
                for pi, _, _ in ordered:
                    runmax.append(m)
                    m = max(m, pi)
                later_max = [0] * len(ordered)
                m = -1
                for i in range(len(ordered) - 1, -1, -1):
                    later_max[i] = m
                    m = max(m, ordered[i][0])
                ordered = [
                    row
                    for i, row in enumerate(ordered)
                    if not (row[0] < runmax[i] and later_max[i] >= runmax[i])
                ]
            # vpos 되돌아감 = 쪽 중간 리베이스(구역/리스트 재시작) 신호 — 분절을 갈라
            # 각자의 중앙값으로 판정한다. 리베이스된 블록은 자기 기준으로 자기일관이라
            # 조용해지고, 같은 분절 안의 상대 이동(진짜 결함)만 남는다.
            segments = []
            cur = []
            prev_v = None
            for pi, y, v in ordered:
                # 되돌아감 = 리베이스 신호(기존). **큰 전방 점프**(>30000u=400px)
                # 도 분절을 끊는다 — 컬럼 직속으로 샌 글상자 줄(로컬 vpos)이
                # 되돌아감으로 갈라진 뒤 다음 본문 세그먼트에 합류해 +741px 로
                # 오검출되는 샌드위치 형상(세종 5690000-202000006 p20 실측).
                # 정상 문서의 거대 개체 줄 뒤 갭도 같은 기준으로 갈라지지만,
                # 각 분절이 자기 중앙값으로 판정되므로 위양성이 늘지 않는다.
                if prev_v is not None and (v < prev_v - 100 or v > prev_v + 30_000):
                    segments.append(cur)
                    cur = []
                cur.append((pi, y, v))
                prev_v = v
            segments.append(cur)
            for seg in segments:
                if len(seg) < 4:
                    continue
                med = statistics.median(y - v / HU_PER_PX for _, y, v in seg)
                # 귀속 교정: 결함이 분절 중후반의 다수 줄을 밀면 중앙값이 밀린
                # 편에 붙어 **정위치 줄이 편차로 오지목**된다(기장군·아세안·
                # 2135039 등 5사례). 분절 머리(첫 3줄)는 결함 지점 위에서 쪽
                # 원점을 세우므로, 머리 기준과 중앙값이 갈리면 머리를 기준으로
                # 삼는다 — 판정(DRIFT 여부)은 불변, 어느 줄을 짚는지만 바뀐다.
                head = statistics.median(y - v / HU_PER_PX for _, y, v in seg[:3])
                if abs(head - med) > threshold:
                    med = head
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
    ap.add_argument("--jobs", type=int, default=1, help="병렬 프로세스 수(--list 모드)")
    args = ap.parse_args()

    exe = Path(args.exe)
    if args.target:
        v = analyze(exe, Path(args.target), args.threshold)
        print(f"{Path(args.target).name}\t{v[0]}\tpages={v[1]}\tworst={v[2]:.1f}px\tflagged={v[3]}\t{v[4]}")
        return 0
    if not args.list:
        ap.error("target 또는 --list 필요")
    import concurrent.futures as cf

    paths = [l.strip() for l in Path(args.list).read_text(encoding="utf-8").splitlines() if l.strip()]

    def one(p: str):
        try:
            return p, analyze(exe, Path(p), args.threshold)
        except subprocess.TimeoutExpired:
            return p, ("ERR", 0, 0.0, 0, "timeout")
        except Exception as e:  # noqa: BLE001 — 한 파일이 스윕을 못 죽이게
            return p, ("ERR", 0, 0.0, 0, str(e)[:120])

    out = open(args.out, "w", encoding="utf-8", newline="\n") if args.out else sys.stdout
    out.write("sample\tverdict\tpages_checked\tworst_px\tflagged_lines\tdetail\n")
    n = 0
    with cf.ThreadPoolExecutor(max_workers=max(1, args.jobs)) as ex:
        for p, v in ex.map(one, paths):
            out.write(f"{p}\t{v[0]}\t{v[1]}\t{v[2]:.1f}\t{v[3]}\t{v[4]}\n")
            n += 1
            if n % 100 == 0:
                out.flush()
                print(f"# 진행 {n}/{len(paths)}", file=sys.stderr)
    if args.out:
        out.close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
