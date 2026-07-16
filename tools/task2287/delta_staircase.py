"""#2287 단일 문서 per-PI 델타 계단 — rhwp vs 한글 COM.

표 밀집/그림 스택 문서의 과소분할 이탈 지점 특정용. 각 본문 PI 의 rhwp 배치 쪽과
한글(SetPos 캐럿) 쪽을 전부 대조하고, delta(rhwp−hwp)가 변하는 지점(계단)만 출력한다.
verify_pi_page_vs_hangul.py 가 파일 단위 판정만 주는 것과 달리, 문서 안에서 어느
문단/표에서 몇 쪽이 벌어지는지 위치를 준다.

사용:
    python tools/task2287/delta_staircase.py <doc.hwp> [--exe <rhwp.exe>]

요구: Windows + 한컴오피스 + pyhwpx. 캐럿-개체 분리 오탐(#1757/#1920)은 그대로
노출되므로 HiddenEmptyPara/표 anchor 의 ±수십 스텝은 시각 확인으로 걸러낸다.
"""
import argparse
import re
import subprocess
import sys
from pathlib import Path

PG = re.compile(r"=== 페이지 (\d+) \(global_idx=\d+, section=(\d+),")
PI = re.compile(r"\bpi=(\d+)")


def rhwp_pi_pages(exe: str, path: str):
    out = subprocess.run([exe, "dump-pages", path], capture_output=True,
                         text=True, encoding="utf-8", errors="replace", timeout=600)
    start = {}
    kinds = {}
    cur_page = 0
    cur_sec = 0
    max_pi = {}
    total = 0
    for ln in out.stdout.splitlines():
        m = PG.search(ln)
        if m:
            cur_page = int(m.group(1))
            cur_sec = int(m.group(2))
            total = max(total, cur_page)
            continue
        if "[미주]" in ln:
            continue
        q = PI.search(ln)
        if q and cur_page:
            pi = int(q.group(1))
            key = (cur_sec, pi)
            if key not in start or cur_page < start[key]:
                start[key] = cur_page
                kinds[key] = ln.lstrip().split()[0]
            max_pi[cur_sec] = max(max_pi.get(cur_sec, 0), pi)
    sec_counts = {s: max_pi[s] + 1 for s in max_pi}
    offsets = {}
    acc = 0
    for s in sorted(sec_counts):
        offsets[s] = acc
        acc += sec_counts[s]
    abs_map = {}
    kind_map = {}
    for (s, pi), pg in start.items():
        abs_map[offsets.get(s, 0) + pi] = pg
        kind_map[offsets.get(s, 0) + pi] = kinds[(s, pi)]
    return abs_map, kind_map, total


def hwp_pi_pages(path: str):
    from pyhwpx import Hwp
    hwp = Hwp(visible=False)
    try:
        hwp.open(path)
        total = hwp.PageCount
        hwp.MoveDocEnd()
        end = hwp.GetPos()
        mp = {}
        for para in range(end[1] + 1):
            hwp.SetPos(0, para, 0)
            mp[para] = hwp.current_page
        hwp.clear(option=1)
    finally:
        hwp.quit()
    return mp, total


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("doc")
    ap.add_argument("--exe", default=str(Path(__file__).resolve().parents[2]
                                         / "target" / "debug" / "rhwp.exe"))
    args = ap.parse_args()

    rmap, kmap, rtotal = rhwp_pi_pages(args.exe, args.doc)
    hmap, htotal = hwp_pi_pages(args.doc)
    print(f"rhwp_pages={rtotal} hwp_pages={htotal} delta={rtotal - htotal}")
    prev_delta = 0
    for idx in sorted(set(rmap) & set(hmap)):
        rp, hp = rmap[idx], hmap[idx]
        d = rp - hp
        if d != prev_delta:
            print(f"pi={idx:5d} rhwp_p{rp:4d} hwp_p{hp:4d} delta={d:+3d} "
                  f"(step {d - prev_delta:+d}) kind={kmap.get(idx, '?')}")
            prev_delta = d
    sys.stdout.flush()


if __name__ == "__main__":
    main()
