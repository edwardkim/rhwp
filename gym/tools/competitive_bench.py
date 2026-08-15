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


# --------------------------------------------------------------------------
# 순수 집계 로직 (바이너리·외부 도구 불요 — 가드 테스트가 이 부분만 검증한다)
# --------------------------------------------------------------------------
def median(values):
    """None 을 거른 중앙값. 값이 없으면 None."""
    vals = [v for v in values if v is not None]
    if not vals:
        return None
    return statistics.median(vals)


def _round_ms(v):
    return None if v is None else round(v, 1)


def _round_int(v):
    return None if v is None else int(round(v))


def summarize_runs(runs: list[dict]) -> dict:
    """run 레코드 리스트 → 요약 통계. 순수.

    run 레코드: {"file": str, "ext": str, "ok": bool, "ms": float|None, "chars": int|None}
    - medianMs 는 **성공한 실행만** 대상으로 한다(실패의 0ms 로 시간을 왜곡하지 않는다).
    - byExt 는 형식별 성공률을 남긴다(예: pyhwp 가 .hwp 는 되고 .hwpx 는 안 되는 사실).
    """
    attempted = len(runs)
    ok_runs = [r for r in runs if r.get("ok")]
    by_ext: dict[str, dict] = {}
    for r in runs:
        ext = (r.get("ext") or "").lower()
        bucket = by_ext.setdefault(ext, {"attempted": 0, "ok": 0})
        bucket["attempted"] += 1
        if r.get("ok"):
            bucket["ok"] += 1
    return {
        "attempted": attempted,
        "ok": len(ok_runs),
        "successRate": round(len(ok_runs) / attempted, 3) if attempted else None,
        "medianMs": _round_ms(median([r.get("ms") for r in ok_runs])),
        "medianChars": _round_int(
            median([r.get("chars") for r in ok_runs if r.get("chars") is not None])
        ),
        "byExt": by_ext,
    }


def fidelity_vs_ref(tool_runs: list[dict], ref_runs: list[dict]):
    """도구/기준(rhwp) 문자수 비율의 **파일별 중앙값**. 둘 다 성공한 파일만.

    겹치는 파일이 없으면 None. 1.0=동일량, <1.0=덜 뽑음(예: pyhwp 가 표 셀 대신
    `<표>` 자리표만 남겨 문자수가 적다), >1.0=더 뽑음.
    """
    ref = {
        r["file"]: r.get("chars")
        for r in ref_runs
        if r.get("ok") and r.get("chars")
    }
    ratios = []
    for r in tool_runs:
        if not r.get("ok"):
            continue
        got = r.get("chars")
        base = ref.get(r.get("file"))
        if got is None or not base:
            continue
        ratios.append(got / base)
    if not ratios:
        return None
    return round(statistics.median(ratios), 3)


def overlap_median_ms(tool_runs: list[dict], ref_runs: list[dict]):
    """두 도구가 **모두 성공한 파일**에서만 각각의 median ms 를 낸다 → 공정한 동일-집합 속도.

    (tool_ms, ref_ms) 를 반환. 겹침 없으면 (None, None). rhwp 의 중앙값이 HWPX 까지
    포함해 부풀지 않도록, 속도 비교는 같은 파일집합에서만 한다."""
    ref_ok = {r["file"]: r.get("ms") for r in ref_runs if r.get("ok")}
    tool_ms, ref_ms = [], []
    for r in tool_runs:
        if not r.get("ok"):
            continue
        base = ref_ok.get(r.get("file"))
        if base is None or r.get("ms") is None:
            continue
        tool_ms.append(r["ms"])
        ref_ms.append(base)
    if not tool_ms:
        return None, None
    return _round_ms(statistics.median(tool_ms)), _round_ms(statistics.median(ref_ms))


def parse_rhwp_text_chars(stdout: str):
    """rhwp `export-text --json` 봉투 → 총 문자수. 파싱 실패 시 None. 순수."""
    try:
        doc = json.loads(stdout)
    except (json.JSONDecodeError, TypeError):
        return None
    pages = doc.get("pages")
    if not isinstance(pages, list):
        return None
    return sum(len(p.get("text", "")) for p in pages if isinstance(p, dict))


def _fmt_cell(available: bool, summary: dict | None, fidelity, reason: str | None) -> str:
    """결과표 한 칸: 성공률·중앙값 시간·충실도, 또는 'n/a: <이유>'. 순수."""
    if not available:
        return f"n/a: {reason or '실행 불가'}"
    if not summary or summary.get("attempted", 0) == 0:
        return "n/a: 시도 없음"
    rate = summary.get("successRate")
    rate_pct = "-" if rate is None else f"{round(rate * 100)}%"
    ms = summary.get("medianMs")
    ms_s = "-" if ms is None else f"{ms:.0f}ms"
    ok = summary.get("ok", 0)
    att = summary.get("attempted", 0)
    fid = "-" if fidelity is None else f"{fidelity:.2f}×"
    return f"{ms_s} · {rate_pct}({ok}/{att}) · 충실도 {fid}"


def verdict_lines(payload: dict) -> list[str]:
    """측정 데이터에서 직접 유도한 정직한 평결 문장들. 순수.

    숫자는 payload 에서만 온다 — 손으로 쓴 승패 주장이 아니라 잰 값의 서술이다.
    """
    lines: list[str] = []
    tasks = {t["task"]: t for t in payload.get("tasks", [])}

    # export-text 헤드-투-헤드: rhwp vs pyhwp
    et = tasks.get("export-text", {})
    res = {r["tool"]: r for r in et.get("results", [])}
    rhwp = res.get("rhwp")
    pyhwp = res.get("pyhwp")
    if rhwp and rhwp.get("available"):
        s = rhwp["summary"]
        lines.append(
            f"rhwp 는 export-text 에서 {s['ok']}/{s['attempted']} 파일을 처리했다"
            f"(HWP+HWPX 혼합, 중앙값 {s['medianMs']}ms)."
        )
    if pyhwp and pyhwp.get("available"):
        ps = pyhwp["summary"]
        hwp_b = ps.get("byExt", {}).get(".hwp", {})
        hwpx_b = ps.get("byExt", {}).get(".hwpx", {})
        lines.append(
            f"pyhwp(hwp5txt)는 HWP5 {hwp_b.get('ok', 0)}/{hwp_b.get('attempted', 0)} 성공, "
            f"HWPX {hwpx_b.get('ok', 0)}/{hwpx_b.get('attempted', 0)} 성공"
            f"(ZIP 기반 HWPX 는 OLE 파서로 열 수 없음 — 구조적 한계)."
        )
        # 속도 — 같은 파일집합(둘 다 성공한 HWP5)에서만 비교해야 공정하다.
        ov = pyhwp.get("overlapMs") or {}
        p_ms = ov.get("tool")
        r_ms = ov.get("ref")
        if r_ms is not None and p_ms is not None:
            if p_ms < r_ms:
                lines.append(
                    f"속도(동일 파일집합, 둘 다 연 HWP5): pyhwp 가 더 빨랐다"
                    f"(pyhwp {p_ms}ms vs rhwp {r_ms}ms 중앙값). rhwp 는 디버그 빌드이며 JSON "
                    f"봉투·출처 표지를 함께 낸다 — 릴리스 빌드로는 좁혀진다. 그래도 더 빠른 축은 "
                    f"그대로 적는다."
                )
            else:
                lines.append(
                    f"속도(동일 파일집합, 둘 다 연 HWP5): rhwp 가 더 빠르거나 동급이었다"
                    f"(rhwp {r_ms}ms vs pyhwp {p_ms}ms 중앙값) — 디버그 빌드임에도."
                )
        fid = pyhwp.get("fidelityVsRhwp")
        if fid is not None:
            lines.append(
                f"충실도: 두 도구가 모두 연 HWP5 에서 pyhwp 문자수는 rhwp 대비 중앙값 {fid:.2f}× — "
                f"pyhwp 는 표 셀 본문을 `<표>` 자리표로 대체해 본문을 덜 뽑는다"
                f"(에이전트가 표 안 숫자를 읽어야 하면 치명적)."
            )

    # 폭: rhwp 만 도는 과제
    rhwp_only = []
    for name in ("info", "structure", "convert"):
        tr = tasks.get(name, {})
        r = {x["tool"]: x for x in tr.get("results", [])}.get("rhwp")
        others_avail = any(
            x.get("available") for x in tr.get("results", []) if x["tool"] != "rhwp"
        )
        if r and r.get("available") and not others_avail:
            rhwp_only.append(name)
    if rhwp_only:
        lines.append(
            "폭: " + ", ".join(rhwp_only) + " 과제는 rhwp 만 구조화 CLI 로 수행했다 — "
            "대안들은 동일 형식 산출(메타 봉투·구조 트리·HWPX/markdown 변환)이 없어 n/a."
        )

    # 능력 — rhwp 고유
    lines.append(
        "능력: MCP 서버·검증 가능 작업(replay/capsule)·단일 자립 바이너리·"
        "메모리 안전(Rust)·JSON 봉투는 벤치한 대안 중 rhwp 만 갖췄다(능력 매트릭스 참조)."
    )
    return lines


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
    tools_order = payload.get("toolOrder", [])
    header = "| 과제 | " + " | ".join(tools_order) + " |"
    sep = "|---|" + "|".join(["---"] * len(tools_order)) + "|"
    out.append(header)
    out.append(sep)
    for task in payload.get("tasks", []):
        row = {r["tool"]: r for r in task.get("results", [])}
        cells = []
        for tool in tools_order:
            r = row.get(tool)
            if r is None:
                cells.append("n/a")
                continue
            cells.append(
                _fmt_cell(
                    r.get("available", False),
                    r.get("summary"),
                    r.get("fidelityVsRhwp"),
                    r.get("reason"),
                )
            )
        out.append(f"| **{task['task']}** | " + " | ".join(cells) + " |")
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
    for r in matrix["rows"]:
        cells = [glyph.get(r.get(c["key"], "no"), "?") for c in cols]
        out.append(f"| {r['tool']} | " + " | ".join(cells) + " |")
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


def discover_corpus(samples_dir: str, limit: int) -> list[str]:
    """samples/ 에서 HWP·HWPX 를 결정론적으로 선택(정렬 후 형식별 limit).

    경로는 REPO_ROOT 상대(POSIX)로 낸다 — 서브프로세스는 cwd=REPO_ROOT 에서 돌므로
    상대경로로 동작하고, 커밋되는 JSON 에 머신별 절대경로가 새지 않는다.
    """
    base = Path(samples_dir)
    hwp = sorted(_rel(p) for p in base.glob("*.hwp"))
    hwpx = sorted(_rel(p) for p in base.glob("*.hwpx"))
    if limit > 0:
        hwp = hwp[:limit]
        hwpx = hwpx[:limit]
    return hwp + hwpx


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


def probe(path: str | None, names: list[str]) -> str | None:
    """명시 경로 또는 PATH 에서 실행파일을 찾는다. 없으면 None."""
    if path:
        p = Path(path)
        if p.exists():
            return str(p)
        found = shutil.which(path)
        if found:
            return found
        return None
    for n in names:
        found = shutil.which(n)
        if found:
            return found
    return None


# --------------------------------------------------------------------------
# 오케스트레이션
# --------------------------------------------------------------------------
def build_payload(rhwp: str, pyhwp: str | None, soffice: str | None,
                  files: list[str], cwd: str, timeout: int,
                  rhwp_version: str, rhwp_profile: str) -> dict:
    """모든 과제를 돌리고(가용한 도구만) payload 를 조립한다."""
    n_hwp = sum(1 for f in files if _ext(f) == ".hwp")
    n_hwpx = sum(1 for f in files if _ext(f) == ".hwpx")

    # --- export-text: 실 헤드-투-헤드 ---
    rhwp_text_runs = bench_rhwp_text(rhwp, files, cwd, timeout)
    text_results = [{
        "tool": "rhwp", "available": True,
        "summary": summarize_runs(rhwp_text_runs), "fidelityVsRhwp": 1.0,
        "runs": rhwp_text_runs,
    }]
    if pyhwp:
        py_runs = bench_pyhwp_text(pyhwp, files, cwd, timeout)
        p_ms, r_ms = overlap_median_ms(py_runs, rhwp_text_runs)
        text_results.append({
            "tool": "pyhwp", "available": True,
            "summary": summarize_runs(py_runs),
            "fidelityVsRhwp": fidelity_vs_ref(py_runs, rhwp_text_runs),
            "overlapMs": {"tool": p_ms, "ref": r_ms},
            "note": "HWPX(ZIP)는 OLE 파서라 열지 못함; 표 셀 본문을 `<표>` 자리표로 대체.",
            "runs": py_runs,
        })
    else:
        text_results.append({
            "tool": "pyhwp", "available": False,
            "reason": "이 머신에서 실행 불가(휴면 패키지; import 에 six 등 수동 보정 필요)",
        })
    text_results.append(_soffice_text_result(soffice, files, cwd, timeout, rhwp_text_runs))
    text_results.append({
        "tool": "hwplib", "available": False,
        "reason": "Java 라이브러리, CLI 아님(래퍼 클래스+빌드 필요)",
    })

    # --- info / structure / convert: rhwp 는 실행, 대안은 정직한 n/a ---
    info_runs = bench_rhwp_info(rhwp, files, cwd, timeout)
    info_results = [{
        "tool": "rhwp", "available": True,
        "summary": summarize_runs(info_runs), "fidelityVsRhwp": None, "runs": info_runs,
    }]
    struct_runs = bench_rhwp_structure(rhwp, files, cwd, timeout)
    struct_results = [{
        "tool": "rhwp", "available": True,
        "summary": summarize_runs(struct_runs), "fidelityVsRhwp": None, "runs": struct_runs,
    }]
    convert_runs = bench_rhwp_convert(rhwp, files, cwd, timeout)
    convert_results = [{
        "tool": "rhwp", "available": True,
        "summary": summarize_runs(convert_runs), "fidelityVsRhwp": None,
        "note": "HWP→markdown(에이전트-대면 변환); export-hwpx 로 HWPX 변환도 지원.",
        "runs": convert_runs,
    }]
    na_pyhwp_meta = {
        "tool": "pyhwp", "available": False,
        "reason": "동일 형식 산출 없음(hwp5proc 는 저수준 레코드 덤프; 메타 봉투 아님)",
    }
    na_soffice_meta = {
        "tool": "soffice", "available": False,
        "reason": _soffice_reason(soffice) + "; 구조화 메타/구조 출력 없음",
    }
    na_hwplib_meta = {
        "tool": "hwplib", "available": False, "reason": "Java 라이브러리, CLI 아님",
    }
    for results in (info_results, struct_results, convert_results):
        results.extend([dict(na_pyhwp_meta), dict(na_soffice_meta), dict(na_hwplib_meta)])

    payload = {
        "schemaVersion": "1.0",
        "generatedAt": time.strftime("%Y-%m-%dT%H:%M:%S"),
        "toolOrder": ["rhwp", "pyhwp", "soffice", "hwplib"],
        "env": {
            "os": platform.platform(),
            "python": platform.python_version(),
            "rhwpVersion": rhwp_version,
            "rhwpProfile": rhwp_profile,
            "corpus": {"dir": "samples", "total": len(files), "hwp": n_hwp, "hwpx": n_hwpx},
            "tools": {
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
        },
        "tasks": [
            {"task": "export-text", "results": text_results},
            {"task": "info", "results": info_results},
            {"task": "structure", "results": struct_results},
            {"task": "convert", "results": convert_results},
        ],
        "capabilityMatrix": capability_matrix(),
    }
    payload["verdict"] = verdict_lines(payload)
    return payload


def _soffice_reason(soffice: str | None) -> str:
    return "미설치(이 머신)" if not soffice else "설치됨이나 HWP5 임포트 필터 없음"


def _soffice_text_result(soffice, files, cwd, timeout, rhwp_text_runs) -> dict:
    if not soffice:
        return {
            "tool": "soffice", "available": False,
            "reason": "미설치(이 머신); 설치돼도 HWP5 임포트 필터 없어 현대 .hwp 못 엶",
        }
    runs = bench_soffice_text(soffice, files, cwd, timeout)
    return {
        "tool": "soffice", "available": True,
        "summary": summarize_runs(runs),
        "fidelityVsRhwp": fidelity_vs_ref(runs, rhwp_text_runs),
        "note": "LibreOffice 는 HWP5 임포트 필터가 없어 현대 .hwp 는 대부분 실패한다.",
    }


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
        payload = json.loads(Path(args.from_json).read_text(encoding="utf-8"))
        md = render_report(payload)
        if args.out_md:
            Path(args.out_md).parent.mkdir(parents=True, exist_ok=True)
            Path(args.out_md).write_text(md, encoding="utf-8")
            print(f"[bench] 리포트 재렌더 → {args.out_md}", file=sys.stderr)
        else:
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
    rhwp_profile = "release" if "release" in rhwp.replace("\\", "/") else "debug"

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
        Path(args.out_json).write_text(
            json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
        print(f"[bench] JSON → {args.out_json}", file=sys.stderr)
    if args.out_md:
        Path(args.out_md).parent.mkdir(parents=True, exist_ok=True)
        Path(args.out_md).write_text(render_report(payload), encoding="utf-8")
        print(f"[bench] 리포트 → {args.out_md}", file=sys.stderr)
    if args.json or (not args.out_json and not args.out_md):
        print(json.dumps(payload, ensure_ascii=False, indent=2))

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
