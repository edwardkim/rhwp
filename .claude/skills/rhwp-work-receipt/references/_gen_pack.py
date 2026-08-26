#!/usr/bin/env python3
"""Generate rhwp-work-receipt fixtures, layouts, transcripts, and examples.

Deterministic. No rhwp binary. No gym. Existing CLI only: replay / audit / lineage
(and run only where the skill must emit a real on-disk output before a child capsule).

Hashes are SHA-256 of documented fixture bytes, not live HWP. Tests lock the
relationships (planText↔planSha256, parent file↔parent.sha256, parent output
== child input, reproducedRate = reproduced/total, non-recursive *.capsule.json).
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIX = ROOT / "fixtures"
EXAMPLES = ROOT / "examples"
SCHEMA = "1.0"
TOOL = "0.8.4"
SAMPLE = "samples/basic/issue2007_nested_cell_pagination_42065.hwp"
ZERO64 = "0" * 64


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def dump(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(obj, ensure_ascii=False, indent=2) + "\n"
    path.write_text(text, encoding="utf-8", newline="\n")


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text, encoding="utf-8", newline="\n")


def compact(obj) -> str:
    return json.dumps(obj, ensure_ascii=False, separators=(",", ":"))


def input_bytes(label: str) -> bytes:
    return f"RHWP-WORK-RECEIPT-INPUT\n{label}\n".encode("utf-8")


def output_bytes(label: str, mutation: str) -> bytes:
    return f"RHWP-WORK-RECEIPT-OUTPUT\n{label}\n{mutation}\n".encode("utf-8")


def make_plan(inp: str, out: str, find: str, replace: str, extra_steps=None):
    steps = [{"action": "replace_text", "find": find, "replace": replace}]
    if extra_steps:
        steps.extend(extra_steps)
    return {
        "planVersion": "1.0",
        "input": inp,
        "output": out,
        "steps": steps,
    }


def receipt(
    *,
    mode: str,
    inp: str,
    in_sha: str,
    plan_sha: str,
    out_sha: str,
    steps: int,
    reproduced,
    expected,
    tool: str = TOOL,
):
    return {
        "schemaVersion": SCHEMA,
        "mode": mode,
        "input": inp,
        "inputSha256": in_sha,
        "planSha256": plan_sha,
        "outputSha256": out_sha,
        "toolVersion": tool,
        "steps": steps,
        "reproduced": reproduced,
        "expectedOutputSha256": expected,
        "untrustedContent": False,
        "untrustedFields": [],
    }


def capsule(plan, plan_text: str, rec, parent=None):
    return {
        "schemaVersion": SCHEMA,
        "kind": "workCapsule",
        "parent": parent,
        "plan": plan,
        "planText": plan_text,
        "receipt": rec,
    }


def envelope_meta(exit_code: int, branch: str, command: str, note: str):
    return {
        "_skillMeta": {
            "exit": exit_code,
            "branch": branch,
            "command": command,
            "note": note,
            "stdoutSilentOnFail": exit_code in (1, 2) and command != "run",
        }
    }


# ---------------------------------------------------------------------------
# Work items used across capsules / recipes / scenarios
# ---------------------------------------------------------------------------

WORK = [
    {
        "id": "notice_year",
        "title": "공문 연도 치환",
        "find": "2025년",
        "replace": "2026년",
        "why": "연도만 바꾸고 3해시 영수증을 넘긴다",
    },
    {
        "id": "org_rename",
        "title": "기관명 치환",
        "find": "한국수자원공사",
        "replace": "한강홍수통제소",
        "why": "기관 개편 후 직인 전 문구만 고친다",
    },
    {
        "id": "form_fill_echo",
        "title": "누름틀 멱등 치환",
        "find": "신청인",
        "replace": "신청인",
        "why": "바이트 중립 계획으로 입력=산출 해시를 가르친다",
    },
    {
        "id": "deadline",
        "title": "마감일 정정",
        "find": "3월 31일",
        "replace": "4월 15일",
        "why": "공고 정정 한 줄",
    },
    {
        "id": "phone",
        "title": "연락처 갱신",
        "find": "02-123-4567",
        "replace": "02-987-6543",
        "why": "담당 부서 번호 교체",
    },
    {
        "id": "addr",
        "title": "주소 치환",
        "find": "세종특별자치시",
        "replace": "대전광역시",
        "why": "이전 공문",
    },
    {
        "id": "law_ref",
        "title": "조문 호수 정정",
        "find": "제12조",
        "replace": "제13조",
        "why": "인용 조문 오기",
    },
    {
        "id": "amount",
        "title": "금액 단위 정정",
        "find": "1,000,000원",
        "replace": "10,000,000원",
        "why": "자릿수 오기 — 영수증이 바이트를 고정",
    },
    {
        "id": "checkbox_mark",
        "title": "체크 표시 치환",
        "find": "□ 동의",
        "replace": "☑ 동의",
        "why": "체크박스는 글리프 치환이다",
    },
    {
        "id": "privacy_mask",
        "title": "주민번호 자리 마스킹",
        "find": "900101-1",
        "replace": "******-*",
        "why": "제출 전 자리만 가린다. redact 가 아니라 replace_text",
    },
    {
        "id": "title_fix",
        "title": "제목 오기 정정",
        "find": "보조금 신청셔",
        "replace": "보조금 신청서",
        "why": "오탈자 1건",
    },
    {
        "id": "date_iso",
        "title": "날짜 표기 통일",
        "find": "2026. 8. 18.",
        "replace": "2026-08-18",
        "why": "내부 양식 ISO 통일",
    },
    {
        "id": "dept",
        "title": "부서명 개편",
        "find": "총무과",
        "replace": "운영지원과",
        "why": "직제 개편",
    },
    {
        "id": "seal_caption",
        "title": "직인 안내문",
        "find": "(인)",
        "replace": "(직인)",
        "why": "안내문만 고친다. 그림 삽입 아님",
    },
    {
        "id": "page_ref",
        "title": "쪽 지시 정정",
        "find": "3쪽 참조",
        "replace": "4쪽 참조",
        "why": "본문 쪽수 재매김",
    },
    {
        "id": "en_typo",
        "title": "영문 오탈자",
        "find": "reciept",
        "replace": "receipt",
        "why": "영문 병기 정정",
    },
    {
        "id": "hwpx_year",
        "title": "HWPX 연도",
        "find": "2024",
        "replace": "2026",
        "why": "형식 보존: 입력 hwpx → 산출 hwpx",
        "ext": "hwpx",
    },
    {
        "id": "multi_step",
        "title": "연도+기관 두 스텝",
        "find": "2025",
        "replace": "2026",
        "extra": [{"action": "replace_text", "find": "구기관", "replace": "신기관"}],
        "why": "한 계획에 step 2 — receipt.steps 는 2",
    },
    {
        "id": "idempotent",
        "title": "이미 반영된 문구",
        "find": "2026년",
        "replace": "2026년",
        "why": "멱등 재실행. 산출 해시는 입력과 다를 수 있다(엔진이 다시 쓴다)",
    },
    {
        "id": "ws_only",
        "title": "공백 정리",
        "find": "  ",
        "replace": " ",
        "why": "이중 공백 축소",
    },
    {
        "id": "newline_keep",
        "title": "개행 유지 치환",
        "find": "붙임: 1부",
        "replace": "붙임: 2부",
        "why": "줄바꿈 없는 짧은 토큰",
    },
    {
        "id": "table_label",
        "title": "표 제목 치환",
        "find": "산출내역",
        "replace": "산출 명세",
        "why": "표 안 글자도 replace_text 범위",
    },
    {
        "id": "footer",
        "title": "바닥글 문구",
        "find": "대외비",
        "replace": "공개",
        "why": "배포 등급 변경",
    },
    {
        "id": "version_tag",
        "title": "문서 버전 표기",
        "find": "v0.7",
        "replace": "v0.8",
        "why": "머리글 버전 태그",
    },
]


def build_item_capsule(item, *, parent=None, mode="attest", expected=None, tool=TOOL):
    ext = item.get("ext", "hwp")
    inp = SAMPLE if ext == "hwp" else SAMPLE.replace(".hwp", ".hwpx")
    out = f"out/{item['id']}.{ext}"
    extra = item.get("extra")
    plan = make_plan(inp, out, item["find"], item["replace"], extra)
    plan_text = compact(plan)
    plan_sha = sha256_hex(plan_text.encode("utf-8"))
    in_sha = sha256_hex(input_bytes(item["id"]))
    out_sha = sha256_hex(output_bytes(item["id"], f"{item['find']}->{item['replace']}"))
    rec = receipt(
        mode=mode,
        inp=inp,
        in_sha=in_sha,
        plan_sha=plan_sha,
        out_sha=out_sha,
        steps=len(plan["steps"]),
        reproduced=None if expected is None else (expected == out_sha),
        expected=expected,
        tool=tool,
    )
    return capsule(plan, plan_text, rec, parent), {
        "id": item["id"],
        "inputSha256": in_sha,
        "planSha256": plan_sha,
        "outputSha256": out_sha,
        "planText": plan_text,
        "plan": plan,
        "input": inp,
        "output": out,
        "steps": len(plan["steps"]),
    }


def write_capsule_file(path: Path, cap) -> str:
    dump(path, cap)
    return sha256_hex(path.read_bytes())


def generate_capsules():
    index = []
    meta = {}
    cap_dir = FIX / "capsules"
    for item in WORK:
        cap, info = build_item_capsule(item)
        name = f"{item['id']}.capsule.json"
        path = cap_dir / name
        file_sha = write_capsule_file(path, cap)
        info["file"] = name
        info["fileSha256"] = file_sha
        info["parent"] = None
        meta[item["id"]] = info
        index.append(
            {
                "file": name,
                "id": item["id"],
                "title": item["title"],
                "why": item["why"],
                "inputSha256": info["inputSha256"],
                "planSha256": info["planSha256"],
                "outputSha256": info["outputSha256"],
                "fileSha256": file_sha,
                "steps": info["steps"],
                "parent": None,
            }
        )

    # Child chain: child input bytes labeled as parent output so lineageOk can
    # be asserted from fixture fields (parent.outputSha256 == child.inputSha256).
    chain_pairs = [
        ("notice_year", "deadline", "chain_deadline"),
        ("org_rename", "dept", "chain_dept"),
        ("title_fix", "footer", "chain_footer"),
        ("date_iso", "version_tag", "chain_version"),
        ("law_ref", "page_ref", "chain_page"),
        ("phone", "addr", "chain_addr"),
        ("amount", "table_label", "chain_table"),
        ("hwpx_year", "en_typo", "chain_en"),
    ]
    children = []
    for parent_id, child_base, child_id in chain_pairs:
        parent_info = meta[parent_id]
        parent_name = parent_info["file"]
        parent_sha = parent_info["fileSha256"]
        base = next(x for x in WORK if x["id"] == child_base)
        child_item = dict(base)
        child_item["id"] = child_id
        child_item["title"] = f"{base['title']} (체인 {parent_id}→)"
        cap, info = build_item_capsule(child_item)
        # Force lineage invariant: child input hash == parent output hash.
        cap["receipt"]["inputSha256"] = parent_info["outputSha256"]
        cap["receipt"]["input"] = parent_info["output"]
        cap["plan"]["input"] = parent_info["output"]
        cap["planText"] = compact(cap["plan"])
        cap["receipt"]["planSha256"] = sha256_hex(cap["planText"].encode("utf-8"))
        cap["parent"] = {"capsule": parent_name, "sha256": parent_sha}
        name = f"{child_id}.capsule.json"
        path = cap_dir / name
        file_sha = write_capsule_file(path, cap)
        info["file"] = name
        info["fileSha256"] = file_sha
        info["inputSha256"] = cap["receipt"]["inputSha256"]
        info["planSha256"] = cap["receipt"]["planSha256"]
        info["parent"] = parent_name
        meta[child_id] = info
        children.append(
            {
                "file": name,
                "id": child_id,
                "parent": parent_name,
                "parentSha256": parent_sha,
                "inputSha256": info["inputSha256"],
                "outputSha256": info["outputSha256"],
                "fileSha256": file_sha,
                "lineageOk": info["inputSha256"] == parent_info["outputSha256"],
                "parentPathRelativeToCapsuleFile": True,
            }
        )
        index.append(
            {
                "file": name,
                "id": child_id,
                "title": child_item["title"],
                "why": f"부모 {parent_id} 산출을 입력으로 잇는다",
                "inputSha256": info["inputSha256"],
                "planSha256": info["planSha256"],
                "outputSha256": info["outputSha256"],
                "fileSha256": file_sha,
                "steps": info["steps"],
                "parent": parent_name,
            }
        )

    # Tamper variants (still valid JSON capsules; audit/lineage should fail).
    tampers = []
    src = json.loads((cap_dir / "notice_year.capsule.json").read_text(encoding="utf-8"))

    out_flip = json.loads(json.dumps(src, ensure_ascii=False))
    out_flip["receipt"]["outputSha256"] = ZERO64
    dump(cap_dir / "tamper_output_sha.capsule.json", out_flip)
    tampers.append(
        {
            "file": "tamper_output_sha.capsule.json",
            "kind": "outputSha256",
            "expectAudit": "failed",
            "note": "영수증 산출 해시를 0으로. 재실행 해시와 불일치",
        }
    )

    in_flip = json.loads(json.dumps(src, ensure_ascii=False))
    in_flip["receipt"]["inputSha256"] = ZERO64
    dump(cap_dir / "tamper_input_sha.capsule.json", in_flip)
    tampers.append(
        {
            "file": "tamper_input_sha.capsule.json",
            "kind": "inputSha256",
            "expectAudit": "failed",
            "note": "입력 영수증 변조. 산출 대조 전에 잡힌다",
        }
    )

    plan_drift = json.loads(json.dumps(src, ensure_ascii=False))
    plan_drift["plan"]["output"] = "out/neutral-tamper.hwp"
    dump(cap_dir / "tamper_plan_vs_text.capsule.json", plan_drift)
    tampers.append(
        {
            "file": "tamper_plan_vs_text.capsule.json",
            "kind": "planText",
            "expectAudit": "failed",
            "note": "plan 객체만 바꿈. plan 과 planText 불일치",
        }
    )

    steps_flip = json.loads(json.dumps(src, ensure_ascii=False))
    steps_flip["receipt"]["steps"] = 99
    dump(cap_dir / "tamper_steps.capsule.json", steps_flip)
    tampers.append(
        {
            "file": "tamper_steps.capsule.json",
            "kind": "steps",
            "expectAudit": "failed",
            "note": "receipt.steps 와 plan.steps 길이 불일치",
        }
    )

    pretty = json.loads(json.dumps(src, ensure_ascii=False))
    pretty["_editorComment"] = "포맷터로 저장하면 부모 해시가 깨진다"
    dump(cap_dir / "tamper_pretty_print.capsule.json", pretty)
    tampers.append(
        {
            "file": "tamper_pretty_print.capsule.json",
            "kind": "immutability",
            "expectAudit": "may-still-reproduce",
            "note": "필드 추가는 재현과 별개로 파일 바이트를 바꾼다. 자식 parent.sha256 이 깨진다",
        }
    )

    tool_mis = json.loads(json.dumps(src, ensure_ascii=False))
    tool_mis["receipt"]["toolVersion"] = "0.1.0-old"
    dump(cap_dir / "toolversion_mismatch.capsule.json", tool_mis)
    tampers.append(
        {
            "file": "toolversion_mismatch.capsule.json",
            "kind": "toolVersion",
            "expectAudit": "live-binary-may-differ",
            "note": "픽스처 버전과 실행 바이너리 버전이 다르면 산출 해시가 갈릴 수 있다",
        }
    )

    dump(
        FIX / "capsule_index.json",
        {
            "catalogVersion": "1.0",
            "skill": "rhwp-work-receipt",
            "issue": 5308,
            "hashAlg": "SHA-256",
            "note": "fileSha256 는 이 생성기가 쓴 파일 바이트. 에디터로 다시 저장하면 바뀐다.",
            "roots": [x for x in index if x["parent"] is None],
            "children": children,
            "tampers": tampers,
        },
    )
    return meta, children, tampers, index


def generate_plans():
    valid = {
        "valid_replace_text": make_plan(SAMPLE, "out/a.hwp", "2025년", "2026년"),
        "valid_fill_fields": {
            "planVersion": "1.0",
            "input": "samples/field-01.hwp",
            "output": "out/filled.hwp",
            "steps": [{"action": "fill_fields", "data": {"회사명": "페타플로"}}],
        },
        "valid_set_cell": {
            "planVersion": "1.0",
            "input": "samples/table-001.hwp",
            "output": "out/cell.hwp",
            "steps": [
                {
                    "action": "set_cell",
                    "table": 0,
                    "row": 1,
                    "col": 1,
                    "text": "1,234",
                }
            ],
        },
        "valid_set_checkbox": {
            "planVersion": "1.0",
            "input": SAMPLE,
            "output": "out/box.hwp",
            "steps": [{"action": "set_checkbox", "occurrence": 0}],
        },
        "valid_multi_step": {
            "planVersion": "1.0",
            "input": SAMPLE,
            "output": "out/multi.hwp",
            "steps": [
                {"action": "replace_text", "find": "2025", "replace": "2026"},
                {"action": "replace_text", "find": "구기관", "replace": "신기관"},
            ],
        },
        "valid_preconditions": {
            "planVersion": "1.0",
            "input": SAMPLE,
            "output": "out/cas.hwp",
            "preconditions": {"inputSha256": sha256_hex(input_bytes("cas"))},
            "steps": [{"action": "replace_text", "find": "A", "replace": "B"}],
        },
        "valid_hwpx": make_plan(
            "samples/field-01.hwpx", "out/a.hwpx", "2024", "2026"
        ),
    }
    invalid = {
        "invalid_missing_input": {
            "planVersion": "1.0",
            "output": "out/x.hwp",
            "steps": [{"action": "replace_text", "find": "A", "replace": "B"}],
        },
        "invalid_empty_steps": {
            "planVersion": "1.0",
            "input": SAMPLE,
            "output": "out/x.hwp",
            "steps": [],
        },
        "invalid_missing_plan_version": {
            "input": SAMPLE,
            "output": "out/x.hwp",
            "steps": [{"action": "replace_text", "find": "A", "replace": "B"}],
        },
        "invalid_unknown_action": {
            "planVersion": "1.0",
            "input": SAMPLE,
            "output": "out/x.hwp",
            "steps": [{"action": "insert_image", "image": "x.png"}],
        },
        "invalid_camel_action": {
            "planVersion": "1.0",
            "input": SAMPLE,
            "output": "out/x.hwp",
            "steps": [{"action": "replaceText", "find": "A", "replace": "B"}],
        },
        "invalid_wrong_keys_source_op": {
            "planVersion": "1.0",
            "source": SAMPLE,
            "op": "replace",
            "steps": [{"op": "replace_text"}],
        },
        "invalid_numeric_plan_version": {
            "planVersion": 1.0,
            "input": SAMPLE,
            "output": "out/x.hwp",
            "steps": [{"action": "replace_text", "find": "A", "replace": "B"}],
        },
    }
    names_valid = []
    names_invalid = []
    for name, plan in valid.items():
        dump(FIX / "plans" / f"{name}.json", plan)
        names_valid.append(f"{name}.json")
    for name, plan in invalid.items():
        dump(FIX / "plans" / f"{name}.json", plan)
        names_invalid.append(f"{name}.json")
    return names_valid, names_invalid


def generate_envelopes(meta):
    names = []

    def put(name, body, exit_code, branch, command, note):
        obj = dict(body)
        obj.update(envelope_meta(exit_code, branch, command, note))
        dump(FIX / "envelopes" / name, obj)
        names.append(name)

    n = meta["notice_year"]
    put(
        "replay_attest.json",
        {
            "schemaVersion": SCHEMA,
            "mode": "attest",
            "input": n["input"],
            "inputSha256": n["inputSha256"],
            "planSha256": n["planSha256"],
            "outputSha256": n["outputSha256"],
            "toolVersion": TOOL,
            "steps": n["steps"],
            "reproduced": None,
            "expectedOutputSha256": None,
            "untrustedContent": False,
            "untrustedFields": [],
        },
        0,
        "attest",
        "replay",
        "3해시 발급. 사용자 output 경로는 만들지 않는다",
    )
    put(
        "replay_verify_match.json",
        {
            "schemaVersion": SCHEMA,
            "mode": "verify",
            "input": n["input"],
            "inputSha256": n["inputSha256"],
            "planSha256": n["planSha256"],
            "outputSha256": n["outputSha256"],
            "toolVersion": TOOL,
            "steps": n["steps"],
            "reproduced": True,
            "expectedOutputSha256": n["outputSha256"],
            "untrustedContent": False,
            "untrustedFields": [],
        },
        0,
        "verify-match",
        "replay",
        "--expect-output-sha256 가 재실행 산출과 같다",
    )
    put(
        "replay_verify_mismatch.json",
        {
            "schemaVersion": SCHEMA,
            "mode": "verify",
            "input": n["input"],
            "inputSha256": n["inputSha256"],
            "planSha256": n["planSha256"],
            "outputSha256": n["outputSha256"],
            "toolVersion": TOOL,
            "steps": n["steps"],
            "reproduced": False,
            "expectedOutputSha256": ZERO64,
            "untrustedContent": False,
            "untrustedFields": [],
        },
        3,
        "verify-mismatch",
        "replay",
        "주장 기각. 도구 고장이 아니라 판정 데이터",
    )
    put(
        "replay_usage.json",
        {
            "stderr": "사용법: rhwp replay <계획.json> [--plan-json <json>] [--expect-output-sha256 <hex>] [--json]"
        },
        2,
        "usage",
        "replay",
        "계획 없음. stdout 0바이트",
    )
    put(
        "replay_io.json",
        {"stderr": "오류: 계획을 읽을 수 없습니다 - missing-plan.json: ..."},
        1,
        "io",
        "replay",
        "계획 파일 IO. stdout 0바이트",
    )
    put(
        "replay_expect_not_hex.json",
        {"stderr": "오류: --expect-output-sha256 값은 64자리 16진이어야 합니다: zz"},
        2,
        "usage-hex",
        "replay",
        "64hex 계약. 짧은 값·비hex 는 사용법",
    )
    put(
        "replay_parent_same_file.json",
        {
            "stderr": "오류: --capsule과 --parent가 같은 기존 파일을 가리킵니다 — 부모 캡슐을 덮어쓰지 않습니다."
        },
        2,
        "usage-same-file",
        "replay",
        "부모 덮어쓰기 방지",
    )
    put(
        "replay_parent_missing.json",
        {"stderr": "오류: 부모 캡슐을 읽을 수 없습니다 - ghost.capsule.json: ..."},
        1,
        "io-parent",
        "replay",
        "부모 경로 IO. exit 1",
    )
    put(
        "audit_all_ok.json",
        {
            "schemaVersion": SCHEMA,
            "root": "fixtures/audit-layouts/all-ok",
            "total": 3,
            "reproduced": 3,
            "failed": [],
            "reproducedRate": 1.0,
            "untrustedContent": False,
            "untrustedFields": [],
        },
        0,
        "audit-all-ok",
        "audit",
        "직속 *.capsule.json 3건 전부 재현",
    )
    put(
        "audit_mixed.json",
        {
            "schemaVersion": SCHEMA,
            "root": "fixtures/audit-layouts/mixed",
            "total": 3,
            "reproduced": 2,
            "failed": [
                {
                    "capsule": "tampered.capsule.json",
                    "expected": n["outputSha256"],
                    "actual": ZERO64
                    if False
                    else sha256_hex(output_bytes("tampered", "x")),
                }
            ],
            "reproducedRate": 2 / 3,
            "untrustedContent": False,
            "untrustedFields": [],
        },
        3,
        "audit-mixed",
        "audit",
        "실패 1건 = exit 3. 회계는 봉투. reproducedRate=2/3",
    )
    put(
        "audit_empty.json",
        {"stderr": "오류: fixtures/audit-layouts/empty 에 *.capsule.json 이 없습니다 — 감사 대상 없음."},
        2,
        "audit-empty",
        "audit",
        "0개면 봉투 없이 exit 2",
    )
    put(
        "audit_nested_ignored.json",
        {
            "schemaVersion": SCHEMA,
            "root": "fixtures/audit-layouts/nested-ignored",
            "total": 1,
            "reproduced": 1,
            "failed": [],
            "reproducedRate": 1.0,
            "ignoredNested": ["nested/hidden.capsule.json"],
            "untrustedContent": False,
            "untrustedFields": [],
        },
        0,
        "audit-non-recursive",
        "audit",
        "하위 폴더 *.capsule.json 은 세지 않는다",
    )
    put(
        "audit_input_tamper.json",
        {
            "schemaVersion": SCHEMA,
            "root": "fixtures/audit-layouts/input-tamper",
            "total": 1,
            "reproduced": 0,
            "failed": [
                {
                    "capsule": "input.capsule.json",
                    "kind": "inputSha256",
                    "expected": ZERO64,
                    "actual": n["inputSha256"],
                }
            ],
            "reproducedRate": 0.0,
            "untrustedContent": False,
            "untrustedFields": [],
        },
        3,
        "audit-input-tamper",
        "audit",
        "kind=inputSha256. 산출 크레딧 전에 실패",
    )
    put(
        "lineage_root.json",
        {
            "schemaVersion": SCHEMA,
            "head": "notice_year.capsule.json",
            "depth": 1,
            "valid": True,
            "brokenAt": None,
            "links": [
                {
                    "capsule": "notice_year.capsule.json",
                    "inputSha256": n["inputSha256"],
                    "outputSha256": n["outputSha256"],
                    "parentOk": None,
                    "lineageOk": None,
                    "reproduced": None,
                }
            ],
            "untrustedContent": False,
            "untrustedFields": [],
        },
        0,
        "lineage-root",
        "lineage",
        "뿌리 하나. 3축은 전부 null",
    )
    child = meta["chain_deadline"]
    put(
        "lineage_two_link.json",
        {
            "schemaVersion": SCHEMA,
            "head": "chain_deadline.capsule.json",
            "depth": 2,
            "valid": True,
            "brokenAt": None,
            "links": [
                {
                    "capsule": "chain_deadline.capsule.json",
                    "inputSha256": child["inputSha256"],
                    "outputSha256": child["outputSha256"],
                    "parentOk": True,
                    "lineageOk": True,
                    "reproduced": None,
                },
                {
                    "capsule": "notice_year.capsule.json",
                    "inputSha256": n["inputSha256"],
                    "outputSha256": n["outputSha256"],
                    "parentOk": None,
                    "lineageOk": None,
                    "reproduced": None,
                },
            ],
            "untrustedContent": False,
            "untrustedFields": [],
        },
        0,
        "lineage-two-ok",
        "lineage",
        "parentOk 와 lineageOk 가 참. --deep 없으면 reproduced=null",
    )
    put(
        "lineage_deep.json",
        {
            "schemaVersion": SCHEMA,
            "head": "chain_deadline.capsule.json",
            "depth": 2,
            "valid": True,
            "brokenAt": None,
            "links": [
                {
                    "capsule": "chain_deadline.capsule.json",
                    "parentOk": True,
                    "lineageOk": True,
                    "reproduced": True,
                },
                {
                    "capsule": "notice_year.capsule.json",
                    "parentOk": None,
                    "lineageOk": None,
                    "reproduced": True,
                },
            ],
            "untrustedContent": False,
            "untrustedFields": [],
        },
        0,
        "lineage-deep",
        "lineage",
        "--deep 은 링크마다 재실행. 비용 = 링크 수",
    )
    put(
        "lineage_parent_tamper.json",
        {
            "schemaVersion": SCHEMA,
            "head": "chain_deadline.capsule.json",
            "depth": 2,
            "valid": False,
            "brokenAt": "notice_year.capsule.json",
            "links": [
                {
                    "capsule": "chain_deadline.capsule.json",
                    "parentOk": True,
                    "lineageOk": True,
                    "reproduced": None,
                },
                {
                    "capsule": "notice_year.capsule.json",
                    "parentOk": False,
                    "lineageOk": True,
                    "reproduced": None,
                },
            ],
            "untrustedContent": False,
            "untrustedFields": [],
        },
        3,
        "lineage-parentOk-false",
        "lineage",
        "부모 파일을 포맷터로 저장. parentOk=false, brokenAt 명세",
    )
    put(
        "lineage_broken_invariant.json",
        {
            "schemaVersion": SCHEMA,
            "head": "broken_child.capsule.json",
            "depth": 1,
            "valid": False,
            "brokenAt": "broken_child.capsule.json",
            "links": [
                {
                    "capsule": "broken_child.capsule.json",
                    "parentOk": True,
                    "lineageOk": False,
                    "reproduced": None,
                }
            ],
            "untrustedContent": False,
            "untrustedFields": [],
        },
        3,
        "lineage-lineageOk-false",
        "lineage",
        "부모 산출 해시 != 자식 입력 해시. 연대기가 아니다",
    )
    put(
        "lineage_missing_head.json",
        {"stderr": "오류: 캡슐을 읽을 수 없습니다 - missing.capsule.json: ..."},
        1,
        "lineage-io",
        "lineage",
        "머리 캡슐 없음 = exit 1 IO",
    )
    put(
        "lineage_usage.json",
        {
            "stderr": "사용법: rhwp lineage <캡슐.json> [--deep] [--keyring <키링.json>] [--anchor-log <로그>] [--json]"
        },
        2,
        "lineage-usage",
        "lineage",
        "무인자. stdout 0바이트",
    )
    put(
        "lineage_missing_parent_sha.json",
        {
            "schemaVersion": SCHEMA,
            "head": "no_sha.capsule.json",
            "depth": 1,
            "valid": False,
            "brokenAt": "no_sha.capsule.json",
            "links": [
                {
                    "capsule": "no_sha.capsule.json",
                    "error": "parent.sha256 가 없거나 64자리 16진이 아님",
                }
            ],
            "untrustedContent": False,
            "untrustedFields": [],
        },
        3,
        "lineage-fail-closed-sha",
        "lineage",
        "해시 누락은 생략이 아니라 fail-closed",
    )
    return names


def generate_layouts(meta):
    """Real folder trees so tests can walk *.capsule.json non-recursively."""

    def copy_named(src_id, dest: Path):
        src = FIX / "capsules" / f"{src_id}.capsule.json"
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(src.read_bytes())

    all_ok = FIX / "audit-layouts" / "all-ok"
    for i, wid in enumerate(("notice_year", "org_rename", "deadline")):
        copy_named(wid, all_ok / f"{wid}.capsule.json")
    write_text(
        all_ok / "notes.txt",
        "이 파일은 *.capsule.json 이 아니다. audit 가 세면 안 된다.\n",
    )

    mixed = FIX / "audit-layouts" / "mixed"
    copy_named("notice_year", mixed / "ok_a.capsule.json")
    copy_named("org_rename", mixed / "ok_b.capsule.json")
    tampered = json.loads(
        (FIX / "capsules" / "notice_year.capsule.json").read_text(encoding="utf-8")
    )
    tampered["receipt"]["outputSha256"] = ZERO64
    dump(mixed / "tampered.capsule.json", tampered)

    nested = FIX / "audit-layouts" / "nested-ignored"
    copy_named("notice_year", nested / "top.capsule.json")
    copy_named("deadline", nested / "nested" / "hidden.capsule.json")
    write_text(
        nested / "README.md",
        "nested/hidden.capsule.json 은 비재귀 규약상 감사 대상이 아니다.\n",
    )

    empty = FIX / "audit-layouts" / "empty"
    write_text(empty / "README.md", "캡슐 0개. audit 는 exit 2.\n")

    mixed_ext = FIX / "audit-layouts" / "mixed-ext"
    copy_named("title_fix", mixed_ext / "keep.capsule.json")
    dump(mixed_ext / "notes.json", {"not": "a capsule"})
    write_text(mixed_ext / "keep.capsule.json.bak", "{}\n")
    write_text(mixed_ext / "keep.capsule.json.txt", "위장 확장자\n")

    same = FIX / "audit-layouts" / "same-folder-chain"
    copy_named("notice_year", same / "a.capsule.json")
    copy_named("chain_deadline", same / "b.capsule.json")

    sub = FIX / "lineage-layouts" / "relative-subdir"
    copy_named("notice_year", sub / "root" / "a.capsule.json")
    parent_path = sub / "root" / "a.capsule.json"
    parent_sha = sha256_hex(parent_path.read_bytes())
    child_item = {
        "id": "rel_child",
        "find": "x",
        "replace": "y",
        "title": "상대 경로 자식",
        "why": "부모는 ../root/a.capsule.json",
    }
    cap, _ = build_item_capsule(child_item)
    cap["receipt"]["inputSha256"] = meta["notice_year"]["outputSha256"]
    cap["plan"]["input"] = "out/notice_year.hwp"
    cap["planText"] = compact(cap["plan"])
    cap["receipt"]["planSha256"] = sha256_hex(cap["planText"].encode("utf-8"))
    # parent path is relative to the child capsule file (in child/).
    cap["parent"] = {"capsule": "../root/a.capsule.json", "sha256": parent_sha}
    dump(sub / "child" / "b.capsule.json", cap)
    write_text(
        sub / "README.md",
        "parent.capsule 은 자식 파일 기준 상대 경로다. 호출 cwd 가 아니다.\n",
    )

    broken = FIX / "lineage-layouts" / "lineage-broken"
    copy_named("notice_year", broken / "a.capsule.json")
    parent_sha = sha256_hex((broken / "a.capsule.json").read_bytes())
    cap, _ = build_item_capsule(
        {"id": "broken_child", "find": "p", "replace": "q", "title": "깨진 연대", "why": "x"}
    )
    # Deliberately different input hash.
    cap["receipt"]["inputSha256"] = sha256_hex(b"not-the-parent-output")
    cap["plan"]["input"] = "out/unrelated.hwp"
    cap["planText"] = compact(cap["plan"])
    cap["receipt"]["planSha256"] = sha256_hex(cap["planText"].encode("utf-8"))
    cap["parent"] = {"capsule": "a.capsule.json", "sha256": parent_sha}
    dump(broken / "b.capsule.json", cap)

    no_sha = FIX / "lineage-layouts" / "missing-parent-sha"
    copy_named("notice_year", no_sha / "a.capsule.json")
    cap, _ = build_item_capsule(
        {"id": "no_sha", "find": "p", "replace": "q", "title": "해시 누락", "why": "x"}
    )
    cap["parent"] = {"capsule": "a.capsule.json"}
    dump(no_sha / "b.capsule.json", cap)

    three = FIX / "lineage-layouts" / "three-link"
    copy_named("notice_year", three / "a.capsule.json")
    copy_named("chain_deadline", three / "b.capsule.json")
    # Grandchild of chain_deadline
    mid = meta["chain_deadline"]
    g_item = {
        "id": "grand",
        "find": "4월 15일",
        "replace": "4월 30일",
        "title": "손자",
        "why": "3링크",
    }
    gcap, _ = build_item_capsule(g_item)
    gcap["receipt"]["inputSha256"] = mid["outputSha256"]
    gcap["plan"]["input"] = mid["output"]
    gcap["planText"] = compact(gcap["plan"])
    gcap["receipt"]["planSha256"] = sha256_hex(gcap["planText"].encode("utf-8"))
    gcap["parent"] = {
        "capsule": "b.capsule.json",
        "sha256": sha256_hex((three / "b.capsule.json").read_bytes()),
    }
    dump(three / "c.capsule.json", gcap)

    layouts = [
        {
            "id": "all-ok",
            "root": "fixtures/audit-layouts/all-ok",
            "nonRecursiveCapsuleCount": 3,
            "nestedCapsuleCount": 0,
            "expectedTotal": 3,
            "expectedReproduced": 3,
            "expectedRate": 1.0,
            "expectedExit": 0,
        },
        {
            "id": "mixed",
            "root": "fixtures/audit-layouts/mixed",
            "nonRecursiveCapsuleCount": 3,
            "expectedTotal": 3,
            "expectedReproduced": 2,
            "expectedRate": 2 / 3,
            "expectedExit": 3,
        },
        {
            "id": "nested-ignored",
            "root": "fixtures/audit-layouts/nested-ignored",
            "nonRecursiveCapsuleCount": 1,
            "nestedCapsuleCount": 1,
            "expectedTotal": 1,
            "expectedExit": 0,
        },
        {
            "id": "empty",
            "root": "fixtures/audit-layouts/empty",
            "nonRecursiveCapsuleCount": 0,
            "expectedExit": 2,
            "stdoutBytes": 0,
        },
        {
            "id": "mixed-ext",
            "root": "fixtures/audit-layouts/mixed-ext",
            "nonRecursiveCapsuleCount": 1,
            "decoys": ["notes.json", "keep.capsule.json.bak", "keep.capsule.json.txt"],
            "expectedTotal": 1,
            "expectedExit": 0,
        },
        {
            "id": "same-folder-chain",
            "root": "fixtures/audit-layouts/same-folder-chain",
            "nonRecursiveCapsuleCount": 2,
            "note": "같은 폴더 상대 이름 a.capsule.json",
        },
        {
            "id": "relative-subdir",
            "root": "fixtures/lineage-layouts/relative-subdir",
            "parentPath": "../root/a.capsule.json",
            "parentPathRelativeTo": "child/b.capsule.json",
        },
        {
            "id": "lineage-broken",
            "root": "fixtures/lineage-layouts/lineage-broken",
            "expectLineageOk": False,
            "expectedExit": 3,
        },
        {
            "id": "missing-parent-sha",
            "root": "fixtures/lineage-layouts/missing-parent-sha",
            "expectedExit": 3,
            "errorContains": "parent.sha256",
        },
        {
            "id": "three-link",
            "root": "fixtures/lineage-layouts/three-link",
            "expectedDepth": 3,
        },
    ]
    dump(FIX / "layout_index.json", {"layouts": layouts})
    return layouts


def generate_hash_vectors(meta):
    vectors = []
    for item in WORK[:8]:
        plan = make_plan(
            SAMPLE if item.get("ext", "hwp") == "hwp" else SAMPLE.replace(".hwp", ".hwpx"),
            f"out/{item['id']}.{item.get('ext', 'hwp')}",
            item["find"],
            item["replace"],
            item.get("extra"),
        )
        text = compact(plan)
        vectors.append(
            {
                "id": f"plan-{item['id']}",
                "alg": "SHA-256",
                "kind": "planText",
                "payloadUtf8": text,
                "sha256": sha256_hex(text.encode("utf-8")),
                "matchesCapsule": meta[item["id"]]["planSha256"],
            }
        )
        ib = input_bytes(item["id"])
        ob = output_bytes(item["id"], f"{item['find']}->{item['replace']}")
        vectors.append(
            {
                "id": f"input-{item['id']}",
                "alg": "SHA-256",
                "kind": "syntheticInput",
                "payloadUtf8": ib.decode("utf-8"),
                "sha256": sha256_hex(ib),
            }
        )
        vectors.append(
            {
                "id": f"output-{item['id']}",
                "alg": "SHA-256",
                "kind": "syntheticOutput",
                "payloadUtf8": ob.decode("utf-8"),
                "sha256": sha256_hex(ob),
            }
        )
    # Format gates
    vectors.append(
        {
            "id": "expect-hex-ok",
            "kind": "expect-output-sha256",
            "value": meta["notice_year"]["outputSha256"],
            "valid": True,
            "reason": "64 ascii hex",
        }
    )
    vectors.append(
        {
            "id": "expect-hex-short",
            "kind": "expect-output-sha256",
            "value": "abc",
            "valid": False,
            "exit": 2,
            "reason": "길이 != 64",
        }
    )
    vectors.append(
        {
            "id": "expect-hex-badchar",
            "kind": "expect-output-sha256",
            "value": "z" * 64,
            "valid": False,
            "exit": 2,
            "reason": "비hex",
        }
    )
    vectors.append(
        {
            "id": "expect-hex-upper",
            "kind": "expect-output-sha256",
            "value": meta["notice_year"]["outputSha256"].upper(),
            "valid": True,
            "normalized": "lowercase",
            "reason": "CLI 가 ascii lowercase 로 정규화",
        }
    )
    dump(
        FIX / "hash-vectors" / "vectors.json",
        {
            "alg": "SHA-256",
            "encoding": "utf-8",
            "note": "planSha256 은 계획 원문 바이트. 공백·키 순서가 바뀌면 해시가 바뀐다.",
            "vectors": vectors,
        },
    )
    return [v["id"] for v in vectors]


def generate_transcripts(meta):
    names = []

    def put(name, obj):
        dump(FIX / "transcripts" / name, obj)
        names.append(name)

    n = meta["notice_year"]
    put(
        "attest_notice.json",
        {
            "argv": [
                "rhwp",
                "replay",
                "--plan-json",
                n["planText"],
                "--json",
            ],
            "exit": 0,
            "stdoutKeys": [
                "mode",
                "inputSha256",
                "planSha256",
                "outputSha256",
                "toolVersion",
                "steps",
                "reproduced",
            ],
            "mode": "attest",
            "createsUserOutput": False,
            "envelope": "fixtures/envelopes/replay_attest.json",
        },
    )
    put(
        "verify_match.json",
        {
            "argv": [
                "rhwp",
                "replay",
                "--plan-json",
                n["planText"],
                "--expect-output-sha256",
                n["outputSha256"],
                "--json",
            ],
            "exit": 0,
            "mode": "verify",
            "reproduced": True,
            "envelope": "fixtures/envelopes/replay_verify_match.json",
        },
    )
    put(
        "verify_mismatch.json",
        {
            "argv": [
                "rhwp",
                "replay",
                "--plan-json",
                n["planText"],
                "--expect-output-sha256",
                ZERO64,
                "--json",
            ],
            "exit": 3,
            "mode": "verify",
            "reproduced": False,
            "judgmentNotCrash": True,
            "envelope": "fixtures/envelopes/replay_verify_mismatch.json",
        },
    )
    put(
        "capsule_issue.json",
        {
            "argv": [
                "rhwp",
                "replay",
                "--plan-json",
                n["planText"],
                "--capsule",
                "a.capsule.json",
                "--json",
            ],
            "exit": 0,
            "writes": ["a.capsule.json"],
            "capsuleKind": "workCapsule",
            "parent": None,
        },
    )
    put(
        "capsule_parent.json",
        {
            "argv": [
                "rhwp",
                "replay",
                "--plan-json",
                meta["chain_deadline"]["planText"],
                "--capsule",
                "b.capsule.json",
                "--parent",
                "a.capsule.json",
                "--json",
            ],
            "exit": 0,
            "parentStoredRelativeToCapsuleFile": True,
            "parentField": {"capsule": "a.capsule.json", "sha256": "64hex"},
        },
    )
    put(
        "run_then_replay.json",
        {
            "steps": [
                {
                    "argv": ["rhwp", "run", "planA.json", "--json"],
                    "why": "실산출 O1. replay 는 사용자 경로를 쓰지 않는다",
                },
                {
                    "argv": [
                        "rhwp",
                        "replay",
                        "--plan-json",
                        "<planB input=O1>",
                        "--capsule",
                        "b.capsule.json",
                        "--parent",
                        "a.capsule.json",
                        "--json",
                    ]
                },
            ]
        },
    )
    put(
        "audit_folder.json",
        {
            "argv": ["rhwp", "audit", "fixtures/audit-layouts/all-ok", "--json"],
            "exit": 0,
            "nonRecursive": True,
            "glob": "*.capsule.json",
            "envelope": "fixtures/envelopes/audit_all_ok.json",
        },
    )
    put(
        "audit_empty.json",
        {
            "argv": ["rhwp", "audit", "fixtures/audit-layouts/empty", "--json"],
            "exit": 2,
            "stdoutBytes": 0,
        },
    )
    put(
        "lineage_head.json",
        {
            "argv": ["rhwp", "lineage", "b.capsule.json", "--json"],
            "exit": 0,
            "axes": ["parentOk", "lineageOk", "reproduced"],
            "envelope": "fixtures/envelopes/lineage_two_link.json",
        },
    )
    put(
        "lineage_deep.json",
        {
            "argv": ["rhwp", "lineage", "b.capsule.json", "--deep", "--json"],
            "exit": 0,
            "reproducedIsNullWithoutDeep": True,
            "envelope": "fixtures/envelopes/lineage_deep.json",
        },
    )
    put(
        "lineage_missing.json",
        {
            "argv": ["rhwp", "lineage", "missing.capsule.json", "--json"],
            "exit": 1,
            "stdoutBytes": 0,
        },
    )
    put(
        "replay_no_args.json",
        {"argv": ["rhwp", "replay"], "exit": 2, "stdoutBytes": 0},
    )
    put(
        "audit_no_args.json",
        {"argv": ["rhwp", "audit"], "exit": 2, "stdoutBytes": 0},
    )
    put(
        "lineage_no_args.json",
        {"argv": ["rhwp", "lineage"], "exit": 2, "stdoutBytes": 0},
    )
    put(
        "same_file_parent.json",
        {
            "argv": [
                "rhwp",
                "replay",
                "--plan-json",
                n["planText"],
                "--capsule",
                "a.capsule.json",
                "--parent",
                "a.capsule.json",
            ],
            "exit": 2,
        },
    )
    put(
        "toolversion_check.json",
        {
            "argv": [
                "rhwp",
                "replay",
                "--plan-json",
                n["planText"],
                "--expect-output-sha256",
                n["outputSha256"],
                "--json",
            ],
            "precheck": "envelope.toolVersion == claimed.toolVersion",
            "pitfall": "버전이 다른데 reproduced:false 이면 주장이 아니라 도구가 갈렸을 수 있다",
        },
    )
    return names


def generate_scenarios(meta):
    scenarios = []

    def add(**kwargs):
        scenarios.append(kwargs)

    # Attest / verify
    for item in WORK:
        add(
            id=f"attest-{item['id']}",
            family="replay-attest",
            request=f"{item['title']} 증명해 줘",
            command="replay",
            argv=["replay", "--plan-json", "<plan>", "--json"],
            expectExit=0,
            expectKeys=["inputSha256", "planSha256", "outputSha256", "toolVersion", "mode"],
            mode="attest",
            pitfall="output 경로가 생기지 않는다. 실산출은 run",
            reference="replay-attest.md",
        )
        add(
            id=f"verify-{item['id']}",
            family="replay-verify",
            request=f"{item['title']} 산출 해시가 맞는지 확인해",
            command="replay",
            argv=[
                "replay",
                "--plan-json",
                "<plan>",
                "--expect-output-sha256",
                "<64hex>",
                "--json",
            ],
            expectExit=[0, 3],
            expectKeys=["reproduced", "expectedOutputSha256", "outputSha256"],
            mode="verify",
            pitfall="reproduced:false 는 exit 3 판정이지 크래시가 아니다",
            reference="replay-attest.md",
        )

    add(
        id="verify-bad-hex",
        family="replay-verify",
        request="해시가 짧다",
        command="replay",
        argv=["replay", "--plan-json", "<plan>", "--expect-output-sha256", "abc"],
        expectExit=2,
        pitfall="64hex 가 아니면 판정 전에 사용법",
        reference="exit-codes.md",
    )
    add(
        id="attest-plan-file",
        family="replay-attest",
        request="계획 파일로 영수증",
        command="replay",
        argv=["replay", "plan.json", "--json"],
        expectExit=0,
        pitfall="위치 인자는 계획 경로. 캡슐 경로가 아니다",
        reference="replay-attest.md",
    )
    add(
        id="attest-missing-plan",
        family="replay-attest",
        request="없는 계획",
        command="replay",
        argv=["replay", "missing.json"],
        expectExit=1,
        stdoutBytes=0,
        reference="exit-codes.md",
    )
    add(
        id="capsule-issue",
        family="capsule",
        request="작업 캡슐 만들어",
        command="replay",
        argv=["replay", "--plan-json", "<plan>", "--capsule", "a.capsule.json", "--json"],
        expectExit=0,
        writes=["a.capsule.json"],
        kind="workCapsule",
        reference="capsule-chain.md",
    )
    add(
        id="capsule-parent",
        family="capsule",
        request="이어서 기록",
        command="replay",
        argv=[
            "replay",
            "--plan-json",
            "<planB>",
            "--capsule",
            "b.capsule.json",
            "--parent",
            "a.capsule.json",
            "--json",
        ],
        expectExit=0,
        parentRelativeTo="b.capsule.json",
        reference="capsule-chain.md",
    )
    add(
        id="capsule-same-file",
        family="capsule",
        request="부모와 같은 파일",
        command="replay",
        argv=[
            "replay",
            "--plan-json",
            "<plan>",
            "--capsule",
            "a.capsule.json",
            "--parent",
            "a.capsule.json",
        ],
        expectExit=2,
        reference="capsule-chain.md",
    )
    add(
        id="capsule-immutability",
        family="capsule",
        request="캡슐을 고쳐서 재제출",
        command="lineage",
        argv=["lineage", "child.capsule.json", "--json"],
        expectExit=3,
        pitfall="에디터 저장은 파일 바이트를 바꾼다. 재발급하라",
        reference="capsule-chain.md",
    )
    add(
        id="run-then-chain",
        family="capsule",
        request="실산출을 다음 입력으로",
        command="run",
        argv=["run", "planA.json", "--json"],
        next=["replay --capsule b --parent a"],
        pitfall="replay 만으로는 O1 이 디스크에 없다",
        reference="capsule-chain.md",
    )

    add(
        id="audit-all",
        family="audit",
        request="이 폴더 전수 재현율",
        command="audit",
        argv=["audit", "<dir>", "--json"],
        expectKeys=["total", "reproduced", "reproducedRate", "failed"],
        glob="*.capsule.json",
        recursive=False,
        reference="audit-accounting.md",
    )
    add(
        id="audit-empty",
        family="audit",
        request="빈 폴더 감사",
        command="audit",
        argv=["audit", "empty", "--json"],
        expectExit=2,
        stdoutBytes=0,
        reference="audit-accounting.md",
    )
    add(
        id="audit-nested",
        family="audit",
        request="하위 폴더 캡슐도 세어줘",
        command="audit",
        argv=["audit", "nested-ignored", "--json"],
        expectedTotalIgnoresNested=True,
        pitfall="비재귀. 하위는 그 폴더에서 따로 audit",
        reference="audit-accounting.md",
    )
    add(
        id="audit-rate",
        family="audit",
        request="재현율 숫자",
        command="audit",
        argv=["audit", "mixed", "--json"],
        formula="reproduced/total",
        expectExit=3,
        reference="audit-accounting.md",
    )
    add(
        id="audit-decoy-ext",
        family="audit",
        request="bak 도 세나",
        command="audit",
        argv=["audit", "mixed-ext", "--json"],
        countedSuffix=".capsule.json",
        notCounted=[".json", ".bak", ".txt"],
        reference="audit-accounting.md",
    )

    add(
        id="lineage-head",
        family="lineage",
        request="계보 검증",
        command="lineage",
        argv=["lineage", "b.capsule.json", "--json"],
        expectKeys=["head", "depth", "valid", "brokenAt", "links"],
        axes=["parentOk", "lineageOk", "reproduced"],
        reference="lineage-chronicle.md",
    )
    add(
        id="lineage-deep",
        family="lineage",
        request="링크마다 재실행까지",
        command="lineage",
        argv=["lineage", "b.capsule.json", "--deep", "--json"],
        cost="링크 수",
        reference="lineage-chronicle.md",
    )
    add(
        id="lineage-missing",
        family="lineage",
        request="없는 머리",
        command="lineage",
        argv=["lineage", "missing.capsule.json"],
        expectExit=1,
        reference="exit-codes.md",
    )
    add(
        id="lineage-noargs",
        family="lineage",
        request="인자 없이",
        command="lineage",
        argv=["lineage"],
        expectExit=2,
        reference="exit-codes.md",
    )
    add(
        id="lineage-parentOk",
        family="lineage",
        request="부모 파일을 고쳤다",
        command="lineage",
        argv=["lineage", "b.capsule.json", "--json"],
        expectExit=3,
        brokenAxis="parentOk",
        reference="lineage-chronicle.md",
    )
    add(
        id="lineage-lineageOk",
        family="lineage",
        request="입력이 부모 산출이 아니다",
        command="lineage",
        argv=["lineage", "b.capsule.json", "--json"],
        expectExit=3,
        brokenAxis="lineageOk",
        reference="lineage-chronicle.md",
    )
    add(
        id="lineage-relative",
        family="lineage",
        request="부모 경로가 하위 폴더",
        command="lineage",
        argv=["lineage", "child/b.capsule.json", "--json"],
        parentResolvedFrom="capsule file",
        reference="lineage-chronicle.md",
    )

    add(
        id="toolversion-precheck",
        family="pitfall",
        request="다른 버전에서 재현 실패",
        command="replay",
        argv=["replay", "--plan-json", "<plan>", "--expect-output-sha256", "<hex>", "--json"],
        pitfall="toolVersion 을 먼저 대조. 불일치를 상대 부정으로 단정하지 마라",
        reference="pitfalls.md",
    )
    add(
        id="no-attribution",
        family="pitfall",
        request="누가 했는지 증명해",
        command="replay",
        argv=["replay", "--plan-json", "<plan>", "--json"],
        proves="what/how/bytes",
        doesNotProve="who",
        reference="pitfalls.md",
    )
    add(
        id="no-new-cli",
        family="boundary",
        request="receipt 명령 만들어",
        command=None,
        refuse=True,
        existing=["replay", "audit", "lineage"],
        reference="SKILL.md",
    )
    add(
        id="no-gym",
        family="boundary",
        request="gym pack 으로 채점",
        command=None,
        refuse=True,
        reference="SKILL.md",
    )
    add(
        id="exit3-is-data",
        family="exit",
        request="재현 실패가 났는데 크래시야?",
        command="replay",
        expectExit=3,
        judgment=True,
        reference="exit-codes.md",
    )
    add(
        id="fail-stdout-silent",
        family="exit",
        request="exit 2 인데 JSON 이 안 나와",
        command="replay",
        expectExit=2,
        stdoutBytes=0,
        reference="exit-codes.md",
    )

    # More request-routing recipes (real agent phrasings)
    phrases = [
        ("이 작업 증명해", "replay", "attest"),
        ("영수증 남겨", "replay", "attest"),
        ("타인 산출물 재현 검증", "replay", "verify"),
        ("작업 캡슐 만들어", "replay", "capsule"),
        ("체인 만들어", "replay", "parent"),
        ("캡슐 폴더 감사", "audit", "folder"),
        ("재현율", "audit", "rate"),
        ("계보 검증", "lineage", "head"),
        ("deep 로 다시", "lineage", "deep"),
        ("부모 해시 깨졌어", "lineage", "parentOk"),
        ("산출이 다음 입력이냐", "lineage", "lineageOk"),
        ("서명으로 누가 했는지", None, "no-attribution"),
        ("새 명령 work-receipt", None, "no-new-cli"),
    ]
    for i, (phrase, cmd, tag) in enumerate(phrases):
        add(
            id=f"phrase-{i:02d}-{tag}",
            family="routing",
            request=phrase,
            command=cmd,
            tag=tag,
            reference="decision-tree.md",
        )

    dump(
        FIX / "scenario_catalog.json",
        {
            "catalogVersion": "1.0",
            "skill": "rhwp-work-receipt",
            "issue": 5308,
            "count": len(scenarios),
            "families": sorted({s["family"] for s in scenarios}),
            "scenarios": scenarios,
        },
    )
    return scenarios


def generate_examples():
    rows = [
        ("01_attest_three_hashes.md", "영수증", "attest 3해시 발급. 사용자 경로 무생성"),
        ("02_verify_expect_output.md", "영수증", "--expect-output-sha256 제3자 검증"),
        ("03_verify_mismatch_exit3.md", "영수증", "reproduced:false = exit 3 판정"),
        ("04_plan_file_vs_inline.md", "영수증", "위치 인자 계획 파일 vs --plan-json"),
        ("05_capsule_issue.md", "캡슐", "workCapsule 자기완결 교환"),
        ("06_parent_same_folder.md", "캡슐", "같은 폴더 --parent 상대 이름"),
        ("07_parent_relative_subdir.md", "캡슐", "자식 파일 기준 ../root/a.capsule.json"),
        ("08_immutability.md", "캡슐", "포맷터 저장이 parentOk 를 깬다"),
        ("09_same_file_rejected.md", "캡슐", "--capsule == --parent 거부"),
        ("10_run_then_chain.md", "캡슐", "실산출은 run, 증명은 replay"),
        ("11_audit_all_ok.md", "감사", "reproducedRate 1.0"),
        ("12_audit_mixed_rate.md", "감사", "2/3 회계 + exit 3"),
        ("13_audit_non_recursive.md", "감사", "하위 폴더 캡슐 무시"),
        ("14_audit_empty_exit2.md", "감사", "0개 = 사용법"),
        ("15_lineage_root.md", "계보", "뿌리 3축 null"),
        ("16_lineage_two_link.md", "계보", "parentOk·lineageOk"),
        ("17_lineage_deep.md", "계보", "--deep 재실행"),
        ("18_lineage_broken_at.md", "계보", "brokenAt 명세"),
        ("19_toolversion_pitfall.md", "함정", "버전 불일치 선대조"),
        ("20_no_attribution.md", "함정", "누가 했는지는 증명하지 않는다"),
    ]
    readme = [
        "# rhwp-work-receipt 워크스루",
        "",
        "실사용 에이전트가 기존 `replay` / `audit` / `lineage` 만으로 노동을",
        "증명하는 표본이다. 새 CLI 는 없다. gym 은 없다.",
        "",
        "레퍼런스:",
        "",
        "- [replay-attest.md](../references/replay-attest.md)",
        "- [capsule-chain.md](../references/capsule-chain.md)",
        "- [audit-accounting.md](../references/audit-accounting.md)",
        "- [lineage-chronicle.md](../references/lineage-chronicle.md)",
        "- [exit-codes.md](../references/exit-codes.md)",
        "- [pitfalls.md](../references/pitfalls.md)",
        "",
        "픽스처 목록은 [../fixtures/catalog.json](../fixtures/catalog.json).",
        "",
        "## 목록",
        "",
        "| # | 파일 | 단 | 보여주는 것 |",
        "|---|------|----|-------------|",
    ]
    for i, (name, rung, what) in enumerate(rows, 1):
        readme.append(f"| {i:02d} | [{name}]({name}) | {rung} | {what} |")
    readme += [
        "",
        "## 공통 규칙",
        "",
        "1. 입력은 공개 샘플 경로 또는 워크스루가 밝히는 가명이다. 새 HWP 바이너리를 두지 않는다.",
        "2. 명령 머리는 `replay` / `audit` / `lineage` / `run` 뿐이다. `run` 은 실산출이 필요할 때만.",
        "3. exit 3 은 판정 데이터다. 크래시로 재시도하지 않는다.",
        "4. 캡슐은 발급 후 불변이다. 고치려면 재발급한다.",
        "5. 귀속(누가)과 서명은 이 스킬 1부의 범위가 아니다.",
        "",
    ]
    write_text(EXAMPLES / "README.md", "\n".join(readme))

    bodies = {
        "01_attest_three_hashes.md": """# 01 — 단건 영수증 발급 (attest)

단: 영수증. 목표: 계획 하나를 임시 재실행해 입력·계획·산출 SHA-256 을 받는다.
사용자 `output` 경로는 **생기지 않는다**.

권위: [replay-attest.md](../references/replay-attest.md).
픽스처: [../fixtures/envelopes/replay_attest.json](../fixtures/envelopes/replay_attest.json).

## 0. 하지 않는 것

- 새 `receipt` 명령을 만들지 않는다. 기존 `rhwp replay` 다.
- 산출 파일을 사용자 경로에 쓰지 않는다. 필요하면 10 편 `run`.
- 해시를 손으로 지어내지 않는다.

## 1. 계획

```bash
rhwp replay --plan-json '{"planVersion":"1.0","input":"samples/basic/issue2007_nested_cell_pagination_42065.hwp","output":"out/notice.hwp","steps":[{"action":"replace_text","find":"2025년","replace":"2026년"}]}' --json
```

## 2. 읽는 필드

| 키 | 뜻 |
|----|----|
| `mode` | `attest` |
| `inputSha256` | 입력 문서 바이트 |
| `planSha256` | **계획 원문** 바이트. 공백이 바뀌면 해시가 바뀐다 |
| `outputSha256` | 임시 재실행 산출 바이트 |
| `toolVersion` | 재현 조건. 19 편 |
| `reproduced` | attest 에서는 `null` |
| `steps` | 숫자. `run` 저널 배열과 동명 다른 타입 |

## 3. 전달

영수증 JSON 을 산출물(있다면 `run` 이 쓴 파일)과 함께 넘긴다.
제3자는 02 편으로 검증한다.

## 4. 명령 체크리스트

- [ ] `rhwp replay` 이다
- [ ] `--json` 으로 3해시를 읽었다
- [ ] 사용자 경로에 파일이 생기지 않았음을 확인했다
- [ ] `toolVersion` 을 같이 적었다
""",
        "02_verify_expect_output.md": """# 02 — 제3자 검증 (`--expect-output-sha256`)

단: 영수증. 목표: 상대가 준 64hex 가 같은 계획의 재실행 산출과 같은지 판정한다.

권위: [replay-attest.md](../references/replay-attest.md).
픽스처: [../fixtures/envelopes/replay_verify_match.json](../fixtures/envelopes/replay_verify_match.json).

## 1. 요구할 것

상대에게 **같은 계획 원문** 과 **outputSha256** 과 **toolVersion** 을 받는다.
계획 공백이 다르면 `planSha256` 이 달라 다른 작업이다.

## 2. 호출

```bash
rhwp replay --plan-json '<같은 계획>' --expect-output-sha256 <64hex> --json
```

## 3. 판정

| 봉투 | exit | 다음 |
|------|-----:|------|
| `reproduced: true` | 0 | 주장 채택 |
| `reproduced: false` | 3 | 주장 기각. 03 편. **재시도 금지** |
| 짧은 해시 / 비hex | 2 | 호출 조립을 고친다 |

`mode` 는 `verify`. `expectedOutputSha256` 이 요청 값이다.

## 4. 하지 않는 것

- 불일치를 도구 고장으로 승격하지 않는다.
- 상대 해시를 계획 없이 믿지 않는다. 재실행이 증명이다.
""",
        "03_verify_mismatch_exit3.md": """# 03 — 검증 불일치 (exit 3)

단: 영수증. 목표: `reproduced:false` 를 판정 데이터로 읽고 멈춘다.

권위: [exit-codes.md](../references/exit-codes.md).
픽스처: [../fixtures/envelopes/replay_verify_mismatch.json](../fixtures/envelopes/replay_verify_mismatch.json).

```bash
rhwp replay --plan-json '<계획>' --expect-output-sha256 0000000000000000000000000000000000000000000000000000000000000000 --json
```

기대:

- exit **3**
- `mode: "verify"`
- `reproduced: false`
- `outputSha256` 는 재실행 실측 (64hex)
- `expectedOutputSha256` 는 주장 값

이 숫자는 도구 크래시(1)도 사용법(2)도 아니다. 봉투를 사용자에게 보여 주고
주장을 기각한다. 같은 명령을 그대로 다시 돌리지 않는다 — 결정론이면 같은 판정이다.

선대조: `toolVersion` 이 상대와 다르면 19 편.
""",
        "04_plan_file_vs_inline.md": """# 04 — 계획 파일과 `--plan-json`

단: 영수증. 목표: 두 입력 경로가 같은 명령을 쓰는지 고정한다.

```bash
rhwp replay plan.json --json
rhwp replay --plan-json "$(cat plan.json)" --json
```

`planSha256` 은 **원문 바이트**다. `cat` 이 개행을 바꾸거나 에디터가
pretty-print 하면 해시가 갈린다. 제3자 검증은 **같은 바이트**를 써야 한다.

함정: 위치 인자는 계획이지 캡슐이 아니다.

```bash
# 하지 말 것 — 캡슐을 계획으로 넣음
rhwp replay a.capsule.json --json
```

캡슐 검증은 `lineage` / `audit` 이다.
""",
        "05_capsule_issue.md": """# 05 — 작업 캡슐 발급

단: 캡슐. 목표: 계획+영수증을 자기완결 파일로 남긴다.

권위: [capsule-chain.md](../references/capsule-chain.md).

```bash
rhwp replay --plan-json '<계획A>' --capsule a.capsule.json --json
```

파일 골격:

- `kind: "workCapsule"`
- `parent: null` (뿌리)
- `plan` / `planText` — 객체와 원문이 같아야 한다
- `receipt` — 01 편의 봉투

제3자는 이 파일만 받으면 `audit` 한 폴더 또는 `lineage` 한 머리로 재현할 수 있다.
`plan.output` 은 발급 당시 사용자 경로를 보존한다. 재실행은 임시 경로로 덮어쓴다.

발급 후 파일을 열어서 저장하지 마라. 08 편.
""",
        "06_parent_same_folder.md": """# 06 — 같은 폴더 해시 체인

단: 캡슐. 목표: 다음 작업의 입력이 이전 실산출일 때 `--parent` 로 잇는다.

```bash
rhwp run planA.json --json
rhwp replay --plan-json '<계획A>' --capsule a.capsule.json --json
rhwp replay --plan-json '<계획B: input=O1>' --capsule b.capsule.json --parent a.capsule.json --json
```

같은 폴더면 `parent.capsule` 은 `a.capsule.json` 이다. 저장·해석은 **캡슐 파일
기준**이지 호출 cwd 가 아니다.

16 편 `lineage` 로 `parentOk` 와 `lineageOk` 를 읽는다.
""",
        "07_parent_relative_subdir.md": """# 07 — 자식 파일 기준 상대 경로

단: 캡슐. 픽스처: [../fixtures/lineage-layouts/relative-subdir](../fixtures/lineage-layouts/relative-subdir).

부모가 `root/a.capsule.json`, 자식이 `child/b.capsule.json` 이면 저장 값은
`../root/a.capsule.json` 이다.

```bash
rhwp replay --plan-json '<계획B>' --capsule child/b.capsule.json --parent root/a.capsule.json --json
```

`lineage` 는 현재 캡슐의 부모 디렉터리에 상대 경로를 붙인다. cwd 에서 해석하면
깨진 체인으로 오진한다.

체크리스트:

- [ ] `parent.capsule` 에 `..` 또는 같은 폴더 이름만 있다
- [ ] 절대 경로가 들어갔다면 다른 볼륨·다른 머신에서 깨진다
""",
        "08_immutability.md": """# 08 — 캡슐 불변

단: 캡슐. 목표: 에디터·포맷터가 파일 바이트를 바꾸면 자식의 `parent.sha256` 이 깨진다.

의도된 동작이다. 변조 검출.

```bash
# 하지 말 것
# a.capsule.json 을 열어 들여쓰기만 바꿔 저장
rhwp lineage b.capsule.json --json
# → valid:false, parentOk:false, brokenAt, exit 3
```

고치려면 **재발급**한다. 필드를 손으로 고친 캡슐은 더 이상 그 체인의 부모가 아니다.

픽스처: `fixtures/capsules/tamper_pretty_print.capsule.json`.
""",
        "09_same_file_rejected.md": """# 09 — 부모 덮어쓰기 방지

```bash
rhwp replay --plan-json '<계획>' --capsule a.capsule.json --parent a.capsule.json
```

기대: exit **2**, 부모 파일을 덮어쓰지 않는다.
픽스처: [../fixtures/envelopes/replay_parent_same_file.json](../fixtures/envelopes/replay_parent_same_file.json).

새 작업은 새 파일명이다. `a2.capsule.json --parent a.capsule.json`.
""",
        "10_run_then_chain.md": """# 10 — 실산출은 `run`, 증명은 `replay`

단: 캡슐. `replay` 는 임시 산출만 해시한 뒤 지운다. 다음 계획의 `input` 이
파일이어야 하면 먼저 `run` 한다.

```bash
rhwp run planA.json --json          # O1 생성, outputSha256 저널
rhwp replay --plan-json '<계획A>' --capsule a.capsule.json --json
# 계획B.input = O1
rhwp replay --plan-json '<계획B>' --capsule b.capsule.json --parent a.capsule.json --json
rhwp lineage b.capsule.json --json
```

`lineageOk` 는 **부모 산출 해시 == 자식 입력 해시** 다. run↔replay 교차
결정론이 이 등식을 받친다 (`tests/lineage_contract.rs`).
""",
        "11_audit_all_ok.md": """# 11 — 폴더 전수 재현율 1.0

단: 감사.

```bash
rhwp audit fixtures/audit-layouts/all-ok --json
```

기대 키: `total`, `reproduced`, `reproducedRate`, `failed`.
이 레이아웃은 직속 캡슐 3, 재현 3, 비율 1.0, `failed: []`, exit 0.

`notes.txt` 는 세지 않는다.
""",
        "12_audit_mixed_rate.md": """# 12 — 재현율 회계 (실패 포함)

```bash
rhwp audit fixtures/audit-layouts/mixed --json
```

- `total: 3`
- `reproduced: 2`
- `reproducedRate: 0.666…` (= 2/3)
- `failed[0].capsule == "tampered.capsule.json"`
- exit **3**

회계는 봉투로 읽고, 실패 캡슐만 01·02 편의 verify 로 추적한다.
한 건의 실패가 나머지 성공을 지우지 않는다 — `reproduced` 가 그 숫자다.
""",
        "13_audit_non_recursive.md": """# 13 — 비재귀 `*.capsule.json`

```bash
rhwp audit fixtures/audit-layouts/nested-ignored --json
```

직속 `top.capsule.json` 만 센다. `nested/hidden.capsule.json` 은 없다.
`total: 1`.

하위 폴더를 감사하려면 그 경로로 다시 `audit` 한다. 재귀 플래그는 없다.
이 스킬이 `--recursive` 를 발명하지 않는다.
""",
        "14_audit_empty_exit2.md": """# 14 — 빈 폴더는 사용법

```bash
rhwp audit fixtures/audit-layouts/empty --json
```

exit **2**, stdout 0바이트. 봉투의 `total: 0` 이 아니다 — 봉투 자체가 없다.

대상을 만들고 다시 부른다. 같은 빈 폴더를 루프로 재시도하지 않는다.
""",
        "15_lineage_root.md": """# 15 — 뿌리 계보

```bash
rhwp lineage notice_year.capsule.json --json
```

- `depth: 1`
- `valid: true`
- `brokenAt: null`
- `links[0].parentOk == null`
- `links[0].lineageOk == null`
- `links[0].reproduced == null` (`--deep` 없음)

뿌리는 대조할 자식 기록이 없다. null 은 "아직 모름"이지 "실패"가 아니다.
""",
        "16_lineage_two_link.md": """# 16 — 두 링크 연대기

```bash
rhwp lineage b.capsule.json --json
```

링크 판정 3축:

| 축 | 물음 |
|----|------|
| `parentOk` | 부모 파일이 발급 당시 바이트인가 (`parent.sha256`) |
| `lineageOk` | 부모 `outputSha256` == 자식 `inputSha256` |
| `reproduced` | `--deep` 일 때만. 아니면 `null` |

둘 다 true, `valid: true`, `brokenAt: null`, exit 0.
""",
        "17_lineage_deep.md": """# 17 — `--deep` 재실행

```bash
rhwp lineage b.capsule.json --deep --json
```

각 링크의 `reproduced` 가 true/false 로 채워진다. 비용은 링크 수다.
한 링크가 false 면 `valid: false`, `brokenAt` 이 그 캡슐, exit 3.

얕은 lineage 가 이미 깨졌으면 deep 을 돌릴 필요가 없다.
""",
        "18_lineage_broken_at.md": """# 18 — `brokenAt` 명세

부모를 포맷터로 저장한 뒤:

```bash
rhwp lineage b.capsule.json --json
```

- exit 3
- `valid: false`
- `brokenAt` 이 깨진 캡슐 경로
- 해당 링크 `parentOk: false` 또는 `lineageOk: false` 또는 `error`

머리 캡슐이 없으면 exit **1** (IO). 인자가 없으면 exit **2**.
누락 `parent.sha256` 은 생략이 아니라 fail-closed (exit 3).
""",
        "19_toolversion_pitfall.md": """# 19 — `toolVersion` 선대조

같은 계획이어도 rhwp 버전이 다르면 산출 바이트가 갈릴 수 있다.
`reproduced: false` 를 상대 부정으로 단정하기 전에 영수증의 `toolVersion` 을
지금 바이너리와 대조한다.

```bash
rhwp replay --plan-json '<계획>' --json   # toolVersion 확인
# 불일치 → 같은 버전으로 재현하거나, 불일치를 보고서에 적고 멈춘다
```

이 스킬은 버전을 맞추는 새 플래그를 만들지 않는다. 기존 영수증 필드다.
""",
        "20_no_attribution.md": """# 20 — 귀속을 주장하지 않는다

영수증·캡슐·audit·lineage 는 **무엇을·어떤 계획으로·어떤 바이트가** 나왔는지를
재실행으로 증명한다. **누가** 했는지는 증명하지 않는다.

`--sign-key` / `keygen` / `verify-signature` 는 4년 축이고 이 스킬 1부의
기본 경로가 아니다. 에이전트가 "서명했으니 작성자가 확인됐다"고 쓰지 않는다.

사용자가 작성자를 물으면: 3해시는 신원이 아니다. 운영 절차(PR 작성자, 키
등록부)는 별 축이다.
""",
    }
    names = []
    for name, _rung, _what in rows:
        write_text(EXAMPLES / name, bodies[name])
        names.append(name)
    return names


def generate_catalog(valid_plans, invalid_plans, envelopes, transcripts, examples, layouts):
    refs = [
        "replay-attest.md",
        "capsule-chain.md",
        "audit-accounting.md",
        "lineage-chronicle.md",
        "exit-codes.md",
        "pitfalls.md",
        "decision-tree.md",
        "envelope-field-catalog.md",
        "recipe-index.md",
        "README.md",
    ]
    catalog = {
        "catalogVersion": "1.0",
        "skill": "rhwp-work-receipt",
        "issue": 5308,
        "note": "기존 replay/audit/lineage 계약을 표본으로 고정한다. 새 CLI 없음. gym 없음.",
        "commands": ["replay", "audit", "lineage"],
        "helperCommands": ["run"],
        "hashes": ["inputSha256", "planSha256", "outputSha256"],
        "lineageAxes": ["parentOk", "lineageOk", "reproduced"],
        "exits": {"ok": 0, "io": 1, "usage": 2, "judgment": 3},
        "plans": {"valid": valid_plans, "invalid": invalid_plans},
        "envelopes": envelopes,
        "transcripts": transcripts,
        "examples": examples,
        "references": refs,
        "layouts": [x["id"] for x in layouts],
        "auditGlob": "*.capsule.json",
        "auditRecursive": False,
        "attributionClaim": False,
        "signatureClaim": False,
    }
    dump(FIX / "catalog.json", catalog)
    return catalog


def main():
    meta, children, tampers, index = generate_capsules()
    valid_plans, invalid_plans = generate_plans()
    envelopes = generate_envelopes(meta)
    layouts = generate_layouts(meta)
    vectors = generate_hash_vectors(meta)
    transcripts = generate_transcripts(meta)
    scenarios = generate_scenarios(meta)
    examples = generate_examples()
    catalog = generate_catalog(
        valid_plans, invalid_plans, envelopes, transcripts, examples, layouts
    )
    print(
        f"generated capsules={len(index)} envelopes={len(envelopes)} "
        f"scenarios={len(scenarios)} examples={len(examples)}"
    )


if __name__ == "__main__":
    main()
