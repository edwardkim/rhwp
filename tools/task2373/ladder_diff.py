# -*- coding: utf-8 -*-
"""#2373 — 원본 vs 한글 재저장 HWPX 의 문단 vpos ladder 대조 (rhwp dump-pages 파서 사용).

각 문서에 대해 dump-pages 로 (pi → 첫 vpos[HWPUNIT]) 를 뽑고, 문단 스텝
(다음 body 문단 vpos − 현 문단 vpos, 음수=쪽 리셋 SKIP)을 원본/재저장 대조.
tac-host 발동 pi 의 스텝 차이(fresh − stored)가 host_px(≈font_size*75HU@96dpi)와
일치하면 '한글이 host 줄박스를 실제 가산하는 문서'(가산군).
"""
import os, re, subprocess, sys

sys.stdout.reconfigure(encoding="utf-8")
EXE = r"C:\Users\planet\rhwp\target\debug\rhwp.exe"
RES = r"C:\Users\planet\rhwp\output\poc\task2373\resaved"

VP_RE = re.compile(r"(FullParagraph|PartialParagraph|Table|PartialTable)\s+pi=(\d+)\b.*?vpos=(-?\d+)")


def ladder(path):
    out = subprocess.run([EXE, "dump-pages", path], capture_output=True, text=True,
                         encoding="utf-8", errors="replace", timeout=180).stdout
    lad = {}
    for m in VP_RE.finditer(out):
        pi, vp = int(m.group(2)), int(m.group(3))
        lad.setdefault(pi, vp)  # 문단 첫 등장만
    return lad


def firing_pis(path):
    env = dict(os.environ); env["RHWP_DIAG_TACHOST"] = "1"
    err = subprocess.run([EXE, "dump-pages", path], capture_output=True, text=True,
                         encoding="utf-8", errors="replace", timeout=180, env=env).stderr
    out = {}
    for m in re.finditer(r"DIAG_TACHOST pi=(\d+) host_px=([\d.]+)", err):
        out.setdefault(int(m.group(1)), float(m.group(2)))
    return out


def step(lad, pi):
    if pi not in lad:
        return None
    nxt = [q for q in lad if q > pi]
    if not nxt:
        return None
    q = min(nxt)
    s = lad[q] - lad[pi]
    return s if s > 0 else None


def main():
    for fn in sorted(os.listdir(RES)):
        dst = os.path.join(RES, fn)
        stem = fn[3:]
        # 원본 경로 복원
        src = None
        for root in (r"C:\Users\planet\hwpdocs\samples\노원소방서 현장대응단",
                     r"C:\Users\planet\hwpdocs\samples\미래공간기획관 도시활력담당관"):
            c = os.path.join(root, stem)
            if os.path.exists(c):
                src = c; break
        if src is None:
            with open(r"C:\Users\planet\rhwp\output\poc\survey10k_r16_20260719\sample10000.txt", encoding="utf-8") as fh:
                for l in fh:
                    l = l.strip()
                    if os.path.basename(l) == stem:
                        src = l; break
        if src is None:
            print(f"!! 원본 미발견 {stem}"); continue
        lo, lr = ladder(src), ladder(dst)
        fire = firing_pis(src)
        print(f"\n== {stem[:50]} (발동 {len(fire)}개 pi)")
        for pi, hpx in sorted(fire.items()):
            so, sr = step(lo, pi), step(lr, pi)
            if so is None or sr is None:
                print(f"   pi={pi} host_px={hpx:5.1f} step o={so} r={sr} SKIP")
                continue
            d = sr - so
            host_hu = hpx * 75  # 96dpi: 1px = 75 HWPUNIT
            verdict = "ADD" if abs(d - host_hu) < 150 else ("NONE" if abs(d) < 150 else "OTHER")
            print(f"   pi={pi} host_px={hpx:5.1f}({host_hu:6.0f}HU) step o={so:6d} r={sr:6d} d={d:6d} {verdict}")

if __name__ == "__main__":
    main()
