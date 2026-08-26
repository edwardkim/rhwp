#!/usr/bin/env python3
"""Emit the V-bon Best-of-N corpus of distinct candidate sets.

Each set is N final-outcome envelopes (dry-run / --verify / ir-diff) plus
an expectedRank computed by rank.py. process_steps is never written.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from envelopes import lift_candidate_record  # noqa: E402
from rank import CLAIM_ID, SCHEMA_VERSION, rank_candidates  # noqa: E402
from schema import CommandFamily, Mode, invalid_fingerprint  # noqa: E402

CORPUS = HERE / "corpus"
DEFAULT_MIN_LINES = 102000
DEFAULT_MIN_SETS = 1666
SETS_PER_SHARD = 64

AGENCIES = (
    "법제처",
    "행정안전부",
    "국세청",
    "대법원",
    "특허청",
    "교육부",
    "보건복지부",
    "국토교통부",
    "고용노동부",
    "외교부",
    "기획재정부",
    "공정거래위원회",
    "금융위원회",
    "방송통신위원회",
    "개인정보보호위원회",
    "국민권익위원회",
    "국가인권위원회",
    "통계청",
    "기상청",
    "관세청",
    "검찰청",
    "경찰청",
    "소방청",
    "해양경찰청",
    "병무청",
    "산림청",
    "농촌진흥청",
    "중소벤처기업부",
    "과학기술정보통신부",
    "문화체육관광부",
    "환경부",
    "해양수산부",
)

KINDS = (
    "고시",
    "훈령",
    "예규",
    "공고",
    "지침",
    "서식",
    "질의회신",
    "업무계획",
    "예산서",
    "회의록",
    "계약서",
    "점검표",
)

YEARS = tuple(range(2016, 2027))

FIELD_NAMES = (
    "신청인",
    "생년월일",
    "주소",
    "연락처",
    "전자우편",
    "사업자등록번호",
    "법인명",
    "대표자",
    "담당부서",
    "담당자",
    "문서번호",
    "시행일자",
    "수신",
    "제목",
    "금액",
    "계좌번호",
    "주민등록번호",
    "면허번호",
    "시설위치",
    "처리기한",
)

CELL_LABELS = (
    "구분",
    "성명",
    "직급",
    "소속",
    "연락처",
    "비고",
    "수량",
    "단가",
    "금액",
    "일자",
)

IR_CATEGORIES = (
    "para_count",
    "char_offsets",
    "table_cell",
    "ctrl_id",
    "section_def",
    "cc",
    "bullet",
    "header_footer",
)


@dataclass(frozen=True)
class OutcomeCell:
    name: str
    invalid: Any
    exit_class: int
    identical: bool | None
    offset: int
    absolute: int | None = None

    def changed_count(self, intended: int) -> int:
        if self.absolute is not None:
            return self.absolute
        return max(0, intended + self.offset)


def _inv(reason: str, **extra: Any) -> list[dict[str, Any]]:
    item = {"reason": reason, **extra}
    return [item]


VERIFY_CELLS: tuple[OutcomeCell, ...] = (
    OutcomeCell("exact_ok", [], 0, True, 0),
    OutcomeCell("over_1", [], 0, True, 1),
    OutcomeCell("over_2", [], 0, True, 2),
    OutcomeCell("over_3", [], 0, True, 3),
    OutcomeCell("over_5", [], 0, True, 5),
    OutcomeCell("over_8", [], 0, True, 8),
    OutcomeCell("under_1", [], 0, True, -1),
    OutcomeCell("under_2", [], 0, True, -2),
    OutcomeCell("under_3", [], 0, True, -3),
    OutcomeCell("zero_change", [], 0, True, 0, 0),
    OutcomeCell("verify_fail", [], 3, False, 0),
    OutcomeCell("verify_fail_over", [], 3, False, 2),
    OutcomeCell("verify_fail_big", [], 3, False, 8),
    OutcomeCell("exit0_ident_false", [], 0, False, 0),
    OutcomeCell("exit3_ident_true", [], 3, True, 0),
    OutcomeCell("page_fail", [], 4, False, 0),
    OutcomeCell("page_fail_over", [], 4, False, 3),
    OutcomeCell("io_missing", [], 1, None, 0, 0),
    OutcomeCell("usage_flag", [], 2, None, 0, 0),
    OutcomeCell("invalid_dim", _inv("rowCountMismatch", expected=12, actual=9), 2, None, 0, 0),
    OutcomeCell("invalid_field", _inv("notFound", name="신청인"), 2, None, 0, 0),
    OutcomeCell("invalid_covered", _inv("coveredCellNotEmpty", row=2, col=1), 2, None, 0, 0),
    OutcomeCell("invalid_control", _inv("controlCharacter", row=1, col=0), 2, None, 0, 0),
    OutcomeCell("invalid_bool", True, 2, None, 0, 0),
    OutcomeCell("verify_omitted", [], 0, None, 0),
    OutcomeCell("under_zero_exit3", [], 3, False, 0, 0),
)

DRY_CELLS: tuple[OutcomeCell, ...] = (
    OutcomeCell("dry_exact", [], 0, None, 0),
    OutcomeCell("dry_over_1", [], 0, None, 1),
    OutcomeCell("dry_over_2", [], 0, None, 2),
    OutcomeCell("dry_over_4", [], 0, None, 4),
    OutcomeCell("dry_under_1", [], 0, None, -1),
    OutcomeCell("dry_under_2", [], 0, None, -2),
    OutcomeCell("dry_zero", [], 0, None, 0, 0),
    OutcomeCell("dry_io", [], 1, None, 0, 0),
    OutcomeCell("dry_usage", [], 2, None, 0, 0),
    OutcomeCell("dry_invalid_dim", _inv("rowCountMismatch", expected=8, actual=5), 2, None, 0, 0),
    OutcomeCell("dry_invalid_field", _inv("notFound", name="문서번호"), 2, None, 0, 0),
    OutcomeCell("dry_invalid_covered", _inv("coveredCellNotEmpty", row=3, col=2), 2, None, 0, 0),
    OutcomeCell("dry_invalid_control", _inv("controlCharacter", row=0, col=4), 2, None, 0, 0),
    OutcomeCell("dry_invalid_bool", True, 2, None, 0, 0),
    OutcomeCell("dry_page", [], 4, None, 0, 0),
    OutcomeCell("dry_over_7", [], 0, None, 7),
)

IR_CELLS: tuple[OutcomeCell, ...] = (
    OutcomeCell("ir_identical", [], 0, True, 0, 0),
    OutcomeCell("ir_one", [], 3, False, 0, 1),
    OutcomeCell("ir_two", [], 3, False, 0, 2),
    OutcomeCell("ir_three", [], 3, False, 0, 3),
    OutcomeCell("ir_five", [], 3, False, 0, 5),
    OutcomeCell("ir_eight", [], 3, False, 0, 8),
    OutcomeCell("ir_twelve", [], 3, False, 0, 12),
    OutcomeCell("ir_exit0_false", [], 0, False, 0, 2),
    OutcomeCell("ir_exit3_true", [], 3, True, 0, 0),
    OutcomeCell("ir_io", [], 1, None, 0, 0),
    OutcomeCell("ir_usage", [], 2, None, 0, 0),
    OutcomeCell("ir_page", [], 4, False, 0, 4),
    OutcomeCell("ir_invalid", _inv("leftUnreadable"), 1, None, 0, 0),
    OutcomeCell("ir_omitted", [], 0, None, 0, 0),
)

COMMAND_MODES: tuple[tuple[CommandFamily, Mode], ...] = (
    (CommandFamily.FILL_FIELDS, Mode.VERIFY),
    (CommandFamily.FILL_FIELDS, Mode.DRY_RUN),
    (CommandFamily.CSV_TO_TABLE, Mode.VERIFY),
    (CommandFamily.CSV_TO_TABLE, Mode.DRY_RUN),
    (CommandFamily.IR_DIFF, Mode.IR_DIFF),
    (CommandFamily.CONVERT, Mode.VERIFY),
    (CommandFamily.REPLACE_TEXT, Mode.VERIFY),
    (CommandFamily.REPLACE_TEXT, Mode.DRY_RUN),
    (CommandFamily.REDACT, Mode.DRY_RUN),
    (CommandFamily.REDACT, Mode.VERIFY),
    (CommandFamily.SET_CELL, Mode.VERIFY),
    (CommandFamily.SET_CELL, Mode.DRY_RUN),
    (CommandFamily.CSV_TO_CHART, Mode.DRY_RUN),
    (CommandFamily.CSV_TO_CHART, Mode.VERIFY),
    (CommandFamily.SANITIZE, Mode.DRY_RUN),
    (CommandFamily.SANITIZE, Mode.VERIFY),
    (CommandFamily.RUN, Mode.VERIFY),
)

N_VALUES = (2, 3, 4, 5, 6, 7, 8)
INTENDED_VALUES = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 16, 20)


def cells_for(mode: Mode) -> tuple[OutcomeCell, ...]:
    if mode is Mode.DRY_RUN:
        return DRY_CELLS
    if mode is Mode.IR_DIFF:
        return IR_CELLS
    return VERIFY_CELLS


def pick_cells(mode: Mode, n: int, salt: int) -> list[OutcomeCell]:
    pool = cells_for(mode)
    start = salt % len(pool)
    picked: list[OutcomeCell] = []
    for i in range(n):
        picked.append(pool[(start + i * (1 + salt % 3)) % len(pool)])
    # Guarantee distinct cell names in a set; walk forward on collision.
    names = set()
    unique: list[OutcomeCell] = []
    cursor = start
    guard = 0
    for cell in picked:
        current = cell
        while current.name in names:
            cursor = (cursor + 1) % len(pool)
            current = pool[cursor]
            guard += 1
            if guard > len(pool) * 3:
                raise RuntimeError("cannot pick distinct outcome cells")
        names.add(current.name)
        unique.append(current)
    return unique


def sample_identity(index: int) -> dict[str, Any]:
    agency = AGENCIES[index % len(AGENCIES)]
    kind = KINDS[(index // len(AGENCIES)) % len(KINDS)]
    year = YEARS[(index // (len(AGENCIES) * len(KINDS))) % len(YEARS)]
    serial = 1 + (index % 997)
    ext = "hwpx" if index % 3 else "hwp"
    rel = f"samples/gov/{agency}/{kind}/{year}/{serial:04d}.{ext}"
    return {
        "agency": agency,
        "docKind": kind,
        "year": year,
        "serial": serial,
        "sample": rel,
        "sourceFormat": "hwpx" if ext == "hwpx" else "hwp5",
        "output": f"out/v-bon/{agency}/{kind}/{year}/{serial:04d}.out.{ext}",
    }


def field_plan(intended: int, salt: int) -> list[str]:
    n = max(1, intended)
    return [FIELD_NAMES[(salt + i) % len(FIELD_NAMES)] for i in range(n)]


def filled_items(plan: list[str], count: int, salt: int) -> list[dict[str, Any]]:
    items = []
    listed = min(count, 4)
    for i in range(listed):
        name = plan[i % len(plan)] if plan else FIELD_NAMES[i % len(FIELD_NAMES)]
        items.append(
            {
                "name": name,
                "occurrence": i // max(1, len(plan)),
                "value": f"{name}-{salt:04d}-{i:02d}",
            }
        )
    return items


def changed_cells(count: int, salt: int, rows: int, cols: int) -> list[dict[str, Any]]:
    items = []
    for i in range(count):
        row = 1 + ((salt + i) % max(1, rows - 1))
        col = (salt + i * 3) % cols
        items.append(
            {
                "row": row,
                "col": col,
                "oldText": f"old-{salt}-{row}-{col}",
                "newText": f"new-{salt}-{row}-{col}-{i}",
            }
        )
        if len(items) >= 4:
            break
    return items


def ir_categories(diff_count: int, salt: int) -> dict[str, int]:
    if diff_count <= 0:
        return {}
    cats: dict[str, int] = {}
    remaining = diff_count
    idx = 0
    while remaining > 0:
        name = IR_CATEGORIES[(salt + idx) % len(IR_CATEGORIES)]
        take = 1 + ((salt + idx) % 3)
        take = min(take, remaining)
        cats[name] = cats.get(name, 0) + take
        remaining -= take
        idx += 1
    return cats


def build_envelope(
    command: CommandFamily,
    mode: Mode,
    ident: dict[str, Any],
    cell: OutcomeCell,
    intended: int,
    salt: int,
) -> dict[str, Any]:
    changed = cell.changed_count(intended)
    dry = mode is Mode.DRY_RUN
    verify: dict[str, Any] | None
    if cell.identical is None:
        verify = None
    else:
        verify = {
            "identical": cell.identical,
            "diffCount": 0 if cell.identical else max(1, abs(changed - intended) + (salt % 4)),
        }
    plan = field_plan(max(intended, 1), salt)
    rows = 6 + (salt % 17)
    cols = 3 + (salt % 6)
    env: dict[str, Any] = {
        "schemaVersion": "1.0",
        "source": ident["sample"],
        "dryRun": dry,
        "changedCount": changed,
        "invalid": cell.invalid,
        "exitClass": cell.exit_class,
    }
    if command is CommandFamily.FILL_FIELDS:
        env.update(
            {
                "filledCount": changed,
                "notFound": [] if not cell.invalid else [plan[0]],
                "ambiguous": [],
                "filled": filled_items(plan, changed, salt),
                "changedPages": None if dry else [0, salt % 3],
            }
        )
    elif command is CommandFamily.CSV_TO_TABLE:
        env.update(
            {
                "csv": ident["sample"].replace(".hwp", ".csv").replace(".hwpx", ".csv"),
                "table": salt % 5,
                "rowCount": rows,
                "colCount": cols,
                "changed": changed_cells(min(changed, 8), salt, rows, cols),
                "changedPages": None if dry else [0],
            }
        )
    elif command is CommandFamily.IR_DIFF:
        env.pop("dryRun", None)
        env.update(
            {
                "left": ident["sample"],
                "right": ident["output"],
                "identical": bool(cell.identical) if cell.identical is not None else False,
                "diffCount": changed,
                "categories": ir_categories(changed, salt),
            }
        )
        if cell.identical is None:
            env["identical"] = None
    elif command is CommandFamily.CONVERT:
        env.update(
            {
                "outputFormat": "hwp5",
                "wasDistribution": bool(salt % 2),
                "bytes": 40_000 + salt * 17 + changed * 64,
            }
        )
    elif command is CommandFamily.REPLACE_TEXT:
        env.update(
            {
                "replacedCount": changed,
                "find": CELL_LABELS[salt % len(CELL_LABELS)],
                "replace": f"치환-{salt}",
                "occurrence": None if changed != 1 else 1 + (salt % 4),
            }
        )
    elif command is CommandFamily.REDACT:
        env.update(
            {
                "findingCount": max(changed, 1 if cell.exit_class == 0 else 0),
                "redactedCount": 0 if dry else changed,
                "kinds": ["rrn", "phone", "email"][ : 1 + (salt % 3)],
            }
        )
    elif command is CommandFamily.SET_CELL:
        env.update(
            {
                "table": salt % 4,
                "row": 1 + (salt % rows),
                "col": salt % cols,
                "oldText": f"old-{salt}",
                "newText": f"new-{salt}-{changed}",
                "overflow": False,
            }
        )
    elif command is CommandFamily.CSV_TO_CHART:
        env.update(
            {
                "chart": 1 + (salt % 6),
                "csv": ident["sample"].rsplit(".", 1)[0] + ".chart.csv",
                "changed": [
                    {"series": 1 + (i % 3), "point": i, "from": float(i), "to": float(i + salt % 5)}
                    for i in range(min(changed, 6))
                ],
            }
        )
    elif command is CommandFamily.SANITIZE:
        env.update(
            {
                "removedCount": changed,
                "removed": [f"meta:{k}" for k in ("author", "company", "date")[: max(0, min(3, changed))]],
            }
        )
    elif command is CommandFamily.RUN:
        env.update(
            {
                "planVersion": "1.0",
                "assertions": {"verify": mode is Mode.VERIFY, "notFoundEmpty": True},
                "steps": [
                    {"action": "fill_fields", "filledCount": min(changed, intended), "invalid": []}
                ],
            }
        )
    if not dry and command is not CommandFamily.IR_DIFF:
        env["output"] = ident["output"]
        env["outputFormat"] = ident["sourceFormat"]
    if command is CommandFamily.IR_DIFF:
        if cell.identical is not None:
            env["verify"] = {"identical": cell.identical, "diffCount": changed}
        else:
            env["verify"] = None
    else:
        env["verify"] = verify
    return env


def argv_for(command: CommandFamily, mode: Mode) -> list[str]:
    flags = list(mode.argv_flag())
    return ["rhwp", *command.argv_head, *flags]


def make_set(
    serial: int,
    command: CommandFamily,
    mode: Mode,
    n: int,
    intended: int,
    ident_index: int,
) -> dict[str, Any]:
    ident = sample_identity(ident_index)
    cells = pick_cells(mode, n, serial * 17 + ident_index)
    raw_candidates: list[dict[str, Any]] = []
    outcomes = []
    for idx, cell in enumerate(cells):
        cid = f"c{idx}"
        env = build_envelope(command, mode, ident, cell, intended, serial + idx * 13)
        rec = {
            "candidateId": cid,
            "archetype": cell.name,
            "changedCount": cell.changed_count(intended),
            "invalid": cell.invalid,
            "verify": env.get("verify"),
            "exitClass": cell.exit_class,
            "envelope": env,
        }
        raw_candidates.append(rec)
        outcomes.append(lift_candidate_record(rec))
    ranked = rank_candidates(
        outcomes,
        intended_changed_count=intended,
        set_id=f"v-bon-{serial:06d}",
        command=command.value,
        mode=mode.value,
    )
    by_id = {row.candidate.candidate_id: row.expected_rank for row in ranked.ranking}
    for rec in raw_candidates:
        rec["expectedRank"] = by_id[rec["candidateId"]]
    return {
        "schemaVersion": SCHEMA_VERSION,
        "claim": CLAIM_ID,
        "kind": "bestOfNSet",
        "setId": f"v-bon-{serial:06d}",
        "command": command.value,
        "mode": mode.value,
        "argv": argv_for(command, mode),
        "n": n,
        "intendedChangedCount": intended,
        "sample": ident["sample"],
        "sourceFormat": ident["sourceFormat"],
        "agency": ident["agency"],
        "docKind": ident["docKind"],
        "year": ident["year"],
        "serial": ident["serial"],
        "rankFields": ["changedCount", "invalid", "verify.identical", "exitClass"],
        "winnerId": ranked.winner_id,
        "candidates": raw_candidates,
    }


def identity_key(blob: dict[str, Any]) -> tuple[Any, ...]:
    cands = tuple(
        (
            rec["candidateId"],
            rec["archetype"],
            rec["changedCount"],
            invalid_fingerprint(rec["invalid"]),
            None
            if rec["verify"] is None
            else rec["verify"].get("identical")
            if isinstance(rec["verify"], dict)
            else rec["verify"],
            rec["exitClass"],
            rec["expectedRank"],
        )
        for rec in blob["candidates"]
    )
    return (
        blob["command"],
        blob["mode"],
        blob["sample"],
        blob["intendedChangedCount"],
        blob["n"],
        cands,
    )


def axis_iter():
    serial = 0
    ident_index = 0
    while True:
        for command, mode in COMMAND_MODES:
            for n in N_VALUES:
                for intended in INTENDED_VALUES:
                    serial += 1
                    ident_index += 1
                    yield serial, command, mode, n, intended, ident_index


def dump_json(path: Path, payload: Any) -> int:
    text = json.dumps(payload, ensure_ascii=False, indent=2) + "\n"
    path.write_text(text, encoding="utf-8", newline="\n")
    return text.count("\n")


def generate(
    min_lines: int,
    sets_per_shard: int,
    out_dir: Path,
    min_sets: int = DEFAULT_MIN_SETS,
) -> dict[str, Any]:
    out_dir.mkdir(parents=True, exist_ok=True)
    for stale in out_dir.glob("shard_*.json"):
        stale.unlink()
    sets: list[dict[str, Any]] = []
    keys: set[tuple[Any, ...]] = set()
    # Generate in memory per shard to keep peak memory bounded.
    shards: list[dict[str, Any]] = []
    shard_buf: list[dict[str, Any]] = []
    total_lines = 0
    for serial, command, mode, n, intended, ident_index in axis_iter():
        blob = make_set(serial, command, mode, n, intended, ident_index)
        key = identity_key(blob)
        if key in keys:
            raise RuntimeError(f"duplicate identity at {blob['setId']}")
        keys.add(key)
        sets.append(blob)
        shard_buf.append(blob)
        if len(shard_buf) >= sets_per_shard:
            name = f"shard_{len(shards):04d}.json"
            written = dump_json(out_dir / name, shard_buf)
            total_lines += written
            shards.append(
                {
                    "path": f"corpus/{name}",
                    "sets": len(shard_buf),
                    "lines": written,
                    "first": shard_buf[0]["setId"],
                    "last": shard_buf[-1]["setId"],
                }
            )
            shard_buf = []
            if total_lines >= min_lines and len(sets) >= min_sets:
                break
        if serial > 30000:
            raise RuntimeError("failed to reach min_lines/min_sets before safety cap")
    if shard_buf and (total_lines < min_lines or len(sets) < min_sets):
        name = f"shard_{len(shards):04d}.json"
        written = dump_json(out_dir / name, shard_buf)
        total_lines += written
        shards.append(
            {
                "path": f"corpus/{name}",
                "sets": len(shard_buf),
                "lines": written,
                "first": shard_buf[0]["setId"],
                "last": shard_buf[-1]["setId"],
            }
        )
    elif shard_buf:
        # Flush remainder even after the line gate so the last sets are kept.
        name = f"shard_{len(shards):04d}.json"
        written = dump_json(out_dir / name, shard_buf)
        total_lines += written
        shards.append(
            {
                "path": f"corpus/{name}",
                "sets": len(shard_buf),
                "lines": written,
                "first": shard_buf[0]["setId"],
                "last": shard_buf[-1]["setId"],
            }
        )

    by_command: dict[str, int] = {}
    by_mode: dict[str, int] = {}
    for blob in sets:
        by_command[blob["command"]] = by_command.get(blob["command"], 0) + 1
        by_mode[blob["mode"]] = by_mode.get(blob["mode"], 0) + 1

    manifest = {
        "schemaVersion": SCHEMA_VERSION,
        "claim": CLAIM_ID,
        "kind": "bestOfNCorpus",
        "setCount": len(sets),
        "shardCount": len(shards),
        "lineCount": total_lines,
        "minLines": min_lines,
        "minSets": min_sets,
        "setsPerShard": sets_per_shard,
        "rankFields": ["changedCount", "invalid", "verify.identical", "exitClass"],
        "forbidden": ["process_steps", "processSteps", "proseScore", "llmScore"],
        "byCommand": dict(sorted(by_command.items())),
        "byMode": dict(sorted(by_mode.items())),
        "shards": shards,
        "notes": [
            "Each record is a distinct N-candidate outcome set, not comment padding.",
            "expectedRank is rank.py of changedCount/invalid/verify.identical/exitClass.",
            "V-step process_steps is out of scope (#5490).",
        ],
    }
    dump_json(out_dir / "manifest.json", manifest)
    if total_lines < min_lines:
        raise RuntimeError(f"lineCount {total_lines} < min_lines {min_lines}")
    # `sets` after the break may omit a flushed remainder already counted; recount.
    recounted = 0
    for shard in shards:
        recounted += shard["sets"]
    manifest["setCount"] = recounted
    dump_json(out_dir / "manifest.json", manifest)
    return manifest


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--min-lines", type=int, default=DEFAULT_MIN_LINES)
    parser.add_argument("--min-sets", type=int, default=DEFAULT_MIN_SETS)
    parser.add_argument("--sets-per-shard", type=int, default=SETS_PER_SHARD)
    parser.add_argument("--out-dir", type=Path, default=CORPUS)
    args = parser.parse_args(argv)
    manifest = generate(
        args.min_lines, args.sets_per_shard, args.out_dir, min_sets=args.min_sets
    )
    json.dump(
        {
            "setCount": manifest["setCount"],
            "lineCount": manifest["lineCount"],
            "shards": manifest["shardCount"],
        },
        sys.stdout,
        ensure_ascii=False,
    )
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
