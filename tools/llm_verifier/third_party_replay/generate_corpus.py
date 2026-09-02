#!/usr/bin/env python3
"""Emit the committed V-replay third-party labor corpus.

Each row is a distinct
``(plan, expect_sha, reproduced, toolVersion, verdict)`` case.
Comment padding is not used. Plans are real-looking rhwp plan JSON
(the `planSha256` byte target). expect_sha follows the replay hex contract.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

try:
    from .corpus_io import CORPUS_DIR, HERE, write_shard
    from .decide import (
        CLAIM_ID,
        KIND,
        SCHEMA_VERSION,
        decide_observation,
    )
    from .hexutil import expect_sha_defect
    from .schema import CASE_COLUMNS, ReplayCase, ReplayMode, ReplayObservation, ReplaySource
except ImportError:  # python generate_corpus.py
    from corpus_io import CORPUS_DIR, HERE, write_shard
    from decide import CLAIM_ID, KIND, SCHEMA_VERSION, decide_observation
    from hexutil import expect_sha_defect
    from schema import CASE_COLUMNS, ReplayCase, ReplayMode, ReplayObservation, ReplaySource

DEFAULT_TARGET = 126000
SHARD_ROWS = 4200

FAMILIES: tuple[str, ...] = (
    "LABOR_ACCEPTED",
    "LABOR_REJECTED",
    "ATTEST_NOT_THIRD_PARTY",
    "PROSE_NOT_EVIDENCE",
    "NO_EXPECT_SHA",
    "INVALID_EXPECT_SHA",
    "TOOL_VERSION_MISSING",
    "TOOL_VERSION_MISMATCH",
    "NO_PLAN",
)

AGENCIES: tuple[str, ...] = (
    "과학기술정보통신부",
    "행정안전부",
    "기획재정부",
    "법무부",
    "교육부",
    "국방부",
    "보건복지부",
    "고용노동부",
    "국토교통부",
    "환경부",
    "산업통상자원부",
    "중소벤처기업부",
    "문화체육관광부",
    "농림축산식품부",
    "해양수산부",
    "여성가족부",
    "통일부",
    "외교부",
    "국가보훈부",
    "인사혁신처",
    "국무조정실",
    "감사원",
    "공정거래위원회",
    "금융위원회",
    "개인정보보호위원회",
    "서울특별시",
    "부산광역시",
    "대구광역시",
    "인천광역시",
    "광주광역시",
    "경기도",
    "강원특별자치도",
    "충청북도",
    "전북특별자치도",
    "경상남도",
    "제주특별자치도",
    "서울중앙지방법원",
    "특허법원",
    "헌법재판소",
    "대검찰청",
    "국세청",
    "조달청",
    "통계청",
    "특허청",
    "소방청",
    "질병관리청",
    "한국지능정보사회진흥원",
    "한국연구재단",
    "국민건강보험공단",
    "한국토지주택공사",
)

DOC_TYPES: tuple[str, ...] = (
    "과업지시서",
    "제안요청서",
    "입찰공고",
    "계약서",
    "일반기안문",
    "시행문",
    "훈령",
    "예규",
    "고시",
    "공고",
    "지침",
    "규정",
    "예산요구서",
    "사업계획서",
    "결과보고서",
    "감사보고서",
    "회의록",
    "출장복명서",
    "민원회신",
    "질의회신",
    "유권해석",
    "판결문",
    "결정문",
    "보도자료",
    "입법예고",
)

ACTIONS: tuple[str, ...] = (
    "replace_text",
    "fill_fields",
    "set_cell",
    "set_checkbox",
    "redact",
    "sanitize",
    "replace_table_label",
    "set_header_text",
)

FINDS: tuple[str, ...] = (
    "2025년 1차",
    "3월 31일",
    "과학기술정보통신부",
    "붙임 1. 과업지시서",
    "수신자 내부결재",
    "금일천만원정",
    "사업기간 2025.01.01",
    "담당 홍길동",
    "제3조 필수기능",
    "별표 2 제출서류",
    "본 계약은 2024년에",
    "비밀유지 의무",
    "개인정보 수집·이용",
    "납기 2025-06-30",
    "검토 의견 없음",
    "한컴 공식 출력과 동일",
)

REPLACES: tuple[str, ...] = (
    "2026년 2차",
    "4월 15일",
    "행정안전부",
    "붙임 2. 산출내역서",
    "수신자 대외공문",
    "금이천만원정",
    "사업기간 2026.03.01",
    "담당 김민준",
    "제3조 선택기능",
    "별표 3 검수기준",
    "본 계약은 2026년에",
    "비밀유지 및 보안서약",
    "개인정보 파기 절차",
    "납기 2026-12-15",
    "검토 의견 조건부 가결",
    "제3자 재현 해시만 인정",
)

TOOL_VERSIONS: tuple[str, ...] = (
    "0.8.4",
    "0.8.3",
    "0.8.2",
    "0.8.1",
    "0.8.0",
    "0.7.15",
    "0.7.14",
    "0.7.13",
)

INVALID_SHAS: tuple[str, ...] = (
    "deadbeef",
    "0x" + "ab" * 32,
    "G" * 64,
    "abc",
    "AB" * 32,
    " " + "a" * 64,
    "a" * 63,
    "a" * 65,
    "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
    "not-a-sha256-digest",
)

PROSE_CLAIMS: tuple[str, ...] = (
    "로컬에서 재실행했고 성공했습니다. 해시 대조는 생략했습니다.",
    "작업이 완료되었습니다. 산출물이 맞다고 확신합니다.",
    "capsule 없이 편집을 끝냈으니 노동을 인정해 주십시오.",
    "attest 영수증만 발급했지만 제3자 검증과 같다고 봅니다.",
    "toolVersion 을 확인하지 않았지만 재현된 것으로 간주합니다.",
    "계획 원문을 pretty-print 한 뒤 돌려도 같은 작업입니다.",
    "구현자 산문만으로 재현율을 100% 로 적겠습니다.",
    "exit 3 이 났지만 도구 고장이므로 통과로 바꾸겠습니다.",
)


def sha256_hex(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def make_plan(index: int) -> tuple[str, str, str]:
    agency = AGENCIES[index % len(AGENCIES)]
    doc = DOC_TYPES[(index // len(AGENCIES)) % len(DOC_TYPES)]
    action = ACTIONS[(index // 7) % len(ACTIONS)]
    find = FINDS[index % len(FINDS)]
    replace = REPLACES[(index // 3) % len(REPLACES)]
    serial = index + 1
    ext = "hwpx" if index % 5 == 0 else "hwp"
    input_path = f"samples/{agency}/{doc}-{serial:06d}.{ext}"
    output_path = f"out/{agency}/{doc}-{serial:06d}-vreplay.{ext}"
    step: dict[str, object]
    if action == "replace_text":
        step = {"action": "replace_text", "find": find, "replace": replace}
    elif action == "fill_fields":
        step = {"action": "fill_fields", "values": {f"필드{index % 17}": replace}}
    elif action == "set_cell":
        step = {
            "action": "set_cell",
            "table": index % 4,
            "row": index % 12,
            "col": index % 8,
            "text": replace,
        }
    elif action == "set_checkbox":
        step = {"action": "set_checkbox", "name": f"동의{index % 9}", "checked": True}
    elif action == "redact":
        step = {"action": "redact", "find": find}
    elif action == "sanitize":
        step = {"action": "sanitize"}
    elif action == "replace_table_label":
        step = {"action": "replace_text", "find": f"표 {1 + index % 9}", "replace": find}
    else:
        step = {"action": "replace_text", "find": "머리글", "replace": replace}
    plan_obj = {
        "planVersion": "1.0",
        "input": input_path,
        "output": output_path,
        "steps": [step],
    }
    plan = json.dumps(plan_obj, ensure_ascii=False, separators=(",", ":"))
    return plan, input_path, action


def make_observation(index: int, family: str) -> ReplayObservation:
    plan, _, _ = make_plan(index)
    version = TOOL_VERSIONS[index % len(TOOL_VERSIONS)]
    output_sha = sha256_hex(f"output|{index}|{plan}")
    if family == "LABOR_ACCEPTED":
        source = ReplaySource.CAPSULE if index % 2 == 0 else ReplaySource.REPLAY
        return ReplayObservation(
            plan=plan,
            expect_sha=output_sha,
            reproduced=True,
            tool_version=version,
            mode=ReplayMode.VERIFY,
            source=source,
            expected_tool_version=version,
        )
    if family == "LABOR_REJECTED":
        source = ReplaySource.CAPSULE if index % 3 == 0 else ReplaySource.REPLAY
        return ReplayObservation(
            plan=plan,
            expect_sha=sha256_hex(f"wrong|{index}|{plan}"),
            reproduced=False,
            tool_version=version,
            mode=ReplayMode.VERIFY,
            source=source,
            expected_tool_version=version,
        )
    if family == "ATTEST_NOT_THIRD_PARTY":
        source = ReplaySource.CAPSULE if index % 2 else ReplaySource.REPLAY
        return ReplayObservation(
            plan=plan,
            expect_sha="",
            reproduced=None,
            tool_version=version,
            mode=ReplayMode.ATTEST,
            source=source,
            expected_tool_version=version,
        )
    if family == "PROSE_NOT_EVIDENCE":
        claim = PROSE_CLAIMS[index % len(PROSE_CLAIMS)]
        return ReplayObservation(
            plan=f"{claim} case={index:06d} {plan}",
            expect_sha="",
            reproduced=None,
            tool_version="",
            mode=ReplayMode.ABSENT,
            source=ReplaySource.PROSE,
        )
    if family == "NO_EXPECT_SHA":
        return ReplayObservation(
            plan=plan,
            expect_sha="",
            reproduced=None,
            tool_version=version,
            mode=ReplayMode.VERIFY,
            source=ReplaySource.REPLAY,
            expected_tool_version=version,
        )
    if family == "INVALID_EXPECT_SHA":
        bad = INVALID_SHAS[index % len(INVALID_SHAS)]
        if bad == "AB" * 32:
            # uppercase is normalized by CLI; force a true defect
            bad = "g" * 64
        return ReplayObservation(
            plan=plan,
            expect_sha=f"{bad}:{index:06d}" if ":" not in bad else bad + f"{index % 10}",
            reproduced=False,
            tool_version=version,
            mode=ReplayMode.VERIFY,
            source=ReplaySource.REPLAY,
            expected_tool_version=version,
        )
    if family == "TOOL_VERSION_MISSING":
        return ReplayObservation(
            plan=plan,
            expect_sha=output_sha,
            reproduced=True,
            tool_version="",
            mode=ReplayMode.VERIFY,
            source=ReplaySource.REPLAY,
        )
    if family == "TOOL_VERSION_MISMATCH":
        other = TOOL_VERSIONS[(index + 3) % len(TOOL_VERSIONS)]
        if other == version:
            other = "0.6.0"
        return ReplayObservation(
            plan=plan,
            expect_sha=output_sha,
            reproduced=True,
            tool_version=version,
            mode=ReplayMode.VERIFY,
            source=ReplaySource.REPLAY,
            expected_tool_version=other,
        )
    # NO_PLAN
    return ReplayObservation(
        plan="",
        expect_sha=sha256_hex(f"noplan|{index}"),
        reproduced=True,
        tool_version=version,
        mode=ReplayMode.VERIFY,
        source=ReplaySource.REPLAY,
        expected_tool_version=version,
    )


def implementer_claim_for(index: int, family: str) -> str:
    agency = AGENCIES[index % len(AGENCIES)]
    doc = DOC_TYPES[(index // len(AGENCIES)) % len(DOC_TYPES)]
    return (
        f"{agency} {doc} 건(#{index:06d})은 구현자가 끝났다고 썼다. "
        f"가족={family}. 이 문장은 판정에 쓰지 않는다."
    )


def make_case(index: int) -> ReplayCase:
    family = FAMILIES[index % len(FAMILIES)]
    obs = make_observation(index, family)
    decision = decide_observation(obs)
    if decision.verdict != family:
        raise RuntimeError(f"axis drift index={index} family={family} got={decision.verdict}")
    plan, input_path, action = make_plan(index)
    if family == "PROSE_NOT_EVIDENCE":
        plan = obs.plan
        action = "prose"
        input_path = ""
    elif family == "NO_PLAN":
        plan = ""
        action = "missing"
        input_path = ""
    plan_sha = sha256_hex(obs.plan) if obs.plan else ""
    output_sha = sha256_hex(f"output|{index}|{obs.plan or plan}")
    return ReplayCase(
        case_id=f"v-replay-{index + 1:06d}",
        plan=obs.plan,
        expect_sha=obs.expect_sha,
        reproduced=obs.reproduced,
        tool_version=obs.tool_version,
        verdict=decision.verdict,
        mode=obs.mode.value,
        source=obs.source.value,
        plan_sha256=plan_sha,
        output_sha256=output_sha,
        expected_tool_version=obs.expected_tool_version,
        input_path=input_path,
        action=action,
        exit_class=decision.exit_class,
        labor_accepted=decision.labor_accepted,
        evidence_kind=decision.evidence_kind,
        implementer_claim=implementer_claim_for(index, family),
        sha_defect=expect_sha_defect(obs.expect_sha) or "",
        family=family,
    )


def generate(target: int, shard_rows: int, out_dir: Path) -> dict:
    out_dir.mkdir(parents=True, exist_ok=True)
    for stale in out_dir.glob("shard_*.tsv"):
        stale.unlink()

    by_verdict: dict[str, int] = {}
    shards: list[dict] = []
    keys: set[tuple] = set()
    chunk: list[ReplayCase] = []
    shard_index = 0

    for index in range(target):
        case = make_case(index)
        key = case.identity_key()
        if key in keys:
            raise RuntimeError(f"duplicate identity {case.case_id}")
        keys.add(key)
        by_verdict[case.verdict] = by_verdict.get(case.verdict, 0) + 1
        chunk.append(case)
        if len(chunk) >= shard_rows or index + 1 == target:
            name = f"shard_{shard_index:04d}.tsv"
            write_shard(out_dir / name, chunk)
            counts: dict[str, int] = {}
            for item in chunk:
                counts[item.verdict] = counts.get(item.verdict, 0) + 1
            shards.append(
                {
                    "path": f"corpus/{name}",
                    "rows": len(chunk),
                    "first": chunk[0].case_id,
                    "last": chunk[-1].case_id,
                    "byVerdict": dict(sorted(counts.items())),
                }
            )
            shard_index += 1
            chunk = []

    manifest = {
        "schemaVersion": SCHEMA_VERSION,
        "claim": CLAIM_ID,
        "kind": KIND,
        "axis": "third-party-replay",
        "rowCount": target,
        "shardRows": shard_rows,
        "columns": list(CASE_COLUMNS),
        "verdicts": list(FAMILIES),
        "uniqueness": "plan,expect_sha,reproduced,toolVersion,verdict",
        "byVerdict": dict(sorted(by_verdict.items())),
        "shards": shards,
        "notes": [
            "Implementer prose is stored in implementer_claim and is not an input to decide().",
            "Only replay/capsule reproduced and expect-output-sha256 fields decide labor.",
            "Rows are distinct axis tuples. Comment padding is not used.",
        ],
    }
    (out_dir / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    return manifest


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", type=int, default=DEFAULT_TARGET)
    parser.add_argument("--shard-rows", type=int, default=SHARD_ROWS)
    parser.add_argument("--out", type=Path, default=CORPUS_DIR)
    args = parser.parse_args(argv)
    manifest = generate(args.target, args.shard_rows, args.out)
    print(
        json.dumps(
            {
                "rowCount": manifest["rowCount"],
                "shards": len(manifest["shards"]),
                "byVerdict": manifest["byVerdict"],
                "out": str(args.out),
            },
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
