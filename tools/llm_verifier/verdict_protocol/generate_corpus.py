#!/usr/bin/env python3
"""Emit the V-proto corpus of realistic rhwp-shaped JSON envelopes.

Each record is unique on (command, exitClass, judgment fields, sourceTag).
Does not invent a rhwp CLI — envelopes follow existing command families.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CORPUS = ROOT / "corpus"
SHARDS = CORPUS / "shards"

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
]

YEARS = list(range(2018, 2025))  # 7 years → 32*10*7 = 2240

COMMANDS = [
    "info",
    "verify",
    "ir-diff",
    "layout-anomaly",
    "replay",
    "fill-fields",
    "render-diff",
]

FONTS = [
    "함초롬바탕",
    "함초롬돋움",
    "맑은 고딕",
    "굴림",
    "굴림체",
    "돋움",
    "돋움체",
    "바탕",
    "바탕체",
    "나눔고딕",
    "나눔명조",
    "본고딕",
    "본명조",
    "HY신명조",
    "HY중고딕",
    "한양견고딕",
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
]


def sha256_hex(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def hex12(text: str) -> str:
    return sha256_hex(text)[:12]


def empty_judgment() -> dict:
    return {
        "identical": None,
        "hasSignal": None,
        "reproduced": None,
        "findingCount": None,
        "verify": None,
        "failCount": None,
        "passCount": None,
        "verdict": None,
        "regression": None,
        "status": None,
        "clean": None,
        "signalCount": None,
        "valid": None,
        "diffCount": None,
        "strict": None,
        "overflowCount": None,
        "overlapCount": None,
        "emptyPageCount": None,
        "pageCountMismatch": None,
        "overPages": None,
    }


def merge_judgment(extra: dict | None) -> dict:
    out = empty_judgment()
    if extra:
        for key, value in extra.items():
            out[key] = value
    return out


def source_path(agency: str, kind: str, year: int, seq: int, ext: str) -> str:
    return f"samples/gov/{agency}_{kind}_{year}_{seq:04d}.{ext}"


def pick_ext(i: int) -> str:
    return "hwpx" if i % 3 == 0 else "hwp"


def pick_format(ext: str) -> str:
    return "hwpx" if ext == "hwpx" else "hwp5"


def info_envelope(src: str, i: int, title: str, fmt: str) -> dict:
    font_n = 6 + (i % 8)
    warnings = []
    if i % 11 == 0:
        warnings.append(
            {
                "code": "legacyFont",
                "message": f"문서가 {FONTS[i % len(FONTS)]} 메트릭을 가정한다",
            }
        )
    if i % 17 == 0:
        warnings.append(
            {
                "code": "distribution",
                "message": "배포용 문서 — 편집 시 convert 가 필요하다",
            }
        )
    return {
        "schemaVersion": "1.0",
        "source": src,
        "format": fmt,
        "sizeBytes": 24000 + (i * 137) % 900000,
        "version": "5.0.2.5" if fmt == "hwp5" else "owpml-1.0",
        "sections": 1 + (i % 4),
        "pageCount": 1 + (i % 48),
        "paraCount": 3 + (i % 220),
        "fonts": [FONTS[(i + k) % len(FONTS)] for k in range(font_n)],
        "title": title,
        "warnings": warnings,
        "untrustedContent": True,
        "untrustedFields": ["title", "fonts[]"],
    }


def verify_envelope(src: str, i: int, fail: bool) -> dict:
    expect_pages = 1 + (i % 20)
    actual_pages = expect_pages + (2 if fail and i % 2 == 0 else 0)
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
            "pass": True,
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
        "expectations": expectations,
        "passCount": len(expectations) - fail_count,
        "failCount": fail_count,
        "verdict": "fail" if fail_count else "pass",
        "untrustedContent": True,
        "untrustedFields": ["expectations[].actual"],
    }


def ir_diff_envelope(src_a: str, src_b: str, i: int, identical: bool) -> dict:
    if identical:
        categories = {}
        diff_count = 0
    else:
        categories = {
            "text": 1 + (i % 6),
            "paraShape": i % 3,
            "charShape": i % 4,
            "table": i % 5,
            "ctrl": i % 2,
            "section": 1 if i % 7 == 0 else 0,
        }
        categories = {k: v for k, v in categories.items() if v}
        diff_count = sum(categories.values())
    return {
        "schemaVersion": "1.0",
        "a": src_a,
        "b": src_b,
        "identical": identical,
        "diffCount": diff_count,
        "categories": categories,
        "untrustedContent": bool(categories),
        "untrustedFields": ["categories"] if categories else [],
    }


def layout_pages(i: int, overflow: int, overlap: int, empty: int) -> list:
    pages = []
    n = 3 + (i % 6)
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
    return pages


def layout_envelope(src: str, i: int, overflow: int, overlap: int, empty: int, strict: bool) -> dict:
    pages = layout_pages(i, overflow, overlap, empty)
    return {
        "schemaVersion": "1.0",
        "source": src,
        "pageCount": len(pages),
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


def replay_envelope(src: str, i: int, mode: str, reproduced) -> dict:
    plan = sha256_hex(f"plan:{src}:{i}")
    out = sha256_hex(f"out:{src}:{i}:{mode}")
    inp = sha256_hex(f"in:{src}:{i}")
    expected = out if reproduced is True else (sha256_hex(f"exp:{src}:{i}") if mode == "verify" else None)
    return {
        "schemaVersion": "1.0",
        "mode": mode,
        "input": src,
        "inputSha256": inp,
        "planSha256": plan,
        "outputSha256": out,
        "expectedOutputSha256": expected,
        "reproduced": reproduced,
        "steps": 1 + (i % 5),
        "toolVersion": "0.8.4",
        "untrustedContent": False,
        "untrustedFields": [],
    }


def fill_envelope(src: str, i: int, dry: bool, verify_ok: bool | None, not_found: bool) -> dict:
    n = 2 + (i % 6)
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
    env = {
        "schemaVersion": "1.0",
        "source": src,
        "dryRun": dry,
        "filledCount": 0 if not_found and n == 0 else len(filled),
        "filled": filled,
        "notFound": missing,
        "ambiguous": [],
        "confusable": [],
        "changedPages": None if dry else [0, 1][: 1 + (i % 2)],
        "untrustedContent": True,
        "untrustedFields": ["filled[].value", "filled[].name"],
    }
    if not dry:
        env["output"] = src.replace("samples/", "output/").replace(".hwp", "_filled.hwp")
        env["outputFormat"] = "hwpx" if src.endswith(".hwpx") else "hwp5"
    if verify_ok is not None:
        env["verify"] = {
            "identical": verify_ok,
            "diffCount": 0 if verify_ok else 1 + (i % 6),
        }
    return env


def render_pages(i: int, over: int, struct: int) -> list:
    pages = []
    n = 2 + (i % 5)
    for p in range(n):
        disp = 0.2 * ((i + p) % 8)
        if over and p == 0:
            disp = 4.5 + (i % 6)
        pages.append(
            {
                "page": p,
                "maxDisp": round(disp, 3),
                "over": disp > 2.0,
                "structMismatch": bool(struct and p == n - 1),
                "nodeCountA": 40 + (i % 30) + p,
                "nodeCountB": 40 + (i % 30) + p - (1 if struct and p == n - 1 else 0),
            }
        )
    return pages


def render_envelope(src: str, i: int, regression: bool, via: str) -> dict:
    over = 2 if regression else 0
    struct = 1 if regression and i % 3 == 0 else 0
    pages = render_pages(i, over, struct)
    max_disp = max(p["maxDisp"] for p in pages)
    return {
        "schemaVersion": "1.0",
        "mode": "self-roundtrip",
        "sourceA": src,
        "sourceB": src,
        "via": via,
        "pageFilter": None,
        "threshold": 2.0,
        "pageCountA": len(pages),
        "pageCountB": len(pages),
        "pageCountMismatch": False,
        "maxDisp": max_disp,
        "worstPage": 0 if regression else 0,
        "overPages": over,
        "structPages": struct,
        "hardStructPages": 1 if struct else 0,
        "status": "OVER" if regression else "OK",
        "regression": regression,
        "pages": pages,
        "untrustedContent": False,
        "untrustedFields": [],
    }


def convert_verify_pages_envelope(src: str, i: int, pages_ok: bool) -> dict:
    out = src.replace("samples/", "output/").rsplit(".", 1)[0] + ".hwp"
    return {
        "schemaVersion": "1.0",
        "source": src,
        "output": out,
        "format": "hwp5",
        "bytes": 80000 + (i * 91) % 400000,
        "wasDistribution": i % 5 == 0,
        "passwordProtected": False,
        "verify": {
            "identical": True,
            "diffCount": 0,
        },
        "verifyPages": {
            "expected": 4 + (i % 10),
            "actual": 4 + (i % 10) if pages_ok else 3 + (i % 10),
            "match": pages_ok,
        },
        "untrustedContent": False,
        "untrustedFields": [],
    }


def judgment_from_envelope(command: str, env: dict | None) -> dict:
    j = empty_judgment()
    if not env:
        return j
    mapping = [
        ("identical", "identical"),
        ("hasSignal", "hasSignal"),
        ("reproduced", "reproduced"),
        ("findingCount", "findingCount"),
        ("failCount", "failCount"),
        ("passCount", "passCount"),
        ("verdict", "verdict"),
        ("regression", "regression"),
        ("status", "status"),
        ("clean", "clean"),
        ("signalCount", "signalCount"),
        ("valid", "valid"),
        ("diffCount", "diffCount"),
        ("strict", "strict"),
        ("overflowCount", "overflowCount"),
        ("overlapCount", "overlapCount"),
        ("emptyPageCount", "emptyPageCount"),
        ("pageCountMismatch", "pageCountMismatch"),
        ("overPages", "overPages"),
    ]
    for src, dst in mapping:
        if src in env:
            j[dst] = env[src]
    if isinstance(env.get("verify"), dict):
        j["verify"] = {
            "identical": env["verify"].get("identical"),
            "diffCount": env["verify"].get("diffCount"),
        }
    if command == "info":
        pass
    return j


def argv_for(command: str, src: str, extra: list[str] | None = None) -> list[str]:
    if command == "fill-fields":
        head = ["edit", "fill-fields", src, "--data", "@fields.json"]
    elif command == "ir-diff":
        head = ["ir-diff", src, src.replace(".hwp", ".hwpx")]
    elif command == "replay":
        head = ["replay", "plan.json"]
    else:
        head = [command, src]
    tail = list(extra or [])
    if "--json" not in tail:
        tail.append("--json")
    return head + tail


def build_record(idx: int, agency: str, kind: str, year: int, seq: int) -> dict:
    command = COMMANDS[idx % len(COMMANDS)]
    ext = pick_ext(idx)
    fmt = pick_format(ext)
    src = source_path(agency, kind, year, seq, ext)
    title = f"{TITLES[idx % len(TITLES)]} ({agency} {kind} {year})"
    # 5-way exit rotation plus command-local judgment variants.
    lane = (idx // len(COMMANDS)) % 10
    scenario, exit_class, extra_argv, env = scenario_for(command, lane, src, idx, title, fmt)

    source_tag = f"{agency}/{kind}/{year}/{seq:04d}#{command}/{scenario}"
    stdout = env is not None
    judgment = judgment_from_envelope(command, env)
    stderr = None
    if exit_class == 1:
        stderr = "io"
    elif exit_class == 2:
        stderr = "usage"

    return {
        "recordId": f"vproto-{idx + 1:06d}",
        "sourceTag": source_tag,
        "command": command,
        "argv": argv_for(command, src, extra_argv),
        "exitClass": exit_class,
        "stdoutPresent": stdout,
        "stderrKind": stderr,
        "envelope": env,
        "judgment": judgment,
    }


def scenario_for(command: str, lane: int, src: str, i: int, title: str, fmt: str):
    """Return (scenario, exit, extra_argv, envelope)."""
    if command == "info":
        return info_scenario(lane, src, i, title, fmt)
    if command == "verify":
        return verify_scenario(lane, src, i)
    if command == "ir-diff":
        return ir_diff_scenario(lane, src, i)
    if command == "layout-anomaly":
        return layout_scenario(lane, src, i)
    if command == "replay":
        return replay_scenario(lane, src, i)
    if command == "fill-fields":
        return fill_scenario(lane, src, i)
    if command == "render-diff":
        return render_scenario(lane, src, i)
    raise AssertionError(command)


def info_scenario(lane: int, src: str, i: int, title: str, fmt: str):
    if lane in (0, 1, 2, 3):
        return "ok-meta", 0, [], info_envelope(src, i, title, fmt)
    if lane in (4, 5):
        return "io-missing", 1, [], None
    if lane == 6:
        return "io-truncated", 1, [], None
    if lane == 7:
        return "usage-no-file", 2, [], None
    if lane == 8:
        return "usage-unknown-flag", 2, ["--not-a-flag"], None
    # lane 9: 쪽수 검증은 info 가 내지 않음 — 프로토콜이 exit 4 를 그대로 읽는다.
    env = info_envelope(src, i, title, fmt)
    env["pageCount"] = 0
    return "page-verify-unexpected", 4, ["--json"], env


def verify_scenario(lane: int, src: str, i: int):
    if lane in (0, 1, 2):
        env = verify_envelope(src, i, fail=False)
        return "expect-pass", 0, ["--expect-pages", str(1 + i % 20)], env
    if lane == 3:
        env = verify_envelope(src, i, fail=True)
        return "expect-fail", 3, ["--expect-min-chars", "99999"], env
    if lane in (4, 5):
        return "io-missing", 1, ["--expect-pages", "1"], None
    if lane == 6:
        return "usage-no-expect", 2, [], None
    if lane == 7:
        return "usage-bad-flag", 2, ["--expect-pages", "nope"], None
    if lane == 8:
        env = verify_envelope(src, i, fail=True)
        return "fail-contains", 3, ["--expect-contains", "없는문자열"], env
    env = convert_verify_pages_envelope(src, i, pages_ok=False)
    return "verify-pages-mismatch", 4, ["--expect-pages", "1"], env


def ir_diff_scenario(lane: int, src: str, i: int):
    other = src.replace(".hwp", ".hwpx") if src.endswith(".hwp") else src.replace(".hwpx", ".hwp")
    if lane in (0, 1, 2):
        return "self-identical", 0, [other], ir_diff_envelope(src, src, i, True)
    if lane == 3:
        return "cross-format-diff", 3, [other], ir_diff_envelope(src, other, i, False)
    if lane in (4, 5):
        return "io-missing-a", 1, [other], None
    if lane == 6:
        return "usage-one-file", 2, [], None
    if lane == 7:
        return "usage-bad-section", 2, ["-s", "-1"], None
    if lane == 8:
        return "table-only-diff", 3, [other], ir_diff_envelope(src, other, i + 11, False)
    env = ir_diff_envelope(src, other, i, True)
    return "page-verify-not-applicable", 4, [other], env


def layout_scenario(lane: int, src: str, i: int):
    if lane == 0:
        env = layout_envelope(src, i, 0, 0, 0, False)
        return "clean", 0, [], env
    if lane == 1:
        env = layout_envelope(src, i, 2, 0, 0, False)
        return "overflow-nonstrict", 0, [], env
    if lane == 2:
        env = layout_envelope(src, i, 0, 0, 1, True)
        return "empty-strict-ok", 0, ["--strict"], env
    if lane == 3:
        env = layout_envelope(src, i, 3, 1, 0, True)
        return "overflow-strict", 3, ["--strict"], env
    if lane in (4, 5):
        return "io-missing", 1, [], None
    if lane == 6:
        return "usage-bad-page", 2, ["-p", "-4"], None
    if lane == 7:
        return "usage-bad-tol", 2, ["--overflow-tolerance", "x"], None
    if lane == 8:
        env = layout_envelope(src, i, 0, 4, 0, True)
        return "overlap-strict", 3, ["--strict"], env
    env = layout_envelope(src, i, 0, 0, 2, False)
    env["pageCountMismatch"] = True
    return "page-count-mismatch", 4, [], env


def replay_scenario(lane: int, src: str, i: int):
    if lane in (0, 1):
        env = replay_envelope(src, i, "attest", None)
        return "attest", 0, ["--plan-json", "{}"], env
    if lane == 2:
        env = replay_envelope(src, i, "verify", True)
        return "verify-reproduced", 0, ["--expect-output-sha256", sha256_hex(f"out:{src}:{i}:verify")], env
    if lane == 3:
        env = replay_envelope(src, i, "verify", False)
        return "verify-mismatch", 3, ["--expect-output-sha256", "0" * 64], env
    if lane in (4, 5):
        return "io-missing-plan", 1, [], None
    if lane == 6:
        return "usage-no-plan", 2, [], None
    if lane == 7:
        return "usage-bad-hash", 2, ["--expect-output-sha256", "zz"], None
    if lane == 8:
        env = replay_envelope(src, i, "verify", False)
        env["reproduced"] = False
        return "reproduced-false", 3, ["--expect-output-sha256", "ab" * 32], env
    env = replay_envelope(src, i, "attest", None)
    return "page-verify-not-applicable", 4, [], env


def fill_scenario(lane: int, src: str, i: int):
    if lane == 0:
        env = fill_envelope(src, i, True, None, False)
        return "dry-run", 0, ["--dry-run"], env
    if lane == 1:
        env = fill_envelope(src, i, False, True, False)
        return "write-verify-ok", 0, ["-o", "out.hwp", "--verify"], env
    if lane == 2:
        env = fill_envelope(src, i, False, None, True)
        return "not-found-exit0", 0, ["-o", "out.hwp"], env
    if lane == 3:
        env = fill_envelope(src, i, False, False, False)
        return "verify-ir-diff", 3, ["-o", "out.hwp", "--verify"], env
    if lane in (4, 5):
        return "io-missing", 1, ["-o", "out.hwp"], None
    if lane == 6:
        return "usage-no-output", 2, [], None
    if lane == 7:
        return "usage-bad-data", 2, ["--data", "{"], None
    if lane == 8:
        env = fill_envelope(src, i, False, False, False)
        env["findingCount"] = 2 + (i % 3)
        env["findings"] = [
            {
                "kind": "ssn",
                "masked": "******-*******",
                "page": 0,
                "paragraph": 3,
                "charOffset": 8,
            },
            {
                "kind": "phone",
                "masked": "***-****-****",
                "page": 0,
                "paragraph": 5,
                "charOffset": 4,
            },
        ]
        return "verify-fail-with-findings", 3, ["-o", "out.hwp", "--verify"], env
    env = convert_verify_pages_envelope(src, i, pages_ok=False)
    env["filledCount"] = 1
    env["filled"] = [{"name": "제목", "occurrence": 0, "value": title_stub(i)}]
    env["pageCountMismatch"] = True
    return "verify-pages", 4, ["-o", "out.hwp", "--verify"], env


def title_stub(i: int) -> str:
    return TITLES[i % len(TITLES)]


def render_scenario(lane: int, src: str, i: int):
    via = "hwpx" if i % 2 == 0 else "hwp"
    if lane in (0, 1):
        env = render_envelope(src, i, False, via)
        return "ok", 0, [f"--via", via], env
    if lane == 2:
        env = render_envelope(src, i, False, via)
        env["maxDisp"] = 1.2
        env["overPages"] = 0
        return "under-threshold", 0, ["--max-disp", "2"], env
    if lane == 3:
        env = render_envelope(src, i, True, via)
        return "regression", 3, [f"--via", via], env
    if lane in (4, 5):
        return "io-missing", 1, [], None
    if lane == 6:
        return "usage-bad-via", 2, ["--via", "pdf"], None
    if lane == 7:
        return "usage-bad-page", 2, ["-p", "x"], None
    if lane == 8:
        env = render_envelope(src, i, True, via)
        env["status"] = "STRUCT"
        env["structPages"] = 2
        return "struct-mismatch", 3, [], env
    env = render_envelope(src, i, False, via)
    env["pageCountA"] = 3
    env["pageCountB"] = 4
    env["pageCountMismatch"] = True
    return "page-count-mismatch", 4, [], env


def uniqueness_key(rec: dict) -> tuple:
    j = rec["judgment"]
    verify = j.get("verify") or {}
    return (
        rec["command"],
        rec["exitClass"],
        json.dumps(j, sort_keys=True, ensure_ascii=False),
        rec["sourceTag"],
        str(verify.get("identical")),
        str(verify.get("diffCount")),
    )


def generate_records() -> list[dict]:
    records = []
    idx = 0
    seq = 1
    for year in YEARS:
        for kind in KINDS:
            for agency in AGENCIES:
                records.append(build_record(idx, agency, kind, year, seq))
                idx += 1
                seq += 1
    return records


def write_shards(records: list[dict], per_shard: int = 40) -> list[dict]:
    if SHARDS.exists():
        for old in SHARDS.glob("*.json"):
            old.unlink()
    SHARDS.mkdir(parents=True, exist_ok=True)
    metas = []
    for start in range(0, len(records), per_shard):
        chunk = records[start : start + per_shard]
        shard_id = f"{start // per_shard:03d}"
        payload = {
            "schemaVersion": "v-proto.1.0",
            "shardId": shard_id,
            "records": chunk,
        }
        path = SHARDS / f"{shard_id}.json"
        text = json.dumps(payload, ensure_ascii=False, indent=2) + "\n"
        path.write_text(text, encoding="utf-8", newline="\n")
        metas.append(
            {
                "path": f"shards/{path.name}",
                "count": len(chunk),
                "commands": sorted({r["command"] for r in chunk}),
                "exitClasses": sorted({r["exitClass"] for r in chunk}),
            }
        )
    return metas


def write_manifest(records: list[dict], metas: list[dict]) -> None:
    exit_counts: dict[str, int] = {}
    cmd_counts: dict[str, int] = {}
    for rec in records:
        exit_counts[str(rec["exitClass"])] = exit_counts.get(str(rec["exitClass"]), 0) + 1
        cmd_counts[rec["command"]] = cmd_counts.get(rec["command"], 0) + 1
    manifest = {
        "schemaVersion": "v-proto.1.0",
        "generatedBy": "tools/llm_verifier/verdict_protocol/generate_corpus.py",
        "recordCount": len(records),
        "shardCount": len(metas),
        "exitClassCounts": exit_counts,
        "commandCounts": cmd_counts,
        "uniqueness": "command+exitClass+judgmentFingerprint+sourceTag",
        "shards": metas,
    }
    (CORPUS / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
        newline="\n",
    )


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
    lines = 0
    for p in SHARDS.glob("*.json"):
        lines += sum(1 for _ in p.open(encoding="utf-8"))
    manifest_lines = sum(1 for _ in (CORPUS / "manifest.json").open(encoding="utf-8"))
    print(
        f"records={len(records)} shards={len(metas)} "
        f"shard_lines={lines} manifest_lines={manifest_lines} total={lines + manifest_lines}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
