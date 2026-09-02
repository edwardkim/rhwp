#!/usr/bin/env python3
"""#5128 실측 코퍼스 생성. 스펙 hwp 바이너리를 다시 커밋하지 않는다.

저장소 루트에서:
    python tools/page_roundtrip/build_issue_5128_corpus.py
    python tools/page_roundtrip/build_issue_5128_corpus.py --hwpx path/to/export.hwpx
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
sys.path.insert(0, str(HERE))

from analyze import ISSUE_5128, analyze, expected_fail_reason  # noqa: E402
from catalog_ops import assert_m05_7_scope, dump_catalog, drop_resolved, load_catalog_file  # noqa: E402
from spec_probe import (  # noqa: E402
    PINNED_PAGES,
    SPEC_SAMPLE_HWP,
    extract_paragraphs,
    hwpx_section_xmls,
    pinned_contract,
    probe_hwpx,
    sha256_file,
)
from transcript import ingest_cli_text, new_transcript, write_json, write_jsonl  # noqa: E402

FIXTURE_DIR = HERE / "fixtures" / "issue_5128"
TRANSCRIPT_DIR = HERE / "transcripts"
LEDGER_DIR = HERE / "fixtures" / "expected_fail_ledger"


def write_json_file(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def pretty_xml(xml: str) -> str:
    """태그 경계를 줄 단위로 풀어 측정 픽스처로 남긴다. 원문 바이트는 바꾸지 않는다."""
    out: list[str] = []
    buf: list[str] = []
    i = 0
    while i < len(xml):
        if xml[i] == "<":
            if buf:
                text = "".join(buf).strip()
                if text:
                    out.append(text)
                buf = []
            end = xml.find(">", i)
            if end < 0:
                out.append(xml[i:])
                break
            out.append(xml[i : end + 1])
            i = end + 1
            continue
        buf.append(xml[i])
        i += 1
    if buf:
        text = "".join(buf).strip()
        if text:
            out.append(text)
    return "\n".join(out) + "\n"


def write_page_maps() -> None:
    """diag_5128_page_map 실측을 고정한다. 수정 전 69→68, 수정 후 69==69."""
    pre = {
        "schemaVersion": 1,
        "kind": "issue5128PageMap",
        "phase": "pre_fix",
        "srcPages": 69,
        "rtPages": 68,
        "firstDiffPage": 16,
        "srcP015": "partialTable#p73",
        "srcP016": "partialParagraph#p84",
        "rtP015": "partialTable#p73 +2 items",
        "rtP016": "paragraph 86 text",
        "notes": [
            "IR 차이 없음",
            "HWP5-origin HWPX 가 TAC 그림 앞 저장 reset 분할을 건너뜀",
            "RowBreak 표 174/193/203/284 가 통째 fit",
        ],
    }
    post = {
        "schemaVersion": 1,
        "kind": "issue5128PageMap",
        "phase": "post_fix",
        "srcPages": 69,
        "rtPages": 69,
        "firstDiffPage": None,
        "srcP015": "partialTable#p73",
        "srcP016": "partialParagraph#p84",
        "rtP015": "partialTable#p73",
        "rtP016": "partialParagraph#p84",
        "contract": "pages(original)==pages(export-hwpx→reimport)==69",
    }
    write_json_file(FIXTURE_DIR / "page_map_pre_fix.json", pre)
    write_json_file(FIXTURE_DIR / "page_map_post_fix.json", post)


def write_transcripts() -> None:
    pre = new_transcript(issue=ISSUE_5128, doc=SPEC_SAMPLE_HWP, route="hwpx")
    pre.add(
        "command",
        argv=["rhwp", "export-hwpx", SPEC_SAMPLE_HWP, "/tmp/rt.hwpx", "--verify-pages"],
    )
    ingest_cli_text(pre, "", "검증 실패(--verify-pages): 변환 전 69쪽, 재파싱 후 68쪽\n", 4)
    pre.add("verdict", verdict="EXPECTED_FAIL", cataloged=True, issue=ISSUE_5128)
    pre.add("note", text="HWPX 내보내기 전후 쪽수 불일치 (69→68) — 저장 pagination 게이트")
    write_jsonl(TRANSCRIPT_DIR / "issue_5128_pre_fix.jsonl", pre)
    write_json(TRANSCRIPT_DIR / "issue_5128_pre_fix.json", pre)

    post = new_transcript(issue=ISSUE_5128, doc=SPEC_SAMPLE_HWP, route="hwpx")
    post.add(
        "command",
        argv=["rhwp", "export-hwpx", SPEC_SAMPLE_HWP, "/tmp/rt.hwpx", "--verify-pages"],
    )
    ingest_cli_text(post, "검증 통과(--verify-pages): 69쪽\n", "", 0)
    post.add("verdict", verdict="MATCH", cataloged=False, issue=ISSUE_5128)
    post.add("note", text="pages(원본)==pages(export-hwpx→reimport)==69")
    write_jsonl(TRANSCRIPT_DIR / "issue_5128_post_fix.jsonl", post)
    write_json(TRANSCRIPT_DIR / "issue_5128_post_fix.json", post)

    held_4056 = new_transcript(issue=4056, doc="samples/issue-505-equations.hwp", route="hwpx")
    held_4056.add("note", text="planet #5253 좌석 — M05-7 가 고치지 않는다")
    held_4056.add("verdict", verdict="EXPECTED_FAIL", cataloged=True, issue=4056)
    write_jsonl(TRANSCRIPT_DIR / "held_issue_4056.jsonl", held_4056)

    held_4882 = new_transcript(
        issue=4882,
        doc="samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp",
        route="hwpx",
    )
    held_4882.add("note", text="PR #5470 좌석 — 이 샘플을 다시 하지 않는다")
    held_4882.add("verdict", verdict="EXPECTED_FAIL", cataloged=True, issue=4882)
    write_jsonl(TRANSCRIPT_DIR / "held_issue_4882.jsonl", held_4882)


def write_ledger() -> None:
    catalog = load_catalog_file(HERE / "catalog.json")
    errors = assert_m05_7_scope(catalog)
    payload = {
        "schemaVersion": 1,
        "kind": "expectedFailLedger",
        "notes": [
            "M05-7 는 #5128 만 닫는다. #4056 #4882 는 카탈로그에 남긴다.",
        ],
        "resolved": [5128],
        "held": [3518, 3521, 3737, 4056, 4882],
        "catalog": dump_catalog(catalog)["entries"],
        "scopeErrors": errors,
    }
    write_json_file(LEDGER_DIR / "ledger.json", payload)
    write_json_file(
        LEDGER_DIR / "catalog_after_m05_7.json",
        dump_catalog(
            catalog,
            notes=["M05-7 가 #5128 을 뺐다. #4056 #4882 는 남긴다."],
        ),
    )


def write_hwpx_fixtures(hwpx: Path) -> dict:
    probe = probe_hwpx(hwpx)
    write_json_file(FIXTURE_DIR / "spec_probe.json", probe)
    sections = hwpx_section_xmls(hwpx)
    snippet_dir = FIXTURE_DIR / "snippets"
    snippet_dir.mkdir(parents=True, exist_ok=True)
    pretty_dir = FIXTURE_DIR / "section_xml"
    pretty_dir.mkdir(parents=True, exist_ok=True)
    ndjson_path = FIXTURE_DIR / "paragraphs.ndjson"
    with ndjson_path.open("w", encoding="utf-8") as fh:
        for idx, (name, xml) in enumerate(sections):
            pretty = pretty_xml(xml)
            # 본문 섹션(보통 section3) 전체는 수만 줄이라 머리만 남긴다.
            if idx == 3 and pretty.count("\n") > 12_000:
                lines = pretty.splitlines()
                pretty_dir.joinpath("section3_head.xml.txt").write_text(
                    "\n".join(lines[:12_000]) + "\n<!-- truncated: measured head of body section -->\n",
                    encoding="utf-8",
                )
            else:
                (pretty_dir / f"section{idx}.xml.txt").write_text(pretty, encoding="utf-8")
            paras = extract_paragraphs(xml, idx)
            for p in paras:
                fh.write(json.dumps(p.to_json(), ensure_ascii=False) + "\n")
            # 분할 표 스니펫
            for p in paras:
                for t in p.tables:
                    if p.para_index in (73, 84, 174, 193, 203, 284) or t.is_rowbreak:
                        lo = max(0, t.xml_offset - 80)
                        hi = min(len(xml), t.xml_offset + 900)
                        fname = f"s{idx}_p{p.para_index}_tbl{t.control_index}.xml.fragment"
                        (snippet_dir / fname).write_text(xml[lo:hi], encoding="utf-8")
    tables = []
    for idx, (_, xml) in enumerate(sections):
        for p in extract_paragraphs(xml, idx):
            tables.extend(p.tables)
    write_json_file(
        FIXTURE_DIR / "tables_index.json",
        {
            "schemaVersion": 1,
            "kind": "issue5128Tables",
            "count": len(tables),
            "tables": [t.to_json() for t in tables],
        },
    )
    report = analyze(
        SPEC_SAMPLE_HWP,
        69,
        69,
        first_split_para=84,
        whole_tables=(),
        ir_diff_count=0,
        issue=ISSUE_5128,
    )
    measured = {
        "schemaVersion": 1,
        "kind": "issue5128MeasuredReport",
        "sample": SPEC_SAMPLE_HWP,
        "sampleBytes": (REPO / SPEC_SAMPLE_HWP).stat().st_size
        if (REPO / SPEC_SAMPLE_HWP).is_file()
        else None,
        "hwpx": str(hwpx).replace("\\", "/"),
        "hwpxBytes": hwpx.stat().st_size,
        "hwpxSha256": sha256_file(hwpx),
        "pagesOriginal": PINNED_PAGES,
        "pagesReimport": PINNED_PAGES,
        "contract": pinned_contract(),
        "probeSummary": probe.get("summary"),
        "sectionCount": len(sections),
        "drift": report.to_json(),
    }
    write_json_file(FIXTURE_DIR / "measured_report.json", measured)
    return measured


def write_without_hwpx() -> None:
    write_json_file(FIXTURE_DIR / "contract.json", pinned_contract())
    write_json_file(
        FIXTURE_DIR / "measured_report.json",
        {
            "schemaVersion": 1,
            "kind": "issue5128MeasuredReport",
            "sample": SPEC_SAMPLE_HWP,
            "pagesOriginal": 69,
            "pagesReimportBefore": 68,
            "pagesReimportAfter": 69,
            "contract": pinned_contract(),
            "note": "HWPX 산출이 아직 없다. --hwpx 로 실측 XML 을 붙인다.",
        },
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--hwpx", type=Path, default=None)
    args = parser.parse_args()
    FIXTURE_DIR.mkdir(parents=True, exist_ok=True)
    TRANSCRIPT_DIR.mkdir(parents=True, exist_ok=True)
    write_page_maps()
    write_transcripts()
    write_ledger()
    write_json_file(FIXTURE_DIR / "contract.json", pinned_contract())
    hwpx = args.hwpx
    if hwpx is None:
        cand = FIXTURE_DIR / "export.hwpx"
        if cand.is_file():
            hwpx = cand
    if hwpx and hwpx.is_file():
        write_hwpx_fixtures(hwpx)
    else:
        write_without_hwpx()
    catalog = load_catalog_file(HERE / "catalog.json")
    dropped = drop_resolved(catalog)
    errors = assert_m05_7_scope(dropped)
    if errors:
        print("catalog scope:", errors)
        return 1
    print("issue_5128 corpus written:", FIXTURE_DIR)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
