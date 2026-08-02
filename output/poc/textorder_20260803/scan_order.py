"""Text-order fidelity: does the PDF read back in document flow order?

Oracle-free by construction, same shape as `textfidelity_20260723/scan.py`:

  A = rhwp `export-text`   페이지별 렌더 줄 = 문서 흐름 순서 (파서 진실)
  B = rhwp `export-pdf` 의 페이지별 텍스트 = 콘텐츠 스트림 순서 (출력이 내놓는 순서)

같은 쪽 안에서 두 순서가 어긋나면 복사·붙여넣기, 검색 문맥, 스크린리더 읽기가
전부 뒤엉킨다. 픽셀·PI 오라클은 이 결함을 원리적으로 못 본다 — 글자는 제자리에
그려지기 때문이다.

측정
  쪽마다 양쪽에서 **정확히 한 번씩만** 나오는 줄을 뽑아(중복 줄·머리말·꼬리말 자동 제외)
  A 의 순서를 기준으로 B 의 순위 수열을 만들고 **역전쌍 수**를 센다.
  disorder = 역전쌍 / 최대역전쌍(n*(n-1)/2)  → 0 이면 완전 일치, 1 이면 완전 역순.

사용:
  python scan_order.py <list.txt> <out.tsv> [--exe rhwp.exe] [--timeout N]
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

ap = argparse.ArgumentParser()
ap.add_argument("list")
ap.add_argument("out")
ap.add_argument("--exe", default=r"C:\Users\planet\rhwp\target\release\rhwp.exe")
ap.add_argument("--timeout", type=int, default=300)
a = ap.parse_args()
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

try:
    import fitz
except ImportError:
    print("pymupdf required")
    sys.exit(1)

# 줄을 비교 가능한 열쇠로 — 공백·문장부호를 걷어낸다(추출 단계에서 흔들리는 부분).
STRIP = re.compile(r"[\s\u00a0\u2007\u200b·.,()\[\]{}<>「」『』\"'“”‘’…·ㆍ:;/\\|-]+")
MIN_KEY = 10          # 이보다 짧은 줄은 우연 일치가 잦아 버린다
MIN_PAIRS = 4         # 비교 가능한 줄이 이보다 적은 쪽은 판정하지 않는다


def keys_of(lines):
    """정규화한 줄 -> 등장 횟수"""
    out = {}
    for ln in lines:
        k = STRIP.sub("", ln)
        if len(k) < MIN_KEY:
            continue
        out[k] = out.get(k, 0) + 1
    return out


def ir_pages(src, work):
    d = os.path.join(work, "txt")
    subprocess.run([a.exe, "export-text", src, "-o", d],
                   capture_output=True, timeout=a.timeout)
    files = sorted(glob.glob(d + r"\**\*.txt", recursive=True))
    pages = []
    for f in files:
        try:
            pages.append(io.open(f, encoding="utf-8", errors="replace").read().split("\n"))
        except OSError:
            pages.append([])
    return pages


def pdf_pages(src, work):
    p = os.path.join(work, "o.pdf")
    subprocess.run([a.exe, "export-pdf", src, "-o", p],
                   capture_output=True, timeout=a.timeout)
    if not os.path.exists(p):
        return None
    try:
        d = fitz.open(p)
        # sort=False: 콘텐츠 스트림 순서 그대로 — 뷰어가 복사할 때 보는 순서다.
        pages = [d[i].get_text("text").split("\n") for i in range(len(d))]
        d.close()
        return pages
    except Exception:
        return None


def inversions(seq):
    """O(n^2) 로 충분하다 — 쪽당 줄 수는 수십 규모."""
    n = len(seq)
    return sum(1 for i in range(n) for j in range(i + 1, n) if seq[i] > seq[j])


def scan(src):
    work = tempfile.mkdtemp(prefix="tord_")
    try:
        ir = ir_pages(src, work)
        pdf = pdf_pages(src, work)
        if pdf is None:
            return {"status": "NOPDF"}
        if not ir:
            return {"status": "NOIR"}
        pages = min(len(ir), len(pdf))
        tot_pairs = tot_inv = 0
        worst = (0.0, -1, 0)
        for i in range(pages):
            ka, kb = keys_of(ir[i]), keys_of(pdf[i])
            # 양쪽에서 정확히 한 번씩만 나오는 줄만 쓴다
            common = [k for k in ka if ka[k] == 1 and kb.get(k) == 1]
            if len(common) < MIN_PAIRS:
                continue
            order_a = {k: n for n, k in enumerate(
                [ln for ln in (STRIP.sub("", x) for x in ir[i]) if ln in set(common)])}
            seq_b = [order_a[k] for k in
                     (STRIP.sub("", x) for x in pdf[i]) if k in order_a]
            if len(seq_b) < MIN_PAIRS:
                continue
            inv = inversions(seq_b)
            mx = len(seq_b) * (len(seq_b) - 1) // 2
            tot_pairs += mx
            tot_inv += inv
            if mx and inv / mx > worst[0]:
                worst = (inv / mx, i + 1, len(seq_b))
        if tot_pairs == 0:
            return {"status": "NOCMP"}
        return {"status": "OK", "pages": pages, "cmp_pairs": tot_pairs,
                "inversions": tot_inv,
                "disorder": round(tot_inv / tot_pairs, 4),
                "worst_page": worst[1], "worst_disorder": round(worst[0], 4),
                "worst_lines": worst[2]}
    except subprocess.TimeoutExpired:
        return {"status": "TIMEOUT"}
    finally:
        shutil.rmtree(work, ignore_errors=True)


docs = [l.strip() for l in io.open(a.list, encoding="utf-8", errors="replace") if l.strip()]
print(f"docs: {len(docs)}", flush=True)
cols = ["sample", "status", "pages", "cmp_pairs", "inversions", "disorder",
        "worst_page", "worst_disorder", "worst_lines"]
hits = 0
with io.open(a.out, "w", encoding="utf-8", newline="") as fh:
    w = csv.DictWriter(fh, fieldnames=cols, delimiter="\t", extrasaction="ignore")
    w.writeheader()
    for i, src in enumerate(docs, 1):
        r = scan(src)
        r["sample"] = os.path.basename(src)
        w.writerow(r)
        if r.get("inversions"):
            hits += 1
        if i % 20 == 0:
            print(f"  {i}/{len(docs)}  순서 어긋난 문서 {hits}", flush=True)
            fh.flush()
print(f"순서 어긋난 문서 {hits}/{len(docs)} — 기록: {a.out}")
