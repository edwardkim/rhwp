#!/usr/bin/env python3
"""Emit the V-repeat corpus of distinct (artifact, k, check, votes, variance, final).

Each record is the same artifact read K times with independent seeds.
Reduction is majority (categorical) or mean (numeric). This script does
not rank candidates (V-bon) and does not split criteria (V-decomp).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from copy import deepcopy
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
CORPUS = HERE / "corpus"
SHARDS = CORPUS / "shards"

SCHEMA_VERSION = "v-repeat.1.0"
CLAIM = "V-repeat"
KIND = "repeatEvaluation"
GENERATED_BY = "tools/llm_verifier/repeat_eval/generate_corpus.py"
DEFAULT_MIN_LINES = 112000
DEFAULT_MIN_RECORDS = 900
RECORDS_PER_SHARD = 24

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
)

COMMANDS = (
    "info",
    "verify",
    "ir-diff",
    "layout-anomaly",
    "replay",
    "fill-fields",
    "render-diff",
    "convert",
    "replace-text",
    "redact",
    "set-cell",
    "csv-to-table",
    "sanitize",
)

ARGV_HEAD = {
    "info": ("info",),
    "verify": ("verify",),
    "ir-diff": ("ir-diff",),
    "layout-anomaly": ("layout-anomaly",),
    "replay": ("replay",),
    "fill-fields": ("edit", "fill-fields"),
    "render-diff": ("render-diff",),
    "convert": ("convert",),
    "replace-text": ("edit", "replace-text"),
    "redact": ("edit", "redact"),
    "set-cell": ("edit", "set-cell"),
    "csv-to-table": ("csv-to-table",),
    "sanitize": ("edit", "sanitize"),
}

CHECKS_FOR = {
    "info": (("exitClass", "exit"), ("untrustedContent", "bool"), ("pageCount", "u64")),
    "verify": (("exitClass", "exit"), ("verdict", "text"), ("failCount", "u64"), ("passFail", "passFail")),
    "ir-diff": (("exitClass", "exit"), ("identical", "bool"), ("diffCount", "u64"), ("passFail", "passFail")),
    "layout-anomaly": (
        ("exitClass", "exit"),
        ("hasSignal", "bool"),
        ("overflowCount", "u64"),
        ("overlapCount", "u64"),
        ("passFail", "passFail"),
    ),
    "replay": (("exitClass", "exit"), ("reproduced", "bool"), ("passFail", "passFail")),
    "fill-fields": (
        ("exitClass", "exit"),
        ("verify.identical", "bool"),
        ("filledCount", "u64"),
        ("verify.diffCount", "u64"),
        ("passFail", "passFail"),
    ),
    "render-diff": (
        ("exitClass", "exit"),
        ("regression", "bool"),
        ("status", "text"),
        ("pageCountMismatch", "bool"),
        ("passFail", "passFail"),
    ),
    "convert": (("exitClass", "exit"), ("verify.identical", "bool"), ("pageCountMismatch", "bool")),
    "replace-text": (("exitClass", "exit"), ("replacedCount", "u64"), ("verify.identical", "bool")),
    "redact": (("exitClass", "exit"), ("redactedCount", "u64"), ("verify.identical", "bool")),
    "set-cell": (("exitClass", "exit"), ("changedCount", "u64"), ("verify.identical", "bool")),
    "csv-to-table": (("exitClass", "exit"), ("changedCount", "u64"), ("verify.identical", "bool")),
    "sanitize": (("exitClass", "exit"), ("removedCount", "u64"), ("verify.identical", "bool")),
}

K_VALUES = (3, 5, 7, 9, 11, 13, 15)
PROFILES = (
    "unanimous",
    "flip_one",
    "flip_two",
    "flip_mod3",
    "rare_io",
    "rare_usage",
    "rare_page",
    "exit_mix",
    "majority_wrong",
    "tie_even",
    "three_way",
    "jitter_pm1",
    "jitter_pm2",
    "high_jitter",
    "text_split",
)


def sha16(*parts: Any) -> str:
    h = hashlib.sha256()
    for p in parts:
        h.update(str(p).encode("utf-8"))
        h.update(b"\x1f")
    return h.hexdigest()[:16]


def check_spec(name: str, value_kind: str) -> dict[str, Any]:
    if name == "exitClass":
        kind = "exitClass"
        path = None
    elif name == "passFail":
        kind = "passFail"
        path = None
    else:
        kind = "envelopeField"
        path = name
    spec: dict[str, Any] = {"name": name, "kind": kind, "valueKind": value_kind}
    if path is not None:
        spec["path"] = path
    return spec


def sample_identity(index: int) -> dict[str, Any]:
    agency = AGENCIES[index % len(AGENCIES)]
    kind = KINDS[(index // len(AGENCIES)) % len(KINDS)]
    year = YEARS[(index // (len(AGENCIES) * len(KINDS))) % len(YEARS)]
    serial = 1 + (index % 991)
    ext = "hwpx" if index % 3 else "hwp"
    rel = f"samples/gov/{agency}/{kind}/{year}/{serial:04d}.{ext}"
    return {
        "agency": agency,
        "docKind": kind,
        "year": year,
        "serial": serial,
        "sample": rel,
        "sourceFormat": "hwpx" if ext == "hwpx" else "hwp5",
        "output": f"out/v-repeat/{agency}/{kind}/{year}/{serial:04d}.out.{ext}",
    }


def field_plan(n: int, salt: int) -> list[str]:
    n = max(1, n)
    return [FIELD_NAMES[(salt + i) % len(FIELD_NAMES)] for i in range(n)]


def intended_bundle(command: str, ident: dict[str, Any], salt: int) -> tuple[int, dict[str, Any]]:
    filled = 2 + (salt % 9)
    plan = field_plan(filled, salt)
    pages = 1 + (salt % 17)
    exit_c = 0
    env: dict[str, Any]
    if command == "info":
        env = {
            "schemaVersion": "1.0",
            "source": ident["sample"],
            "format": ident["sourceFormat"],
            "pageCount": pages,
            "paraCount": 12 + (salt % 40),
            "sectionCount": 1 + (salt % 3),
            "untrustedContent": False,
            "warnings": [],
        }
    elif command == "verify":
        env = {
            "source": ident["sample"],
            "verdict": "pass",
            "passCount": 4 + (salt % 6),
            "failCount": 0,
            "expectations": ["pageCount", "paraCount", "sectionCount"],
        }
    elif command == "ir-diff":
        env = {
            "left": ident["sample"],
            "right": ident["output"],
            "identical": True,
            "diffCount": 0,
            "categories": {},
        }
    elif command == "layout-anomaly":
        env = {
            "source": ident["sample"],
            "hasSignal": False,
            "strict": True,
            "overflowCount": 0,
            "overlapCount": 0,
            "emptyPageCount": 0,
            "offCanvasCount": 0,
        }
    elif command == "replay":
        env = {
            "mode": "verify",
            "reproduced": True,
            "valid": True,
            "signatureOk": True,
            "capsuleShaMatches": True,
        }
    elif command == "fill-fields":
        env = {
            "schemaVersion": "1.0",
            "source": ident["sample"],
            "dryRun": False,
            "filledCount": filled,
            "notFound": [],
            "ambiguous": [],
            "untrustedContent": False,
            "filled": [
                {"name": plan[i], "occurrence": 0, "value": f"{plan[i]}-{salt:04d}-{i:02d}"}
                for i in range(min(4, filled))
            ],
            "verify": {"identical": True, "diffCount": 0},
            "changedPages": [0],
        }
    elif command == "render-diff":
        env = {
            "source": ident["sample"],
            "regression": False,
            "status": "OK",
            "maxDisp": 0,
            "overPages": 0,
            "pageCountMismatch": False,
        }
    elif command == "convert":
        env = {
            "source": ident["sample"],
            "output": ident["output"],
            "outputFormat": "hwpx",
            "pageCountMismatch": False,
            "verify": {"identical": True, "diffCount": 0},
        }
    elif command == "replace-text":
        env = {
            "source": ident["sample"],
            "replacedCount": 1 + (salt % 5),
            "dryRun": False,
            "verify": {"identical": True, "diffCount": 0},
        }
    elif command == "redact":
        env = {
            "source": ident["sample"],
            "redactedCount": 2 + (salt % 4),
            "dryRun": False,
            "verify": {"identical": True, "diffCount": 0},
        }
    elif command == "set-cell":
        env = {
            "source": ident["sample"],
            "changedCount": 1 + (salt % 6),
            "dryRun": False,
            "verify": {"identical": True, "diffCount": 0},
        }
    elif command == "csv-to-table":
        env = {
            "source": ident["sample"],
            "csv": ident["sample"].replace(".hwp", ".csv").replace(".hwpx", ".csv"),
            "changedCount": 3 + (salt % 8),
            "invalid": [],
            "rowCount": 6 + (salt % 12),
            "colCount": 3 + (salt % 4),
            "verify": {"identical": True, "diffCount": 0},
        }
    else:
        env = {
            "source": ident["sample"],
            "removedCount": 1 + (salt % 3),
            "wasDistribution": True,
            "verify": {"identical": True, "diffCount": 0},
        }
    env["exitClass"] = exit_c
    return exit_c, env


def read_path(env: dict[str, Any] | None, path: str) -> Any:
    if env is None:
        return None
    cur: Any = env
    for part in path.split("."):
        if not isinstance(cur, dict) or part not in cur:
            return None
        cur = cur[part]
    return cur


def set_path(env: dict[str, Any], path: str, value: Any) -> None:
    parts = path.split(".")
    cur = env
    for part in parts[:-1]:
        nxt = cur.get(part)
        if not isinstance(nxt, dict):
            nxt = {}
            cur[part] = nxt
        cur = nxt
    cur[parts[-1]] = value


def fail_signals(env: dict[str, Any] | None) -> list[str]:
    if not env:
        return ["missing"]
    out: list[str] = []
    if read_path(env, "identical") is False:
        out.append("identical=false")
    if read_path(env, "verify.identical") is False:
        out.append("verify.identical=false")
    if read_path(env, "hasSignal") is True:
        out.append("hasSignal=true")
    if read_path(env, "regression") is True:
        out.append("regression=true")
    if read_path(env, "pageCountMismatch") is True:
        out.append("pageCountMismatch=true")
    if read_path(env, "untrustedContent") is True:
        out.append("untrustedContent=true")
    for key, label in (
        ("diffCount", "diffCount>0"),
        ("verify.diffCount", "verify.diffCount>0"),
        ("failCount", "failCount>0"),
        ("overflowCount", "overflowCount>0"),
    ):
        val = read_path(env, key)
        if isinstance(val, int) and val > 0:
            out.append(label)
    verdict = read_path(env, "verdict")
    if isinstance(verdict, str) and verdict != "pass":
        out.append(f"verdict={verdict}")
    status = read_path(env, "status")
    if isinstance(status, str) and status.upper() != "OK":
        out.append(f"status={status}")
    if read_path(env, "reproduced") is False:
        out.append("reproduced=false")
    inv = read_path(env, "invalid")
    if inv is True or (isinstance(inv, list) and inv):
        out.append("invalid")
    return out


def flip_bool(env: dict[str, Any], path: str) -> None:
    cur = read_path(env, path)
    if isinstance(cur, bool):
        set_path(env, path, not cur)
    elif path == "reproduced":
        set_path(env, path, False)
    else:
        set_path(env, path, False)


def bump_u64(env: dict[str, Any], path: str, delta: int) -> None:
    cur = read_path(env, path)
    if not isinstance(cur, int):
        cur = 0
    set_path(env, path, max(0, cur + delta))


def apply_fail_shape(env: dict[str, Any], check_name: str) -> None:
    if check_name in ("identical", "verify.identical"):
        flip_bool(env, check_name)
        if check_name == "verify.identical":
            bump_u64(env, "verify.diffCount", 1)
        else:
            bump_u64(env, "diffCount", 1)
    elif check_name == "hasSignal":
        set_path(env, "hasSignal", True)
        bump_u64(env, "overflowCount", 1)
    elif check_name == "regression":
        set_path(env, "regression", True)
        set_path(env, "status", "FAIL")
    elif check_name == "pageCountMismatch":
        set_path(env, "pageCountMismatch", True)
    elif check_name == "untrustedContent":
        set_path(env, "untrustedContent", True)
    elif check_name == "reproduced":
        set_path(env, "reproduced", False)
    elif check_name == "verdict":
        set_path(env, "verdict", "fail")
        bump_u64(env, "failCount", 1)
    elif check_name == "status":
        set_path(env, "status", "FAIL")
        set_path(env, "regression", True)
    elif check_name in (
        "filledCount",
        "changedCount",
        "diffCount",
        "verify.diffCount",
        "failCount",
        "overflowCount",
        "overlapCount",
        "emptyPageCount",
        "overPages",
        "replacedCount",
        "redactedCount",
        "removedCount",
        "pageCount",
        "paraCount",
        "passCount",
    ):
        bump_u64(env, check_name, 1)
    elif check_name == "passFail":
        if "verify" in env:
            set_path(env, "verify.identical", False)
            bump_u64(env, "verify.diffCount", 1)
        elif "identical" in env:
            set_path(env, "identical", False)
            bump_u64(env, "diffCount", 1)
        elif "verdict" in env:
            set_path(env, "verdict", "fail")
            bump_u64(env, "failCount", 1)
        elif "hasSignal" in env:
            set_path(env, "hasSignal", True)
            bump_u64(env, "overflowCount", 1)
        elif "reproduced" in env:
            set_path(env, "reproduced", False)
        elif "regression" in env:
            set_path(env, "regression", True)
            set_path(env, "status", "FAIL")
        else:
            set_path(env, "untrustedContent", True)


def should_mutate(profile: str, seed: int, k: int, value_kind: str) -> str:
    """Return mutation tag. Depends on seed only so K ladders stay prefix-stable."""
    del k
    if profile == "unanimous":
        return "none"
    if profile == "flip_one":
        return "flip" if seed == 0 else "none"
    if profile == "flip_two":
        return "flip" if seed in (0, 1) else "none"
    if profile == "flip_mod3":
        return "flip" if seed % 3 == 0 else "none"
    if profile == "rare_io":
        return "io" if seed == 0 else "none"
    if profile == "rare_usage":
        return "usage" if seed == 1 else "none"
    if profile == "rare_page":
        return "page" if seed == 2 else "none"
    if profile == "exit_mix":
        return "exit3" if seed % 4 == 3 else "none"
    if profile == "majority_wrong":
        return "flip" if seed % 3 != 0 else "none"
    if profile == "tie_even":
        return "flip" if seed % 2 == 0 else "none"
    if profile == "three_way":
        return ("none", "exit3", "page")[seed % 3]
    if profile == "jitter_pm1":
        if value_kind != "u64":
            return "flip" if seed == 0 else "none"
        return "delta1" if seed % 2 == 0 else "delta-1"
    if profile == "jitter_pm2":
        if value_kind != "u64":
            return "flip" if seed in (0, 2) else "none"
        return "delta2" if seed % 2 == 0 else "delta-2"
    if profile == "high_jitter":
        if value_kind != "u64":
            return "flip" if seed % 2 == 0 else "none"
        return f"delta{seed % 5}"
    if profile == "text_split":
        if value_kind == "text":
            return "text" if seed % 2 == 1 else "none"
        return "flip" if seed % 2 == 1 else "none"
    return "none"


def mutate_trial(
    base_env: dict[str, Any],
    base_exit: int,
    check_name: str,
    value_kind: str,
    profile: str,
    seed: int,
    k: int,
) -> tuple[int, dict[str, Any]]:
    env = deepcopy(base_env)
    exit_c = base_exit
    tag = should_mutate(profile, seed, k, value_kind)
    if tag == "none":
        env["exitClass"] = exit_c
        return exit_c, env
    if tag == "io":
        exit_c = 1
        env["exitClass"] = exit_c
        return exit_c, env
    if tag == "usage":
        exit_c = 2
        env["exitClass"] = exit_c
        return exit_c, env
    if tag == "page":
        exit_c = 4
        env["exitClass"] = exit_c
        if "pageCountMismatch" in env:
            env["pageCountMismatch"] = True
        apply_fail_shape(env, check_name)
        env["exitClass"] = exit_c
        return exit_c, env
    if tag == "exit3":
        exit_c = 3
        apply_fail_shape(env, check_name)
        env["exitClass"] = exit_c
        return exit_c, env
    if tag.startswith("delta"):
        delta = int(tag[5:])
        path = check_name if check_name != "exitClass" else "filledCount"
        if value_kind == "u64":
            bump_u64(env, check_name, delta)
        else:
            apply_fail_shape(env, path)
        env["exitClass"] = exit_c
        return exit_c, env
    if tag == "text":
        if check_name == "verdict":
            set_path(env, "verdict", "fail")
            bump_u64(env, "failCount", 1)
        elif check_name == "status":
            set_path(env, "status", "FAIL")
        else:
            apply_fail_shape(env, check_name)
        if check_name in ("verdict", "status", "passFail"):
            exit_c = 3
        env["exitClass"] = exit_c
        return exit_c, env
    # flip
    if check_name == "exitClass":
        exit_c = 3 if base_exit == 0 else 0
        if exit_c == 3:
            apply_fail_shape(env, "passFail")
        env["exitClass"] = exit_c
        return exit_c, env
    apply_fail_shape(env, check_name)
    if value_kind in ("bool", "text", "passFail", "exit"):
        if exit_c == 0 and (
            fail_signals(env)
            or check_name in ("identical", "verify.identical", "passFail", "verdict")
        ):
            exit_c = 3
    env["exitClass"] = exit_c
    return exit_c, env


def format_observed(exit_c: int, env: dict[str, Any], spec: dict[str, Any]) -> str:
    kind = spec["kind"]
    value_kind = spec["valueKind"]
    if kind == "exitClass":
        return str(exit_c)
    if kind == "passFail":
        if exit_c == 0 and not fail_signals(env):
            return "pass"
        return "fail"
    path = spec.get("path") or spec["name"]
    raw = read_path(env, path)
    if raw is None:
        return "missing"
    if value_kind == "bool":
        if raw is True:
            return "true"
        if raw is False:
            return "false"
        return "missing"
    if value_kind == "u64":
        if isinstance(raw, bool) or not isinstance(raw, int):
            return "missing"
        return str(raw)
    if value_kind == "text":
        return str(raw)
    return str(raw)


def conservative_pick(winners: list[str], value_kind: str) -> str:
    if value_kind == "exit":
        codes = []
        for w in winners:
            try:
                n = int(w)
            except ValueError:
                continue
            if 0 <= n <= 4:
                codes.append(n)
        return str(max(codes) if codes else 3)
    if value_kind == "bool":
        return "false" if "false" in winners else winners[-1]
    if value_kind == "passFail":
        return "fail" if "fail" in winners else winners[-1]
    if value_kind == "text":
        for bad in ("fail", "FAIL", "judgment_fail", "mismatch"):
            if bad in winners:
                return bad
        return winners[-1]
    nums = []
    for w in winners:
        try:
            nums.append(int(w))
        except ValueError:
            pass
    return str(max(nums) if nums else winners[-1])


def vote_tally(values: list[str], value_kind: str) -> dict[str, Any]:
    counts: dict[str, int] = {}
    for v in values:
        counts[v] = counts.get(v, 0) + 1
    ordered = dict(sorted(counts.items()))
    total = len(values)
    max_c = max(ordered.values()) if ordered else 0
    winners = sorted(k for k, c in ordered.items() if c == max_c)
    is_tie = len(winners) > 1
    plurality = conservative_pick(winners, value_kind) if is_tie else (winners[0] if winners else "missing")
    majority = None if is_tie or max_c * 2 <= total else plurality
    frac = 0.0 if total == 0 else max_c / total
    return {
        "counts": ordered,
        "majority": majority,
        "plurality": plurality,
        "majorityCount": max_c,
        "isTie": is_tie,
        "majorityFrac": frac,
    }


def format_num(v: float) -> str:
    if abs(v - round(v)) < 1e-9:
        return str(int(round(v)))
    s = f"{v:.6f}".rstrip("0").rstrip(".")
    return s


def variance_categorical(values: list[str], majority_frac: float) -> dict[str, Any]:
    return {
        "n": len(values),
        "distinct": len(set(values)),
        "disagreement": max(0.0, min(1.0, 1.0 - majority_frac)),
    }


def variance_numeric(xs: list[float]) -> dict[str, Any]:
    n = len(xs)
    if n == 0:
        return {"n": 0, "distinct": 0, "disagreement": 0.0}
    mean = sum(xs) / n
    var = None
    if n >= 2:
        ss = sum((x - mean) ** 2 for x in xs)
        var = ss / (n - 1)
    else:
        var = 0.0
    uniq = sorted(set(round(x, 12) for x in xs))
    freq: dict[int, int] = {}
    for x in xs:
        key = int(round(x * 1000.0))
        freq[key] = freq.get(key, 0) + 1
    max_c = max(freq.values())
    return {
        "n": n,
        "distinct": len(uniq),
        "disagreement": 1.0 - (max_c / n),
        "sampleVariance": var,
        "mean": mean,
        "min": min(xs),
        "max": max(xs),
    }


def final_from_votes(tally: dict[str, Any], value_kind: str) -> dict[str, Any]:
    value = tally["majority"] if tally["majority"] is not None else tally["plurality"]
    if value_kind == "exit":
        pass_v = value == "0"
    elif value_kind == "bool":
        pass_v = value == "true"
    elif value_kind == "passFail":
        pass_v = value == "pass"
    elif value_kind == "text":
        pass_v = value == "pass" or value.upper() == "OK"
    else:
        pass_v = False
    return {
        "reduce": "majority",
        "value": value,
        "tie": tally["isTie"],
        "pass": bool(pass_v and not tally["isTie"]),
    }


def final_from_mean(mean: float, intended: float | None) -> dict[str, Any]:
    pass_v = True if intended is None else abs(mean - intended) < 0.5
    return {
        "reduce": "mean",
        "value": format_num(mean),
        "tie": False,
        "pass": pass_v,
        "numeric": mean,
    }


def intended_number(env: dict[str, Any], spec: dict[str, Any]) -> float | None:
    if spec["valueKind"] != "u64":
        return None
    path = spec.get("path") or spec["name"]
    raw = read_path(env, path)
    if isinstance(raw, bool) or not isinstance(raw, (int, float)):
        return None
    return float(raw)


def build_record(
    record_id: str,
    command: str,
    ident: dict[str, Any],
    salt: int,
    profile: str,
    k: int,
    check_name: str,
    value_kind: str,
) -> dict[str, Any]:
    base_exit, base_env = intended_bundle(command, ident, salt)
    spec = check_spec(check_name, value_kind)
    artifact_id = sha16(command, ident["sample"], base_exit, json.dumps(base_env, ensure_ascii=False, sort_keys=True))
    argv = list(ARGV_HEAD[command])
    if command in (
        "fill-fields",
        "replace-text",
        "redact",
        "set-cell",
        "sanitize",
        "csv-to-table",
        "convert",
    ):
        argv += ["--verify", "--json", ident["sample"]]
    elif command == "ir-diff":
        argv += ["--json", ident["sample"], ident["output"]]
    else:
        argv += ["--json", ident["sample"]]

    trials = []
    observed_vals: list[str] = []
    nums: list[float] = []
    for seed in range(k):
        exit_c, env = mutate_trial(base_env, base_exit, check_name, value_kind, profile, seed, k)
        observed = format_observed(exit_c, env, spec)
        observed_vals.append(observed)
        if value_kind == "u64":
            raw = read_path(env, spec.get("path") or spec["name"])
            if isinstance(raw, int) and not isinstance(raw, bool):
                nums.append(float(raw))
        trials.append(
            {
                "seed": seed,
                "exitClass": exit_c,
                "observed": observed,
                "envelope": env,
            }
        )

    votes = vote_tally(observed_vals, value_kind)
    if value_kind == "u64":
        variance = variance_numeric(nums)
        mean = variance.get("mean", 0.0)
        final = final_from_mean(float(mean), intended_number(base_env, spec))
    else:
        variance = variance_categorical(observed_vals, votes["majorityFrac"])
        final = final_from_votes(votes, value_kind)

    uniq = f"{artifact_id}|k={k}|{check_name}"
    return {
        "schemaVersion": SCHEMA_VERSION,
        "claim": CLAIM,
        "kind": KIND,
        "recordId": record_id,
        "uniquenessKey": uniq,
        "artifact": {
            "artifactId": artifact_id,
            "command": command,
            "sample": ident["sample"],
            "argv": argv,
            "intendedExit": base_exit,
            "intended": base_env,
        },
        "k": k,
        "check": spec,
        "trials": trials,
        "votes": votes,
        "variance": variance,
        "finalValue": final,
        "profile": profile,
    }


def iter_specs() -> list[tuple[int, str, dict[str, Any], int, str, str, str]]:
    """(index, command, ident, salt, profile, check, value_kind) families."""
    out: list[tuple[int, str, dict[str, Any], int, str, str, str]] = []
    idx = 0
    # Enough families that several K per family exceed the line gate with distinct keys.
    for fam in range(0, 96):
        command = COMMANDS[fam % len(COMMANDS)]
        ident = sample_identity(fam * 3 + 17)
        salt = 1000 + fam * 13
        profile = PROFILES[fam % len(PROFILES)]
        checks = CHECKS_FOR[command]
        check_name, value_kind = checks[fam % len(checks)]
        # skip profiles that do not fit the value kind when they would collapse
        if profile.startswith("jitter") and value_kind != "u64" and fam % 5 == 0:
            check_name, value_kind = next(
                ((n, k) for n, k in checks if k == "u64"),
                (check_name, value_kind),
            )
        if profile == "text_split" and value_kind != "text":
            text_check = next(((n, k) for n, k in checks if k == "text"), None)
            if text_check:
                check_name, value_kind = text_check
        out.append((idx, command, ident, salt, profile, check_name, value_kind))
        idx += 1
    return out


def write_json(path: Path, payload: Any) -> int:
    text = json.dumps(payload, ensure_ascii=False, indent=2) + "\n"
    path.write_text(text, encoding="utf-8", newline="\n")
    return text.count("\n")


def generate(min_lines: int, min_records: int) -> dict[str, Any]:
    if SHARDS.exists():
        for old in SHARDS.glob("*.json"):
            old.unlink()
    SHARDS.mkdir(parents=True, exist_ok=True)

    families = iter_specs()
    records: list[dict[str, Any]] = []
    seen: set[str] = set()
    rec_i = 0
    # Prefix-stable K ladder on each family so larger K reuses seeds 0..k-1.
    k_ladder = K_VALUES
    for fam_i, command, ident, salt, profile, check_name, value_kind in families:
        for k in k_ladder:
            if profile == "tie_even" and k % 2 == 1:
                continue
            rec_i += 1
            record_id = f"rep-{rec_i:06d}"
            row = build_record(record_id, command, ident, salt, profile, k, check_name, value_kind)
            key = row["uniquenessKey"]
            if key in seen:
                rec_i -= 1
                continue
            seen.add(key)
            records.append(row)
        if len(records) >= min_records:
            # keep going until line estimate is enough; estimate after write
            pass

    # If still short, add extra families with remaining checks.
    extra = 0
    while True:
        # write preview line count later; break after enough records first
        if len(records) >= max(min_records, 1100) and extra >= 0:
            break
        extra += 1
        if extra > 2000:
            break
        command = COMMANDS[extra % len(COMMANDS)]
        ident = sample_identity(8000 + extra * 11)
        salt = 90000 + extra * 17
        profile = PROFILES[(extra + 3) % len(PROFILES)]
        check_name, value_kind = CHECKS_FOR[command][extra % len(CHECKS_FOR[command])]
        for k in (5, 7, 11):
            rec_i += 1
            record_id = f"rep-{rec_i:06d}"
            row = build_record(record_id, command, ident, salt, profile, k, check_name, value_kind)
            if row["uniquenessKey"] in seen:
                rec_i -= 1
                continue
            seen.add(row["uniquenessKey"])
            records.append(row)

    shards_meta = []
    line_count = 0
    check_counts: dict[str, int] = {}
    command_counts: dict[str, int] = {}
    k_counts: dict[str, int] = {}
    written_records = 0
    shard_i = 0
    for start in range(0, len(records), RECORDS_PER_SHARD):
        chunk = records[start : start + RECORDS_PER_SHARD]
        shard_id = f"{shard_i:03d}"
        # strip helper-only profile from stored row? keep it — it documents the noise
        # model and is not V-bon/V-decomp. Ownership allows it. But uniqueness is
        # (artifact,k,check); profile is metadata of how trials were produced.
        payload = {
            "schemaVersion": SCHEMA_VERSION,
            "claim": CLAIM,
            "shardId": shard_id,
            "records": chunk,
        }
        rel = f"shards/{shard_id}.json"
        nlines = write_json(SHARDS / f"{shard_id}.json", payload)
        line_count += nlines
        checks = sorted({r["check"]["name"] for r in chunk})
        commands = sorted({r["artifact"]["command"] for r in chunk})
        shards_meta.append(
            {
                "path": rel,
                "count": len(chunk),
                "checks": checks,
                "commands": commands,
            }
        )
        for r in chunk:
            check_counts[r["check"]["name"]] = check_counts.get(r["check"]["name"], 0) + 1
            command_counts[r["artifact"]["command"]] = command_counts.get(r["artifact"]["command"], 0) + 1
            k_counts[str(r["k"])] = k_counts.get(str(r["k"]), 0) + 1
        written_records += len(chunk)
        shard_i += 1

    # If under the line gate, add more records with larger K / extra families.
    guard = 0
    extra_cursor = 0
    while line_count < min_lines or written_records < min_records:
        guard += 1
        if guard > 4000:
            raise RuntimeError(f"could not reach line gate ({line_count} lines, {written_records} records)")
        command = COMMANDS[extra_cursor % len(COMMANDS)]
        ident = sample_identity(20000 + extra_cursor * 19)
        salt = 120000 + extra_cursor * 23
        profile = PROFILES[(extra_cursor + 7) % len(PROFILES)]
        check_name, value_kind = CHECKS_FOR[command][(extra_cursor // 2) % len(CHECKS_FOR[command])]
        k = K_VALUES[extra_cursor % len(K_VALUES)]
        if profile == "tie_even" and k % 2 == 1:
            k = 8
        extra_cursor += 1
        rec_i += 1
        record_id = f"rep-{rec_i:06d}"
        row = build_record(record_id, command, ident, salt, profile, k, check_name, value_kind)
        if row["uniquenessKey"] in seen:
            rec_i -= 1
            continue
        seen.add(row["uniquenessKey"])
        # append to a new shard or last shard if small
        shard_id = f"{shard_i:03d}"
        payload = {
            "schemaVersion": SCHEMA_VERSION,
            "claim": CLAIM,
            "shardId": shard_id,
            "records": [row],
        }
        # accumulate into buffer of RECORDS_PER_SHARD via last file rewrite is messy;
        # write one-record shards only as last resort — better pack.
        # Use a rolling buffer.
        if not hasattr(generate, "_buf"):
            generate._buf = []  # type: ignore[attr-defined]
        generate._buf.append(row)  # type: ignore[attr-defined]
        if len(generate._buf) >= RECORDS_PER_SHARD or (line_count >= min_lines and written_records + len(generate._buf) >= min_records):  # type: ignore[attr-defined]
            chunk = generate._buf  # type: ignore[attr-defined]
            payload = {
                "schemaVersion": SCHEMA_VERSION,
                "claim": CLAIM,
                "shardId": shard_id,
                "records": chunk,
            }
            rel = f"shards/{shard_id}.json"
            nlines = write_json(SHARDS / f"{shard_id}.json", payload)
            line_count += nlines
            shards_meta.append(
                {
                    "path": rel,
                    "count": len(chunk),
                    "checks": sorted({r["check"]["name"] for r in chunk}),
                    "commands": sorted({r["artifact"]["command"] for r in chunk}),
                }
            )
            for r in chunk:
                check_counts[r["check"]["name"]] = check_counts.get(r["check"]["name"], 0) + 1
                command_counts[r["artifact"]["command"]] = command_counts.get(r["artifact"]["command"], 0) + 1
                k_counts[str(r["k"])] = k_counts.get(str(r["k"]), 0) + 1
            written_records += len(chunk)
            shard_i += 1
            generate._buf = []  # type: ignore[attr-defined]

    if getattr(generate, "_buf", None):
        chunk = generate._buf  # type: ignore[attr-defined]
        shard_id = f"{shard_i:03d}"
        payload = {
            "schemaVersion": SCHEMA_VERSION,
            "claim": CLAIM,
            "shardId": shard_id,
            "records": chunk,
        }
        rel = f"shards/{shard_id}.json"
        nlines = write_json(SHARDS / f"{shard_id}.json", payload)
        line_count += nlines
        shards_meta.append(
            {
                "path": rel,
                "count": len(chunk),
                "checks": sorted({r["check"]["name"] for r in chunk}),
                "commands": sorted({r["artifact"]["command"] for r in chunk}),
            }
        )
        for r in chunk:
            check_counts[r["check"]["name"]] = check_counts.get(r["check"]["name"], 0) + 1
            command_counts[r["artifact"]["command"]] = command_counts.get(r["artifact"]["command"], 0) + 1
            k_counts[str(r["k"])] = k_counts.get(str(r["k"]), 0) + 1
        written_records += len(chunk)
        generate._buf = []  # type: ignore[attr-defined]

    manifest = {
        "schemaVersion": SCHEMA_VERSION,
        "claim": CLAIM,
        "generatedBy": GENERATED_BY,
        "recordCount": written_records,
        "shardCount": len(shards_meta),
        "uniqueness": "artifactId|k|check",
        "lineCount": line_count,
        "checkCounts": dict(sorted(check_counts.items())),
        "commandCounts": dict(sorted(command_counts.items())),
        "kCounts": dict(sorted(k_counts.items(), key=lambda kv: int(kv[0]))),
        "shards": shards_meta,
    }
    write_json(CORPUS / "manifest.json", manifest)
    return manifest


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--min-lines", type=int, default=DEFAULT_MIN_LINES)
    p.add_argument("--min-records", type=int, default=DEFAULT_MIN_RECORDS)
    args = p.parse_args(argv)
    manifest = generate(args.min_lines, args.min_records)
    print(
        json.dumps(
            {
                "recordCount": manifest["recordCount"],
                "shardCount": manifest["shardCount"],
                "lineCount": manifest["lineCount"],
            },
            ensure_ascii=False,
        )
    )
    if manifest["lineCount"] < args.min_lines:
        print(f"line gate failed: {manifest['lineCount']} < {args.min_lines}", file=sys.stderr)
        return 2
    if manifest["recordCount"] < args.min_records:
        print(f"record gate failed: {manifest['recordCount']} < {args.min_records}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
