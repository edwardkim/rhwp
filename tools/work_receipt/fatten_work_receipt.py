#!/usr/bin/env python3
"""M-rcpt: expand replay / audit / lineage fixtures.

Reads existing CLI contracts (no new flags) and writes exception
envelopes, capsule chains, audit layouts, lineage topologies, and
working docs under tools/work_receipt/. Does not touch gym,
canvaskit, serializers, pdf, layout-anomaly, oracle, render_backend,
proptest, fidelity, hwp5-inventory, inspect, or page-count.

    python tools/work_receipt/fatten_work_receipt.py
    python tools/work_receipt/test_fatten_work_receipt.py
"""

from __future__ import annotations

import argparse
import json
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from catalog import (
    AUDIT_LAYOUTS,
    EXCEPTIONS,
    LINEAGE,
    SCENARIOS,
    Scenario,
    scenario_by_id,
)
from contracts import (
    AUDIT_REQUIRED,
    CAPSULE_REQUIRED,
    CLAIM_ID,
    ENVELOPE_SCHEMA_VERSION,
    GENERATOR,
    ISSUE,
    KIND_CATALOG,
    LINEAGE_REQUIRED,
    NEEDLE,
    PLAN_ACTIONS,
    REPLAY_REQUIRED,
    ZERO64,
    audit_rate,
    canonical_json,
    classify_audit,
    classify_lineage,
    classify_replay,
    is_sha256_hex,
    lineage_ok,
    parent_ok,
    plan_text_of,
    sha256_hex,
    stdout_silent_on_failure,
    validated_capsule_plan,
)

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent


def utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text, encoding="utf-8", newline="\n")


def write_json(path: Path, data: Any) -> None:
    write_text(path, json.dumps(data, ensure_ascii=False, indent=2))


def fixture_hash(ident: str, role: str, payload: str) -> str:
    """Deterministic stand-in for live HWP bytes.

    Fixtures lock field names / exit / needles. They do not claim to
    be a live ``rhwp replay`` of a binary in this sparse checkout.
    """
    return sha256_hex(f"{CLAIM_ID}:{role}:{ident}:{payload}")


def make_plan(scenario: Scenario) -> dict[str, Any]:
    return {
        "planVersion": "1.0",
        "input": scenario.input_path,
        "output": f"out/{scenario.output_name}",
        "steps": scenario.steps,
    }


def make_receipt(
    scenario: Scenario,
    *,
    mode: str,
    expect: str | None,
    reproduced: bool | None,
) -> dict[str, Any]:
    plan = make_plan(scenario)
    text = plan_text_of(plan)
    output_sha = fixture_hash(scenario.ident, "output", text)
    return {
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "mode": mode,
        "input": scenario.input_path,
        "inputSha256": fixture_hash(scenario.ident, "input", scenario.input_path),
        "planSha256": sha256_hex(text),
        "outputSha256": output_sha,
        "toolVersion": "fixture-0.0.0-not-a-release",
        "steps": len(scenario.steps),
        "reproduced": reproduced,
        "expectedOutputSha256": expect,
    }


def make_capsule(
    scenario: Scenario,
    *,
    parent: dict[str, Any] | None,
    parent_bytes: bytes | None,
    parent_rel: str | None,
    tamper: str | None = None,
) -> dict[str, Any]:
    plan = make_plan(scenario)
    text = plan_text_of(plan)
    receipt = make_receipt(scenario, mode="attest", expect=None, reproduced=None)
    parent_link: Any = None
    if parent is not None:
        assert parent_bytes is not None and parent_rel is not None
        parent_link = {
            "capsule": parent_rel,
            "sha256": sha256_hex(parent_bytes),
        }
    capsule: dict[str, Any] = {
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "kind": "workCapsule",
        "parent": parent_link,
        "plan": plan,
        "planText": text,
        "receipt": receipt,
    }
    return apply_tamper(capsule, tamper)


def apply_tamper(capsule: dict[str, Any], tamper: str | None) -> dict[str, Any]:
    if not tamper:
        return capsule
    if tamper == "output":
        capsule["receipt"]["outputSha256"] = ZERO64
    elif tamper == "input":
        capsule["receipt"]["inputSha256"] = ZERO64
    elif tamper == "steps":
        capsule["receipt"]["steps"] = 999
    elif tamper == "plan":
        capsule["plan"] = dict(capsule["plan"])
        capsule["plan"]["output"] = "output-neutral-tamper.hwp"
    elif tamper == "plan_text":
        changed = json.loads(capsule["planText"])
        changed["output"] = "another-output-neutral-tamper.hwp"
        capsule["plan"] = changed
        capsule["planText"] = json.dumps(changed, ensure_ascii=False, separators=(",", ":"))
    elif tamper == "pretty":
        # Extra trailing space inside a string field — file bytes change,
        # parsed plan does not. Lineage parentOk cares; audit re-run may not.
        capsule["receipt"] = dict(capsule["receipt"])
        capsule["receipt"]["_pretty"] = " "
    elif tamper == "kind":
        capsule["kind"] = "note"
    elif tamper == "bad_output_sha":
        capsule["receipt"]["outputSha256"] = "xyz"
    elif tamper == "bad_input_sha":
        capsule["receipt"].pop("inputSha256", None)
    elif tamper == "missing_plan_text":
        capsule.pop("planText", None)
    elif tamper == "missing_plan_sha":
        capsule["receipt"].pop("planSha256", None)
    elif tamper == "missing_parent":
        capsule.pop("parent", None)
    elif tamper == "missing_parent_sha":
        if isinstance(capsule.get("parent"), dict):
            capsule["parent"] = dict(capsule["parent"])
            capsule["parent"].pop("sha256", None)
    elif tamper == "missing_parent_capsule":
        if isinstance(capsule.get("parent"), dict):
            capsule["parent"] = dict(capsule["parent"])
            capsule["parent"].pop("capsule", None)
    elif tamper == "lineage_break":
        capsule["receipt"]["inputSha256"] = ZERO64
    elif tamper == "toolversion":
        capsule["receipt"]["toolVersion"] = "0.0.0-other-build"
    else:
        raise ValueError(tamper)
    return capsule


def replay_case_doc(scenario: Scenario) -> dict[str, Any]:
    plan = make_plan(scenario)
    text = plan_text_of(plan)
    attest = make_receipt(scenario, mode="attest", expect=None, reproduced=None)
    out_sha = attest["outputSha256"]
    attest_exit, attest_mode = classify_replay(
        has_plan=True,
        plan_parse_ok=True,
        has_input=True,
        sign_key=False,
        capsule=False,
        same_file=False,
        expect=None,
        reproduced=None,
    )
    ok_exit, ok_mode = classify_replay(
        has_plan=True,
        plan_parse_ok=True,
        has_input=True,
        sign_key=False,
        capsule=False,
        same_file=False,
        expect=out_sha,
        reproduced=True,
    )
    bad_exit, bad_mode = classify_replay(
        has_plan=True,
        plan_parse_ok=True,
        has_input=True,
        sign_key=False,
        capsule=False,
        same_file=False,
        expect=ZERO64,
        reproduced=False,
    )
    actions = [step["action"] for step in scenario.steps]
    return {
        "schemaVersion": "1.0",
        "kind": "workReceiptReplayCase",
        "ident": scenario.ident,
        "claim": CLAIM_ID,
        "issue": ISSUE,
        "family": scenario.family,
        "genre": scenario.genre,
        "title": scenario.title,
        "why": scenario.why,
        "notes": scenario.notes,
        "input": scenario.input_path,
        "output": f"out/{scenario.output_name}",
        "ext": scenario.ext,
        "actions": actions,
        "stepCount": len(scenario.steps),
        "plan": plan,
        "planText": text,
        "planSha256": sha256_hex(text),
        "command": "replay",
        "receipt": attest,
        "expected": {
            "attest": {
                "exit": attest_exit,
                "mode": attest_mode,
                "reproduced": None,
                "userOutputCreated": False,
            },
            "verify_ok": {
                "exit": ok_exit,
                "mode": ok_mode,
                "reproduced": True,
                "expectedOutputSha256": out_sha,
            },
            "verify_mismatch": {
                "exit": bad_exit,
                "mode": bad_mode,
                "reproduced": False,
                "expectedOutputSha256": ZERO64,
                "judgment": True,
            },
        },
        "claimedOutputPathUntouched": True,
        "generator": GENERATOR,
    }


def exception_doc(spec: Any) -> dict[str, Any]:
    silent = stdout_silent_on_failure(spec.exit, spec.stdout_bytes < 0)
    return {
        "schemaVersion": "1.0",
        "kind": "workReceiptExceptionEnvelope",
        "ident": spec.ident,
        "claim": CLAIM_ID,
        "issue": ISSUE,
        "command": spec.command,
        "family": spec.family,
        "argv": spec.argv,
        "exit": spec.exit,
        "stdoutBytes": spec.stdout_bytes,
        "stdoutSilent": silent if spec.stdout_bytes == 0 else False,
        "needle": spec.needle,
        "envelopeKeys": list(spec.envelope_keys),
        "why": spec.why,
        "judgment": spec.exit == 3,
        "io": spec.exit == 1,
        "usage": spec.exit == 2,
        "notes": (
            "exit 3 은 도구 고장이 아니라 판정 데이터다. "
            "exit 1 은 IO, exit 2 는 사용법. "
            "실패 경로 stdout 은 0바이트가 기본이고, "
            "replay --json 엔진 오류만 {schemaVersion,error} 를 싣는다."
        ),
        "generator": GENERATOR,
    }


class Bundle:
    def __init__(self, out_root: Path) -> None:
        self.out_root = out_root
        self.written: list[str] = []
        self.replay: list[dict[str, Any]] = []
        self.exceptions: list[dict[str, Any]] = []
        self.capsules: list[dict[str, Any]] = []
        self.audits: list[dict[str, Any]] = []
        self.lineage: list[dict[str, Any]] = []

    def put(self, rel: str, data: Any | str) -> Path:
        path = self.out_root / rel
        if isinstance(data, str):
            write_text(path, data)
        else:
            write_json(path, data)
        self.written.append(rel.replace("\\", "/"))
        return path


def emit_schemas(bundle: Bundle) -> None:
    bundle.put(
        "schema/replay_case.v1.json",
        {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "rhwp.work_receipt.replay_case.v1",
            "title": "replay 영수증 케이스",
            "type": "object",
            "required": [
                "schemaVersion",
                "kind",
                "ident",
                "plan",
                "planText",
                "planSha256",
                "expected",
            ],
            "properties": {
                "kind": {"const": "workReceiptReplayCase"},
                "expected": {
                    "type": "object",
                    "required": ["attest", "verify_ok", "verify_mismatch"],
                },
            },
        },
    )
    bundle.put(
        "schema/exception_envelope.v1.json",
        {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "rhwp.work_receipt.exception_envelope.v1",
            "title": "replay/audit/lineage 예외 봉투",
            "type": "object",
            "required": ["ident", "command", "argv", "exit", "why"],
            "properties": {
                "exit": {"enum": [0, 1, 2, 3]},
                "kind": {"const": "workReceiptExceptionEnvelope"},
            },
        },
    )
    bundle.put(
        "schema/work_capsule.v1.json",
        {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "rhwp.work_receipt.work_capsule.v1",
            "title": "작업 캡슐",
            "type": "object",
            "required": list(CAPSULE_REQUIRED),
            "properties": {
                "kind": {"const": "workCapsule"},
                "schemaVersion": {"const": ENVELOPE_SCHEMA_VERSION},
            },
        },
    )
    bundle.put(
        "schema/audit_layout.v1.json",
        {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "rhwp.work_receipt.audit_layout.v1",
            "title": "감사 폴더 레이아웃",
            "type": "object",
            "required": ["ident", "total", "reproduced", "reproducedRate", "exit"],
        },
    )
    bundle.put(
        "schema/lineage_topology.v1.json",
        {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "rhwp.work_receipt.lineage_topology.v1",
            "title": "계보 토폴로지",
            "type": "object",
            "required": ["ident", "depth", "valid", "exit", "links"],
        },
    )


def emit_replay(bundle: Bundle) -> dict[str, dict[str, Any]]:
    capsules: dict[str, dict[str, Any]] = {}
    for scenario in SCENARIOS:
        doc = replay_case_doc(scenario)
        bundle.replay.append(doc)
        bundle.put(f"fixtures/replay/cases/{scenario.ident}.json", doc)
        capsule = make_capsule(
            scenario, parent=None, parent_bytes=None, parent_rel=None
        )
        capsules[scenario.ident] = capsule
        bundle.put(f"fixtures/capsules/{scenario.ident}.capsule.json", capsule)
        bundle.capsules.append(
            {
                "ident": scenario.ident,
                "kind": capsule["kind"],
                "steps": capsule["receipt"]["steps"],
                "planSha256": capsule["receipt"]["planSha256"],
                "parent": capsule["parent"],
            }
        )
    return capsules


def emit_tamper_capsules(
    bundle: Bundle, base: dict[str, dict[str, Any]]
) -> dict[str, dict[str, Any]]:
    mapping = {
        "tamper_output": ("notice_year", "output"),
        "tamper_input": ("org_rename", "input"),
        "tamper_steps": ("dept_rename", "steps"),
        "tamper_plan": ("addr_fix", "plan"),
        "tamper_plan_text": ("phone_hyphen", "plan_text"),
        "pretty_print": ("amount_comma", "pretty"),
        "wrong_kind": ("deadline_iso", "kind"),
        "bad_output_sha": ("law_ref", "bad_output_sha"),
        "bad_input_sha": ("page_ref", "bad_input_sha"),
        "missing_plan_text": ("footer_page", "missing_plan_text"),
        "toolversion_other": ("title_typo", "toolversion"),
    }
    out: dict[str, dict[str, Any]] = {}
    for ident, (src, tamper) in mapping.items():
        raw = json.loads(json.dumps(base[src], ensure_ascii=False))
        capsule = apply_tamper(raw, tamper)
        out[ident] = capsule
        bundle.put(f"fixtures/capsules/{ident}.capsule.json", capsule)
        check = validated_capsule_plan(capsule)
        bundle.capsules.append(
            {
                "ident": ident,
                "tamper": tamper,
                "source": src,
                "validation": check if isinstance(check, str) else "ok",
            }
        )
    # invalid json is not a capsule object
    bundle.put("fixtures/capsules/invalid_json.capsule.json", "{this is not json\n")
    return out


def emit_chain_capsules(
    bundle: Bundle, base: dict[str, dict[str, Any]]
) -> dict[str, tuple[dict[str, Any], bytes]]:
    """A→B chain used by audit same-folder and lineage two-link."""
    parent = base["notice_year"]
    parent_text = canonical_json(parent)
    parent_bytes = parent_text.encode("utf-8")
    child_sc = scenario_by_id("org_rename")
    # Chronicle: child input hash equals parent output hash.
    child = make_capsule(
        child_sc,
        parent=parent,
        parent_bytes=parent_bytes,
        parent_rel="notice_year.capsule.json",
    )
    child["receipt"]["inputSha256"] = parent["receipt"]["outputSha256"]
    child_bytes = canonical_json(child).encode("utf-8")
    bundle.put("fixtures/capsules/chain_child.capsule.json", child)
    assert lineage_ok(parent["receipt"]["outputSha256"], child["receipt"]["inputSha256"])
    assert parent_ok(child["parent"]["sha256"], sha256_hex(parent_bytes))
    broken = json.loads(json.dumps(child, ensure_ascii=False))
    broken["receipt"]["inputSha256"] = ZERO64
    bundle.put("fixtures/capsules/chain_child_broken.capsule.json", broken)
    no_sha = json.loads(json.dumps(child, ensure_ascii=False))
    no_sha = apply_tamper(no_sha, "missing_parent_sha")
    bundle.put("fixtures/capsules/chain_child_no_parent_sha.capsule.json", no_sha)
    no_field = json.loads(json.dumps(child, ensure_ascii=False))
    no_field = apply_tamper(no_field, "missing_parent")
    bundle.put("fixtures/capsules/chain_child_no_parent.capsule.json", no_field)
    return {
        "parent": (parent, parent_bytes),
        "child": (child, child_bytes),
        "broken": (broken, canonical_json(broken).encode("utf-8")),
    }


def emit_exceptions(bundle: Bundle) -> None:
    for spec in EXCEPTIONS:
        doc = exception_doc(spec)
        bundle.exceptions.append(doc)
        bundle.put(f"fixtures/exceptions/{spec.command}/{spec.ident}.json", doc)


def emit_audit(bundle: Bundle, capsules: dict[str, dict[str, Any]], tampers: dict[str, dict[str, Any]]) -> None:
    pool = dict(capsules)
    pool.update(tampers)
    pool["chain_child"] = json.loads(
        (bundle.out_root / "fixtures/capsules/chain_child.capsule.json").read_text(
            encoding="utf-8"
        )
    )
    pool["invalid_json"] = None
    for layout in AUDIT_LAYOUTS:
        failed = []
        for member in layout.members:
            if member == "invalid_json":
                failed.append(
                    {
                        "capsule": f"{member}.capsule.json",
                        "error": "JSON 파싱 실패",
                    }
                )
                continue
            capsule = pool[member]
            check = validated_capsule_plan(capsule) if capsule.get("kind") == "workCapsule" else NEEDLE["audit_kind"]
            if isinstance(check, str):
                failed.append({"capsule": f"{member}.capsule.json", "error": check})
                continue
            if member.startswith("tamper_output") or (
                capsule["receipt"].get("outputSha256") == ZERO64
                and is_sha256_hex(capsule["receipt"].get("outputSha256", ""))
            ):
                if member == "tamper_output":
                    failed.append(
                        {
                            "capsule": f"{member}.capsule.json",
                            "expected": capsule["receipt"]["outputSha256"],
                            "actual": fixture_hash("notice_year", "output", "live"),
                        }
                    )
            if member == "tamper_input":
                failed.append(
                    {
                        "capsule": f"{member}.capsule.json",
                        "kind": "inputSha256",
                        "expected": capsule["receipt"]["inputSha256"],
                        "actual": fixture_hash("org_rename", "input", "live"),
                    }
                )
        # Prefer catalog accounting — fixtures are the declared matrix.
        rate = 0.0 if layout.total == 0 else audit_rate(layout.reproduced, layout.total)
        exit_code = classify_audit(
            dir_exists=True, total=layout.total, failed=layout.total - layout.reproduced
        )
        if layout.ident == "empty":
            exit_code = classify_audit(dir_exists=True, total=0, failed=0)
        env = {
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "root": f"fixtures/audit-layouts/{layout.ident}",
            "total": layout.total,
            "reproduced": layout.reproduced,
            "failed": [
                {"capsule": f"{name}.capsule.json", "kind": kind}
                for name, kind in zip(
                    [m for m in layout.members if m.startswith("tamper") or m in layout.failed_kinds or m.startswith("bad") or m.startswith("wrong") or m.startswith("missing") or m == "invalid_json"],
                    layout.failed_kinds,
                )
            ],
            "reproducedRate": rate,
        }
        # Rebuild failed from declared kinds to keep rate/len consistent.
        env["failed"] = []
        fail_members = [m for m in layout.members if m not in ("notice_year", "org_rename", "dept_rename", "chain_child", "pretty_print", "toolversion_other")]
        if layout.ident == "pretty-print":
            fail_members = []
        if layout.ident == "toolversion-mismatch":
            fail_members = []
        for member, kind in zip(fail_members, layout.failed_kinds):
            item: dict[str, Any] = {"capsule": f"{member}.capsule.json"}
            if kind in ("outputSha256", "inputSha256", "steps"):
                item["kind"] = kind
            else:
                item["error"] = {
                    "plan_vs_text": NEEDLE["audit_plan_vs_text"],
                    "plan_text_sha": NEEDLE["audit_plan_text_sha"],
                    "kind": NEEDLE["audit_kind"],
                    "output_sha": NEEDLE["audit_output_sha"],
                    "input_sha": NEEDLE["audit_input_sha"],
                    "json": "JSON 파싱 실패",
                    "plan_text_missing": NEEDLE["audit_plan_text_missing"],
                    "steps": NEEDLE["audit_steps"],
                }.get(kind, kind)
            env["failed"].append(item)
        doc = {
            "schemaVersion": "1.0",
            "kind": "workReceiptAuditLayout",
            "ident": layout.ident,
            "title": layout.title,
            "why": layout.why,
            "notes": layout.notes,
            "members": list(layout.members),
            "ignored": list(layout.ignored),
            "recursive": False,
            "exit": exit_code,
            "required": list(AUDIT_REQUIRED),
            "envelope": env,
            "command": ["audit", f"fixtures/audit-layouts/{layout.ident}", "--json"],
            "generator": GENERATOR,
        }
        bundle.audits.append(doc)
        bundle.put(f"fixtures/audit-layouts/{layout.ident}/layout.json", doc)
        for member in layout.members:
            if member == "invalid_json":
                bundle.put(
                    f"fixtures/audit-layouts/{layout.ident}/{member}.capsule.json",
                    "{this is not json\n",
                )
                continue
            src = pool[member]
            bundle.put(
                f"fixtures/audit-layouts/{layout.ident}/{member}.capsule.json",
                src,
            )
        for ignored in layout.ignored:
            if ignored.endswith(".capsule.json"):
                bundle.put(
                    f"fixtures/audit-layouts/{layout.ident}/{ignored}",
                    capsules["notice_year"],
                )
            elif ignored.endswith(".txt"):
                bundle.put(
                    f"fixtures/audit-layouts/{layout.ident}/{ignored}",
                    "이 파일은 *.capsule.json 이 아니라 감사 대상이 아니다.\n",
                )
            else:
                bundle.put(
                    f"fixtures/audit-layouts/{layout.ident}/{ignored}",
                    {"note": "not a capsule"},
                )


def emit_lineage(bundle: Bundle, chain: dict[str, tuple[dict[str, Any], bytes]]) -> None:
    parent, parent_bytes = chain["parent"]
    child, _child_bytes = chain["child"]
    for topo in LINEAGE:
        env = {
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "head": topo.links[0]["capsule"] if topo.links else None,
            "depth": topo.depth,
            "valid": topo.valid,
            "brokenAt": None
            if topo.valid
            else (topo.links[-1]["capsule"] if topo.links else None),
            "links": list(topo.links),
        }
        if topo.broken_axis == "parentOk":
            env["brokenAt"] = "a.capsule.json"
        if topo.broken_axis == "lineageOk":
            env["brokenAt"] = "b.capsule.json"
        if topo.broken_axis == "reproduced":
            env["brokenAt"] = "b.capsule.json"
        exit_code = classify_lineage(
            has_head_arg=topo.ident not in {"usage"},
            head_readable=topo.ident not in {"head-missing"},
            valid=topo.valid,
            usage_error=topo.ident == "usage",
            io_error=topo.ident == "head-missing",
        )
        doc = {
            "schemaVersion": "1.0",
            "kind": "workReceiptLineageTopology",
            "ident": topo.ident,
            "title": topo.title,
            "why": topo.why,
            "notes": topo.notes,
            "depth": topo.depth,
            "valid": topo.valid,
            "brokenAxis": topo.broken_axis,
            "deep": topo.deep,
            "exit": exit_code,
            "required": list(LINEAGE_REQUIRED),
            "envelope": env,
            "command": (
                ["lineage", env["head"] or "HEAD", "--json"]
                + (["--deep"] if topo.deep else [])
            ),
            "axes": {
                "parentOk": "부모 파일 바이트 == 자식이 기록한 parent.sha256",
                "lineageOk": "부모 receipt.outputSha256 == 자식 receipt.inputSha256",
                "reproduced": "--deep 일 때만 임시 재실행 해시 대조",
            },
            "generator": GENERATOR,
        }
        bundle.lineage.append(doc)
        bundle.put(f"fixtures/lineage/{topo.ident}/topology.json", doc)
        # Materialize a small chain for the common valid/invalid cases.
        if topo.ident in {"root", "two-link", "deep-ok", "parent-tamper", "lineage-break"}:
            bundle.put(
                f"fixtures/lineage/{topo.ident}/a.capsule.json",
                apply_tamper(
                    json.loads(json.dumps(parent, ensure_ascii=False)),
                    "output" if topo.ident == "parent-tamper" else None,
                ),
            )
            if topo.ident != "root":
                src = child if topo.ident != "lineage-break" else chain["broken"][0]
                bundle.put(f"fixtures/lineage/{topo.ident}/b.capsule.json", src)


def emit_hash_vectors(bundle: Bundle) -> None:
    vectors = []
    empty = sha256_hex(b"")
    vectors.append(
        {
            "ident": "empty_bytes",
            "input": "",
            "sha256": empty,
            "why": "빈 바이트 SHA-256 고정값. 64hex 가드 표본.",
        }
    )
    vectors.append(
        {
            "ident": "zero64_is_valid_hex_not_a_hash_of_empty",
            "input": ZERO64,
            "sha256": sha256_hex(ZERO64),
            "why": "ZERO64 는 유효한 64hex 이지만 빈 바이트의 해시가 아니다. "
            "verify 불일치 주장에 쓰는 센티널.",
        }
    )
    for scenario in SCENARIOS:
        text = plan_text_of(make_plan(scenario))
        vectors.append(
            {
                "ident": f"plan:{scenario.ident}",
                "byteLength": len(text.encode("utf-8")),
                "sha256": sha256_hex(text),
                "family": scenario.family,
            }
        )
    # CLI lowercases --expect-output-sha256
    upper = "A" * 64
    vectors.append(
        {
            "ident": "expect_upper_hex",
            "input": upper,
            "normalized": upper.lower(),
            "accepted": True,
            "why": "CLI 는 expect 값을 trim + ascii_lowercase 한다. 대문자 64hex 는 통과.",
        }
    )
    vectors.append(
        {
            "ident": "expect_short",
            "input": "abc",
            "accepted": False,
            "needle": NEEDLE["replay_expect_not_hex"],
            "why": "짧은 값은 usage. 엔진을 타지 않는다.",
        }
    )
    meta = [item for item in vectors if not str(item["ident"]).startswith("plan:")]
    plans = [item for item in vectors if str(item["ident"]).startswith("plan:")]
    bundle.put(
        "fixtures/hash-vectors/index.json",
        {
            "schemaVersion": "1.0",
            "kind": "workReceiptHashVectors",
            "count": len(vectors),
            "meta": meta,
            "planCount": len(plans),
            "generator": GENERATOR,
        },
    )
    bundle.put(
        "fixtures/hash-vectors/plans.tsv",
        tsv([["ident", "family", "byteLength", "sha256"]] + [[p["ident"], p["family"], p["byteLength"], p["sha256"]] for p in plans]),
    )


def emit_argv_catalog(bundle: Bundle) -> None:
    bundle.put(
        "fixtures/argv/catalog.json",
        {
            "schemaVersion": "1.0",
            "kind": "workReceiptArgvCatalog",
            "commands": ["replay", "audit", "lineage"],
            "flags": {
                "replay": [
                    "--json",
                    "--plan-json",
                    "--expect-output-sha256",
                    "--capsule",
                    "--parent",
                    "--sign-key",
                ],
                "audit": ["--json"],
                "lineage": ["--json", "--deep", "--keyring", "--anchor-log"],
            },
            "notInvented": [
                "--recursive",
                "--rate-threshold",
                "--format=jsonl",
                "--attest-only",
            ],
            "exceptionIdents": [spec.ident for spec in EXCEPTIONS],
            "generator": GENERATOR,
        },
    )


def tsv(rows: list[list[Any]]) -> str:
    return "\n".join("\t".join(str(c).replace("\t", " ").replace("\n", " ") for c in row) for row in rows) + "\n"


def emit_reports(bundle: Bundle) -> None:
    replay_rows = [["ident", "family", "genre", "actions", "steps", "title"]]
    for scenario in SCENARIOS:
        replay_rows.append(
            [
                scenario.ident,
                scenario.family,
                scenario.genre,
                "+".join(step["action"] for step in scenario.steps),
                len(scenario.steps),
                scenario.title,
            ]
        )
    bundle.put("reports/replay_cases.tsv", tsv(replay_rows))
    exc_rows = [["ident", "command", "exit", "family", "needle", "why"]]
    for spec in EXCEPTIONS:
        exc_rows.append([spec.ident, spec.command, spec.exit, spec.family, spec.needle, spec.why])
    bundle.put("reports/exceptions.tsv", tsv(exc_rows))
    aud_rows = [["ident", "total", "reproduced", "rate", "exit", "title"]]
    for layout in AUDIT_LAYOUTS:
        rate = "" if layout.total == 0 else f"{layout.reproduced}/{layout.total}"
        aud_rows.append([layout.ident, layout.total, layout.reproduced, rate, layout.exit, layout.title])
    bundle.put("reports/audit_layouts.tsv", tsv(aud_rows))
    lin_rows = [["ident", "depth", "valid", "broken", "deep", "exit", "title"]]
    for topo in LINEAGE:
        lin_rows.append(
            [topo.ident, topo.depth, topo.valid, topo.broken_axis or "", topo.deep, topo.exit, topo.title]
        )
    bundle.put("reports/lineage_topologies.tsv", tsv(lin_rows))

    family_counts = Counter(s.family for s in SCENARIOS)
    action_counts: Counter[str] = Counter()
    for scenario in SCENARIOS:
        for step in scenario.steps:
            action_counts[step["action"]] += 1
    exit_counts = Counter(e.exit for e in EXCEPTIONS)
    summary = {
        "claim": CLAIM_ID,
        "issue": ISSUE,
        "generatedAt": utc_now(),
        "replayCases": len(SCENARIOS),
        "exceptions": len(EXCEPTIONS),
        "auditLayouts": len(AUDIT_LAYOUTS),
        "lineageTopologies": len(LINEAGE),
        "planActions": dict(action_counts),
        "families": dict(family_counts),
        "exceptionExits": {str(k): v for k, v in sorted(exit_counts.items())},
        "files": len(bundle.written),
        "allowedCommands": ["replay", "audit", "lineage"],
        "forbiddenSeats": [
            "canvaskit",
            "serializers",
            "pdf",
            "layout-anomaly",
            "oracle",
            "render_backend",
            "proptest",
            "fidelity",
            "hwp5-inventory",
            "inspect",
            "page-count",
            "gym",
        ],
    }
    bundle.put("reports/fatten_summary.json", summary)
    md = [
        "# M-rcpt 성적표",
        "",
        f"- 이슈: #{ISSUE}",
        f"- replay 케이스: **{len(SCENARIOS)}**",
        f"- 예외 봉투: **{len(EXCEPTIONS)}**",
        f"- 감사 레이아웃: **{len(AUDIT_LAYOUTS)}**",
        f"- 계보 토폴로지: **{len(LINEAGE)}**",
        "",
        "## 문서 가족",
        "",
        "| 가족 | 건수 |",
        "| --- | ---: |",
    ]
    for family, count in sorted(family_counts.items(), key=lambda kv: (-kv[1], kv[0])):
        md.append(f"| {family} | {count} |")
    md.extend(
        [
            "",
            "## plan action",
            "",
            "| action | step 수 |",
            "| --- | ---: |",
        ]
    )
    for action in PLAN_ACTIONS:
        md.append(f"| `{action}` | {action_counts.get(action, 0)} |")
    md.extend(
        [
            "",
            "## 예외 exit",
            "",
            "| exit | 의미 | 건수 |",
            "| ---: | --- | ---: |",
            f"| 1 | IO | {exit_counts.get(1, 0)} |",
            f"| 2 | 사용법 | {exit_counts.get(2, 0)} |",
            f"| 3 | 판정 | {exit_counts.get(3, 0)} |",
            "",
        ]
    )
    bundle.put("reports/fatten_summary.md", "\n".join(md) + "\n")
    bundle.put(
        "reports/kind_counts.md",
        "\n".join(
            [
                "# 픽스처 kind",
                "",
                "| kind | 위치 |",
                "| --- | --- |",
                "| workReceiptReplayCase | fixtures/replay/cases/ |",
                "| workReceiptExceptionEnvelope | fixtures/exceptions/ |",
                "| workCapsule | fixtures/capsules/ |",
                "| workReceiptAuditLayout | fixtures/audit-layouts/ |",
                "| workReceiptLineageTopology | fixtures/lineage/ |",
                "| workReceiptHashVectors | fixtures/hash-vectors/ |",
                "",
            ]
        )
    )


def emit_docs(bundle: Bundle) -> None:
    working = f"""# M-rcpt: 작업 영수증·감사·계보 픽스처 고도화

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/{ISSUE}
브랜치: `feat/m-rcpt-fatten` (`upstream/devel` 기준 격리 worktree)
범위: `tools/work_receipt/` 만
비범위: gym · canvaskit · serializer · pdf · layout-anomaly · oracle ·
render_backend · proptest · fidelity · hwp5-inventory · inspect · page-count

## 무엇을

devel 에 이미 있는 `rhwp replay` / `audit` / `lineage` 의 **기존** 플래그와
봉투 필드를 픽스처로 닫는다. 새 CLI 는 없다.

| 단 | 명령 | 픽스처 | 고정하는 것 |
| --- | --- | --- | --- |
| 영수증 | `replay` | `fixtures/replay/cases/` | attest / verify, 3해시, 사용자 경로 무훼손 |
| 캡슐 | `replay --capsule/--parent` | `fixtures/capsules/` | workCapsule, 상대 부모, 덮어쓰기 거부 |
| 감사 | `audit` | `fixtures/audit-layouts/` | 비재귀, rate=reproduced/total, exit 3 |
| 계보 | `lineage` | `fixtures/lineage/` | parentOk · lineageOk · reproduced · brokenAt |
| 예외 | 세 명령 | `fixtures/exceptions/` | exit 1/2/3 바늘, stdout 0바이트 |

실측: replay **{len(SCENARIOS)}**, 예외 **{len(EXCEPTIONS)}**,
감사 **{len(AUDIT_LAYOUTS)}**, 계보 **{len(LINEAGE)}**.

## 왜

에이전트 노동은 말이 아니라 재실행으로 증명한다. 같은 계획은 같은
바이트를 내고, 그 바이트의 SHA-256 이 영수증이다. 픽스처가 없으면
exit 3 을 도구 고장으로 오독하거나, 빈 폴더(exit 2)와 없는 폴더
(exit 1)를 섞거나, `parent=null` 뿌리와 `parent` 키 없음(fail-closed)을
같은 것으로 본다.

## 어떻게

1. `contracts.py` 가 `src/main.rs` 의 필드·바늘·exit 를 파이썬으로 재현한다.
2. `catalog.py` 가 한국 공공문서 가족(공문·서식·표·계약·고시·시험…)과
   예외·감사·계보 행렬을 가진다. 인덱스만 다른 복제는 케이스가 아니다.
3. `fatten_work_receipt.py` 가 planText UTF-8 의 실제 SHA-256 을 계산해
   디스크에 다시 쓴다.
4. `test_fatten_work_receipt.py` 가 라이브 분류 함수와 픽스처를 대조한다.

## 판정 규약

- 판정은 예외가 아니라 봉투 데이터: `reproduced` · `reproducedRate` ·
  `valid` · `brokenAt`.
- 재현 실패·깨진 체인 = **exit 3**.
- IO = exit 1, 사용법 = exit 2.
- 실패 경로 stdout 은 0바이트. 예외: `replay --json` 엔진 오류 봉투.

## 하지 않은 것

- 새 플래그 / 새 하위명령 없음
- gym pack 없음
- 다른 MEGA 석 파일 없음
- DocumentCore · 렌더 · serializer 없음

## 검증

```bash
python tools/work_receipt/fatten_work_receipt.py
python tools/work_receipt/test_fatten_work_receipt.py
cargo fmt --all -- --check
```
"""
    bundle.put("WORKING.md", working)

    replay_doc = """# replay — 단건 영수증

기존 CLI: `rhwp replay [--plan-json <json> | <계획.json>] [--expect-output-sha256 <hex>]
[--capsule <파일>] [--parent <캡슐>] [--sign-key <키>] [--json]`

## attest

기대를 주지 않으면 `mode=attest`. 임시 산출로 재실행해
`inputSha256` · `planSha256` · `outputSha256` 을 발급한다. 계획의
`output` 경로는 만들어지지 않는다. `reproduced` 와
`expectedOutputSha256` 은 JSON null.

`planSha256` 은 **원문 바이트**의 SHA-256 이다. pretty-print 하거나
키 순서를 바꾸면 해시가 바뀐다.

## verify

`--expect-output-sha256` 이 있으면 `mode=verify`. 값은 trim 후
소문자화하고 64자리 ascii hex 여야 한다. 짧거나 16진이 아니면
**exit 2** (엔진 진입 전).

- 일치: `reproduced=true`, exit 0
- 불일치: `reproduced=false`, exit 3, 주장 해시는
  `expectedOutputSha256` 에 에코

ZERO64 (`0`×64) 는 유효한 hex 이므로 usage 가 아니라 판단 실패다.

## 캡슐

`--capsule` 은 계획(원본 output 보존) + 영수증의 자기완결 교환 파일.
`kind=workCapsule`. `--parent` 는 부모 **파일 바이트** SHA-256 을
저장한다. 상대 경로는 **캡슐 파일 기준**이지 cwd 가 아니다.

같은 파일을 `--capsule` 과 `--parent` 로 주면 거부(부모 덮어쓰기 방지).
`--sign-key` 는 `--capsule` 없이 쓸 수 없다.

## 픽스처

- `fixtures/replay/cases/*.json` — 가족·장르별로 attest/verify 3경로
- `fixtures/capsules/*.capsule.json` — 발급형 + 변조형
- `fixtures/exceptions/replay/` — usage / IO / 판단

결정론: 같은 `planText` 의 두 attest 는 같은 `outputSha256` 을 낸다.
이 전제가 제3자 재현을 가능하게 한다.
"""
    bundle.put("docs/replay-attest.md", replay_doc)

    verify_doc = """# replay verify — 제3자 재현

상대가 준 것은 계획과 산출 해시 두 장이다. 파일을 믿으라는 말이 아니다.

```
rhwp replay --plan-json '<같은 계획>' --expect-output-sha256 <64hex> --json
```

## 읽는 법

| 봉투 | 의미 |
| --- | --- |
| `mode=verify` | 기대를 줬다 |
| `reproduced=true` | 임시 재실행 산출 해시 = 주장 |
| `reproduced=false` | 주장 기각. exit 3 |
| `outputSha256` | 방금 재실행한 실측 |
| `expectedOutputSha256` | 상대 주장 에코 |
| `toolVersion` | 재현 불일치 때 먼저 대조할 힌트 |

`toolVersion` 이 다르다고 audit/lineage 가 자동 실패하지는 않는다.
스킬 pitfalls 가 선대조를 요구하는 이유다. 픽스처
`audit-layouts/toolversion-mismatch` 가 그 축을 고정한다.

## 함정

- 기대 해시 형식 오류는 판단이 아니라 사용법이다.
- 사용자 `output` 경로는 verify 중에도 만들어지지 않는다.
- 계획 파일이 없으면 exit 1 (IO). 깨진 JSON 은 exit 2.
"""
    bundle.put("docs/replay-verify.md", verify_doc)

    capsule_doc = """# 작업 캡슐과 --parent 체인

캡슐은 발급 후 **불변**이다. 에디터·포맷터가 공백 하나를 넣으면
자식이 기록한 `parent.sha256` 과 실물이 갈라지고 `parentOk=false` 가 된다.

## 필드

| 키 | 의미 |
| --- | --- |
| `kind` | 항상 `workCapsule` |
| `parent` | 뿌리이면 JSON null. 키 자체가 없으면 lineage fail-closed |
| `parent.capsule` | 부모 경로. 캡슐 파일 기준 상대 |
| `parent.sha256` | 발급 당시 부모 **파일** SHA-256 |
| `plan` | 파싱된 계획 (원본 output 경로 보존) |
| `planText` | 해시된 원문 |
| `receipt` | replay 봉투 |

`validated_capsule_plan` 은 다음을 이 순서로 본다.

1. `planText` 존재
2. `receipt.planSha256` 64hex
3. `sha256(planText) == receipt.planSha256`
4. `planText` JSON 객체
5. `plan ==` 파싱 결과
6. `receipt.steps` 음이 아닌 정수
7. `plan.steps` 배열 길이 == `receipt.steps`

산출이 같아도 이 가드 중 하나면 audit/lineage 는 실패한다.
`audit-layouts/plan-vs-text` 와 `plan-text-sha` 가 두 변조를 가른다.
"""
    bundle.put("docs/capsule-chain.md", capsule_doc)

    audit_doc = """# audit — 폴더 재현율 회계

```
rhwp audit <캡슐 폴더> [--json]
```

대상은 폴더 **직속** `*.capsule.json` (비재귀, 이름 정렬).
0개면 exit 2 + stdout 0바이트. 없는 폴더는 exit 1.

## 회계

```
reproducedRate = reproduced / total
```

빈 폴더에 0.0 을 주지 않는다 — 그 경로는 usage 다.

`failed[]` 가 하나라도 있으면 exit 3. 회계는 봉투로 읽고, 실패
캡슐만 replay verify 로 개별 추적한다.

## 실패 종류

| 종류 | 픽스처 | 봉투 |
| --- | --- | --- |
| 출력 해시 불일치 | `mixed`, `tamper_output` | `expected`/`actual` |
| 입력 해시 불일치 | `tamper_input` | `kind=inputSha256` |
| steps 길이 | `steps-tamper` | `kind=steps` 또는 error 바늘 |
| plan≠planText | `plan-vs-text` | error 바늘 |
| planText 해시 | `plan-text-sha` | error 바늘 |
| kind | `wrong-kind` | error 바늘 |
| JSON | `invalid-json` | 파싱 실패 |
| 기형 hex | `bad-output-sha` | 가드 |

audit 는 체인을 따라가지 않는다. 같은 폴더의 부모·자식은 각각
재실행된다 (`same-folder-chain`). 연대기는 `lineage` 의 일이다.

확장자 필터는 `ends_with(".capsule.json")`. `.bak` / `.txt` /
중간 `.json` 은 무시 (`mixed-ext`).
"""
    bundle.put("docs/audit-accounting.md", audit_doc)

    lineage_doc = """# lineage — 연대기 무결

```
rhwp lineage <머리캡슐> [--deep] [--keyring <키링>] [--anchor-log <로그>] [--json]
```

머리에서 뿌리로 거슬러 오른다. `parent.capsule` 이 상대 경로면
**현재 캡슐의 디렉터리**에 붙인다.

## 3축

| 축 | 언제 | 참인 조건 |
| --- | --- | --- |
| `parentOk` | 자식이 부모를 지목할 때 | 기록 해시 == 부모 파일 바이트 |
| `lineageOk` | 같은 순간 | 부모 `outputSha256` == 자식 `inputSha256` |
| `reproduced` | `--deep` | 임시 재실행 해시·step 수·입력 해시 일치 |

뿌리 링크는 세 축이 모두 null 이다. 그래도 `valid=true`, `depth=1`.

하나라도 false 면 `brokenAt` 이 그 캡슐을 가리키고 exit 3.
머리 파일 없음은 exit 1 (IO). 중간 부모 없음은 체인 깨짐(exit 3).

## fail-closed

다음을 root 로 오인하지 않는다.

- `parent` 키 없음
- `parent.sha256` 없음 / 비 hex
- `parent.capsule` 없음
- `planSha256` 없음
- `plan` ≠ `planText`
- `receipt.steps` ≠ `plan.steps.len`

`--keyring` 이 없으면 `signerOk` 축 자체가 없다.
`--anchor-log` 의 `anchoredOk=false` 는 체인을 깨지 않는다
(등재 강제는 게이트의 일).

순환은 가드 1000. 픽스처는 바늘만 고정하고 1000링크를 만들지 않는다.
"""
    bundle.put("docs/lineage-chronicle.md", lineage_doc)

    exit_doc = """# exit 코드

세 명령이 같은 가족을 쓴다 (#2707).

| 코드 | 이름 | 언제 | stdout |
| ---: | --- | --- | --- |
| 0 | OK | attest 성공, verify 일치, audit 전건, lineage valid | 봉투 |
| 1 | IO | 파일/폴더를 읽을 수 없음. 머리 캡슐 없음 | 0바이트 (replay --json 엔진 오류만 봉투) |
| 2 | 사용법 | 인자·플래그·형식·빈 감사 폴더 | 0바이트 |
| 3 | 판정 | verify 불일치, audit failed[], lineage invalid | 봉투 |

exit 3 을 재시도하거나 도구 버그로 올리지 않는다. 봉투의
`reproduced` / `failed` / `brokenAt` 을 읽는다.

`fixtures/exceptions/` 가 명령×코드 행렬을 닫는다.
"""
    bundle.put("docs/exit-codes.md", exit_doc)

    pitfalls = """# 함정 실록

1. **캡슐을 포맷터로 저장하지 말라.** 부모 해시가 깨지는 것이 의도된
   변조 검출이다. 고치려면 재발급한다.
2. **`--parent` 상대 경로는 cwd 가 아니다.** 같은 폴더에 두는 것이
   가장 단순하다. `relative-parent` 토폴로지가 그 해석을 고정한다.
3. **같은 파일 capsule=parent 거부.** 부모를 자식으로 덮어쓰지 않는다.
4. **빈 감사 폴더 ≠ 재현율 0.** usage exit 2.
5. **없는 폴더 ≠ 빈 폴더.** IO exit 1.
6. **`parent: null` ≠ parent 키 없음.** 후자는 fail-closed.
7. **`toolVersion` 은 힌트다.** audit 가 버전을 이유로 실패하지 않는다.
   불일치가 나면 사람/에이전트가 먼저 버전을 대조한다.
8. **영수증은 귀속을 증명하지 않는다.** 누가 했는지는 서명 축(#4511).
   `--keyring` 없는 lineage 에 `signerOk` 를 기대해서는 안 된다.
9. **audit 는 체인을 따라가지 않는다.** 연대기는 lineage.
10. **find 빈 문자열은 계획 스키마가 거부한다.** delete 는 replace=""
    이지 find="" 가 아니다 (`delete_draft`).
11. **사용자 output 경로는 replay 가 만들지 않는다.** 실산출은 `rhwp run`.
12. **새 플래그를 만들지 말라.** 이 픽스처 폴더의 일은 기존 CLI 를
    설명하는 것이다.
"""
    bundle.put("docs/pitfalls.md", pitfalls)

    # Per-family walkthroughs — each points at real fixtures.
    by_family: dict[str, list[Scenario]] = {}
    for scenario in SCENARIOS:
        by_family.setdefault(scenario.family, []).append(scenario)
    for family, items in sorted(by_family.items()):
        lines = [
            f"# {family} 워크스루",
            "",
            f"replay 케이스 {len(items)} 건. 각 파일은 attest / verify 일치 / "
            f"verify 불일치 세 경로를 닫는다. 상세 why 는 케이스 JSON 과 "
            f"`catalog.py` 가 정본이다.",
            "",
            "| ident | 장르 | steps | 제목 | why |",
            "| --- | --- | ---: | --- | --- |",
        ]
        for item in items:
            lines.append(
                f"| `{item.ident}` | {item.genre} | {len(item.steps)} | {item.title} | {item.why} |"
            )
        lines.append("")
        bundle.put(f"docs/families/{family}.md", "\n".join(lines) + "\n")


def emit_index(bundle: Bundle) -> None:
    index = {
        "schemaVersion": "1.0",
        "kind": KIND_CATALOG,
        "claim": CLAIM_ID,
        "issue": ISSUE,
        "generator": GENERATOR,
        "generatedAt": utc_now(),
        "replay": [s.ident for s in SCENARIOS],
        "exceptions": [e.ident for e in EXCEPTIONS],
        "auditLayouts": [a.ident for a in AUDIT_LAYOUTS],
        "lineage": [t.ident for t in LINEAGE],
        "fileCount": len(bundle.written),
    }
    bundle.put("fixtures/index.json", index)
    readme = """# tools/work_receipt

MEGA QUEUE M-rcpt (#5478). 기존 `replay` / `audit` / `lineage` CLI 의
픽스처·예외 봉투·작업 문서다. 새 명령은 없다.

```
python tools/work_receipt/fatten_work_receipt.py
python tools/work_receipt/test_fatten_work_receipt.py
```

- `contracts.py` — exit·바늘·검증 함수 (main.rs 대응)
- `catalog.py` — 한국 공공문서 시나리오·예외·레이아웃·토폴로지
- `fixtures/` — 생성된 정본
- `docs/` · `WORKING.md` — 사람용
- `schema/` — JSON 스키마
"""
    bundle.put("README.md", readme)


def build(out_root: Path) -> Bundle:
    bundle = Bundle(out_root)
    emit_schemas(bundle)
    base = emit_replay(bundle)
    tampers = emit_tamper_capsules(bundle, base)
    chain = emit_chain_capsules(bundle, base)
    emit_exceptions(bundle)
    emit_audit(bundle, base, tampers)
    emit_lineage(bundle, chain)
    emit_hash_vectors(bundle)
    emit_argv_catalog(bundle)
    emit_docs(bundle)
    emit_reports(bundle)
    emit_index(bundle)
    return bundle


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="M-rcpt fixture fatten")
    parser.add_argument("--out", type=Path, default=HERE)
    args = parser.parse_args(argv)
    bundle = build(args.out)
    print(f"wrote {len(bundle.written)} files under {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
