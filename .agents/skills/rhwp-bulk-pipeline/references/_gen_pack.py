#!/usr/bin/env python3
"""[#5311] rhwp-bulk-pipeline 레퍼런스·픽스처·예제 생성기.

새 CLI 를 발명하지 않는다. 명령·봉투·종료 코드는 cli_commands.md §batch,
레시피 9, 레시피 5, cli_json_pipeline_guide.md, 기존 계약
(batch_axes / batch_extract_data / batch_fill / batch_parallel_determinism)
이 이미 고정한 표면만 복제한다.
"""

from __future__ import annotations

import json
from pathlib import Path

SKILL = Path(__file__).resolve().parents[1]
REF = SKILL / "references"
FIXT = SKILL / "fixtures"
EX = SKILL / "examples"
TRANS = EX / "transcripts"
LISTS = EX / "lists"
DATA = EX / "data"

ISSUE = 5311
SCHEMA = "1.0"

SAMPLES = {
    "plan2022": {
        "path": "samples/2022년 국립국어원 업무계획.hwp",
        "format": "hwp5",
        "pageCount": 35,
        "paraCount": 630,
        "title": "2022년 국립국어원 업무계획",
        "counts": {"amount": 65, "date": 29, "number": 203},
        "totalItemCount": 297,
        "note": "레시피 9 실측. info/export-text/extract-data 첫 행.",
    },
    "trade": {
        "path": "samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp",
        "format": "hwp5",
        "pageCount": 19,
        "counts": {"amount": 0, "date": 22, "number": 124},
        "totalItemCount": 146,
        "note": "레시피 9 실측. 금액 키 0 은 '없다'가 아니라 kind=all 에서 금액 미검출.",
    },
    "field01": {
        "path": "samples/field-01.hwp",
        "format": "hwp5",
        "pageCount": 3,
        "fieldCount": 11,
        "counts": {"amount": 0, "date": 0, "number": 0},
        "totalItemCount": 0,
        "note": "누름틀 11. extract-data 0건은 오류가 아니다.",
    },
    "hwp3": {
        "path": "samples/hwp3-sample.hwp",
        "format": "hwp3",
        "pageCount": 16,
        "fieldCount": 0,
        "counts": {"amount": 0, "date": 0, "number": 11},
        "totalItemCount": 11,
        "note": "HWP3 표본. fields 0 은 축 전환 신호.",
    },
    "missing": {
        "path": "samples/없는파일.hwp",
        "missing": True,
        "error": "문서를 열 수 없습니다: 지정된 파일을 찾을 수 없습니다. (os error 2)",
        "exitClass": "runtime",
        "note": "레시피 9 가 일부러 섞은 없는 파일. 실패 봉투 원형.",
    },
    "form01": {
        "path": "samples/form-01.hwp",
        "format": "hwp5",
        "fieldCount": 1,
        "names": ["myMsg01"],
        "note": "메일머지 서식 최소 표본.",
    },
    "table001": {
        "path": "samples/table-001.hwp",
        "format": "hwp5",
        "hasMerge": True,
        "note": "batch_axes_contract 표 병합 표본.",
    },
    "handbook": {
        "path": "samples/2025 행정업무운영 편람(최종).hwpx",
        "format": "hwpx",
        "pageCount": 387,
        "convertBytes": 9083392,
        "convertMs": 428,
        "note": "레시피 9 convert 실측. 387쪽 / 428ms / 9_083_392 bytes.",
    },
    "handbookHwp": {
        "path": "samples/2025 행정업무운영 편람(최종).hwp",
        "format": "hwp5",
        "note": "extract-data 계약 오라클과 같은 편람 HWP5.",
    },
}

AXES = [
    {
        "id": "info",
        "cmd": "batch info",
        "stdin": True,
        "flags": ["--json", "--threads"],
        "successKeys": ["schemaVersion", "source", "format", "pageCount"],
        "sameAs": "info --json",
        "purpose": "메타 스윕. 본문보다 싸다(271건 실측 3.0s vs export-text 67.4s).",
    },
    {
        "id": "export-text",
        "cmd": "batch export-text",
        "stdin": True,
        "flags": ["--json", "--threads"],
        "successKeys": ["schemaVersion", "source", "pageCount", "text"],
        "sameAs": "export-text --json 의 문서 단위 축약(pages[] 대신 text)",
        "purpose": "본문 일괄. 페이지 단위가 필요하면 선별 후 단건.",
    },
    {
        "id": "export-structure",
        "cmd": "batch export-structure",
        "stdin": True,
        "flags": ["--json", "--threads", "--mode"],
        "successKeys": ["schemaVersion", "source", "mode"],
        "sameAs": "export-structure --json",
        "purpose": "개요/조문 일괄. --mode auto|outline|clause.",
    },
    {
        "id": "export-tables",
        "cmd": "batch export-tables",
        "stdin": True,
        "flags": ["--json", "--threads"],
        "successKeys": ["schemaVersion", "source", "tableCount", "tables"],
        "sameAs": "export-tables --json",
        "purpose": "격자 JSON. 병합 rowSpan/colSpan 보존.",
    },
    {
        "id": "fields",
        "cmd": "batch fields",
        "stdin": True,
        "flags": ["--json", "--threads"],
        "successKeys": ["schemaVersion", "source", "fieldCount", "fields"],
        "sameAs": "fields --json",
        "purpose": "서식 템플릿 일괄 조사. fieldCount 0 은 오류가 아님.",
    },
    {
        "id": "search",
        "cmd": "batch search",
        "stdin": True,
        "flags": ["--json", "--threads", "--query"],
        "successKeys": ["schemaVersion", "source", "query", "matchCount", "matches"],
        "sameAs": "search --json",
        "purpose": "아카이브 전역 검색. --query 필수, 파일당 1000건 상한.",
    },
    {
        "id": "extract-data",
        "cmd": "batch extract-data",
        "stdin": True,
        "flags": ["--json", "--threads", "--kind", "--limit"],
        "successKeys": [
            "schemaVersion",
            "source",
            "kind",
            "itemCount",
            "totalItemCount",
            "truncated",
            "counts",
            "items",
        ],
        "sameAs": "extract-data --json",
        "purpose": "날짜·금액·수량. --limit 은 문서마다.",
    },
    {
        "id": "convert",
        "cmd": "batch convert",
        "stdin": True,
        "flags": ["--json", "--threads", "--out-dir", "--verify", "--verify-pages"],
        "successKeys": ["schemaVersion", "source", "format", "output", "bytes"],
        "sameAs": "convert --json",
        "purpose": "HWP5 일괄 변환. 이름 예약. CLI 전용(MCP 제외).",
    },
    {
        "id": "fill",
        "cmd": "batch fill",
        "stdin": False,
        "flags": [
            "--json",
            "--threads",
            "--form",
            "--data",
            "--out-dir",
            "--name-field",
            "--verify",
            "--dry-run",
        ],
        "successKeys": [
            "schemaVersion",
            "source",
            "row",
            "dryRun",
            "filledCount",
            "filled",
            "notFound",
            "ambiguous",
        ],
        "sameAs": "edit fill-fields --json + row",
        "purpose": "서식 1 + 데이터 N. stdin 목록이 아니다.",
    },
]

STOP_RULES = [
    ("B01", "목록이 비었거나 경로가 디렉터리", "find/Get-ChildItem 부터. 본작업 금지"),
    ("B02", "info 전건 error", "작업 디렉터리·상대경로 확인"),
    ("B03", "batch 에 --password 계열", "exit 2. 단건 명령으로 분리"),
    ("B04", "질문이 규모/형식뿐", "info 에서 정지"),
    ("B05", "export-text error 행", "jq 로 실패만 재시도"),
    ("B06", "export-structure --mode 오타", "exit 2. auto|outline|clause"),
    ("B07", "tableCount 0", "실패 아님. 표 없는 문서"),
    ("B08", "fieldCount 0", "누름틀 없음. table-exchange 후보"),
    ("B09", "search --query 없음", "exit 2. 입력 미소비"),
    ("B10", "extract-data truncated", "문서마다 한도. counts 는 절단 전"),
    ("B11", "convert 이름 충돌", "exit 2, 한 파일도 안 씀"),
    ("B12", "fill 에 stdin 목록", "--form + --data 로 다시"),
    ("B13", "N ≠ 성공+실패", "파이프 중간(head/grep) 의심"),
    ("B14", "exit 1 인데 실패 행 안 보임", "stderr 요약을 stdout 과 섞지 말 것"),
    ("B15", "verify IR 차이", "exit 3. 산출은 남음"),
    ("B16", "verify-pages 불일치", "exit 4 (error 없을 때)"),
    ("B17", "질문이 이미 답", "다음 축으로 내려가지 않음"),
    ("B18", "--out-dir 가 - 로 시작", "./-결과 로 명시"),
]

EXIT_ROWS = [
    {"code": 0, "when": "전부 통과", "example": "성공 5, 실패 0, verify 없음"},
    {"code": 1, "when": "error 레코드가 하나라도", "example": "성공 4 + 실패 1 (레시피 9)"},
    {"code": 2, "when": "사용법", "example": "--password, --query 누락, 이름 예약 충돌, 빈 CSV"},
    {"code": 3, "when": "error 없고 --verify IR 차이만", "example": "convert/fill --verify"},
    {"code": 4, "when": "error 없고 --verify-pages 불일치", "example": "convert --verify-pages"},
]


def dump(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(obj, ensure_ascii=False, indent=2)
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text, encoding="utf-8", newline="\n")


def write_md(path: Path, body: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not body.endswith("\n"):
        body += "\n"
    path.write_text(body.replace("\r\n", "\n"), encoding="utf-8", newline="\n")


def meta(extra=None):
    obj = {"schemaVersion": SCHEMA, "issue": ISSUE, "skill": "rhwp-bulk-pipeline", "notGym": True, "noNewCli": True}
    if extra:
        obj.update(extra)
    return obj


def recipe9_info_rows():
    rows = []
    for key in ("plan2022", "trade", "field01", "hwp3"):
        s = SAMPLES[key]
        rec = {
            "schemaVersion": SCHEMA,
            "source": s["path"],
            "format": s["format"],
            "pageCount": s["pageCount"],
        }
        if "paraCount" in s:
            rec["paraCount"] = s["paraCount"]
        if "title" in s:
            rec["title"] = s["title"]
        rec["untrustedContent"] = False
        rec["untrustedFields"] = []
        rows.append(rec)
    rows.append(failure_record(SAMPLES["missing"]["path"], SAMPLES["missing"]["error"]))
    return rows


def recipe9_text_rows():
    rows = []
    snippets = {
        "plan2022": " \n \n\n\n\n2022년 국립국어원 업무계획\n…",
        "trade": "2024년 5월 월간 수출입 현황(확정치)\n…",
        "field01": "필드 서식 본문\n…",
        "hwp3": "HWP3 표본 본문\n…",
    }
    for key in ("plan2022", "trade", "field01", "hwp3"):
        s = SAMPLES[key]
        rows.append(
            {
                "schemaVersion": SCHEMA,
                "source": s["path"],
                "pageCount": s["pageCount"],
                "text": snippets[key],
            }
        )
    rows.append(failure_record(SAMPLES["missing"]["path"], SAMPLES["missing"]["error"]))
    return rows


def recipe9_extract_rows(limit=None):
    rows = []
    for key in ("plan2022", "trade", "field01", "hwp3"):
        s = SAMPLES[key]
        total = s["totalItemCount"]
        item_count = min(limit, total) if limit is not None else total
        rows.append(
            {
                "schemaVersion": SCHEMA,
                "source": s["path"],
                "kind": "all",
                "itemCount": item_count,
                "totalItemCount": total,
                "truncated": bool(limit is not None and total > limit),
                "counts": s["counts"],
                "items": [],
            }
        )
    rows.append(failure_record(SAMPLES["missing"]["path"], SAMPLES["missing"]["error"]))
    return rows


def failure_record(source: str, error: str, **extra):
    rec = {
        "error": error,
        "exitClass": "runtime",
        "schemaVersion": SCHEMA,
        "source": source,
        "untrustedContent": False,
        "untrustedFields": [],
    }
    rec.update(extra)
    return rec


def ndjson_text(rows) -> str:
    return "".join(json.dumps(r, ensure_ascii=False, separators=(",", ":")) + "\n" for r in rows)


# ---------------------------------------------------------------------------
# fixtures
# ---------------------------------------------------------------------------


def build_skill_index():
    refs = [
        "00_tree.md",
        "01_stdin_ndjson.md",
        "02_failure_envelope.md",
        "03_order_threads.md",
        "04_axis_info.md",
        "05_axis_export_text.md",
        "06_axis_export_structure.md",
        "07_axis_export_tables.md",
        "08_axis_fields.md",
        "09_axis_search.md",
        "10_axis_extract_data.md",
        "11_axis_convert.md",
        "12_axis_fill.md",
        "13_jq_split_retry.md",
        "14_gate_n_equals.md",
        "15_no_global_password.md",
        "16_convert_name_reservation.md",
        "17_fill_not_stdin.md",
        "18_exit_aggregation.md",
        "19_pitfalls.md",
        "20_handoff.md",
        "21_journeys.md",
        "22_intent_matrix.md",
        "23_envelopes.md",
        "24_stderr_summary.md",
        "25_listing.md",
        "26_worked_traces.md",
        "27_gate_recipes.md",
        "28_retry_classes.md",
        "29_windows_powershell.md",
        "30_corpus.md",
        "31_folder_menu.md",
        "README.md",
    ]
    examples = [
        "01_list_then_info.md",
        "02_export_text_retry.md",
        "03_extract_data_harvest.md",
        "04_convert_reserve.md",
        "05_fill_mailmerge.md",
        "06_search_archive.md",
        "07_structure_outline.md",
        "08_tables_harvest.md",
        "09_fields_survey.md",
        "10_mixed_failure_gate.md",
        "11_password_split.md",
        "12_windows_listing.md",
    ]
    return meta(
        {
            "references": refs,
            "examples": examples,
            "forbiddenSkillsTouch": [
                "rhwp-onboarding",
                "rhwp-mcp-session",
                "rhwp-safe-edit",
                "rhwp-provenance",
                "rhwp-doc-triage",
                "rhwp-form-fill",
            ],
            "forbiddenTrees": ["gym/"],
            "coreTopics": [
                "stdin one-path-per-line",
                "stdout pure NDJSON",
                "stderr human summary",
                "failure-as-envelope",
                "order preserved under --threads",
                "jq split success vs error",
                "retry only failed rows",
                "gate input N = success + failure",
                "no global --password",
                "convert name reservation",
                "fill is form+data",
                "exit aggregation 0/1/2/3/4",
            ],
            "axes": [a["id"] for a in AXES],
            "authority": [
                "mydocs/manual/cli_commands.md",
                "mydocs/manual/cli_json_pipeline_guide.md",
                "mydocs/manual/recipes/09_bulk_extract_convert.md",
                "mydocs/manual/recipes/05_mail_merge_batch_fill.md",
            ],
        }
    )


def build_tree():
    return meta(
        {
            "coreReuse": [
                "batch info",
                "batch export-text",
                "batch export-structure",
                "batch export-tables",
                "batch fields",
                "batch search",
                "batch extract-data",
                "batch convert",
                "batch fill",
            ],
            "steps": [
                "list",
                "batch info",
                "select axis",
                "jq split",
                "retry failed",
                "gate N",
            ],
            "fillIsNotStdinList": True,
            "passwordRejected": True,
            "convertReservesNames": True,
        }
    )


def build_stop_rules():
    return meta(
        {
            "rules": [
                {"id": i, "when": w, "action": a, "notGym": True} for i, w, a in STOP_RULES
            ]
        }
    )


def build_axes():
    return meta({"axes": AXES})


def build_envelopes():
    return meta(
        {
            "failure": {
                "required": ["schemaVersion", "source", "error", "exitClass"],
                "exitClass": "runtime",
                "example": failure_record(
                    SAMPLES["missing"]["path"], SAMPLES["missing"]["error"]
                ),
            },
            "successByAxis": {a["id"]: a["successKeys"] for a in AXES},
            "isomorphicToSingle": {a["id"]: a["sameAs"] for a in AXES},
            "fillAdds": ["row"],
        }
    )


def build_exit_codes():
    return meta(
        {
            "aggregation": EXIT_ROWS,
            "usageDoesNotConsumeStdin": [
                "batch --password",
                "batch --password-stdin",
                "batch --output-password",
                "batch --output-password-stdin",
                "batch search without --query",
                "batch convert name collision",
                "batch convert --out-dir starting with dash flag",
                "batch fill empty header-only csv",
            ],
            "priority": [
                "usage(2) before any record",
                "any error record -> 1 (overrides 3/4)",
                "verify-pages 4 over verify 3 when both set and no error",
                "all ok -> 0",
            ],
        }
    )


def build_gate():
    return meta(
        {
            "formula": "inputN == success + failure",
            "jqSuccess": "select(.error|not)",
            "jqFailure": "select(.error)",
            "recipe9": {"input": 5, "success": 4, "failure": 1, "exit": 1},
            "evaporationSuspects": ["head", "grep", "Select-String", "broken pipe", "encoding"],
        }
    )


def journey_catalog():
    items = []

    def add(jid, title, steps, stop, sample="plan2022", axis=None):
        items.append(
            {
                "id": jid,
                "title": title,
                "steps": steps,
                "stop": stop,
                "sample": sample,
                "axis": axis,
                "notGym": True,
            }
        )

    add("J01", "목록만 만들고 멈춤", ["list"], "B17")
    add("J02", "빈 폴더", ["list"], "B01")
    add("J03", "info 선점검만", ["list", "batch info"], "B04")
    add("J04", "info 전건 실패", ["list", "batch info"], "B02", sample="missing")
    add("J05", "암호 플래그 거부", ["batch info --password"], "B03")
    add("J06", "본문 추출 전건 성공", ["list", "info", "export-text", "gate"], "B17", axis="export-text")
    add("J07", "본문 추출 혼합 실패", ["list", "export-text", "jq split", "retry"], "B05", sample="missing", axis="export-text")
    add("J08", "개요 일괄 auto", ["list", "export-structure --mode auto"], "B17", axis="export-structure")
    add("J09", "조문 일괄 clause", ["list", "export-structure --mode clause"], "B17", axis="export-structure")
    add("J10", "mode 오타", ["export-structure --mode chapters"], "B06", axis="export-structure")
    add("J11", "표 수확", ["list", "export-tables"], "B07", sample="table001", axis="export-tables")
    add("J12", "표 없는 문서", ["export-tables"], "B07", sample="hwp3", axis="export-tables")
    add("J13", "서식 조사", ["list", "fields"], "B08", sample="field01", axis="fields")
    add("J14", "누름틀 0", ["fields"], "B08", sample="hwp3", axis="fields")
    add("J15", "전역 검색", ["list", "search --query 위임전결"], "B17", axis="search")
    add("J16", "검색어 누락", ["search --json"], "B09", axis="search")
    add("J17", "날짜 금액 수확", ["list", "extract-data --limit 3"], "B10", axis="extract-data")
    add("J18", "kind=amount 만", ["extract-data --kind amount"], "B17", sample="plan2022", axis="extract-data")
    add("J19", "limit 절단", ["extract-data --limit 3"], "B10", sample="plan2022", axis="extract-data")
    add("J20", "convert 성공", ["list", "convert --out-dir out"], "B17", sample="handbook", axis="convert")
    add("J21", "convert 이름 충돌", ["convert --out-dir out"], "B11", axis="convert")
    add("J22", "convert verify", ["convert --verify"], "B15", axis="convert")
    add("J23", "convert verify-pages", ["convert --verify-pages"], "B16", axis="convert")
    add("J24", "메일머지", ["fields", "batch fill --form --data --out-dir"], "B17", sample="form01", axis="fill")
    add("J25", "fill 에 stdin 목록", ["printf paths | batch fill"], "B12", axis="fill")
    add("J26", "fill dry-run", ["batch fill --dry-run"], "B17", sample="form01", axis="fill")
    add("J27", "fill 빈 CSV", ["batch fill --data empty.csv"], "B12", sample="form01", axis="fill")
    add("J28", "게이트 통과 5=4+1", ["export-text", "gate"], "B17", sample="missing")
    add("J29", "게이트 실패 증발", ["export-text | head", "gate"], "B13")
    add("J30", "stderr 요약만 읽고 행 무시", ["export-text"], "B14")
    add("J31", "out-dir 대시", ["convert --out-dir -결과"], "B18", axis="convert")
    add("J32", "선별 후 추출", ["info", "jq pageCount>=10", "export-text"], "B17")
    add("J33", "암호 문서 분리", ["info", "단건 --password"], "B03")
    add("J34", "Windows 목록", ["Get-ChildItem", "info"], "B17")
    add("J35", "threads 8 순서 보존", ["export-text --threads 8"], "B17", axis="export-text")
    add("J36", "재시도 후 게이트", ["export-text", "retry", "concat", "gate"], "B05")
    add("J37", "검색 매치 0", ["search --query ZZNOHIT"], "B17", axis="search")
    add("J38", "fields 후 form-fill 인계", ["fields"], "B08", sample="field01")
    add("J39", "tables 후 table-exchange 인계", ["export-tables"], "B07", sample="table001")
    add("J40", "질문이 검색만", ["search --query 위임전결"], "B17", axis="search")

    titles = [
        ("폴더 스윕 후 10쪽 이상만 본문", ["info", "jq", "export-text"], "B05", "export-text"),
        ("HWPX 만 convert", ["list hwpx", "convert"], "B17", "convert"),
        ("HWP3 와 HWP5 혼합 info", ["info"], "B04", "info"),
        ("편람 금액만 수확", ["extract-data --kind amount"], "B10", "extract-data"),
        ("편람 날짜만 수확", ["extract-data --kind date"], "B17", "extract-data"),
        ("편람 수량만 수확", ["extract-data --kind number"], "B17", "extract-data"),
        ("검색 후 매치 문서만 본문", ["search", "jq matchCount>0", "export-text"], "B05", "search"),
        ("서식 있는 파일만 fill 후보", ["fields", "jq fieldCount>0"], "B08", "fields"),
        ("표 있는 파일만 수확", ["export-tables", "jq tableCount>0"], "B07", "export-tables"),
        ("convert 후 verify 게이트", ["convert --verify --verify-pages"], "B16", "convert"),
        ("fill name-field 성명", ["batch fill --name-field 성명"], "B17", "fill"),
        ("fill 이름 겹침 _2", ["batch fill"], "B17", "fill"),
        ("fill verify 행별", ["batch fill --verify"], "B15", "fill"),
        ("PowerShell Get-Content 파이프", ["Get-Content 목록.txt | rhwp batch info --json"], "B04", "info"),
        ("UTF-8 경로 한글", ["list", "info"], "B04", "info"),
        ("상대경로 vs 절대경로", ["info"], "B02", "info"),
        ("같은 파일을 stdin 두 번", ["extract-data --limit 3"], "B10", "extract-data"),
        ("MCP 로 convert 시도", ["hwp_batch convert"], "B17", "convert"),
        ("단건 triage 후 폴더로 확대", ["doc-triage", "list", "info"], "B04", "info"),
        ("보안 스윕 후 본문", ["security-sweep", "export-text"], "B05", "export-text"),
        ("작업 영수증으로 배치 증빙", ["export-text", "replay"], "B17", "export-text"),
        ("jq 로 실패 경로만 수정", ["export-text", "jq error", "rewrite list"], "B05", "export-text"),
        ("os error 2 부류는 재시도 금지", ["export-text"], "B05", "export-text"),
        ("암호 문서는 재시도하지 않고 분리", ["info"], "B03", "info"),
        ("panic 행도 봉투", ["export-text"], "B05", "export-text"),
        ("broken pipe 후 게이트 실패", ["export-text | head -1"], "B13", "export-text"),
        ("NDJSON 을 JSON 배열로 오파싱", ["jq without -s"], "B14", None),
        ("stderr 를 결과 파일에 리다이렉트", ["2>&1"], "B14", None),
        ("성공 행 pageCount 집계", ["info", "jq add"], "B04", "info"),
        ("검색 대소문자 구분", ["search --query Hwp"], "B17", "search"),
        ("structure outline 만", ["export-structure --mode outline"], "B17", "export-structure"),
        ("convert out-dir 필수 누락", ["convert --json"], "B18", "convert"),
        ("fill out-dir 필수 누락", ["batch fill --form --data"], "B12", "fill"),
        ("fill dry-run 에도 out-dir", ["batch fill --dry-run --out-dir"], "B17", "fill"),
        ("threads 1 결정성", ["export-text --threads 1"], "B17", "export-text"),
        ("threads 기본 CPU", ["export-text"], "B17", "export-text"),
        ("목록에 빈 줄", ["list", "info"], "B01", "info"),
        ("목록에 주석 줄", ["list", "info"], "B02", "info"),
        ("PDF 혼입", ["list", "info"], "B02", "info"),
        ("디렉터리 경로 혼입", ["list", "info"], "B01", "info"),
        ("같은 이름 다른 폴더 convert", ["convert"], "B11", "convert"),
        ("Report.HWP 와 report.hwp", ["convert"], "B11", "convert"),
        ("메일머지 12행", ["batch fill"], "B17", "fill"),
        ("메일머지 notFound 행", ["batch fill"], "B15", "fill"),
        ("메일머지 ambiguous 행", ["batch fill"], "B15", "fill"),
        ("info 후 질문 종료", ["info"], "B17", "info"),
        ("검색 0건을 실패로 오독 금지", ["search"], "B17", "search"),
        ("extract-data 0건을 실패로 오독 금지", ["extract-data"], "B17", "extract-data"),
        ("fields 0건을 실패로 오독 금지", ["fields"], "B08", "fields"),
        ("exit 1 을 파이프 전체 실패로만 읽지 않기", ["export-text"], "B14", "export-text"),
    ]
    for i, (title, steps, stop, axis) in enumerate(titles, start=41):
        add(f"J{i:02d}", title, steps, stop, axis=axis)

    assert len(items) >= 80, len(items)
    return meta({"count": len(items), "journeys": items})


def intent_catalog():
    pairs = [
        ("폴더 문서들 몇 쪽이야", "batch info --json", "B04"),
        ("형식부터 훑어줘", "batch info --json", "B04"),
        ("본문 전부 뽑아", "batch export-text --json", "B05"),
        ("텍스트로 한꺼번에", "batch export-text --json", "B05"),
        ("개요만 일괄", "batch export-structure --json --mode outline", "B17"),
        ("조문 구조 뽑아", "batch export-structure --json --mode clause", "B17"),
        ("표 전부 CSV 말고 JSON 으로", "batch export-tables --json", "B07"),
        ("서식에 누름틀 있는 파일만", "batch fields --json", "B08"),
        ("위임전결 어디 있어", "batch search --query 위임전결 --json", "B17"),
        ("아카이브 전역 검색", "batch search --query <q> --json", "B09"),
        ("날짜 금액 수확", "batch extract-data --json", "B10"),
        ("금액만", "batch extract-data --json --kind amount", "B17"),
        ("너무 많으니 문서당 3개만", "batch extract-data --json --limit 3", "B10"),
        ("HWPX 를 HWP5 로 일괄", "batch convert --out-dir <dir> --json", "B11"),
        ("변환하고 검증까지", "batch convert --out-dir <dir> --verify --verify-pages --json", "B16"),
        ("신청서에 명단 채워", "batch fill --form --data --out-dir --json", "B12"),
        ("메일머지", "batch fill --form --data --out-dir --json", "B12"),
        ("미리 채움만 확인", "batch fill --dry-run --form --data --out-dir --json", "B17"),
        ("실패만 다시", "jq select(.error) | batch <same axis>", "B05"),
        ("숫자가 맞아?", "gate input N = success + failure", "B13"),
        ("비밀번호 넣어서 배치", "거부. 단건 --password", "B03"),
        ("batch --password", "exit 2", "B03"),
        ("검색어 없이 검색", "exit 2", "B09"),
        ("같은 이름 두 파일 변환", "이름 예약 충돌 exit 2", "B11"),
        ("폴더 전체를 텍스트로", "batch export-text --json", "B05"),
        ("한꺼번에 변환", "batch convert --out-dir --json", "B11"),
        ("여러 hwp 대량 처리", "batch info 후 축 선택", "B04"),
        ("코퍼스 추출", "batch export-text --json", "B05"),
        ("서식 하나에 여러 명", "batch fill", "B12"),
        ("rhwp batch", "축을 물어보고 진행", "B17"),
        ("threads 높여", "batch <axis> --threads N --json", "B17"),
        ("순서가 뒤섞이면 안 돼", "--threads 해도 입력 순서 보존", "B17"),
        ("stderr 요약 어디", "사람용. 파이프에 태우지 말 것", "B14"),
        ("NDJSON 이 아니라 JSON 배열로", "jq -s 는 게이트에서만", "B14"),
        ("없는 파일 섞여 있어", "실패 봉투. exit 1 정상", "B05"),
        ("표 병합 유지해", "batch export-tables (markdown 금지)", "B07"),
        ("누름틀 없는 문서 채워", "이 스킬 아님. form-fill/table-exchange", "B08"),
        ("한 문서만 파악", "rhwp-doc-triage", "B17"),
        ("배포 전 점검", "rhwp-security-sweep", "B17"),
        ("MCP 로 배치 변환", "convert 는 CLI 전용", "B11"),
        ("Windows 에서 목록", "Get-ChildItem ... | rhwp batch", "B17"),
        ("CP949 명단", "UTF-8 로 재저장", "B12"),
        ("-결과 폴더에 변환", "./-결과", "B18"),
        ("verify 실패면 파일 없나", "산출은 남음. exit 3", "B15"),
        ("페이지 수 불일치", "exit 4", "B16"),
        ("성공만 세면 되지", "실패를 지우면 N 게이트가 깨짐", "B13"),
        ("head 로 미리보기", "게이트 전에 쓰지 말 것", "B13"),
        ("grep error 한 줄", "요약을 지울 수 있음. jq 사용", "B14"),
        ("같은 문서 두 번 extract", "limit 은 문서마다 독립", "B10"),
        ("필드 조사 후 채움", "fields 배치 → 단건/fill 스킬", "B08"),
        ("조문 모드 기본?", "기본 auto", "B06"),
        ("search limit", "파일당 1000. 단건 --limit 과 같은 취지", "B17"),
        ("대소문자 무시 검색", "구분한다. 다른 쿼리를 두 번", "B17"),
        ("info 스키마가 단건과 같나", "같다. 같은 소비 코드", "B04"),
        ("fill 레코드에 row", "0 기준 행 번호", "B17"),
        ("name-field 생략", "1 기준 순번 최소 4자리", "B17"),
        ("이름 겹치면", "_2 접미", "B17"),
        ("파일명 금지 문자", "_ 치환", "B17"),
        ("dry-run 인데 out-dir 왜", "실행 줄에서 --dry-run 만 빼면 되도록", "B17"),
        ("빈 헤더 CSV", "exit 2", "B12"),
        ("데이터 파일 stdin", "안 됨. --data 파일", "B12"),
        ("목록을 인자로", "하지 말 것. stdin", "B01"),
        ("암호화 산출", "batch 미지원", "B03"),
        ("output-password", "exit 2", "B03"),
        ("password-stdin 배치", "exit 2", "B03"),
        ("성공 4 실패 1 코드는", "1", "B14"),
        ("전부 성공 코드는", "0", "B17"),
        ("사용법 오류 코드는", "2", "B03"),
        ("페이지 검증 코드는", "4", "B16"),
        ("IR 검증 코드는", "3", "B15"),
        ("파이프 중간에 jq select", "행 수 변함. 게이트는 원본 목록 기준", "B13"),
        ("재시도 결과를 원본에 덮어쓰기", "실패 행만 치환. 성공 유지", "B05"),
        ("원본 HWP 를 convert 가 덮나", "out-dir 로 분리. 원본 불변", "B11"),
        ("fill 이 서식을 덮나", "out-dir 산출. 서식 불변", "B12"),
        ("271건 스윕 시간", "info 3.0s (가이드 실측)", "B04"),
        ("271건 본문 시간", "export-text 67.4s (가이드 실측)", "B05"),
        ("10쪽 이상만", "info 후 jq pageCount>=10", "B04"),
        ("RAG 청킹", "배치 text 후 필요 문서만 단건 pages[]", "B05"),
        ("CI 무인", "exit 1 + jq 실패 행 보고", "B14"),
        ("손상 파일", "실패 봉투. 스트림 계속", "B05"),
        ("panic 격리", "exitClass runtime 레코드", "B05"),
        ("스키마 필드 추가", "허용. 삭제·변경은 cli_json_contract", "B17"),
        ("단건 실패 stdout", "0바이트. 배치와 다름", "B14"),
        ("배치 실패 stdout", "실패 레코드 1줄", "B14"),
        ("capabilities batch", "단일 출처", "B17"),
        ("extract-data 가 §batch 목록에 없다", "capabilities + 레시피 9가 근거", "B10"),
        ("hwp_batch 도구", "읽기 축. convert 쓰기 제외", "B11"),
        ("gym 과제 만들까", "금지", "B17"),
        ("새 batch merge 명령", "발명 금지. fill 사용", "B12"),
        ("batch export-markdown", "없음. 단건 export-markdown", "B17"),
        ("batch thumbnail", "없음", "B17"),
        ("batch redact", "없음. security-sweep", "B17"),
        ("폴더를 인자로", "stdin 목록으로 바꿔", "B01"),
        ("*.hwp 글롭을 batch 에", "쉘이 펼치면 인자 한계. 목록 파일", "B01"),
        ("병렬이라 순서가 달라도", "아님. 입력 순서 보존", "B17"),
        ("CPU 기본 스레드", "코어 수", "B17"),
        ("threads 0", "사용법 확인. 추측 금지", "B06"),
        ("limit 을 배치 전체 상한으로", "구현 오류. 문서마다", "B10"),
        ("counts 가 limit 과 같다", "아님. counts 는 절단 전", "B10"),
        ("truncated 없이 자름", "계약 위반. truncated:true 필수", "B10"),
        ("금액 0 키 생략?", "kind=all 실측은 키를 넣기도. 단건 명문은 미요청 키 생략", "B10"),
        ("정규화 실패", "normalized null. raw 만 신뢰", "B10"),
        ("표 자동번호", "export-tables 한계. 빈 자리", "B07"),
        ("1x1 래퍼 표", "표로 잡힘. 소비자가 필터", "B07"),
        ("머리말 안 표", "export-tables 재귀 수집. info 는 놓칠 수 있음", "B07"),
        ("검색 매치 1000 초과", "잘림. 단건 search --limit 과 같은 취지", "B17"),
        ("convert 출력 이름 규칙", "<out-dir>/<입력이름>.hwp", "B11"),
        ("HWP5 입력을 convert", "다시 HWP5 로 씀. 이름은 .hwp", "B11"),
        ("한 건도 안 써진 이유", "이름 예약 실패. 로그 stderr", "B11"),
        ("절반만 변환됨", "예약 규약이면 일어나면 안 됨", "B11"),
        ("Linux 와 Windows 이름", "대소문자 충돌을 동일하게 거부", "B11"),
        ("fill 행 실패도 남김", "다른 축과 같은 실패 스키마 + row", "B12"),
        ("서식 못 열면", "시작 전 한 번만 판정", "B12"),
        ("명단 80행", "산출 80. 레코드 80", "B12"),
        ("성명으로 파일명", "--name-field 성명", "B17"),
        ("제출 정리", "이 스킬 아님. form-fill sanitize", "B17"),
        ("원본 in-place", "하지 말 것", "B17"),
        ("목록 인코딩", "UTF-8. PowerShell Out-File -Encoding utf8", "B01"),
        ("BOM 목록", "첫 경로가 깨질 수 있음", "B01"),
        ("CRLF 목록", "허용. 한 줄 = 한 경로", "B01"),
        ("공백 있는 경로", "따옴표 없이 한 줄 전체", "B01"),
        ("네트워크 경로", "os error 가능. 실패 봉투", "B05"),
        ("잠긴 파일", "런타임 실패 봉투", "B05"),
        ("권한 거부", "런타임 실패 봉투", "B05"),
        ("디스크 가득", "쓰기 축 런타임. 읽기 축은 성공 가능", "B11"),
        ("같은 목록으로 여러 축", "info → 본작업. 목록 재사용", "B04"),
        ("축을 한 프로세스에 섞기", "안 됨. 호출을 나눔", "B17"),
        ("NDJSON 이어 붙이기", "재시도 결과를 실패 자리에 치환", "B05"),
        ("성공 파일을 다시 돌리기", "낭비. 실패만", "B05"),
        ("게이트를 wc -l 결과만", "NDJSON 줄 수와 목록 줄 수를 같이", "B13"),
        ("jq -s 메모리", "대량이면 스트리밍 카운트", "B13"),
        ("Python 으로 게이트", "error 키 유무로 분기", "B13"),
        ("PowerShell ConvertFrom-Json", "한 줄씩", "B14"),
        ("Select-String error", "본문에 error 단어가 있는 성공 행을 오탐", "B14"),
        ("실패 메시지 한글", "os error 2 실측 문구 유지", "B05"),
        ("untrustedContent", "실패 봉투에도 출처 표지 필드가 실림", "B14"),
        ("schemaVersion 1.0", "계약", "B17"),
        ("필드 추가만 허용", "삭제 금지", "B17"),
        ("이슈 번호", "5311", "B17"),
        ("gym 에서 돌려", "금지. 실 에이전트 경로", "B17"),
    ]
    intents = []
    for i, (utter, cmd, stop) in enumerate(pairs, start=1):
        intents.append(
            {
                "id": f"I{i:03d}",
                "utterance": utter,
                "command": cmd,
                "stop": stop,
                "notGym": True,
            }
        )
    return meta({"count": len(intents), "intents": intents})


def build_pitfalls():
    items = [
        {"id": "P01", "trap": "batch --password", "correct": "exit 2. 단건 --password"},
        {"id": "P02", "trap": "convert 이름 충돌을 덮어쓰기", "correct": "한 파일도 안 쓰고 exit 2"},
        {"id": "P03", "trap": "fill 에 stdin 목록", "correct": "--form + --data"},
        {"id": "P04", "trap": "실패 행 삭제 후 성공만 저장", "correct": "게이트가 깨짐"},
        {"id": "P05", "trap": "limit 을 배치 전체 상한으로 해석", "correct": "문서마다"},
        {"id": "P06", "trap": "stderr 요약을 stdout 으로 파싱", "correct": "2> 분리"},
        {"id": "P07", "trap": "head 로 미리보고 게이트", "correct": "원본 목록 줄 수 기준"},
        {"id": "P08", "trap": "search 없이 --query", "correct": "exit 2"},
        {"id": "P09", "trap": "--out-dir -결과", "correct": "./-결과"},
        {"id": "P10", "trap": "tableCount 0 을 실패", "correct": "빈 표는 성공"},
        {"id": "P11", "trap": "fieldCount 0 을 실패", "correct": "축 전환 신호"},
        {"id": "P12", "trap": "itemCount 0 을 실패", "correct": "추출 0건은 exit 0"},
        {"id": "P13", "trap": "matchCount 0 을 실패", "correct": "검색 0은 성공"},
        {"id": "P14", "trap": "exit 1 이면 전부 실패", "correct": "행별 봉투"},
        {"id": "P15", "trap": "병렬이면 순서 뒤섞임", "correct": "입력 순서 보존"},
        {"id": "P16", "trap": "MCP 로 convert", "correct": "CLI 전용"},
        {"id": "P17", "trap": "CP949 --data", "correct": "UTF-8"},
        {"id": "P18", "trap": "counts == itemCount 항상", "correct": "절단 전이 counts"},
        {"id": "P19", "trap": "성공 행 재처리", "correct": "실패만 재시도"},
        {"id": "P20", "trap": "새 batch 서브커맨드 발명", "correct": "기존 9축만"},
        {"id": "P21", "trap": "gym pack 작성", "correct": "금지"},
        {"id": "P22", "trap": "목록을 argv 로", "correct": "stdin"},
        {"id": "P23", "trap": "2>&1 로 섞기", "correct": "NDJSON 오염"},
        {"id": "P24", "trap": "verify 실패면 산출 없음", "correct": "산출은 남고 exit 3"},
        {"id": "P25", "trap": "verify 와 verify-pages 코드 혼동", "correct": "3 vs 4"},
        {"id": "P26", "trap": "같은 이름 다른 대소문자 허용", "correct": "충돌"},
        {"id": "P27", "trap": "fill dry-run 에서 out-dir 생략", "correct": "필수"},
        {"id": "P28", "trap": "Select-String error", "correct": "본문 error 오탐. jq"},
        {"id": "P29", "trap": "단건 실패처럼 stdout 빈 줄 기대", "correct": "배치는 실패 레코드"},
        {"id": "P30", "trap": "extract-data 축이 없다 판단", "correct": "capabilities + 레시피 9"},
    ]
    return meta({"pitfalls": items})


def build_command_ladder():
    return meta(
        {
            "ladder": [
                {"step": 1, "action": "목록 작성", "cmd": "find / Get-ChildItem"},
                {"step": 2, "action": "선점검", "cmd": "batch info --json"},
                {"step": 3, "action": "본작업", "cmd": "batch <axis> --json"},
                {"step": 4, "action": "분리", "cmd": "jq select(.error)"},
                {"step": 5, "action": "재시도", "cmd": "실패 목록만 같은 축"},
                {"step": 6, "action": "게이트", "cmd": "N = success + failure"},
            ]
        }
    )


def build_password_reject():
    return meta(
        {
            "rejectedFlags": [
                "--password",
                "--password-stdin",
                "--output-password",
                "--output-password-stdin",
            ],
            "exit": 2,
            "consumesStdin": False,
            "splitRecipe": "암호 문서는 단건 info/export-text --password 로 빼고 나머지는 batch",
        }
    )


def build_convert_names():
    return meta(
        {
            "rule": "<out-dir>/<stem>.hwp",
            "reserveBeforeWrite": True,
            "caseCollisionIsError": True,
            "partialWrite": False,
            "exitOnCollision": 2,
            "mcpExcluded": True,
            "cases": [
                {"inputs": ["A.hwp", "B.hwpx"], "ok": True},
                {"inputs": ["A.hwp", "A.hwpx"], "ok": False, "reason": "same stem"},
                {"inputs": ["Report.HWP", "report.hwp"], "ok": False, "reason": "case"},
                {"inputs": ["dir1/x.hwp", "dir2/x.hwp"], "ok": False, "reason": "stem only"},
            ],
        }
    )


def build_fill_contract():
    return meta(
        {
            "stdinIsNotFileList": True,
            "required": ["--form", "--data", "--out-dir"],
            "dataFormats": [".jsonl", ".csv"],
            "dataEncoding": "UTF-8",
            "rowZeroBased": True,
            "nameFieldOptional": True,
            "defaultName": "1-based zero-padded min 4 digits",
            "dryRunStillNeedsOutDir": True,
            "emptyCsvExit": 2,
        }
    )


def build_samples_fixture():
    return meta({"samples": SAMPLES})


def build_recipe9_gate():
    return meta(
        {
            "list": [
                SAMPLES["plan2022"]["path"],
                SAMPLES["trade"]["path"],
                SAMPLES["field01"]["path"],
                SAMPLES["hwp3"]["path"],
                SAMPLES["missing"]["path"],
            ],
            "input": 5,
            "success": 4,
            "failure": 1,
            "exit": 1,
            "measured": True,
            "source": "mydocs/manual/recipes/09_bulk_extract_convert.md",
        }
    )


TRACE_SPECS = [
    ("T01", "info 5=4+1", "batch info --json", "info", 5, 4, 1, 1, "B05"),
    ("T02", "export-text 5=4+1", "batch export-text --json --threads 4", "export-text", 5, 4, 1, 1, "B05"),
    ("T03", "extract-data --limit 3", "batch extract-data --json --limit 3", "extract-data", 5, 4, 1, 1, "B10"),
    ("T04", "convert 편람 1건", "batch convert --out-dir out/bulk --json", "convert", 1, 1, 0, 0, "B17"),
    ("T05", "search --query 의", "batch search --query 의 --json", "search", 2, 2, 0, 0, "B17"),
    ("T06", "search 쿼리 없음", "batch search --json", "search", 0, 0, 0, 2, "B09"),
    ("T07", "password 거부", "batch info --password x --json", "info", 0, 0, 0, 2, "B03"),
    ("T08", "convert 이름 충돌", "batch convert --out-dir out --json", "convert", 0, 0, 0, 2, "B11"),
    ("T09", "fields 조사", "batch fields --json", "fields", 2, 2, 0, 0, "B08"),
    ("T10", "export-tables 병합", "batch export-tables --json", "export-tables", 1, 1, 0, 0, "B07"),
    ("T11", "export-structure auto", "batch export-structure --json --mode auto", "export-structure", 2, 2, 0, 0, "B17"),
    ("T12", "fill dry-run", "batch fill --form form.hwp --data rows.jsonl --out-dir out --dry-run --json", "fill", 3, 3, 0, 0, "B17"),
    ("T13", "fill 실행", "batch fill --form form.hwp --data rows.jsonl --out-dir out --name-field 성명 --json", "fill", 3, 3, 0, 0, "B17"),
    ("T14", "fill 빈 CSV", "batch fill --form form.hwp --data empty.csv --out-dir out --json", "fill", 0, 0, 0, 2, "B12"),
    ("T15", "fill stdin 오용", "printf paths | batch fill --json", "fill", 0, 0, 0, 2, "B12"),
    ("T16", "threads 8 순서", "batch export-text --json --threads 8", "export-text", 5, 4, 1, 1, "B05"),
    ("T17", "게이트 증발", "export-text | head", "export-text", 5, 1, 0, None, "B13"),
    ("T18", "verify 차이", "batch convert --out-dir out --verify --json", "convert", 1, 1, 0, 3, "B15"),
    ("T19", "verify-pages", "batch convert --out-dir out --verify-pages --json", "convert", 1, 1, 0, 4, "B16"),
    ("T20", "info 선별 75건", "batch info 후 jq pageCount>=10", "info", 270, 270, 0, 0, "B04"),
]


def more_trace_specs():
    extra = []
    axis_cycle = ["info", "export-text", "export-structure", "export-tables", "fields", "search", "extract-data", "convert", "fill"]
    stops = [s[0] for s in STOP_RULES]
    for i in range(21, 49):
        axis = axis_cycle[(i - 21) % len(axis_cycle)]
        stop = stops[(i - 21) % len(stops)]
        extra.append(
            (
                f"T{i:02d}",
                f"{axis} 여정 {i}",
                f"batch {axis} --json",
                axis,
                4 if axis != "fill" else 3,
                3,
                1 if axis != "fill" else 0,
                1 if axis != "fill" else 0,
                stop,
            )
        )
    return extra


def build_traces_index():
    specs = TRACE_SPECS
    traces = []
    for spec in specs:
        tid, title, cmd, axis, inp, ok, bad, exit_code, stop = spec
        traces.append(
            {
                "id": tid,
                "title": title,
                "command": cmd,
                "axis": axis,
                "input": inp,
                "success": ok,
                "failure": bad,
                "exit": exit_code,
                "stop": stop,
                "notGym": True,
                "transcript": f"examples/transcripts/{tid}.ndjson",
            }
        )
    return meta({"count": len(traces), "traces": traces})


def build_jq_recipes():
    return meta(
        {
            "recipes": [
                {
                    "id": "Q01",
                    "name": "실패 경로",
                    "jq": "jq -r 'select(.error) | .source'",
                },
                {
                    "id": "Q02",
                    "name": "성공 경로+쪽",
                    "jq": "jq -r 'select(.error|not) | \"\\(.source)\\t\\(.pageCount)쪽\"'",
                },
                {
                    "id": "Q03",
                    "name": "성공 수",
                    "jq": "jq -s '[.[]|select(.error|not)]|length'",
                },
                {
                    "id": "Q04",
                    "name": "실패 수",
                    "jq": "jq -s '[.[]|select(.error)]|length'",
                },
                {
                    "id": "Q05",
                    "name": "검색 히트만",
                    "jq": "jq -c 'select(.matchCount > 0) | {source, pages:[.matches[].page]}'",
                },
                {
                    "id": "Q06",
                    "name": "서식만",
                    "jq": "jq -c 'select(.fieldCount>0) | {source, fieldCount}'",
                },
                {
                    "id": "Q07",
                    "name": "표만",
                    "jq": "jq -c 'select(.tableCount>0) | {source, tableCount}'",
                },
                {
                    "id": "Q08",
                    "name": "10쪽 이상",
                    "jq": "jq -r 'select(.pageCount >= 10) | .source'",
                },
                {
                    "id": "Q09",
                    "name": "절단된 extract",
                    "jq": "jq -c 'select(.truncated==true) | {source, itemCount, totalItemCount}'",
                },
                {
                    "id": "Q10",
                    "name": "fill 실패 행",
                    "jq": "jq -c 'select((.notFound|length>0) or (.ambiguous|length>0) or .error)'",
                },
                {
                    "id": "Q11",
                    "name": "verify 불일치",
                    "jq": "jq -c 'select(.verify != null and .verify.identical==false)'",
                },
                {
                    "id": "Q12",
                    "name": "exitClass",
                    "jq": "jq -r 'select(.error) | [.source, .exitClass, .error] | @tsv'",
                },
            ]
        }
    )


def build_retry_classes():
    return meta(
        {
            "classes": [
                {
                    "id": "R-PATH",
                    "signal": "os error 2",
                    "retry": False,
                    "action": "목록 경로 수정",
                },
                {
                    "id": "R-PERM",
                    "signal": "os error 13 / Access is denied",
                    "retry": False,
                    "action": "권한·잠금 해제",
                },
                {
                    "id": "R-PASS",
                    "signal": "암호 / encrypted",
                    "retry": False,
                    "action": "단건 --password 로 분리. batch 플래그 금지",
                },
                {
                    "id": "R-PARSE",
                    "signal": "문서를 열 수 없습니다 (파서)",
                    "retry": False,
                    "action": "손상 파일. 코퍼스에서 격리",
                },
                {
                    "id": "R-TRANSIENT",
                    "signal": "일시적 IO / sharing violation",
                    "retry": True,
                    "action": "같은 축으로 실패 목록만 재시도",
                },
                {
                    "id": "R-USAGE",
                    "signal": "exit 2, 레코드 없음",
                    "retry": False,
                    "action": "플래그·이름 예약·쿼리부터 수정",
                },
                {
                    "id": "R-VERIFY",
                    "signal": "exit 3/4, error 키 없음",
                    "retry": False,
                    "action": "행별 verify 봉투. 재시도로 해결되지 않음",
                },
            ]
        }
    )


def write_fixtures():
    FIXT.mkdir(parents=True, exist_ok=True)
    dump(FIXT / "skill_index.json", build_skill_index())
    dump(FIXT / "tree.json", build_tree())
    dump(FIXT / "stop_rules.json", build_stop_rules())
    dump(FIXT / "axes.json", build_axes())
    dump(FIXT / "envelopes.json", build_envelopes())
    dump(FIXT / "exit_codes.json", build_exit_codes())
    dump(FIXT / "gate.json", build_gate())
    dump(FIXT / "journeys.json", journey_catalog())
    dump(FIXT / "intent_matrix.json", intent_catalog())
    dump(FIXT / "pitfalls.json", build_pitfalls())
    dump(FIXT / "command_ladder.json", build_command_ladder())
    dump(FIXT / "password_reject.json", build_password_reject())
    dump(FIXT / "convert_names.json", build_convert_names())
    dump(FIXT / "fill_contract.json", build_fill_contract())
    dump(FIXT / "samples.json", build_samples_fixture())
    dump(FIXT / "recipe9_gate.json", build_recipe9_gate())
    dump(FIXT / "traces_index.json", build_traces_index())
    dump(FIXT / "jq_recipes.json", build_jq_recipes())
    dump(FIXT / "retry_classes.json", build_retry_classes())
    dump(FIXT / "info_rows.json", meta({"rows": recipe9_info_rows()}))
    dump(FIXT / "export_text_rows.json", meta({"rows": recipe9_text_rows()}))
    dump(FIXT / "extract_rows.json", meta({"rows": recipe9_extract_rows(limit=3)}))
    dump(
        FIXT / "failure_os2.json",
        meta({"row": failure_record(SAMPLES["missing"]["path"], SAMPLES["missing"]["error"])}),
    )


def write_lists_and_data():
    LISTS.mkdir(parents=True, exist_ok=True)
    DATA.mkdir(parents=True, exist_ok=True)
    recipe9 = [
        SAMPLES["plan2022"]["path"],
        SAMPLES["trade"]["path"],
        SAMPLES["field01"]["path"],
        SAMPLES["hwp3"]["path"],
        SAMPLES["missing"]["path"],
    ]
    write_md(LISTS / "recipe9.txt", "\n".join(recipe9) + "\n")
    write_md(LISTS / "ok4.txt", "\n".join(recipe9[:4]) + "\n")
    write_md(LISTS / "missing_only.txt", SAMPLES["missing"]["path"] + "\n")
    write_md(
        LISTS / "convert_collision.txt",
        "inbox/Report.HWP\ninbox/report.hwp\n",
    )
    write_md(
        LISTS / "convert_ok.txt",
        SAMPLES["handbook"]["path"] + "\n",
    )
    write_md(
        LISTS / "search_pair.txt",
        SAMPLES["hwp3"]["path"] + "\n" + SAMPLES["hwp3"]["path"] + "\n",
    )
    write_md(
        LISTS / "fields_pair.txt",
        SAMPLES["field01"]["path"] + "\n" + SAMPLES["hwp3"]["path"] + "\n",
    )
    write_md(
        LISTS / "tables_one.txt",
        SAMPLES["table001"]["path"] + "\n",
    )
    write_md(
        DATA / "mailmerge_3.jsonl",
        "".join(
            json.dumps({"성명": n, "myMsg01": f"{n} 귀하"}, ensure_ascii=False) + "\n"
            for n in ("홍길동", "김철수", "이영희")
        ),
    )
    write_md(
        DATA / "mailmerge_3.csv",
        "성명,myMsg01\n홍길동,홍길동 귀하\n김철수,김철수 귀하\n이영희,이영희 귀하\n",
    )
    write_md(DATA / "empty_header_only.csv", "성명,myMsg01\n")
    write_md(
        DATA / "mailmerge_12.jsonl",
        "".join(
            json.dumps({"성명": f"신청자{i:02d}", "myMsg01": f"신청자{i:02d} 귀하"}, ensure_ascii=False)
            + "\n"
            for i in range(1, 13)
        ),
    )


def transcript_for(spec):
    tid, title, cmd, axis, inp, ok, bad, exit_code, stop = spec
    if tid == "T01":
        return recipe9_info_rows()
    if tid == "T02" or tid == "T16":
        return recipe9_text_rows()
    if tid == "T03":
        return recipe9_extract_rows(limit=3)
    if tid == "T04":
        return [
            {
                "schemaVersion": SCHEMA,
                "source": SAMPLES["handbook"]["path"],
                "format": "hwp5",
                "output": "out/bulk\\2025 행정업무운영 편람(최종).hwp",
                "bytes": SAMPLES["handbook"]["convertBytes"],
            }
        ]
    if tid == "T05":
        return [
            {
                "schemaVersion": SCHEMA,
                "source": SAMPLES["hwp3"]["path"],
                "query": "의",
                "matchCount": 3,
                "matches": [{"page": 0, "text": "의"}],
            },
            {
                "schemaVersion": SCHEMA,
                "source": SAMPLES["hwp3"]["path"],
                "query": "의",
                "matchCount": 3,
                "matches": [{"page": 0, "text": "의"}],
            },
        ]
    if tid in {"T06", "T07", "T08", "T14", "T15"}:
        return []
    if tid == "T09":
        return [
            {
                "schemaVersion": SCHEMA,
                "source": SAMPLES["field01"]["path"],
                "fieldCount": 11,
                "fields": [{"name": "회사명"}],
            },
            {
                "schemaVersion": SCHEMA,
                "source": SAMPLES["hwp3"]["path"],
                "fieldCount": 0,
                "fields": [],
            },
        ]
    if tid == "T10":
        return [
            {
                "schemaVersion": SCHEMA,
                "source": SAMPLES["table001"]["path"],
                "tableCount": 1,
                "tables": [
                    {
                        "index": 0,
                        "rows": 2,
                        "cols": 3,
                        "cells": [{"row": 0, "col": 0, "colSpan": 2, "rowSpan": 1, "text": "병합"}],
                    }
                ],
            }
        ]
    if tid == "T11":
        return [
            {
                "schemaVersion": SCHEMA,
                "source": SAMPLES["plan2022"]["path"],
                "mode": "auto",
                "outlineCount": 8,
            },
            {
                "schemaVersion": SCHEMA,
                "source": SAMPLES["hwp3"]["path"],
                "mode": "auto",
                "outlineCount": 2,
            },
        ]
    if tid in {"T12", "T13"}:
        rows = []
        for i, name in enumerate(("홍길동", "김철수", "이영희")):
            rec = {
                "schemaVersion": SCHEMA,
                "source": SAMPLES["form01"]["path"],
                "row": i,
                "dryRun": tid == "T12",
                "filledCount": 1,
                "filled": [{"name": "myMsg01", "occurrence": 0, "value": f"{name} 귀하"}],
                "notFound": [],
                "ambiguous": [],
            }
            if tid == "T13":
                rec["output"] = f"out/{name}.hwp"
                rec["outputFormat"] = "hwp5"
            rows.append(rec)
        return rows
    if tid == "T17":
        return recipe9_text_rows()[:1]
    if tid == "T18":
        return [
            {
                "schemaVersion": SCHEMA,
                "source": SAMPLES["handbook"]["path"],
                "format": "hwp5",
                "output": "out/x.hwp",
                "bytes": 100,
                "verify": {"identical": False, "diffCount": 2},
            }
        ]
    if tid == "T19":
        return [
            {
                "schemaVersion": SCHEMA,
                "source": SAMPLES["handbook"]["path"],
                "format": "hwp5",
                "output": "out/x.hwp",
                "bytes": 100,
                "verifyPages": {"ok": False, "expected": 387, "actual": 388},
            }
        ]
    if tid == "T20":
        return [
            {
                "schemaVersion": SCHEMA,
                "source": f"docs/doc{i:03d}.hwp",
                "format": "hwp5",
                "pageCount": 5 + (i % 40),
            }
            for i in range(12)
        ]
    # generic mixed
    if axis == "fill":
        return [
            {
                "schemaVersion": SCHEMA,
                "source": SAMPLES["form01"]["path"],
                "row": 0,
                "dryRun": False,
                "filledCount": 1,
                "filled": [{"name": "myMsg01", "occurrence": 0, "value": "x"}],
                "notFound": [],
                "ambiguous": [],
                "output": "out/0001.hwp",
            }
        ]
    rows = []
    for i in range(max(ok, 1)):
        rows.append(
            {
                "schemaVersion": SCHEMA,
                "source": f"corpus/{axis}-{i:02d}.hwp",
                "pageCount": 3 + i,
                "axis": axis,
            }
        )
    for j in range(bad):
        rows.append(failure_record(f"corpus/{axis}-missing-{j}.hwp", "문서를 열 수 없습니다: 지정된 파일을 찾을 수 없습니다. (os error 2)"))
    return rows


def write_transcripts():
    TRANS.mkdir(parents=True, exist_ok=True)
    specs = TRACE_SPECS
    for spec in specs:
        rows = transcript_for(spec)
        write_md(TRANS / f"{spec[0]}.ndjson", ndjson_text(rows) if rows else "")
        dump(
            FIXT / "traces" / f"{spec[0]}.json",
            meta(
                {
                    "id": spec[0],
                    "title": spec[1],
                    "command": spec[2],
                    "axis": spec[3],
                    "input": spec[4],
                    "success": spec[5],
                    "failure": spec[6],
                    "exit": spec[7],
                    "stop": spec[8],
                    "rows": rows,
                    "stderrExample": None
                    if spec[7] == 2
                    else f"batch: {spec[4]}건 중 {spec[5]} 성공, {spec[6]} 실패"
                    if spec[4]
                    else None,
                }
            ),
        )


def write_stderr_examples():
    dump(
        FIXT / "stderr_summaries.json",
        meta(
            {
                "examples": [
                    {
                        "axis": "export-text",
                        "stderr": "batch: 5건 중 4 성공, 1 실패",
                        "stdoutLines": 5,
                        "exit": 1,
                    },
                    {
                        "axis": "info",
                        "stderr": "batch: 5건 중 4 성공, 1 실패",
                        "stdoutLines": 5,
                        "exit": 1,
                    },
                    {
                        "axis": "convert",
                        "stderr": "batch convert: 이름 충돌 — 산출을 쓰지 않습니다",
                        "stdoutLines": 0,
                        "exit": 2,
                    },
                    {
                        "axis": "search",
                        "stderr": "error: --query 가 필요합니다",
                        "stdoutLines": 0,
                        "exit": 2,
                    },
                    {
                        "axis": "info",
                        "stderr": "error: batch 는 --password 를 지원하지 않습니다",
                        "stdoutLines": 0,
                        "exit": 2,
                    },
                    {
                        "axis": "fill",
                        "stderr": "batch fill: 3행 중 3 성공",
                        "stdoutLines": 3,
                        "exit": 0,
                    },
                    {
                        "axis": "convert",
                        "stderr": "batch: 1건 중 1 성공, 검증 판정 차이",
                        "stdoutLines": 1,
                        "exit": 3,
                    },
                    {
                        "axis": "convert",
                        "stderr": "batch: 1건 중 1 성공, 페이지 검증 불일치",
                        "stdoutLines": 1,
                        "exit": 4,
                    },
                ]
            }
        ),
    )


if __name__ == "__main__":
    write_fixtures()
    write_lists_and_data()
    write_transcripts()
    write_stderr_examples()
    print("fixtures+transcripts written", ISSUE)
