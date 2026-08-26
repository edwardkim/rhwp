"""Inventory / diff / table-probe 모델.

Rust `hwp5_inventory` · `hwp5_inventory_diff` · `hwp5_table_probe` 의
행 언어를 Python 으로 재현한다. 바이너리 HWP 를 열지 않고, 픽스처
레코드만 다룬다. 페이지 수는 계산하지 않는다.
"""

from __future__ import annotations

import hashlib
import struct
from collections import defaultdict
from dataclasses import asdict, dataclass, field
from typing import Any, Iterable, Sequence

from .catalog import (
    TABLE_CTRL_FIELDS,
    TABLE_RECORD_FIELDS,
    FieldSpec,
    TAGS,
    tag_by_name,
)


def nfc_hex(data: bytes) -> str:
    return " ".join(f"{byte:02x}" for byte in data)


def payload_hash(data: bytes) -> str:
    digest = hashlib.sha256(data).hexdigest()
    return f"blake3:{digest}"


def record_uid(stream_path: str, record_index: int) -> str:
    stream = stream_path.lstrip("/").replace("/", ".")
    return f"{stream}#{record_index}"


def structural_signature(item: "InventoryItem") -> str:
    return f"tag={item.tag_id}|role={item.tuple_role}|ctrl={item.control_id or '-'}"


def pack_u16(value: int) -> bytes:
    return struct.pack("<H", value & 0xFFFF)


def pack_u32(value: int) -> bytes:
    return struct.pack("<I", value & 0xFFFFFFFF)


def pack_i32(value: int) -> bytes:
    return struct.pack("<i", value)


def read_u16(data: bytes, offset: int) -> int | None:
    if offset + 2 > len(data):
        return None
    return struct.unpack_from("<H", data, offset)[0]


def read_u32(data: bytes, offset: int) -> int | None:
    if offset + 4 > len(data):
        return None
    return struct.unpack_from("<I", data, offset)[0]


def read_i32(data: bytes, offset: int) -> int | None:
    if offset + 4 > len(data):
        return None
    return struct.unpack_from("<i", data, offset)[0]


def ctrl_id_from_fourcc(fourcc: str) -> int:
    raw = fourcc.encode("ascii")
    if len(raw) != 4:
        raise ValueError(fourcc)
    return (raw[0] << 24) | (raw[1] << 16) | (raw[2] << 8) | raw[3]


def fourcc_bytes(fourcc: str) -> bytes:
    return pack_u32(ctrl_id_from_fourcc(fourcc))


@dataclass
class InventoryItem:
    sample: str
    source_path: str
    stream_path: str
    section: int | None
    record_index: int
    record_uid: str
    level: int
    tag_id: int
    tag_name: str
    size: int
    owner: str
    parent_uid: str | None
    parent_scope: str | None
    scope_path: str
    body_order: int | None
    control_id: str | None
    control_name: str | None
    tuple_role: str
    tuple_index: int
    payload_head_hex: str
    key_payload: str
    payload_hash: str
    note: str | None
    payload_hex: str

    @property
    def payload_bytes(self) -> bytes:
        if not self.payload_hex:
            return b""
        return bytes(int(part, 16) for part in self.payload_hex.split())

    def to_public_dict(self) -> dict[str, Any]:
        data = asdict(self)
        data.pop("payload_hex", None)
        return data


@dataclass
class DiffItem:
    align_mode: str
    alignment_status: str
    diff_kind: str
    key: str
    stream_path: str
    section: int | None
    record_index: int
    oracle_record_index: int | None
    generated_record_index: int | None
    oracle_record_uid: str | None
    generated_record_uid: str | None
    signature: str
    changed_fields: list[str]
    oracle: dict[str, Any] | None
    generated: dict[str, Any] | None
    note: str | None

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass
class AlignmentStats:
    matched: int = 0
    changed: int = 0
    missing: int = 0
    extra: int = 0


@dataclass
class TableFieldRow:
    tag_name: str
    field_name: str
    offset: str
    oracle: str
    generated: str
    status: str


@dataclass
class ProbeAxisRow:
    key: str
    oracle_record: str
    generated_record: str
    fields: list[str]
    oracle_values: list[str]
    generated_values: list[str]


@dataclass
class ProbeAxis:
    name: str
    record_kind: str
    description: str
    rows: list[ProbeAxisRow] = field(default_factory=list)


def make_item(
    *,
    sample: str,
    source_path: str,
    stream_path: str,
    record_index: int,
    tag_name: str,
    level: int,
    payload: bytes,
    control_fourcc: str | None = None,
    control_name: str | None = None,
    parent_uid: str | None = None,
    parent_scope: str | None = None,
    body_order: int | None = None,
    tuple_index: int = 0,
    note: str | None = None,
) -> InventoryItem:
    tag = tag_by_name(tag_name)
    section = None
    if stream_path.startswith("/BodyText/Section"):
        section = int(stream_path.rsplit("Section", 1)[1])
    uid = record_uid(stream_path, record_index)
    ctrl_id = None
    ctrl_name = control_name
    if control_fourcc:
        value = ctrl_id_from_fourcc(control_fourcc)
        ctrl_id = f"0x{value:08x}"
        if ctrl_name is None:
            from .catalog import control_by_fourcc

            ctrl_name = control_by_fourcc(control_fourcc).ctrl_name
    head_len = len(payload) if tag_name == "CTRL_HEADER" and len(payload) <= 128 else min(32, len(payload))
    head = nfc_hex(payload[:head_len])
    if tag_name == "CTRL_HEADER" and len(payload) >= 4:
        raw_id = read_u32(payload, 0) or 0
        key = f"ctrl_id=0x{raw_id:08x}({ctrl_name or 'Unknown'}); head{head_len}={head}"
    else:
        key = f"head{head_len}={head}"
    scope_parts = [stream_path.lstrip("/")]
    if parent_scope:
        scope_parts.append(parent_scope.split("@lv")[0])
    scope_parts.append(f"{tag_name}#{record_index}")
    return InventoryItem(
        sample=sample,
        source_path=source_path,
        stream_path=stream_path,
        section=section,
        record_index=record_index,
        record_uid=uid,
        level=level,
        tag_id=tag.tag_id,
        tag_name=tag_name,
        size=len(payload),
        owner=tag.owner,
        parent_uid=parent_uid,
        parent_scope=parent_scope,
        scope_path="/".join(scope_parts),
        body_order=body_order,
        control_id=ctrl_id,
        control_name=ctrl_name,
        tuple_role=tag.role,
        tuple_index=tuple_index,
        payload_head_hex=nfc_hex(payload[: min(32, len(payload))]),
        key_payload=key,
        payload_hash=payload_hash(payload),
        note=note,
        payload_hex=nfc_hex(payload),
    )


def record_summary(item: InventoryItem) -> dict[str, Any]:
    return {
        "record_uid": item.record_uid,
        "tag_id": item.tag_id,
        "tag_name": item.tag_name,
        "size": item.size,
        "tuple_role": item.tuple_role,
        "tuple_index": item.tuple_index,
        "control_id": item.control_id,
        "control_name": item.control_name,
        "scope_path": item.scope_path,
        "payload_hash": item.payload_hash,
    }


def _diff_item(
    align_mode: str,
    diff_kind: str,
    alignment_status: str,
    key: str,
    oracle: InventoryItem | None,
    generated: InventoryItem | None,
    changed_fields: list[str],
    note: str | None,
) -> DiffItem:
    anchor = oracle or generated
    assert anchor is not None
    return DiffItem(
        align_mode=align_mode,
        alignment_status=alignment_status,
        diff_kind=diff_kind,
        key=key,
        stream_path=anchor.stream_path,
        section=anchor.section,
        record_index=anchor.record_index,
        oracle_record_index=oracle.record_index if oracle else None,
        generated_record_index=generated.record_index if generated else None,
        oracle_record_uid=oracle.record_uid if oracle else None,
        generated_record_uid=generated.record_uid if generated else None,
        signature=structural_signature(anchor),
        changed_fields=changed_fields,
        oracle=record_summary(oracle) if oracle else None,
        generated=record_summary(generated) if generated else None,
        note=note,
    )


def build_index_diff(
    oracle_items: Sequence[InventoryItem],
    generated_items: Sequence[InventoryItem],
) -> tuple[list[DiffItem], AlignmentStats]:
    oracle_map = {item.record_uid: item for item in oracle_items}
    generated_map = {item.record_uid: item for item in generated_items}
    keys = sorted(set(oracle_map) | set(generated_map))
    items: list[DiffItem] = []
    stats = AlignmentStats()
    for key in keys:
        oracle = oracle_map.get(key)
        generated = generated_map.get(key)
        if oracle and generated:
            before = len(items)
            if oracle.tag_id != generated.tag_id:
                items.append(
                    _diff_item(
                        "index",
                        "tag_changed",
                        "changed",
                        key,
                        oracle,
                        generated,
                        ["tag"],
                        f"{oracle.tag_name} -> {generated.tag_name}",
                    )
                )
            if oracle.size != generated.size:
                items.append(
                    _diff_item(
                        "index",
                        "size_changed",
                        "changed",
                        key,
                        oracle,
                        generated,
                        ["size"],
                        f"{oracle.size} -> {generated.size}",
                    )
                )
            if oracle.payload_hash != generated.payload_hash:
                items.append(
                    _diff_item(
                        "index",
                        "payload_changed",
                        "changed",
                        key,
                        oracle,
                        generated,
                        ["payload_hash"],
                        None,
                    )
                )
            if oracle.scope_path != generated.scope_path:
                items.append(
                    _diff_item(
                        "index",
                        "scope_changed",
                        "changed",
                        key,
                        oracle,
                        generated,
                        ["scope_path"],
                        None,
                    )
                )
            if (
                oracle.control_id != generated.control_id
                or oracle.control_name != generated.control_name
            ):
                items.append(
                    _diff_item(
                        "index",
                        "control_changed",
                        "changed",
                        key,
                        oracle,
                        generated,
                        ["control"],
                        None,
                    )
                )
            if len(items) == before:
                stats.matched += 1
            else:
                stats.changed += len(items) - before
        elif oracle:
            items.append(
                _diff_item(
                    "index",
                    "missing",
                    "missing",
                    key,
                    oracle,
                    None,
                    [],
                    "oracle에만 존재",
                )
            )
            stats.missing += 1
        else:
            items.append(
                _diff_item(
                    "index",
                    "extra",
                    "extra",
                    key,
                    None,
                    generated,
                    [],
                    "generated에만 존재",
                )
            )
            stats.extra += 1
    return items, stats


def _lcs_ops(oracle: Sequence[InventoryItem], generated: Sequence[InventoryItem]) -> list[tuple[str, int, int | None]]:
    if not oracle:
        return [("extra", j, None) for j in range(len(generated))]
    if not generated:
        return [("missing", i, None) for i in range(len(oracle))]
    oracle_sig = [structural_signature(item) for item in oracle]
    generated_sig = [structural_signature(item) for item in generated]
    rows = len(oracle) + 1
    cols = len(generated) + 1
    dp = [0] * (rows * cols)
    for i in range(len(oracle)):
        for j in range(len(generated)):
            if oracle_sig[i] == generated_sig[j]:
                value = dp[i * cols + j] + 1
            else:
                value = max(dp[i * cols + j + 1], dp[(i + 1) * cols + j])
            dp[(i + 1) * cols + j + 1] = value
    ops: list[tuple[str, int, int | None]] = []
    i = len(oracle)
    j = len(generated)
    while i > 0 or j > 0:
        if i > 0 and j > 0 and oracle_sig[i - 1] == generated_sig[j - 1]:
            ops.append(("pair", i - 1, j - 1))
            i -= 1
            j -= 1
        elif j > 0 and (i == 0 or dp[i * cols + j - 1] >= dp[(i - 1) * cols + j]):
            ops.append(("extra", j - 1, None))
            j -= 1
        else:
            ops.append(("missing", i - 1, None))
            i -= 1
    ops.reverse()
    return ops


def build_lcs_diff(
    oracle_items: Sequence[InventoryItem],
    generated_items: Sequence[InventoryItem],
) -> tuple[list[DiffItem], AlignmentStats]:
    by_oracle: dict[str, list[InventoryItem]] = defaultdict(list)
    by_generated: dict[str, list[InventoryItem]] = defaultdict(list)
    for item in oracle_items:
        by_oracle[item.stream_path].append(item)
    for item in generated_items:
        by_generated[item.stream_path].append(item)
    streams = sorted(set(by_oracle) | set(by_generated))
    items: list[DiffItem] = []
    stats = AlignmentStats()
    for stream in streams:
        oracle_stream = by_oracle.get(stream, [])
        generated_stream = by_generated.get(stream, [])
        for op, left, right in _lcs_ops(oracle_stream, generated_stream):
            if op == "pair":
                oracle = oracle_stream[left]
                generated = generated_stream[right or 0]
                changed: list[str] = []
                if oracle.size != generated.size:
                    changed.append("size")
                if oracle.payload_hash != generated.payload_hash:
                    changed.append("payload_hash")
                if oracle.control_name != generated.control_name:
                    changed.append("control_name")
                if not changed:
                    stats.matched += 1
                else:
                    stats.changed += 1
                    key = (
                        oracle.record_uid
                        if oracle.record_uid == generated.record_uid
                        else f"{oracle.record_uid}~{generated.record_uid}"
                    )
                    items.append(
                        _diff_item(
                            "lcs",
                            "changed",
                            "changed",
                            key,
                            oracle,
                            generated,
                            changed,
                            None,
                        )
                    )
            elif op == "missing":
                oracle = oracle_stream[left]
                stats.missing += 1
                items.append(
                    _diff_item(
                        "lcs",
                        "missing",
                        "missing",
                        oracle.record_uid,
                        oracle,
                        None,
                        [],
                        "LCS alignment에서 oracle에만 존재",
                    )
                )
            else:
                generated = generated_stream[left]
                stats.extra += 1
                items.append(
                    _diff_item(
                        "lcs",
                        "extra",
                        "extra",
                        generated.record_uid,
                        None,
                        generated,
                        [],
                        "LCS alignment에서 generated에만 존재",
                    )
                )
    return items, stats


def role_control(item: DiffItem) -> tuple[str, str]:
    record = item.oracle or item.generated
    if not record:
        return "-", "-"
    role = str(record["tuple_role"])
    control = record.get("control_name") or record.get("control_id") or "-"
    return role, str(control)


def is_table_candidate(item: DiffItem) -> bool:
    role, control = role_control(item)
    return role == "table" or control == "Table"


def is_picture_shape_candidate(item: DiffItem) -> bool:
    role, control = role_control(item)
    return role in {"pic", "shape_component"} or control == "GenShape"


def is_ctrl_header_candidate(item: DiffItem) -> bool:
    return role_control(item)[0] == "ctrl_header"


def matches_focus(item: DiffItem, focus: str) -> bool:
    if focus == "all":
        return True
    if focus == "table":
        return is_table_candidate(item)
    if focus == "shape":
        return is_picture_shape_candidate(item)
    if focus == "ctrl":
        return is_ctrl_header_candidate(item)
    if focus == "missing":
        return item.alignment_status == "missing"
    if focus == "docinfo":
        return role_control(item)[0] == "docinfo"
    raise ValueError(focus)


def _read_field(spec: FieldSpec, data: bytes) -> str:
    if spec.kind == "u16":
        value = read_u16(data, spec.offset)
        return str(value) if value is not None else "<short>"
    if spec.kind == "u32":
        value = read_u32(data, spec.offset)
        return str(value) if value is not None else "<short>"
    if spec.kind == "u32_hex":
        value = read_u32(data, spec.offset)
        return f"0x{value:08x} ({value})" if value is not None else "<short>"
    if spec.kind == "i32":
        value = read_i32(data, spec.offset)
        return str(value) if value is not None else "<short>"
    end = min(len(data), spec.offset + spec.width)
    if spec.offset >= len(data):
        return "<none>"
    return nfc_hex(data[spec.offset:end])


def table_field_rows(
    oracle: InventoryItem | None,
    generated: InventoryItem | None,
) -> list[TableFieldRow]:
    tag_name = (oracle or generated).tag_name if (oracle or generated) else "-"
    rows: list[TableFieldRow] = []

    def meta(name: str, offset: str, getter) -> None:
        left = str(getter(oracle)) if oracle else "<missing>"
        right = str(getter(generated)) if generated else "<missing>"
        rows.append(
            TableFieldRow(
                tag_name=tag_name,
                field_name=name,
                offset=offset,
                oracle=left,
                generated=right,
                status="same" if left == right else "diff",
            )
        )

    meta("record_size", "-", lambda item: item.size)
    meta("payload_len", "-", lambda item: len(item.payload_bytes))
    specs: Iterable[FieldSpec]
    if tag_name == "CTRL_HEADER":
        specs = TABLE_CTRL_FIELDS
    elif tag_name == "TABLE":
        specs = TABLE_RECORD_FIELDS
    else:
        meta("payload_head", "0x00", lambda item: item.payload_head_hex)
        return rows
    for spec in specs:
        left = _read_field(spec, oracle.payload_bytes) if oracle else "<missing>"
        right = _read_field(spec, generated.payload_bytes) if generated else "<missing>"
        rows.append(
            TableFieldRow(
                tag_name=tag_name,
                field_name=spec.field_name,
                offset=f"0x{spec.offset:02x}",
                oracle=left,
                generated=right,
                status="same" if left == right else "diff",
            )
        )
    return rows


def inventory_side(
    items: Sequence[InventoryItem],
    stream_path: str,
    record_index: int | None,
) -> InventoryItem | None:
    if record_index is None:
        return None
    for item in items:
        if item.stream_path == stream_path and item.record_index == record_index:
            return item
    return None


def table_probe_axes(
    candidates: Sequence[DiffItem],
    oracle_items: Sequence[InventoryItem],
    generated_items: Sequence[InventoryItem],
) -> list[ProbeAxis]:
    axes = [
        ProbeAxis("ctrl_outer_margin", "CTRL_HEADER(Table)", "TABLE control wrapper의 바깥 여백 4필드"),
        ProbeAxis("ctrl_common_attr", "CTRL_HEADER(Table)", "TABLE control wrapper 공통 속성 비트"),
        ProbeAxis("table_attr", "TABLE", "TABLE record 첫 4바이트 속성 비트"),
        ProbeAxis("table_tail", "TABLE", "TABLE record 0x16 이후 tail payload"),
    ]
    wanted = {
        "ctrl_outer_margin": (
            "out_margin_left",
            "out_margin_right",
            "out_margin_top",
            "out_margin_bottom",
        ),
        "ctrl_common_attr": ("common_attr",),
        "table_attr": ("table_attr",),
    }
    for item in candidates:
        oracle = inventory_side(oracle_items, item.stream_path, item.oracle_record_index)
        generated = inventory_side(
            generated_items, item.stream_path, item.generated_record_index
        )
        rows = table_field_rows(oracle, generated)
        tag_name = (oracle or generated).tag_name if (oracle or generated) else "-"
        if tag_name == "CTRL_HEADER":
            for axis_name in ("ctrl_outer_margin", "ctrl_common_attr"):
                picked = [
                    row
                    for row in rows
                    if row.field_name in wanted[axis_name] and row.status == "diff"
                ]
                if picked:
                    axes[["ctrl_outer_margin", "ctrl_common_attr"].index(axis_name)].rows.append(
                        ProbeAxisRow(
                            key=item.key,
                            oracle_record=oracle.record_uid if oracle else "<missing>",
                            generated_record=generated.record_uid if generated else "<missing>",
                            fields=[row.field_name for row in picked],
                            oracle_values=[f"{row.field_name}={row.oracle}" for row in picked],
                            generated_values=[
                                f"{row.field_name}={row.generated}" for row in picked
                            ],
                        )
                    )
        elif tag_name == "TABLE":
            picked = [
                row for row in rows if row.field_name == "table_attr" and row.status == "diff"
            ]
            if picked:
                axes[2].rows.append(
                    ProbeAxisRow(
                        key=item.key,
                        oracle_record=oracle.record_uid if oracle else "<missing>",
                        generated_record=generated.record_uid if generated else "<missing>",
                        fields=["table_attr"],
                        oracle_values=[f"table_attr={picked[0].oracle}"],
                        generated_values=[f"table_attr={picked[0].generated}"],
                    )
                )
            oracle_payload = oracle.payload_bytes if oracle else b""
            generated_payload = generated.payload_bytes if generated else b""
            if len(oracle_payload) >= 0x16 and len(generated_payload) >= 0x16:
                if oracle_payload[0x16:] != generated_payload[0x16:]:
                    axes[3].rows.append(
                        ProbeAxisRow(
                            key=item.key,
                            oracle_record=oracle.record_uid if oracle else "<missing>",
                            generated_record=generated.record_uid if generated else "<missing>",
                            fields=["table_tail_full"],
                            oracle_values=[
                                f"table_tail_full={len(oracle_payload[0x16:])} bytes: {nfc_hex(oracle_payload[0x16:])}"
                            ],
                            generated_values=[
                                f"table_tail_full={len(generated_payload[0x16:])} bytes: {nfc_hex(generated_payload[0x16:])}"
                            ],
                        )
                    )
    return axes


def pack_table_payload(
    *,
    table_attr: int,
    rows: int,
    cols: int,
    cell_spacing: int = 0,
    in_margin: tuple[int, int, int, int] = (140, 140, 140, 140),
    row_hint: int | None = None,
    col_hint: int | None = None,
    tail: bytes = b"\x00\x00",
) -> bytes:
    payload = bytearray()
    payload.extend(pack_u32(table_attr))
    payload.extend(pack_u16(rows))
    payload.extend(pack_u16(cols))
    payload.extend(pack_u16(cell_spacing))
    for value in in_margin:
        payload.extend(pack_u16(value))
    payload.extend(pack_u16(row_hint if row_hint is not None else rows))
    payload.extend(pack_u16(col_hint if col_hint is not None else cols))
    payload.extend(tail)
    return bytes(payload)


def pack_table_ctrl_payload(
    *,
    common_attr: int,
    x: int,
    y: int,
    width: int,
    height: int,
    z_or_instance: int = 1,
    out_margin: tuple[int, int, int, int] = (0, 0, 0, 0),
    tail: bytes = b"",
) -> bytes:
    payload = bytearray()
    payload.extend(fourcc_bytes("tbl "))
    payload.extend(pack_u32(common_attr))
    payload.extend(pack_i32(x))
    payload.extend(pack_i32(y))
    payload.extend(pack_i32(width))
    payload.extend(pack_i32(height))
    payload.extend(pack_u32(z_or_instance))
    for value in out_margin:
        payload.extend(pack_u16(value))
    payload.extend(tail)
    return bytes(payload)


def known_tag_names() -> set[str]:
    return {tag.tag_name for tag in TAGS}
