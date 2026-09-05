"""Case catalog: dimension, coveredCellNotEmpty, dry-run, verify, extract.

Each case is a distinct (table, mutation, mode, command) contract point.
No case invents a merge writer or resizes the table.
"""

from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass, field
from typing import Any, Callable, Iterable

from . import ISSUE, SCHEMA_VERSION, SKILL
from .catalog import FALLBACK_COMMAND, output_format_for
from .csv_codec import write_csv
from .judge import (
    Judgment,
    csv_to_table_envelope,
    export_table_entry,
    export_tables_envelope,
    judge_csv_to_table,
    silent_runtime,
    silent_usage,
    table_csv_entry,
    table_to_csv_envelope,
)
from .occupancy import Occupancy
from .tables import TableSpec, all_base_tables, documented_tables, pattern_tables


@dataclass
class Case:
    case_id: str
    family: str
    command: str
    title: str
    sample: str
    table_index: int
    rows: int
    cols: int
    mode: str
    argv: list[str]
    expect_exit: int
    writes: bool
    csv_roundtrip: str
    notes: str
    next_action: str
    documented: bool
    occupancy_public: dict[str, Any]
    csv_name: str | None
    csv_text: str | None
    invalid: list[dict[str, Any]]
    changed: list[dict[str, Any]]
    envelope: dict[str, Any]
    source_refs: list[str] = field(default_factory=list)

    def to_ledger_card(self) -> dict[str, Any]:
        reasons = [item.get("reason") for item in self.invalid]
        card: dict[str, Any] = {
            "caseId": self.case_id,
            "family": self.family,
            "command": self.command,
            "title": self.title,
            "sample": self.sample,
            "tableIndex": self.table_index,
            "size": f"{self.rows}x{self.cols}",
            "mode": self.mode,
            "argv": " ".join(self.argv),
            "expectExit": self.expect_exit,
            "writes": self.writes,
            "csvRoundtrip": self.csv_roundtrip,
            "reasons": reasons,
            "invalid": self.invalid,
            "changedCount": len(self.changed),
            "coveredCount": self.occupancy_public.get("coveredCount", 0),
            "mergedAnchorCount": self.occupancy_public.get("mergedAnchorCount", 0),
            "cellCount": self.occupancy_public.get("cellCount"),
            "areaFits": self.occupancy_public.get("areaFits"),
            "dryRun": self.envelope.get("dryRun"),
            "changedPages": self.envelope.get("changedPages"),
            "verify": self.envelope.get("verify"),
            "outputKept": (self.envelope.get("_skillMeta") or {}).get("outputKept"),
            "nextAction": self.next_action,
            "notes": self.notes,
            "documented": self.documented,
        }
        merged = self.occupancy_public.get("mergedAnchors") or []
        if merged:
            card["mergedAnchors"] = merged
        covered = self.occupancy_public.get("covered") or []
        if covered:
            card["covered"] = covered
        return card

    def to_public_dict(self) -> dict[str, Any]:
        occ = {
            key: self.occupancy_public[key]
            for key in (
                "rows",
                "cols",
                "cellCount",
                "coveredCount",
                "mergedAnchorCount",
                "areaSum",
                "gridArea",
                "areaFits",
            )
            if key in self.occupancy_public
        }
        occ["mergedAnchors"] = self.occupancy_public.get("mergedAnchors", [])
        occ["covered"] = self.occupancy_public.get("covered", [])
        env = self.envelope
        envelope_card = {
            "dryRun": env.get("dryRun"),
            "changedCount": env.get("changedCount"),
            "changedPages": env.get("changedPages"),
            "output": env.get("output"),
            "verify": env.get("verify"),
            "outputFormat": env.get("outputFormat"),
            "invalidCount": len(env.get("invalid") or []),
            "_skillMeta": env.get("_skillMeta"),
        }
        changed = self.changed if len(self.changed) <= 12 else self.changed[:3] + [
            {"_truncated": True, "omitted": len(self.changed) - 3}
        ]
        return {
            "schemaVersion": SCHEMA_VERSION,
            "caseId": self.case_id,
            "issue": ISSUE,
            "skill": SKILL,
            "family": self.family,
            "command": self.command,
            "title": self.title,
            "sample": self.sample,
            "tableIndex": self.table_index,
            "rows": self.rows,
            "cols": self.cols,
            "mode": self.mode,
            "argv": self.argv,
            "expectExit": self.expect_exit,
            "writes": self.writes,
            "csvRoundtrip": self.csv_roundtrip,
            "notes": self.notes,
            "nextAction": self.next_action,
            "documented": self.documented,
            "occupancy": occ,
            "csvName": self.csv_name,
            "csvLineCount": 0 if self.csv_text is None else self.csv_text.count("\n"),
            "invalid": self.invalid,
            "changed": changed,
            "envelope": envelope_card,
            "sourceRefs": self.source_refs,
        }


def _csv_name(case_id: str) -> str:
    return f"{case_id}.csv"


def _out_name(sample: str, case_id: str) -> str:
    if sample.lower().endswith(".hwpx"):
        return f"out/{case_id}.hwpx"
    return f"out/{case_id}.hwp"


def _argv_csv_to_table(
    sample: str,
    csv_name: str,
    table: int,
    mode: str,
    output: str | None,
) -> list[str]:
    argv = [
        "rhwp",
        "csv-to-table",
        sample,
        "--csv",
        csv_name,
        "--table",
        str(table),
        "--json",
    ]
    if mode == "dry-run":
        argv.insert(-1, "--dry-run")
        if output:
            argv.extend(["-o", output])
    elif mode == "verify":
        assert output is not None
        argv.extend(["-o", output, "--verify"])
    elif mode == "write":
        assert output is not None
        argv.extend(["-o", output])
    return argv


def _reasons(judgment: Judgment) -> list[str]:
    return [item.reason for item in judgment.invalid]


def _next_for_invalid(reasons: list[str], csv_roundtrip: str) -> str:
    if "coveredCellNotEmpty" in reasons or csv_roundtrip == "extract-only":
        return FALLBACK_COMMAND
    if "rowCountMismatch" in reasons or "colCountMismatch" in reasons:
        return "뽑은 CSV 를 표 치수에 맞춰 재생성"
    if "controlCharacter" in reasons:
        return "LF/TAB 을 공백으로 치환 후 --dry-run"
    if "csvParse" in reasons:
        return "CSV 라이브러리로 재생성"
    return "csv-to-table --verify"


def _refs(*names: str) -> list[str]:
    return list(names)


def _make_csv_case(
    *,
    case_id: str,
    family: str,
    title: str,
    table: TableSpec,
    records: list[list[str]] | None,
    raw: str | None,
    mode: str,
    notes: str,
    verify_diff: int | None = None,
    extra_refs: Iterable[str] = (),
) -> Case:
    csv_name = _csv_name(case_id)
    output = _out_name(table.sample, case_id)
    if raw is None:
        assert records is not None
        csv_text = write_csv(records)
    else:
        csv_text = raw
    judgment = judge_csv_to_table(table.occupancy, csv_text, table.index)
    envelope = csv_to_table_envelope(
        source=table.sample,
        table_index=table.index,
        occupancy=table.occupancy,
        judgment=judgment,
        csv_name=csv_name,
        mode=mode,
        output=output,
        verify_diff_count=verify_diff,
    )
    exit_code = envelope["_skillMeta"]["exit"]
    writes = bool(envelope["_skillMeta"]["writes"])
    reasons = _reasons(judgment)
    return Case(
        case_id=case_id,
        family=family,
        command="csv-to-table",
        title=title,
        sample=table.sample,
        table_index=table.index,
        rows=table.rows,
        cols=table.cols,
        mode=mode,
        argv=_argv_csv_to_table(table.sample, csv_name, table.index, mode, output),
        expect_exit=exit_code,
        writes=writes,
        csv_roundtrip=table.csv_roundtrip,
        notes=notes,
        next_action=_next_for_invalid(reasons, table.csv_roundtrip),
        documented=table.documented,
        occupancy_public=table.occupancy.to_public_dict(),
        csv_name=csv_name,
        csv_text=csv_text,
        invalid=[item.to_dict() for item in judgment.invalid],
        changed=[item.to_dict() for item in judgment.changed] if judgment.ok else [],
        envelope=envelope,
        source_refs=_refs(
            "tests/table_csv_contract.rs",
            ".agents/skills/rhwp-table-exchange/references/csv_to_table_contract.md",
            *extra_refs,
        ),
    )


def clone_grid(occupancy: Occupancy) -> list[list[str]]:
    return [row[:] for row in occupancy.grid_texts()]


def mutate_row_short(grid: list[list[str]], drop: int) -> list[list[str]]:
    if drop <= 0 or drop >= len(grid):
        drop = 1
    return [row[:] for row in grid[:-drop]]


def mutate_row_long(grid: list[list[str]], extra: int) -> list[list[str]]:
    out = [row[:] for row in grid]
    width = len(grid[0]) if grid else 0
    for i in range(extra):
        out.append([f"extra{i}_{c}" for c in range(width)])
    return out


def mutate_col_short(grid: list[list[str]], which: str) -> list[list[str]]:
    out: list[list[str]] = []
    for idx, row in enumerate(grid):
        if which == "all" or (which == "first" and idx == 0) or (which == "last" and idx == len(grid) - 1) or (which == "mid" and idx == len(grid) // 2):
            out.append(row[:-1] if len(row) > 1 else [])
        else:
            out.append(row[:])
    return out


def mutate_col_long(grid: list[list[str]], which: str) -> list[list[str]]:
    out: list[list[str]] = []
    for idx, row in enumerate(grid):
        copied = row[:]
        if which == "all" or (which == "first" and idx == 0) or (which == "last" and idx == len(grid) - 1) or (which == "mid" and idx == len(grid) // 2):
            copied.append("남는열")
        out.append(copied)
    return out


def mutate_header_drop(grid: list[list[str]]) -> list[list[str]]:
    return [row[:] for row in grid[1:]]


def mutate_empty() -> list[list[str]]:
    return []


def mutate_edit_body(grid: list[list[str]], token: str) -> list[list[str]]:
    out = [row[:] for row in grid]
    if len(out) >= 2 and out[1]:
        out[1][0] = token
    elif out and out[0]:
        out[0][0] = token
    return out


def mutate_edit_many(grid: list[list[str]], start_row: int = 1) -> list[list[str]]:
    out = [row[:] for row in grid]
    for r in range(start_row, len(out)):
        for c in range(len(out[r])):
            if out[r][c] == "":
                out[r][c] = f"v{r}_{c}"
            else:
                out[r][c] = f"{out[r][c]}*"
    return out


def mutate_fill_covered(occupancy: Occupancy, which: str, value: str) -> list[list[str]]:
    grid = clone_grid(occupancy)
    covered = occupancy.covered_coords()
    if not covered:
        return grid
    if which == "first":
        r, c = covered[0]
        grid[r][c] = value
    elif which == "last":
        r, c = covered[-1]
        grid[r][c] = value
    elif which == "all":
        for r, c in covered:
            grid[r][c] = f"{value}{r}_{c}"
    elif which == "second" and len(covered) > 1:
        r, c = covered[1]
        grid[r][c] = value
    return grid


def mutate_control(grid: list[list[str]], kind: str) -> tuple[list[list[str]] | None, str | None]:
    if not grid or not grid[0]:
        return grid, None
    r = 1 if len(grid) > 1 else 0
    c = 0
    if kind == "lf":
        raw_rows = [row[:] for row in grid]
        raw_rows[r][c] = "줄\n바꿈"
        return None, write_csv(raw_rows)
    if kind == "tab":
        raw_rows = [row[:] for row in grid]
        raw_rows[r][c] = "탭\t값"
        return None, write_csv(raw_rows)
    if kind == "crlf_in_quotes":
        # write_csv will quote the LF
        raw_rows = [row[:] for row in grid]
        raw_rows[r][c] = "여러\n줄"
        return None, write_csv(raw_rows)
    return grid, None


DIMENSION_TABLES = (
    "hwp_table_test_t0",
    "issue2007_t0",
    "issue2007_t1",
    "chujin",
    "hwpx_basic_01",
    "jichi_body_12",
    "treatise_body",
    "shape_2x2_m0",
    "shape_3x4_m0",
    "shape_5x5_m0",
    "shape_8x4_m0",
    "shape_9x2_m0",
)


def _tables_by_id() -> dict[str, TableSpec]:
    return {table.spec_id: table for table in all_base_tables()}


def build_dimension_cases() -> list[Case]:
    by_id = _tables_by_id()
    cases: list[Case] = []
    mutations: list[tuple[str, str, Callable[[list[list[str]]], list[list[str]]]]] = [
        ("row_short_1", "행 1개 부족 — 한 칸도 안 씀", lambda g: mutate_row_short(g, 1)),
        ("row_long_1", "행 1개 초과", lambda g: mutate_row_long(g, 1)),
        ("col_short_all", "모든 행 열 부족", lambda g: mutate_col_short(g, "all")),
        ("col_long_first", "0행만 남는열", lambda g: mutate_col_long(g, "first")),
        ("both_short", "행·열 동시 부족 — 이유 전부 수집", lambda g: mutate_col_short(mutate_row_short(g, 1), "all")),
        ("header_drop", "헤더 행을 빼면 치수도 깨진다", mutate_header_drop),
    ]
    for spec_id in DIMENSION_TABLES:
        table = by_id[spec_id]
        if table.rows < 2 and spec_id != "wrapper_1x1":
            continue
        base = clone_grid(table.occupancy)
        for mut_id, title, fn in mutations:
            if table.rows == 1 and mut_id.startswith("row_short"):
                continue
            if table.cols == 1 and mut_id.startswith("col_short"):
                continue
            records = fn(base)
            cases.append(
                _make_csv_case(
                    case_id=f"D-{spec_id}-{mut_id}",
                    family="dimension",
                    title=f"{spec_id} {title}",
                    table=table,
                    records=records,
                    raw=None,
                    mode="dry-run",
                    notes=f"{table.rows}×{table.cols} 치수 계약. 조용한 절삭 금지.",
                    extra_refs=(
                        ".agents/skills/rhwp-table-exchange/references/csv_to_table_contract.md",
                    ),
                )
            )
        cases.append(
            _make_csv_case(
                case_id=f"D-{spec_id}-empty",
                family="dimension",
                title=f"{spec_id} 빈 CSV",
                table=table,
                records=[],
                raw=None,
                mode="dry-run",
                notes="빈 CSV 는 rowCountMismatch.",
            )
        )
        if table.cols >= 2 and table.rows >= 3 and spec_id in {
            "hwp_table_test_t0",
            "chujin",
            "shape_5x5_m0",
        }:
            ragged = clone_grid(table.occupancy)
            ragged[1] = ragged[1][:-1]
            cases.append(
                _make_csv_case(
                    case_id=f"D-{spec_id}-ragged-mid",
                    family="dimension",
                    title=f"{spec_id} 중간 행만 열 부족",
                    table=table,
                    records=ragged,
                    raw=None,
                    mode="write",
                    notes="colCountMismatch 는 어긋난 행마다 한 줄.",
                )
            )
    # documented table-001 both mismatch (2x2 csv vs 19x9)
    table = by_id["table_001"]
    cases.append(
        _make_csv_case(
            case_id="D-table_001-both-2x2",
            family="dimension",
            title="table-001 2×2 CSV vs 19×9 — 두 이유 전부",
            table=table,
            records=[["a", "b"], ["c", "d"]],
            raw=None,
            mode="dry-run",
            notes="playbook §10-5. rowCountMismatch + colCountMismatch.",
            extra_refs=(
                ".agents/skills/rhwp-table-exchange/fixtures/envelopes/csv_to_table_table001_both_mismatch.json",
            ),
        )
    )
    # unclosed quote
    for spec_id in ("hwp_table_test_t0", "issue2007_t1", "shape_3x4_m0"):
        table = by_id[spec_id]
        cases.append(
            _make_csv_case(
                case_id=f"D-{spec_id}-unclosed-quote",
                family="dimension",
                title=f"{spec_id} 닫히지 않은 따옴표는 csvParse",
                table=table,
                records=None,
                raw='"닫히지 않은 따옴표',
                mode="dry-run",
                notes="malformed_csv_is_invalid_not_a_panic.",
            )
        )
    return cases


def build_covered_cases() -> list[Case]:
    cases: list[Case] = []
    seen: set[str] = set()
    for table in pattern_tables():
        if table.occupancy.covered_count == 0:
            continue
        if table.spec_id in seen:
            continue
        seen.add(table.spec_id)
        for which, label in (
            ("first", "첫 덮인 칸에 값"),
            ("last", "마지막 덮인 칸에 값"),
            ("all", "덮인 칸 전부에 값"),
        ):
            if which == "all" and table.occupancy.covered_count > 24:
                # still real: every covered cell is a distinct invalid
                pass
            records = mutate_fill_covered(table.occupancy, which, "덮인칸값")
            cases.append(
                _make_csv_case(
                    case_id=f"C-{table.spec_id}-{which}",
                    family="covered",
                    title=f"{table.spec_id} {label}",
                    table=table,
                    records=records,
                    raw=None,
                    mode="dry-run",
                    notes=f"{table.notes} covered={table.occupancy.covered_count}. 한 칸도 안 씀.",
                    extra_refs=(
                        ".agents/skills/rhwp-table-exchange/references/csv_to_table_contract.md",
                        "tests/table_csv_contract.rs value_in_a_merged_covered_cell_is_invalid",
                    ),
                )
            )
        if table.occupancy.covered_count > 1 and table.spec_id in {
            "table001_header",
            "block_2x2",
            "header_plus_note",
            "many_small",
        }:
            records = mutate_fill_covered(table.occupancy, "second", "두번째덮인")
            cases.append(
                _make_csv_case(
                    case_id=f"C-{table.spec_id}-second",
                    family="covered",
                    title=f"{table.spec_id} 두 번째 덮인 칸",
                    table=table,
                    records=records,
                    raw=None,
                    mode="write",
                    notes="앵커가 아니라 덮인 좌표. set-cell 로 갈아탄다.",
                )
            )
        if table.spec_id in {"table001_header", "block_2x2", "colspan2_r0c0", "rowspan3_note"}:
            empty = clone_grid(table.occupancy)
            cases.append(
                _make_csv_case(
                    case_id=f"C-{table.spec_id}-empty-covered-ok-dim",
                    family="covered",
                    title=f"{table.spec_id} 덮인 칸 공란은 치수 통과",
                    table=table,
                    records=empty,
                    raw=None,
                    mode="dry-run",
                    notes="덮인 칸 빈 문자열은 coveredCellNotEmpty 가 아니다. 그래도 병합 표 왕복은 extract-only.",
                )
            )
            raw_rows = clone_grid(table.occupancy)
            ar, ac = table.occupancy.anchor_coords()[0]
            raw_rows[ar][ac] = "줄\n바꿈"
            cases.append(
                _make_csv_case(
                    case_id=f"C-{table.spec_id}-control-on-anchor",
                    family="covered",
                    title=f"{table.spec_id} 앵커 제어문자",
                    table=table,
                    records=None,
                    raw=write_csv(raw_rows),
                    mode="dry-run",
                    notes="병합 여부와 별개로 controlCharacter.",
                )
            )
    return cases


def build_dry_run_cases() -> list[Case]:
    by_id = _tables_by_id()
    cases: list[Case] = []
    success_ids = (
        "hwp_table_test_t0",
        "issue2007_t1",
        "chujin",
        "hwpx_basic_01",
        "jichi_body_12",
        "shape_3x4_m0",
        "shape_5x5_m0",
        "shape_8x4_m0",
        "shape_9x2_m0",
    )
    for spec_id in success_ids:
        table = by_id[spec_id]
        edited = mutate_edit_many(clone_grid(table.occupancy), 1 if table.rows > 1 else 0)
        cases.append(
            _make_csv_case(
                case_id=f"R-{spec_id}-preview",
                family="dry-run",
                title=f"{spec_id} dry-run 선확인 — 디스크 무변경",
                table=table,
                records=edited,
                raw=None,
                mode="dry-run",
                notes="changedPages=null, output=null, -o 를 줘도 파일 없음.",
                extra_refs=(
                    ".agents/skills/rhwp-table-exchange/references/dry_run_verify.md",
                    "tests/table_csv_contract.rs dry_run_writes_no_file",
                ),
            )
        )
        identical = clone_grid(table.occupancy)
        cases.append(
            _make_csv_case(
                case_id=f"R-{spec_id}-identical",
                family="dry-run",
                title=f"{spec_id} 동일 CSV dry-run — changedCount 0",
                table=table,
                records=identical,
                raw=None,
                mode="dry-run",
                notes="changedCount 0 + invalid [] 는 성공. 실패가 아니다.",
            )
        )
    # recipe02 canonical 9 changes
    table = by_id["hwp_table_test_t0"]
    recipe = [
        ["제목", "담당자", "세부 내용"],
        ["서버 이관", "홍길동", "1차 완료"],
        ["DB 백업", "김철수", "진행중"],
        ["문서 정리", "박영희", "대기"],
    ]
    cases.append(
        _make_csv_case(
            case_id="R-recipe02-edited",
            family="dry-run",
            title="레시피 02 편집본 dry-run changedCount 9",
            table=table,
            records=recipe,
            raw=None,
            mode="dry-run",
            notes="헤더 3칸은 old==new 라 changed 에서 빠진다. 12-3=9.",
        )
    )
    # control characters on dry-run
    for spec_id, kind in (
        ("hwp_table_test_t0", "lf"),
        ("hwp_table_test_t0", "tab"),
        ("issue2007_t1", "lf"),
        ("chujin", "tab"),
        ("shape_3x4_m0", "crlf_in_quotes"),
    ):
        table = by_id[spec_id]
        recs, raw = mutate_control(clone_grid(table.occupancy), kind)
        cases.append(
            _make_csv_case(
                case_id=f"R-{spec_id}-ctrl-{kind}",
                family="dry-run",
                title=f"{spec_id} dry-run {kind} 제어문자",
                table=table,
                records=recs,
                raw=raw,
                mode="dry-run",
                notes="RFC 인용으로 감싸도 파싱된 값을 본다.",
            )
        )
    return cases


def build_verify_cases() -> list[Case]:
    by_id = _tables_by_id()
    cases: list[Case] = []
    for spec_id in (
        "hwp_table_test_t0",
        "issue2007_t1",
        "chujin",
        "hwpx_basic_01",
        "jichi_body_12",
        "shape_3x4_m0",
        "shape_5x5_m0",
        "shape_8x4_m0",
    ):
        table = by_id[spec_id]
        identical = clone_grid(table.occupancy)
        cases.append(
            _make_csv_case(
                case_id=f"V-{spec_id}-identical",
                family="verify",
                title=f"{spec_id} 왕복 동일 — identical true, changed 0",
                table=table,
                records=identical,
                raw=None,
                mode="verify",
                verify_diff=0,
                notes=f"outputFormat={output_format_for(table.sample)}. identical_csv_writes_nothing_and_verifies.",
                extra_refs=("tests/table_csv_contract.rs identical_csv_writes_nothing_and_verifies",),
            )
        )
        edited = mutate_edit_body(clone_grid(table.occupancy), "표값-2026")
        cases.append(
            _make_csv_case(
                case_id=f"V-{spec_id}-write-ok",
                family="verify",
                title=f"{spec_id} 한 칸 수정 후 verify 통과",
                table=table,
                records=edited,
                raw=None,
                mode="verify",
                verify_diff=0,
                notes="저장 + 재파싱. identical true.",
            )
        )
        cases.append(
            _make_csv_case(
                case_id=f"V-{spec_id}-write-no-verify-flag",
                family="verify",
                title=f"{spec_id} --verify 없으면 verify 키는 null",
                table=table,
                records=edited,
                raw=None,
                mode="write",
                notes="null 을 통과로 읽지 마라. 확인하지 않은 것이다.",
            )
        )
    # verify failures keep output
    fail_specs = (
        ("hwp_table_test_t0", 2),
        ("issue2007_t1", 1),
        ("chujin", 3),
        ("hwpx_basic_01", 1),
        ("shape_5x5_m0", 4),
        ("jichi_body_12", 2),
    )
    for spec_id, diff in fail_specs:
        table = by_id[spec_id]
        edited = mutate_edit_many(clone_grid(table.occupancy), 1 if table.rows > 1 else 0)
        cases.append(
            _make_csv_case(
                case_id=f"V-{spec_id}-exit3-diff{diff}",
                family="verify",
                title=f"{spec_id} verify 실패 diffCount={diff} — exit 3, 산출 유지",
                table=table,
                records=edited,
                raw=None,
                mode="verify",
                verify_diff=diff,
                notes="고장이 아니라 판정 데이터. invalid [] 이고 outputKept true.",
                extra_refs=(
                    ".agents/skills/rhwp-table-exchange/references/dry_run_verify.md",
                    ".agents/skills/rhwp-table-exchange/fixtures/envelopes/csv_to_table_verify_fail.json",
                ),
            )
        )
    # recipe02 write+verify success
    table = by_id["hwp_table_test_t0"]
    recipe = [
        ["제목", "담당자", "세부 내용"],
        ["서버 이관", "홍길동", "1차 완료"],
        ["DB 백업", "김철수", "진행중"],
        ["문서 정리", "박영희", "대기"],
    ]
    cases.append(
        _make_csv_case(
            case_id="V-recipe02-verify-ok",
            family="verify",
            title="레시피 02 저장+verify 성공",
            table=table,
            records=recipe,
            raw=None,
            mode="verify",
            verify_diff=0,
            notes="changedCount 9, identical true, outputFormat hwp5.",
        )
    )
    return cases


def build_export_tables_cases() -> list[Case]:
    cases: list[Case] = []
    # per documented sample: scan envelope
    grouped: dict[str, list[TableSpec]] = {}
    for table in documented_tables():
        grouped.setdefault(table.sample, []).append(table)
    for sample, tables in grouped.items():
        entries = []
        for table in tables:
            full = table.rows * table.cols <= 40
            entries.append(
                export_table_entry(
                    table.occupancy,
                    table.index,
                    paragraph=table.paragraph,
                    control=table.control,
                    container=table.container,
                    include_full_cells=full,
                )
            )
        entries.sort(key=lambda item: item["index"])
        envelope = export_tables_envelope(source=sample, tables=entries)
        first = tables[0]
        cases.append(
            Case(
                case_id=f"E-scan-{first.spec_id}",
                family="export-tables",
                command="export-tables",
                title=f"{sample} 좌표·병합 스캔",
                sample=sample,
                table_index=first.index,
                rows=first.rows,
                cols=first.cols,
                mode="scan",
                argv=["rhwp", "export-tables", sample, "--json"],
                expect_exit=0,
                writes=False,
                csv_roundtrip="pick-first" if len(tables) > 1 else first.csv_roundtrip,
                notes=f"tableCount={len(entries)}. index 는 배열 순번이 아니다.",
                next_action="containerPath 없는 표의 index 로 --table",
                documented=True,
                occupancy_public=first.occupancy.to_public_dict(),
                csv_name=None,
                csv_text=None,
                invalid=[],
                changed=[],
                envelope=envelope,
                source_refs=_refs(
                    ".agents/skills/rhwp-table-exchange/references/export_tables_matrix.md",
                    "tests/table_extract_json_contract.rs",
                ),
            )
        )
    # occupancy-only cases for merge patterns
    for table in pattern_tables():
        if table.occupancy.merged_anchor_count == 0 and not table.documented:
            continue
        full = table.rows * table.cols <= 36
        entry = export_table_entry(
            table.occupancy,
            table.index,
            container=table.container,
            include_full_cells=full,
        )
        envelope = export_tables_envelope(source=table.sample, tables=[entry])
        cases.append(
            Case(
                case_id=f"E-occ-{table.spec_id}",
                family="export-tables",
                command="export-tables",
                title=f"{table.spec_id} 점유 행렬 cellCount={table.occupancy.cell_count}",
                sample=table.sample,
                table_index=table.index,
                rows=table.rows,
                cols=table.cols,
                mode="scan",
                argv=["rhwp", "export-tables", table.sample, "--json"],
                expect_exit=0,
                writes=False,
                csv_roundtrip=table.csv_roundtrip,
                notes=(
                    f"covered={table.occupancy.covered_count}, "
                    f"areaSum={table.occupancy.area_sum} <= {table.rows * table.cols}, "
                    f"roundtrip={table.csv_roundtrip}."
                ),
                next_action=(
                    FALLBACK_COMMAND
                    if table.csv_roundtrip in {"extract-only", "forbidden", "outer-only"}
                    else "table-to-csv --table N"
                ),
                documented=table.documented,
                occupancy_public=table.occupancy.to_public_dict(),
                csv_name=None,
                csv_text=None,
                invalid=[],
                changed=[],
                envelope=envelope,
                source_refs=_refs(
                    ".agents/skills/rhwp-table-exchange/references/export_tables_matrix.md"
                ),
            )
        )
    # silent failures
    cases.append(
        Case(
            case_id="E-missing-file",
            family="export-tables",
            command="export-tables",
            title="없는 파일은 exit 1 · stdout 0바이트",
            sample="samples/does-not-exist.hwp",
            table_index=0,
            rows=0,
            cols=0,
            mode="scan",
            argv=["rhwp", "export-tables", "samples/does-not-exist.hwp", "--json"],
            expect_exit=1,
            writes=False,
            csv_roundtrip="forbidden",
            notes="원본 불변. 단건 실패는 봉투가 없다.",
            next_action="경로를 고친다",
            documented=True,
            occupancy_public={"rows": 0, "cols": 0, "cellCount": 0, "coveredCount": 0},
            csv_name=None,
            csv_text=None,
            invalid=[],
            changed=[],
            envelope=silent_runtime(
                "export-tables",
                ["rhwp", "export-tables", "samples/does-not-exist.hwp", "--json"],
                "파일을 열 수 없습니다",
            ),
            source_refs=_refs("tests/table_extract_json_contract.rs"),
        )
    )
    cases.append(
        Case(
            case_id="E-usage-two-files",
            family="export-tables",
            command="export-tables",
            title="파일 positional 두 개는 exit 2 · stdout 0바이트",
            sample="samples/hwp_table_test.hwp",
            table_index=0,
            rows=0,
            cols=0,
            mode="usage",
            argv=[
                "rhwp",
                "export-tables",
                "samples/hwp_table_test.hwp",
                "samples/table-001.hwp",
            ],
            expect_exit=2,
            writes=False,
            csv_roundtrip="forbidden",
            notes="사용법 오류는 치수 봉투가 아니다.",
            next_action="인자 조립을 고친다",
            documented=True,
            occupancy_public={"rows": 0, "cols": 0, "cellCount": 0, "coveredCount": 0},
            csv_name=None,
            csv_text=None,
            invalid=[],
            changed=[],
            envelope=silent_usage(
                "export-tables",
                [
                    "rhwp",
                    "export-tables",
                    "samples/hwp_table_test.hwp",
                    "samples/table-001.hwp",
                ],
            ),
            source_refs=_refs("tests/table_extract_json_contract.rs"),
        )
    )
    return cases


def build_table_to_csv_cases() -> list[Case]:
    cases: list[Case] = []
    by_id = _tables_by_id()
    extract_ids = (
        "hwp_table_test_t0",
        "issue2007_t0",
        "issue2007_t1",
        "chujin",
        "hwpx_basic_01",
        "jichi_body_12",
        "table_001",
        "shape_3x4_m0",
        "shape_5x5_m0",
        "colspan2_r0c0",
        "rowspan3_note",
        "block_2x2",
        "header_plus_note",
        "table001_header",
        "many_small",
    )
    for spec_id in extract_ids:
        table = by_id[spec_id]
        entry = table_csv_entry(table.occupancy, table.index, output=f"table{table.index}.csv")
        envelope = table_to_csv_envelope(
            source=table.sample,
            tables=[entry],
            bom=False,
            output=f"table{table.index}.csv",
            output_is_dir=False,
        )
        cases.append(
            Case(
                case_id=f"T-{spec_id}-extract",
                family="table-to-csv",
                command="table-to-csv",
                title=f"{spec_id} 격자 CSV — 덮인 칸은 빈 필드",
                sample=table.sample,
                table_index=table.index,
                rows=table.rows,
                cols=table.cols,
                mode="extract",
                argv=[
                    "rhwp",
                    "table-to-csv",
                    table.sample,
                    "--table",
                    str(table.index),
                    "-o",
                    f"table{table.index}.csv",
                    "--json",
                ],
                expect_exit=0,
                writes=True,
                csv_roundtrip=table.csv_roundtrip,
                notes="행마다 필드 수 = colCount. 병합 채움이 빠지면 열이 밀린다.",
                next_action=(
                    FALLBACK_COMMAND
                    if table.csv_roundtrip == "extract-only"
                    else "외부 편집 후 csv-to-table --dry-run"
                ),
                documented=table.documented,
                occupancy_public=table.occupancy.to_public_dict(),
                csv_name=f"table{table.index}.csv",
                csv_text=entry["csv"],
                invalid=[],
                changed=[],
                envelope=envelope,
                source_refs=_refs(
                    ".agents/skills/rhwp-table-exchange/references/table_to_csv_envelopes.md",
                    "tests/table_csv_contract.rs merged_table_csv_is_a_full_rectangle",
                ),
            )
        )
        bom_env = table_to_csv_envelope(
            source=table.sample,
            tables=[
                table_csv_entry(
                    table.occupancy, table.index, output=f"table{table.index}_bom.csv"
                )
            ],
            bom=True,
            output=f"table{table.index}_bom.csv",
            output_is_dir=False,
        )
        cases.append(
            Case(
                case_id=f"T-{spec_id}-bom",
                family="table-to-csv",
                command="table-to-csv",
                title=f"{spec_id} --bom 은 파일만, 봉투 csv 에는 U+FEFF 없음",
                sample=table.sample,
                table_index=table.index,
                rows=table.rows,
                cols=table.cols,
                mode="extract",
                argv=[
                    "rhwp",
                    "table-to-csv",
                    table.sample,
                    "--table",
                    str(table.index),
                    "-o",
                    f"table{table.index}_bom.csv",
                    "--bom",
                    "--json",
                ],
                expect_exit=0,
                writes=True,
                csv_roundtrip=table.csv_roundtrip,
                notes="bom_flag_only_affects_the_file_not_the_envelope.",
                next_action="엑셀은 파일만 연다. 봉투 csv 를 붙여넣지 마라.",
                documented=table.documented,
                occupancy_public=table.occupancy.to_public_dict(),
                csv_name=f"table{table.index}_bom.csv",
                csv_text=entry["csv"],
                invalid=[],
                changed=[],
                envelope=bom_env,
                source_refs=_refs("tests/table_csv_contract.rs bom_flag_only_affects_the_file_not_the_envelope"),
            )
        )
    # rfc4180 quoting
    table = by_id["hwp_table_test_t0"]
    quoted = clone_grid(table.occupancy)
    quoted[0][0] = '가,나"다'
    # pretend the document already has that cell (extract after set-cell)
    from .occupancy import Anchor, anchors_from_spans, build_occupancy

    texts = {(0, 0): '가,나"다', (0, 1): "담당자", (0, 2): "세부 내용"}
    occ = build_occupancy(4, 3, anchors_from_spans(4, 3, (), texts=texts))
    quoted_table = deepcopy(table)
    quoted_table.occupancy = occ
    entry = table_csv_entry(occ, 0, output="quoted.csv")
    cases.append(
        Case(
            case_id="T-recipe02-rfc4180",
            family="table-to-csv",
            command="table-to-csv",
            title="쉼표·따옴표는 RFC 4180 인용. 열이 밀리지 않는다",
            sample=table.sample,
            table_index=0,
            rows=4,
            cols=3,
            mode="extract",
            argv=["rhwp", "table-to-csv", table.sample, "--table", "0", "--json"],
            expect_exit=0,
            writes=False,
            csv_roundtrip="allowed",
            notes='값은 가,나"다 . CSV 는 "가,나""다".',
            next_action="판독기가 \"\" 를 한 따옴표로 되돌리는지 본다",
            documented=True,
            occupancy_public=occ.to_public_dict(),
            csv_name="quoted.csv",
            csv_text=entry["csv"],
            invalid=[],
            changed=[],
            envelope=table_to_csv_envelope(
                source=table.sample,
                tables=[entry],
                bom=False,
                output=None,
                output_is_dir=False,
            ),
            source_refs=_refs("tests/table_csv_contract.rs rfc4180_quoting_survives_a_round_trip_through_the_document"),
        )
    )
    # unknown table
    cases.append(
        Case(
            case_id="T-unknown-table-99999",
            family="table-to-csv",
            command="table-to-csv",
            title="--table 99999 는 exit 1 · stdout 0바이트",
            sample="samples/hwp_table_test.hwp",
            table_index=99999,
            rows=0,
            cols=0,
            mode="extract",
            argv=["rhwp", "table-to-csv", "samples/hwp_table_test.hwp", "--table", "99999", "--json"],
            expect_exit=1,
            writes=False,
            csv_roundtrip="forbidden",
            notes="본문 최상위 표 없음. export-tables 의 실제 index 를 본다.",
            next_action="export-tables --json",
            documented=True,
            occupancy_public={"rows": 0, "cols": 0, "cellCount": 0, "coveredCount": 0},
            csv_name=None,
            csv_text=None,
            invalid=[],
            changed=[],
            envelope=silent_runtime(
                "table-to-csv",
                ["rhwp", "table-to-csv", "samples/hwp_table_test.hwp", "--table", "99999", "--json"],
                "본문 최상위 표가 없습니다",
            ),
            source_refs=_refs("tests/table_csv_contract.rs unknown_top_level_table_is_a_runtime_error_with_silent_stdout"),
        )
    )
    for argv, cid, title in (
        (["rhwp", "table-to-csv"], "T-usage-no-file", "table-to-csv 인자 없음"),
        (["rhwp", "csv-to-table"], "T-usage-csv-no-args", "csv-to-table 인자 없음"),
        (
            ["rhwp", "csv-to-table", "samples/hwp_table_test.hwp"],
            "T-usage-csv-no-csv-flag",
            "csv-to-table --csv/--table 없음",
        ),
    ):
        cases.append(
            Case(
                case_id=cid,
                family="table-to-csv",
                command=argv[1],
                title=f"{title} — exit 2 · stdout 0바이트",
                sample="samples/hwp_table_test.hwp",
                table_index=0,
                rows=0,
                cols=0,
                mode="usage",
                argv=argv,
                expect_exit=2,
                writes=False,
                csv_roundtrip="forbidden",
                notes="조립 버그. 치수 봉투가 아니다.",
                next_action="플래그를 고친다",
                documented=True,
                occupancy_public={"rows": 0, "cols": 0, "cellCount": 0, "coveredCount": 0},
                csv_name=None,
                csv_text=None,
                invalid=[],
                changed=[],
                envelope=silent_usage(argv[1], argv),
                source_refs=_refs("tests/table_csv_contract.rs missing_arguments_are_usage_errors_with_silent_stdout"),
            )
        )
    # multi-table folder harvest
    t0 = by_id["hwp_table_test_t0"]
    t1 = by_id["issue2007_t1"]
    # represent multi-table-001 as six synthetic 2x2 for harvest shape
    harvest_tables = []
    for idx in range(6):
        from .tables import unmerged

        spec = unmerged(f"multi_{idx}", 2, 2, "samples/multi-table-001.hwp", index=idx)
        harvest_tables.append(table_csv_entry(spec.occupancy, idx, output=f"table{idx}.csv"))
    cases.append(
        Case(
            case_id="T-multi-folder",
            family="table-to-csv",
            command="table-to-csv",
            title="--table 없이 -o 폴더 → table<index>.csv",
            sample="samples/multi-table-001.hwp",
            table_index=0,
            rows=2,
            cols=2,
            mode="extract",
            argv=["rhwp", "table-to-csv", "samples/multi-table-001.hwp", "-o", "out/tables", "--json"],
            expect_exit=0,
            writes=True,
            csv_roundtrip="pick-first",
            notes="되돌릴 때는 표마다 --table index 를 따로.",
            next_action="export-tables 로 index 확인 후 하나만 csv-to-table",
            documented=True,
            occupancy_public=t0.occupancy.to_public_dict(),
            csv_name="out/tables",
            csv_text=None,
            invalid=[],
            changed=[],
            envelope=table_to_csv_envelope(
                source="samples/multi-table-001.hwp",
                tables=harvest_tables,
                bom=False,
                output="out/tables",
                output_is_dir=True,
            ),
            source_refs=_refs(".agents/skills/rhwp-table-exchange/references/table_to_csv_envelopes.md"),
        )
    )
    _ = t1
    return cases


def build_all_cases() -> list[Case]:
    cases = (
        build_dimension_cases()
        + build_covered_cases()
        + build_dry_run_cases()
        + build_verify_cases()
        + build_export_tables_cases()
        + build_table_to_csv_cases()
    )
    ids = [case.case_id for case in cases]
    if len(ids) != len(set(ids)):
        dup = sorted({item for item in ids if ids.count(item) > 1})
        raise RuntimeError(f"duplicate case ids: {dup}")
    cases.sort(key=lambda case: case.case_id)
    return cases


CASES = build_all_cases()


def cases_by_family() -> dict[str, list[Case]]:
    grouped: dict[str, list[Case]] = {}
    for case in CASES:
        grouped.setdefault(case.family, []).append(case)
    return grouped


def assert_catalog_coverage() -> dict[str, int]:
    grouped = cases_by_family()
    counts = {family: len(items) for family, items in grouped.items()}
    if counts.get("dimension", 0) < 40:
        raise AssertionError(f"dimension cases too few: {counts}")
    if counts.get("covered", 0) < 20:
        raise AssertionError(f"covered cases too few: {counts}")
    if counts.get("dry-run", 0) < 10:
        raise AssertionError(f"dry-run cases too few: {counts}")
    if counts.get("verify", 0) < 10:
        raise AssertionError(f"verify cases too few: {counts}")
    if counts.get("export-tables", 0) < 10:
        raise AssertionError(f"export-tables cases too few: {counts}")
    if counts.get("table-to-csv", 0) < 10:
        raise AssertionError(f"table-to-csv cases too few: {counts}")
    return counts
