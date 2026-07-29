# -*- coding: utf-8 -*-
"""#2373 수정 10k 모집단 페이지 게이트 — r16 기록 한글 쪽수(COM 정답) 재사용, rhwp-only.

r16 results_final 에서 (sample, hwp_pages, r16_rhwp_pages) 를 읽어 새 exe 의
페이지 수와 대조. 개선/회귀/불변 집계를 출력한다.
사용: python pop_gate.py <exe> <out_tsv>
"""
import csv, subprocess, sys, re
from pathlib import Path

sys.stdout.reconfigure(encoding="utf-8")
ROOT = Path(r"C:\Users\planet\rhwp")
R16 = ROOT / "output/poc/survey10k_r16_20260719"
EXE = sys.argv[1] if len(sys.argv) > 1 else str(ROOT / "target/release/rhwp.exe")
OUT = Path(sys.argv[2]) if len(sys.argv) > 2 else R16.parent / "task2373" / "pop_gate.tsv"

PAT = re.compile(r"페이지 수:\s*(\d+)")

def load():
    best = {}
    PRIO = {"MATCH":3,"PI_MISMATCH":3,"PI_MISMATCH_CARET":3,"PAGE_DELTA":3,"PARA_COUNT":3,"PROTECTED_SKIP":3,"STALL":2,"ERR":1}
    for f in sorted((R16/"results_final").glob("chunk_*.tsv")):
        for r in csv.reader(open(f, encoding="utf-8"), delimiter="\t"):
            if not r or r[0].startswith("#") or r[0]=="sample": continue
            if r[0] not in best or PRIO.get(r[1],0) >= PRIO.get(best[r[0]][1],0): best[r[0]] = r
    return best

def main():
    r16 = load()
    paths = {}
    for l in open(R16/"sample10000.txt", encoding="utf-8"):
        l = l.strip()
        if l: paths[Path(l).name] = l
    done = set()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    if OUT.exists():
        for r in csv.reader(open(OUT, encoding="utf-8"), delimiter="\t"):
            if r: done.add(r[0])
    n = imp = reg = same_ok = same_bad = err = 0
    with open(OUT, "a", encoding="utf-8", newline="") as fh:
        w = csv.writer(fh, delimiter="\t")
        for s, r in sorted(r16.items()):
            if s in done or s not in paths: continue
            try:
                hp, rp16 = int(r[3]), int(r[2])
            except (ValueError, IndexError):
                continue  # ERR/PROTECTED 등
            try:
                out = subprocess.run([EXE, "info", paths[s]], capture_output=True, timeout=90)
                m = PAT.search(out.stdout.decode("utf-8", "replace"))
                rp = int(m.group(1)) if m else -1
            except Exception:
                rp = -1
            w.writerow([s, hp, rp16, rp]); fh.flush()
            n += 1
            if rp < 0: err += 1
            elif rp == hp and rp16 != hp: imp += 1
            elif rp != hp and rp16 == hp: reg += 1
            elif rp == hp: same_ok += 1
            else: same_bad += 1
            if n % 500 == 0:
                print(f"[{n}] imp={imp} reg={reg} same_ok={same_ok} same_bad={same_bad} err={err}", flush=True)
    print(f"DONE n={n} imp={imp} reg={reg} same_ok={same_ok} same_bad={same_bad} err={err}", flush=True)

if __name__ == "__main__":
    main()
