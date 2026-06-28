"""렌더링 페이지 정합 게이트 (Task #1600 통제셋).
tests/fixtures/render_page_controlset.tsv 의 각 문서에 대해 현재 rhwp 페이지수(`rhwp info`)를
한글 정답지(hangul_pages)와 비교. 수정 전/후 실행해 −1쪽 해소 − 회귀 > 0 를 판정.

사용: python tools/render_page_gate.py [--root C:/Users/planet/hwpdocs] [--exe ./target/release/rhwp.exe]
      [--baseline out.tsv 저장]
"""
import csv, sys, subprocess, re, os, argparse
from collections import Counter

ap = argparse.ArgumentParser()
ap.add_argument("--root", default="C:/Users/planet/hwpdocs")
ap.add_argument("--exe", default="./target/release/rhwp.exe")
ap.add_argument("--fixture", default="tests/fixtures/render_page_controlset.tsv")
ap.add_argument("--save", default=None)
a = ap.parse_args()

pat = re.compile(r"페이지 수:\s*(\d+)")
rows = list(csv.DictReader(open(a.fixture, encoding="utf-8"), delimiter="\t"))

conf = Counter()      # delta(rhwp-hangul) → count
results = []
for r in rows:
    rel = r["rel"]; hangul = int(r["hangul_pages"])
    p = os.path.join(a.root, rel.replace("/", "\\"))
    try:
        out = subprocess.run([a.exe, "info", p], capture_output=True, timeout=60)
        m = pat.search(out.stdout.decode("utf-8", "replace"))
        rp = int(m.group(1)) if m else -1
    except Exception:
        rp = -1
    d = rp - hangul if rp > 0 else None
    conf[d] += 1
    results.append((rel, hangul, rp, d))

total = sum(1 for _, _, rp, _ in results if rp > 0)
match = conf.get(0, 0)
print(f"=== 렌더링 페이지 게이트 ({total} 측정) ===")
print(f"일치(rhwp==한글): {match} ({100.0*match/total:.1f}%)")
print("delta(rhwp-한글) 분포:")
for d in sorted(k for k in conf if k is not None):
    tag = "일치" if d == 0 else ("부족(-)" if d < 0 else "초과(+)")
    print(f"  {d:+d}: {conf[d]}  {tag}")
if conf.get(None):
    print(f"  ERR(info 실패): {conf[None]}")

if a.save:
    with open(a.save, "w", encoding="utf-8") as f:
        f.write("rel\thangul\trhwp\tdelta\n")
        for rel, h, rp, d in results:
            f.write(f"{rel}\t{h}\t{rp}\t{d if d is not None else 'ERR'}\n")
    print(f"저장: {a.save}")
