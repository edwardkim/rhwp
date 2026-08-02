"""Glyph-overlap fidelity: 같은 줄의 글자가 서로 겹쳐 못 읽게 되는가.

오라클 불필요(자기정합). 렌더 결과(SVG)만 본다 — 한글 COM 도, PDF 도 필요 없다.

기존 축이 못 보는 자리
  쪽수·PI      글자가 **어느 쪽**에 있나
  쪽 밖 글자    글자가 **사라졌나**
  텍스트 손실   글자가 **추출되나**
  텍스트 순서   글자 **순서**가 맞나
  → 넷 다 "제자리에 정상 간격으로 그려졌나" 는 안 본다. 겹친 글자는 쪽 안에 있고,
    추출되고, 순서도 맞다. 그런데 사람 눈에는 뭉개져 안 읽힌다.

측정
  SVG `<text>` 를 같은 baseline(y)끼리 묶고 x 순으로 정렬해 이웃 간 전진폭을 본다.
    OVERPRINT  전진폭 <= 0.05em   같은 자리에 겹쳐 찍힘 (사실상 판독 불가)
    CRUSH      전진폭 <  0.65em   한글 전각이 반각 아래로 뭉개짐
  em 은 그 글자의 font-size 다. 폭이 원래 좁은 글자(ASCII·문장부호)는 오탐이 되므로
  **양쪽 다 한글/전각일 때만** 센다.

사용:
  python scan_overlap.py <list.txt> <out.tsv> [--exe rhwp.exe] [--timeout N]
"""
import argparse
import csv
import glob
import io
import os
import re
import shutil
import subprocess
import sys
import tempfile
from collections import Counter

ap = argparse.ArgumentParser()
ap.add_argument("list")
ap.add_argument("out")
ap.add_argument("--exe", default=r"C:\Users\planet\rhwp\target\release\rhwp.exe")
ap.add_argument("--timeout", type=int, default=300)
a = ap.parse_args()
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

TEXT = re.compile(
    r'<text[^>]*?\bx="([-\d.]+)"[^>]*?\by="([-\d.]+)"[^>]*?\bfont-size="([\d.]+)"[^>]*?>([^<]*)</text>'
)
OVERPRINT = 0.05
# 0.65 는 실측 보정값이다 — 건전한 문서의 쪽별 최소 전진폭이 0.77~0.98em 이고,
# #2525 계열 압축 겹침은 0.6× 였다. 그 사이를 가른다.
CRUSH = 0.65
BASELINE_EPS = 0.6          # 같은 줄로 볼 y 차이(px)


def is_wide(s):
    """전각(한글 음절·한자)인가 — 폭이 1em 에 가까운 글자만 판정 대상으로 삼는다.

    **자모는 뺀다(실측 보정).** 결합 자모(U+1100~11FF)는 한 음절로 합쳐지는 조합
    문자라 설계상 같은 자리에 겹쳐 그려지고, 호환 자모(U+3130~318F)는 글머리표로
    쓰여 전각 가정이 안 맞는다. 넣어 두면 CRUSH 오탐이 문서 38건 규모로 섞인다
    (실측: 오탐 쌍이 전부 '호환자모 -> 음절' 이었다).
    """
    if len(s) != 1:
        return False
    c = ord(s)
    return (0xAC00 <= c <= 0xD7A3      # 한글 음절
            or 0x4E00 <= c <= 0x9FFF)  # 한자


def scan_svg(text):
    rows = []
    for x, y, fs, s in TEXT.findall(text):
        if not s.strip():
            continue
        rows.append((float(y), float(x), float(fs), s))
    lines = {}
    for y, x, fs, s in rows:
        key = round(y / BASELINE_EPS)
        lines.setdefault(key, []).append((x, fs, s))
    over = crush = pairs = 0
    worst = 1e9
    for key, items in lines.items():
        items.sort()
        for (x0, f0, s0), (x1, f1, s1) in zip(items, items[1:]):
            if not (is_wide(s0) and is_wide(s1)):
                continue
            em = max(f0, 1e-6)
            adv = (x1 - x0) / em
            pairs += 1
            if adv <= OVERPRINT:
                over += 1
            elif adv < CRUSH:
                crush += 1
            worst = min(worst, adv)
    return pairs, over, crush, (None if worst > 1e8 else round(worst, 3))


def scan(src):
    work = tempfile.mkdtemp(prefix="govl_")
    try:
        d = os.path.join(work, "svg")
        subprocess.run([a.exe, "export-svg", src, "-o", d],
                       capture_output=True, timeout=a.timeout)
        files = sorted(glob.glob(d + r"\*.svg"))
        if not files:
            return {"status": "NOSVG"}
        tp = to = tc = 0
        wmin = 1e9
        wpage = -1
        for i, f in enumerate(files, 1):
            try:
                t = io.open(f, encoding="utf-8", errors="replace").read()
            except OSError:
                continue
            p, o, c, w = scan_svg(t)
            tp += p; to += o; tc += c
            if w is not None and w < wmin:
                wmin, wpage = w, i
        if tp == 0:
            return {"status": "NOCMP"}
        return {"status": "OK", "pages": len(files), "pairs": tp,
                "overprint": to, "crush": tc,
                "rate": round((to + tc) / tp, 5),
                "worst_adv_em": None if wmin > 1e8 else wmin, "worst_page": wpage}
    except subprocess.TimeoutExpired:
        return {"status": "TIMEOUT"}
    finally:
        shutil.rmtree(work, ignore_errors=True)


docs = [l.strip() for l in io.open(a.list, encoding="utf-8", errors="replace") if l.strip()]
print(f"docs: {len(docs)}", flush=True)
cols = ["sample", "status", "pages", "pairs", "overprint", "crush", "rate",
        "worst_adv_em", "worst_page"]
hits = 0
with io.open(a.out, "w", encoding="utf-8", newline="") as fh:
    w = csv.DictWriter(fh, fieldnames=cols, delimiter="\t", extrasaction="ignore")
    w.writeheader()
    for i, src in enumerate(docs, 1):
        r = scan(src)
        r["sample"] = os.path.basename(src)
        w.writerow(r)
        if r.get("overprint") or r.get("crush"):
            hits += 1
        if i % 20 == 0:
            print(f"  {i}/{len(docs)}  겹침 있는 문서 {hits}", flush=True)
            fh.flush()
print(f"겹침 있는 문서 {hits}/{len(docs)} — 기록: {a.out}")
