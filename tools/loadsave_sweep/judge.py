#!/usr/bin/env python3
"""판정기 — Phase A 저널과 Phase B 오라클 측정을 합쳐 문서×경로별 판정을 낸다.

판정 어휘 (경로 하나에 여러 개 겹칠 수 있다 · 심각도 순):
    CONVERT_FAIL      rhwp 가 변환 자체를 못 함 (FAIL/TIMEOUT/SPAWN_FAIL)
    OPEN_FAIL         rhwp 산출물을 한글이 못 엶  ← 저장하기 치명 결함
    MEASURE_FAIL      한글이 열었지만 측정 중 오류 (판정 불가)
    TEXT_MISMATCH     본문 텍스트 불일치            ← 텍스트 누락/변형
    CTRL_DIFF         컨트롤 집계 불일치 (표·그림 등) ← 개체 누락
    PAGE_DIFF         페이지 수 불일치               ← 레이아웃 신호 (참고)
    OK                위 어느 것도 아님

원본이 한글에서 안 열리는 문서(ORACLE_ORIG_FAIL)는 판정 모수에서 제외해 따로 센다.
rhwp 자기검증(exit 3/4)은 selfVerify 열에 참고로 싣는다 — 오라클 판정과 독립이다.

원본 유령 성공 의심(원본 텍스트 0자 + 페이지 1)은 origSuspect 열로 표시한다. 보안 모듈
미등록 등으로 대화상자가 자동 거부되면 "빈 문서를 연 성공"이 나온다(메모리: 빈 PDF 사례).

사용:
    python judge.py --master master.tsv --phase-a <out>/phase_a.ndjson \
        --oracle <oracle_out>/result.tsv --texts <oracle_out>/texts --out <oracle_out>/verdicts
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROUTES = {"hwp": ["h2h", "h2x"], "hwpx": ["x2h", "x2x"]}
CONVERT_OK = ("OK", "VERIFY_DIFF", "PAGE_DIFF")


def norm_text(raw: bytes) -> str:
    s = raw.decode("utf-8", "replace")
    s = s.replace("\r\n", "\n").replace("\r", "\n")
    # 행말 공백과 문서 끝 공백만 무시한다 — 본문 문자·순서는 그대로 비교한다.
    return "\n".join(line.rstrip() for line in s.split("\n")).rstrip("\n")


def first_diff(a: str, b: str) -> str:
    la, lb = a.split("\n"), b.split("\n")
    for i, (x, y) in enumerate(zip(la, lb)):
        if x != y:
            return f"line {i}: orig={x[:60]!r} var={y[:60]!r}"
    if len(la) != len(lb):
        i = min(len(la), len(lb))
        longer = la if len(la) > len(lb) else lb
        side = "orig" if len(la) > len(lb) else "var"
        return f"line {i}: {side} extra {abs(len(la)-len(lb))} lines, first={longer[i][:60]!r}"
    return "?"


def parse_ctrls(s: str) -> dict[str, int]:
    out: dict[str, int] = {}
    for part in s.split(","):
        if ":" in part:
            k, v = part.rsplit(":", 1)
            try:
                out[k] = int(v)
            except ValueError:
                pass
    return out


def ctrl_diff_str(a: dict[str, int], b: dict[str, int]) -> str:
    keys = sorted(set(a) | set(b))
    return ",".join(f"{k}:{a.get(k, 0)}->{b.get(k, 0)}" for k in keys if a.get(k, 0) != b.get(k, 0))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--master", required=True)
    ap.add_argument("--phase-a", required=True)
    ap.add_argument("--oracle", required=True, help="oracle result.tsv")
    ap.add_argument("--texts", required=True)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    docs = []
    for line in Path(args.master).read_text(encoding="utf-8").splitlines():
        if line.strip():
            docid, fmt, src = line.split("\t", 2)
            docs.append({"docid": docid, "format": fmt, "src": src})

    phase_a: dict[tuple[str, str], dict] = {}
    for line in Path(args.phase_a).read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        r = json.loads(line)
        if r.get("kind") == "meta":
            continue
        phase_a[(r["docid"], r["route"])] = r  # 뒤 레코드(재실행)가 앞을 덮는다

    oracle: dict[str, dict] = {}
    for line in Path(args.oracle).read_text(encoding="utf-8").splitlines():
        cols = line.rstrip("\n").split("\t")
        if len(cols) < 8 or cols[0] == "__VERSION_MISMATCH__":
            continue
        oracle[cols[0]] = {
            "status": cols[1], "pages": int(cols[2]), "textLen": int(cols[3]),
            "textSha": cols[4], "ctrls": cols[5], "fileBytes": int(cols[6]), "err": cols[7],
        }

    texts_dir = Path(args.texts)
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)

    def load_text(key: str) -> str | None:
        p = texts_dir / f"{key}.txt"
        if not p.is_file():
            return None
        return norm_text(p.read_bytes())

    rows = []
    counts: dict[str, Counter] = defaultdict(Counter)
    n_orig_fail = 0
    n_orig_missing = 0
    n_orig_suspect = 0

    for doc in docs:
        docid, fmt = doc["docid"], doc["format"]
        orig = oracle.get(f"{docid}.orig")
        if orig is None:
            n_orig_missing += 1
            continue  # Phase B 미도달 (부분 실행) — 모수에서 제외
        if orig["status"] != "OK":
            n_orig_fail += 1
            rows.append([docid, fmt, "-", "ORACLE_ORIG_FAIL", "", "", "", "", orig["err"][:200], doc["src"]])
            continue
        orig_suspect = orig["textLen"] == 0 and orig["pages"] <= 1
        if orig_suspect:
            n_orig_suspect += 1
        orig_text = None  # 필요할 때만 읽는다
        orig_ctrls = parse_ctrls(orig["ctrls"])

        for route in ROUTES[fmt]:
            a = phase_a.get((docid, route))
            verdicts = []
            detail = ""
            self_verify = ""
            pages_str = ""
            len_delta = ""
            if a is None or a["status"] not in CONVERT_OK:
                verdicts.append("CONVERT_FAIL")
                detail = (a or {}).get("status", "NO_JOURNAL") + ": " + (a or {}).get("stderr", "")[:200]
            else:
                if a["status"] != "OK":
                    self_verify = a["status"]  # rhwp 자기검증 exit 3/4 — 참고 데이터
                var = oracle.get(f"{docid}.{route}")
                if var is None:
                    verdicts.append("MEASURE_FAIL")
                    detail = "no oracle row (phase B incomplete?)"
                elif var["status"] != "OK":
                    verdicts.append("OPEN_FAIL")
                    detail = var["err"][:200]
                else:
                    pages_str = f"{orig['pages']}->{var['pages']}"
                    len_delta = str(var["textLen"] - orig["textLen"])
                    if var["textSha"] != orig["textSha"]:
                        if orig_text is None:
                            orig_text = load_text(f"{docid}.orig")
                        var_text = load_text(f"{docid}.{route}")
                        if orig_text is None or var_text is None:
                            verdicts.append("MEASURE_FAIL")
                            detail = "text file missing"
                        elif orig_text != var_text:
                            verdicts.append("TEXT_MISMATCH")
                            detail = first_diff(orig_text, var_text)[:300]
                    cd = ctrl_diff_str(orig_ctrls, parse_ctrls(var["ctrls"]))
                    if cd:
                        verdicts.append("CTRL_DIFF")
                        detail = (detail + " | " if detail else "") + cd[:200]
                    if var["pages"] != orig["pages"]:
                        verdicts.append("PAGE_DIFF")
            verdict = ";".join(verdicts) if verdicts else "OK"
            counts[route][verdicts[0] if verdicts else "OK"] += 1
            rows.append([docid, fmt, route, verdict, self_verify, pages_str, len_delta,
                         "SUSPECT" if orig_suspect else "", detail, doc["src"]])

    header = ["docid", "format", "route", "verdict", "selfVerify", "pages", "textLenDelta",
              "origSuspect", "detail", "src"]
    verdict_path = out_dir / "verdicts.tsv"
    with verdict_path.open("w", encoding="utf-8", newline="\n") as f:
        f.write("\t".join(header) + "\n")
        for r in rows:
            f.write("\t".join(str(c) for c in r) + "\n")

    # 요약
    lines = ["# load/save 스윕 판정 요약", ""]
    total_judged = sum(sum(c.values()) for c in counts.values())
    lines.append(f"- 문서: {len(docs)} (원본 오라클 실패 {n_orig_fail}, Phase B 미도달 {n_orig_missing}, "
                 f"원본 유령성공 의심 {n_orig_suspect})")
    lines.append(f"- 판정된 (문서×경로): {total_judged}")
    lines.append("")
    lines.append("| route | OK | CONVERT_FAIL | OPEN_FAIL | TEXT_MISMATCH | CTRL_DIFF | PAGE_DIFF | MEASURE_FAIL |")
    lines.append("|---|---|---|---|---|---|---|---|")
    for route in ("h2h", "h2x", "x2h", "x2x"):
        c = counts[route]
        lines.append(f"| {route} | {c['OK']} | {c['CONVERT_FAIL']} | {c['OPEN_FAIL']} | "
                     f"{c['TEXT_MISMATCH']} | {c['CTRL_DIFF']} | {c['PAGE_DIFF']} | {c['MEASURE_FAIL']} |")
    lines.append("")
    lines.append("(표의 셀은 첫 번째(최고 심각도) 판정 기준. 겹친 판정 전체는 verdicts.tsv 의 verdict 열.)")
    bad = [r for r in rows if r[3] not in ("OK", "ORACLE_ORIG_FAIL")]
    if bad:
        lines.append("")
        lines.append(f"## 결함 상위 예시 ({min(len(bad), 20)}/{len(bad)})")
        sev = {"CONVERT_FAIL": 0, "OPEN_FAIL": 1, "MEASURE_FAIL": 2, "TEXT_MISMATCH": 3,
               "CTRL_DIFF": 4, "PAGE_DIFF": 5}
        bad.sort(key=lambda r: sev.get(r[3].split(";")[0], 9))
        for r in bad[:20]:
            lines.append(f"- `{r[0]}.{r[2]}` [{r[3]}] {r[8][:160]} — {Path(r[9]).name}")
    summary_path = out_dir / "summary.md"
    summary_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print("\n".join(lines))
    print(f"\n[judge] {verdict_path}")
    print(f"[judge] {summary_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
