#!/usr/bin/env python3
"""Emit the committed V-lineage hash-chain corpus.

Each row is a distinct
``(parent_out, child_in, parentOk, lineageOk, brokenAt, verdict)`` case.
Comment padding is not used. Hashes are 64-hex tokens (or closed defects).
`reproduced` is stored only to prove decide() ignores V-replay.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

try:
    from .corpus_io import CORPUS_DIR, write_shard
    from .decide import CLAIM_ID, KIND, SCHEMA_VERSION, decide_observation
    from .hexutil import sha_defect
    from .schema import (
        CASE_COLUMNS,
        LineageCase,
        LineageObservation,
        LineageSource,
        ParentState,
    )
except ImportError:  # python generate_corpus.py
    from corpus_io import CORPUS_DIR, write_shard
    from decide import CLAIM_ID, KIND, SCHEMA_VERSION, decide_observation
    from hexutil import sha_defect
    from schema import (
        CASE_COLUMNS,
        LineageCase,
        LineageObservation,
        LineageSource,
        ParentState,
    )

DEFAULT_TARGET = 126000
SHARD_ROWS = 4200

FAMILIES: tuple[str, ...] = (
    "CHAIN_ACCEPTED",
    "LINEAGE_BROKEN",
    "PARENT_TAMPERED",
    "ROOT_ONLY",
    "PROSE_NOT_EVIDENCE",
    "HEAD_MISSING",
    "USAGE",
    "PARENT_SHA_MISSING",
    "PARENT_FIELD_MISSING",
    "KIND_NOT_CAPSULE",
    "HASH_DEFECT",
    "ENVELOPE_CONTRADICTS",
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

PROSE_CLAIMS: tuple[str, ...] = (
    "작업 사슬이 이어졌다고 봅니다. 해시는 대조하지 않았습니다.",
    "부모 산출과 자식 입력이 같을 것이라고 확신합니다.",
    "capsule 을 포맷터로 저장했지만 같은 작업입니다.",
    "lineage 봉투 없이 산문으로 연대기를 인정해 주십시오.",
    "parentOk 를 확인하지 않았지만 체인이 맞습니다.",
    "brokenAt 이 비었으니 통과로 적겠습니다.",
    "제3자 재실행 성공이 곧 계보 성공입니다.",
    "exit 3 이 났지만 도구 고장이므로 유효로 바꾸겠습니다.",
)

CONTRADICTION_KINDS: tuple[str, ...] = (
    "ok_true_hashes_differ",
    "ok_false_hashes_equal",
    "break_without_broken_at",
    "valid_has_broken_at",
    "tamper_without_broken_at",
    "root_has_broken_at",
    "parent_ok_true_lineage_null",
    "hashes_differ_lineage_null",
)

HASH_DEFECT_KINDS: tuple[str, ...] = (
    "short",
    "long",
    "nonhex",
    "prefixed",
    "whitespace",
    "one_missing",
    "claim_without_hashes",
)


def sha256_hex(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def agency_of(index: int) -> str:
    return AGENCIES[index % len(AGENCIES)]


def doc_of(index: int) -> str:
    return DOC_TYPES[(index // len(AGENCIES)) % len(DOC_TYPES)]


def action_of(index: int) -> str:
    return ACTIONS[(index // 7) % len(ACTIONS)]


def capsule_name(index: int, role: str) -> str:
    agency = agency_of(index)
    doc = doc_of(index)
    return f"capsules/{agency}/{doc}-{index + 1:06d}-{role}.capsule.json"


def bad_sha(index: int, kind: str) -> str:
    seed = sha256_hex(f"v-lineage|bad|{kind}|{index}")
    if kind == "short":
        return f"dead{index:08x}"
    if kind == "long":
        return seed + f"{index % 16:x}"
    if kind == "nonhex":
        return f"g{index:063d}"
    if kind == "prefixed":
        return "0x" + seed
    if kind == "whitespace":
        return " " + seed
    return f"not-hex-{index:06d}"


def make_observation(index: int, family: str) -> LineageObservation:
    parent_hash = sha256_hex(f"v-lineage|parent-out|{index}|{agency_of(index)}|{doc_of(index)}")
    child_same = parent_hash
    child_other = sha256_hex(f"v-lineage|child-in|{index}|{agency_of(index)}|{doc_of(index)}")
    broken = capsule_name(index, "broken")

    if family == "CHAIN_ACCEPTED":
        source = LineageSource.CAPSULE if index % 2 == 0 else LineageSource.LINEAGE
        kind = "workCapsule" if source is LineageSource.CAPSULE else "lineage"
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_same,
            parent_ok=True,
            lineage_ok=True,
            broken_at="",
            source=source,
            kind=kind,
            parent_state=ParentState.OK,
            valid=True,
            reproduced=True if index % 5 == 0 else None,
        )
    if family == "LINEAGE_BROKEN":
        source = LineageSource.CAPSULE if index % 3 == 0 else LineageSource.LINEAGE
        kind = "workCapsule" if source is LineageSource.CAPSULE else "lineage"
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_other,
            parent_ok=True,
            lineage_ok=False,
            broken_at=broken,
            source=source,
            kind=kind,
            parent_state=ParentState.OK,
            valid=False,
            reproduced=False if index % 7 == 0 else None,
        )
    if family == "PARENT_TAMPERED":
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_same,
            parent_ok=False,
            lineage_ok=True,
            broken_at=capsule_name(index, "parent"),
            source=LineageSource.LINEAGE,
            kind="lineage",
            parent_state=ParentState.OK,
            valid=False,
        )
    if family == "ROOT_ONLY":
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_other,
            parent_ok=None,
            lineage_ok=None,
            broken_at="",
            source=LineageSource.LINEAGE,
            kind="lineage",
            parent_state=ParentState.ROOT,
            valid=True,
        )
    if family == "PROSE_NOT_EVIDENCE":
        claim = PROSE_CLAIMS[index % len(PROSE_CLAIMS)]
        return LineageObservation(
            parent_out=sha256_hex(f"v-lineage|prose-p|{index}|{claim}"),
            child_in=sha256_hex(f"v-lineage|prose-c|{index}|{claim}"),
            parent_ok=None,
            lineage_ok=None,
            broken_at="",
            source=LineageSource.PROSE,
            kind="",
            parent_state=ParentState.ABSENT,
        )
    if family == "HEAD_MISSING":
        return LineageObservation(
            parent_out=sha256_hex(f"v-lineage|io-p|{index}"),
            child_in=sha256_hex(f"v-lineage|io-c|{index}"),
            parent_ok=None,
            lineage_ok=None,
            broken_at="",
            source=LineageSource.IO,
            kind="",
            parent_state=ParentState.ABSENT,
        )
    if family == "USAGE":
        return LineageObservation(
            parent_out=sha256_hex(f"v-lineage|usage-p|{index}"),
            child_in=sha256_hex(f"v-lineage|usage-c|{index}"),
            parent_ok=None,
            lineage_ok=None,
            broken_at="",
            source=LineageSource.USAGE,
            kind="",
            parent_state=ParentState.ABSENT,
        )
    if family == "PARENT_SHA_MISSING":
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_same,
            parent_ok=True,
            lineage_ok=True,
            broken_at=broken,
            source=LineageSource.LINEAGE,
            kind="lineage",
            parent_state=ParentState.SHA_MISSING,
            valid=False,
        )
    if family == "PARENT_FIELD_MISSING":
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_same,
            parent_ok=True,
            lineage_ok=True,
            broken_at=broken,
            source=LineageSource.CAPSULE,
            kind="workCapsule",
            parent_state=ParentState.FIELD_MISSING,
            valid=False,
        )
    if family == "KIND_NOT_CAPSULE":
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_same,
            parent_ok=True,
            lineage_ok=True,
            broken_at=broken,
            source=LineageSource.CAPSULE,
            kind=f"note-{index % 17}",
            parent_state=ParentState.OK,
        )
    if family == "HASH_DEFECT":
        defect_kind = HASH_DEFECT_KINDS[index % len(HASH_DEFECT_KINDS)]
        if defect_kind == "one_missing":
            return LineageObservation(
                parent_out=parent_hash,
                child_in="",
                parent_ok=True,
                lineage_ok=True,
                broken_at="",
                source=LineageSource.LINEAGE,
                kind="lineage",
                parent_state=ParentState.OK,
            )
        if defect_kind == "claim_without_hashes":
            return LineageObservation(
                parent_out="",
                child_in="",
                parent_ok=True,
                lineage_ok=True,
                broken_at=capsule_name(index, "nohash"),
                source=LineageSource.LINEAGE,
                kind="lineage",
                parent_state=ParentState.OK,
            )
        bad = bad_sha(index, defect_kind)
        return LineageObservation(
            parent_out=bad,
            child_in=parent_hash if defect_kind != "nonhex" else bad_sha(index + 99, "nonhex"),
            parent_ok=True,
            lineage_ok=True,
            broken_at="",
            source=LineageSource.LINEAGE,
            kind="lineage",
            parent_state=ParentState.OK,
        )
    # ENVELOPE_CONTRADICTS
    contra = CONTRADICTION_KINDS[index % len(CONTRADICTION_KINDS)]
    if contra == "ok_true_hashes_differ":
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_other,
            parent_ok=True,
            lineage_ok=True,
            broken_at=broken,
            source=LineageSource.LINEAGE,
            kind="lineage",
            parent_state=ParentState.OK,
            valid=True,
        )
    if contra == "ok_false_hashes_equal":
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_same,
            parent_ok=True,
            lineage_ok=False,
            broken_at=broken,
            source=LineageSource.LINEAGE,
            kind="lineage",
            parent_state=ParentState.OK,
            valid=False,
        )
    if contra == "break_without_broken_at":
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_other,
            parent_ok=True,
            lineage_ok=False,
            broken_at="",
            source=LineageSource.LINEAGE,
            kind="lineage",
            parent_state=ParentState.OK,
            valid=False,
        )
    if contra == "valid_has_broken_at":
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_same,
            parent_ok=True,
            lineage_ok=True,
            broken_at=broken,
            source=LineageSource.LINEAGE,
            kind="lineage",
            parent_state=ParentState.OK,
            valid=True,
        )
    if contra == "tamper_without_broken_at":
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_same,
            parent_ok=False,
            lineage_ok=True,
            broken_at="",
            source=LineageSource.LINEAGE,
            kind="lineage",
            parent_state=ParentState.OK,
            valid=False,
        )
    if contra == "root_has_broken_at":
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_other,
            parent_ok=None,
            lineage_ok=None,
            broken_at=broken,
            source=LineageSource.LINEAGE,
            kind="lineage",
            parent_state=ParentState.ROOT,
            valid=False,
        )
    if contra == "parent_ok_true_lineage_null":
        return LineageObservation(
            parent_out=parent_hash,
            child_in=child_same,
            parent_ok=True,
            lineage_ok=None,
            broken_at="",
            source=LineageSource.LINEAGE,
            kind="lineage",
            parent_state=ParentState.OK,
        )
    return LineageObservation(
        parent_out=parent_hash,
        child_in=child_other,
        parent_ok=True,
        lineage_ok=None,
        broken_at=broken,
        source=LineageSource.LINEAGE,
        kind="lineage",
        parent_state=ParentState.OK,
    )


def implementer_claim_for(index: int, family: str) -> str:
    agency = agency_of(index)
    doc = doc_of(index)
    return (
        f"{agency} {doc} 사슬(#{index:06d})은 구현자가 이어졌다고 썼다. "
        f"가족={family}. 이 문장은 판정에 쓰지 않는다. "
        f"연대기의 정의는 부모 산출 해시가 자식 입력 해시와 같은 것이다."
    )


def hash_defect_cell(obs: LineageObservation) -> str:
    if not str(obs.parent_out).strip() and not str(obs.child_in).strip():
        return "missing"
    left = sha_defect(obs.parent_out)
    right = sha_defect(obs.child_in)
    if left is None and right is None:
        return ""
    return left or right or ""


def make_case(index: int) -> LineageCase:
    family = FAMILIES[index % len(FAMILIES)]
    obs = make_observation(index, family)
    decision = decide_observation(obs)
    if decision.verdict != family:
        raise RuntimeError(f"axis drift index={index} family={family} got={decision.verdict}")
    return LineageCase(
        case_id=f"v-lineage-{index + 1:06d}",
        parent_out=obs.parent_out,
        child_in=obs.child_in,
        parent_ok=obs.parent_ok,
        lineage_ok=obs.lineage_ok,
        broken_at=obs.broken_at,
        verdict=decision.verdict,
        source=obs.source.value,
        kind=obs.kind,
        parent_state=obs.parent_state.value,
        valid=obs.valid,
        reproduced=obs.reproduced,
        exit_class=decision.exit_class,
        chain_accepted=decision.chain_accepted,
        evidence_kind=decision.evidence_kind,
        head=capsule_name(index, "head"),
        child_capsule=capsule_name(index, "child"),
        parent_capsule=capsule_name(index, "parent"),
        depth="1" if family == "ROOT_ONLY" else "2",
        agency=agency_of(index),
        doc_type=doc_of(index),
        action=action_of(index),
        implementer_claim=implementer_claim_for(index, family),
        family=family,
        hash_defect=hash_defect_cell(obs),
    )


def generate(target: int, shard_rows: int, out_dir: Path) -> dict:
    out_dir.mkdir(parents=True, exist_ok=True)
    for stale in out_dir.glob("shard_*.tsv"):
        stale.unlink()

    by_verdict: dict[str, int] = {}
    shards: list[dict] = []
    keys: set[tuple] = set()
    chunk: list[LineageCase] = []
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
        "axis": "lineage-chain",
        "rowCount": target,
        "shardRows": shard_rows,
        "columns": list(CASE_COLUMNS),
        "verdicts": list(FAMILIES),
        "uniqueness": "parent_out,child_in,parentOk,lineageOk,brokenAt,verdict",
        "byVerdict": dict(sorted(by_verdict.items())),
        "shards": shards,
        "notes": [
            "Implementer prose is stored in implementer_claim and is not an input to decide().",
            "parent outputSha256 must equal child inputSha256. That equality is lineageOk.",
            "reproduced/--deep is V-replay and is ignored by decide().",
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
