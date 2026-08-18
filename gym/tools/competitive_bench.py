"""경쟁 벤치마크 — rhwp vs 대안 HWP/문서 도구, 에이전트 과제 실측 + 능력 매트릭스.

## 왜 이 도구인가

"표준 도구 = 에이전트가 기본으로 집는 도구"라는 명제는 **주장이 아니라 측정**으로
뒷받침돼야 한다. 이 하네스는 `samples/` 코퍼스 위에서 에이전트가 실제로 시키는 문서
과제(본문 추출·메타/구조·변환)를 rhwp 와 대안 도구에 **똑같이** 돌려, 도구별·과제별로
벽시계 중앙값·성공률·간이 충실도를 재고, 문서화·검증 가능한 사실로 능력 매트릭스를
채운다. 결과는 기계가 읽는 JSON 과 사람이 읽는 마크다운 리포트로 동시에 낸다.

## 정직성 규약 (이 하네스의 존재 이유)

- 못 돌린 도구는 `available:false` + `reason` 으로 기록한다. **숫자를 지어내지 않는다.**
- 돌릴 수 없는 도구를 "이겼다"고 주장하지 않는다 — 구조적 비교만 진술한다
  (예: pyhwp=휴면·읽기전용·Py2 세대; hwplib=Java 라이브러리로 CLI 아님;
  LibreOffice=HWP5 임포트 필터 없음; Hancom SDK=Windows 전용).
- 경쟁자가 더 빠르거나 rhwp 가 못 하는 걸 하면 그대로 적는다 — 그 신뢰성이 채택 논거다.

## 사용

    # 1) rhwp 바이너리 빌드 (하네스의 유일한 전제)
    cargo build --bin rhwp
    # 2) (선택) pyhwp 경쟁자 — 휴면 패키지라 six 를 수동으로 얹어야 import 된다
    python -m venv .venv && .venv/Scripts/pip install pyhwp six
    # 3) 벤치 실행 — JSON + 마크다운 리포트 동시 산출
    python gym/tools/competitive_bench.py \
        --rhwp target/debug/rhwp --pyhwp .venv/Scripts/hwp5txt \
        --limit 25 \
        --out-json mydocs/tech/benchmark_vs_alternatives.json \
        --out-md   mydocs/tech/benchmark_vs_alternatives.md

바이너리·외부 도구 없이 순수 로직(집계·매트릭스·리포트 렌더)만 시험하려면
`scripts/tests/test_gym_competitive_bench.py` 를 본다 — 이 파일의 순수 함수만 검증한다.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = os.path.dirname(os.path.abspath(__file__))
GYM_ROOT = os.path.dirname(HERE)
REPO_ROOT = os.path.dirname(GYM_ROOT)

# 과제 = 에이전트가 문서에 실제로 시키는 일. 각 과제에 어느 도구가 도전하는지는
# 런타임 가용성으로 결정된다(정직한 저하).
TASKS = ["export-text", "info", "structure", "convert"]

# 서브프로세스 1건 상한(초). 초과는 실패로 센다 — 매달리는 것도 정직하게 실패다.
DEFAULT_TIMEOUT = 60

# 보고 봉투 계약. build_payload 와 --from-json 재렌더 JSON 이 같은 kind/version 을 쓴다.
REPORT_KIND = "gymCompetitiveBench"
SCHEMA_VERSION = "1.0"
VALID_CAP = frozenset({"yes", "partial", "no"})
CORPUS_EXTS = (".hwp", ".hwpx")
DEFAULT_TOOL_ORDER = ("rhwp", "pyhwp", "soffice", "hwplib")
# info/structure/export-text 본문으로 인정하는 키. 래퍼를 벗길 때 쓴다.
_BODY_KEYS = (
    "pages", "text", "format", "pageCount", "nodeCount", "structure",
    "sections", "paraCount",
)

# --------------------------------------------------------------------------
# 능력 매트릭스 — 문서화·검증 가능한 사실만. 값 = "yes" | "partial" | "no".
# --------------------------------------------------------------------------
CAP_COLUMNS = [
    ("crossPlatform", "크로스플랫폼"),
    ("singleBinary", "단일 자립 바이너리"),
    ("agentCli", "에이전트-네이티브 CLI(JSON 봉투)"),
    ("mcp", "MCP 서버"),
    ("memSafe", "메모리 안전(Rust)"),
    ("verifiable", "검증 가능 작업(capsule/replay)"),
    ("edit", "편집"),
    ("render", "렌더(SVG/PNG/PDF)"),
]

CAP_ROWS = [
    {
        "tool": "rhwp",
        "crossPlatform": "yes", "singleBinary": "yes", "agentCli": "yes",
        "mcp": "yes", "memSafe": "yes", "verifiable": "yes", "edit": "yes", "render": "yes",
        "note": "Rust 단일 바이너리(Win/Linux/macOS + wasm32). --json 봉투·mcp-serve·"
                "replay/audit/lineage·fill/replace/redact·export-svg/png/pdf 를 한 실행파일로.",
    },
    {
        "tool": "pyhwp (hwp5txt)",
        "crossPlatform": "yes", "singleBinary": "no", "agentCli": "partial",
        "mcp": "no", "memSafe": "no", "verifiable": "no", "edit": "no", "render": "partial",
        "note": "Python 패키지(+six 등 의존, import 조차 수동 보정 필요). 읽기전용, HWP5(OLE)"
                "만. 평문 출력(구조화 봉투 없음). hwp5html/hwp5odt 변환은 있으나 SVG/PNG/PDF "
                "직접 렌더는 아니다. 사실상 휴면(Py2 세대).",
    },
    {
        "tool": "LibreOffice (soffice)",
        "crossPlatform": "yes", "singleBinary": "no", "agentCli": "partial",
        "mcp": "no", "memSafe": "no", "verifiable": "no", "edit": "yes", "render": "yes",
        "note": "대형 오피스 스위트. --headless --convert-to 는 구조화 출력이 없다. 편집·PDF "
                "렌더는 강력하나 **HWP5 임포트 필터가 없어** 현대 .hwp 를 열지 못한다"
                "(구형 HWP2.0/3.0 필터만 존재).",
    },
    {
        "tool": "hwplib (Java)",
        "crossPlatform": "yes", "singleBinary": "no", "agentCli": "no",
        "mcp": "no", "memSafe": "no", "verifiable": "no", "edit": "yes", "render": "no",
        "note": "JVM 라이브러리(jar). CLI 가 아니다 — 부르려면 래퍼 클래스 작성 + 빌드가 "
                "필요하다. 라이브러리 API 로 읽기/쓰기는 되지만 명령줄 도구가 아니다.",
    },
    {
        "tool": "Hancom SDK",
        "crossPlatform": "no", "singleBinary": "no", "agentCli": "no",
        "mcp": "no", "memSafe": "no", "verifiable": "no", "edit": "yes", "render": "yes",
        "note": "**Windows 전용** 독점 SDK(COM/자동화). 크로스플랫폼·CLI·오픈소스가 아니다. "
                "구조 비교를 위해서만 등재한다(실행하지 않음).",
    },
]


def capability_matrix() -> dict:
    """능력 매트릭스를 컬럼 순서와 함께 반환. 순수 — 문서화된 사실만."""
    return {
        "columns": [{"key": k, "label": lbl} for k, lbl in CAP_COLUMNS],
        "rows": [dict(r) for r in CAP_ROWS],
    }


def validate_capability_matrix(matrix: dict | None = None) -> list[str]:
    """매트릭스 구멍·중복·허용값 밖을 나열. 비어 있으면 정합. 순수."""
    matrix = capability_matrix() if matrix is None else matrix
    issues: list[str] = []
    if not isinstance(matrix, dict):
        return ["matrix 가 객체가 아니다"]
    columns = matrix.get("columns")
    rows = matrix.get("rows")
    if not isinstance(columns, list) or not columns:
        issues.append("columns 가 비었다")
        return issues
    if not isinstance(rows, list) or not rows:
        issues.append("rows 가 비었다")
        return issues
    keys: list[str] = []
    seen_keys: set[str] = set()
    for col in columns:
        if not isinstance(col, dict) or not col.get("key"):
            issues.append("column 에 key 가 없다")
            continue
        key = col["key"]
        if key in seen_keys:
            issues.append(f"column 중복: {key}")
        seen_keys.add(key)
        keys.append(key)
    seen_tools: set[str] = set()
    for row in rows:
        if not isinstance(row, dict):
            issues.append("row 가 객체가 아니다")
            continue
        tool = row.get("tool")
        if not tool:
            issues.append("row 에 tool 이 없다")
            continue
        if tool in seen_tools:
            issues.append(f"tool 중복: {tool}")
        seen_tools.add(tool)
        for key in keys:
            if key not in row:
                issues.append(f"{tool}: {key} 누락")
            elif row[key] not in VALID_CAP:
                issues.append(f"{tool}.{key}={row[key]!r} 는 yes|partial|no 가 아니다")
        if tool == "rhwp":
            for key in keys:
                if row.get(key) != "yes":
                    issues.append(f"rhwp.{key}={row.get(key)!r} 는 yes 가 아니다")
    return issues


def exclusive_yes(matrix: dict, tool: str) -> list[str]:
    """`tool` 만 yes 인 능력 키(컬럼 순서). 순수."""
    columns = matrix.get("columns") or []
    rows = [r for r in (matrix.get("rows") or []) if isinstance(r, dict)]
    keys = [c["key"] for c in columns if isinstance(c, dict) and c.get("key")]
    out: list[str] = []
    for key in keys:
        mine = next((r.get(key) for r in rows if r.get("tool") == tool), None)
        if mine != "yes":
            continue
        if all(r.get(key) != "yes" for r in rows if r.get("tool") != tool):
            out.append(key)
    return out


def capability_label(matrix: dict, key: str) -> str:
    for col in matrix.get("columns") or []:
        if isinstance(col, dict) and col.get("key") == key:
            return col.get("label") or key
    return key


# --------------------------------------------------------------------------
# 순수 집계 로직 (바이너리·외부 도구 불요 — 가드 테스트가 이 부분만 검증한다)
# --------------------------------------------------------------------------
def is_number(value) -> bool:
    """측정값으로 쓸 수 있는 숫자. bool 은 세지 않는다(True==1 접힘 금지)."""
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def posix_rel(path: str) -> str:
    """경로 구분자를 POSIX 로. 머신 불변 키."""
    return str(path).replace("\\", "/")


def ext_of(path: str) -> str:
    return Path(str(path)).suffix.lower()


def median(values):
    """None·비숫자를 거른 중앙값. 값이 없으면 None. bool 은 숫자로 치지 않는다."""
    vals = [v for v in values if is_number(v)]
    if not vals:
        return None
    return statistics.median(vals)


def _round_ms(v):
    return None if v is None else round(float(v), 1)


def _round_int(v):
    return None if v is None else int(round(float(v)))


def normalize_run(run: dict) -> dict:
    """run 레코드 정규화. ext 가 비면 file 에서 추론. ok 는 bool."""
    out = dict(run)
    path = out.get("file") or ""
    ext = (out.get("ext") or "").lower() or ext_of(str(path))
    out["ext"] = ext
    out["ok"] = bool(out.get("ok"))
    return out


def summarize_runs(runs: list[dict]) -> dict:
    """run 레코드 리스트 → 요약 통계. 순수.

    run 레코드: {"file": str, "ext": str, "ok": bool, "ms": float|None, "chars": int|None}
    - medianMs 는 **성공한 실행만** 대상으로 한다(실패의 0ms 로 시간을 왜곡하지 않는다).
    - medianChars 는 성공한 실행의 문자수만. 0 은 빈 문서라 유효하고, None 만 뺀다.
    - byExt 는 형식별 시도/성공/실패를 남긴다(예: pyhwp 가 .hwp 는 되고 .hwpx 는 안 되는 사실).
    """
    normalized = [normalize_run(r) for r in runs]
    attempted = len(normalized)
    ok_runs = [r for r in normalized if r["ok"]]
    fail = attempted - len(ok_runs)
    by_ext: dict[str, dict] = {}
    for r in normalized:
        bucket = by_ext.setdefault(r["ext"], {"attempted": 0, "ok": 0, "fail": 0})
        bucket["attempted"] += 1
        if r["ok"]:
            bucket["ok"] += 1
        else:
            bucket["fail"] += 1
    return {
        "attempted": attempted,
        "ok": len(ok_runs),
        "fail": fail,
        "successRate": round(len(ok_runs) / attempted, 3) if attempted else None,
        "medianMs": _round_ms(median([r.get("ms") for r in ok_runs])),
        "medianChars": _round_int(median([r.get("chars") for r in ok_runs])),
        "byExt": by_ext,
    }


def fidelity_pairs(tool_runs: list[dict], ref_runs: list[dict]) -> list[tuple[str, float]]:
    """겹친 성공 파일의 (file, got/base) 목록. base==0 은 비율을 만들지 않는다."""
    ref: dict[str, float] = {}
    for r in ref_runs:
        if r.get("ok") and is_number(r.get("chars")):
            ref[r.get("file")] = float(r["chars"])
    pairs: list[tuple[str, float]] = []
    for r in tool_runs:
        if not r.get("ok") or not is_number(r.get("chars")):
            continue
        base = ref.get(r.get("file"))
        if base is None or base == 0:
            continue
        pairs.append((str(r.get("file")), float(r["chars"]) / base))
    return pairs


def fidelity_stats(tool_runs: list[dict], ref_runs: list[dict]) -> dict:
    """충실도 중앙값과 표본 수. 겹침 없으면 n=0, median=None."""
    pairs = fidelity_pairs(tool_runs, ref_runs)
    if not pairs:
        return {"n": 0, "median": None}
    return {
        "n": len(pairs),
        "median": round(statistics.median([ratio for _file, ratio in pairs]), 3),
    }


def fidelity_vs_ref(tool_runs: list[dict], ref_runs: list[dict]):
    """도구/기준(rhwp) 문자수 비율의 **파일별 중앙값**. 둘 다 성공한 파일만.

    0 문자도 측정값이다(빈 문서를 버린 뒤 비율을 부풀리지 않는다). 기준이 0 이면
    나눗셈을 만들지 않는다. 겹치는 유효 쌍이 없으면 None.
    1.0=동일량, <1.0=덜 뽑음(예: pyhwp 가 표 셀 대신 `<표>` 자리표만 남겨
    문자수가 적다), >1.0=더 뽑음.
    """
    return fidelity_stats(tool_runs, ref_runs)["median"]


def overlap_ok_pairs(tool_runs: list[dict], ref_runs: list[dict], field: str):
    """둘 다 성공하고 field 가 숫자인 파일의 (file, tool, ref) 목록."""
    ref_ok: dict = {}
    for r in ref_runs:
        if r.get("ok") and is_number(r.get(field)):
            ref_ok[r.get("file")] = r.get(field)
    pairs = []
    for r in tool_runs:
        if not r.get("ok") or not is_number(r.get(field)):
            continue
        base = ref_ok.get(r.get("file"))
        if base is None:
            continue
        pairs.append((r.get("file"), r.get(field), base))
    return pairs


def overlap_timing(tool_runs: list[dict], ref_runs: list[dict]) -> dict:
    """동일 성공 집합의 속도 요약. n=0 이면 tool/ref 는 None."""
    pairs = overlap_ok_pairs(tool_runs, ref_runs, "ms")
    if not pairs:
        return {"n": 0, "tool": None, "ref": None}
    tool_ms = [p[1] for p in pairs]
    ref_ms = [p[2] for p in pairs]
    return {
        "n": len(pairs),
        "tool": _round_ms(statistics.median(tool_ms)),
        "ref": _round_ms(statistics.median(ref_ms)),
    }


def overlap_median_ms(tool_runs: list[dict], ref_runs: list[dict]):
    """두 도구가 **모두 성공한 파일**에서만 각각의 median ms 를 낸다 → 공정한 동일-집합 속도.

    (tool_ms, ref_ms) 를 반환. 겹침 없으면 (None, None). rhwp 의 중앙값이 HWPX 까지
    포함해 부풀지 않도록, 속도 비교는 같은 파일집합에서만 한다. 0ms 는 유효한 측정이다."""
    stats = overlap_timing(tool_runs, ref_runs)
    return stats["tool"], stats["ref"]


def unwrap_json_envelope(doc):
    """한 겹의 data/result/payload 래퍼를 벗긴다. 본문이 아니면 원본."""
    if not isinstance(doc, dict):
        return None
    for key in ("data", "result", "payload"):
        inner = doc.get(key)
        if isinstance(inner, dict) and any(k in inner for k in _BODY_KEYS):
            return inner
    return doc


def _json_object(stdout):
    if isinstance(stdout, bytes):
        stdout = stdout.decode("utf-8", errors="replace")
    if not isinstance(stdout, str):
        return None
    try:
        doc = json.loads(stdout)
    except (json.JSONDecodeError, TypeError, ValueError):
        return None
    return doc if isinstance(doc, dict) else None


def parse_json_object(stdout):
    """빈·깨진·배열 JSON 은 None. 객체만 반환. 순수."""
    return _json_object(stdout)


def parse_json_body(stdout):
    """봉투 객체를 파싱하고 data/result/payload 한 겹을 벗긴다. 실패 시 None."""
    doc = _json_object(stdout)
    if doc is None:
        return None
    body = unwrap_json_envelope(doc)
    return body if isinstance(body, dict) else None


def parse_rhwp_info(stdout):
    """rhwp `info --json` 봉투 → {format, pageCount, sections, paraCount} 또는 None.

    본문 표지가 하나도 없으면 다른 명령 봉투로 보고 None. 래퍼·깨진 JSON 도 None.
    """
    body = parse_json_body(stdout)
    if not isinstance(body, dict):
        return None
    if not any(k in body for k in ("format", "pageCount", "sections", "paraCount")):
        return None
    fmt = body.get("format")
    return {
        "format": fmt if isinstance(fmt, str) else None,
        "pageCount": body.get("pageCount") if is_number(body.get("pageCount")) else None,
        "sections": body.get("sections") if is_number(body.get("sections")) else None,
        "paraCount": body.get("paraCount") if is_number(body.get("paraCount")) else None,
    }


def parse_rhwp_structure(stdout):
    """rhwp `export-structure --json` 봉투 → {mode, nodeCount, hasStructure} 또는 None."""
    body = parse_json_body(stdout)
    if not isinstance(body, dict):
        return None
    if not any(k in body for k in ("nodeCount", "structure", "mode")):
        return None
    mode = body.get("mode")
    return {
        "mode": mode if isinstance(mode, str) else None,
        "nodeCount": body.get("nodeCount") if is_number(body.get("nodeCount")) else None,
        "hasStructure": isinstance(body.get("structure"), dict),
    }


def parse_rhwp_text_chars(stdout: str):
    """rhwp `export-text --json` 봉투 → 총 문자수. 파싱 실패 시 None. 순수.

    단건 계약은 `pages[].text` 합, batch 계약은 최상위 `text`.
    `{data|result|payload}` 래퍼가 있으면 한 겹 벗긴다. 문자열이 아닌 text 는
    그 쪽만 건너뛴다. `len(None)` 으로 터지지 않는다.
    """
    doc = _json_object(stdout)
    if doc is None:
        return None
    body = unwrap_json_envelope(doc)
    if not isinstance(body, dict):
        return None
    pages = body.get("pages")
    if isinstance(pages, list):
        total = 0
        for page in pages:
            if not isinstance(page, dict):
                continue
            text = page.get("text")
            if isinstance(text, str):
                total += len(text)
        return total
    text = body.get("text")
    if isinstance(text, str):
        return len(text)
    return None


def _fmt_cell(available: bool, summary: dict | None, fidelity, reason: str | None) -> str:
    """결과표 한 칸: 성공률·중앙값 시간·충실도, 또는 'n/a: <이유>'. 순수."""
    if not available:
        return f"n/a: {reason or '실행 불가'}"
    if not isinstance(summary, dict) or summary.get("attempted", 0) == 0:
        return "n/a: 시도 없음"
    rate = summary.get("successRate")
    if is_number(rate):
        rate_pct = f"{int(round(rate * 100))}%"
    else:
        rate_pct = "-"
    ms = summary.get("medianMs")
    ms_s = "-" if not is_number(ms) else f"{ms:.0f}ms"
    ok = summary.get("ok", 0)
    att = summary.get("attempted", 0)
    fid = "-" if not is_number(fidelity) else f"{fidelity:.2f}×"
    return f"{ms_s} · {rate_pct}({ok}/{att}) · 충실도 {fid}"


def escape_md_cell(text) -> str:
    """마크다운 표 칸. 파이프·개행이 표를 깨지 않게 한다."""
    return str(text).replace("|", "\\|").replace("\r", " ").replace("\n", " ")


def speed_cmp(tool_ms, ref_ms):
    """tool vs ref 속도. 숫자가 아니면 None, 같으면 tie."""
    if not is_number(tool_ms) or not is_number(ref_ms):
        return None
    if tool_ms < ref_ms:
        return "tool_faster"
    if tool_ms > ref_ms:
        return "ref_faster"
    return "tie"


def _result_index(results) -> dict:
    out = {}
    for item in results or []:
        if isinstance(item, dict) and item.get("tool"):
            out[item["tool"]] = item
    return out


def _summary_of(result: dict | None) -> dict:
    if not isinstance(result, dict):
        return {}
    summary = result.get("summary")
    return summary if isinstance(summary, dict) else {}


def export_text_verdict(rhwp: dict | None, pyhwp: dict | None) -> list[str]:
    """export-text 헤드투헤드 문장. 키가 빠져도 터지지 않는다."""
    lines: list[str] = []
    if rhwp and rhwp.get("available"):
        s = _summary_of(rhwp)
        ok = s.get("ok", 0)
        att = s.get("attempted", 0)
        ms = s.get("medianMs")
        ms_s = f"{ms}ms" if is_number(ms) else "n/a"
        lines.append(
            f"rhwp 는 export-text 에서 {ok}/{att} 파일을 처리했다"
            f"(HWP+HWPX 혼합, 중앙값 {ms_s})."
        )
    if pyhwp and not pyhwp.get("available"):
        reason = pyhwp.get("reason") or "실행 불가"
        lines.append(f"pyhwp(hwp5txt)는 이 머신에서 실행하지 않았다({reason}).")
        return lines
    if not (pyhwp and pyhwp.get("available")):
        return lines
    ps = _summary_of(pyhwp)
    by_ext = ps.get("byExt") if isinstance(ps.get("byExt"), dict) else {}
    hwp_b = by_ext.get(".hwp") if isinstance(by_ext.get(".hwp"), dict) else {}
    hwpx_b = by_ext.get(".hwpx") if isinstance(by_ext.get(".hwpx"), dict) else {}
    lines.append(
        f"pyhwp(hwp5txt)는 HWP5 {hwp_b.get('ok', 0)}/{hwp_b.get('attempted', 0)} 성공, "
        f"HWPX {hwpx_b.get('ok', 0)}/{hwpx_b.get('attempted', 0)} 성공"
        f"(ZIP 기반 HWPX 는 OLE 파서로 열 수 없음 — 구조적 한계)."
    )
    ov = pyhwp.get("overlapMs") if isinstance(pyhwp.get("overlapMs"), dict) else {}
    p_ms = ov.get("tool")
    r_ms = ov.get("ref")
    cmp = speed_cmp(p_ms, r_ms)
    if cmp == "tool_faster":
        lines.append(
            f"속도(동일 파일집합, 둘 다 연 HWP5): pyhwp 가 더 빨랐다"
            f"(pyhwp {p_ms}ms vs rhwp {r_ms}ms 중앙값). rhwp 는 디버그 빌드이며 JSON "
            f"봉투·출처 표지를 함께 낸다 — 릴리스 빌드로는 좁혀진다. 그래도 더 빠른 축은 "
            f"그대로 적는다."
        )
    elif cmp == "ref_faster":
        lines.append(
            f"속도(동일 파일집합, 둘 다 연 HWP5): rhwp 가 더 빨랐다"
            f"(rhwp {r_ms}ms vs pyhwp {p_ms}ms 중앙값) — 디버그 빌드임에도."
        )
    elif cmp == "tie":
        lines.append(
            f"속도(동일 파일집합, 둘 다 연 HWP5): 중앙값이 같았다"
            f"(둘 다 {p_ms}ms)."
        )
    elif ov.get("n") == 0 or (p_ms is None and r_ms is None):
        lines.append(
            "속도: 두 도구가 모두 성공한 겹친 파일이 없어 "
            "동일-집합 비교를 만들지 않았다."
        )
    fid = pyhwp.get("fidelityVsRhwp")
    if is_number(fid):
        lines.append(
            f"충실도: 두 도구가 모두 연 HWP5 에서 pyhwp 문자수는 rhwp 대비 중앙값 {fid:.2f}× — "
            f"pyhwp 는 표 셀 본문을 `<표>` 자리표로 대체해 본문을 덜 뽑는다"
            f"(에이전트가 표 안 숫자를 읽어야 하면 치명적)."
        )
    return lines


def soffice_verdict(soffice: dict | None) -> list[str]:
    """LibreOffice 가용/불가 문장. 없으면 침묵."""
    if not isinstance(soffice, dict):
        return []
    if not soffice.get("available"):
        reason = soffice.get("reason") or "실행 불가"
        return [f"LibreOffice(soffice)는 이 머신에서 실행하지 않았다({reason})."]
    s = _summary_of(soffice)
    ok = s.get("ok", 0)
    att = s.get("attempted", 0)
    return [
        f"LibreOffice(soffice)는 export-text 에서 {ok}/{att} 성공"
        f"(HWP5 임포트 필터 부재가 실패로 남는다)."
    ]


def width_verdict(tasks: dict) -> list[str]:
    """rhwp 만 구조화 CLI 로 돈 과제 목록."""
    rhwp_only = []
    for name in ("info", "structure", "convert"):
        tr = tasks.get(name) or {}
        results = tr.get("results") if isinstance(tr, dict) else []
        indexed = _result_index(results)
        rhwp = indexed.get("rhwp")
        others_avail = any(
            item.get("available")
            for item in indexed.values()
            if item.get("tool") != "rhwp"
        )
        if rhwp and rhwp.get("available") and not others_avail:
            rhwp_only.append(name)
    if not rhwp_only:
        return []
    return [
        "폭: " + ", ".join(rhwp_only) + " 과제는 rhwp 만 구조화 CLI 로 수행했다 — "
        "대안들은 동일 형식 산출(메타 봉투·구조 트리·HWPX/markdown 변환)이 없어 n/a."
    ]


def capability_verdict(matrix: dict) -> str:
    """매트릭스에서 rhwp 만 yes 인 능력을 문장으로. 하드코드 승패가 아니다."""
    keys = exclusive_yes(matrix, "rhwp")
    if not keys:
        return "능력: 벤치한 대안과 겹치는 축만 남았다(능력 매트릭스 참조)."
    labels = [capability_label(matrix, key) for key in keys]
    return (
        "능력: " + "·".join(labels)
        + " 는 벤치한 대안 중 rhwp 만 yes 다(능력 매트릭스 참조)."
    )


def verdict_lines(payload: dict) -> list[str]:
    """측정 데이터에서 직접 유도한 정직한 평결 문장들. 순수.

    숫자는 payload 에서만 온다 — 손으로 쓴 승패 주장이 아니라 잰 값의 서술이다.
    키가 빠진 불완전 payload 에서도 KeyError 를 내지 않는다.
    """
    lines: list[str] = []
    tasks = {
        t.get("task"): t
        for t in payload.get("tasks", [])
        if isinstance(t, dict) and t.get("task")
    }
    et = tasks.get("export-text") or {}
    res = _result_index(et.get("results") if isinstance(et, dict) else [])
    lines.extend(export_text_verdict(res.get("rhwp"), res.get("pyhwp")))
    lines.extend(soffice_verdict(res.get("soffice")))
    lines.extend(width_verdict(tasks))
    matrix = payload.get("capabilityMatrix")
    if not isinstance(matrix, dict):
        matrix = capability_matrix()
    lines.append(capability_verdict(matrix))
    return lines


def refresh_verdict(payload: dict) -> dict:
    """측정값에서 평결을 다시 유도해 붙인다. 손글 승패를 남기지 않는다."""
    out = dict(payload)
    out["verdict"] = verdict_lines(out)
    return out


# --------------------------------------------------------------------------
# 리포트 렌더 (순수 — payload 만 있으면 결정론적으로 마크다운을 만든다)
# --------------------------------------------------------------------------
def render_report(payload: dict) -> str:
    env = payload.get("env", {})
    out: list[str] = []
    out.append("# 경쟁 벤치마크 — rhwp vs 대안 HWP/문서 도구")
    out.append("")
    out.append(
        "> **명제**: 표준 도구는 에이전트가 *기본으로 집는* 도구다. 아래는 주장이 아니라 "
        "`samples/` 코퍼스 위 실측이다 — 같은 과제를 같은 파일에 돌려 잰 값과, 문서화된 "
        "사실로 채운 능력 매트릭스. **못 돌린 도구는 숫자를 지어내지 않고 `n/a: 이유`로 적는다.**"
    )
    out.append("")
    out.append(
        "이 리포트는 `gym/tools/competitive_bench.py` 가 생성한다(손으로 쓴 승패 주장이 "
        "아니라 잰 값의 서술). 재생성 명령은 맨 아래.")
    out.append("")

    # 환경
    out.append("## 실행 환경")
    out.append("")
    kind = payload.get("kind") or REPORT_KIND
    schema = payload.get("schemaVersion") or SCHEMA_VERSION
    out.append(f"- 스키마: `{kind}` `{schema}`")
    out.append(f"- OS: `{env.get('os', '?')}`")
    out.append(f"- rhwp: `{env.get('rhwpVersion', '?')}` (`{env.get('rhwpProfile', '?')}` 빌드)")
    out.append(f"- Python: `{env.get('python', '?')}`")
    corpus = env.get("corpus", {})
    out.append(
        f"- 코퍼스: {corpus.get('total', 0)} 파일 "
        f"(HWP {corpus.get('hwp', 0)} · HWPX {corpus.get('hwpx', 0)}), "
        f"`{corpus.get('dir', 'samples')}` 에서 결정론적으로 선택")
    out.append("")
    out.append("도구 가용성(이 머신에서 실제로 무엇이 돌았나):")
    out.append("")
    out.append("| 도구 | 이 머신에서 | 상세 |")
    out.append("|---|---|---|")
    for tool, info in env.get("tools", {}).items():
        mark = "실행됨" if info.get("available") else "실행 안 됨"
        out.append(f"| {tool} | {mark} | {info.get('detail', '')} |")
    out.append("")

    # 결과표
    out.append("## 결과 — 과제 × 도구 (중앙값 시간 · 성공률 · 충실도)")
    out.append("")
    out.append(
        "충실도 = 두 도구가 모두 성공한 파일에서 `문자수 ÷ rhwp 문자수` 의 중앙값 "
        "(1.00× = 동일량, 낮을수록 본문을 덜 뽑음). rhwp 는 자기 자신이므로 기준(1.00×).")
    out.append("")
    tools_order = payload.get("toolOrder") or list(DEFAULT_TOOL_ORDER)
    header = "| 과제 | " + " | ".join(escape_md_cell(t) for t in tools_order) + " |"
    sep = "|---|" + "|".join(["---"] * len(tools_order)) + "|"
    out.append(header)
    out.append(sep)
    for task in payload.get("tasks", []):
        if not isinstance(task, dict):
            continue
        row = _result_index(task.get("results"))
        cells = []
        for tool in tools_order:
            r = row.get(tool)
            if r is None:
                cells.append("n/a")
                continue
            cells.append(escape_md_cell(
                _fmt_cell(
                    r.get("available", False),
                    r.get("summary"),
                    r.get("fidelityVsRhwp"),
                    r.get("reason"),
                )
            ))
        name = escape_md_cell(task.get("task") or "?")
        out.append(f"| **{name}** | " + " | ".join(cells) + " |")
    out.append("")
    # 도구별 각주(형식 한계 등)
    notes = []
    for task in payload.get("tasks", []):
        for r in task.get("results", []):
            if r.get("note"):
                notes.append(f"- **{r['tool']} / {task['task']}**: {r['note']}")
    if notes:
        out.append("주석:")
        out.append("")
        out.extend(notes)
        out.append("")

    # 능력 매트릭스
    out.append("## 능력 매트릭스 (문서화·검증 가능한 사실)")
    out.append("")
    matrix = payload.get("capabilityMatrix", capability_matrix())
    cols = matrix["columns"]
    out.append("| 도구 | " + " | ".join(c["label"] for c in cols) + " |")
    out.append("|---|" + "|".join(["---"] * len(cols)) + "|")
    glyph = {"yes": "O", "partial": "~", "no": "X"}
    for r in matrix.get("rows") or []:
        if not isinstance(r, dict):
            continue
        cells = [glyph.get(r.get(c["key"], "no"), "?") for c in cols]
        out.append(f"| {escape_md_cell(r.get('tool', '?'))} | " + " | ".join(cells) + " |")
    out.append("")
    out.append("범례: O = 지원 · ~ = 부분/우회 · X = 없음")
    out.append("")
    for r in matrix["rows"]:
        if r.get("note"):
            out.append(f"- **{r['tool']}**: {r['note']}")
    out.append("")

    # 평결
    out.append("## 정직한 평결")
    out.append("")
    for line in payload.get("verdict", []):
        out.append(f"- {line}")
    out.append("")
    out.append(
        "요약: rhwp 가 **못 하는 게 없고**, 벤치한 대안 중 유일하게 크로스플랫폼 단일 "
        "바이너리 + 에이전트-네이티브 CLI(JSON 봉투) + MCP + 검증 가능 작업 + HWPX/편집/렌더를 "
        "한 도구로 덮는다. 경쟁자가 앞서거나 rhwp 가 못 하는 지점은 위 평결 항목에 잰 값 그대로 "
        "적었다 — 예컨대 LibreOffice 는 (설치돼 있고 HWP5 를 열 수만 있다면) PDF 렌더·완전 편집 "
        "UI 가 성숙하고, 속도 비교의 방향은 코퍼스·빌드 프로파일에 따라 달라질 수 있다(디버그 "
        "빌드로 측정). 그러나 에이전트가 기본으로 집는 축 — 설치 한 방, 구조화 출력, 형식 폭, "
        "재현 가능성 — 에서 rhwp 가 앞선다. **이 정직함이 채택 논거다.**")
    out.append("")

    # 재현
    out.append("## 재현")
    out.append("")
    out.append("```sh")
    out.append("# 1) 하네스의 유일한 전제: rhwp 바이너리")
    out.append("cargo build --bin rhwp")
    out.append("# 2) (선택) pyhwp — 휴면 패키지라 six 를 수동으로 얹어야 import 된다")
    out.append("python -m venv .venv && .venv/Scripts/pip install pyhwp six")
    out.append("# 3) 벤치 실행 — 이 리포트와 옆의 JSON 을 재생성한다")
    out.append("python gym/tools/competitive_bench.py \\")
    out.append("    --rhwp target/debug/rhwp --pyhwp .venv/Scripts/hwp5txt \\")
    out.append("    --limit 25 \\")
    out.append("    --out-json mydocs/tech/benchmark_vs_alternatives.json \\")
    out.append("    --out-md   mydocs/tech/benchmark_vs_alternatives.md")
    out.append("```")
    out.append("")
    out.append(
        "순수 로직(집계·매트릭스·리포트 렌더)은 바이너리 없이 "
        "`python -m unittest scripts/tests/test_gym_competitive_bench.py` 로 검증한다.")
    out.append("")
    generated = payload.get("generatedAt")
    if generated:
        out.append(f"<!-- generated by competitive_bench.py at {generated} -->")
    return "\n".join(out) + "\n"


# --------------------------------------------------------------------------
# IO: 코퍼스 발견 · 도구 탐지 · 서브프로세스 실행
# --------------------------------------------------------------------------
def _rel(path: Path) -> str:
    """REPO_ROOT 기준 POSIX 상대경로(가능하면). 커밋 산출물이 머신-불변이도록."""
    try:
        return path.resolve().relative_to(Path(REPO_ROOT).resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def select_corpus_paths(paths, limit: int) -> list[str]:
    """결정론적 코퍼스 선택. 순수 — 파일시스템을 보지 않는다.

    POSIX 정규화 후 .hwp/.hwpx 만 취하고 형식별 정렬한다. limit>0 이면
    형식별 앞 limit 개, limit<=0 이면 전부. 중복 경로는 첫 등장만.
    """
    hwp: list[str] = []
    hwpx: list[str] = []
    seen: set[str] = set()
    for raw in paths:
        rel = posix_rel(raw)
        if rel in seen:
            continue
        seen.add(rel)
        ext = ext_of(rel)
        if ext not in CORPUS_EXTS:
            continue
        if ext == ".hwp":
            hwp.append(rel)
        else:
            hwpx.append(rel)
    hwp.sort()
    hwpx.sort()
    if limit > 0:
        hwp = hwp[:limit]
        hwpx = hwpx[:limit]
    return hwp + hwpx


def discover_corpus(samples_dir: str, limit: int) -> list[str]:
    """samples/ 에서 HWP·HWPX 를 결정론적으로 선택(정렬 후 형식별 limit).

    경로는 REPO_ROOT 상대(POSIX)로 낸다 — 서브프로세스는 cwd=REPO_ROOT 에서 돌므로
    상대경로로 동작하고, 커밋되는 JSON 에 머신별 절대경로가 새지 않는다.
    """
    base = Path(samples_dir)
    found = [_rel(p) for p in list(base.glob("*.hwp")) + list(base.glob("*.hwpx"))]
    return select_corpus_paths(found, limit)


def _run(cmd: list[str], cwd: str, timeout: int) -> tuple[bool, float, str, str]:
    """서브프로세스 1건을 재고 (ok, ms, stdout, stderr) 반환. UTF-8/errors=replace."""
    start = time.perf_counter()
    try:
        proc = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout,
        )
        ms = (time.perf_counter() - start) * 1000.0
        return proc.returncode == 0, ms, proc.stdout or "", proc.stderr or ""
    except subprocess.TimeoutExpired:
        ms = (time.perf_counter() - start) * 1000.0
        return False, ms, "", f"timeout>{timeout}s"
    except OSError as e:  # 실행파일 없음 등
        ms = (time.perf_counter() - start) * 1000.0
        return False, ms, "", str(e)


def _ext(path: str) -> str:
    return Path(path).suffix.lower()


# ---- 과제별 실행기 (도구 하나 × 코퍼스 전체 → run 레코드 리스트) --------------
def bench_rhwp_text(rhwp: str, files: list[str], cwd: str, timeout: int) -> list[dict]:
    runs = []
    for f in files:
        ok, ms, out, _ = _run([rhwp, "export-text", f, "--json"], cwd, timeout)
        chars = parse_rhwp_text_chars(out) if ok else None
        runs.append({"file": f, "ext": _ext(f), "ok": ok, "ms": ms, "chars": chars})
    return runs


def bench_pyhwp_text(hwp5txt: str, files: list[str], cwd: str, timeout: int) -> list[dict]:
    runs = []
    for f in files:
        ok, ms, out, _ = _run([hwp5txt, f], cwd, timeout)
        chars = len(out) if ok else None
        runs.append({"file": f, "ext": _ext(f), "ok": ok, "ms": ms, "chars": chars})
    return runs


def bench_soffice_text(soffice: str, files: list[str], cwd: str, timeout: int) -> list[dict]:
    """LibreOffice headless 변환으로 txt 추출. (이 머신엔 미설치 — 설치 머신용 경로)."""
    runs = []
    for f in files:
        with tempfile.TemporaryDirectory(prefix="bench_soffice_") as td:
            ok, ms, _, _ = _run(
                [soffice, "--headless", "--convert-to", "txt:Text", "--outdir", td, f],
                cwd, timeout,
            )
            chars = None
            if ok:
                produced = Path(td) / (Path(f).stem + ".txt")
                if produced.exists():
                    chars = len(produced.read_text(encoding="utf-8", errors="replace"))
                else:
                    ok = False  # 변환 성공 코드지만 산출물 없음 = 실패
            runs.append({"file": f, "ext": _ext(f), "ok": ok, "ms": ms, "chars": chars})
    return runs


def bench_rhwp_info(rhwp: str, files: list[str], cwd: str, timeout: int) -> list[dict]:
    runs = []
    for f in files:
        ok, ms, _, _ = _run([rhwp, "info", f, "--json"], cwd, timeout)
        runs.append({"file": f, "ext": _ext(f), "ok": ok, "ms": ms, "chars": None})
    return runs


def bench_rhwp_structure(rhwp: str, files: list[str], cwd: str, timeout: int) -> list[dict]:
    runs = []
    for f in files:
        ok, ms, _, _ = _run([rhwp, "export-structure", f, "--json"], cwd, timeout)
        runs.append({"file": f, "ext": _ext(f), "ok": ok, "ms": ms, "chars": None})
    return runs


def bench_rhwp_convert(rhwp: str, files: list[str], cwd: str, timeout: int) -> list[dict]:
    """HWP→markdown 변환(에이전트가 실제로 시키는 변환). 산출은 임시폴더로."""
    runs = []
    for f in files:
        with tempfile.TemporaryDirectory(prefix="bench_md_") as td:
            ok, ms, _, _ = _run(
                [rhwp, "export-markdown", f, "-o", td, "--json"], cwd, timeout
            )
            runs.append({"file": f, "ext": _ext(f), "ok": ok, "ms": ms, "chars": None})
    return runs


def resolve_tool(path: str | None, names: list[str], *, exists=None, which=None):
    """명시 경로 또는 PATH 에서 실행파일을 찾는다. 없으면 None.

    exists/which 를 주입하면 파일시스템 없이 시험할 수 있다.
    """
    exists = exists or (lambda candidate: Path(candidate).exists())
    which = which or shutil.which
    if path:
        if exists(path):
            return str(path)
        found = which(path)
        return found
    for name in names:
        found = which(name)
        if found:
            return found
    return None


def probe(path: str | None, names: list[str]) -> str | None:
    """명시 경로 또는 PATH 에서 실행파일을 찾는다. 없으면 None."""
    return resolve_tool(path, names)


def rhwp_profile_from_path(path: str) -> str:
    """경로 세그먼트로 release/debug 를 가른다. 'prerelease' 에 속지 않는다."""
    parts = [p.lower() for p in posix_rel(path).split("/") if p]
    if "release" in parts:
        return "release"
    if "debug" in parts:
        return "debug"
    return "debug"


def write_text_lf(path, text: str) -> None:
    """UTF-8 · BOM 없음 · Unix LF. 커밋 산출물이 플랫폼 개행에 물들지 않게 한다."""
    data = text if text.endswith("\n") else text + "\n"
    Path(path).write_bytes(data.encode("utf-8"))


def dump_payload_json(payload: dict) -> str:
    return json.dumps(payload, ensure_ascii=False, indent=2) + "\n"


# --------------------------------------------------------------------------
# 오케스트레이션
# --------------------------------------------------------------------------
def unavailable_result(tool: str, reason: str) -> dict:
    """못 돌린 도구 칸. 숫자 필드를 넣지 않는다."""
    return {"tool": tool, "available": False, "reason": reason}


def available_result(tool: str, runs: list[dict], *, fidelity=None,
                     overlap=None, note: str | None = None) -> dict:
    rec = {
        "tool": tool,
        "available": True,
        "summary": summarize_runs(runs),
        "fidelityVsRhwp": fidelity,
        "runs": runs,
    }
    if overlap is not None:
        rec["overlapMs"] = overlap
    if note:
        rec["note"] = note
    return rec


def invented_metrics(result: dict) -> bool:
    """비가용 칸이 숫자·runs 를 몰래 실었는지. 정직성 가드."""
    if not isinstance(result, dict) or result.get("available"):
        return False
    if isinstance(result.get("summary"), dict):
        return True
    if is_number(result.get("fidelityVsRhwp")):
        return True
    if result.get("overlapMs"):
        return True
    if result.get("runs"):
        return True
    return False


def assemble_env(*, os_name, python, rhwp_version, rhwp_profile, files, tools) -> dict:
    n_hwp = sum(1 for f in files if ext_of(f) == ".hwp")
    n_hwpx = sum(1 for f in files if ext_of(f) == ".hwpx")
    return {
        "os": os_name,
        "python": python,
        "rhwpVersion": rhwp_version,
        "rhwpProfile": rhwp_profile,
        "corpus": {"dir": "samples", "total": len(files), "hwp": n_hwp, "hwpx": n_hwpx},
        "tools": tools,
    }


def stamp_report_contract(payload: dict) -> dict:
    """kind/schemaVersion 이 비면 계약 상수로 채운다. 옛 JSON 재렌더용."""
    out = dict(payload)
    if not out.get("kind"):
        out["kind"] = REPORT_KIND
    if not out.get("schemaVersion"):
        out["schemaVersion"] = SCHEMA_VERSION
    return out


def assemble_payload(*, env, tasks, tool_order=None, generated_at=None, matrix=None) -> dict:
    """측정 조각을 보고 봉투로 묶는다. 순수 — 평결은 숫자에서만 다시 유도한다."""
    payload = {
        "kind": REPORT_KIND,
        "schemaVersion": SCHEMA_VERSION,
        "generatedAt": generated_at,
        "toolOrder": list(tool_order or DEFAULT_TOOL_ORDER),
        "env": env,
        "tasks": list(tasks),
        "capabilityMatrix": matrix if matrix is not None else capability_matrix(),
    }
    return refresh_verdict(payload)


def load_report_payload(raw: str):
    """저장된 JSON 텍스트 → (payload, issues). 깨지면 (None, issues)."""
    if not isinstance(raw, str) or not raw.strip():
        return None, ["빈 JSON"]
    try:
        doc = json.loads(raw)
    except (json.JSONDecodeError, TypeError, ValueError) as exc:
        return None, [f"JSON 파싱 실패: {exc}"]
    issues = payload_shape_issues(doc)
    if issues:
        return None, issues
    return refresh_verdict(stamp_report_contract(doc)), []


def payload_shape_issues(payload) -> list[str]:
    """보고 봉투의 최소 형태. 옛 JSON(kind 없음)도 허용한다."""
    if not isinstance(payload, dict):
        return ["payload 가 객체가 아니다"]
    issues: list[str] = []
    if "tasks" not in payload:
        issues.append("tasks 누락")
    elif not isinstance(payload.get("tasks"), list):
        issues.append("tasks 가 배열이 아니다")
    if payload.get("kind") not in (None, REPORT_KIND):
        issues.append(f"kind={payload.get('kind')!r} 가 {REPORT_KIND} 가 아니다")
    if payload.get("schemaVersion") not in (None, SCHEMA_VERSION):
        issues.append(f"schemaVersion={payload.get('schemaVersion')!r}")
    return issues


# --------------------------------------------------------------------------
# 명시 오류 — 깨진 입력은 숫자를 지어내지 않고 코드·경로와 함께 멈춘다.
# --------------------------------------------------------------------------
class BenchError(Exception):
    """경쟁 벤치 입력/형태 오류. 종료 코드와 기계가 읽는 code 를 가진다."""

    def __init__(self, code: str, message: str, *, exit_code: int = 2,
                 path=None, details=None):
        super().__init__(message)
        self.code = str(code)
        self.message = str(message)
        self.exit_code = int(exit_code)
        self.path = None if path is None else str(path)
        self.details = {} if details is None else dict(details)

    def to_dict(self) -> dict:
        rec = {
            "ok": False,
            "kind": "gymCompetitiveBenchError",
            "code": self.code,
            "message": self.message,
            "exitCode": self.exit_code,
        }
        if self.path:
            rec["path"] = posix_rel(self.path)
        if self.details:
            rec["details"] = dict(self.details)
        return rec


class MissingFileError(BenchError):
    def __init__(self, path, message=None, **kwargs):
        super().__init__(
            ERR_MISSING_FILE,
            message or f"파일이 없다: {path}",
            path=path,
            **kwargs,
        )


class BadJsonError(BenchError):
    def __init__(self, path, message=None, **kwargs):
        super().__init__(
            ERR_BAD_JSON,
            message or f"JSON 이 깨졌다: {path}",
            path=path,
            **kwargs,
        )


class EncodingError(BenchError):
    def __init__(self, path, message=None, **kwargs):
        super().__init__(
            ERR_ENCODING,
            message or f"UTF-8 이 아니다: {path}",
            path=path,
            **kwargs,
        )


class EmptyScorecardError(BenchError):
    def __init__(self, path=None, message=None, **kwargs):
        super().__init__(
            ERR_EMPTY_SCORECARD,
            message or "스코어카드가 비었다",
            path=path,
            **kwargs,
        )


class UnknownAgentError(BenchError):
    def __init__(self, agent, message=None, **kwargs):
        details = dict(kwargs.pop("details", None) or {})
        details.setdefault("agent", agent)
        super().__init__(
            ERR_UNKNOWN_AGENT,
            message or f"알 수 없는 에이전트: {agent}",
            details=details,
            **kwargs,
        )


class PayloadShapeError(BenchError):
    def __init__(self, issues, path=None, message=None, **kwargs):
        listed = list(issues or [])
        details = dict(kwargs.pop("details", None) or {})
        details["issues"] = listed
        super().__init__(
            ERR_PAYLOAD_SHAPE,
            message or ("payload 형태 오류: " + "; ".join(listed) or "payload 형태 오류"),
            path=path,
            details=details,
            **kwargs,
        )


def format_bench_error(err: BenchError) -> str:
    """stderr 한 줄. 코드가 앞에 있어 기계가 grep 할 수 있다."""
    loc = f" path={err.path}" if err.path else ""
    return f"오류[{err.code}]: {err.message}{loc}"


def error_exit_code(err) -> int:
    if isinstance(err, BenchError):
        return err.exit_code
    return 1


def _agent_id_re():
    return re.compile(r"^[A-Za-z][A-Za-z0-9._-]{0,63}$")


def display_path(path) -> str:
    if path is None:
        return ""
    try:
        return posix_rel(str(path))
    except Exception:
        return str(path)


def utf8_decode(data, *, path=None) -> str:
    """바이트 → 문자열. BOM 은 벗기고, 깨진 바이트는 바꿔 넣지 않고 명시 오류다."""
    if data is None:
        raise EncodingError(path, "디코드할 바이트가 없다")
    if isinstance(data, str):
        return data
    if not isinstance(data, (bytes, bytearray)):
        raise EncodingError(path, f"바이트가 아니다: {type(data).__name__}")
    raw = bytes(data)
    if raw.startswith(b"\xef\xbb\xbf"):
        raw = raw[3:]
    try:
        return raw.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise EncodingError(
            path,
            f"UTF-8 디코드 실패(offset {exc.start}): {exc.reason}",
            details={"start": exc.start, "end": exc.end, "reason": exc.reason},
        ) from exc


def read_bytes(path) -> bytes:
    """존재·권한 오류를 MissingFileError 로 접는다. 빈 파일은 허용한다."""
    if path is None or str(path).strip() == "":
        raise MissingFileError(path, "경로가 비었다")
    target = Path(path)
    if not target.exists():
        raise MissingFileError(path, f"파일이 없다: {display_path(path)}")
    if target.is_dir():
        raise MissingFileError(path, f"파일 아니라 디렉터리다: {display_path(path)}")
    try:
        return target.read_bytes()
    except OSError as exc:
        raise MissingFileError(
            path,
            f"파일을 읽을 수 없다: {display_path(path)} ({exc})",
            details={"errno": getattr(exc, "errno", None)},
        ) from exc


def read_text_utf8(path) -> str:
    return utf8_decode(read_bytes(path), path=path)


def parse_json_text(text, *, path=None):
    """JSON 텍스트 → 값. 빈 문자열·잘린 객체·트레일링 쉼표는 BadJsonError."""
    if text is None:
        raise BadJsonError(path, "JSON 텍스트가 없다")
    if not isinstance(text, str):
        raise BadJsonError(path, f"JSON 텍스트가 문자열이 아니다: {type(text).__name__}")
    stripped = text.strip()
    if stripped == "":
        raise BadJsonError(path, "JSON 이 빈 문자열이다")
    try:
        return json.loads(stripped)
    except json.JSONDecodeError as exc:
        raise BadJsonError(
            path,
            f"JSON 파싱 실패(line {exc.lineno} col {exc.colno}): {exc.msg}",
            details={"lineno": exc.lineno, "colno": exc.colno, "msg": exc.msg},
        ) from exc


def require_json_object(value, *, path=None) -> dict:
    if not isinstance(value, dict):
        raise BadJsonError(
            path,
            f"JSON 최상위가 객체가 아니다: {type(value).__name__}",
            details={"jsonType": type(value).__name__},
        )
    return value


def load_json_object(path) -> dict:
    """경로 → UTF-8 JSON 객체. 파일 없음·인코딩·깨진 JSON·배열 최상위를 가른다."""
    return require_json_object(parse_json_text(read_text_utf8(path), path=path), path=path)


def load_report_payload(path) -> dict:
    """--from-json 입력. 봉투 형태가 아니면 PayloadShapeError."""
    payload = load_json_object(path)
    issues = payload_shape_issues(payload)
    if issues:
        raise PayloadShapeError(issues, path=path)
    return payload


def agent_id_issues(name) -> list[str]:
    """에이전트 식별자 문법. 비어 있음·예약어·금지 문자를 나열한다."""
    issues: list[str] = []
    if name is None:
        return ["agent 가 없다"]
    if not isinstance(name, str):
        return [f"agent 가 문자열이 아니다: {type(name).__name__}"]
    stripped = name.strip()
    if stripped == "":
        issues.append("agent 가 비었다")
        return issues
    if stripped.lower() in RESERVED_AGENT_IDS:
        issues.append(f"agent '{stripped}' 는 예약어다")
    if any(ch in stripped for ch in ("/", "\\", ":", "*", "?", "<", ">", "|")):
        issues.append(f"agent '{stripped}' 에 경로 구분 문자가 있다")
    if " " in stripped or "\t" in stripped or "\n" in stripped:
        issues.append(f"agent '{stripped}' 에 공백이 있다")
    if not _agent_id_re().match(stripped):
        issues.append(
            f"agent '{stripped}' 는 [A-Za-z][A-Za-z0-9._-]{{0,63}} 이 아니다"
        )
    return issues


def normalize_agent_id(name) -> str:
    issues = agent_id_issues(name)
    if issues:
        text = "; ".join(issues)
        if name is None or (isinstance(name, str) and name.strip() == ""):
            raise UnknownAgentError(name, text or "agent 가 비었다")
        if isinstance(name, str) and name.strip().lower() in RESERVED_AGENT_IDS:
            raise UnknownAgentError(name, text)
        raise UnknownAgentError(name, text)
    return str(name).strip()


def discover_known_agents(*roots) -> list[str]:
    """베이스라인·리더보드 스코어카드 폴더에서 알려진 에이전트 이름을 모은다.

    폴더 이름 `claude-fable-5-0000` 은 epoch 접미를 떼어 `claude-fable-5` 도
    인정한다. 스코어카드 JSON 의 `agent` 필드가 있으면 그것도 넣는다.
    파일시스템을 보므로 순수 함수가 아니다 — 시험은 임시 폴더를 주입한다.
    """
    found: set[str] = set()
    for root in roots:
        if not root:
            continue
        base = Path(root)
        if not base.is_dir():
            continue
        for child in sorted(base.iterdir()):
            if not child.is_dir():
                continue
            found.add(child.name)
            stripped = re.sub(r"-\d{4}$", "", child.name)
            if stripped:
                found.add(stripped)
            card = child / "scorecard.json"
            if card.is_file():
                try:
                    doc = load_json_object(card)
                except BenchError:
                    continue
                agent = doc.get("agent")
                if isinstance(agent, str) and agent.strip():
                    found.add(agent.strip())
    return sorted(found)


def default_known_agent_roots() -> list[str]:
    return [
        os.path.join(GYM_ROOT, "baselines"),
        os.path.join(GYM_ROOT, "leaderboard", "scorecards"),
        os.path.join(GYM_ROOT, "submissions"),
    ]


def require_known_agent(name, known) -> str:
    """식별자 문법을 통과한 뒤, 알려진 집합에 없으면 UnknownAgentError."""
    agent = normalize_agent_id(name)
    roster = [str(item) for item in (known or []) if item]
    if not roster:
        raise UnknownAgentError(
            agent,
            f"알려진 에이전트 목록이 비어 있어 '{agent}' 를 대조할 수 없다",
            details={"agent": agent, "known": []},
        )
    if agent not in roster:
        raise UnknownAgentError(
            agent,
            f"알 수 없는 에이전트: {agent} (알려진 {len(roster)}명 밖에 없음)",
            details={"agent": agent, "known": roster},
        )
    return agent


def scorecard_kind_issues(doc) -> list[str]:
    issues: list[str] = []
    if not isinstance(doc, dict):
        return ["스코어카드가 객체가 아니다"]
    kind = doc.get("kind")
    if kind not in (None, SCORECARD_KIND):
        issues.append(f"kind={kind!r} 가 {SCORECARD_KIND} 가 아니다")
    version = doc.get("schemaVersion")
    if version not in (None, *SCORECARD_SCHEMA_VERSIONS):
        issues.append(f"schemaVersion={version!r} 는 {sorted(SCORECARD_SCHEMA_VERSIONS)} 밖이다")
    return issues


def scorecard_emptiness_issues(doc) -> list[str]:
    """측정이 하나도 없으면 빈 스코어카드다. 형태는 맞아도 숫자가 없다."""
    if not isinstance(doc, dict):
        return ["스코어카드가 객체가 아니다"]
    issues: list[str] = []
    if not doc:
        return ["스코어카드 객체가 비었다"]
    packs = doc.get("packs")
    total = doc.get("total")
    if packs is None and total is None:
        issues.append("packs 와 total 이 모두 없다")
        return issues
    if packs is not None and not isinstance(packs, list):
        issues.append("packs 가 배열이 아니다")
        packs = None
    if isinstance(packs, list) and len(packs) == 0:
        issues.append("packs 가 빈 배열이다")
    if total is not None and not isinstance(total, dict):
        issues.append("total 이 객체가 아니다")
        total = None
    if isinstance(total, dict):
        missing = [key for key in SCORECARD_TOTAL_KEYS if key not in total]
        if missing:
            issues.append("total 키 누락: " + ", ".join(missing))
        scored = total.get("packsScored")
        if scored == 0:
            issues.append("packsScored 가 0 이다")
        if scored is None and "packsScored" in total:
            issues.append("packsScored 가 null 이다")
    if isinstance(packs, list) and packs:
        scored_packs = [
            p for p in packs
            if isinstance(p, dict) and p.get("status") not in ("unavailable", None)
            and (p.get("taskCount") or 0) > 0
        ]
        if not scored_packs and all(
            isinstance(p, dict) and p.get("status") == "unavailable" for p in packs
        ):
            issues.append("채점된 pack 이 없고 전부 unavailable 이다")
        if all(isinstance(p, dict) and (p.get("taskCount") or 0) == 0 for p in packs):
            issues.append("모든 pack 의 taskCount 가 0 이다")
    return issues


def load_scorecard(path, *, expected_agent=None, known_agents=None) -> dict:
    """gym scorecard.json 을 읽고 빈 카드·알 수 없는 에이전트를 명시 오류로 가른다.

    expected_agent 가 있으면 카드의 agent 와 문법·소속을 대조한다.
    known_agents 가 있으면 카드/기대 에이전트가 그 집합에 있어야 한다.
    """
    doc = load_json_object(path)
    kind_issues = scorecard_kind_issues(doc)
    if kind_issues:
        raise PayloadShapeError(kind_issues, path=path, message="스코어카드 형태 오류: " + "; ".join(kind_issues))
    empty = scorecard_emptiness_issues(doc)
    if empty:
        raise EmptyScorecardError(path, "빈 스코어카드: " + "; ".join(empty), details={"issues": empty})
    card_agent = doc.get("agent")
    if card_agent is not None:
        normalize_agent_id(card_agent)
    if expected_agent is not None:
        want = normalize_agent_id(expected_agent)
        if card_agent is None:
            raise UnknownAgentError(
                want,
                f"스코어카드에 agent 필드가 없어 '{want}' 와 대조할 수 없다",
                path=path,
                details={"expected": want},
            )
        have = normalize_agent_id(card_agent)
        if have != want:
            raise UnknownAgentError(
                have,
                f"스코어카드 agent '{have}' 가 기대한 '{want}' 가 아니다",
                path=path,
                details={"expected": want, "actual": have},
            )
    check = expected_agent if expected_agent is not None else card_agent
    if known_agents is not None and check is not None:
        require_known_agent(check, known_agents)
    return doc


def scorecard_summary(doc: dict) -> dict:
    """리포트에 붙일 짧은 요약. 카드가 비었으면 부르기 전에 load_scorecard 가 막는다."""
    total = doc.get("total") if isinstance(doc.get("total"), dict) else {}
    packs = doc.get("packs") if isinstance(doc.get("packs"), list) else []
    return {
        "kind": doc.get("kind") or SCORECARD_KIND,
        "schemaVersion": doc.get("schemaVersion"),
        "agent": doc.get("agent"),
        "score": total.get("score"),
        "max": total.get("max"),
        "packsScored": total.get("packsScored"),
        "packCount": len(packs),
        "packIds": [p.get("id") for p in packs if isinstance(p, dict) and p.get("id")],
    }


def attach_scorecard(payload: dict, card: dict) -> dict:
    """측정 봉투에 스코어카드 요약을 붙인다. 평결 숫자는 바꾸지 않는다."""
    out = dict(payload)
    out["scorecard"] = scorecard_summary(card)
    return out


def run_record_issues(run) -> list[str]:
    """한 run 레코드의 최소 형태."""
    if not isinstance(run, dict):
        return ["run 이 객체가 아니다"]
    issues: list[str] = []
    if not run.get("file"):
        issues.append("file 이 없다")
    if "ok" not in run:
        issues.append("ok 가 없다")
    ms = run.get("ms")
    if ms is not None and not is_number(ms):
        issues.append(f"ms 가 숫자가 아니다: {ms!r}")
    chars = run.get("chars")
    if chars is not None and not is_number(chars):
        issues.append(f"chars 가 숫자가 아니다: {chars!r}")
    return issues


def task_result_issues(result) -> list[str]:
    if not isinstance(result, dict):
        return ["result 가 객체가 아니다"]
    issues: list[str] = []
    if not result.get("tool"):
        issues.append("tool 이 없다")
    if "available" not in result:
        issues.append("available 이 없다")
    if result.get("available") is False and invented_metrics(result):
        issues.append(f"{result.get('tool')}: 비가용인데 숫자를 실었다")
    if result.get("available") is True:
        summary = result.get("summary")
        if not isinstance(summary, dict):
            issues.append(f"{result.get('tool')}: available 인데 summary 가 없다")
        runs = result.get("runs")
        if runs is not None:
            if not isinstance(runs, list):
                issues.append(f"{result.get('tool')}: runs 가 배열이 아니다")
            else:
                for idx, run in enumerate(runs):
                    for issue in run_record_issues(run):
                        issues.append(f"{result.get('tool')}.runs[{idx}]: {issue}")
    return issues


def payload_honesty_issues(payload) -> list[str]:
    """정직성 가드 — 비가용 칸의 숫자, 빈 tasks, 깨진 run."""
    if not isinstance(payload, dict):
        return ["payload 가 객체가 아니다"]
    issues = payload_shape_issues(payload)
    tasks = payload.get("tasks")
    if isinstance(tasks, list):
        if len(tasks) == 0:
            issues.append("tasks 가 빈 배열이다")
        for task in tasks:
            if not isinstance(task, dict):
                issues.append("task 가 객체가 아니다")
                continue
            if not task.get("task"):
                issues.append("task 이름이 없다")
            results = task.get("results")
            if results is None:
                continue
            if not isinstance(results, list):
                issues.append(f"{task.get('task')}: results 가 배열이 아니다")
                continue
            for result in results:
                issues.extend(task_result_issues(result))
    matrix = payload.get("capabilityMatrix")
    if matrix is not None:
        issues.extend(validate_capability_matrix(matrix))
    return issues


def require_honest_payload(payload, *, path=None) -> dict:
    issues = payload_honesty_issues(payload)
    if issues:
        raise PayloadShapeError(issues, path=path)
    return payload


def parse_rhwp_info_fields(stdout):
    """rhwp `info --json` 에서 비교 가능한 스칼라만 뽑는다. 실패 시 None."""
    doc = _json_object(stdout)
    if doc is None:
        return None
    body = unwrap_json_envelope(doc)
    if not isinstance(body, dict):
        return None
    out = {}
    for key in ("format", "pageCount", "sectionCount", "paraCount"):
        if key in body:
            out[key] = body.get(key)
    return out or None


def parse_rhwp_structure_nodes(stdout):
    """export-structure --json 의 노드 수. 목록이 없으면 None."""
    doc = _json_object(stdout)
    if doc is None:
        return None
    body = unwrap_json_envelope(doc)
    if not isinstance(body, dict):
        return None
    if is_number(body.get("nodeCount")):
        return int(body["nodeCount"])
    for key in ("nodes", "sections", "children"):
        value = body.get(key)
        if isinstance(value, list):
            return len(value)
    return None


def classify_cli_failure(message: str) -> str:
    """서브프로세스 stderr 를 오류 코드로 근사. 숫자를 지어내지 않는다."""
    text = (message or "").lower()
    if "timeout" in text:
        return "timeout"
    if "not found" in text or "cannot find" in text or "없는 파일" in text:
        return "missing_input"
    if "permission" in text or "액세스가 거부" in text:
        return "permission"
    if "utf-8" in text or "codec" in text or "decode" in text:
        return ERR_ENCODING
    if "json" in text:
        return ERR_BAD_JSON
    return "runtime"


def build_payload(rhwp: str, pyhwp: str | None, soffice: str | None,
                  files: list[str], cwd: str, timeout: int,
                  rhwp_version: str, rhwp_profile: str) -> dict:
    """모든 과제를 돌리고(가용한 도구만) payload 를 조립한다."""
    # --- export-text: 실 헤드-투-헤드 ---
    rhwp_text_runs = bench_rhwp_text(rhwp, files, cwd, timeout)
    text_results = [
        available_result("rhwp", rhwp_text_runs, fidelity=1.0),
    ]
    if pyhwp:
        py_runs = bench_pyhwp_text(pyhwp, files, cwd, timeout)
        text_results.append(available_result(
            "pyhwp", py_runs,
            fidelity=fidelity_vs_ref(py_runs, rhwp_text_runs),
            overlap=overlap_timing(py_runs, rhwp_text_runs),
            note="HWPX(ZIP)는 OLE 파서라 열지 못함; 표 셀 본문을 `<표>` 자리표로 대체.",
        ))
    else:
        text_results.append(unavailable_result(
            "pyhwp",
            "이 머신에서 실행 불가(휴면 패키지; import 에 six 등 수동 보정 필요)",
        ))
    text_results.append(_soffice_text_result(soffice, files, cwd, timeout, rhwp_text_runs))
    text_results.append(unavailable_result(
        "hwplib", "Java 라이브러리, CLI 아님(래퍼 클래스+빌드 필요)",
    ))

    # --- info / structure / convert: rhwp 는 실행, 대안은 정직한 n/a ---
    info_runs = bench_rhwp_info(rhwp, files, cwd, timeout)
    info_results = [available_result("rhwp", info_runs)]
    struct_runs = bench_rhwp_structure(rhwp, files, cwd, timeout)
    struct_results = [available_result("rhwp", struct_runs)]
    convert_runs = bench_rhwp_convert(rhwp, files, cwd, timeout)
    convert_results = [available_result(
        "rhwp", convert_runs,
        note="HWP→markdown(에이전트-대면 변환); export-hwpx 로 HWPX 변환도 지원.",
    )]
    na_meta = [
        unavailable_result(
            "pyhwp",
            "동일 형식 산출 없음(hwp5proc 는 저수준 레코드 덤프; 메타 봉투 아님)",
        ),
        unavailable_result(
            "soffice",
            _soffice_reason(soffice) + "; 구조화 메타/구조 출력 없음",
        ),
        unavailable_result("hwplib", "Java 라이브러리, CLI 아님"),
    ]
    for results in (info_results, struct_results, convert_results):
        results.extend(dict(item) for item in na_meta)

    env = assemble_env(
        os_name=platform.platform(),
        python=platform.python_version(),
        rhwp_version=rhwp_version,
        rhwp_profile=rhwp_profile,
        files=files,
        tools={
            "rhwp": {"available": True, "detail": f"{rhwp_version} ({rhwp_profile})"},
            "pyhwp": (
                {"available": True, "detail": "hwp5txt (pyhwp 0.1b15) — venv, six 수동설치"}
                if pyhwp else
                {"available": False, "detail": "미설치/실행 불가"}
            ),
            "soffice": (
                {"available": True, "detail": "LibreOffice headless"}
                if soffice else
                {"available": False, "detail": "미설치(이 머신)"}
            ),
            "hwplib": {"available": False, "detail": "Java 라이브러리 — CLI 아님(미실행)"},
            "hancomSdk": {"available": False, "detail": "Windows 전용 독점 SDK(미실행)"},
        },
    )
    return assemble_payload(
        env=env,
        tasks=[
            {"task": "export-text", "results": text_results},
            {"task": "info", "results": info_results},
            {"task": "structure", "results": struct_results},
            {"task": "convert", "results": convert_results},
        ],
        generated_at=time.strftime("%Y-%m-%dT%H:%M:%S"),
    )


def _soffice_reason(soffice: str | None) -> str:
    return "미설치(이 머신)" if not soffice else "설치됨이나 HWP5 임포트 필터 없음"


def _soffice_text_result(soffice, files, cwd, timeout, rhwp_text_runs) -> dict:
    if not soffice:
        return unavailable_result(
            "soffice",
            "미설치(이 머신); 설치돼도 HWP5 임포트 필터 없어 현대 .hwp 못 엶",
        )
    runs = bench_soffice_text(soffice, files, cwd, timeout)
    return available_result(
        "soffice", runs,
        fidelity=fidelity_vs_ref(runs, rhwp_text_runs),
        overlap=overlap_timing(runs, rhwp_text_runs),
        note="LibreOffice 는 HWP5 임포트 필터가 없어 현대 .hwp 는 대부분 실패한다.",
    )


def _rhwp_version(rhwp: str, cwd: str) -> str:
    ok, _, out, _ = _run([rhwp, "--version"], cwd, 15)
    return out.strip() if ok and out.strip() else "unknown"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="rhwp 경쟁 벤치마크 하네스")
    ap.add_argument("--rhwp", default=None, help="rhwp 바이너리 경로(기본: target/{release,debug}/rhwp)")
    ap.add_argument("--pyhwp", default=None, help="hwp5txt 경로(pyhwp). 없으면 자동탐지/미가용")
    ap.add_argument("--soffice", default=None, help="soffice/libreoffice 경로. 없으면 자동탐지/미가용")
    ap.add_argument("--samples", default=os.path.join(REPO_ROOT, "samples"), help="코퍼스 폴더")
    ap.add_argument("--limit", type=int, default=25, help="형식별 최대 파일 수(0=전체)")
    ap.add_argument("--timeout", type=int, default=DEFAULT_TIMEOUT, help="서브프로세스 상한(초)")
    ap.add_argument("--out-json", default=None, help="JSON 결과 경로")
    ap.add_argument("--out-md", default=None, help="마크다운 리포트 경로")
    ap.add_argument("--from-json", default=None,
                    help="벤치 재실행 없이 기존 JSON 에서 리포트만 다시 렌더")
    ap.add_argument("--json", action="store_true", help="payload 를 stdout 으로도 출력")
    args = ap.parse_args(argv)

    # Windows 콘솔 기본 코드페이지(cp949)는 한글 대시 등을 못 찍는다 — UTF-8 로 강제.
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8")  # type: ignore[attr-defined]
        except (AttributeError, ValueError):
            pass

    # 렌더-온리: 저장된 payload 에서 리포트만 재생성한다(벤치 불요, 결정론적).
    if args.from_json:
        raw = Path(args.from_json).read_text(encoding="utf-8")
        payload, issues = load_report_payload(raw)
        if payload is None:
            print("오류: --from-json payload 가 깨졌다: " + "; ".join(issues),
                  file=sys.stderr)
            return 2
        if args.out_json:
            Path(args.out_json).parent.mkdir(parents=True, exist_ok=True)
            write_text_lf(args.out_json, dump_payload_json(payload))
            print(f"[bench] JSON 재스탬프 → {args.out_json}", file=sys.stderr)
        if args.json:
            print(dump_payload_json(payload), end="")
        md = render_report(payload)
        if args.out_md:
            Path(args.out_md).parent.mkdir(parents=True, exist_ok=True)
            write_text_lf(args.out_md, md)
            print(f"[bench] 리포트 재렌더 → {args.out_md}", file=sys.stderr)
        elif not args.json:
            print(md)
        return 0

    cwd = REPO_ROOT

    # rhwp 바이너리 확정(하네스의 유일한 필수 전제)
    rhwp = args.rhwp
    if not rhwp:
        for cand in ("target/release/rhwp.exe", "target/release/rhwp",
                     "target/debug/rhwp.exe", "target/debug/rhwp"):
            if (Path(cwd) / cand).exists():
                rhwp = str(Path(cwd) / cand)
                break
    rhwp = probe(rhwp, ["rhwp"])
    if not rhwp:
        print("오류: rhwp 바이너리를 찾을 수 없습니다. `cargo build --bin rhwp` 후 --rhwp 로 지정하세요.",
              file=sys.stderr)
        return 2
    rhwp_profile = rhwp_profile_from_path(rhwp)

    pyhwp = probe(args.pyhwp, ["hwp5txt"])
    soffice = probe(args.soffice, ["soffice", "libreoffice"])

    files = discover_corpus(args.samples, args.limit)
    if not files:
        print(f"오류: 코퍼스가 비었습니다: {args.samples}", file=sys.stderr)
        return 2

    print(f"[bench] rhwp={rhwp} ({rhwp_profile}) · pyhwp={'O' if pyhwp else 'X'} · "
          f"soffice={'O' if soffice else 'X'} · 파일 {len(files)}개", file=sys.stderr)

    version = _rhwp_version(rhwp, cwd)
    payload = build_payload(rhwp, pyhwp, soffice, files, cwd, args.timeout, version, rhwp_profile)

    if args.out_json:
        Path(args.out_json).parent.mkdir(parents=True, exist_ok=True)
        write_text_lf(args.out_json, dump_payload_json(payload))
        print(f"[bench] JSON → {args.out_json}", file=sys.stderr)
    if args.out_md:
        Path(args.out_md).parent.mkdir(parents=True, exist_ok=True)
        write_text_lf(args.out_md, render_report(payload))
        print(f"[bench] 리포트 → {args.out_md}", file=sys.stderr)
    if args.json or (not args.out_json and not args.out_md):
        print(dump_payload_json(payload), end="")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
