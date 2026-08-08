#!/usr/bin/env python3
"""에이전트 태스크 벤치마크 하니스 — 풀이 디렉터리를 받아 태스크별 기계 채점.

1차 범위(#4220 T2): 살아있는 LLM 호출 없음. 태스크 정의(tasks.json)의
(fixture, 목표, 오라클 명령열)에 대해 "풀이 스크립트"를 실행하고, 오라클이
산출물을 devel 에 실존하는 rhwp CLI 명령으로 재독해 pass/fail 을 판정한다.

사용:
    RHWP_BIN=target/release/rhwp python tools/agent_bench/run_bench.py \
        --solutions tools/agent_bench/reference_solutions [--tasks t01,t02] \
        [--json-out results.json] [--keep-work]

종료 코드 규약:
    0 — 전 태스크 pass
    1 — 하나 이상 fail (풀이 없음·풀이 오류·오라클 판정 실패 포함)
    2 — 사용법/구성 오류 (RHWP_BIN 없음, tasks.json 손상 등)

풀이 인터페이스(각 태스크 공통):
    <solutions>/<task_id>.py 를 `python <파일>` 로 실행한다. 환경 변수 —
      RHWP_BIN           rhwp 바이너리 절대 경로
      BENCH_TASK_ID      태스크 id
      BENCH_INPUT        1차 입력 문서 절대 경로
      BENCH_INPUTS_JSON  입력 문서 절대 경로 배열(JSON)
      BENCH_PARAMS_JSON  태스크 매개변수(JSON) — tasks.json 의 params 그대로
      BENCH_OUT_DIR      산출물을 써야 하는 디렉터리 절대 경로
    풀이는 tasks.json 의 outputs 에 선언된 파일명을 BENCH_OUT_DIR 에 만들어야
    하며, 문서 조작은 rhwp CLI 를 통해서만 한다(오라클은 산출물만 본다).
"""

from __future__ import annotations

import argparse
import csv
import difflib
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TASKS_JSON = Path(__file__).resolve().parent / "tasks.json"
ORACLE_STEP_TIMEOUT_DEFAULT = 180


def _reconfigure_stdout() -> None:
    """Windows 콘솔(cp949)에서 한글 출력이 깨지지 않게 UTF-8 로 고정한다."""
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except (AttributeError, io.UnsupportedOperation):
            pass


# ── 값 참조/경로 해석 ────────────────────────────────────────────────


def json_path(value, path: str):
    """`a.b[0].c` 형태의 미니 경로. 실패하면 KeyError."""
    if path in ("", "."):
        return value
    token_re = re.compile(r"([^.\[\]]+)|\[(\d+)\]")
    pos = 0
    cur = value
    for m in token_re.finditer(path):
        if m.start() != pos:
            raise KeyError(f"경로 구문 오류: {path!r} (offset {pos})")
        pos = m.end()
        if pos < len(path) and path[pos] == ".":
            pos += 1
        key, idx = m.group(1), m.group(2)
        if key is not None:
            if not isinstance(cur, dict) or key not in cur:
                raise KeyError(f"경로 {path!r} 의 {key!r} 가 없습니다")
            cur = cur[key]
        else:
            i = int(idx)
            if not isinstance(cur, list) or i >= len(cur):
                raise KeyError(f"경로 {path!r} 의 [{idx}] 가 없습니다")
            cur = cur[i]
    if pos != len(path):
        raise KeyError(f"경로 구문 오류: {path!r} (tail {path[pos:]!r})")
    return cur


OPS = {
    "eq": lambda a, b: a == b,
    "ne": lambda a, b: a != b,
    "ge": lambda a, b: a >= b,
    "le": lambda a, b: a <= b,
    "gt": lambda a, b: a > b,
    "lt": lambda a, b: a < b,
    "contains": lambda a, b: b in a,
    "notContains": lambda a, b: b not in a,
    "isEmpty": lambda a, _b: len(a) == 0,
    "isNotEmpty": lambda a, _b: len(a) > 0,
    "lenEq": lambda a, b: len(a) == b,
}


class CheckFailure(Exception):
    pass


class TaskContext:
    """한 태스크 실행의 자리표·저장값 컨텍스트."""

    def __init__(self, rhwp: Path, fixtures: list[Path], out_dir: Path, work_dir: Path):
        self.rhwp = rhwp
        self.fixtures = fixtures
        self.out_dir = out_dir
        self.work_dir = work_dir
        self.saves: dict[str, object] = {}
        self.save_text: dict[str, str] = {}

    def expand(self, text: str) -> str:
        mapping = {
            "RHWP": str(self.rhwp),
            "REPO": str(REPO_ROOT),
            "OUT": str(self.out_dir),
            "WORK": str(self.work_dir),
            "FIXTURE": str(self.fixtures[0]) if self.fixtures else "",
        }
        for i, f in enumerate(self.fixtures):
            mapping[f"FIXTURE{i}"] = str(f)
        for key, val in mapping.items():
            text = text.replace("{" + key + "}", val)
        return text

    def resolve_value(self, spec):
        """검사 항목의 값 자리 — 리터럴 또는 {from,path} / {file} 참조."""
        if isinstance(spec, dict) and ("from" in spec or "file" in spec):
            if "from" in spec:
                save = spec["from"]
                if save not in self.saves:
                    raise CheckFailure(f"저장값 {save!r} 이 없습니다")
                return json_path(self.saves[save], spec.get("path", ""))
            path = Path(self.expand(spec["file"]))
            if not path.is_file():
                raise CheckFailure(f"파일이 없습니다: {path}")
            return path.read_text(encoding="utf-8", errors="replace")
        return spec


# ── 오라클 실행 ──────────────────────────────────────────────────────


def run_step(ctx: TaskContext, step: dict) -> None:
    # 명령 없는 스텝: 풀이 산출 JSON 파일을 읽어 저장값으로 올린다.
    if "loadJson" in step:
        path = Path(ctx.expand(step["loadJson"]))
        if not path.is_file():
            raise CheckFailure(f"산출물이 없습니다: {path}")
        try:
            ctx.saves[step["save"]] = json.loads(path.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, UnicodeDecodeError) as exc:
            raise CheckFailure(f"산출물이 JSON 이 아닙니다({exc}): {path}") from exc
        return

    cmd = [ctx.expand(str(part)) for part in step["cmd"]]
    timeout = step.get("timeoutSec", ORACLE_STEP_TIMEOUT_DEFAULT)
    try:
        proc = subprocess.run(
            cmd,
            capture_output=True,
            timeout=timeout,
            cwd=str(ctx.work_dir),
        )
    except subprocess.TimeoutExpired as exc:
        raise CheckFailure(f"오라클 명령 타임아웃({timeout}s): {cmd}") from exc
    expect_exit = step.get("expectExit", 0)
    if proc.returncode != expect_exit:
        stderr = proc.stderr.decode("utf-8", errors="replace")[:400]
        raise CheckFailure(
            f"오라클 명령 exit {proc.returncode} (기대 {expect_exit}): {cmd}\n{stderr}"
        )
    stdout = proc.stdout.decode("utf-8", errors="replace")
    save = step.get("save")
    if save:
        ctx.save_text[save] = stdout
        if step.get("parse", "json") == "json":
            try:
                ctx.saves[save] = json.loads(stdout)
            except json.JSONDecodeError as exc:
                raise CheckFailure(
                    f"오라클 stdout 이 JSON 이 아닙니다({exc}): {cmd}\n{stdout[:300]}"
                ) from exc
        else:
            ctx.saves[save] = stdout
    if step.get("stdoutTo"):
        dest = Path(ctx.expand(step["stdoutTo"]))
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(proc.stdout)
    if step.get("extractPagesTextTo"):
        # export-text --json 봉투의 pages[].text 를 이어 붙여 파일로 남긴다.
        if save is None or not isinstance(ctx.saves.get(save), dict):
            raise CheckFailure("extractPagesTextTo 는 save+JSON 파싱을 전제합니다")
        pages = ctx.saves[save].get("pages")
        if not isinstance(pages, list):
            raise CheckFailure(f"봉투에 pages 배열이 없습니다: {cmd}")
        joined = "\n".join(str(p.get("text", "")) for p in pages)
        dest = Path(ctx.expand(step["extractPagesTextTo"]))
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(joined, encoding="utf-8")


def _read_csv(path: Path) -> list[list[str]]:
    with path.open(newline="", encoding="utf-8-sig") as fh:
        return [row for row in csv.reader(fh)]


def _normalize_ws(text: str) -> str:
    return "\n".join(" ".join(line.split()) for line in text.splitlines() if line.strip())


def run_check(ctx: TaskContext, check: dict) -> None:
    kind = check["type"]
    label = check.get("label", kind)

    def fail(msg: str):
        raise CheckFailure(f"[{label}] {msg}")

    if kind == "file_exists":
        path = Path(ctx.expand(check["path"]))
        if not path.is_file() or path.stat().st_size == 0:
            fail(f"산출물이 없거나 비었습니다: {path}")
        return

    if kind == "json":
        actual = ctx.resolve_value({"from": check["from"], "path": check.get("path", "")})
        expected = ctx.resolve_value(check.get("value"))
        op = check.get("op", "eq")
        if not OPS[op](actual, expected):
            fail(f"{check.get('path')} = {actual!r} (기대 {op} {expected!r})")
        return

    if kind == "json_list":
        seq = ctx.resolve_value({"from": check["from"], "path": check["listPath"]})
        if not isinstance(seq, list):
            fail(f"{check['listPath']} 가 배열이 아닙니다")
        where = check.get("where", {})
        matched = [
            item
            for item in seq
            if isinstance(item, dict) and all(item.get(k) == v for k, v in where.items())
        ]
        if not matched:
            fail(f"{check['listPath']} 에서 {where} 항목을 찾지 못했습니다")
        actual = matched[0].get(check["field"]) if "field" in check else matched[0]
        expected = ctx.resolve_value(check.get("value"))
        op = check.get("op", "eq")
        if not OPS[op](actual, expected):
            fail(f"{where} 의 {check.get('field')} = {actual!r} (기대 {op} {expected!r})")
        return

    if kind == "json_sum":
        seq = ctx.resolve_value({"from": check["from"], "path": check["listPath"]})
        if not isinstance(seq, list):
            fail(f"{check['listPath']} 가 배열이 아닙니다")
        try:
            total = sum(item[check["field"]] for item in seq)
        except (KeyError, TypeError) as exc:
            fail(f"합산 실패: {exc}")
        expected = ctx.resolve_value(check.get("value"))
        op = check.get("op", "eq")
        if not OPS[op](total, expected):
            fail(f"합계 {total!r} (기대 {op} {expected!r})")
        return

    if kind == "text_contains":
        text = ctx.resolve_value(
            {"file": check["file"]} if "file" in check else {"from": check["from"]}
        )
        if not isinstance(text, str):
            text = json.dumps(text, ensure_ascii=False)
        needle = check["needle"]
        present = needle in text
        if present != check.get("expect", True):
            fail(f"{needle!r} 포함 여부 {present} (기대 {check.get('expect', True)})")
        return

    if kind == "bytes_equal":
        a = Path(ctx.expand(check["a"]))
        b = Path(ctx.expand(check["b"]))
        if not a.is_file() or not b.is_file():
            fail(f"비교 파일이 없습니다: {a} / {b}")
        same = a.read_bytes() == b.read_bytes()
        if same != check.get("expect", True):
            fail(f"바이트 동일 여부 {same} (기대 {check.get('expect', True)})")
        return

    if kind == "text_similarity":
        a = ctx.resolve_value({"file": check["a"]})
        b = ctx.resolve_value({"file": check["b"]})
        ratio = difflib.SequenceMatcher(
            None, _normalize_ws(a), _normalize_ws(b)
        ).ratio()
        threshold = check.get("value", 0.98)
        if not OPS[check.get("op", "ge")](ratio, threshold):
            fail(f"유사도 {ratio:.4f} (기대 {check.get('op', 'ge')} {threshold})")
        return

    if kind == "csv_cells":
        rows = _read_csv(Path(ctx.expand(check["file"])))
        for cell in check["cells"]:
            r, c = cell["row"], cell["col"]
            if r >= len(rows) or c >= len(rows[r]):
                fail(f"CSV 에 ({r},{c}) 칸이 없습니다")
            if rows[r][c] != cell["eq"]:
                fail(f"({r},{c}) = {rows[r][c]!r} (기대 {cell['eq']!r})")
        return

    if kind == "csv_equal_except":
        rows_a = _read_csv(Path(ctx.expand(check["a"])))
        rows_b = _read_csv(Path(ctx.expand(check["b"])))
        if len(rows_a) != len(rows_b):
            fail(f"행 수 불일치: {len(rows_a)} vs {len(rows_b)}")
        allowed = {tuple(cell) for cell in check.get("except", [])}
        diffs = []
        for r, (ra, rb) in enumerate(zip(rows_a, rows_b)):
            if len(ra) != len(rb):
                fail(f"{r}행 열 수 불일치: {len(ra)} vs {len(rb)}")
            for c, (va, vb) in enumerate(zip(ra, rb)):
                if va != vb:
                    diffs.append((r, c))
        unexpected = [d for d in diffs if d not in allowed]
        missing = [d for d in allowed if d not in diffs]
        if unexpected:
            fail(f"허용 밖 변경 칸: {unexpected[:5]}")
        if missing:
            fail(f"바뀌어야 할 칸이 안 바뀌었습니다: {missing}")
        return

    fail(f"알 수 없는 검사 유형: {kind}")


# ── 태스크 실행 ──────────────────────────────────────────────────────


def run_task(task: dict, rhwp: Path, solutions_dir: Path, keep_work: bool) -> dict:
    task_id = task["id"]
    started = time.monotonic()
    result = {"id": task_id, "title": task.get("title", ""), "passed": False}

    fixtures = [REPO_ROOT / f for f in task.get("fixtures", [])]
    for f in fixtures:
        if not f.is_file():
            result.update(status="FIXTURE_MISSING", detail=str(f))
            return result

    work_root = Path(tempfile.mkdtemp(prefix=f"agent-bench-{task_id}-"))
    out_dir = work_root / "out"
    work_dir = work_root / "work"
    out_dir.mkdir()
    work_dir.mkdir()
    ctx = TaskContext(rhwp, fixtures, out_dir, work_dir)

    try:
        # 1) setup — 오라클과 같은 스텝 문법. 실패는 하니스 오류다.
        for step in task.get("setup", []):
            try:
                run_step(ctx, step)
            except CheckFailure as exc:
                result.update(status="SETUP_ERROR", detail=str(exc))
                return result

        solution_input = ctx.expand(task.get("solutionInput", "{FIXTURE}"))

        # 2) 풀이 실행
        solution = solutions_dir / f"{task_id}.py"
        if not solution.is_file():
            result.update(status="NO_SOLUTION", detail=str(solution))
            return result
        env = os.environ.copy()
        env.update(
            RHWP_BIN=str(rhwp),
            BENCH_TASK_ID=task_id,
            BENCH_INPUT=solution_input,
            BENCH_INPUTS_JSON=json.dumps(
                [solution_input] if len(fixtures) <= 1 else [str(f) for f in fixtures],
                ensure_ascii=False,
            ),
            BENCH_PARAMS_JSON=json.dumps(task.get("params", {}), ensure_ascii=False),
            BENCH_OUT_DIR=str(out_dir),
            PYTHONIOENCODING="utf-8",
        )
        timeout = task.get("timeoutSec", 300)
        try:
            proc = subprocess.run(
                [sys.executable, str(solution)],
                capture_output=True,
                timeout=timeout,
                cwd=str(out_dir),
                env=env,
            )
        except subprocess.TimeoutExpired:
            result.update(status="SOLUTION_TIMEOUT", detail=f"{timeout}s")
            return result
        if proc.returncode != 0:
            stderr = proc.stderr.decode("utf-8", errors="replace")[:400]
            result.update(status="SOLUTION_ERROR", detail=f"exit {proc.returncode}: {stderr}")
            return result

        # 3) 오라클 — 명령열 실행 후 검사 평가
        try:
            for step in task["oracle"].get("steps", []):
                run_step(ctx, step)
            for check in task["oracle"].get("checks", []):
                run_check(ctx, check)
        except CheckFailure as exc:
            result.update(status="ORACLE_FAIL", detail=str(exc))
            return result

        result.update(status="PASS", passed=True)
        return result
    finally:
        result["durationSec"] = round(time.monotonic() - started, 2)
        if keep_work:
            result["workDir"] = str(work_root)
        else:
            shutil.rmtree(work_root, ignore_errors=True)


def main() -> int:
    _reconfigure_stdout()
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--solutions", required=True, help="풀이 디렉터리(<task_id>.py 모음)")
    parser.add_argument("--tasks", help="쉼표로 구분한 태스크 id 부분집합")
    parser.add_argument("--rhwp-bin", help="rhwp 바이너리 경로(기본: env RHWP_BIN)")
    parser.add_argument("--json-out", help="결과 JSON 저장 경로")
    parser.add_argument("--keep-work", action="store_true", help="작업 디렉터리 보존(디버깅)")
    args = parser.parse_args()

    rhwp_raw = args.rhwp_bin or os.environ.get("RHWP_BIN")
    if not rhwp_raw:
        print("오류: RHWP_BIN 환경 변수 또는 --rhwp-bin 이 필요합니다.", file=sys.stderr)
        return 2
    rhwp = Path(rhwp_raw).resolve()
    if not rhwp.is_file():
        print(f"오류: rhwp 바이너리가 없습니다 - {rhwp}", file=sys.stderr)
        return 2

    solutions_dir = Path(args.solutions).resolve()
    if not solutions_dir.is_dir():
        print(f"오류: 풀이 디렉터리가 없습니다 - {solutions_dir}", file=sys.stderr)
        return 2

    try:
        spec = json.loads(TASKS_JSON.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"오류: tasks.json 을 읽을 수 없습니다 - {exc}", file=sys.stderr)
        return 2
    tasks = spec["tasks"]
    if args.tasks:
        wanted = {t.strip() for t in args.tasks.split(",") if t.strip()}
        unknown = wanted - {t["id"] for t in tasks}
        if unknown:
            print(f"오류: 알 수 없는 태스크 id - {sorted(unknown)}", file=sys.stderr)
            return 2
        tasks = [t for t in tasks if t["id"] in wanted]

    results = [run_task(t, rhwp, solutions_dir, args.keep_work) for t in tasks]
    passed = sum(1 for r in results if r["passed"])

    width = max(len(r["id"]) for r in results)
    print(f"\n{'태스크':<{width}}  판정          시간    상세")
    print("-" * (width + 50))
    for r in results:
        detail = "" if r["passed"] else (r.get("detail", "") or "")[:80].replace("\n", " ")
        print(f"{r['id']:<{width}}  {r['status']:<12}  {r.get('durationSec', 0):>5.1f}s  {detail}")
    print("-" * (width + 50))
    print(f"성공률: {passed}/{len(results)}")

    if args.json_out:
        payload = {
            "summary": {"total": len(results), "passed": passed, "failed": len(results) - passed},
            "rhwpBin": str(rhwp),
            "solutionsDir": str(solutions_dir),
            "tasks": results,
        }
        Path(args.json_out).write_text(
            json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8"
        )

    return 0 if passed == len(results) else 1


if __name__ == "__main__":
    sys.exit(main())
