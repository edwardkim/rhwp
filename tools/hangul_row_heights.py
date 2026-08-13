"""#1658/#2148 한글 표 행높이 추출(COM) + rhwp 행높이 대조.

cut↔render↔한글 fidelity 의 **한글 행높이 기준**을 COM 으로 직접 추출한다(PDF 배율 차단 우회).

방법(2026-08-13 Windows 실측으로 재설계):
  1. `rhwp info` 로 표 카탈로그(표N [구역S:문단P]: R행×C열)를 얻는다 — pi/행수/열수의 출처.
  2. `RHWP_TABLE_DRIFT=1 rhwp dump-pages` 의 `TABLE_DRIFT: pi=N ... mt_row_heights=[...]`
     로 rhwp per-row 높이(px)를 얻는다.
  3. 한글은 HeadCtrl 순회로 **앵커가 (List=0, Para=pi)** 인 tbl 을 고르고,
     `select_ctrl` 로 진입한 뒤 `goto_addr("A1".."<열><행>")` 로 전 셀을 방문해
     행별 최소 높이를 그 행의 높이로 복원한다.

왜 이렇게 바뀌었나 — 옛 구현이 깨진 세 지점:
  * `SetPosBySet+FindCtrl` 진입: 표 앵커와 같은 문단에 누름틀(%clk)·그림(gso)이 있으면
    FindCtrl 이 그쪽을 잡아 표 진입이 실패한다(36404953). `select_ctrl` 은 컨트롤 객체를
    직접 선택해 우회한다. pyhwpx 의 get_into_nth_table 도 옛 경로라 같이 깨진다.
  * `cut_rows=[...]` 침묵 폴백: `--pi` 가 아무것도 못 맞히면 조용히 **엉뚱한 표**의 옛
    표기로 폴백해 그럴듯한 가짜 대조표를 찍었다(76076 `--pi 10` → 표5 31행). 이제 죽는다.
  * `TableLowerCell` 행 순회: 세로 병합 셀을 한 행으로 세어 병합 구간 합이 한 행으로
    나온다(21761835: 한글 31스텝 vs 실제 78행). goto_addr 스윕이 이를 없앤다.

두 좌표계 주의: `--table-index` 는 한글 HeadCtrl 순번, `--pi` 는 rhwp 문단 인덱스다.
**`--pi` 를 쓰면 양쪽이 같은 좌표로 선택되므로 항상 `--pi` 를 권한다.**

사용:
  python tools/hangul_row_heights.py <file> --exe <rhwp> --pi <문단인덱스>
  python tools/hangul_row_heights.py <file> --exe <rhwp> --all   # 측정가능 표 전수
요구: Windows + 한컴 + pyhwpx. rhwp release 바이너리(--exe).
"""
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path

MM_TO_PX = 96.0 / 25.4
TOL_PX = 0.5

INFO_RE = re.compile(r"표(\d+) \[구역(\d+):문단(\d+)\]: (\d+)행×(\d+)열, 셀 (\d+)개")
DRIFT_RE = re.compile(r"TABLE_DRIFT: pi=(\d+) .*?mt_row_heights=\[([^\]]*)\]")


def _run(cmd: list[str], env: dict | None = None) -> str:
    r = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8",
                       errors="replace", env=env, timeout=1800)
    return r.stdout + r.stderr


def table_catalog(src: Path, exe: str) -> dict[int, dict]:
    """pi -> {no, sec, rows, cols, cells}. `rhwp info` 의 표 목록."""
    out = _run([exe, "info", str(src)])
    cat = {}
    for m in INFO_RE.finditer(out):
        no, sec, pi, rows, cols, cells = (int(x) for x in m.groups())
        cat.setdefault(pi, dict(no=no, sec=sec, rows=rows, cols=cols, cells=cells))
    return cat


def rhwp_row_heights(src: Path, exe: str) -> dict[int, list[float]]:
    """pi -> rhwp per-row 높이(px). 같은 pi 가 여러 번이면 첫 줄(전체 표)."""
    env = dict(os.environ, RHWP_TABLE_DRIFT="1")
    out = _run([exe, "dump-pages", str(src)], env=env)
    got: dict[int, list[float]] = {}
    for m in DRIFT_RE.finditer(out):
        got.setdefault(int(m.group(1)),
                       [float(x) for x in m.group(2).split(",") if x.strip()])
    return got


def _colname(c: int) -> str:
    s = ""
    c += 1
    while c:
        c, rem = divmod(c - 1, 26)
        s = chr(65 + rem) + s
    return s


class Hangul:
    """한컴 COM 세션 — 문서 하나를 열고 표 여러 개를 훑는다."""

    def __init__(self, src: Path):
        import subprocess as sp

        from pyhwpx import Hwp

        sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)
        self._sp = sp
        self.hwp = Hwp(new=True, visible=False)
        self.hwp.open(str(src))

    def close(self):
        # 스윕 중 한글이 죽으면(RPC 서버 사용 불가) clear/quit 도 같이 던진다 — 삼킨다.
        for fn in (lambda: self.hwp.clear(option=1), self.hwp.quit):
            try:
                fn()
            except Exception:
                pass
        self._sp.run(["taskkill", "/F", "/IM", "Hwp.exe"], capture_output=True)

    def find_table(self, pi: int | None, table_index: int):
        """pi 지정 시 앵커(List=0,Para=pi) 로, 아니면 HeadCtrl 순번으로 tbl 을 고른다."""
        ctrl, seen = self.hwp.HeadCtrl, 0
        while ctrl is not None:
            if ctrl.CtrlID == "tbl":
                if pi is not None:
                    ps = ctrl.GetAnchorPos(0)
                    if ps.Item("List") == 0 and ps.Item("Para") == pi:
                        return ctrl
                else:
                    if seen == table_index:
                        return ctrl
                    seen += 1
            ctrl = ctrl.Next
        return None

    def row_heights(self, tbl, nrows: int, ncols: int) -> tuple[dict[int, float], int]:
        """goto_addr 전 셀 스윕 → {행: 높이px}, 방문 셀 수.

        세로 병합 셀은 여러 행에 같은 높이를 주므로 행별 **최소**를 취한다
        (rowspan=1 셀이 그 행에 하나라도 있으면 정확). 검증은 호출부의 합 대조.
        """
        hwp = self.hwp
        if not (hwp.select_ctrl(tbl) and hwp.ShapeObjTableSelCell()):
            return {}, 0
        hwp.Cancel()
        seen: dict[int, float] = {}
        visits = 0
        for c in range(ncols):
            for rw in range(nrows):
                try:
                    if not hwp.goto_addr(f"{_colname(c)}{rw + 1}"):
                        continue
                    m = re.match(r"([A-Z]+)([0-9]+)$", str(hwp.get_cell_addr()))
                    if not m:
                        continue
                    h = float(hwp.get_row_height()) * MM_TO_PX
                except Exception as e:
                    # 한글이 스윕 중 죽는 문서가 있다(21761835 사선 셀). 부분 결과를
                    # 살려 반환하고, 죽었다는 사실은 호출부가 복원율로 드러낸다.
                    self.dead = True
                    print(f"  [경고] {_colname(c)}{rw + 1} 에서 COM 실패: "
                          f"{type(e).__name__} — 스윕 중단", file=sys.stderr)
                    return seen, visits
                row = int(m.group(2)) - 1
                visits += 1
                if row not in seen or h < seen[row]:
                    seen[row] = h
        return seen, visits


def report(pi: int, meta: dict, hg: dict[int, float], visits: int,
           rh: list[float]) -> float:
    n = len(rh)
    missing = [i for i in range(n) if i not in hg]
    tot_h = sum(hg.values())
    tot_r = sum(rh)
    print(f"\n=== 표{meta['no']} [구역{meta['sec']}:문단{pi}] "
          f"{meta['rows']}행×{meta['cols']}열, 셀 {meta['cells']}개 ===")
    print(f"방문 셀 {visits} (info 셀수 {meta['cells']}), 복원 행 {len(hg)}/{n}"
          + (f", 미복원 {missing[:10]}" if missing else ""))
    print(f"합계  한글 {tot_h:.2f}px   rhwp {tot_r:.2f}px   차 {tot_r - tot_h:+.2f}px")
    bad = [i for i in sorted(hg) if i < n and abs(rh[i] - hg[i]) > TOL_PX]
    print(f"행별 |diff|>{TOL_PX}px : {len(bad)}/{len(hg)}")
    if bad:
        print(f"{'row':>4} {'한글_px':>9} {'rhwp_px':>9} {'diff':>8}")
        for i in bad:
            print(f"{i:>4} {hg[i]:>9.2f} {rh[i]:>9.2f} {rh[i] - hg[i]:>+8.2f}")
    return tot_r - tot_h


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("src", type=Path)
    ap.add_argument("--exe", default="target/release/rhwp.exe"
                    if sys.platform == "win32" else "target/release/rhwp")
    ap.add_argument("--table-index", type=int, default=0,
                    help="한글 HeadCtrl 순번(--pi 없을 때만)")
    ap.add_argument("--pi", type=int, default=None, help="rhwp 문단 인덱스(권장)")
    ap.add_argument("--all", action="store_true",
                    help="TABLE_DRIFT 가 있는 최상위 표 전수 대조")
    ap.add_argument("--full", action="store_true", help="일치 행까지 전부 출력")
    a = ap.parse_args()

    cat = table_catalog(a.src, a.exe)
    drift = rhwp_row_heights(a.src, a.exe)
    print(f"info 표 {len(cat)}개, TABLE_DRIFT 표 {len(drift)}개")

    if a.all:
        targets = sorted(set(cat) & set(drift))
        skipped = sorted(set(cat) - set(drift))
        print(f"대상 {len(targets)}개, TABLE_DRIFT 없어 제외 {len(skipped)}개: {skipped[:20]}")
    else:
        if a.pi is None:
            print("경고: --pi 없이 --table-index 로 고릅니다. 두 좌표계가 달라 "
                  "엉뚱한 표를 볼 수 있습니다.", file=sys.stderr)
        pi = a.pi
        if pi is not None and pi not in drift:
            print(f"에러: pi={pi} 의 TABLE_DRIFT 가 없습니다. 이 표는 이 진단으로 "
                  f"측정할 수 없습니다. 사용 가능한 pi: {sorted(drift)[:30]}", file=sys.stderr)
            return 2
        targets = [pi] if pi is not None else []

    hg_session = Hangul(a.src)
    try:
        if not targets:  # --table-index 경로
            tbl = hg_session.find_table(None, a.table_index)
            if tbl is None:
                print("한글 표 못 찾음", file=sys.stderr)
                return 2
            ps = tbl.GetAnchorPos(0)
            pi = ps.Item("Para") if ps.Item("List") == 0 else None
            if pi is None or pi not in drift:
                print(f"에러: HeadCtrl #{a.table_index} 표의 앵커 pi={pi} 에 "
                      f"TABLE_DRIFT 가 없습니다.", file=sys.stderr)
                return 2
            targets = [pi]

        worst = []
        for pi in targets:
            meta = cat.get(pi)
            rh = drift[pi]
            if meta is None:
                print(f"\n=== 문단{pi}: info 카탈로그에 없음(중첩 표 추정) — 건너뜀 ===")
                continue
            if meta["rows"] != len(rh):
                print(f"\n=== 표{meta['no']} [문단{pi}] 행수 불일치: "
                      f"info {meta['rows']}행 vs drift {len(rh)}행 — 건너뜀"
                      f"(래퍼/중첩 표로 pi 충돌) ===")
                continue
            tbl = hg_session.find_table(pi, 0)
            if tbl is None:
                print(f"\n=== 문단{pi}: 한글 쪽 표 못 찾음 — 건너뜀 ===")
                continue
            hg, visits = hg_session.row_heights(tbl, meta["rows"], meta["cols"])
            if not hg:
                print(f"\n=== 문단{pi}: 한글 표 진입 실패 — 건너뜀 ===")
                continue
            if a.full:
                print(f"\n--- 표{meta['no']} [문단{pi}] 전체 ---")
                for i in range(len(rh)):
                    h = hg.get(i)
                    print(f"{i:>4} {(f'{h:.2f}' if h else '-'):>9} {rh[i]:>9.2f} "
                          f"{(f'{rh[i] - h:+.2f}' if h else '-'):>8}")
            worst.append((abs(report(pi, meta, hg, visits, rh)), pi))

        if a.all and worst:
            worst.sort(reverse=True)
            print(f"\n=== 표 총합 |차| 상위 ===")
            for d, pi in worst[:15]:
                print(f"  문단{pi}: {d:.2f}px")
            print(f"측정 표 {len(worst)}개, |차|>1px 인 표 "
                  f"{sum(1 for d, _ in worst if d > 1.0)}개")
    finally:
        hg_session.close()
    return 0


if __name__ == "__main__":
    sys.exit(main())
