# -*- coding: utf-8 -*-
"""#2430 실측 TSV → font_metrics_data.rs LATIN_0 테이블 생성.

ladder_<face>.tsv (adv_em, 14pt 실측) → em1024 정수 배열 (0x20..0x7E, 95개).
미측정 글자는 실측/기존 비율 중앙값으로 기존 HY 대응값을 스케일해 보간.

--verify: 생성 배열을 커밋된 `static <NAME>_LATIN_0` 와 정확 비교(전 face 일치
시 0, 불일치/누락 시 non-zero). COM 불필요 — TSV 만 있으면 어느 OS 에서든
재검증 가능.
"""
import argparse
import os
import re
import statistics
import sys

sys.stdout.reconfigure(encoding="utf-8")
REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

_ap = argparse.ArgumentParser(description="ladder_<face>.tsv → font_metrics_data.rs LATIN_0")
_ap.add_argument(
    "--ladder-dir",
    default=os.path.join("output", "poc", "task2430"),
    help="ladder_<face>.tsv 디렉터리(리포 루트 기준 상대 허용)",
)
_ap.add_argument(
    "--src",
    default=os.path.join("src", "renderer", "font_metrics_data.rs"),
    help="기존 HY 메트릭 참조용 소스",
)
_ap.add_argument(
    "--verify",
    action="store_true",
    help="생성 배열을 --src 의 커밋된 static 과 정확 비교(불일치 시 non-zero)",
)
_args = _ap.parse_args()
OUT = _args.ladder_dir if os.path.isabs(_args.ladder_dir) else os.path.join(REPO_ROOT, _args.ladder_dir)
SRC = _args.src if os.path.isabs(_args.src) else os.path.join(REPO_ROOT, _args.src)

# (표시명, tsv face, 새 메트릭명, 기존 HY 메트릭 LATIN_0 static 이름)
TARGETS = [
    ("한양신명조", "한양신명조", "HanyangSinMyeongJo", "FONT_276_LATIN_0"),
    ("한양중고딕", "한양중고딕", "HanyangJungGothic", None),  # HY중고딕 static 탐색
    ("한양견명조", "한양견명조", "HanyangKyunMyeongJo", None),
    ("한양견고딕", "한양견고딕", "HanyangKyunGothic", None),
    ("휴먼명조", "휴먼명조", "HumanMyeongJo", "FONT_276_LATIN_0"),
]
# 기존 HY 메트릭명 → hangul/기타 참조용
HY_OF = {
    "HanyangSinMyeongJo": "HYSinMyeongJo-Medium",
    "HanyangJungGothic": "HYGothic-Medium",
    "HanyangKyunMyeongJo": "HYMyeongJo-Extra",
    "HanyangKyunGothic": "HYGothic-Extra",
    "HumanMyeongJo": "HYSinMyeongJo-Medium",
}

data = open(SRC, encoding="utf-8").read()

def hy_latin0_of(metric_name):
    m = re.search(r'name: "%s",.*?latin_ranges: &(\w+)_LATIN_RANGES' % re.escape(metric_name), data, re.S)
    base = m.group(1)
    m2 = re.search(r"static %s_LATIN_0: \[u16; \d+\] = \[(.*?)\];" % base, data, re.S)
    return [int(x) for x in re.findall(r"\d+", m2.group(1))], base

def hy_ranges_of(metric_name):
    m = re.search(r'name: "%s",.*?latin_ranges: &(\w+)_LATIN_RANGES,\s*hangul: (Some\(&\w+\)|None)' % re.escape(metric_name), data, re.S)
    return m.group(1), m.group(2)

def committed_latin0_of(newname):
    """--src 에 커밋된 static <NAME>_LATIN_0 배열을 읽는다(없으면 None)."""
    m = re.search(
        r"static %s_LATIN_0: \[u16; \d+\] = \[(.*?)\];" % newname.upper(), data, re.S
    )
    return [int(x) for x in re.findall(r"\d+", m.group(1))] if m else None


n_bad = 0
for disp, face, newname, _ in TARGETS:
    tsv = os.path.join(OUT, f"ladder_{face}.tsv")
    if not os.path.exists(tsv):
        print(f"// {disp}: TSV 없음 {os.path.relpath(tsv, REPO_ROOT)}")
        n_bad += 1
        continue
    meas = {}
    for line in open(tsv, encoding="utf-8"):
        p = line.rstrip("\n").split("\t")
        if p[0] in ("font", "face") or len(p) < 4:
            continue
        code, em = int(p[1]), float(p[3])
        if 0x20 <= code <= 0x7E and 0.01 < em < 2.5:
            meas[code] = em
    hy_metric = HY_OF[newname]
    hy_vals, base = hy_latin0_of(hy_metric)
    ratios = [meas[c] * 1024 / hy_vals[c - 0x20] for c in meas if hy_vals[c - 0x20] > 0]
    med_ratio = statistics.median(ratios)
    arr = []
    n_meas = n_interp = 0
    for c in range(0x20, 0x7F):
        if c in meas:
            arr.append(round(meas[c] * 1024))
            n_meas += 1
        else:
            arr.append(round(hy_vals[c - 0x20] * med_ratio))
            n_interp += 1
    if _args.verify:
        committed = committed_latin0_of(newname)
        if committed is None:
            print(f"{disp} → {newname}: 커밋 static 없음 — MISMATCH")
            n_bad += 1
        elif committed == arr:
            print(f"{disp} → {newname}: 95/95 exact match — OK")
        else:
            diffs = [
                (c, g, k)
                for c, g, k in zip(range(0x20, 0x7F), arr, committed)
                if g != k
            ]
            print(f"{disp} → {newname}: {len(diffs)}개 불일치 — MISMATCH "
                  f"(예: {[(hex(c), g, k) for c, g, k in diffs[:5]]})")
            n_bad += 1
        continue
    rng_base, hangul_ref = hy_ranges_of(hy_metric)
    print(f"// {disp} → {newname}: 실측 {n_meas}/95, 보간 {n_interp} (중앙비 {med_ratio:.3f}, 기준 {hy_metric})")
    body = ", ".join(str(v) for v in arr)
    print(f"pub(crate) static {newname.upper()}_LATIN_0: [u16; 95] = [{body}];")
    print(f"// ranges base: {rng_base}_LATIN_RANGES (LATIN_1+ 재사용), hangul: {hangul_ref}")
    print()

if _args.verify:
    sys.exit(1 if n_bad else 0)
if n_bad:
    sys.exit(1)
