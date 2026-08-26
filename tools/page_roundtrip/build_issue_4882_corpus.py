#!/usr/bin/env python3
"""#4882 실측 코퍼스 생성. 100MB hwp 를 커밋하지 않고 XML 스니펫·리포트를 남긴다.

저장소 루트에서:
    python tools/page_roundtrip/build_issue_4882_corpus.py
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
sys.path.insert(0, str(HERE))

from analyze import (  # noqa: E402
    ISSUE_4882,
    PINNED_4882_IR,
    PINNED_4882_PATHS,
    analyze,
    expected_fail_reason,
)
from catalog_ops import dump_catalog  # noqa: E402
from harness import CatalogEntry  # noqa: E402
from note_probe import (  # noqa: E402
    extract_notes_from_section_xml,
    hwpx_entry_names,
    hwpx_section_xml,
    notes_to_json,
    sha256_file,
    summarize_notes,
)
from transcript import ingest_cli_text, new_transcript, write_json, write_jsonl  # noqa: E402

SAMPLE_HWP = "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp"
SAMPLE_HWPX = "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx"
FIXTURE_DIR = HERE / "fixtures" / "issue_4882"
TRANSCRIPT_DIR = HERE / "transcripts"
LEDGER_DIR = HERE / "fixtures" / "expected_fail_ledger"


def write_json_file(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def build_note_corpus() -> dict:
    hwpx = REPO / SAMPLE_HWPX
    if not hwpx.is_file():
        raise SystemExit(f"HWPX 표본이 없다: {hwpx}")
    names = hwpx_entry_names(hwpx)
    xml = hwpx_section_xml(hwpx)
    notes = extract_notes_from_section_xml(xml)
    payload = notes_to_json(notes, source=SAMPLE_HWPX)
    payload["zipEntries"] = names
    payload["section0Bytes"] = len(xml.encode("utf-8"))
    payload["section0Sha256"] = hashlib.sha256(xml.encode("utf-8")).hexdigest()
    write_json_file(FIXTURE_DIR / "notes_index.json", payload)

    zero = [n for n in notes if n.has_hwp5_zero_pattern]
    write_json_file(
        FIXTURE_DIR / "hwp5_zero_vpos_notes.json",
        {
            "schemaVersion": 1,
            "kind": "issue4882ZeroVposNotes",
            "source": SAMPLE_HWPX,
            "count": len(zero),
            "notes": [n.to_json() for n in zero],
        },
    )

    # 줄 단위 NDJSON — 측정 리포트. 원본 XML 을 되살릴 수 있는 9필드.
    ndjson_path = FIXTURE_DIR / "note_linesegs.ndjson"
    with ndjson_path.open("w", encoding="utf-8") as fh:
        for n in notes:
            for p in n.paragraphs:
                for si, seg in enumerate(p.segs):
                    row = {
                        "kind": n.kind,
                        "number": n.number,
                        "instId": n.inst_id,
                        "inTable": n.in_table,
                        "para": p.para_index,
                        "seg": si,
                        **seg.to_json(),
                        "text": p.text[:80],
                    }
                    fh.write(json.dumps(row, ensure_ascii=False) + "\n")

    # 스니펫: 전 줄 vpos=0 인 각주 XML 을 잘라 재파싱 픽스처로 남긴다.
    snippet_dir = FIXTURE_DIR / "snippets"
    snippet_dir.mkdir(parents=True, exist_ok=True)
    written = 0
    for n in zero:
        if written >= 24:
            break
        # xml_offset 은 body start. 앞뒤 400자를 잘라 문맥을 남긴다.
        lo = max(0, n.xml_offset - 200)
        hi = min(len(xml), n.xml_offset + 1600)
        snippet = xml[lo:hi]
        name = f"note_{n.kind}_{n.inst_id or n.number or written:04}.xml.fragment"
        (snippet_dir / name).write_text(snippet, encoding="utf-8")
        written += 1

    return {
        "notes": len(notes),
        "zero": len(zero),
        "summary": summarize_notes(notes),
        "section0Bytes": len(xml.encode("utf-8")),
    }


def build_measured_report(note_stats: dict) -> None:
    hwp = REPO / SAMPLE_HWP
    hwpx = REPO / SAMPLE_HWPX
    report = {
        "schemaVersion": 1,
        "kind": "issue4882MeasuredReport",
        "issue": ISSUE_4882,
        "doc": SAMPLE_HWP,
        "companionHwpx": SAMPLE_HWPX,
        "pagesOriginal": 215,
        "pagesExportReimportBeforeFix": 223,
        "pagesExportReimportAfterFix": 215,
        "deltaBeforeFix": 8,
        "irDiffsBeforeFix": [
            {"path": path, "diff": diff}
            for path, diff in zip(PINNED_4882_PATHS, PINNED_4882_IR)
        ],
        "files": {
            "hwp": {
                "path": SAMPLE_HWP,
                "bytes": hwp.stat().st_size if hwp.is_file() else None,
                "sha256": sha256_file(hwp) if hwp.is_file() else None,
            },
            "hwpx": {
                "path": SAMPLE_HWPX,
                "bytes": hwpx.stat().st_size if hwpx.is_file() else None,
                "sha256": sha256_file(hwpx) if hwpx.is_file() else None,
            },
        },
        "notes": note_stats,
        "repro": [
            "python tools/page_roundtrip/harness.py --file "
            + SAMPLE_HWP
            + " --route hwpx",
            "rhwp export-hwpx \"" + SAMPLE_HWP + "\" /tmp/rt.hwpx --verify-pages --verify --json",
        ],
        "outOfScope": [
            "ole/shape-component",
            "char_shapes",
            "#4056 issue-505-equations.hwp",
            "#5128 한글문서파일형식_5.0_revision1.3.hwp",
        ],
    }
    write_json_file(FIXTURE_DIR / "measured_report.json", report)


def build_transcripts() -> None:
    TRANSCRIPT_DIR.mkdir(parents=True, exist_ok=True)
    # 이슈 본문에 실린 기계 판정 원문.
    t = new_transcript(doc=SAMPLE_HWP, route="hwpx", issue=ISSUE_4882)
    t.add(
        "command",
        argv=[
            "rhwp",
            "export-hwpx",
            SAMPLE_HWP,
            "/tmp/tmp.OnkvEoBfAZ/rt.hwpx",
            "--verify-pages",
            "--verify",
        ],
    )
    stdout = "저장 완료: /tmp/tmp.OnkvEoBfAZ/rt.hwpx (5246KB)\n"
    stderr_lines = [
        "검증 실패(--verify-pages): 변환 전 215쪽, 재파싱 후 223쪽",
        "검증 실패(--verify): /tmp/tmp.OnkvEoBfAZ/rt.hwpx 재파싱 후 IR 차이 5건",
    ]
    for path, diff in zip(PINNED_4882_PATHS, PINNED_4882_IR):
        stderr_lines.append(f"  [차이] {path} linesegs: {diff}")
    stderr = "\n".join(stderr_lines) + "\n"
    ingest_cli_text(t, stdout, stderr, 4)
    t.add("verdict", verdict="EXPECTED_FAIL", cataloged=True, issue=ISSUE_4882)
    t.add(
        "note",
        text="M05-6 수정 전 기계 판정. 쪽수 215→223, 각주 vertpos 5건.",
    )
    write_jsonl(TRANSCRIPT_DIR / "issue_4882_pre_fix.jsonl", t)
    write_json(TRANSCRIPT_DIR / "issue_4882_pre_fix.json", t)

    t2 = new_transcript(doc=SAMPLE_HWP, route="hwpx", issue=ISSUE_4882)
    t2.add(
        "command",
        argv=["rhwp", "export-hwpx", SAMPLE_HWP, "/tmp/rt.hwpx", "--verify-pages", "--json"],
    )
    ingest_cli_text(
        t2,
        json.dumps(
            {
                "schemaVersion": 1,
                "verifyPages": {"before": 215, "after": 215, "identical": True},
            },
            ensure_ascii=False,
        ),
        "검증 통과(--verify-pages): 215쪽\n",
        0,
    )
    t2.add("verdict", verdict="MATCH", cataloged=False, issue=ISSUE_4882)
    t2.add("note", text="M05-6 수정 후 기대 봉투. 쪽수 215==215.")
    write_jsonl(TRANSCRIPT_DIR / "issue_4882_post_fix.jsonl", t2)
    write_json(TRANSCRIPT_DIR / "issue_4882_post_fix.json", t2)

    # 카탈로그에 남는 다른 좌석 전사 — 고치지 않았다는 기록.
    held = [
        (
            4056,
            "samples/issue-505-equations.hwp",
            4,
            1,
            "검증 실패(--verify-pages): 변환 전 4쪽, 재파싱 후 1쪽\n",
        ),
        (
            5128,
            "samples/한글문서파일형식_5.0_revision1.3.hwp",
            69,
            68,
            "검증 실패(--verify-pages): 변환 전 69쪽, 재파싱 후 68쪽\n",
        ),
    ]
    for issue, doc, before, after, err in held:
        th = new_transcript(doc=doc, route="hwpx", issue=issue)
        th.add("command", argv=["rhwp", "export-hwpx", doc, "/tmp/rt.hwpx", "--verify-pages"])
        ingest_cli_text(th, "", err, 4)
        th.add("verdict", verdict="EXPECTED_FAIL", cataloged=True, issue=issue)
        th.add("note", text=expected_fail_reason(issue, before, after))
        write_jsonl(TRANSCRIPT_DIR / f"held_issue_{issue}.jsonl", th)


def build_ledger() -> None:
    LEDGER_DIR.mkdir(parents=True, exist_ok=True)
    entries = [
        CatalogEntry(
            "samples/hwp3-sample16.hwp",
            "hwpx",
            3518,
            "HWPX 내보내기 전후 쪽수 불일치 (64→65) — M05-2",
        ),
        CatalogEntry(
            "samples/synam-001.hwp",
            "hwpx",
            3521,
            "HWPX 내보내기 전후 쪽수 불일치 (35→36) — M05-3",
        ),
        CatalogEntry(
            "samples/hwp3-sample11.hwp",
            "hwpx",
            3737,
            "HWPX 내보내기 전후 쪽수 불일치 (151→152) — M05-4",
        ),
        CatalogEntry(
            "samples/issue-505-equations.hwp",
            "hwpx",
            4056,
            expected_fail_reason(4056, 4, 1),
        ),
        CatalogEntry(
            "samples/한글문서파일형식_5.0_revision1.3.hwp",
            "hwpx",
            5128,
            expected_fail_reason(5128, 69, 68),
        ),
    ]
    history = []
    for e in entries:
        history.append(
            {
                "doc": e.doc,
                "route": e.route,
                "issue": e.issue,
                "reason": e.reason,
                "state": "held" if e.issue in {4056, 5128, 3518, 3521, 3737} else "open",
                "seat": {
                    3518: "M05-2",
                    3521: "M05-3",
                    3737: "M05-4",
                    4056: "foreign",
                    5128: "foreign",
                }.get(e.issue or 0, ""),
            }
        )
    history.append(
        {
            "doc": SAMPLE_HWP,
            "route": "hwpx",
            "issue": 4882,
            "reason": expected_fail_reason(4882, 215, 223),
            "state": "resolved",
            "seat": "M05-6",
            "pagesBefore": 215,
            "pagesAfterFix": 215,
        }
    )
    write_json_file(
        LEDGER_DIR / "ledger.json",
        {
            "schemaVersion": 1,
            "kind": "pageRoundtripExpectedFailLedger",
            "notes": [
                "M05-6 는 #4882 만 닫는다. #4056 #5128 은 카탈로그에 남긴다.",
                "ole/shape-component · char_shapes 는 다른 좌석.",
            ],
            "entries": history,
        },
    )
    write_json_file(
        LEDGER_DIR / "catalog_after_m05_6.json",
        dump_catalog(
            entries,
            notes=[
                "M05-6 이후 expected-fail. #4882 는 빠졌다.",
                "#4056 #5128 은 고치지 않았다.",
            ],
        ),
    )


def build_drift_reports() -> None:
    hwpx = REPO / SAMPLE_HWPX
    notes = extract_notes_from_section_xml(hwpx_section_xml(hwpx)) if hwpx.is_file() else []
    pre = analyze(
        doc=SAMPLE_HWP,
        before=215,
        after=223,
        notes=notes,
        ir_diffs=[f"{p} linesegs: {d}" for p, d in zip(PINNED_4882_PATHS, PINNED_4882_IR)],
        issue=ISSUE_4882,
    )
    post = analyze(
        doc=SAMPLE_HWP,
        before=215,
        after=215,
        notes=notes,
        ir_diffs=[],
        issue=ISSUE_4882,
    )
    write_json_file(FIXTURE_DIR / "drift_pre_fix.json", pre.to_json())
    write_json_file(FIXTURE_DIR / "drift_post_fix.json", post.to_json())


def main() -> int:
    FIXTURE_DIR.mkdir(parents=True, exist_ok=True)
    stats = build_note_corpus()
    build_measured_report(stats)
    build_transcripts()
    build_ledger()
    build_drift_reports()
    print(
        json.dumps(
            {
                "ok": True,
                "notes": stats["notes"],
                "zeroVpos": stats["zero"],
                "fixtureDir": str(FIXTURE_DIR.relative_to(REPO)),
            },
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
