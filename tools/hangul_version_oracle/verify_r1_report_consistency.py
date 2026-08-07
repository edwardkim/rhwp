#!/usr/bin/env python3
"""Machine verification of the r1 oracle report's internal consistency.

Re-derives every derivable figure in mydocs/report/hangul_version_oracle_r1_20260807.md
from the row-level tables the report itself carries (121 PAGE_DELTA + 123 BREAK_DIFF +
3 PARA_DIFF + 7 unconfirmed + 5 of 2020<->2022), cross-checks the reproduction guide and
the harness scripts, and bounds the 2020-baseline prediction:

    diff(2020,2024)  ==  (diff(2022,2024) \\ overlap-that-vanished)  UNION
                         (diff(2020,2022) \\ raw diff(2022,2024))

Docs absent from raw diff(2022,2024) have equal 2022/2024 tuples, and docs absent from
diff(2020,2022) have equal 2020/2022 tuples, so only the overlap needs a value check.
Everything runs without Hangul or the corpus.

Verdicts: PASS / FAIL (a claim is wrong and fixable here) / CONTRADICTION (two in-repo
statements provably conflict, resolving which side is right needs the raw measurement
data) / QUESTION (not decidable from in-repo data at all).

Usage:  python tools/hangul_version_oracle/verify_r1_report_consistency.py [--tsv OUT]
Exit code: 1 if any FAIL, else 2 if any CONTRADICTION, else 0.
"""

import argparse
import html
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
REPORT = REPO / "mydocs/report/hangul_version_oracle_r1_20260807.md"
MANUAL = REPO / "mydocs/manual/verification/hangul_version_oracle.md"
TOOLS = REPO / "tools/hangul_version_oracle"

results = []  # (id, verdict, subject, detail)
DOC_CELL = r"(?:``.*?``|`[^`]+`|<code>.*?</code>)"


def add(check_id, verdict, subject, detail=""):
    results.append((check_id, verdict, subject, detail))


def section(text, start_pat, end_pat):
    m = re.search(start_pat, text)
    if not m:
        return ""
    rest = text[m.end():]
    e = re.search(end_pat, rest)
    return rest[: e.start()] if e else rest


def int_(s):
    return int(s.replace(",", "").replace("*", ""))


def document_path(cell):
    if cell.startswith("``"):
        return cell[2:-2]
    if cell.startswith("`"):
        return cell[1:-1]
    return html.unescape(cell.removeprefix("<code>").removesuffix("</code>"))


def parse_tables(text):
    s4 = section(text, r"\n## 4\. ", r"\n## 5\. ")
    s51 = section(text, r"### 5\.1 ", r"### 5\.2 ")
    s52 = section(text, r"### 5\.2 ", r"### 5\.3 ")
    s53 = section(text, r"### 5\.3 ", r"### 5\.4 ")
    s54 = section(text, r"### 5\.4 ", r"### 5\.5 ")
    s55 = section(text, r"### 5\.5 ", r"\n## 6\. ")
    s6 = section(text, r"\n## 6\. ", r"\n## 7\. ")
    s8 = section(text, r"\n## 8\. ", r"\n## 9\. ")
    t = {}

    t["page_delta"] = [
        {"doc": document_path(m[0]), "p2022": int_(m[1]), "p2024": int_(m[2]), "delta": int(m[3])}
        for m in re.findall(
            rf"^\| ({DOC_CELL}) \| ([\d,]+) \| ([\d,]+) \| ([+-]\d+) \|$", s51, re.M
        )
    ]
    t["break_diff"] = [
        {"doc": document_path(m[0]), "pages": int_(m[1]), "d2022": m[2], "d2024": m[3]}
        for m in re.findall(
            rf"^\| ({DOC_CELL}) \| ([\d,]+) \| `([^`]+)` \| `([^`]+)` \|$", s52, re.M
        )
    ]
    t["para_diff"] = [
        {"doc": document_path(m[0]), "n2022": int_(m[1]), "n2024": int_(m[2])}
        for m in re.findall(
            rf"^\| ({DOC_CELL}) \| ([\d,]+) \| ([\d,]+) \|$", s53, re.M
        )
    ]
    t["src_table"] = {
        m[0]: int_(m[1]) for m in re.findall(r"^\| `([a-z_]+)` \| ([\d,]+) \|$", s54, re.M)
    }
    m = re.search(r"([\d,]+)건으로 압도적", s54)
    t["src_prose_top"] = int_(m.group(1)) if m else None

    t["hwpx_rows"] = [
        (m[1], int_(m[2]), float(m[3]))
        for m in re.findall(
            r"^\| \*{0,2}([\d—]+)\*{0,2} \| \*{0,2}([^|*]+?)\*{0,2} \| \*{0,2}([\d,]+)\*{0,2} \| \*{0,2}([\d.]+)%\*{0,2} \|$",
            s55, re.M,
        )
    ]
    t["hwp5_rows"] = [
        (m[0], int_(m[1]), float(m[2]))
        for m in re.findall(r"^\| ([\d.]+|나머지) \| ([\d,]+) \| ([\d.]+)% \|$", s55, re.M)
    ]

    t["unconfirmed"] = [
        {"verdict": m[0].strip(), "doc": document_path(m[1]), "detail": m[2].strip(), "rerun": m[3].strip()}
        for m in re.findall(
            rf"^\| ([A-Z_]+) \| ({DOC_CELL}) \| ([^|]+) \| ([^|]+) \|$", s6, re.M
        )
    ]
    t["d2020_2022"] = [
        {"kind": m[0], "doc": document_path(m[1]), "detail": m[2].strip()}
        for m in re.findall(
            rf"^\| ([A-Z_]+) \| ({DOC_CELL}) \| ([^|]+) \|$", s8, re.M
        )
    ]

    m = re.search(
        r"^\| 다름 ([\d,]+)건 \| \*\*([\d,]+) \(([\d.]+)%\)\*\* \| (\d+) \| (\d+)(?: \| (\d+))? \|$",
        s4,
        re.M,
    )
    t["verif"] = (
        {"raw": int_(m.group(1)), "conf": int_(m.group(2)), "pct": float(m.group(3)),
         "fp": int(m.group(4)), "fail": int(m.group(5)), "initial_err": int(m.group(6) or 0)}
        if m else None
    )
    m = re.search(r"^\| 동일 ([\d,]+)건 \| \*\*0\*\* \| ([\d,]+) \| 0 \|$", s4, re.M)
    t["controls"] = int_(m.group(1)) if m else None

    t["raw3"] = {
        (m[0], m[1]): int_(m[2])
        for m in re.findall(r"^\| (\d{4}) ↔ (\d{4}) \| \*{0,2}([\d,]+)\*{0,2} \|$", s8, re.M)
    }
    m = re.search(r"([\d,]+)건은 동일하다", text)
    t["same_claim"] = int_(m.group(1)) if m else None
    m = re.search(r"\(\.hwp ([\d,]+) / \.hwpx ([\d,]+)\)", text)
    t["corpus"] = (int_(m.group(1)), int_(m.group(2))) if m else None
    return t


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tsv", type=Path, help="also write findings as TSV")
    args = ap.parse_args()

    text = REPORT.read_text(encoding="utf-8")
    manual = MANUAL.read_text(encoding="utf-8")
    t = parse_tables(text)
    pd_, bd, pdiff = t["page_delta"], t["break_diff"], t["para_diff"]
    unconf, d2020 = t["unconfirmed"], t["d2020_2022"]

    # --- row counts against the report's own headline figures -------------------
    add("R01", "PASS" if len(pd_) == 121 else "FAIL", "5.1 PAGE_DELTA rows",
        f"parsed {len(pd_)} / claimed 121")
    add("R02", "PASS" if len(bd) == 123 else "FAIL", "5.2 BREAK_DIFF rows",
        f"parsed {len(bd)} / claimed 123")
    add("R03", "PASS" if len(pdiff) == 3 else "FAIL", "5.3 PARA_DIFF rows",
        f"parsed {len(pdiff)} / claimed 3")
    total = len(pd_) + len(bd) + len(pdiff)
    add("R04", "PASS" if total == 247 else "FAIL", "confirmed total",
        f"{len(pd_)}+{len(bd)}+{len(pdiff)} = {total} / claimed 247")

    # --- 5.1 arithmetic ---------------------------------------------------------
    bad_delta = [r for r in pd_ if r["p2024"] - r["p2022"] != r["delta"]]
    add("R05", "PASS" if not bad_delta else "FAIL",
        "5.1 per-row delta == 2024-2022",
        "; ".join(f"{r['doc'][:40]}... {r['p2022']}->{r['p2024']} labelled {r['delta']:+d}"
                  for r in bad_delta) or f"all {len(pd_)} rows consistent")
    more = sum(1 for r in pd_ if r["delta"] > 0)
    less = sum(1 for r in pd_ if r["delta"] < 0)
    add("R06", "PASS" if (more, less) == (81, 40) else "FAIL",
        "5.1 direction counts", f"2024 more {more} / fewer {less} vs claimed 81/40")
    pm1 = sum(1 for r in pd_ if abs(r["delta"]) == 1)
    pct = round(100 * pm1 / len(pd_)) if pd_ else 0
    add("R07", "PASS" if pm1 == 87 and pct == 72 else "FAIL",
        "5.1 +-1 page count", f"{pm1} rows ({pct}%) vs claimed 87 (72%)")
    tops = {"국제통계 동향과 분석": -77, "성범죄 동향 및 추세 분석": 48,
            "이용자 만족도 조사": 33, "전기사업법": -15}
    top_bad = []
    for key, want in tops.items():
        row = next((r for r in pd_ if key in r["doc"]), None)
        if row is None or row["delta"] != want:
            top_bad.append(f"{key}: {row['delta'] if row else 'missing'} vs {want:+d}")
    add("R08", "PASS" if not top_bad else "FAIL", "5.1 headline examples",
        "; ".join(top_bad) or "top deltas match the prose")

    # --- duplicates / cross-membership -----------------------------------------
    all_docs = [r["doc"] for r in pd_] + [r["doc"] for r in bd] + [r["doc"] for r in pdiff]
    dupes = {d for d in all_docs if all_docs.count(d) > 1}
    add("R10", "PASS" if not dupes else "FAIL", "no doc in two confirmed tables",
        "; ".join(sorted(dupes)) or f"{len(set(all_docs))} unique docs")
    overlap_unconf = set(all_docs) & {r["doc"] for r in unconf}
    add("R35", "PASS" if not overlap_unconf else "FAIL",
        "unconfirmed docs not in confirmed list",
        "; ".join(sorted(overlap_unconf)) or "disjoint")
    d8_unconf = {r["doc"] for r in d2020} & {r["doc"] for r in unconf}
    add("R42", "PASS" if not d8_unconf else "FAIL",
        "8. docs not among the 7 unconfirmed",
        "; ".join(sorted(d8_unconf)) or "disjoint (keeps the R21 case split clean)")

    # --- 5.4 source distribution ------------------------------------------------
    recount = {}
    for d in all_docs:
        recount[d.split("\\")[0]] = recount.get(d.split("\\")[0], 0) + 1
    diff_rows = {k: (t["src_table"].get(k), recount.get(k))
                 for k in set(t["src_table"]) | set(recount)
                 if t["src_table"].get(k) != recount.get(k)}
    add("R11", "PASS" if not diff_rows else "FAIL", "5.4 table vs recount",
        "; ".join(f"{k}: table={a} actual={b}" for k, (a, b) in sorted(diff_rows.items()))
        or "all rows match")
    add("R11b", "PASS" if sum(t["src_table"].values()) == total else "FAIL",
        "5.4 table sums to confirmed total", f"sum={sum(t['src_table'].values())} vs {total}")
    top_src = max(recount, key=recount.get) if recount else None
    add("R12", "PASS" if t["src_prose_top"] == recount.get(top_src) else "FAIL",
        "5.4 prose top-source count vs rows",
        f"prose says {t['src_prose_top']}, rows give {top_src}={recount.get(top_src)}")

    # --- 5.5 app-version distribution (parsed from the document) -----------------
    hwpx_sum = sum(n for _, n, _ in t["hwpx_rows"])
    hwp5_sum = sum(n for _, n, _ in t["hwp5_rows"])
    corpus = t["corpus"] or (None, None)
    add("R13", "PASS" if len(t["hwpx_rows"]) == 7 and hwpx_sum == corpus[1] else "FAIL",
        "5.5 HWPX rows sum to corpus .hwpx count",
        f"{len(t['hwpx_rows'])} rows, sum={hwpx_sum} vs {corpus[1]}")
    pct_bad = [f"{label}: {claimed}% vs {100 * n / hwpx_sum:.1f}%"
               for label, n, claimed in t["hwpx_rows"]
               if abs(claimed - 100 * n / hwpx_sum) > 0.051]
    n2020 = next((n for label, n, _ in t["hwpx_rows"] if "2020" in label), 0)
    le2022 = sum(n for label, n, _ in t["hwpx_rows"] if label.strip() in
                 ("2010", "2014", "2018", "2020", "2022"))
    claims = [
        ("70.7" in text, f"2020 share {100 * n2020 / hwpx_sum:.1f}%"),
        (f"98.4%({le2022:,}건)" in text.replace("건)다", "건)"),
         f"<=2022 {le2022} ({100 * le2022 / hwpx_sum:.1f}%)"),
    ]
    add("R13b", "PASS" if not pct_bad and abs(100 * n2020 / hwpx_sum - 70.7) <= 0.051
        and abs(100 * le2022 / hwpx_sum - 98.4) <= 0.051 and le2022 == 3362 else "FAIL",
        "5.5 HWPX percentages", "; ".join(pct_bad) or
        f"all row %s match; 2020={100 * n2020 / hwpx_sum:.1f}%, <=2022 {le2022} ({100 * le2022 / hwpx_sum:.1f}%)")
    pct_bad5 = [f"{label}: {claimed}% vs {100 * n / hwp5_sum:.1f}%"
                for label, n, claimed in t["hwp5_rows"]
                if abs(claimed - 100 * n / hwp5_sum) > 0.051]
    add("R14", "PASS" if len(t["hwp5_rows"]) == 6 and hwp5_sum == corpus[0] and not pct_bad5 else "FAIL",
        "5.5 HWP5 rows sum to corpus .hwp count",
        "; ".join(pct_bad5) or f"{len(t['hwp5_rows'])} rows, sum={hwp5_sum} vs {corpus[0]}")
    add("R15", "PASS" if corpus[0] + corpus[1] == 10000 else "FAIL",
        "corpus composition", f"{corpus[0]:,} + {corpus[1]:,} == 10,000")

    # --- section 4 vs section 6 -------------------------------------------------
    v = t["verif"]
    add("R16", "PASS" if v and v["conf"] + v["fp"] + v["fail"] + v["initial_err"] == v["raw"]
        and abs(100 * v["conf"] / (v["raw"] - v["fail"]) - v["pct"]) <= 0.051 else "FAIL",
        "4. verification table arithmetic",
        f"{v['raw']} = {v['conf']}+{v['fp']}+{v['fail']}+{v['initial_err']}; {v['pct']}% = {v['conf']}/{v['raw'] - v['fail']} "
        "(denominator excludes rerun failures)" if v else "table row not parsed")
    add("R17", "PASS" if v and len(unconf) == v["raw"] - v["conf"] else "FAIL",
        "6. unconfirmed rows", f"parsed {len(unconf)} vs {v['raw']}-{v['conf']}" if v else "n/a")
    rerun_match = sum(1 for r in unconf if "MATCH" in r["rerun"])
    rerun_fail = sum(
        1 for r in unconf
        if r["verdict"] != "ERR" and ("ERR" in r["rerun"] or "failed" in r["rerun"])
    )
    initial_err = sum(1 for r in unconf if r["verdict"] == "ERR")
    ok = v and (rerun_match, rerun_fail, initial_err) == (v["fp"], v["fail"], v["initial_err"])
    add("R18", "PASS" if ok else "CONTRADICTION",
        "4. false-positive/failure split vs 6. table",
        f"4. claims {v['fp']} rerun-identical + {v['fail']} rerun failures + {v['initial_err']} initial ERR; 6. rows show "
        f"rerun=MATCH x{rerun_match}, rerun failure x{rerun_fail}, initial ERR x{initial_err}. Which side is right "
        "needs the raw diff_unconfirmed.tsv" if not ok else
        f"{rerun_match}/{rerun_fail}/{initial_err} matches 4. table")

    # --- section 8: the 2020 baseline -------------------------------------------
    add("R19", "PASS" if len(d2020) == 5 == t["raw3"].get(("2020", "2022")) else "FAIL",
        "8. 2020<->2022 rows", f"parsed {len(d2020)} / table says {t['raw3'].get(('2020', '2022'))}")
    raw254 = set(all_docs) | {r["doc"] for r in unconf}
    d22_24 = t["raw3"].get(("2022", "2024"))
    d20_24 = t["raw3"].get(("2020", "2024"))
    add("R19b", "PASS" if v and d22_24 == v["raw"] else "FAIL",
        "8. re-measured diff(2022,2024) equals r1 raw",
        f"8. table {d22_24} vs 4. raw {v['raw'] if v else '?'}")

    conf_by_doc = {r["doc"]: r for r in pd_}
    overlap = [r for r in d2020 if r["doc"] in raw254]
    d1_only = [r for r in d2020 if r["doc"] not in raw254]
    olap_bad = []
    stay_definite, indeterminate = 0, []
    for r in overlap:
        m20 = re.search(r"2020=(\d+)p", r["detail"])
        m22 = re.search(r"2022=(\d+)p", r["detail"])
        row51 = conf_by_doc.get(r["doc"])
        if r["kind"] == "PAGE_DELTA" and m20 and m22 and row51:
            if row51["p2022"] != int(m22.group(1)):
                olap_bad.append(f"{r['doc']}: 8. says 2022={m22.group(1)} vs 5.1 says {row51['p2022']}")
            if int(m20.group(1)) != row51["p2024"]:
                stay_definite += 1  # page counts differ -> tuples differ
            else:
                indeterminate.append(r["doc"])  # equal pages could still be BREAK/PARA_DIFF
        else:
            indeterminate.append(r["doc"])  # unconfirmed or non-5.1 overlap: no values to compare
    add("R20", "PASS" if not olap_bad else "FAIL",
        "8. overlap docs agree with 5.1 on the 2022 value",
        "; ".join(olap_bad) or
        f"{len(overlap)} overlap doc(s): {stay_definite} value-checked, {len(indeterminate)} indeterminate")

    # Docs absent from raw diff(2022,2024) have equal 2022/2024 tuples, so every
    # 2020-only diff doc must join diff(2020,2024). Overlap docs are case-checked above;
    # equal page counts do NOT imply the diff vanished (paras/fingerprint may differ).
    lo = (d22_24 - len(overlap)) + len(d1_only) + stay_definite
    hi = lo + len(indeterminate)
    exact = "exact" if not indeterminate else f"range [{lo},{hi}] ({len(indeterminate)} indeterminate)"
    add("R21", "PASS" if lo <= d20_24 <= hi else "FAIL",
        "8. predicted |diff(2020,2024)| brackets the measured value",
        f"({d22_24}-{len(overlap)} overlap) + {len(d1_only)} 2020-only + {stay_definite} stayed "
        f"= {lo}..{hi} vs measured {d20_24} ({exact}). Note: given 5/254/overlap this is a "
        "1-bit test on counts; element-swap errors stay invisible")

    # diff(2020,2024) is a subset of the union of the other two diffs: a non-MATCH pair
    # implies a non-OK status (recorded as ERR/MISSING in whichever diff touches that
    # version -- compare_passes.ps1 records those as rows too) or unequal tuples.
    union = t["raw3"].get(("2020", "2022"), 0) + d22_24 - len(overlap)
    m263 = re.search(r"합집합 (?:\*\*)?([\d,]+)건", text)
    claimed_union = int_(m263.group(1)) if m263 else None
    add("R22", "PASS" if claimed_union == union else "CONTRADICTION",
        "8. verify-list union count",
        f"pairwise union = 5+{d22_24}-{len(overlap)} = {union}; report claims {claimed_union}. "
        f"ERR/MISSING rows are already inside the pairwise counts and a 2018 pass would add "
        f"thousands of MISSING rows, so {claimed_union} cannot come from pairwise verdicts; "
        f"likely true split is {union} diffs + 100 controls = 358 (controls default; 4. used 100). "
        "Needs verify_list_3way.txt / diff TSVs to settle")

    # --- report vs manual vs scripts --------------------------------------------
    clsids = set(re.findall(r"\{2291CF00-[0-9A-F-]+\}", text + manual))
    for ps1 in sorted(TOOLS.glob("*.ps1")):
        clsids |= set(re.findall(r"\{2291CF00-[0-9A-F-]+\}", ps1.read_text(encoding="utf-8")))
    add("R23", "PASS" if len(clsids) == 1 else "FAIL", "single CLSID everywhere",
        "; ".join(sorted(clsids)))
    non_ascii = [p.name for p in sorted(TOOLS.glob("*.ps1"))
                 if any(b > 127 for b in p.read_bytes())]
    add("R26", "PASS" if not non_ascii else "FAIL", "harness .ps1 files are ASCII",
        "; ".join(non_ascii) or "all 7 scripts ASCII-clean")
    run = (TOOLS / "page_oracle_run.ps1").read_text(encoding="utf-8")
    worker = (TOOLS / "page_oracle_worker.ps1").read_text(encoding="utf-8")
    checks = [
        ("R27", r"\[int\]\$Workers = 1", run, "-Workers defaults to 1"),
        ("R27b", r"\[int\]\$StallSeconds = 300", run, "-StallSeconds defaults to 300"),
        ("R27c", r"\[int\]\$WarmupDocs = 5", run, "-WarmupDocs defaults to 5"),
        ("R27d", r"\[switch\]\$HideWindow", run, "-HideWindow is opt-in"),
        ("R28", r"FileShare", run, "watcher opens TSV with FileShare"),
        ("R29", r"exit 3", worker, "worker exits 3 on major mismatch"),
        ("R32", r"Wow6432Node", run, "run script writes both registry views"),
    ]
    for cid, pat, src, label in checks:
        add(cid, "PASS" if re.search(pat, src) else "FAIL", label, f"pattern `{pat}`")
    kinds = {"PAGE_DELTA", "BREAK_DIFF", "PARA_DIFF", "ERR", "MISSING"}
    cmp_lines = [
        ln for ln in (TOOLS / "compare_passes.ps1").read_text(encoding="utf-8").splitlines()
        if not ln.lstrip().startswith("#")
    ]
    missing_kinds = {k for k in kinds if not any(k in ln for ln in cmp_lines)}
    add("R25", "PASS" if not missing_kinds else "FAIL",
        "compare_passes verdict kinds (in code, not comments)",
        "; ".join(sorted(missing_kinds)) or "all five kinds emitted by code")
    write_line = r"\$rel`t\$status`t\$pages`t\$paras`t\$bc`t\$fp"
    add("R24", "PASS" if re.search(write_line, worker) else "FAIL",
        "worker TSV write matches the documented column order",
        "relpath / status / pages / paras / breakCount / fingerprint")

    # --- residual claim checks ---------------------------------------------------
    add("R41", "PASS" if v and t["same_claim"] == 10000 - v["raw"] else "FAIL",
        "overview: identical-doc count",
        f"claimed {t['same_claim']:,} == 10,000 - {v['raw']} raw non-MATCH "
        "(the raw count includes 1 ERR doc, so 'identical' means MATCH verdicts)" if v else "n/a")
    add("R40", "PASS" if round(100 * total / 10000, 2) == 2.47 and round(100 * len(d2020) / 10000, 2) == 0.05 else "FAIL",
        "headline percentages", f"{total}/10k = 2.47%, {len(d2020)}/10k = 0.05%")
    same_bd = [r["doc"] for r in bd if r["d2022"] == r["d2024"]]
    add("R39", "PASS" if not same_bd else "FAIL",
        "5.2 divergence cells differ per row", "; ".join(same_bd) or "all rows differ")
    same_para = [r["doc"] for r in pdiff if r["n2022"] == r["n2024"]]
    add("R38", "PASS" if not same_para else "FAIL",
        "5.3 paragraph counts differ per row", "; ".join(same_para) or "all rows differ")

    # --- emit --------------------------------------------------------------------
    width = max(len(r[2]) for r in results)
    tally = {"PASS": 0, "FAIL": 0, "CONTRADICTION": 0, "QUESTION": 0}
    for cid, verdict, subject, detail in results:
        tally[verdict] += 1
        print(f"{cid:5} {verdict:13} {subject:{width}}  {detail}")
    print(f"\n{len(results)} checks: " + ", ".join(f"{n} {k}" for k, n in tally.items() if n))
    if args.tsv:
        with args.tsv.open("w", encoding="utf-8", newline="") as f:
            f.write("id\tverdict\tsubject\tdetail\n")
            for row in results:
                f.write("\t".join(row) + "\n")
    return 1 if tally["FAIL"] else (2 if tally["CONTRADICTION"] else 0)


if __name__ == "__main__":
    sys.exit(main())
