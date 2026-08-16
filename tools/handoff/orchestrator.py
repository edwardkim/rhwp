#!/usr/bin/env python3
"""Agent Capability Handoff 오케스트레이터 — 외부 에이전트에 task 를 위임하고,
반환된 결과·capability 를 검증한 뒤에만 RHWP 에이전트 루프로 반입한다.

## 왜 있는가 (기존 축과의 관계)

이 저장소의 에이전트 축은 이미 넷이다 — `run`(Plan→Act→Verify: 정적 선검증·CAS·
원자 실행·R23 지문 저널), `tools/repair_loop/loop.py`(Retry/Continue: 진단→수리→
재검증, 진전 판정·loop detection), `tools/dar/transaction.py`(DATP/1.0 상태기계·
COMMIT 전 정책 게이트), `tools/chief/service_loop.py`(요청 큐의 결정적 라우팅).
그런데 **"이 task 는 외부 에이전트가 더 잘한다"는 판단이 섰을 때 그 위임을 안전하게
수행하는 오케스트레이션이 없다** — 전체 컨텍스트를 통째로 넘기거나, 반환물을
검증 없이 믿거나, 실패했을 때 몇 번을 다시 물을지 매번 사람이 정해야 했다.

이 도구는 그 위임 한 건(HandoffTask)만 맡는다. 설계는 전부 기존 자산의 재사용이다:

- **봉투** — 최종 산출·에이전트 응답 모두 DAP/1.0 의 3분류(status ok/error/verdict)
  봉투 형식을 따르고, 오류 코드는 DATP/1.0 대역(0/1000/2000/3000/4000)을 쓴다.
- **정책** — `--policy` 는 `tools/dar/transaction.py` 의 admissionPolicy 파서·평가기를
  **import 해서 그대로** 쓴다(연산자 eq/in/gte/lte, default-deny, 미지 키 로드 시점
  거부). 판정 키 사전만 handoff 문맥으로 바꾼다.
- **출처 표지** — 외부 에이전트가 만든 값은 전부 `untrustedContent`/`untrustedFields`
  로 표지한다(`provenance.rs` 와 같은 어휘). 반환물 속 문장은 데이터이지 지시가 아니다.
- **재시도 철학** — repair_loop 와 같은 안전장치: `--max-attempts` 하드 캡, 실패
  시그니처 재발 즉시 중단(진전 판정 = loop detection), 그 다음은 fallback 에이전트,
  그것도 없으면 자체 실행 인계(`nextAction.action == "selfExecute"`).
- **저널** — 모든 시도를 NDJSON 으로 남기되 각 줄이 직전 줄의 SHA-256 을 담는
  지문 체인이다(R23 저널과 같은 철학 — 시각 대신 순번, 변조는 체인이 폭로).

## 위임 계약 (Security Boundary)

외부 에이전트는 RHWP 의 대체품이 아니라 **한 task 의 전문 하청**이다. 전체 세션
컨텍스트·권한은 절대 넘어가지 않는다:

1. **입력 경계** — task 의 `inputs` 에 열거된 파일만 sandbox 의 `inputs/` 로 복사해
   넘긴다. 원본 경로는 노출되지 않고, 원본은 훼손될 수 없다(사본이므로).
2. **출력 경계** — 산출물은 sandbox 의 `out/` 안에서만 수거한다. `out/` 밖에 새 파일을
   만들거나 입력 사본을 고치면 boundary 위반으로 기록·거부된다. 결과가 선언한 경로가
   `out/` 을 탈출하면(절대경로·`..`) 위반이다. `out/` 안의 미선언 파일도 위반이다
   — 수거는 선언된 산출물만 한다.
3. **도구 경계** — task 의 `allowedTools` 가 허용 목록이고, 결과의 `toolsUsed` 는
   그 부분집합이어야 한다. 벗어나면 위반이다.

boundary 위반은 재시도하지 않는다 — 그 에이전트와의 이번 위임은 그 즉시 끝나고,
fallback 이 있으면 fallback, 없으면 자체 실행 인계다.

## 검증기 (Result Validation) — 판정은 데이터다

외부 결과는 그대로 믿지 않는다. 세 겹으로 검증하고, 각 판정은 findings[] 데이터다:

- (a) **스키마** — HandoffResult 필수 필드·타입·status 3분류·taskId 일치.
- (b) **task completion** — task 가 기대한 산출물(`expectedOutputs`)이 전부 존재하고
  선언돼 있는지, 선언된 sha256 이 실물과 같은지.
- (c) **consistency** — `mustParse` 산출물은 rhwp CLI(`info --json`)로 실제로 다시
  열어 본다(`--bin` 지정 시). 에이전트의 성공 보고를 믿지 않고 재검증한다 —
  DATP VALIDATE 와 같은 태도다.

## 사용

    python tools/handoff/orchestrator.py --task task.json \\
        --agent "python 외부에이전트.py" \\
        --fallback-agent "python 예비에이전트.py" \\
        --bin target/release/rhwp --max-attempts 3 \\
        --work-dir output/handoff --json

    python tools/handoff/orchestrator.py --verify-journal output/handoff/handoff.journal.ndjson --json

어댑터 계약(서브프로세스): 에이전트 명령은 sandbox 를 cwd 로 실행되고, stdin 으로
HandoffTask JSON(wire 형식)을 받아 stdout 으로 HandoffResult JSON 하나를 낸다.

HandoffTask (파일):
    {"handoffVersion":"1.0","taskId":"t1","objective":"…",
     "inputs":["samples/a.hwpx"],"allowedTools":["rhwp export-hwpx"],
     "timeoutSec":30,
     "expectedOutputs":[{"path":"converted.hwpx","mustParse":true}]}

HandoffResult (에이전트 stdout):
    {"handoffVersion":"1.0","taskId":"t1","status":"ok",
     "outputs":[{"path":"converted.hwpx","sha256":"…"}],
     "capabilities":[{"name":"hwpx-convert","kind":"command","detail":"…"}],
     "toolsUsed":["rhwp export-hwpx"]}

종료 코드는 DATP/1.0 대역의 상위 1자리다: 0 수용 / 1 런타임(spawn·timeout 소진·
결과 파싱 불가) / 2 사용법(task 스키마·인자 오류) / 3 판정(검증 실패로 인계 —
실패가 아니라 결과다) / 4 정책·boundary 거부.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, Optional

REPO_ROOT = Path(__file__).resolve().parents[2]
HANDOFF_VERSION = "1.0"
TOOL_NAME = "rhwp-handoff-orchestrator"

RESULT_STATUSES = ("ok", "error", "verdict")
CAPABILITY_KINDS = ("command", "knowledge", "artifact")

# handoff 수용 정책의 판정 키 사전 — 정책 언어(파서·평가기)는 tools/dar/transaction.py
# 를 import 해 재사용하고, 사전만 이 문맥으로 바꾼다(transaction.py 의 --policy 와
# 같은 구조: 같은 언어, 다른 사전).
POLICY_JUDGMENT_KEYS = ("status", "validated", "violationCount", "agent", "attempt")

# 재시도해 볼 가치가 있는 실패 — repair_loop 와 같은 시그니처 진전 판정이 상한이다.
RETRYABLE = {"timeout", "agentError", "unparseableResult", "schemaViolation",
             "incompleteResult", "inconsistentResult"}
# boundary·정책 위반과 에이전트 자신의 판정은 같은 에이전트에게 재시도하지 않는다.
ADAPTER_FINAL = {"securityViolation", "policyRejected", "agentVerdict", "spawnError"}


def log(msg: str) -> None:
    print(msg, file=sys.stderr)


def sha256_file(p: Path) -> str:
    h = hashlib.sha256()
    with p.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def sha256_obj(obj: Any) -> str:
    return sha256_text(json.dumps(obj, ensure_ascii=False, sort_keys=True))


def _load_dar_policy_module():
    """정책 파서·평가기의 단일 출처는 tools/dar/transaction.py 다 — 재구현하지 않는다."""
    path = REPO_ROOT / "tools" / "dar" / "transaction.py"
    spec = importlib.util.spec_from_file_location("rhwp_dar_transaction", path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _agent_argv(cmd: str) -> list[str]:
    # repair_loop 의 --bin 과 같은 규약: 보통 실행파일 하나지만
    # `<파이썬> <스크립트.py>` 같은 다중 토큰도 허용한다.
    return shlex.split(cmd, posix=(os.name != "nt"))


# ---------------------------------------------------------------------------
# HandoffTask — 위임 명세의 정적 선검증 (`run` 의 invalid[] 철학)
# ---------------------------------------------------------------------------

def validate_task(task: Any) -> list[str]:
    """task 스키마 위반 목록 — 비어 있지 않으면 위임을 시작하지 않는다(사용법 오류)."""
    problems: list[str] = []
    if not isinstance(task, dict):
        return ["task 최상위는 객체여야 한다"]
    if task.get("handoffVersion") != HANDOFF_VERSION:
        problems.append(f"handoffVersion 은 '{HANDOFF_VERSION}' 이어야 한다: "
                        f"{task.get('handoffVersion')!r}")
    task_id = task.get("taskId")
    if not isinstance(task_id, str) or not task_id or any(
            c for c in task_id if not (c.isalnum() or c in "._-")):
        problems.append("taskId 는 [A-Za-z0-9._-] 비어 있지 않은 문자열이어야 한다")
    if not isinstance(task.get("objective"), str) or not task.get("objective"):
        problems.append("objective 는 비어 있지 않은 문자열이어야 한다")
    inputs = task.get("inputs")
    if not isinstance(inputs, list) or not all(isinstance(i, str) for i in inputs):
        problems.append("inputs 는 문자열 배열이어야 한다")
    else:
        names = [Path(i).name for i in inputs]
        if len(set(names)) != len(names):
            problems.append("inputs 파일 이름이 겹친다 — sandbox 복사에서 충돌한다")
        for i in inputs:
            if not Path(i).is_file():
                problems.append(f"입력 파일이 없다: {i}")
    tools = task.get("allowedTools")
    if not isinstance(tools, list) or not all(isinstance(t, str) for t in tools):
        problems.append("allowedTools 는 문자열 배열이어야 한다")
    timeout = task.get("timeoutSec")
    if not isinstance(timeout, (int, float)) or isinstance(timeout, bool) or timeout <= 0:
        problems.append("timeoutSec 는 0 보다 큰 수여야 한다")
    expected = task.get("expectedOutputs")
    if not isinstance(expected, list):
        problems.append("expectedOutputs 는 배열이어야 한다")
    else:
        for idx, e in enumerate(expected):
            if not isinstance(e, dict) or not isinstance(e.get("path"), str) or not e.get("path"):
                problems.append(f"expectedOutputs[{idx}].path 는 비어 있지 않은 문자열이어야 한다")
                continue
            if not _is_safe_relative(e["path"]):
                problems.append(f"expectedOutputs[{idx}].path 는 out/ 안의 상대 경로여야 한다: {e['path']}")
            if "mustParse" in e and not isinstance(e["mustParse"], bool):
                problems.append(f"expectedOutputs[{idx}].mustParse 는 불리언이어야 한다")
    return problems


def _is_safe_relative(rel: str) -> bool:
    p = Path(rel)
    if p.is_absolute():
        return False
    parts = p.parts
    return bool(parts) and ".." not in parts and not any(":" in part for part in parts)


# ---------------------------------------------------------------------------
# 어댑터 — 서브프로세스 외부 에이전트 (sandbox 에서 실제 프로세스를 띄운다)
# ---------------------------------------------------------------------------

def prepare_sandbox(root: Path, task: dict) -> dict:
    """입력 경계 — inputs 만 사본으로 넘긴다. 반환값은 사본 지문(변조 감지 기준)."""
    inputs_dir = root / "inputs"
    out_dir = root / "out"
    inputs_dir.mkdir(parents=True)
    out_dir.mkdir(parents=True)
    input_hashes: dict[str, str] = {}
    for src in task["inputs"]:
        dst = inputs_dir / Path(src).name
        shutil.copy2(src, dst)
        input_hashes[dst.name] = sha256_file(dst)
    return {"root": root, "inputsDir": inputs_dir, "outDir": out_dir,
            "inputHashes": input_hashes}


def wire_task(task: dict) -> dict:
    """에이전트에게 넘기는 형식 — 원본 경로 대신 sandbox 상대 경로만 노출한다."""
    return {
        "handoffVersion": HANDOFF_VERSION,
        "taskId": task["taskId"],
        "objective": task["objective"],
        "inputs": ["inputs/" + Path(i).name for i in task["inputs"]],
        "outDir": "out",
        "allowedTools": list(task["allowedTools"]),
        "timeoutSec": task["timeoutSec"],
        "expectedOutputs": [e["path"] for e in task["expectedOutputs"]],
    }


def run_agent(agent_cmd: str, sandbox: dict, task: dict) -> dict:
    """외부 에이전트 1회 실행 — 진짜 프로세스, stdin: task JSON, stdout: result JSON."""
    argv = _agent_argv(agent_cmd)
    stdin_text = json.dumps(wire_task(task), ensure_ascii=False)
    try:
        p = subprocess.run(
            argv, input=stdin_text, capture_output=True, text=True,
            encoding="utf-8", errors="replace", cwd=str(sandbox["root"]),
            timeout=float(task["timeoutSec"]),
        )
    except subprocess.TimeoutExpired:
        return {"timedOut": True, "returncode": None, "stdout": "", "stderr": ""}
    except OSError as e:
        return {"spawnError": str(e), "returncode": None, "stdout": "", "stderr": ""}
    return {"timedOut": False, "returncode": p.returncode,
            "stdout": p.stdout, "stderr": p.stderr}


# ---------------------------------------------------------------------------
# 검증기 — (a) 스키마 (b) completion (c) consistency + boundary
# ---------------------------------------------------------------------------

def _finding(check: str, code: str, reason: str) -> dict:
    return {"check": check, "code": code, "reason": reason}


def validate_result_schema(task: dict, result: Any) -> list[dict]:
    f: list[dict] = []
    if not isinstance(result, dict):
        return [_finding("schema", "notObject", "HandoffResult 최상위는 객체여야 한다")]
    if result.get("handoffVersion") != HANDOFF_VERSION:
        f.append(_finding("schema", "badVersion",
                          f"handoffVersion 불일치: {result.get('handoffVersion')!r}"))
    if result.get("taskId") != task["taskId"]:
        f.append(_finding("schema", "taskIdMismatch",
                          f"taskId 불일치: {result.get('taskId')!r} != {task['taskId']!r}"))
    if result.get("status") not in RESULT_STATUSES:
        f.append(_finding("schema", "badStatus",
                          f"status 는 {list(RESULT_STATUSES)} 중 하나여야 한다: "
                          f"{result.get('status')!r}"))
    outputs = result.get("outputs")
    if not isinstance(outputs, list):
        f.append(_finding("schema", "badOutputs", "outputs 는 배열이어야 한다"))
    else:
        for idx, o in enumerate(outputs):
            if not isinstance(o, dict) or not isinstance(o.get("path"), str) or not o.get("path"):
                f.append(_finding("schema", "badOutputEntry",
                                  f"outputs[{idx}].path 는 비어 있지 않은 문자열이어야 한다"))
                continue
            claimed = o.get("sha256")
            if (not isinstance(claimed, str) or len(claimed) != 64
                    or any(ch not in "0123456789abcdefABCDEF" for ch in claimed)):
                f.append(_finding(
                    "schema",
                    "badOutputSha256",
                    f"outputs[{idx}].sha256 은 64자리 16진수 SHA-256 이어야 한다",
                ))
    caps = result.get("capabilities", [])
    if not isinstance(caps, list):
        f.append(_finding("schema", "badCapabilities", "capabilities 는 배열이어야 한다"))
    else:
        for idx, c in enumerate(caps):
            if (not isinstance(c, dict) or not isinstance(c.get("name"), str)
                    or not c.get("name") or c.get("kind") not in CAPABILITY_KINDS):
                f.append(_finding("schema", "badCapability",
                                  f"capabilities[{idx}] 는 name(문자열)·kind({list(CAPABILITY_KINDS)}) "
                                  "가 있어야 한다"))
    tools_used = result.get("toolsUsed", [])
    if not isinstance(tools_used, list) or not all(isinstance(t, str) for t in tools_used):
        f.append(_finding("schema", "badToolsUsed", "toolsUsed 는 문자열 배열이어야 한다"))
    return f


def validate_boundary(task: dict, result: dict, sandbox: dict) -> list[dict]:
    """Security Boundary — 입력 불훼손 · out/ 밖 무기록 · 도구 허용 목록 준수."""
    f: list[dict] = []
    root: Path = sandbox["root"]
    out_dir: Path = sandbox["outDir"]

    # 1. 입력 사본 변조
    for name, digest in sandbox["inputHashes"].items():
        p = sandbox["inputsDir"] / name
        if not p.is_file():
            f.append(_finding("boundary", "inputDeleted", f"입력 사본이 삭제됐다: inputs/{name}"))
        elif sha256_file(p) != digest:
            f.append(_finding("boundary", "inputModified", f"입력 사본이 변조됐다: inputs/{name}"))

    # 2. out/ 밖에 만든 파일
    known = {sandbox["inputsDir"] / n for n in sandbox["inputHashes"]}
    for p in sorted(root.rglob("*")):
        if not p.is_file():
            continue
        try:
            p.relative_to(out_dir)
            continue  # out/ 안은 3에서 다룬다
        except ValueError:
            pass
        if p not in known:
            f.append(_finding("boundary", "wroteOutsideOut",
                              f"out/ 밖에 파일을 만들었다: {p.relative_to(root).as_posix()}"))

    # 3. 선언 산출 경로의 탈출과 미선언 산출
    declared: set[Path] = set()
    for o in result.get("outputs", []):
        rel = o.get("path")
        if not isinstance(rel, str) or not rel:
            continue
        if not _is_safe_relative(rel):
            f.append(_finding("boundary", "outputPathEscape",
                              f"산출 경로가 out/ 을 탈출한다: {rel}"))
            continue
        target = (out_dir / rel).resolve()
        try:
            target.relative_to(out_dir.resolve())
        except ValueError:
            f.append(_finding("boundary", "outputPathEscape",
                              f"산출 경로가 out/ 을 탈출한다: {rel}"))
            continue
        declared.add(target)
    for p in sorted(out_dir.rglob("*")):
        if p.is_file() and p.resolve() not in declared:
            f.append(_finding("boundary", "undeclaredOutput",
                              f"선언되지 않은 산출물: out/{p.relative_to(out_dir).as_posix()} "
                              "— 수거는 선언된 것만 한다"))

    # 4. 도구 허용 목록
    allowed = set(task["allowedTools"])
    for t in result.get("toolsUsed", []):
        if isinstance(t, str) and t not in allowed:
            f.append(_finding("boundary", "toolNotAllowed",
                              f"허용되지 않은 도구를 썼다고 선언했다: {t}"))
    return f


def validate_completion(task: dict, result: dict, sandbox: dict) -> list[dict]:
    """task completion — 기대 산출물 존재·선언·해시 일치."""
    f: list[dict] = []
    out_dir: Path = sandbox["outDir"]
    declared = {o["path"]: o for o in result.get("outputs", [])
                if isinstance(o, dict) and isinstance(o.get("path"), str)}
    for e in task["expectedOutputs"]:
        rel = e["path"]
        p = out_dir / rel
        if not p.is_file():
            f.append(_finding("completion", "missingExpectedOutput",
                              f"기대 산출물이 없다: out/{rel}"))
            continue
        if rel not in declared:
            f.append(_finding("completion", "expectedOutputUndeclared",
                              f"기대 산출물이 결과에 선언되지 않았다: {rel}"))
    for rel, o in declared.items():
        p = out_dir / rel
        if not _is_safe_relative(rel) or not p.is_file():
            if _is_safe_relative(rel):
                f.append(_finding("completion", "missingDeclaredOutput",
                                  f"선언한 산출물이 실제로 없다: out/{rel}"))
            continue
        claimed = o.get("sha256")
        if isinstance(claimed, str) and claimed.lower() != sha256_file(p):
            f.append(_finding("completion", "sha256Mismatch",
                              f"선언한 sha256 이 실물과 다르다: {rel}"))
    return f


def validate_consistency(task: dict, sandbox: dict, bin_path: Optional[str],
                         timeout: float = 60.0) -> list[dict]:
    """consistency — mustParse 산출물은 rhwp 로 실제로 다시 열어 본다."""
    f: list[dict] = []
    out_dir: Path = sandbox["outDir"]
    for e in task["expectedOutputs"]:
        if not e.get("mustParse"):
            continue
        p = out_dir / e["path"]
        if not p.is_file():
            continue  # completion 이 이미 잡았다
        if not bin_path:
            f.append(_finding("consistency", "unchecked",
                              f"mustParse 산출물을 재검증할 rhwp 가 없다(--bin 미지정): {e['path']}"))
            continue
        try:
            proc = subprocess.run(
                _agent_argv(bin_path) + ["info", str(p), "--json"],
                capture_output=True, text=True, encoding="utf-8",
                errors="replace", timeout=timeout)
        except (subprocess.TimeoutExpired, OSError) as exc:
            f.append(_finding("consistency", "reverifyFailed",
                              f"재검증 실행 실패({e['path']}): {exc}"))
            continue
        if proc.returncode != 0:
            f.append(_finding("consistency", "notParseable",
                              f"산출물이 rhwp 로 열리지 않는다(exit {proc.returncode}): {e['path']}"))
    return f


# ---------------------------------------------------------------------------
# 진단 — 시도 하나를 (category, signature, findings) 로 분류한다
# ---------------------------------------------------------------------------

def diagnose_attempt(task: dict, run: dict, sandbox: dict,
                     bin_path: Optional[str]) -> dict:
    if run.get("timedOut"):
        return {"category": "timeout", "signature": ["timeout"],
                "findings": [_finding("adapter", "timeout",
                                      f"에이전트가 {task['timeoutSec']}초 안에 끝나지 않았다")],
                "result": None}
    if run.get("spawnError"):
        return {"category": "spawnError", "signature": ["spawnError"],
                "findings": [_finding("adapter", "spawnError", run["spawnError"])],
                "result": None}
    try:
        result = json.loads(run["stdout"])
    except json.JSONDecodeError:
        return {"category": "unparseableResult", "signature": ["unparseableResult"],
                "findings": [_finding("adapter", "unparseableResult",
                                      "stdout 이 JSON HandoffResult 가 아니다")],
                "result": None}

    schema = validate_result_schema(task, result)
    if schema:
        return {"category": "schemaViolation",
                "signature": ["schemaViolation"] + sorted(x["code"] for x in schema),
                "findings": schema, "result": result}

    if result["status"] == "verdict":
        # 에이전트 자신의 판정(예: "이 입력으로는 불가") — 실패가 아니라 결과다.
        return {"category": "agentVerdict", "signature": ["agentVerdict"],
                "findings": [], "result": result}
    if result["status"] == "error":
        return {"category": "agentError",
                "signature": ["agentError", str((result.get("error") or {}).get("code"))],
                "findings": [_finding("adapter", "agentError",
                                      "에이전트가 status=error 를 반환했다")],
                "result": result}

    boundary = validate_boundary(task, result, sandbox)
    if boundary:
        return {"category": "securityViolation",
                "signature": ["securityViolation"] + sorted(x["code"] for x in boundary),
                "findings": boundary, "result": result}

    completion = validate_completion(task, result, sandbox)
    if completion:
        return {"category": "incompleteResult",
                "signature": ["incompleteResult"] + sorted(x["code"] for x in completion),
                "findings": completion, "result": result}

    consistency = validate_consistency(task, sandbox, bin_path)
    if consistency:
        return {"category": "inconsistentResult",
                "signature": ["inconsistentResult"] + sorted(x["code"] for x in consistency),
                "findings": consistency, "result": result}

    return {"category": "accepted", "signature": ["accepted"], "findings": [],
            "result": result}


# ---------------------------------------------------------------------------
# 저널 — NDJSON 지문 체인 (R23 철학: 순번 + 직전 줄 해시)
# ---------------------------------------------------------------------------

class Journal:
    def __init__(self, path: Path):
        self.path = path
        self.seq = 0
        self.prev_sha: Optional[str] = None
        path.parent.mkdir(parents=True, exist_ok=True)
        if path.is_file():
            # 기존 저널에 이어 쓴다 — 연번과 지문 체인을 끊지 않는다.
            lines = [l for l in path.read_text(encoding="utf-8").splitlines() if l.strip()]
            if lines:
                self.seq = len(lines)
                self.prev_sha = sha256_text(lines[-1])

    def append(self, record: dict) -> None:
        self.seq += 1
        full = {"seq": self.seq, "prevSha256": self.prev_sha, **record}
        line = json.dumps(full, ensure_ascii=False)
        with self.path.open("a", encoding="utf-8") as fh:
            fh.write(line + "\n")
        self.prev_sha = sha256_text(line)


def verify_journal(path: Path) -> dict:
    """저널 자기 무결 — 연번과 지문 체인을 재계산한다. 깨짐은 오류가 아니라 데이터."""
    lines = [l for l in path.read_text(encoding="utf-8").splitlines() if l.strip()]
    prev_sha: Optional[str] = None
    broken_at: Optional[int] = None
    for i, line in enumerate(lines, start=1):
        try:
            rec = json.loads(line)
        except json.JSONDecodeError:
            broken_at = i
            break
        if rec.get("seq") != i or rec.get("prevSha256") != prev_sha:
            broken_at = i
            break
        prev_sha = sha256_text(line)
    return {"entries": len(lines), "chainValid": broken_at is None,
            "brokenAt": broken_at}


# ---------------------------------------------------------------------------
# 오케스트레이션 — Delegate → Validate → Retry/Fallback → Integrate
# ---------------------------------------------------------------------------

def orchestrate(task: dict, agents: list[dict], bin_path: Optional[str],
                max_attempts: int, work_dir: Path, journal: Journal,
                policy: Optional[dict], dar) -> dict:
    task_sha = sha256_obj(task)
    attempts_log: list[dict] = []
    accepted: Optional[dict] = None
    accepted_agent: Optional[str] = None
    accepted_sandbox: Optional[dict] = None
    last_category: Optional[str] = None

    for agent in agents:
        label = agent["label"]
        prev_signature: Optional[list] = None
        for attempt in range(1, max_attempts + 1):
            sandbox_root = work_dir / f"sandbox_{label}_a{attempt}"
            if sandbox_root.exists():
                shutil.rmtree(sandbox_root)
            sandbox = prepare_sandbox(sandbox_root, task)
            run = run_agent(agent["cmd"], sandbox, task)
            diag = diagnose_attempt(task, run, sandbox, bin_path)
            last_category = diag["category"]

            result_sha = sha256_text(run["stdout"]) if run.get("stdout") else None
            validated = diag["category"] == "accepted"
            policy_verdict = None
            if validated and policy is not None:
                judgments = {"status": diag["result"]["status"], "validated": True,
                             "violationCount": 0, "agent": label, "attempt": attempt}
                allow, violations = dar.evaluate_policy(policy, judgments)
                policy_verdict = {"allow": allow, "violations": violations}
                if not allow:
                    diag = {"category": "policyRejected",
                            "signature": ["policyRejected"],
                            "findings": [_finding("policy", "policyRejected",
                                                  f"수용 정책이 이 결과를 거부했다: {policy['name']}")],
                            "result": diag["result"]}
                    last_category = "policyRejected"
                    validated = False

            if validated:
                next_action = {"action": "consume",
                               "why": "검증을 전부 통과한 결과 — RHWP 에이전트 루프가 후속 작업에 쓴다"}
            elif diag["category"] in RETRYABLE and attempt < max_attempts \
                    and diag["signature"] != prev_signature:
                next_action = {"action": "retry",
                               "why": f"{diag['category']} — 재시도 여지가 있다"}
            elif agent is not agents[-1]:
                next_action = {"action": "fallback",
                               "why": f"{diag['category']} — 이 에이전트로는 더 진행하지 않는다"}
            else:
                next_action = {"action": "selfExecute",
                               "why": f"{diag['category']} — 위임을 접고 자체 실행으로 전환한다"}

            record = {
                "event": "attempt",
                "taskId": task["taskId"],
                "agent": label,
                "attempt": attempt,
                "taskSha256": task_sha,
                "inputsSha256": sandbox["inputHashes"],
                "resultSha256": result_sha,
                "category": diag["category"],
                "findings": diag["findings"],
                "policy": policy_verdict,
                "nextAction": next_action,
            }
            journal.append(record)
            attempts_log.append({"agent": label, "attempt": attempt,
                                 "category": diag["category"],
                                 "findings": diag["findings"],
                                 "nextAction": next_action})

            if validated:
                accepted, accepted_agent, accepted_sandbox = diag["result"], label, sandbox
                break
            if diag["category"] in ADAPTER_FINAL or diag["category"] == "policyRejected":
                break
            if diag["category"] not in RETRYABLE:
                break
            if prev_signature is not None and diag["signature"] == prev_signature:
                break  # 진전 없음 = loop detection — repair_loop 와 같은 불변식
            prev_signature = diag["signature"]
        if accepted is not None:
            break

    # 수용된 산출물만 sandbox 밖으로 수거한다 (출력 경계의 마지막 절반)
    collected: list[dict] = []
    if accepted is not None and accepted_sandbox is not None:
        collect_dir = work_dir / "collected" / task["taskId"]
        collect_dir.mkdir(parents=True, exist_ok=True)
        for o in accepted.get("outputs", []):
            src = accepted_sandbox["outDir"] / o["path"]
            dst = collect_dir / o["path"]
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dst)
            collected.append({"path": str(dst), "sha256": sha256_file(dst)})

    if accepted is not None:
        outcome, status, code = "accepted", "ok", 0
    elif last_category in ("securityViolation", "policyRejected"):
        outcome, status, code = "rejected", "verdict", 4000
    elif last_category in ("timeout", "spawnError", "unparseableResult"):
        outcome, status, code = "handoff", "error", 1000
    else:
        outcome, status, code = "handoff", "verdict", 3000

    final_next = attempts_log[-1]["nextAction"] if attempts_log else \
        {"action": "selfExecute", "why": "시도할 에이전트가 없었다"}

    untrusted_fields: list[str] = []
    if accepted is not None:
        untrusted_fields = ["result", "capabilities[].name", "capabilities[].kind",
                            "capabilities[].detail"]

    envelope = {
        "protocol": "DAP/1.0",
        "operation": "agent.handoff",
        "tool": TOOL_NAME,
        "handoffVersion": HANDOFF_VERSION,
        "taskId": task["taskId"],
        "taskSha256": task_sha,
        "status": status,
        "code": code,
        "outcome": outcome,
        "agentsTried": [a["label"] for a in agents],
        "acceptedAgent": accepted_agent,
        "attempts": attempts_log,
        "result": accepted,
        "capabilities": (accepted or {}).get("capabilities", []),
        "collectedOutputs": collected,
        "nextAction": final_next,
        "untrustedContent": accepted is not None,
        "untrustedFields": untrusted_fields,
        "journal": str(journal.path),
    }
    journal.append({"event": "final", "taskId": task["taskId"],
                    "outcome": outcome, "code": code,
                    "acceptedAgent": accepted_agent,
                    "collectedOutputs": collected,
                    "nextAction": final_next})
    return envelope


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main(argv: Optional[list[str]] = None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--task", help="HandoffTask JSON 파일")
    ap.add_argument("--agent", help="외부 에이전트 명령(서브프로세스 어댑터)")
    ap.add_argument("--agent-label", default="primary")
    ap.add_argument("--fallback-agent", help="1차 에이전트 소진 시 예비 명령")
    ap.add_argument("--fallback-label", default="fallback")
    ap.add_argument("--bin", help="consistency 재검증용 rhwp (기본: RHWP_BIN)")
    ap.add_argument("--max-attempts", type=int, default=3)
    ap.add_argument("--work-dir", help="sandbox·수거물·저널 루트 (기본: 임시 디렉터리)")
    ap.add_argument("--journal", help="NDJSON 저널 경로 (기본: <work-dir>/handoff.journal.ndjson)")
    ap.add_argument("--policy", help="admissionPolicy JSON — dar 정책 언어, handoff 판정 키")
    ap.add_argument("--verify-journal", help="저널 지문 체인만 검증하고 끝낸다")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args(argv)

    if args.verify_journal:
        p = Path(args.verify_journal)
        if not p.is_file():
            log(f"저널이 없다: {p}")
            return 2
        v = verify_journal(p)
        out = {"protocol": "DAP/1.0", "operation": "agent.handoff.verifyJournal",
               "status": "ok" if v["chainValid"] else "verdict",
               "code": 0 if v["chainValid"] else 3000,
               "untrustedContent": False, "untrustedFields": [], **v}
        print(json.dumps(out, ensure_ascii=False, indent=1))
        return 0 if v["chainValid"] else 3

    if not args.task or not args.agent:
        log("--task 와 --agent 는 필수다 (또는 --verify-journal 단독).")
        return 2
    task_path = Path(args.task)
    if not task_path.is_file():
        log(f"task 파일이 없다: {task_path}")
        return 2
    try:
        task = json.loads(task_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        log(f"task JSON 오류: {e}")
        return 2
    problems = validate_task(task)
    if problems:
        for p in problems:
            log(f"task 스키마 위반: {p}")
        return 2
    if args.max_attempts < 1:
        log("--max-attempts 는 1 이상이어야 한다")
        return 2

    dar = _load_dar_policy_module()
    policy = None
    if args.policy:
        try:
            policy = dar.parse_policy(Path(args.policy).read_text(encoding="utf-8"),
                                      judgment_keys=POLICY_JUDGMENT_KEYS)
        except (OSError, ValueError, json.JSONDecodeError) as e:
            log(f"정책 사용법 오류: {e}")
            return 2

    bin_path = args.bin or os.environ.get("RHWP_BIN")

    cleanup_tmp = None
    if args.work_dir:
        work_dir = Path(args.work_dir)
        work_dir.mkdir(parents=True, exist_ok=True)
    else:
        cleanup_tmp = tempfile.mkdtemp(prefix="handoff_")
        work_dir = Path(cleanup_tmp)

    journal = Journal(Path(args.journal) if args.journal
                      else work_dir / "handoff.journal.ndjson")

    agents = [{"label": args.agent_label, "cmd": args.agent}]
    if args.fallback_agent:
        agents.append({"label": args.fallback_label, "cmd": args.fallback_agent})

    envelope = orchestrate(task, agents, bin_path, args.max_attempts,
                           work_dir, journal, policy, dar)
    exit_code = envelope["code"] // 1000 if envelope["code"] else 0

    if args.json:
        print(json.dumps(envelope, ensure_ascii=False, indent=1))
    else:
        print(f"handoff[{task['taskId']}] — {envelope['outcome']} "
              f"(시도 {len(envelope['attempts'])}, 코드 {envelope['code']})")
        for a in envelope["attempts"]:
            mark = "✓" if a["category"] == "accepted" else "✗"
            print(f"  {mark} {a['agent']} 시도 {a['attempt']}: {a['category']} "
                  f"→ {a['nextAction']['action']}")

    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
