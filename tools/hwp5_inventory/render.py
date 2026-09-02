"""CLI 리포트 모양의 Markdown/JSONL 전사."""

from __future__ import annotations

from collections import Counter, defaultdict
from typing import Iterable, Sequence

from .catalog import PROBE_VARIANTS
from .model import (
    AlignmentStats,
    DiffItem,
    InventoryItem,
    ProbeAxis,
    inventory_side,
    is_ctrl_header_candidate,
    is_picture_shape_candidate,
    is_table_candidate,
    matches_focus,
    role_control,
    table_field_rows,
    table_probe_axes,
)


def escape_md(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ")


def short_hash(value: str) -> str:
    digest = value.removeprefix("blake3:")
    return digest[:16]


def render_side(record: dict | None) -> str:
    if not record:
        return "`-`"
    ctrl = record.get("control_name") or record.get("control_id") or "-"
    return (
        f"`{record['tag_name']}#{record.get('tuple_index', 0)}/"
        f"{ctrl}/{record['size']}/{short_hash(record['payload_hash'])}`"
    )


def render_inventory_markdown(items: Sequence[InventoryItem], *, source: str, sample: str) -> str:
    streams = sorted({item.stream_path for item in items})
    lines = [
        "# HWP5 Inventory",
        "",
        f"- source: `{source}`",
        f"- sample: `{sample}`",
        "- compressed: `true`",
        "- encrypted: `false`",
        "- distribution: `false`",
        f"- section_count: `{sum(1 for stream in streams if stream.startswith('/BodyText/Section')) or 1}`",
        "",
        "## Streams",
        "",
    ]
    for stream in streams:
        lines.append(f"- `{stream}`")
    lines.extend(
        [
            "",
            "## Records",
            "",
            "| stream | section | idx | uid | level | tag | role | tuple | ctrl | size | owner | parent | scope | key payload | hash |",
            "|---|---:|---:|---|---:|---|---|---:|---|---:|---|---|---|---|---|",
        ]
    )
    for item in items:
        section = "-" if item.section is None else str(item.section)
        parent = item.parent_scope or "-"
        ctrl = "-"
        if item.control_id and item.control_name:
            ctrl = f"{item.control_id}/{item.control_name}"
        elif item.control_id:
            ctrl = item.control_id
        lines.append(
            "| `{stream}` | {section} | {idx} | `{uid}` | {level} | `{tag}` | `{role}` | {tuple_i} | `{ctrl}` | {size} | `{owner}` | `{parent}` | `{scope}` | `{key}` | `{hash}` |".format(
                stream=escape_md(item.stream_path),
                section=section,
                idx=item.record_index,
                uid=escape_md(item.record_uid),
                level=item.level,
                tag=item.tag_name,
                role=item.tuple_role,
                tuple_i=item.tuple_index,
                ctrl=escape_md(ctrl),
                size=item.size,
                owner=escape_md(item.owner),
                parent=escape_md(parent),
                scope=escape_md(item.scope_path),
                key=escape_md(item.key_payload),
                hash=short_hash(item.payload_hash),
            )
        )
    lines.append("")
    return "\n".join(lines)


def _summary_counts(items: Sequence[DiffItem]) -> list[tuple[str, int]]:
    counts: Counter[str] = Counter(item.diff_kind for item in items)
    return sorted(counts.items())


def _tuple_anchor_summary(items: Sequence[DiffItem]) -> list[tuple[tuple[str, str], dict[str, int]]]:
    summary: dict[tuple[str, str], dict[str, int]] = {}
    for item in items:
        role, control = role_control(item)
        bucket = summary.setdefault((role, control), {"missing": 0, "extra": 0, "changed": 0})
        if item.alignment_status == "missing":
            bucket["missing"] += 1
        elif item.alignment_status == "extra":
            bucket["extra"] += 1
        else:
            bucket["changed"] += 1
    rows = sorted(summary.items(), key=lambda pair: (-sum(pair[1].values()), pair[0]))
    return rows


def render_diff_markdown(
    items: Sequence[DiffItem],
    stats: AlignmentStats,
    *,
    oracle_path: str,
    generated_path: str,
    align_mode: str,
) -> str:
    lines = [
        "# HWP5 Inventory Diff",
        "",
        f"- oracle: `{oracle_path}`",
        f"- generated: `{generated_path}`",
        f"- align_mode: `{align_mode}`",
        f"- diff_count: `{len(items)}`",
        "",
        "## Alignment Summary",
        "",
        "| status | count |",
        "|---|---:|",
        f"| `matched` | {stats.matched} |",
        f"| `changed` | {stats.changed} |",
        f"| `missing` | {stats.missing} |",
        f"| `extra` | {stats.extra} |",
        "",
        "## Summary",
        "",
        "| kind | count |",
        "|---|---:|",
    ]
    for kind, count in _summary_counts(items):
        lines.append(f"| `{kind}` | {count} |")
    if not items:
        lines.append("| - | 0 |")
    lines.extend(
        [
            "",
            "## Tuple Anchor Summary",
            "",
            "| role | control | missing | extra | changed | total |",
            "|---|---|---:|---:|---:|---:|",
        ]
    )
    anchors = _tuple_anchor_summary(items)
    if not anchors:
        lines.append("| - | - | 0 | 0 | 0 | 0 |")
    for (role, control), counts in anchors:
        total = counts["missing"] + counts["extra"] + counts["changed"]
        lines.append(
            f"| `{escape_md(role)}` | `{escape_md(control)}` | {counts['missing']} | {counts['extra']} | {counts['changed']} | {total} |"
        )
    lines.extend(
        [
            "",
            "## Diff Rows",
            "",
            "| status | kind | key | fields | oracle | generated | note |",
            "|---|---|---|---|---|---|---|",
        ]
    )
    if not items:
        lines.append("| - | - | - | - | - | - | no diff |")
    for item in items:
        lines.append(
            "| `{status}` | `{kind}` | `{key}` | `{fields}` | {oracle} | {generated} | {note} |".format(
                status=escape_md(item.alignment_status),
                kind=escape_md(item.diff_kind),
                key=escape_md(item.key),
                fields=escape_md(",".join(item.changed_fields)),
                oracle=render_side(item.oracle),
                generated=render_side(item.generated),
                note=escape_md(item.note or "-"),
            )
        )
    lines.append("")
    return "\n".join(lines)


def render_hints_markdown(
    items: Sequence[DiffItem],
    stats: AlignmentStats,
    *,
    oracle_path: str,
    generated_path: str,
    align_mode: str,
) -> str:
    lines = [
        "# HWP5 Contract Violation Hints",
        "",
        "## Input",
        "",
        f"- oracle: `{oracle_path}`",
        f"- generated: `{generated_path}`",
        f"- align_mode: `{align_mode}`",
        "- report_mode: `hints`",
        f"- diff_count: `{len(items)}`",
        "",
        "## Alignment Summary",
        "",
        "| status | count |",
        "|---|---:|",
        f"| `matched` | {stats.matched} |",
        f"| `changed` | {stats.changed} |",
        f"| `missing` | {stats.missing} |",
        f"| `extra` | {stats.extra} |",
        "",
        "## Top Role/Control Buckets",
        "",
        "| role | control | missing | extra | changed | total |",
        "|---|---|---:|---:|---:|---:|",
    ]
    for (role, control), counts in _tuple_anchor_summary(items)[:50]:
        total = sum(counts.values())
        lines.append(
            f"| `{escape_md(role)}` | `{escape_md(control)}` | {counts['missing']} | {counts['extra']} | {counts['changed']} | {total} |"
        )
    if not items:
        lines.append("| - | - | 0 | 0 | 0 | 0 |")
    lines.extend(["", "## Missing Records", "", "| key | stream | oracle | note |", "|---|---|---|---|"])
    missing = [item for item in items if item.alignment_status == "missing"]
    if not missing:
        lines.append("| - | - | - | missing record 없음 |")
    for item in missing[:80]:
        lines.append(
            f"| `{escape_md(item.key)}` | `{escape_md(item.stream_path)}` | {render_side(item.oracle)} | {escape_md(item.note or '-')} |"
        )
    def _candidate(title: str, predicate) -> None:
        lines.extend(
            [
                "",
                f"## {title}",
                "",
                "| status | fields | key | oracle | generated | note |",
                "|---|---|---|---|---|---|",
            ]
        )
        rows = [item for item in items if predicate(item)]
        if not rows:
            lines.append("| - | - | - | - | - | 후보 없음 |")
            return
        for item in rows[:80]:
            lines.append(
                "| `{status}` | `{fields}` | `{key}` | {oracle} | {generated} | {note} |".format(
                    status=escape_md(item.alignment_status),
                    fields=escape_md(",".join(item.changed_fields)),
                    key=escape_md(item.key),
                    oracle=render_side(item.oracle),
                    generated=render_side(item.generated),
                    note=escape_md(item.note or "-"),
                )
            )

    _candidate("Table Candidates", is_table_candidate)
    _candidate("Picture/Shape Candidates", is_picture_shape_candidate)
    _candidate("CtrlHeader Candidates", is_ctrl_header_candidate)
    lines.extend(
        [
            "",
            "## Next Probe Suggestions",
            "",
        ]
    )
    if stats.missing:
        lines.append("- missing record는 oracle의 record 단위 graft 후보로 먼저 분리한다.")
    table_count = sum(1 for item in items if is_table_candidate(item))
    picture_count = sum(1 for item in items if is_picture_shape_candidate(item))
    ctrl_count = sum(1 for item in items if is_ctrl_header_candidate(item))
    docinfo_count = sum(1 for item in items if role_control(item)[0] == "docinfo")
    if table_count:
        lines.append(
            f"- TABLE 후보 `{table_count}`건은 CTRL_HEADER(Table) + TABLE payload + child header의 tuple contract로 묶어 검증한다."
        )
    if picture_count:
        lines.append(
            f"- 그림/도형 후보 `{picture_count}`건은 DocInfo BinData reference와 PIC/SHAPE_COMPONENT payload를 함께 본다."
        )
    if ctrl_count:
        lines.append(
            f"- CTRL_HEADER 후보 `{ctrl_count}`건은 다음 record tag와 묶어 control별 필수 payload를 비교한다."
        )
    if docinfo_count:
        lines.append(
            f"- DocInfo 후보 `{docinfo_count}`건은 body record보다 먼저 count/reference table 계약 위반 가능성을 확인한다."
        )
    if not (stats.missing or table_count or picture_count or ctrl_count or docinfo_count):
        lines.append("- 후보가 작다. index alignment report로 scope_path 수준 차이를 재확인한다.")
    lines.append("")
    return "\n".join(lines)


def render_bundles_markdown(
    items: Sequence[DiffItem],
    oracle_items: Sequence[InventoryItem],
    generated_items: Sequence[InventoryItem],
    stats: AlignmentStats,
    *,
    oracle_path: str,
    generated_path: str,
    align_mode: str,
    focus: str,
    window: int = 2,
) -> str:
    candidates = [item for item in items if matches_focus(item, focus)]
    lines = [
        "# HWP5 Candidate Bundles",
        "",
        "## Input",
        "",
        f"- oracle: `{oracle_path}`",
        f"- generated: `{generated_path}`",
        f"- align_mode: `{align_mode}`",
        "- report_mode: `bundles`",
        f"- focus: `{focus}`",
        f"- window: `{window}`",
        f"- candidate_count: `{len(candidates)}`",
        "",
        "## Alignment Summary",
        "",
        "| status | count |",
        "|---|---:|",
        f"| `matched` | {stats.matched} |",
        f"| `changed` | {stats.changed} |",
        f"| `missing` | {stats.missing} |",
        f"| `extra` | {stats.extra} |",
        "",
        "## Candidate Summary",
        "",
        "| role | control | count |",
        "|---|---|---:|",
    ]
    counts: dict[tuple[str, str], int] = defaultdict(int)
    for item in candidates:
        counts[role_control(item)] += 1
    if not counts:
        lines.append("| - | - | 0 |")
    for (role, control), count in sorted(counts.items(), key=lambda pair: (-pair[1], pair[0])):
        lines.append(f"| `{escape_md(role)}` | `{escape_md(control)}` | {count} |")
    lines.extend(["", "## Bundles", ""])
    for ordinal, item in enumerate(candidates[:80], start=1):
        lines.extend(
            [
                f"### Bundle {ordinal}: `{escape_md(item.key)}`",
                "",
                f"- status: `{item.alignment_status}`",
                f"- fields: `{','.join(item.changed_fields)}`",
                f"- stream: `{item.stream_path}`",
                f"- oracle: {render_side(item.oracle)}",
                f"- generated: {render_side(item.generated)}",
            ]
        )
        if item.note:
            lines.append(f"- note: `{escape_md(item.note)}`")
        lines.append("")
        for title, side_items, center in (
            ("Oracle Window", oracle_items, item.oracle_record_index),
            ("Generated Window", generated_items, item.generated_record_index),
        ):
            lines.extend([f"#### {title}", ""])
            if center is None:
                lines.extend(["해당 side record 없음", ""])
                continue
            start = max(0, center - window)
            end = center + window
            rows = [
                row
                for row in side_items
                if row.stream_path == item.stream_path and start <= row.record_index <= end
            ]
            lines.extend(
                [
                    "| focus | idx | level | tag | role | tuple | ctrl | size | key payload | head | hash | scope |",
                    "|---|---:|---:|---|---|---:|---|---:|---|---|---|---|",
                ]
            )
            for row in rows:
                focus_mark = ">" if row.record_index == center else ""
                ctrl = "-"
                if row.control_id and row.control_name:
                    ctrl = f"{row.control_id}/{row.control_name}"
                lines.append(
                    "| `{focus}` | {idx} | {level} | `{tag}` | `{role}` | {tuple_i} | `{ctrl}` | {size} | `{key}` | `{head}` | `{hash}` | `{scope}` |".format(
                        focus=focus_mark,
                        idx=row.record_index,
                        level=row.level,
                        tag=escape_md(row.tag_name),
                        role=escape_md(row.tuple_role),
                        tuple_i=row.tuple_index,
                        ctrl=escape_md(ctrl),
                        size=row.size,
                        key=escape_md(row.key_payload),
                        head=escape_md(row.payload_head_hex),
                        hash=short_hash(row.payload_hash),
                        scope=escape_md(row.scope_path),
                    )
                )
            lines.append("")
    lines.append("")
    return "\n".join(lines)


def render_table_fields_markdown(
    items: Sequence[DiffItem],
    oracle_items: Sequence[InventoryItem],
    generated_items: Sequence[InventoryItem],
    *,
    oracle_path: str,
    generated_path: str,
    align_mode: str,
) -> str:
    candidates = [item for item in items if is_table_candidate(item)]
    lines = [
        "# HWP5 Table Field Diff",
        "",
        "## Input",
        "",
        f"- oracle: `{oracle_path}`",
        f"- generated: `{generated_path}`",
        f"- align_mode: `{align_mode}`",
        "- report_mode: `table-fields`",
        f"- candidate_count: `{len(candidates)}`",
        "",
        "## Summary",
        "",
        "### Candidate Tags",
        "",
        "| tag | count |",
        "|---|---:|",
    ]
    tag_counts: Counter[str] = Counter()
    field_counts: Counter[str] = Counter()
    for item in candidates:
        oracle = inventory_side(oracle_items, item.stream_path, item.oracle_record_index)
        generated = inventory_side(generated_items, item.stream_path, item.generated_record_index)
        tag = (oracle or generated).tag_name if (oracle or generated) else "-"
        tag_counts[tag] += 1
        for row in table_field_rows(oracle, generated):
            if row.status == "diff":
                field_counts[row.field_name] += 1
    if not tag_counts:
        lines.append("| - | 0 |")
    for tag, count in tag_counts.items():
        lines.append(f"| `{escape_md(tag)}` | {count} |")
    lines.extend(["", "### Diff Fields", "", "| field | diff count |", "|---|---:|"])
    if not field_counts:
        lines.append("| - | 0 |")
    for name, count in field_counts.most_common():
        lines.append(f"| `{escape_md(name)}` | {count} |")
    lines.extend(
        [
            "",
            "## Table Field Rows",
            "",
            "| key | tag | field | offset | oracle | generated | status |",
            "|---|---|---|---:|---|---|---|",
        ]
    )
    if not candidates:
        lines.append("| - | - | - | - | - | - | no table candidate |")
    for item in candidates:
        oracle = inventory_side(oracle_items, item.stream_path, item.oracle_record_index)
        generated = inventory_side(generated_items, item.stream_path, item.generated_record_index)
        for row in table_field_rows(oracle, generated):
            lines.append(
                f"| `{escape_md(item.key)}` | `{escape_md(row.tag_name)}` | `{escape_md(row.field_name)}` | `{escape_md(row.offset)}` | `{escape_md(row.oracle)}` | `{escape_md(row.generated)}` | `{row.status}` |"
            )
    lines.append("")
    return "\n".join(lines)


def render_table_probe_plan_markdown(
    items: Sequence[DiffItem],
    oracle_items: Sequence[InventoryItem],
    generated_items: Sequence[InventoryItem],
    *,
    oracle_path: str,
    generated_path: str,
    align_mode: str,
) -> str:
    candidates = [item for item in items if is_table_candidate(item)]
    axes = table_probe_axes(candidates, oracle_items, generated_items)
    lines = [
        "# HWP5 Table Probe Plan",
        "",
        "## Input",
        "",
        f"- oracle: `{oracle_path}`",
        f"- generated: `{generated_path}`",
        f"- align_mode: `{align_mode}`",
        "- report_mode: `table-probe-plan`",
        f"- candidate_count: `{len(candidates)}`",
        "",
        "## Probe Axes",
        "",
        "| axis | record kind | affected records | meaning |",
        "|---|---|---:|---|",
    ]
    for axis in axes:
        lines.append(
            f"| `{axis.name}` | `{axis.record_kind}` | {len(axis.rows)} | {axis.description} |"
        )
    lines.extend(
        [
            "",
            "## Recommended Probe Matrix",
            "",
            "| variant | graft axes | purpose |",
            "|---|---|---|",
        ]
    )
    for name, axes_names, purpose in PROBE_VARIANTS:
        lines.append(f"| `{name}` | `{', '.join(axes_names)}` | {purpose} |")
    lines.extend(["", "## Axis Details", ""])
    for axis in axes:
        lines.extend(
            [
                f"### `{axis.name}`",
                "",
                f"- record_kind: `{axis.record_kind}`",
                f"- affected_records: `{len(axis.rows)}`",
                f"- description: {axis.description}",
                "",
                "| key | oracle record | generated record | fields | oracle values | generated values |",
                "|---|---|---|---|---|---|",
            ]
        )
        if not axis.rows:
            lines.append("| - | - | - | - | - | no field on this axis |")
        for row in axis.rows:
            lines.append(
                f"| `{escape_md(row.key)}` | `{escape_md(row.oracle_record)}` | `{escape_md(row.generated_record)}` | `{escape_md(', '.join(row.fields))}` | `{escape_md('; '.join(row.oracle_values))}` | `{escape_md('; '.join(row.generated_values))}` |"
            )
        lines.append("")
    lines.extend(
        [
            "## Notes",
            "",
            "- 이 문서는 판정용 HWP를 직접 생성하지 않는다. 다음 단계에서 바이너리 graft 또는 저장기 projection을 만들 때 사용할 작업 지시서다.",
            "- 필드명은 현재 P0 decoder의 관찰명이다. `tail_after_0x16`, `z_order_or_instance` 등은 contract 이름으로 확정하지 않는다.",
            "- probe 생성 시에는 한 번에 한 축만 바꾸는 파일과 전체 positive guard 파일을 함께 만들어야 한다.",
            "- 페이지 수 로직은 이 계획이 바꾸지 않는다. `rhwp_pages` 칸은 관측 메모일 뿐 serializer 입력이 아니다.",
            "",
        ]
    )
    return "\n".join(lines)


def render_table_probe_generation(
    case_id: str,
    sample: str,
    axes: Sequence[ProbeAxis],
    *,
    oracle_path: str,
    generated_path: str,
) -> str:
    affected = {axis.name: len(axis.rows) for axis in axes}
    lines = [
        f"# table-probe generation — {case_id}",
        "",
        f"- sample: `{sample}`",
        f"- oracle: `{oracle_path}`",
        f"- generated: `{generated_path}`",
        "- command: `rhwp hwp5-table-probe <oracle.hwp> <generated.hwp> --out-dir <dir> --section 0`",
        "- 이 전사는 바이너리 HWP 를 쓰지 않는다. 축별 이식 횟수만 고정한다.",
        "",
        "## Patch counts (fixture)",
        "",
        "| variant | ctrl_outer_margin | ctrl_common_attr | table_attr | table_tail |",
        "|---|---:|---:|---:|---:|",
    ]
    for name, axis_names, _purpose in PROBE_VARIANTS:
        counts = {key: (affected.get(key, 0) if key in axis_names else 0) for key in affected}
        lines.append(
            "| `{name}` | {m} | {c} | {a} | {t} |".format(
                name=name,
                m=counts.get("ctrl_outer_margin", 0) if "ctrl_outer_margin" in axis_names else 0,
                c=counts.get("ctrl_common_attr", 0) if "ctrl_common_attr" in axis_names else 0,
                a=counts.get("table_attr", 0) if "table_attr" in axis_names else 0,
                t=counts.get("table_tail", 0) if "table_tail" in axis_names else 0,
            )
        )
    lines.extend(
        [
            "",
            "## 해석",
            "",
            "- 횟수가 0 인 축은 그 variant 가 no-op 이다. 한컴 판정에 쓰면 안 된다.",
            "- 08_all_table_axes 만 성공하고 01-04 가 실패하면 원인을 분리하지 못한 것이다.",
            "- rhwp-studio 재로드 성공은 한컴 호환의 충분 조건이 아니다.",
            "",
        ]
    )
    return "\n".join(lines)
