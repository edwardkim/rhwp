#!/usr/bin/env python3
"""Emit the V-step process-reward corpus.

Each record is one existing edit step plus four existing mechanical checks
(verify, layout-anomaly, pageCount via info, fill-fields --verify).
Process reward is pass/fail of those checks — not Best-of-N ranking.
Does not invent a rhwp CLI.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CORPUS = ROOT / "corpus"
SHARDS = CORPUS / "shards"

SCHEMA_VERSION = "v-step.1.0"

AGENCIES = [
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
    "산업통상자원부",
    "여성가족부",
    "통일부",
    "국가보훈부",
]

KINDS = [
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
    "시행규칙",
    "고시별표",
]

YEARS = list(range(2018, 2026))

STEP_KINDS = [
    "fill-fields",
    "replace-text",
    "delete-text",
    "insert-text",
    "redact",
    "sanitize",
    "csv-to-table",
    "insert-table",
    "delete-row",
    "apply-char-format",
]

FIELD_NAMES = [
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
]

TITLES = [
    "규제 심사(안) 개요",
    "개인정보 처리방침 개정 고시",
    "지방세 감면 신청서",
    "공사계약 일반조건",
    "정보공개 청구서",
    "출장여비 정산서",
    "예산 전용 요구서",
    "민원 처리 결과 통지",
    "감사 지적사항 조치 결과",
    "회의 결과 보고",
    "보안 점검 체크리스트",
    "누름틀 일괄 채움 점검",
]


def pick_ext(i: int) -> str:
    return "hwpx" if i % 3 == 0 else "hwp"


def source_path(agency: str, kind: str, year: int, seq: int, ext: str) -> str:
    return f"samples/gov/{agency}_{kind}_{year}_{seq:04d}.{ext}"


def output_path(src: str, step_index: int) -> str:
    stem, ext = src.rsplit(".", 1)
    return stem.replace("samples/", "output/") + f"_s{step_index}.{ext}"


def empty_fields() -> dict:
    return {
        "verdict": None,
        "failCount": None,
        "passCount": None,
        "hasSignal": None,
        "strict": None,
        "overflowCount": None,
        "overlapCount": None,
        "emptyPageCount": None,
        "pageCount": None,
        "expectedPageCount": None,
        "pageCountMismatch": None,
        "verifyIdentical": None,
        "verifyDiffCount": None,
        "filledCount": None,
        "notFoundCount": None,
        "identical": None,
    }


def merge_fields(extra: dict | None) -> dict:
    out = empty_fields()
    if extra:
        out.update(extra)
    return out


def page_counts_differ(fields: dict) -> bool:
    a = fields.get("pageCount")
    e = fields.get("expectedPageCount")
    return a is not None and e is not None and a != e


def fail_signals(check: str, fields: dict) -> list[str]:
    out: list[str] = []
    if check == "verify":
        if (fields.get("failCount") or 0) > 0:
            out.append("failCount>0")
        if fields.get("verdict") in ("fail", "FAIL", "invalid", "mismatch"):
            out.append("verdict=fail")
    elif check == "layout-anomaly":
        strict = fields.get("strict") is True
        overflow = (fields.get("overflowCount") or 0) > 0
        overlap = (fields.get("overlapCount") or 0) > 0
        if strict and (overflow or overlap):
            out.append("layout-strict-signal")
            if fields.get("hasSignal") is True:
                out.append("hasSignal=true")
    elif check == "pageCount":
        if fields.get("pageCountMismatch") is True:
            out.append("pageCountMismatch=true")
        if page_counts_differ(fields):
            out.append("pageCount!=expected")
    elif check == "fill-verify":
        if fields.get("verifyIdentical") is False:
            out.append("verify.identical=false")
        if (fields.get("verifyDiffCount") or 0) > 0 and fields.get("verifyIdentical") is not True:
            out.append("verify.diffCount>0")
    return out


def layout_strict_fail(fields: dict) -> bool:
    return fields.get("strict") is True and (
        (fields.get("overflowCount") or 0) > 0 or (fields.get("overlapCount") or 0) > 0
    )


def explicit_success(check: str, fields: dict) -> bool:
    if check == "verify":
        return fields.get("verdict") == "pass" and fields.get("failCount") == 0
    if check == "layout-anomaly":
        return (
            fields.get("hasSignal") is False
            and (fields.get("overflowCount") or 0) == 0
            and (fields.get("overlapCount") or 0) == 0
        )
    if check == "pageCount":
        return (
            fields.get("pageCountMismatch") is False
            and not page_counts_differ(fields)
            and fields.get("pageCount") is not None
        )
    if check == "fill-verify":
        return fields.get("verifyIdentical") is True and fields.get("verifyDiffCount") == 0
    return False


def score_check(check: str, exit_class: int, has_envelope: bool, fields: dict) -> tuple[bool, bool, str]:
    if exit_class == 1:
        return False, True, "exit.io"
    if exit_class == 2:
        return False, True, "exit.usage"
    if exit_class == 4:
        if not has_envelope:
            return False, False, "page_verify.missing-envelope"
        if (
            fields.get("pageCountMismatch") is False
            and fields.get("verifyIdentical") is not False
            and (fields.get("failCount") or 0) == 0
            and not page_counts_differ(fields)
        ):
            return False, False, "page_verify.exit4-but-ok-fields"
        return False, True, "exit.page_verify"
    if exit_class == 0:
        if not has_envelope:
            return False, False, "ok.missing-envelope"
        if fail_signals(check, fields):
            return False, False, "ok.fail-signal-present"
        if check == "verify" and isinstance(fields.get("verdict"), str) and fields["verdict"].lower() == "fail":
            return False, False, "ok.verify-fail-verdict"
        if check == "layout-anomaly" and layout_strict_fail(fields):
            return False, False, "ok.layout-strict-signal"
        if check == "pageCount" and page_counts_differ(fields):
            return False, False, "ok.page-count-differ"
        if check == "fill-verify" and fields.get("verifyIdentical") is False:
            return False, False, "ok.fill-verify-false"
        return True, True, "exit.ok"
    if exit_class == 3:
        if not has_envelope:
            return False, False, "judgment.missing-envelope"
        if explicit_success(check, fields):
            return False, False, "judgment.success-fields"
        return False, True, "exit.judgment"
    raise AssertionError(exit_class)


def score_step(checks: list[dict]) -> dict:
    pass_count = 0
    fail_count = 0
    failed: list[str] = []
    worst = 0
    consistent = True
    for c in checks:
        ok, cons, _rule = score_check(c["check"], c["exitClass"], c.get("envelope") is not None, c["fields"])
        c["pass"] = ok
        c["failSignals"] = fail_signals(c["check"], c["fields"])
        if not cons:
            consistent = False
        if ok:
            pass_count += 1
        else:
            fail_count += 1
            failed.append(c["check"])
        worst = max(worst, c["exitClass"])
    return {
        "pass": fail_count == 0 and bool(checks) and consistent,
        "checkCount": len(checks),
        "passCount": pass_count,
        "failCount": fail_count,
        "failedChecks": failed,
        "worstExitClass": worst,
        "consistent": consistent,
    }


def verify_envelope(src: str, i: int, fail: bool, pages: int) -> dict:
    expect_pages = pages
    actual_pages = pages + (2 if fail and i % 2 == 0 else 0)
    expect_chars = 80 + (i % 400)
    actual_chars = expect_chars - (15 if fail and i % 2 == 1 else 0)
    expectations = [
        {
            "kind": "pages",
            "expected": expect_pages,
            "actual": actual_pages,
            "pass": actual_pages == expect_pages,
        },
        {
            "kind": "min-chars",
            "expected": expect_chars,
            "actual": actual_chars,
            "pass": actual_chars >= expect_chars,
        },
        {
            "kind": "min-tables",
            "expected": 1,
            "actual": 1 + (i % 5),
            "pass": True,
        },
        {
            "kind": "contains",
            "expected": FIELD_NAMES[i % len(FIELD_NAMES)],
            "actual": FIELD_NAMES[i % len(FIELD_NAMES)],
            "pass": not fail or i % 2 == 0,
        },
        {
            "kind": "format",
            "expected": "hwp5",
            "actual": "hwp5" if i % 3 else "hwpx",
            "pass": (i % 3) != 0 or not fail,
        },
    ]
    fail_count = sum(1 for e in expectations if not e["pass"])
    return {
        "schemaVersion": "1.0",
        "source": src,
        "pageCount": actual_pages,
        "expectations": expectations,
        "passCount": len(expectations) - fail_count,
        "failCount": fail_count,
        "verdict": "fail" if fail_count else "pass",
        "untrustedContent": True,
        "untrustedFields": ["expectations[].actual"],
    }


def drop_nones(value):
    if isinstance(value, dict):
        return {k: drop_nones(v) for k, v in value.items() if v is not None}
    if isinstance(value, list):
        return [drop_nones(v) for v in value]
    return value


def layout_envelope(
    src: str, i: int, overflow: int, overlap: int, empty: int, strict: bool
) -> dict:
    n = 2
    pages = []
    for p in range(n):
        pages.append(
            {
                "page": p,
                "widthPx": 794,
                "heightPx": 1123,
                "overflow": overflow > 0 and p == 0,
                "overlap": overlap > 0 and p == (1 % n),
                "emptyPage": empty > 0 and p == n - 1 and n > 2,
                "overflowItems": (
                    [
                        {
                            "kind": "para",
                            "section": 0,
                            "paragraph": 2 + p,
                            "dx": 4.5 + (i % 9),
                            "dy": 0.0,
                            "edge": "right",
                        }
                    ]
                    if overflow > 0 and p == 0
                    else []
                ),
                "overlapItems": (
                    [
                        {
                            "kind": "text-text",
                            "a": {"section": 0, "paragraph": 4},
                            "b": {"section": 0, "paragraph": 5},
                            "overlapPx": 3.0 + (i % 5),
                        }
                    ]
                    if overlap > 0 and p == (1 % n)
                    else []
                ),
            }
        )
    return {
        "schemaVersion": "1.0",
        "source": src,
        "pageCount": n,
        "pageFilter": None,
        "overflowTolerancePx": 1.0,
        "overlapTolerancePx": 1.0,
        "strict": strict,
        "overflowCount": overflow,
        "overlapCount": overlap,
        "emptyPageCount": empty,
        "hasSignal": (overflow + overlap + empty) > 0,
        "pages": pages,
        "untrustedContent": False,
        "untrustedFields": [],
    }


def info_pagecount_envelope(src: str, i: int, actual: int, expected: int) -> dict:
    return {
        "schemaVersion": "1.0",
        "source": src,
        "format": "hwpx" if src.endswith(".hwpx") else "hwp5",
        "sizeBytes": 24000 + (i * 137) % 900000,
        "version": "owpml-1.0" if src.endswith(".hwpx") else "5.0.2.5",
        "sections": 1 + (i % 4),
        "pageCount": actual,
        "expectedPageCount": expected,
        "pageCountMismatch": actual != expected,
        "paraCount": 3 + (i % 220),
        "title": TITLES[i % len(TITLES)],
        "untrustedContent": True,
        "untrustedFields": ["title"],
    }


def fill_verify_envelope(src: str, i: int, ok: bool, not_found: bool) -> dict:
    n = 2 + (i % 2)
    filled = []
    for k in range(n):
        name = FIELD_NAMES[(i + k) % len(FIELD_NAMES)]
        filled.append(
            {
                "name": name,
                "occurrence": k,
                "value": f"{name}-값-{i:04d}-{k}",
            }
        )
    missing = [f"없는필드{i % 9}"] if not_found else []
    return {
        "schemaVersion": "1.0",
        "source": src,
        "dryRun": False,
        "filledCount": len(filled),
        "filled": filled,
        "notFound": missing,
        "ambiguous": [],
        "confusable": [],
        "verify": {
            "identical": ok,
            "diffCount": 0 if ok else 1 + (i % 6),
        },
        "untrustedContent": True,
        "untrustedFields": ["filled[].value", "filled[].name"],
    }


def fields_from_verify(env: dict | None) -> dict:
    if not env:
        return empty_fields()
    return merge_fields(
        {
            "verdict": env.get("verdict"),
            "failCount": env.get("failCount"),
            "passCount": env.get("passCount"),
            "pageCount": env.get("pageCount"),
        }
    )


def fields_from_layout(env: dict | None) -> dict:
    if not env:
        return empty_fields()
    return merge_fields(
        {
            "hasSignal": env.get("hasSignal"),
            "strict": env.get("strict"),
            "overflowCount": env.get("overflowCount"),
            "overlapCount": env.get("overlapCount"),
            "emptyPageCount": env.get("emptyPageCount"),
            "pageCount": env.get("pageCount"),
        }
    )


def fields_from_pagecount(env: dict | None) -> dict:
    if not env:
        return empty_fields()
    return merge_fields(
        {
            "pageCount": env.get("pageCount"),
            "expectedPageCount": env.get("expectedPageCount"),
            "pageCountMismatch": env.get("pageCountMismatch"),
        }
    )


def fields_from_fill(env: dict | None) -> dict:
    if not env:
        return empty_fields()
    verify = env.get("verify") or {}
    return merge_fields(
        {
            "verifyIdentical": verify.get("identical"),
            "verifyDiffCount": verify.get("diffCount"),
            "filledCount": env.get("filledCount"),
            "notFoundCount": len(env.get("notFound") or []),
        }
    )


def make_check(check: str, argv: list[str], exit_class: int, env: dict | None, fields: dict) -> dict:
    return {
        "check": check,
        "argv": argv,
        "exitClass": exit_class,
        "pass": False,
        "failSignals": [],
        "envelope": env,
        "fields": fields,
    }


def argv_for_step(kind: str, src: str, out: str) -> list[str]:
    head = {
        "fill-fields": ["edit", "fill-fields", src, "--data", "@fields.json", "-o", out, "--verify"],
        "replace-text": ["edit", "replace-text", src, "--find", "임시", "--replace", "확정", "-o", out],
        "delete-text": ["edit", "delete-text", src, "--find", "삭제대상", "-o", out],
        "insert-text": ["edit", "insert-text", src, "--text", "삽입문", "-o", out],
        "redact": ["edit", "redact", src, "-o", out],
        "sanitize": ["edit", "sanitize", src, "-o", out],
        "csv-to-table": ["edit", "csv-to-table", src, "--csv", "@table.csv", "-o", out],
        "insert-table": ["edit", "insert-table", src, "--rows", "3", "--cols", "4", "-o", out],
        "delete-row": ["edit", "delete-row", src, "--table", "0", "--row", "1", "-o", out],
        "apply-char-format": ["edit", "apply-char-format", src, "--bold", "-o", out],
    }[kind]
    if "--json" not in head:
        head = head + ["--json"]
    return head


def lane_spec(lane: int, i: int) -> dict:
    """Consistent check outcomes. No rank / Best-of-N."""
    pages = 2 + (i % 18)
    if lane == 0:
        return {
            "name": "all-pass",
            "verify": ("ok", False, pages),
            "layout": ("ok", 0, 0, 0, False),
            "page": ("ok", pages, pages),
            "fill": ("ok", True, False),
            "edit_exit": 0,
        }
    if lane == 1:
        return {
            "name": "verify-fail",
            "verify": ("fail", True, pages),
            "layout": ("ok", 0, 0, 0, False),
            "page": ("ok", pages, pages),
            "fill": ("ok", True, False),
            "edit_exit": 0,
        }
    if lane == 2:
        return {
            "name": "layout-strict-overflow",
            "verify": ("ok", False, pages),
            "layout": ("fail", 2 + (i % 4), 0, 0, True),
            "page": ("ok", pages, pages),
            "fill": ("ok", True, False),
            "edit_exit": 0,
        }
    if lane == 3:
        return {
            "name": "pagecount-mismatch",
            "verify": ("ok", False, pages),
            "layout": ("ok", 0, 0, 0, True),
            "page": ("mismatch", pages + 1 + (i % 3), pages),
            "fill": ("ok", True, False),
            "edit_exit": 0,
        }
    if lane == 4:
        return {
            "name": "fill-verify-ir-diff",
            "verify": ("ok", False, pages),
            "layout": ("ok", 0, 0, 0, False),
            "page": ("ok", pages, pages),
            "fill": ("fail", False, False),
            "edit_exit": 3,
        }
    if lane == 5:
        return {
            "name": "verify-io",
            "verify": ("io", False, pages),
            "layout": ("ok", 0, 0, 0, False),
            "page": ("ok", pages, pages),
            "fill": ("ok", True, False),
            "edit_exit": 1,
        }
    if lane == 6:
        return {
            "name": "layout-usage",
            "verify": ("ok", False, pages),
            "layout": ("usage", 0, 0, 0, False),
            "page": ("ok", pages, pages),
            "fill": ("ok", True, False),
            "edit_exit": 2,
        }
    if lane == 7:
        return {
            "name": "verify-and-layout-fail",
            "verify": ("fail", True, pages),
            "layout": ("fail", 0, 3 + (i % 3), 0, True),
            "page": ("ok", pages, pages),
            "fill": ("ok", True, False),
            "edit_exit": 3,
        }
    if lane == 8:
        return {
            "name": "pagecount-and-fill-fail",
            "verify": ("ok", False, pages),
            "layout": ("ok", 0, 0, 1, True),
            "page": ("mismatch", pages + 2, pages),
            "fill": ("fail", False, True),
            "edit_exit": 4,
        }
    return {
        "name": "all-pass-empty-page",
        "verify": ("ok", False, pages),
        "layout": ("ok", 0, 0, 1 + (i % 2), True),
        "page": ("ok", pages, pages),
        "fill": ("ok", True, False),
        "edit_exit": 0,
    }


def build_checks(src: str, out: str, i: int, spec: dict) -> list[dict]:
    v_state, v_fail, pages = spec["verify"]
    if v_state == "io":
        v_env, v_exit = None, 1
    elif v_state == "usage":
        v_env, v_exit = None, 2
    elif v_state == "fail":
        v_env, v_exit = verify_envelope(out, i, True, pages), 3
    else:
        v_env, v_exit = verify_envelope(out, i, False, pages), 0

    l_state, ovf, ovl, empty, strict = spec["layout"]
    if l_state == "io":
        l_env, l_exit = None, 1
    elif l_state == "usage":
        l_env, l_exit = None, 2
    elif l_state == "fail":
        l_env, l_exit = layout_envelope(out, i, ovf, ovl, empty, True), 3
    else:
        l_env, l_exit = layout_envelope(out, i, ovf, ovl, empty, strict), 0

    p_state, actual, expected = spec["page"]
    if p_state == "io":
        p_env, p_exit = None, 1
    elif p_state == "mismatch":
        p_env, p_exit = info_pagecount_envelope(out, i, actual, expected), 4
    else:
        p_env, p_exit = info_pagecount_envelope(out, i, actual, expected), 0

    f_state, f_ok, f_missing = spec["fill"]
    if f_state == "io":
        f_env, f_exit = None, 1
    elif f_state == "fail":
        f_env, f_exit = fill_verify_envelope(out, i, False, f_missing), 3
    else:
        f_env, f_exit = fill_verify_envelope(out, i, True, f_missing), 0

    return [
        make_check(
            "verify",
            ["verify", out, "--expect-pages", str(pages), "--json"],
            v_exit,
            v_env,
            fields_from_verify(v_env),
        ),
        make_check(
            "layout-anomaly",
            (["layout-anomaly", out, "--strict", "--json"] if strict else ["layout-anomaly", out, "--json"]),
            l_exit,
            l_env,
            fields_from_layout(l_env),
        ),
        make_check(
            "pageCount",
            ["info", out, "--json"],
            p_exit,
            p_env,
            fields_from_pagecount(p_env),
        ),
        make_check(
            "fill-verify",
            ["edit", "fill-fields", src, "--data", "@fields.json", "-o", out, "--verify", "--json"],
            f_exit,
            f_env,
            fields_from_fill(f_env),
        ),
    ]


def build_record(
    idx: int,
    agency: str,
    kind: str,
    year: int,
    seq: int,
    step_index: int,
) -> dict:
    ext = pick_ext(idx)
    src = source_path(agency, kind, year, seq, ext)
    out = output_path(src, step_index)
    step_kind = STEP_KINDS[(idx + step_index) % len(STEP_KINDS)]
    lane = (idx + step_index * 3) % 10
    spec = lane_spec(lane, idx + step_index * 17)
    checks = build_checks(src, out, idx + step_index * 17, spec)
    reward = score_step(checks)
    source_tag = (
        f"{agency}/{kind}/{year}/{seq:04d}#{step_kind}/s{step_index}/{lane}/{spec['name']}"
    )
    return {
        "recordId": f"vstep-{idx + 1:06d}-s{step_index}",
        "episodeId": f"ep-{idx + 1:06d}",
        "sourceTag": source_tag,
        "stepIndex": step_index,
        "stepKind": step_kind,
        "source": src,
        "argv": argv_for_step(step_kind, src, out),
        "editExitClass": spec["edit_exit"],
        "checks": checks,
        "processReward": reward,
    }


def uniqueness_key(rec: dict) -> tuple:
    checks = []
    for c in rec["checks"]:
        checks.append(
            (
                c["check"],
                c["exitClass"],
                c["pass"],
                json.dumps(c["fields"], sort_keys=True, ensure_ascii=False),
            )
        )
    return (
        rec["stepKind"],
        rec["stepIndex"],
        tuple(checks),
        rec["processReward"]["pass"],
        tuple(rec["processReward"]["failedChecks"]),
        rec["sourceTag"],
    )


def generate_records() -> list[dict]:
    records = []
    idx = 0
    seq = 1
    for year in YEARS:
        for kind in KINDS:
            for agency in AGENCIES:
                for step_index in (0, 1):
                    records.append(build_record(idx, agency, kind, year, seq, step_index))
                idx += 1
                seq += 1
    return records


def write_shards(records: list[dict], per_shard: int = 24) -> list[dict]:
    if SHARDS.exists():
        for old in SHARDS.glob("*.json"):
            old.unlink()
    SHARDS.mkdir(parents=True, exist_ok=True)
    metas = []
    for start in range(0, len(records), per_shard):
        chunk = records[start : start + per_shard]
        shard_id = f"{start // per_shard:03d}"
        payload = {
            "schemaVersion": SCHEMA_VERSION,
            "shardId": shard_id,
            "records": chunk,
        }
        path = SHARDS / f"{shard_id}.json"
        text = json.dumps(drop_nones(payload), ensure_ascii=False, indent=2) + "\n"
        path.write_text(text, encoding="utf-8", newline="\n")
        metas.append(
            {
                "path": f"shards/{path.name}",
                "count": len(chunk),
                "stepKinds": sorted({r["stepKind"] for r in chunk}),
                "rewardPass": sum(1 for r in chunk if r["processReward"]["pass"]),
                "rewardFail": sum(1 for r in chunk if not r["processReward"]["pass"]),
            }
        )
    return metas


def write_manifest(records: list[dict], metas: list[dict]) -> None:
    kind_counts: dict[str, int] = {}
    pass_n = 0
    fail_n = 0
    for rec in records:
        kind_counts[rec["stepKind"]] = kind_counts.get(rec["stepKind"], 0) + 1
        if rec["processReward"]["pass"]:
            pass_n += 1
        else:
            fail_n += 1
    manifest = {
        "schemaVersion": SCHEMA_VERSION,
        "generatedBy": "tools/llm_verifier/process_steps/generate_corpus.py",
        "recordCount": len(records),
        "shardCount": len(metas),
        "stepKindCounts": kind_counts,
        "rewardPassCount": pass_n,
        "rewardFailCount": fail_n,
        "uniqueness": "stepKind+stepIndex+checkFingerprints+processReward+sourceTag",
        "shards": metas,
    }
    (CORPUS / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )


def count_lines() -> tuple[int, int]:
    shard_lines = 0
    for p in SHARDS.glob("*.json"):
        shard_lines += sum(1 for _ in p.open(encoding="utf-8"))
    manifest_lines = sum(1 for _ in (CORPUS / "manifest.json").open(encoding="utf-8"))
    return shard_lines, manifest_lines


def main() -> int:
    records = generate_records()
    keys = [uniqueness_key(r) for r in records]
    if len(keys) != len(set(keys)):
        print("duplicate uniqueness key", file=sys.stderr)
        seen = set()
        for k in keys:
            if k in seen:
                print(k, file=sys.stderr)
                return 1
            seen.add(k)
    metas = write_shards(records)
    write_manifest(records, metas)
    shard_lines, manifest_lines = count_lines()
    total = shard_lines + manifest_lines
    print(
        f"records={len(records)} shards={len(metas)} "
        f"shard_lines={shard_lines} manifest_lines={manifest_lines} total={total}"
    )
    if total < 100000:
        print(f"SIZE GATE FAIL: {total} < 100000", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
