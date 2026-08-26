#!/usr/bin/env python3
"""M04-f proptest 왕복 변형·예외 카탈로그 생성기.

devel 의 기존 `rhwp run` step 4종(fill_fields · replace_text · set_cell ·
set_checkbox)만 펼친다. DocumentCore 편집 API 를 발명하지 않고, 픽스처가
표현하지 못하는 step 은 skip 정직 표로 남긴다.

생성물:
  tests/fixtures/proptest_m04f/
    catalogs/   JSON·JSONL
    cases/      action 별 변형 JSONL
    matrices/   TSV
    reports/    요약·스킵 정직 표·CI
    schema/     catalog.v1.json
"""

from __future__ import annotations

import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "tests" / "fixtures" / "proptest_m04f"
CATALOG_VERSION = "m04f.v1"
PLAN_VERSION = "1.0"

ACTIONS = ("fill_fields", "replace_text", "set_cell", "set_checkbox")

FIELD_NAMES = (
    "이름",
    "주소",
    "신청인",
    "피규제집단명",
    "title",
    "name",
    "myMsg01",
)
FIELD_OCCURRENCES = (None, 0, 1, 2, 13)
FIELD_VALUES = (
    ("string", "홍길동"),
    ("string", "서울"),
    ("string", "한국"),
    ("string", "2026"),
    ("string", "DONE"),
    ("string", ""),
    ("number", 0),
    ("number", 1),
    ("number", 42),
    ("number", 9999),
    ("boolean", True),
    ("boolean", False),
)

FIND_NEEDLES = (
    "한글",
    "기관명",
    "2024",
    "TODO",
    "example",
    "안녕",
    "Hello",
    "123",
    "오호라",
    "乾坤",
    "구궁산",
    "품질",
    "5월",
    "평가",
)
REPLACE_TEXTS = ("", "한국", "X", "2026", "DONE", "서울")
CELL_TEXTS = ("", "서울", "완료", "1", "1,000", "n/a", "한국")
PATHS = (
    "in.hwp",
    "in.hwpx",
    "form.hml",
    "samples/field-01.hwp",
    "samples/para-001.hwp",
    "samples/table-001.hwp",
    "samples/hwpx/ref/ref_text.hwpx",
    "samples/hwpx/ref/ref_table.hwpx",
    "out.hwp",
    "out.hwpx",
)

# apply_existing_step 이 실제로 내는 skip 이유. 새 mutation 을 만들지 않는다.
SKIP_REASONS = {
    "empty_find": {
        "action": "replace_text",
        "engine": "apply_existing_step",
        "when": "find 가 빈 문자열",
        "honest": "문서 전체를 치환 대상으로 해석하지 않고 skip",
    },
    "no_hits": {
        "action": "replace_text",
        "engine": "DocumentCore::grep",
        "when": "find 가 이 픽스처 본문에 없음",
        "honest": "조용히 0건 치환을 성공으로 보고하지 않고 skip",
    },
    "occurrence_oob": {
        "action": "replace_text",
        "engine": "replace_nth_native",
        "when": "occurrence >= grep hits",
        "honest": "없는 순번을 발명하지 않고 skip",
    },
    "field_missing": {
        "action": "fill_fields",
        "engine": "set_field_value_by_name_at",
        "when": "그 이름(또는 순번)의 누름틀이 없음",
        "honest": "누름틀을 만들어 넣지 않고 skip",
    },
    "table_missing": {
        "action": "set_cell",
        "engine": "extract_tables",
        "when": "table 인덱스가 격자 수 밖",
        "honest": "표를 만들어 넣지 않고 skip",
    },
    "nested_table": {
        "action": "set_cell",
        "engine": "extract_tables.container_path",
        "when": "표가 다른 컨테이너 안 (중첩)",
        "honest": "중첩 표 set_cell 경로를 발명하지 않고 skip",
    },
    "cell_missing": {
        "action": "set_cell",
        "engine": "Table.cells 좌표",
        "when": "(row, col) 앵커 셀이 없음",
        "honest": "빈 칸을 만들어 넣지 않고 skip",
    },
    "cell_control_char": {
        "action": "set_cell",
        "engine": "run_plan_engine 선검증",
        "when": "text 에 \\r · \\n · \\t",
        "honest": "칸 구조를 깨는 값을 쓰지 않고 skip",
    },
    "checkbox_missing": {
        "action": "set_checkbox",
        "engine": "grep('□')",
        "when": "occurrence 번째 빈 체크박스가 없음",
        "honest": "□ 를 삽입하지 않고 skip",
    },
    "all_steps_skipped": {
        "action": "*",
        "engine": "assert_edit_serialize_reparse",
        "when": "시퀀스의 모든 step 이 skip",
        "honest": "무편집 왕복을 '편집 왕복 성공'으로 세지 않고 reject",
    },
    "unclaimed_capability": {
        "action": "*",
        "engine": "catalog",
        "when": "이 픽스처에 대해 step 표현력을 주장하지 않음",
        "honest": "ref_mixed 처럼 내용이 섞여도 추측으로 apply 하지 않음",
    },
}

# 기존 M04-2/3 가 이미 고정한 능력만 적는다. 새 픽스처 능력은 주장하지 않는다.
FIXTURES = (
    {
        "id": "ref_text_hwpx",
        "path": "samples/hwpx/ref/ref_text.hwpx",
        "format": "hwpx",
        "layer": "m04-2",
        "identity": "ir_diff_0",
        "claimed": True,
        "needles": ("안녕", "Hello", "123"),
        "tables": 0,
        "table_rows": 0,
        "table_cols": 0,
        "nested_tables": 0,
        "fields": (),
        "checkboxes": 0,
        "note": "hwpx_roundtrip_integration Stage 1. 표·누름틀·□ 없음.",
    },
    {
        "id": "ref_table_hwpx",
        "path": "samples/hwpx/ref/ref_table.hwpx",
        "format": "hwpx",
        "layer": "m04-2",
        "identity": "ir_diff_0",
        "claimed": True,
        "needles": (),
        "tables": 1,
        "table_rows": 2,
        "table_cols": 3,
        "nested_tables": 0,
        "fields": (),
        "checkboxes": 0,
        "note": "2×3 빈 표. baseline 등급. 본문 needle 을 주장하지 않음.",
    },
    {
        "id": "para001_hwp5",
        "path": "samples/para-001.hwp",
        "format": "hwp5",
        "layer": "m04-3",
        "identity": "ir_diff_0",
        "claimed": True,
        "needles": ("오호라", "乾坤", "구궁산"),
        "tables": 0,
        "table_rows": 0,
        "table_cols": 0,
        "nested_tables": 0,
        "fields": (),
        "checkboxes": 0,
        "note": "hwp5_roundtrip_baseline A등급. 표·누름틀·□ 없음.",
    },
    {
        "id": "table001_hwp5",
        "path": "samples/table-001.hwp",
        "format": "hwp5",
        "layer": "m04-3",
        "identity": "ir_diff_0",
        "claimed": True,
        "needles": ("품질", "5월", "평가"),
        "tables": 1,
        "table_rows": 19,
        "table_cols": 9,
        "nested_tables": 0,
        "fields": (),
        "checkboxes": 0,
        "note": "19×9 표, 중첩 없음. set_cell (0,2,1) 왕복이 손글씨로 고정.",
    },
    {
        "id": "ref_empty_hwpx",
        "path": "samples/hwpx/ref/ref_empty.hwpx",
        "format": "hwpx",
        "layer": "m04-f",
        "identity": "identity_only",
        "claimed": True,
        "needles": (),
        "tables": 0,
        "table_rows": 0,
        "table_cols": 0,
        "nested_tables": 0,
        "fields": (),
        "checkboxes": 0,
        "note": "빈 문서. 편집 step 은 전부 skip. 무편집 왕복만 주장.",
    },
    {
        "id": "ref_mixed_hwpx",
        "path": "samples/hwpx/ref/ref_mixed.hwpx",
        "format": "hwpx",
        "layer": "m04-f",
        "identity": "unclaimed",
        "claimed": False,
        "needles": (),
        "tables": 0,
        "table_rows": 0,
        "table_cols": 0,
        "nested_tables": 0,
        "fields": (),
        "checkboxes": 0,
        "note": "혼합 문서. 표현력을 추측하지 않음 — unclaimed_capability.",
    },
)

INVALID_FAMILIES = (
    "bad_plan_version",
    "empty_steps",
    "missing_input",
    "missing_output",
    "missing_steps",
    "steps_not_array",
    "unknown_action",
    "replace_empty_find",
    "replace_missing_find",
    "replace_missing_replace",
    "set_cell_newline",
    "set_cell_tab",
    "set_cell_cr",
    "set_cell_negative_row",
    "set_cell_negative_col",
    "set_cell_row_overflow",
    "set_cell_col_overflow",
    "set_cell_missing_text",
    "set_cell_missing_table",
    "set_cell_missing_row",
    "set_cell_missing_col",
    "set_checkbox_missing_occurrence",
    "fill_fields_missing_data",
    "fill_fields_data_not_object",
    "multi_key_if",
    "empty_if",
    "unknown_if_key",
    "occurrence_negative",
    "table_negative",
    "case_sensitive_not_bool",
    "keep_style_not_bool",
    "find_not_string",
    "dry_run_not_bool",
    "assertions_not_object",
    "if_not_object",
)


def field_key(name: str, occ) -> str:
    return f"{name}[{occ}]" if occ is not None else name


def dumps(obj) -> str:
    return json.dumps(obj, ensure_ascii=False, separators=(",", ":"), sort_keys=True)


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text.replace("\r\n", "\n"), encoding="utf-8")


def write_json(path: Path, obj) -> None:
    write_text(path, json.dumps(obj, ensure_ascii=False, indent=2, sort_keys=True))


def write_jsonl(path: Path, rows) -> None:
    write_text(path, "\n".join(dumps(row) for row in rows))


def write_tsv(path: Path, header, rows) -> None:
    lines = ["\t".join(header)]
    for row in rows:
        lines.append("\t".join("" if c is None else str(c) for c in row))
    write_text(path, "\n".join(lines))


def cond_none():
    return None


def cond_field_exists(name: str):
    return {"fieldExists": name}


def cond_field_equals(name: str, value: str):
    return {"fieldEquals": {"name": name, "value": value}}


def cond_text_found(text: str):
    return {"textFound": text}


def plan_of(steps, *, input_path="in.hwpx", output_path="out.hwpx", dry_run=None, assertions=None):
    plan = {
        "planVersion": PLAN_VERSION,
        "input": input_path,
        "output": output_path,
        "steps": steps,
    }
    if dry_run is not None:
        plan["dryRun"] = dry_run
    if assertions is not None:
        plan["assertions"] = assertions
    return plan


def step_fill(name: str, value, *, occ=None, cond=None):
    step = {"action": "fill_fields", "data": {field_key(name, occ): value}}
    if cond is not None:
        step["if"] = cond
    return step


def step_replace(find: str, replace: str, *, case=None, occ=None, cond=None):
    step = {"action": "replace_text", "find": find, "replace": replace}
    if case is not None:
        step["caseSensitive"] = case
    if occ is not None:
        step["occurrence"] = occ
    if cond is not None:
        step["if"] = cond
    return step


def step_cell(table: int, row: int, col: int, text: str, *, keep=None, cond=None):
    step = {"action": "set_cell", "table": table, "row": row, "col": col, "text": text}
    if keep is not None:
        step["keepStyle"] = keep
    if cond is not None:
        step["if"] = cond
    return step


def step_check(occurrence: int, *, cond=None):
    step = {"action": "set_checkbox", "occurrence": occurrence}
    if cond is not None:
        step["if"] = cond
    return step


def build_fixtures():
    rows = []
    for fx in FIXTURES:
        rows.append(
            {
                "id": fx["id"],
                "path": fx["path"],
                "format": fx["format"],
                "layer": fx["layer"],
                "identity": fx["identity"],
                "claimed": fx["claimed"],
                "needles": list(fx["needles"]),
                "tables": fx["tables"],
                "tableRows": fx["table_rows"],
                "tableCols": fx["table_cols"],
                "nestedTables": fx["nested_tables"],
                "fields": list(fx["fields"]),
                "checkboxes": fx["checkboxes"],
                "note": fx["note"],
                "applyable": {
                    "fill_fields": bool(fx["claimed"] and fx["fields"]),
                    "replace_text": bool(fx["claimed"] and fx["needles"]),
                    "set_cell": bool(fx["claimed"] and fx["tables"] > 0),
                    "set_checkbox": bool(fx["claimed"] and fx["checkboxes"] > 0),
                },
            }
        )
    return rows


def build_skip_reasons():
    rows = []
    for code, meta in SKIP_REASONS.items():
        rows.append({"code": code, **meta})
    return rows


def skip_row(case_id, fixture, action, reason, step, expected="skip", detail=""):
    return {
        "id": case_id,
        "fixture": fixture["id"],
        "path": fixture["path"],
        "format": fixture["format"],
        "claimed": fixture["claimed"],
        "action": action,
        "reason": reason,
        "expected": expected,
        "step": step,
        "detail": detail,
        "engine": SKIP_REASONS[reason]["engine"],
    }


def build_skip_catalog():
    rows = []
    n = 0

    def add(fx, action, reason, step, expected="skip", detail=""):
        nonlocal n
        n += 1
        rows.append(
            skip_row(f"skip-{n:04d}-{fx['id']}-{reason}", fx, action, reason, step, expected, detail)
        )

    for fx in FIXTURES:
        if not fx["claimed"]:
            for action in ACTIONS:
                add(
                    fx,
                    action,
                    "unclaimed_capability",
                    {"action": action},
                    detail="혼합 픽스처 능력을 추측하지 않음",
                )
            continue

        for name in FIELD_NAMES:
            for occ in (None, 0, 1, 13):
                if name in fx["fields"] and (occ is None or occ == 0):
                    continue
                add(
                    fx,
                    "fill_fields",
                    "field_missing",
                    step_fill(name, "홍길동", occ=occ),
                    detail=f"필드 {field_key(name, occ)} 없음",
                )

        for find in FIND_NEEDLES:
            if find in fx["needles"]:
                add(
                    fx,
                    "replace_text",
                    "occurrence_oob",
                    step_replace(find, "한국", occ=8),
                    detail=f"{find} 는 있어도 occurrence=8 은 밖",
                )
            else:
                add(
                    fx,
                    "replace_text",
                    "no_hits",
                    step_replace(find, "한국"),
                    detail=f"{find} 가 이 픽스처 needle 목록에 없음",
                )
        add(
            fx,
            "replace_text",
            "empty_find",
            step_replace("", "한국"),
            detail="빈 find 는 문서 전체로 해석하지 않음",
        )

        if fx["tables"] == 0:
            for table, row, col in ((0, 0, 0), (0, 1, 1), (1, 0, 0), (2, 8, 8)):
                add(
                    fx,
                    "set_cell",
                    "table_missing",
                    step_cell(table, row, col, "서울"),
                    detail="표가 없는 픽스처",
                )
        else:
            add(
                fx,
                "set_cell",
                "table_missing",
                step_cell(fx["tables"], 0, 0, "서울"),
                detail=f"table={fx['tables']} 는 격자 밖",
            )
            add(
                fx,
                "set_cell",
                "cell_missing",
                step_cell(0, fx["table_rows"], 0, "서울"),
                detail=f"row={fx['table_rows']} 는 격자 밖",
            )
            add(
                fx,
                "set_cell",
                "cell_missing",
                step_cell(0, 0, fx["table_cols"], "서울"),
                detail=f"col={fx['table_cols']} 는 격자 밖",
            )
            for bad, label in (("\n", "LF"), ("\r", "CR"), ("\t", "TAB"), ("a\nb", "LF중간")):
                add(
                    fx,
                    "set_cell",
                    "cell_control_char",
                    step_cell(0, 0, 0, bad),
                    detail=f"칸 값에 {label}",
                )
        if fx["nested_tables"] == 0:
            # 중첩 표가 없다고 주장하는 픽스처에 대한 가드 행 — 실제 중첩이면 skip.
            add(
                fx,
                "set_cell",
                "nested_table",
                step_cell(0, 0, 0, "서울"),
                expected="skip_if_nested",
                detail="container_path 가 비어 있지 않으면 skip (이 픽스처는 중첩 0 주장)",
            )

        for occ in range(0, 8):
            if occ < fx["checkboxes"]:
                continue
            add(
                fx,
                "set_checkbox",
                "checkbox_missing",
                step_check(occ),
                detail=f"□ occurrence={occ} 없음 (claimed {fx['checkboxes']})",
            )

        add(
            fx,
            "*",
            "all_steps_skipped",
            [step_fill("이름", "x"), step_check(0)],
            expected="reject",
            detail="적용 0 이면 왕복 성공으로 세지 않음",
        )

    return rows


def build_valid_plans():
    rows = []
    n = 0

    def add(family, action, plan, **extra):
        nonlocal n
        n += 1
        rows.append(
            {
                "id": f"valid-{n:04d}-{family}",
                "family": family,
                "action": action,
                "expected": "schema_ok",
                "plan": plan,
                **extra,
            }
        )

    conds_small = (
        None,
        cond_field_exists("이름"),
        cond_text_found("한글"),
    )
    values_small = FIELD_VALUES[:8]
    for name in FIELD_NAMES:
        for occ in (None, 0, 1, 13):
            for kind, value in values_small:
                for cond in conds_small:
                    add(
                        "fill_fields_axis",
                        "fill_fields",
                        plan_of([step_fill(name, value, occ=occ, cond=cond)]),
                        field=field_key(name, occ),
                        valueKind=kind,
                    )

    for find in FIND_NEEDLES:
        for replace in REPLACE_TEXTS:
            for case in (None, True, False):
                for occ in (None, 0, 1):
                    add(
                        "replace_text_axis",
                        "replace_text",
                        plan_of(
                            [step_replace(find, replace, case=case, occ=occ)],
                            input_path="in.hwp",
                            output_path="out.hwp",
                        ),
                        find=find,
                        replace=replace,
                        deletion=replace == "",
                    )

    for table in (0, 1, 2):
        for row in (0, 1, 2, 8, 18):
            for col in (0, 1, 2, 8):
                for text in CELL_TEXTS:
                    for keep in (None, True, False):
                        add(
                            "set_cell_axis",
                            "set_cell",
                            plan_of([step_cell(table, row, col, text, keep=keep)]),
                            table=table,
                            row=row,
                            col=col,
                            keepStyle=keep,
                        )

    for occ in range(0, 16):
        for cond in (None, cond_field_exists("신청인"), cond_text_found("TODO"), cond_field_equals("이름", "홍길동")):
            add(
                "set_checkbox_axis",
                "set_checkbox",
                plan_of([step_check(occ, cond=cond)]),
                occurrence=occ,
            )

    # 다 step · 단언 · dryRun · 경로 교차. 기존 4종만 섞는다.
    combos = [
        [step_fill("이름", "홍길동"), step_replace("기관명", "한국")],
        [step_replace("2024", "2026"), step_cell(0, 0, 0, "서울")],
        [step_cell(0, 1, 2, "완료"), step_check(0)],
        [step_fill("피규제집단명", "한국", occ=13), step_check(1)],
        [step_replace("TODO", "DONE", case=True), step_replace("example", "예", occ=0)],
        [step_fill("주소", "서울"), step_fill("이름", "이몽룡")],
        [step_cell(0, 0, 0, ""), step_cell(0, 0, 1, "n/a")],
        [step_replace("한글", "", occ=0), step_check(0)],
        [step_fill("title", "공고", cond=cond_field_exists("title")), step_replace("Hello", "안녕")],
        [step_check(0, cond=cond_text_found("□")), step_replace("□", "☑")],
    ]
    for i, steps in enumerate(combos):
        for src in PATHS[:6]:
            for dst in PATHS[6:]:
                for dry in (None, True, False):
                    for asrt in (
                        None,
                        {"notFoundEmpty": True},
                        {"verify": False},
                        {"notFoundEmpty": True, "verify": False},
                    ):
                        add(
                            "multi_step_envelope",
                            "*",
                            plan_of(steps, input_path=src, output_path=dst, dry_run=dry, assertions=asrt),
                            combo=i,
                        )

    return rows


def build_invalid_plans():
    rows = []
    n = 0

    def add(family, plan, why):
        nonlocal n
        n += 1
        rows.append(
            {
                "id": f"invalid-{n:04d}-{family}",
                "family": family,
                "expected": "schema_reject",
                "why": why,
                "plan": plan,
            }
        )

    seed_step = step_fill("이름", "x")
    for ver in ("0.9", "0.1", "2.0", "1", "1.0.0", "v1", "", "latest"):
        add(
            "bad_plan_version",
            {"planVersion": ver, "input": "in.hwp", "output": "out.hwp", "steps": [seed_step]},
            f"planVersion {ver!r} 는 1.0 만 허용",
        )
    add(
        "empty_steps",
        {"planVersion": "1.0", "input": "in.hwp", "output": "out.hwp", "steps": []},
        "steps minItems=1",
    )
    add(
        "missing_input",
        {"planVersion": "1.0", "output": "out.hwp", "steps": [seed_step]},
        "input 필수",
    )
    add(
        "missing_output",
        {"planVersion": "1.0", "input": "in.hwp", "steps": [seed_step]},
        "output 필수",
    )
    add(
        "missing_steps",
        {"planVersion": "1.0", "input": "in.hwp", "output": "out.hwp"},
        "steps 필수",
    )
    add(
        "steps_not_array",
        {"planVersion": "1.0", "input": "in.hwp", "output": "out.hwp", "steps": "nope"},
        "steps 는 배열",
    )

    invented = (
        "explode",
        "insert_text",
        "delete_text",
        "merge_cells",
        "split_cell",
        "insert_table",
        "delete_row",
        "apply_style",
        "mutate_core",
        "set_field",
        "replace_all",
        "check_box",
        "FillFields",
        "REPLACE_TEXT",
        "",
    )
    for action in invented:
        add(
            "unknown_action",
            {
                "planVersion": "1.0",
                "input": "in.hwp",
                "output": "out.hwp",
                "steps": [{"action": action, "data": {"이름": "x"}}],
            },
            f"action {action!r} 는 스키마 4종 밖 — DocumentCore mutation 발명 금지",
        )

    for find in ("",):
        for replace in REPLACE_TEXTS:
            add(
                "replace_empty_find",
                {
                    "planVersion": "1.0",
                    "input": "in.hwp",
                    "output": "out.hwp",
                    "steps": [step_replace(find, replace)],
                },
                "find minLength=1",
            )
    add(
        "replace_missing_find",
        {
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{"action": "replace_text", "replace": "x"}],
        },
        "find 필수",
    )
    add(
        "replace_missing_replace",
        {
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{"action": "replace_text", "find": "한글"}],
        },
        "replace 필수",
    )

    for text, family, why in (
        ("a\nb", "set_cell_newline", "LF"),
        ("a\tb", "set_cell_tab", "TAB"),
        ("a\rb", "set_cell_cr", "CR"),
        ("\n", "set_cell_newline", "단독 LF"),
        ("\t", "set_cell_tab", "단독 TAB"),
        ("\r", "set_cell_cr", "단독 CR"),
        ("서울\n완료", "set_cell_newline", "한글+LF"),
        ("1\t000", "set_cell_tab", "숫자+TAB"),
    ):
        add(
            family,
            {
                "planVersion": "1.0",
                "input": "in.hwp",
                "output": "out.hwp",
                "steps": [step_cell(0, 0, 0, text)],
            },
            f"set_cell text pattern 이 {why} 거부",
        )

    for family, key, value, why in (
        ("set_cell_negative_row", "row", -1, "row minimum 0"),
        ("set_cell_negative_col", "col", -1, "col minimum 0"),
        ("set_cell_row_overflow", "row", 65536, "row maximum 65535"),
        ("set_cell_col_overflow", "col", 65536, "col maximum 65535"),
        ("set_cell_negative_row", "row", -99, "row 음수"),
        ("set_cell_row_overflow", "row", 100000, "row 과대"),
    ):
        step = step_cell(0, 0, 0, "x")
        step[key] = value
        add(
            family,
            {"planVersion": "1.0", "input": "in.hwp", "output": "out.hwp", "steps": [step]},
            why,
        )

    for missing, family in (
        ("text", "set_cell_missing_text"),
        ("table", "set_cell_missing_table"),
        ("row", "set_cell_missing_row"),
        ("col", "set_cell_missing_col"),
    ):
        step = step_cell(0, 0, 0, "x")
        del step[missing]
        add(
            family,
            {"planVersion": "1.0", "input": "in.hwp", "output": "out.hwp", "steps": [step]},
            f"{missing} 필수",
        )

    add(
        "set_checkbox_missing_occurrence",
        {
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{"action": "set_checkbox"}],
        },
        "occurrence 필수",
    )
    add(
        "fill_fields_missing_data",
        {
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{"action": "fill_fields"}],
        },
        "data 필수",
    )
    add(
        "fill_fields_data_not_object",
        {
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "steps": [{"action": "fill_fields", "data": ["이름", "x"]}],
        },
        "data 는 object",
    )

    for family, iff, why in (
        (
            "multi_key_if",
            {"fieldExists": "이름", "textFound": "한글"},
            "조건은 정확히 한 종류",
        ),
        (
            "multi_key_if",
            {"fieldExists": "이름", "fieldEquals": {"name": "이름", "value": "x"}},
            "fieldExists+fieldEquals",
        ),
        ("empty_if", {}, "minProperties=1"),
        ("unknown_if_key", {"fieldMissing": "이름"}, "닫힌 객체 — 모르는 키 거부"),
        ("unknown_if_key", {"exists": "이름"}, "실행기가 모르는 조건"),
    ):
        add(
            family,
            {
                "planVersion": "1.0",
                "input": "in.hwp",
                "output": "out.hwp",
                "steps": [step_replace("한글", "x", cond=iff)],
            },
            why,
        )

    for family, step, why in (
        (
            "occurrence_negative",
            step_replace("한글", "x", occ=-1),
            "occurrence minimum 0",
        ),
        ("table_negative", step_cell(-1, 0, 0, "x"), "table minimum 0"),
        (
            "case_sensitive_not_bool",
            {**step_replace("한글", "x"), "caseSensitive": "yes"},
            "caseSensitive boolean",
        ),
        (
            "keep_style_not_bool",
            {**step_cell(0, 0, 0, "x"), "keepStyle": "yes"},
            "keepStyle boolean",
        ),
        (
            "find_not_string",
            {"action": "replace_text", "find": 123, "replace": "x"},
            "find string",
        ),
        (
            "if_not_object",
            {**step_replace("한글", "x"), "if": "fieldExists"},
            "if 는 object",
        ),
    ):
        add(
            family,
            {"planVersion": "1.0", "input": "in.hwp", "output": "out.hwp", "steps": [step]},
            why,
        )

    add(
        "dry_run_not_bool",
        {
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "dryRun": "yes",
            "steps": [seed_step],
        },
        "dryRun boolean",
    )
    add(
        "assertions_not_object",
        {
            "planVersion": "1.0",
            "input": "in.hwp",
            "output": "out.hwp",
            "assertions": True,
            "steps": [seed_step],
        },
        "assertions object",
    )

    # 축 교차: 각 family 를 여러 경로·값으로 반복해 함정을 고정한다.
    for src in PATHS[:5]:
        for dst in ("out.hwp", "out.hwpx"):
            add(
                "empty_steps",
                {"planVersion": "1.0", "input": src, "output": dst, "steps": []},
                f"{src} → {dst} 빈 steps",
            )
            add(
                "unknown_action",
                {
                    "planVersion": "1.0",
                    "input": src,
                    "output": dst,
                    "steps": [{"action": "insert_paragraph", "text": "x"}],
                },
                "insert_paragraph 는 run step 4종 밖",
            )
            add(
                "replace_empty_find",
                {
                    "planVersion": "1.0",
                    "input": src,
                    "output": dst,
                    "steps": [step_replace("", "삭제금지")],
                },
                "빈 find",
            )

    for find in FIND_NEEDLES:
        add(
            "replace_empty_find",
            {
                "planVersion": "1.0",
                "input": "in.hwp",
                "output": "out.hwp",
                "steps": [{"action": "replace_text", "find": "", "replace": find}],
            },
            "find 빈 문자열",
        )
        add(
            "replace_missing_replace",
            {
                "planVersion": "1.0",
                "input": "in.hwp",
                "output": "out.hwp",
                "steps": [{"action": "replace_text", "find": find}],
            },
            "replace 누락",
        )

    for name in FIELD_NAMES:
        add(
            "fill_fields_missing_data",
            {
                "planVersion": "1.0",
                "input": "in.hwp",
                "output": "out.hwp",
                "steps": [{"action": "fill_fields", "name": name, "value": "x"}],
            },
            "data 없이 name/value 를 발명하면 거부",
        )
        add(
            "multi_key_if",
            {
                "planVersion": "1.0",
                "input": "in.hwp",
                "output": "out.hwp",
                "steps": [
                    step_fill(
                        name,
                        "x",
                        cond={"fieldExists": name, "textFound": name},
                    )
                ],
            },
            "조건 두 개",
        )

    for table in (0, 1, 3):
        for row, col in ((-1, 0), (0, -1), (65536, 0), (0, 65536)):
            family = (
                "set_cell_negative_row"
                if row < 0
                else "set_cell_negative_col"
                if col < 0
                else "set_cell_row_overflow"
                if row > 65535
                else "set_cell_col_overflow"
            )
            add(
                family,
                {
                    "planVersion": "1.0",
                    "input": "in.hwp",
                    "output": "out.hwp",
                    "steps": [step_cell(table, row, col, "서울")],
                },
                f"table={table} row={row} col={col}",
            )

    for occ in (-1, -2, -8):
        add(
            "occurrence_negative",
            {
                "planVersion": "1.0",
                "input": "in.hwp",
                "output": "out.hwp",
                "steps": [step_check(occ)],
            },
            f"checkbox occurrence={occ}",
        )
        add(
            "occurrence_negative",
            {
                "planVersion": "1.0",
                "input": "in.hwp",
                "output": "out.hwp",
                "steps": [step_replace("한글", "x", occ=occ)],
            },
            f"replace occurrence={occ}",
        )

    return rows


def build_exceptions():
    rows = []
    n = 0

    def add(family, **payload):
        nonlocal n
        n += 1
        rows.append({"id": f"exc-{n:04d}-{family}", "family": family, **payload})

    # 빈 치환 = 삭제. 스키마는 허용, 엔진은 기존 replace_all_native.
    for find in FIND_NEEDLES:
        for fx in FIXTURES:
            present = find in fx["needles"]
            add(
                "empty_replace_deletion",
                fixture=fx["id"],
                action="replace_text",
                step=step_replace(find, ""),
                schema="ok",
                apply="apply" if present and fx["claimed"] else "skip",
                reason=None if present and fx["claimed"] else ("unclaimed_capability" if not fx["claimed"] else "no_hits"),
                note="replace 빈 문자열은 삭제. 새 API 아님.",
            )

    # occurrence None = 전건, 0 = 첫 건. 둘 다 기존 엔진.
    for find in ("안녕", "Hello", "오호라", "품질"):
        for occ, meaning in ((None, "replace_all_native"), (0, "replace_nth_native(0)")):
            add(
                "occurrence_none_vs_zero",
                find=find,
                occurrence=occ,
                engine=meaning,
                step=step_replace(find, "한국", occ=occ),
                note="None 과 0 을 같은 뜻으로 접지 않는다.",
            )

    # caseSensitive 기본 true. false 는 기존 grep 경로의 플래그일 뿐.
    for find in ("Hello", "TODO", "example"):
        for case in (None, True, False):
            add(
                "case_sensitive_flag",
                find=find,
                caseSensitive=case,
                default_when_omitted=True,
                step=step_replace(find, "X", case=case),
                note="기본 구별. 대소문자 폴백 로직을 발명하지 않음.",
            )

    # keepStyle 기본 false.
    for text in CELL_TEXTS:
        for keep in (None, True, False):
            add(
                "keep_style_flag",
                text=text,
                keepStyle=keep,
                default_when_omitted=False,
                step=step_cell(0, 0, 0, text, keep=keep),
                note="참이면 글자색 유지. 기본 검정. 새 서식 API 아님.",
            )

    # 필드 순번. 이름만 = 첫 칸 + ambiguous. [13] 은 그 순번만.
    for name in FIELD_NAMES:
        for occ, meaning in (
            (None, "첫 칸 + 저널 ambiguous"),
            (0, "이름[0] 명시"),
            (13, "동명 필드 13번 — 없으면 field_missing"),
        ):
            add(
                "field_occurrence_address",
                field=field_key(name, occ),
                meaning=meaning,
                step=step_fill(name, "한국", occ=occ),
                note="누름틀을 만들지 않음. 주소만 다르게 지목.",
            )

    # 숫자·불리언 값은 JSON 표기 그대로 문자열화. 엔진 기존 계약.
    for kind, value in FIELD_VALUES:
        add(
            "field_value_coercion",
            valueKind=kind,
            value=value,
            step=step_fill("이름", value),
            note="문자열이 아니면 JSON 표기 그대로 문자열화. 새 타입 변환기 아님.",
        )

    # 조건이 거짓이면 선검증 면제 + skipped.
    for action, step in (
        ("fill_fields", step_fill("이름", "x", cond=cond_field_exists("없는필드"))),
        ("replace_text", step_replace("한글", "x", cond=cond_text_found("없는문자열"))),
        ("set_cell", step_cell(0, 0, 0, "서울", cond=cond_field_exists("이름"))),
        ("set_checkbox", step_check(0, cond=cond_field_equals("이름", "아니오"))),
    ):
        add(
            "condition_false_skips_precheck",
            action=action,
            step=step,
            expected="skipped_true",
            note="거짓 조건 step 은 선검증 면제. 실행기를 바꾸지 않음.",
        )

    # 격자 경계. 있는 칸만 apply.
    for fx in FIXTURES:
        if not fx["claimed"] or fx["tables"] == 0:
            continue
        for row in range(0, fx["table_rows"] + 2):
            for col in range(0, min(fx["table_cols"] + 2, 6)):
                inside = row < fx["table_rows"] and col < fx["table_cols"]
                add(
                    "cell_grid_boundary",
                    fixture=fx["id"],
                    row=row,
                    col=col,
                    apply="apply" if inside else "skip",
                    reason=None if inside else "cell_missing",
                    step=step_cell(0, row, col, "서울"),
                )

    # needle 경계.
    for fx in FIXTURES:
        if not fx["claimed"]:
            continue
        for find in FIND_NEEDLES:
            add(
                "needle_presence",
                fixture=fx["id"],
                find=find,
                present=find in fx["needles"],
                apply="apply" if find in fx["needles"] else "skip",
                reason=None if find in fx["needles"] else "no_hits",
                step=step_replace(find, "한국"),
            )

    return rows


def build_mutations():
    """기존 step 만 이어 붙인 시퀀스. 새 mutation 없음."""
    rows = []
    n = 0

    def add(family, fixture_id, steps, expected, note):
        nonlocal n
        n += 1
        rows.append(
            {
                "id": f"mut-{n:04d}-{family}",
                "family": family,
                "fixture": fixture_id,
                "steps": steps,
                "expected": expected,
                "note": note,
            }
        )

    for fx in FIXTURES:
        fid = fx["id"]
        if not fx["claimed"]:
            add(
                "unclaimed_sequence",
                fid,
                [step_replace("안녕", "한국"), step_cell(0, 0, 0, "서울")],
                "skip_all",
                "능력 미주장 픽스처는 시퀀스 전체를 추측 적용하지 않음",
            )
            continue

        if fx["needles"]:
            first = fx["needles"][0]
            add(
                "replace_then_replace_same",
                fid,
                [step_replace(first, "한국"), step_replace(first, "서울")],
                "apply_then_skip",
                "첫 step 이 needle 을 없애면 둘째는 no_hits skip",
            )
            add(
                "replace_then_replace_result",
                fid,
                [step_replace(first, "한국"), step_replace("한국", "서울")],
                "apply_apply",
                "둘째 find 는 첫 치환 결과. 기존 replace 경로만 두 번",
            )
            add(
                "replace_deletion_then_replace",
                fid,
                [step_replace(first, ""), step_replace(first, "한국")],
                "apply_then_skip",
                "삭제한 needle 을 다시 찾을 수 없음",
            )
            add(
                "replace_occ0_then_all",
                fid,
                [step_replace(first, "한국", occ=0), step_replace(first, "서울")],
                "apply_maybe_skip",
                "첫 건만 바꿈. 남은 hit 이 있으면 둘째 apply",
            )
            for other in fx["needles"][1:]:
                add(
                    "replace_two_needles",
                    fid,
                    [step_replace(first, "한국"), step_replace(other, "서울")],
                    "apply_apply",
                    "서로 다른 needle. 둘 다 기존 grep 경로",
                )

        if fx["tables"]:
            add(
                "set_cell_then_same_cell",
                fid,
                [step_cell(0, 0, 0, "서울"), step_cell(0, 0, 0, "완료")],
                "apply_apply",
                "같은 칸을 두 번. 마지막 값이 남음. 새 셀 API 아님",
            )
            add(
                "set_cell_then_clear",
                fid,
                [step_cell(0, 0, 0, "서울"), step_cell(0, 0, 0, "")],
                "apply_apply",
                "빈 문자열은 칸 비우기. delete+insert 기존 경로",
            )
            if fx["table_cols"] > 1:
                add(
                    "set_cell_neighbor",
                    fid,
                    [step_cell(0, 0, 0, "서울"), step_cell(0, 0, 1, "완료")],
                    "apply_apply",
                    "이웃 칸. 좌표만 다름",
                )
            add(
                "set_cell_then_oob",
                fid,
                [step_cell(0, 0, 0, "서울"), step_cell(0, 99, 99, "완료")],
                "apply_then_skip",
                "둘째는 cell_missing",
            )
            if fx["needles"]:
                find = fx["needles"][0]
                add(
                    "set_cell_then_replace",
                    fid,
                    [step_cell(0, 0, 0, find), step_replace(find, "한국")],
                    "apply_apply",
                    "칸에 넣은 문자열이 본문 grep 에 잡힐 수 있음. 기존 두 경로",
                )
                add(
                    "replace_then_set_cell",
                    fid,
                    [step_replace(find, "한국"), step_cell(0, 0, 0, "서울")],
                    "apply_apply",
                    "본문 치환 후 칸 기록. 서로 다른 엔진 함수",
                )

        add(
            "fill_always_skip_on_claimed_no_fields",
            fid,
            [step_fill("이름", "홍길동"), step_fill("주소", "서울")],
            "skip_all" if not fx["fields"] else "apply_apply",
            "누름틀 없는 픽스처는 fill_fields 를 발명하지 않음",
        )
        add(
            "checkbox_always_skip_on_claimed_none",
            fid,
            [step_check(0), step_check(1)],
            "skip_all" if fx["checkboxes"] == 0 else "apply_apply",
            "□ 없는 픽스처에 체크박스를 삽입하지 않음",
        )
        add(
            "mixed_unexpressible",
            fid,
            [step_fill("이름", "x"), step_check(0), step_cell(9, 9, 9, "x")],
            "reject",
            "전부 skip 이면 all_steps_skipped reject",
        )

        if fx["needles"] and not fx["tables"]:
            add(
                "text_fixture_cell_is_skip",
                fid,
                [step_replace(fx["needles"][0], "한국"), step_cell(0, 0, 0, "서울")],
                "apply_then_skip",
                "텍스트 픽스처의 set_cell 은 table_missing",
            )

    # 계획서 시퀀스 (문서 적용이 아니라 스키마 왕복).
    for a in ACTIONS:
        for b in ACTIONS:
            steps = []
            if a == "fill_fields":
                steps.append(step_fill("이름", "홍길동"))
            elif a == "replace_text":
                steps.append(step_replace("한글", "한국"))
            elif a == "set_cell":
                steps.append(step_cell(0, 0, 0, "서울"))
            else:
                steps.append(step_check(0))
            if b == "fill_fields":
                steps.append(step_fill("주소", "서울"))
            elif b == "replace_text":
                steps.append(step_replace("기관명", "한국"))
            elif b == "set_cell":
                steps.append(step_cell(0, 1, 1, "완료"))
            else:
                steps.append(step_check(1))
            add(
                "schema_action_pair",
                "plan_only",
                steps,
                "schema_ok",
                f"{a}+{b} 계획서 쌍. 실행은 픽스처 능력에 따름",
            )

    return rows


def build_conditions():
    rows = []
    n = 0

    def add(family, cond, ok, why):
        nonlocal n
        n += 1
        rows.append(
            {
                "id": f"cond-{n:04d}-{family}",
                "family": family,
                "condition": cond,
                "schema": "ok" if ok else "reject",
                "why": why,
            }
        )

    for name in FIELD_NAMES:
        add("field_exists", cond_field_exists(name), True, "한 키")
        add("field_exists_indexed", cond_field_exists(field_key(name, 0)), True, "이름[0]")
        add("field_exists_indexed", cond_field_exists(field_key(name, 13)), True, "이름[13]")
        add("field_equals", cond_field_equals(name, "홍길동"), True, "현재 값 일치")
        add("field_equals", cond_field_equals(name, ""), True, "빈 값 일치")
        add("field_exists_empty", cond_field_exists(""), False, "minLength=1")
        add("text_found_empty", cond_text_found(""), False, "minLength=1")
        add(
            "multi",
            {"fieldExists": name, "textFound": "한글"},
            False,
            "maxProperties=1",
        )
        add(
            "multi",
            {"fieldExists": name, "fieldEquals": {"name": name, "value": "x"}},
            False,
            "두 조건",
        )
    for find in FIND_NEEDLES:
        add("text_found", cond_text_found(find), True, "본문 한 건")
    add("empty_object", {}, False, "minProperties=1")
    add("unknown", {"fieldMissing": "이름"}, False, "additionalProperties false")
    add("unknown", {"equals": "x"}, False, "모르는 키")
    add("unknown", {"text_found": "한글"}, False, "snake_case 키 아님")
    return rows


def build_action_cases(valid):
    grouped = {a: [] for a in ACTIONS}
    for row in valid:
        action = row.get("action")
        if action in grouped:
            grouped[action].append(row)
    return grouped


def build_schema():
    return {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "rhwp.proptest_m04f.catalog.v1",
        "title": "M04-f proptest 왕복 카탈로그",
        "description": "기존 run step 4종의 변형·스킵·예외. DocumentCore mutation 발명 금지.",
        "version": CATALOG_VERSION,
        "actions": list(ACTIONS),
        "skipReasons": list(SKIP_REASONS),
        "requiredCaseKeys": ["id", "expected"],
    }


def build_reports(fixtures, skips, valids, invalids, exceptions, mutations, conditions):
    skip_counts = Counter(r["reason"] for r in skips)
    valid_counts = Counter(r["family"] for r in valids)
    invalid_counts = Counter(r["family"] for r in invalids)
    applyable = {a: 0 for a in ACTIONS}
    for fx in fixtures:
        for a, ok in fx["applyable"].items():
            if ok:
                applyable[a] += 1

    summary = {
        "catalogVersion": CATALOG_VERSION,
        "issue": 5465,
        "seat": "M04-f",
        "actions": list(ACTIONS),
        "counts": {
            "fixtures": len(fixtures),
            "skipCatalog": len(skips),
            "validPlans": len(valids),
            "invalidPlans": len(invalids),
            "exceptions": len(exceptions),
            "mutations": len(mutations),
            "conditions": len(conditions),
        },
        "skipReasonCounts": dict(sorted(skip_counts.items())),
        "validFamilyCounts": dict(sorted(valid_counts.items())),
        "invalidFamilyCounts": dict(sorted(invalid_counts.items())),
        "fixturesClaimingApply": applyable,
        "honesty": [
            "누름틀 없는 픽스처에 fill_fields 를 적용하지 않는다.",
            "□ 없는 픽스처에 set_checkbox 를 적용하지 않는다.",
            "표 없는 픽스처에 set_cell 을 적용하지 않는다.",
            "needle 없는 replace_text 는 no_hits skip.",
            "빈 find 는 empty_find skip (문서 전체 치환 금지).",
            "칸 값의 CR/LF/TAB 은 cell_control_char skip.",
            "능력 미주장 픽스처(ref_mixed)는 unclaimed_capability.",
            "적용 0 인 시퀀스는 왕복 성공으로 세지 않는다.",
            "insert_text / merge_cells 등 run step 4종 밖 action 은 스키마 거부.",
        ],
        "outOfScope": [
            "DocumentCore 새 mutation API",
            "canvaskit_policy",
            "pdf renderer",
            "page-count serializer",
            "layout-anomaly",
            "oracle_public",
            "render_backend",
            "gym",
        ],
    }

    md_lines = [
        "# M04-f proptest 왕복 고도화 요약",
        "",
        "이슈 #5465. 기존 `rhwp run` step 4종만 변형·스킵·예외로 펼친다.",
        "DocumentCore 편집 로직은 발명하지 않는다.",
        "",
        "## 수량",
        "",
        "| 항목 | 건수 |",
        "|---|---:|",
        f"| 픽스처 | {len(fixtures)} |",
        f"| skip 정직 표 | {len(skips)} |",
        f"| 유효 계획 | {len(valids)} |",
        f"| 무효 계획 | {len(invalids)} |",
        f"| 예외 | {len(exceptions)} |",
        f"| 변형 시퀀스 | {len(mutations)} |",
        f"| 조건절 | {len(conditions)} |",
        "",
        "## 픽스처가 주장하는 apply",
        "",
        "| action | 주장 픽스처 수 |",
        "|---|---:|",
    ]
    for a in ACTIONS:
        md_lines.append(f"| `{a}` | {applyable[a]} |")
    md_lines += [
        "",
        "## skip 이유 분포",
        "",
        "| reason | 행 |",
        "|---|---:|",
    ]
    for reason, count in sorted(skip_counts.items()):
        md_lines.append(f"| `{reason}` | {count} |")
    md_lines += [
        "",
        "## 정직 규칙",
        "",
    ]
    for line in summary["honesty"]:
        md_lines.append(f"- {line}")
    md_lines += [
        "",
        "## 이 좌석이 만지지 않는 것",
        "",
    ]
    for line in summary["outOfScope"]:
        md_lines.append(f"- {line}")
    md_lines.append("")

    honesty_lines = [
        "# skip 정직 표",
        "",
        "각 행은 `apply_existing_step` 이 skip 하는 이유와 1:1 이다.",
        "새 편집 API 로 skip 을 메우지 않는다.",
        "",
        "| fixture | action | reason | expected |",
        "|---|---|---|---|",
    ]
    # 표는 너무 길 수 있으니 fixture×action×reason 집계만.
    seen = []
    keys = set()
    for row in skips:
        key = (row["fixture"], row["action"], row["reason"], row["expected"])
        if key in keys:
            continue
        keys.add(key)
        seen.append(key)
    for fx, action, reason, expected in seen:
        honesty_lines.append(f"| `{fx}` | `{action}` | `{reason}` | {expected} |")
    honesty_lines.append("")

    ci_lines = [
        "# M04-f CI",
        "",
        "왕복 property 는 퍼지가 아니다. 싼 debug + 이름 필터.",
        "",
        "```bash",
        "python tools/proptest_roundtrip/gen_m04f_catalogs.py",
        "python -m unittest tools.proptest_roundtrip.test_gen_m04f_catalogs",
        "node --test scripts/tests/run-prop-roundtrip.test.mjs",
        "node scripts/rust-test-suite-manifest.mjs --prepare",
        "node scripts/run-prop-roundtrip.mjs --cargo-test",
        "```",
        "",
        "러너 `scripts/run-prop-roundtrip.mjs` 가 집는 원본:",
        "",
        "- 필수: `prop_roundtrip_ci`",
        "- 본체: `prop_hwpx_roundtrip`, `prop_hwp5_roundtrip`",
        "- 계획 생성기: `prop_edit_plan`",
        "- 고도화: `prop_m04f_catalog`, `prop_m04f_skip`, `prop_m04f_plans`, `prop_m04f_exceptions`, `prop_m04f_mutations`",
        "",
        "원본이 없으면 skip (필수 `prop_roundtrip_ci` 제외).",
        "기본 8 cases. 전체 화력은 `PROPTEST_CASES`.",
        "nextest archive 5번째 shard 를 넣지 않는다.",
        "",
        "카탈로그 시험은 JSONL 을 읽어 스키마·스킵 정직만 본다.",
        "문서 parse→serialize 전수는 M04-2/3 의 기존 8 cases 가 맡는다.",
        "",
    ]
    return summary, "\n".join(md_lines), "\n".join(honesty_lines), "\n".join(ci_lines)


def build_matrices(fixtures, skips, valids, invalids):
    fx_step_rows = []
    for fx in fixtures:
        for action in ACTIONS:
            applyable = fx["applyable"][action]
            fx_step_rows.append(
                (
                    fx["id"],
                    fx["format"],
                    fx["layer"],
                    action,
                    "apply_possible" if applyable else "skip_only",
                    fx["needles"] and ",".join(fx["needles"]) or "",
                    fx["tables"],
                    fx["fields"] and ",".join(fx["fields"]) or "",
                    fx["checkboxes"],
                    "yes" if fx["claimed"] else "no",
                )
            )

    skip_count = Counter((r["fixture"], r["reason"]) for r in skips)
    skip_rows = []
    for fx in fixtures:
        for reason in SKIP_REASONS:
            skip_rows.append((fx["id"], reason, skip_count.get((fx["id"], reason), 0)))

    family_rows = []
    for family, count in sorted(Counter(r["family"] for r in valids).items()):
        family_rows.append(("valid", family, count))
    for family, count in sorted(Counter(r["family"] for r in invalids).items()):
        family_rows.append(("invalid", family, count))

    return fx_step_rows, skip_rows, family_rows


def generate(out: Path = OUT):
    fixtures = build_fixtures()
    reasons = build_skip_reasons()
    skips = build_skip_catalog()
    valids = build_valid_plans()
    invalids = build_invalid_plans()
    exceptions = build_exceptions()
    mutations = build_mutations()
    conditions = build_conditions()
    grouped = build_action_cases(valids)
    schema = build_schema()
    summary, summary_md, honesty_md, ci_md = build_reports(
        fixtures, skips, valids, invalids, exceptions, mutations, conditions
    )
    fx_step_rows, skip_rows, family_rows = build_matrices(fixtures, skips, valids, invalids)

    write_json(out / "schema" / "catalog.v1.json", schema)
    write_json(out / "catalogs" / "fixtures.json", fixtures)
    write_json(out / "catalogs" / "skip_reasons.json", reasons)
    write_jsonl(out / "catalogs" / "skip_catalog.jsonl", skips)
    write_jsonl(out / "catalogs" / "valid_plans.jsonl", valids)
    write_jsonl(out / "catalogs" / "invalid_plans.jsonl", invalids)
    write_jsonl(out / "catalogs" / "exception_catalog.jsonl", exceptions)
    write_jsonl(out / "catalogs" / "mutation_sequences.jsonl", mutations)
    write_jsonl(out / "catalogs" / "condition_catalog.jsonl", conditions)
    for action, rows in grouped.items():
        write_jsonl(out / "cases" / f"{action}_variants.jsonl", rows)

    write_tsv(
        out / "matrices" / "fixture_x_step.tsv",
        (
            "fixture",
            "format",
            "layer",
            "action",
            "verdict",
            "needles",
            "tables",
            "fields",
            "checkboxes",
            "claimed",
        ),
        fx_step_rows,
    )
    write_tsv(
        out / "matrices" / "skip_reason_counts.tsv",
        ("fixture", "reason", "rows"),
        skip_rows,
    )
    write_tsv(
        out / "matrices" / "plan_family_counts.tsv",
        ("kind", "family", "rows"),
        family_rows,
    )

    write_json(out / "reports" / "fatten_summary.json", summary)
    write_text(out / "reports" / "fatten_summary.md", summary_md)
    write_text(out / "reports" / "skip_honesty.md", honesty_md)
    write_text(out / "reports" / "ci.md", ci_md)
    write_text(
        out / "README.md",
        "\n".join(
            [
                "# M04-f proptest 왕복 픽스처",
                "",
                "이슈 #5465. `tools/proptest_roundtrip/gen_m04f_catalogs.py` 가 다시 쓴다.",
                "기존 run step 4종만. DocumentCore mutation 발명 금지.",
                "",
                "- `catalogs/` 픽스처·스킵·유효/무효 계획·예외·변형·조건",
                "- `cases/` action 별 변형",
                "- `matrices/` fixture×step · skip 분포",
                "- `reports/` 요약·정직 표·CI",
                "",
                "카탈로그를 손으로 고치지 말고 생성기를 돌린다.",
                "",
            ]
        ),
    )
    return summary


if __name__ == "__main__":
    summary = generate()
    counts = summary["counts"]
    print(dumps(counts))
