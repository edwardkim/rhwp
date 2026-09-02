#!/usr/bin/env python3
"""판정기 — Phase A 저널과 Phase B 오라클 측정을 합쳐 문서×경로별 판정을 낸다.

판정 어휘 (경로 하나에 여러 개 겹칠 수 있다 · 심각도 순):
    CONVERT_FAIL      rhwp 가 변환 자체를 못 함 (FAIL/TIMEOUT/SPAWN_FAIL)
    OPEN_FAIL         rhwp 산출물을 한글이 못 엶  ← 저장하기 치명 결함
    ORACLE_TIMEOUT    감독의 stall-kill 이 만든 실패 ← 결함 아님, 재확인 필요 (#4751)
    MEASURE_FAIL      한글이 열었지만 측정 중 오류 (판정 불가)
    TEXT_MISMATCH     본문 텍스트 불일치            ← 텍스트 누락/변형
    CTRL_DIFF         컨트롤 집계 불일치 (표·그림 등) ← 개체 누락
    PAGE_DIFF         페이지 수 불일치               ← 레이아웃 신호 (참고)
    OK                위 어느 것도 아님

원본이 한글에서 안 열리는 문서(ORACLE_ORIG_FAIL)는 판정 모수에서 제외해 따로 센다.
원본이 stall-kill 에 걸린 문서(ORACLE_ORIG_TIMEOUT)도 모수에서 빠지되 결함이 아니라
재확인 대상이다 — stall-kill 키는 oracle_run.ps1 이 남기는 stall_kills.tsv 로 식별한다
(경로는 --stall-kills, 생략 시 result.tsv 옆에서 자동 탐색).
rhwp 자기검증(exit 3/4)은 selfVerify 열에 참고로 싣는다 — 오라클 판정과 독립이다.

원본 유령 성공 의심(원본 텍스트 0자 + 페이지 1)은 origSuspect 열에 SUSPECT 로 표시한다.
보안 모듈 미등록 등으로 대화상자가 자동 거부되면 "빈 문서를 연 성공"이 나온다(메모리: 빈 PDF 사례).

원본이 여러 쪽인데 텍스트가 0자면 한글의 **텍스트 추출이 실패**한 것이다(문서가 빈 게 아니다).
이때 텍스트 축은 비교 자체가 불가능하므로 TEXT_MISMATCH 를 내지 않고 origSuspect 열에
ORIG_TEXT_FAIL 로 표시한다 — 쪽수·컨트롤 축 판정은 그대로 유효하므로 유지한다.

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
    ap.add_argument("--stall-kills", default=None,
                    help="[#4751] oracle_run.ps1 의 stall_kills.tsv 경로 "
                         "(생략 시 result.tsv 옆의 stall_kills.tsv 자동 탐색)")
    args = ap.parse_args()

    # [#4751] 감독의 stall-kill 이 만든 실패는 실제 한글 크래시와 같은 HRESULT
    # (0x800706BE)로 기록된다. 죽인 키 목록으로 가려내 "결함"이 아니라
    # "측정 실패 — 재확인 필요"(ORACLE_TIMEOUT)로 판정한다.
    stall_path = Path(args.stall_kills) if args.stall_kills else Path(args.oracle).parent / "stall_kills.tsv"
    stall_killed: set[str] = set()
    if stall_path.is_file():
        for line in stall_path.read_text(encoding="utf-8-sig").splitlines():
            if line.strip():
                stall_killed.add(line.split("\t", 1)[0])

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
    n_orig_timeout = 0
    n_orig_missing = 0
    n_orig_suspect = 0
    n_orig_text_fail = 0

    for doc in docs:
        docid, fmt = doc["docid"], doc["format"]
        orig = oracle.get(f"{docid}.orig")
        if orig is None:
            n_orig_missing += 1
            continue  # Phase B 미도달 (부분 실행) — 모수에서 제외
        if orig["status"] != "OK":
            if f"{docid}.orig" in stall_killed:
                # [#4751] 원본이 stall-kill 에 걸리면 그 문서의 모든 경로가 모수에서
                # 빠진다 — 결함이 아니라 측정 실패이므로 별도 판정으로 가른다.
                n_orig_timeout += 1
                rows.append([docid, fmt, "-", "ORACLE_ORIG_TIMEOUT", "", "", "", "",
                             orig["err"][:200], doc["src"]])
            else:
                n_orig_fail += 1
                rows.append([docid, fmt, "-", "ORACLE_ORIG_FAIL", "", "", "", "", orig["err"][:200], doc["src"]])
            continue
        orig_suspect = orig["textLen"] == 0 and orig["pages"] <= 1
        if orig_suspect:
            n_orig_suspect += 1
        # 여러 쪽인데 0자 = 한글의 텍스트 추출 실패. 문서가 빈 게 아니다.
        orig_text_fail = orig["textLen"] == 0 and orig["pages"] > 1
        if orig_text_fail:
            n_orig_text_fail += 1
        # 원본이 0자면 유령 성공이든 추출 실패든 텍스트 축은 비교 자체가 성립하지 않는다
        # (0자 원본과의 차이는 결함과 측정 실패를 구별할 수 없다). 쪽수·컨트롤 축은 유효하다.
        text_axis_void = orig["textLen"] == 0
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
                    # [#4751] stall-kill 이 만든 ERR 는 저장하기 결함(OPEN_FAIL)이 아니라
                    # 측정 실패 — 재확인 대상(ORACLE_TIMEOUT)이다.
                    verdicts.append("ORACLE_TIMEOUT" if f"{docid}.{route}" in stall_killed
                                    else "OPEN_FAIL")
                    detail = var["err"][:200]
                else:
                    pages_str = f"{orig['pages']}->{var['pages']}"
                    len_delta = str(var["textLen"] - orig["textLen"])
                    if var["textSha"] != orig["textSha"] and not text_axis_void:
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
            orig_flag = "SUSPECT" if orig_suspect else ("ORIG_TEXT_FAIL" if orig_text_fail else "")
            rows.append([docid, fmt, route, verdict, self_verify, pages_str, len_delta,
                         orig_flag, detail, doc["src"]])

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
    lines.append(f"- 문서: {len(docs)} (원본 오라클 실패 {n_orig_fail}, 원본 stall-kill 재확인 대상 {n_orig_timeout}, "
                 f"Phase B 미도달 {n_orig_missing}, "
                 f"원본 유령성공 의심 {n_orig_suspect}, 원본 텍스트추출 실패 {n_orig_text_fail})")
    if n_orig_suspect or n_orig_text_fail:
        lines.append(f"  - 원본 텍스트가 0자인 문서 {n_orig_suspect + n_orig_text_fail}건은 텍스트 축 판정을 보류했다 "
                     f"(origSuspect 열: 0자+1쪽은 SUSPECT, 0자+여러 쪽은 ORIG_TEXT_FAIL). "
                     f"0자 원본과의 차이는 결함과 측정 실패를 구별할 수 없다. 쪽수·컨트롤 축은 그대로 판정한다.")
    lines.append(f"- 판정된 (문서×경로): {total_judged}")
    lines.append("")
    lines.append("| route | OK | CONVERT_FAIL | OPEN_FAIL | ORACLE_TIMEOUT | TEXT_MISMATCH | CTRL_DIFF | PAGE_DIFF | MEASURE_FAIL |")
    lines.append("|---|---|---|---|---|---|---|---|---|")
    for route in ("h2h", "h2x", "x2h", "x2x"):
        c = counts[route]
        lines.append(f"| {route} | {c['OK']} | {c['CONVERT_FAIL']} | {c['OPEN_FAIL']} | "
                     f"{c['ORACLE_TIMEOUT']} | "
                     f"{c['TEXT_MISMATCH']} | {c['CTRL_DIFF']} | {c['PAGE_DIFF']} | {c['MEASURE_FAIL']} |")
    lines.append("")
    lines.append("(표의 셀은 첫 번째(최고 심각도) 판정 기준. 겹친 판정 전체는 verdicts.tsv 의 verdict 열.)")
    # [#4751] stall-kill 유발 측정 실패는 결함 목록이 아니라 재확인 절로 안내한다.
    timeouts = [r for r in rows if r[3].split(";")[0] in ("ORACLE_TIMEOUT", "ORACLE_ORIG_TIMEOUT")]
    if timeouts:
        keys = [(f"{r[0]}.orig" if r[3].startswith("ORACLE_ORIG") else f"{r[0]}.{r[2]}")
                for r in timeouts]
        lines.append("")
        lines.append(f"## 재확인 필요 — stall-kill 유발 측정 실패 {len(timeouts)}건 (결함 아님)")
        for k in keys:
            lines.append(f"- `{k}`")
        lines.append("")
        lines.append("재확인: 위 키만 담은 task 파일을 만들어 감독을 넉넉한 임계로 재실행한 뒤 다시 판정한다.")
        lines.append("```")
        lines.append("oracle_run.ps1 -HwpVersion <ver> -TaskPath <재확인.task> -OutDir <재확인_out> -StallSeconds 1200")
        lines.append("```")
    bad = [r for r in rows
           if r[3] not in ("OK", "ORACLE_ORIG_FAIL", "ORACLE_ORIG_TIMEOUT")
           and r[3].split(";")[0] != "ORACLE_TIMEOUT"]
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
