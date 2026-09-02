#!/usr/bin/env python3
"""Existing fields / fill-fields / batch fill CLI contracts as data.

This module does not invent DocumentCore fill logic. It ports the already
shipped envelope, targeting, dry-run, verify, and T07/#4781 first-field
rules so fixtures can be measured by the same functions that generated them.

Authority:
    src/main.rs parse_field_key / fill-fields envelope (#3329 #3476 #3383 #3702)
    tests/edit_fill_fields_contract.rs
    tests/edit_field_occurrence_contract.rs
    tests/edit_verify_contract.rs
    tests/batch_fill_contract.rs
    tests/fields_json_contract.rs
    gym/packs/core-cli/tasks/T07.json — first field 홍길동, do not clone
"""

from __future__ import annotations

import csv
import io
import json
import re
import unicodedata
from collections import Counter
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterable, Mapping, Sequence

SCHEMA_VERSION = "1.0"
HONGGILDONG = "홍길동"
CLAIM_ID = "M-fill"
KIND_CATALOG = "rhwp.form_fill.form_catalog.v1"
KIND_FILL = "rhwp.form_fill.fill_case.v1"
KIND_BATCH = "rhwp.form_fill.batch_row.v1"
KIND_PATH = "rhwp.form_fill.path_contract.v1"
KIND_HONG = "rhwp.form_fill.honggildong_4781.v1"

EXIT_OK = 0
EXIT_RUNTIME = 1
EXIT_USAGE = 2
EXIT_VERIFY = 3

@dataclass(frozen=True)
class FieldRec:
    name: str
    guide: str = ""
    value: str = ""
    memo: str = ""
    field_type: str = "ClickHere"
    editable_in_form: bool = True
    field_id: int = 0
    section: int = 0
    paragraph: int = 0
    nested: tuple[str, ...] = ()

    def to_json(self, index: int) -> dict[str, Any]:
        return {
            "fieldId": self.field_id or (2_000_000_000 + index),
            "fieldType": self.field_type,
            "name": self.name,
            "value": self.value,
            "guide": self.guide,
            "memo": self.memo,
            "editableInForm": self.editable_in_form,
            "location": {
                "section": self.section,
                "paragraph": self.paragraph,
                "nested": list(self.nested),
            },
        }


@dataclass(frozen=True)
class FormCatalog:
    ident: str
    title: str
    family: str
    sample: str
    related_issue: str
    why: str
    fmt: str
    fields: tuple[FieldRec, ...]
    genre: str = ""
    notes: str = ""

    @property
    def field_count(self) -> int:
        return len(self.fields)

    @property
    def first_name(self) -> str:
        if not self.fields:
            return ""
        return self.fields[0].name

    def name_counts(self) -> dict[str, int]:
        counts: dict[str, int] = {}
        for rec in self.fields:
            counts[rec.name] = counts.get(rec.name, 0) + 1
        return counts

    def repeated_names(self) -> list[str]:
        return [name for name, count in self.name_counts().items() if count > 1]

    def unique_names(self) -> list[str]:
        seen: list[str] = []
        counts = self.name_counts()
        for rec in self.fields:
            if rec.name not in seen and counts[rec.name] == 1:
                seen.append(rec.name)
        return seen

    def names_in_order(self) -> list[str]:
        return [rec.name for rec in self.fields]

    def values_of(self, name: str) -> list[str]:
        return [rec.value for rec in self.fields if rec.name == name]


@dataclass(frozen=True)
class FilledHit:
    name: str
    occurrence: int
    value: str

    def to_json(self) -> dict[str, Any]:
        return {"name": self.name, "occurrence": self.occurrence, "value": self.value}


@dataclass
class Ambiguous:
    name: str
    matched: int
    total: int

    def to_json(self) -> dict[str, Any]:
        return {"name": self.name, "matched": self.matched, "total": self.total}


@dataclass
class FillPlan:
    filled: list[FilledHit] = field(default_factory=list)
    not_found: list[str] = field(default_factory=list)
    ambiguous: list[Ambiguous] = field(default_factory=list)

    @property
    def filled_count(self) -> int:
        return len(self.filled)

    def to_json(self) -> dict[str, Any]:
        return {
            "filledCount": self.filled_count,
            "filled": [hit.to_json() for hit in self.filled],
            "notFound": list(self.not_found),
            "ambiguous": [item.to_json() for item in self.ambiguous],
        }


def nfc(value: str) -> str:
    return unicodedata.normalize("NFC", value)


def parse_field_key(key: str) -> tuple[str, int]:
    """Port of src/main.rs parse_field_key (#3476).

    ``피규제집단명[3]`` → ``(피규제집단명, 3)``.
    A trailing ``[…]`` that is not a usize stays part of the name.
    """
    open_at = key.rfind("[")
    if open_at < 0:
        return key, 0
    if not key.endswith("]"):
        return key, 0
    inner = key[open_at + 1 : -1]
    try:
        index = int(inner, 10)
    except ValueError:
        return key, 0
    if str(index) != inner:
        return key, 0
    if index < 0:
        return key, 0
    return key[:open_at], index


def key_has_index_brackets(key: str) -> bool:
    name, _occ = parse_field_key(key)
    return name != key and key.endswith("]")


def survey_fields(form: FormCatalog) -> dict[str, Any]:
    """``fields --json`` envelope (#3281). Empty document is not an error."""
    records = [rec.to_json(index) for index, rec in enumerate(form.fields)]
    return {
        "schemaVersion": SCHEMA_VERSION,
        "source": form.sample,
        "fieldCount": len(records),
        "fields": records,
        "textSecurity": {"status": "clean"},
    }


def plan_fill(form: FormCatalog, data: Mapping[str, str]) -> FillPlan:
    """Name / ``이름[N]`` targeting used by ``edit fill-fields``.

    Plain name + several matches: fill the first occurrence and report
    ``ambiguous`` (#3476). Out-of-range index or unknown name: ``notFound``
    echoes the caller key. This does not write bytes.
    """
    counts = form.name_counts()
    plan = FillPlan()
    for key, value in data.items():
        name, occurrence = parse_field_key(key)
        total = counts.get(name, 0)
        if total == 0 or occurrence >= total:
            plan.not_found.append(key)
            continue
        if occurrence == 0 and total > 1 and "[" not in key:
            plan.ambiguous.append(Ambiguous(name=name, matched=1, total=total))
        plan.filled.append(FilledHit(name=name, occurrence=occurrence, value=str(value)))
    return plan


def apply_values(form: FormCatalog, plan: FillPlan) -> tuple[FieldRec, ...]:
    updated = [rec for rec in form.fields]
    for hit in plan.filled:
        seen = 0
        for index, rec in enumerate(updated):
            if rec.name != hit.name:
                continue
            if seen == hit.occurrence:
                updated[index] = FieldRec(
                    name=rec.name,
                    guide=rec.guide,
                    value=hit.value,
                    memo=rec.memo,
                    field_type=rec.field_type,
                    editable_in_form=rec.editable_in_form,
                    field_id=rec.field_id,
                    section=rec.section,
                    paragraph=rec.paragraph,
                    nested=rec.nested,
                )
                break
            seen += 1
    return tuple(updated)


def output_format_label(source_fmt: str, explicit_out: str | None) -> str:
    """#3383 input-format preserve. HWPX + ``-o *.hwp`` is the only flip."""
    source = "hwpx" if source_fmt == "hwpx" else "hwp5"
    if explicit_out is None:
        return source
    ext = Path(explicit_out).suffix.lower()
    if source == "hwpx" and ext == ".hwp":
        return "hwp5"
    return source


def default_output_name(sample: str) -> str:
    path = Path(sample)
    return f"{path.stem}_filled{path.suffix}"


def fill_envelope(
    form: FormCatalog,
    data: Mapping[str, str],
    *,
    dry_run: bool = False,
    verify: bool = False,
    output: str | None = None,
    source_override: str | None = None,
) -> dict[str, Any]:
    plan = plan_fill(form, data)
    after = apply_values(form, plan)
    env: dict[str, Any] = {
        "schemaVersion": SCHEMA_VERSION,
        "source": source_override or form.sample,
        "dryRun": dry_run,
        "filledCount": plan.filled_count,
        "filled": [hit.to_json() for hit in plan.filled],
        "notFound": list(plan.not_found),
        "ambiguous": [item.to_json() for item in plan.ambiguous],
        "confusable": [],
        "verify": None,
    }
    if not dry_run:
        out_path = output or default_output_name(form.sample)
        env["output"] = out_path
        env["outputFormat"] = output_format_label(form.fmt, output)
        if verify:
            env["verify"] = verify_fill(form.fields, after, plan)
    return env


def verify_fill(
    before: Sequence[FieldRec],
    after: Sequence[FieldRec],
    plan: FillPlan,
) -> dict[str, Any]:
    """Re-parse contrast used by ``--verify`` (#3702).

    ``identical`` here means every requested hit survived on the after
    document. A mismatch is ``identical: false`` and exit 3.
    """
    diffs = 0
    after_values = values_by_name(after)
    for hit in plan.filled:
        got = after_values.get(hit.name, [])
        if hit.occurrence >= len(got) or got[hit.occurrence] != hit.value:
            diffs += 1
    if len(before) != len(after):
        diffs += abs(len(after) - len(before))
    return {"identical": diffs == 0, "diffCount": diffs}


def values_by_name(fields: Sequence[FieldRec]) -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for rec in fields:
        out.setdefault(rec.name, []).append(rec.value)
    return out


def exit_for_envelope(env: Mapping[str, Any], *, usage: bool = False, runtime: bool = False) -> int:
    if usage:
        return EXIT_USAGE
    if runtime:
        return EXIT_RUNTIME
    verify = env.get("verify")
    if isinstance(verify, Mapping) and verify.get("identical") is False:
        return EXIT_VERIFY
    return EXIT_OK


def first_field_honggildong_request(form: FormCatalog) -> dict[str, str]:
    """T07 / #4781: only the first surveyed field is 홍길동."""
    if not form.fields:
        return {}
    return {form.first_name: HONGGILDONG}


def clone_honggildong_request(form: FormCatalog) -> dict[str, str]:
    """Forbidden clone: every unique name becomes 홍길동."""
    return {name: HONGGILDONG for name in form.unique_names() or form.names_in_order()}


def detect_honggildong_clone(
    before: Sequence[FieldRec],
    after: Sequence[FieldRec],
    intended_names: Iterable[str],
) -> dict[str, Any]:
    """First-field 홍길동 must not be copied onto sibling fields (#4781).

    A clone is any field that was not 홍길동 before, is not in the intended
    name set, and is 홍길동 after. T07 scores ``fields[0].value`` only, but
    the task text forbids filling the other fields.
    """
    intended = set(intended_names)
    cloned: list[dict[str, Any]] = []
    first_after = after[0].value if after else ""
    first_ok = first_after == HONGGILDONG
    for index, (old, new) in enumerate(zip(before, after)):
        if new.value != HONGGILDONG:
            continue
        if old.name in intended:
            continue
        if old.value == HONGGILDONG:
            continue
        cloned.append(
            {
                "index": index,
                "name": old.name,
                "before": old.value,
                "after": new.value,
            }
        )
    return {
        "firstFieldName": before[0].name if before else "",
        "firstFieldValue": first_after,
        "firstFieldOk": first_ok,
        "cloned": cloned,
        "cloneCount": len(cloned),
        "verdict": "pass" if first_ok and not cloned else "clone_forbidden",
    }


def honggildong_case(
    form: FormCatalog,
    data: Mapping[str, str],
    *,
    intended: Sequence[str] | None = None,
) -> dict[str, Any]:
    plan = plan_fill(form, data)
    after = apply_values(form, plan)
    intended_names = list(intended) if intended is not None else list(data.keys())
    intended_plain = [parse_field_key(key)[0] for key in intended_names]
    detect = detect_honggildong_clone(form.fields, after, intended_plain)
    return {
        "schemaVersion": SCHEMA_VERSION,
        "kind": KIND_HONG,
        "form": form.ident,
        "sample": form.sample,
        "relatedIssue": "#4781",
        "data": dict(data),
        "plan": plan.to_json(),
        "detect": detect,
        "afterValues": [rec.value for rec in after],
    }


def parse_jsonl_rows(text: str) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for line_no, raw in enumerate(text.splitlines()):
        line = raw.strip()
        if not line:
            continue
        try:
            parsed = json.loads(line)
        except json.JSONDecodeError as exc:
            rows.append({"row": len(rows), "error": f"json: {exc}", "exitClass": "runtime"})
            continue
        if not isinstance(parsed, dict):
            rows.append(
                {
                    "row": len(rows),
                    "error": "jsonl row must be an object",
                    "exitClass": "runtime",
                }
            )
            continue
        rows.append({"row": len(rows), "data": {str(k): str(v) for k, v in parsed.items()}})
    return rows


def parse_csv_rows(text: str) -> list[dict[str, Any]]:
    """UTF-8 BOM + RFC4180. Header cells are field names (#3719)."""
    cleaned = text.lstrip("\ufeff")
    reader = csv.reader(io.StringIO(cleaned))
    try:
        header = next(reader)
    except StopIteration:
        return []
    header = [cell for cell in header]
    if not header:
        return []
    rows: list[dict[str, Any]] = []
    for cells in reader:
        if len(cells) != len(header):
            rows.append(
                {
                    "row": len(rows),
                    "error": f"column count {len(cells)} != {len(header)}",
                    "exitClass": "runtime",
                }
            )
            continue
        data = {header[i]: cells[i] for i in range(len(header))}
        rows.append({"row": len(rows), "data": data})
    return rows


def sanitize_name_field(value: str) -> str:
    cleaned = re.sub(r'[<>:"/\\|?*]', "_", value)
    cleaned = cleaned.replace("..", "_")
    cleaned = cleaned.strip(" .")
    return cleaned or "unnamed"


def batch_output_name(
    row_index: int,
    data: Mapping[str, str],
    *,
    name_field: str | None,
    fmt: str,
    used: Counter[str],
) -> str:
    ext = ".hwpx" if fmt == "hwpx" else ".hwp"
    if name_field:
        base = sanitize_name_field(str(data.get(name_field, "")))
    else:
        base = f"{row_index + 1:04d}"
    key = base.lower()
    used[key] += 1
    if used[key] > 1:
        base = f"{base}_{used[key]}"
    return f"{base}{ext}"


def batch_fill(
    form: FormCatalog,
    rows: Sequence[Mapping[str, Any]],
    *,
    dry_run: bool = False,
    verify: bool = False,
    name_field: str | None = None,
    out_dir: str = "out",
) -> list[dict[str, Any]]:
    used: Counter[str] = Counter()
    records: list[dict[str, Any]] = []
    for raw in rows:
        row_index = int(raw.get("row", len(records)))
        if raw.get("error"):
            records.append(
                {
                    "schemaVersion": SCHEMA_VERSION,
                    "row": row_index,
                    "error": raw["error"],
                    "exitClass": raw.get("exitClass", "runtime"),
                }
            )
            continue
        data = dict(raw.get("data") or {})
        env = fill_envelope(
            form,
            data,
            dry_run=dry_run,
            verify=verify,
            output=None,
        )
        env["row"] = row_index
        if name_field and name_field not in form.name_counts() and name_field not in env["notFound"]:
            env["notFound"] = list(env["notFound"]) + [name_field]
        if not dry_run and "error" not in env:
            filename = batch_output_name(
                row_index, data, name_field=name_field, fmt=form.fmt, used=used
            )
            env["output"] = f"{out_dir.rstrip('/')}/{filename}"
            env["outputFormat"] = "hwpx" if form.fmt == "hwpx" else "hwp5"
        elif dry_run:
            env.pop("output", None)
            env.pop("outputFormat", None)
        records.append(env)
    return records


def batch_exit(records: Sequence[Mapping[str, Any]]) -> int:
    if any(rec.get("error") for rec in records):
        return EXIT_RUNTIME
    if any(
        isinstance(rec.get("verify"), Mapping) and rec["verify"].get("identical") is False
        for rec in records
    ):
        return EXIT_VERIFY
    return EXIT_OK


def gate_single(env: Mapping[str, Any]) -> bool:
    """Recipe 01 machine gate: verify + empty notFound/ambiguous."""
    not_found = env.get("notFound") or []
    ambiguous = env.get("ambiguous") or []
    verify = env.get("verify")
    identical = True if verify is None else bool(verify.get("identical"))
    return identical and len(not_found) == 0 and len(ambiguous) == 0


def gate_batch(records: Sequence[Mapping[str, Any]], name_field: str | None = None) -> bool:
    for rec in records:
        if rec.get("error"):
            return False
        leftover = [name for name in (rec.get("notFound") or []) if name != name_field]
        if leftover:
            return False
        if rec.get("ambiguous"):
            return False
        verify = rec.get("verify")
        if isinstance(verify, Mapping) and verify.get("identical") is False:
            return False
    return True


def argv_fields(sample: str) -> list[str]:
    return ["fields", sample, "--json"]


def argv_fill(
    sample: str,
    data: str,
    *,
    output: str | None = None,
    dry_run: bool = False,
    verify: bool = False,
) -> list[str]:
    args = ["edit", "fill-fields", sample, "--data", data]
    if output:
        args.extend(["-o", output])
    if dry_run:
        args.append("--dry-run")
    if verify:
        args.append("--verify")
    args.append("--json")
    return args


def argv_batch(
    sample: str,
    data_path: str,
    out_dir: str,
    *,
    dry_run: bool = False,
    verify: bool = False,
    name_field: str | None = None,
    threads: int | None = None,
) -> list[str]:
    args = [
        "batch",
        "fill",
        "--form",
        sample,
        "--data",
        data_path,
        "--out-dir",
        out_dir,
    ]
    if name_field:
        args.extend(["--name-field", name_field])
    if dry_run:
        args.append("--dry-run")
    if verify:
        args.append("--verify")
    if threads is not None:
        args.extend(["--threads", str(threads)])
    args.append("--json")
    return args


def catalog_to_json(form: FormCatalog) -> dict[str, Any]:
    return {
        "schemaVersion": SCHEMA_VERSION,
        "kind": KIND_CATALOG,
        "id": form.ident,
        "title": form.title,
        "family": form.family,
        "genre": form.genre,
        "sample": form.sample,
        "format": form.fmt,
        "relatedIssue": form.related_issue,
        "why": form.why,
        "notes": form.notes,
        "fieldCount": form.field_count,
        "firstFieldName": form.first_name,
        "repeatedNames": form.repeated_names(),
        "uniqueNames": form.unique_names(),
        "nameCounts": form.name_counts(),
        "fields": [rec.to_json(index) for index, rec in enumerate(form.fields)],
    }



