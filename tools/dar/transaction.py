#!/usr/bin/env python3
"""DATP/1.0 트랜잭션 드라이버 — 상태기계를 실제로 강제하는 참조 구현.

## 왜 있는가

[DATP/1.0](../../mydocs/tech/standards/document_transaction_protocol.md) 은 불변식
넷을 선언한다 — COMMIT 은 직전 VALIDATE 성공에만 허용, 검증 실패면 COMMIT 불가,
MODIFY 는 원본 무훼손, 한 트랜잭션은 한 입력 해시. **선언만으로는 아무것도 막지
못한다.** `tools/dar/conformance.py` 도 "상태기계 강제 — 미구현"이라고 보고했다.

이 드라이버가 그 강제다. 순서를 어긴 연산은 실행 자체를 거절하고, 검증에 실패한
트랜잭션은 COMMIT 경로가 **물리적으로 닫힌다**. 모든 연산은
[DAP/1.0](../../mydocs/tech/standards/document_agent_protocol.md) 봉투를 낸다.

결정적 코어다 — LLM 이 없어도 그대로 돈다. 에이전트는 PROPOSE 에 무엇을 담을지
정할 뿐이고, 그 제안이 검증을 통과하는지는 이 드라이버가 rhwp 로 판정한다.

## 사용

    T=tools/dar/transaction.py
    python3 $T begin    --doc 원본.hwpx --bin <rhwp> --tx-dir txs/
    python3 $T select   --tx <id> --query "찾을말"
    python3 $T propose  --tx <id> --op replace-text --params '{"find":"A","replace":"B"}'
    python3 $T modify   --tx <id>
    python3 $T validate --tx <id>
    python3 $T diff     --tx <id>
    python3 $T commit   --tx <id> -o 산출.hwpx      # VALIDATE 성공 없으면 거절
    python3 $T rollback --tx <id>
    python3 $T replay   --tx <id>                    # 영수증 재실행 상태를 기록
    python3 $T verify   --tx <id>                    # 영수증·입출력 해시 검증 상태를 기록
    python3 $T replay   --receipt txs/<id>/receipt.json --bin <rhwp>  # 독립 재현

종료 코드는 DAP/1.0 오류 코드의 상위 1자리다: 0 성공 / 1 런타임 / 2 사용법 /
3 판정 / 4 정책. **판정(3)은 실패가 아니라 결과다.**
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import uuid
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL_NAME = "rhwp-datp-driver"
DRIVER_VERSION = "1.0"

# DATP/1.0 기계 정본의 상태기계와 같은 표 — 정본이 바뀌면 여기도 같은 PR 에서 바꾼다.
TRANSITIONS = {
    None:       ["BEGIN"],
    "BEGIN":    ["READ", "SELECT", "PROPOSE", "ROLLBACK"],
    "READ":     ["READ", "SELECT", "PROPOSE", "ROLLBACK"],
    "SELECT":   ["READ", "SELECT", "PROPOSE", "ROLLBACK"],
    "PROPOSE":  ["MODIFY", "PROPOSE", "ROLLBACK"],
    "MODIFY":   ["VALIDATE", "ROLLBACK"],
    "VALIDATE": ["DIFF", "COMMIT", "PROPOSE", "ROLLBACK"],
    "DIFF":     ["COMMIT", "PROPOSE", "ROLLBACK"],
    "COMMIT":   ["REPLAY", "VERIFY"],
    "ROLLBACK": [],
    "REPLAY":   ["VERIFY"],
    "VERIFY":   [],
}

MUTATING_OPS = {"replace-text"}


def sha256_file(p: Path) -> str:
    h = hashlib.sha256()
    with p.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_obj(obj) -> str:
    return hashlib.sha256(
        json.dumps(obj, ensure_ascii=False, sort_keys=True).encode("utf-8")).hexdigest()


def envelope(request_id, operation, status, code, record=None, tx=None,
             untrusted_fields=None, retryable=False, receipt=None):
    env = {
        "protocol": "DAP/1.0",
        "request_id": request_id,
        "operation": operation,
        "status": status,
        "code": code,
        "retryable": retryable,
    }
    if tx:
        env["transaction_id"] = tx
    if record is not None:
        env["record"] = record
    env["untrustedContent"] = bool(untrusted_fields)
    env["untrustedFields"] = untrusted_fields or []
    if receipt:
        env["receipt"] = receipt
    return env


def emit(env) -> int:
    print(json.dumps(env, ensure_ascii=False, indent=1))
    return env["code"] // 1000 if env["code"] else 0


def rhwp(bin_path: str, args: list[str], timeout: float = 60.0):
    try:
        return subprocess.run([bin_path] + args, capture_output=True, text=True,
                              encoding="utf-8", errors="replace", timeout=timeout)
    except subprocess.TimeoutExpired:
        return None


class Tx:
    def __init__(self, path: Path):
        self.path = path
        self.state = json.loads((path / "state.json").read_text(encoding="utf-8"))

    @staticmethod
    def create(tx_dir: Path, doc: Path, bin_path: str, agent: str) -> "Tx":
        tx_id = "tx_" + uuid.uuid4().hex[:16]
        p = tx_dir / tx_id
        p.mkdir(parents=True)
        state = {
            "transactionId": tx_id,
            "parentTransactionId": None,
            "input": str(doc.resolve()),
            "inputSha256": sha256_file(doc),
            "format": doc.suffix.lstrip(".").lower(),
            "bin": bin_path,
            "agentIdentity": agent,
            "toolVersion": f"{TOOL_NAME}/{DRIVER_VERSION}",
            "current": "BEGIN",
            "history": ["BEGIN"],
            "proposal": None,
            "workingOutput": None,
            "validated": False,
            "committed": False,
        }
        (p / "state.json").write_text(json.dumps(state, ensure_ascii=False, indent=1),
                                      encoding="utf-8")
        return Tx(p)

    def save(self) -> None:
        (self.path / "state.json").write_text(
            json.dumps(self.state, ensure_ascii=False, indent=1), encoding="utf-8")

    def can(self, op: str) -> bool:
        return op in TRANSITIONS.get(self.state["current"], [])

    def advance(self, op: str) -> None:
        self.state["current"] = op
        self.state["history"].append(op)

    def input_unchanged(self) -> bool:
        """불변식 3 — MODIFY 는 원본을 바꾸지 않는다. 매 연산마다 확인한다."""
        p = Path(self.state["input"])
        return p.is_file() and sha256_file(p) == self.state["inputSha256"]


def load_tx(tx_dir: Path, tx_id: str) -> Tx | None:
    p = tx_dir / tx_id
    return Tx(p) if (p / "state.json").is_file() else None


def guard(tx: Tx, op: str, rid: str):
    """상태기계·원본 무훼손을 강제한다. 통과면 None, 아니면 봉투."""
    if tx.state["committed"] and op not in ("REPLAY", "VERIFY"):
        return envelope(rid, f"transaction.{op.lower()}", "error", 2000, tx=tx.state["transactionId"],
                        record={"reason": "이미 확정된 트랜잭션이다 — 영수증은 불변이다"})
    if not tx.can(op):
        return envelope(rid, f"transaction.{op.lower()}", "error", 2000,
                        tx=tx.state["transactionId"],
                        record={"reason": "상태기계 위반",
                                "from": tx.state["current"], "attempted": op,
                                "allowed": TRANSITIONS.get(tx.state["current"], [])})
    if not tx.input_unchanged():
        return envelope(rid, f"transaction.{op.lower()}", "error", 4001,
                        tx=tx.state["transactionId"],
                        record={"reason": "입력 문서가 트랜잭션 도중 바뀌었다 — 한 트랜잭션은 한 입력 해시다"})
    return None


def validate_proposal(op: str, params: object) -> str | None:
    """MODIFY 전에 제안의 타입·필수 키를 확정해 런타임 예외를 봉투 오류로 바꾼다."""
    if not isinstance(params, dict):
        return "params 는 객체여야 한다"
    if op == "replace-text":
        for key in ("find", "replace"):
            if not isinstance(params.get(key), str):
                return f"replace-text params.{key} 는 문자열이어야 한다"
    return None


# --- 연산 -------------------------------------------------------------------

def op_begin(a) -> int:
    rid = a.request_id
    doc = Path(a.doc)
    if not doc.is_file():
        return emit(envelope(rid, "transaction.begin", "error", 2001,
                             record={"reason": f"문서가 없다: {doc}"}))
    bin_path = a.bin or shutil.which("rhwp")
    if not bin_path or not (Path(bin_path).is_file() or shutil.which(bin_path)):
        return emit(envelope(rid, "transaction.begin", "error", 2000,
                             record={"reason": "rhwp 바이너리를 찾을 수 없다"}))
    tx = Tx.create(Path(a.tx_dir), doc, bin_path, a.agent)
    return emit(envelope(rid, "transaction.begin", "ok", 0, tx=tx.state["transactionId"],
                         record={"transactionId": tx.state["transactionId"],
                                 "inputSha256": tx.state["inputSha256"],
                                 "format": tx.state["format"],
                                 "state": "BEGIN"}))


def op_read(a, tx: Tx) -> int:
    rid = a.request_id
    p = rhwp(tx.state["bin"], ["info", tx.state["input"], "--json"])
    if not p or p.returncode != 0:
        return emit(envelope(rid, "document.read", "error", 1000, tx=tx.state["transactionId"],
                             record={"exit": p.returncode if p else None}, retryable=True))
    tx.advance("READ"); tx.save()
    return emit(envelope(rid, "document.read", "ok", 0, tx=tx.state["transactionId"],
                         record=json.loads(p.stdout)))


def op_select(a, tx: Tx) -> int:
    rid = a.request_id
    p = rhwp(tx.state["bin"], ["search", tx.state["input"], "--json", "--", a.query])
    if not p or p.returncode != 0:
        return emit(envelope(rid, "document.select", "error", 1000, tx=tx.state["transactionId"],
                             record={"exit": p.returncode if p else None}, retryable=True))
    env = json.loads(p.stdout)
    matches = env.get("matches", [])
    tx.advance("SELECT"); tx.save()
    if not matches:
        # 0건은 오류가 아니라 판정이다.
        return emit(envelope(rid, "document.select", "verdict", 3002, tx=tx.state["transactionId"],
                             record={"query": a.query, "matchCount": 0}))
    if a.expect_one and len(matches) > 1:
        return emit(envelope(rid, "document.select", "verdict", 3003, tx=tx.state["transactionId"],
                             record={"query": a.query, "matchCount": len(matches)}))
    return emit(envelope(rid, "document.select", "ok", 0, tx=tx.state["transactionId"],
                         record={"query": a.query, "matchCount": len(matches),
                                 "matches": matches[:20]},
                         untrusted_fields=["record.matches[].text", "record.matches[].context"]))


def op_propose(a, tx: Tx) -> int:
    rid = a.request_id
    if a.op not in MUTATING_OPS:
        return emit(envelope(rid, "transaction.propose", "error", 2004,
                             tx=tx.state["transactionId"],
                             record={"reason": f"이 드라이버가 모르는 연산: {a.op}",
                                     "known": sorted(MUTATING_OPS)}))
    try:
        params = json.loads(a.params)
    except json.JSONDecodeError as e:
        return emit(envelope(rid, "transaction.propose", "error", 2000,
                             tx=tx.state["transactionId"], record={"reason": f"params JSON 오류: {e}"}))
    invalid = validate_proposal(a.op, params)
    if invalid:
        return emit(envelope(rid, "transaction.propose", "error", 2000,
                             tx=tx.state["transactionId"], record={"reason": invalid}))
    proposal = {"op": a.op, "params": params}
    tx.state["proposal"] = proposal
    tx.state["operationSha256"] = sha256_obj(proposal)
    tx.state["validated"] = False          # 새 제안은 이전 검증을 무효화한다
    tx.advance("PROPOSE"); tx.save()
    return emit(envelope(rid, "transaction.propose", "ok", 0, tx=tx.state["transactionId"],
                         record={"proposal": proposal,
                                 "operationSha256": tx.state["operationSha256"]}))


def op_modify(a, tx: Tx) -> int:
    rid = a.request_id
    prop = tx.state["proposal"]
    out = tx.path / ("working." + tx.state["format"])
    params = prop["params"]
    if prop["op"] == "replace-text":
        args = ["edit", "replace-text", tx.state["input"],
                "--find", params["find"], "--replace", params["replace"],
                "-o", str(out), "--json"]
    else:  # pragma: no cover — MUTATING_OPS 가 이미 막는다
        return emit(envelope(rid, "document.modify", "error", 2004, tx=tx.state["transactionId"]))

    p = rhwp(tx.state["bin"], args)
    if not p or p.returncode != 0:
        out.unlink(missing_ok=True)
        return emit(envelope(rid, "document.modify", "error",
                             1000 if (p and p.returncode == 1) else 2000,
                             tx=tx.state["transactionId"],
                             record={"exit": p.returncode if p else None,
                                     "stderrHead": (p.stderr or "").strip().splitlines()[:3] if p else []}))
    tool_env = json.loads(p.stdout) if p.stdout.strip().startswith("{") else {}
    if not out.is_file():
        # rhwp 는 매치 0건이면 exit 0 이면서 산출을 만들지 않는다 — 런타임 실패가
        # 아니라 "아무것도 고르지 못했다"는 판정이다(DAP 3002).
        if tool_env.get("replacedCount") == 0:
            return emit(envelope(rid, "document.modify", "verdict", 3002,
                                 tx=tx.state["transactionId"],
                                 record={"reason": "제안이 아무것도 고르지 못했다(치환 0건)",
                                         "toolEnvelope": tool_env}))
        return emit(envelope(rid, "document.modify", "error", 1000, tx=tx.state["transactionId"],
                             record={"reason": "산출이 만들어지지 않았다"}, retryable=True))
    tx.state["workingOutput"] = str(out)
    tx.state["modifyEnvelope"] = tool_env or None
    tx.advance("MODIFY"); tx.save()
    return emit(envelope(rid, "document.modify", "ok", 0, tx=tx.state["transactionId"],
                         record={"workingOutput": out.name,
                                 "outputSha256": sha256_file(out),
                                 "originalUntouched": tx.input_unchanged(),
                                 "toolEnvelope": tx.state["modifyEnvelope"]}))


def op_validate(a, tx: Tx) -> int:
    """결정적 사후조건. 하나라도 깨지면 3001 이고, COMMIT 경로가 닫힌다."""
    rid = a.request_id
    out = Path(tx.state["workingOutput"])
    failures = []

    # MODIFY 의 성공 보고를 믿지 않는다 — 산출물을 직접 다시 연다.
    info = rhwp(tx.state["bin"], ["info", str(out), "--json"])
    if not info or info.returncode != 0:
        failures.append("산출이 열리지 않는다(info 실패)")

    # 연산별 사후조건: replace 가 find 를 포함하지 않으면 산출에 find 가 남아선 안 된다.
    prop = tx.state.get("proposal") or {}
    if prop.get("op") == "replace-text":
        find, repl = prop["params"]["find"], prop["params"]["replace"]
        if find and find not in repl:
            s = rhwp(tx.state["bin"], ["search", str(out), "--json", "--", find])
            if s and s.stdout.strip().startswith("{"):
                left = json.loads(s.stdout).get("matchCount", 0)
                if left:
                    failures.append(f"치환이 불완전하다 — 산출에 '{find}' 가 {left}건 남았다")

    # 변경 연산인데 아무것도 안 바뀌었으면 그 제안은 실패다.
    d = rhwp(tx.state["bin"], ["ir-diff", tx.state["input"], str(out), "--json"])
    identical = None
    if d and d.stdout.strip().startswith("{"):
        identical = json.loads(d.stdout).get("identical")
        if identical is True:
            failures.append("변경 연산인데 산출이 입력과 동일하다 — 제안이 아무 효과도 내지 못했다")

    if not tx.input_unchanged():
        failures.append("원본이 훼손됐다")

    tx.state["validated"] = not failures
    tx.advance("VALIDATE"); tx.save()

    if failures:
        return emit(envelope(rid, "document.validate", "verdict", 3001,
                             tx=tx.state["transactionId"],
                             record={"passed": False, "failures": failures,
                                     "commitBlocked": True}))
    return emit(envelope(rid, "document.validate", "ok", 0, tx=tx.state["transactionId"],
                         record={"passed": True, "identical": identical}))


def op_diff(a, tx: Tx) -> int:
    rid = a.request_id
    out = Path(tx.state["workingOutput"])
    d = rhwp(tx.state["bin"], ["ir-diff", tx.state["input"], str(out), "--json"])
    if not d or not d.stdout.strip().startswith("{"):
        return emit(envelope(rid, "document.diff", "error", 1000, tx=tx.state["transactionId"],
                             retryable=True))
    tx.advance("DIFF"); tx.save()
    return emit(envelope(rid, "document.diff", "ok", 0, tx=tx.state["transactionId"],
                         record=json.loads(d.stdout)))


def op_commit(a, tx: Tx) -> int:
    rid = a.request_id
    # 불변식 1·2 — 이것이 이 드라이버의 존재 이유다.
    if not tx.state["validated"]:
        return emit(envelope(rid, "transaction.commit", "verdict", 3001,
                             tx=tx.state["transactionId"],
                             record={"reason": "VALIDATE 성공 없이 COMMIT 할 수 없다",
                                     "state": tx.state["current"],
                                     "validated": False}))
    out = Path(tx.state["workingOutput"])
    final = Path(a.output)
    if final.resolve() == Path(tx.state["input"]).resolve():
        return emit(envelope(rid, "transaction.commit", "error", 2000,
                             tx=tx.state["transactionId"],
                             record={"reason": "산출 경로가 원본 입력과 같다 — 원본을 덮어쓸 수 없다"}))
    if final.exists() and not a.overwrite:
        return emit(envelope(rid, "transaction.commit", "error", 2000,
                             tx=tx.state["transactionId"],
                             record={"reason": f"산출 경로가 이미 있다: {final} (--overwrite 필요)"}))
    final.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(out, final)

    import datetime
    receipt = {
        "protocol": "DATP/1.0",
        "transactionId": tx.state["transactionId"],
        "parentTransactionId": tx.state["parentTransactionId"],
        "inputSha256": tx.state["inputSha256"],
        "operationSha256": tx.state["operationSha256"],
        "outputSha256": sha256_file(final),
        "policySha256": tx.state.get("policySha256"),
        "toolVersion": tx.state["toolVersion"],
        "agentIdentity": tx.state["agentIdentity"],
        "timestamp": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "proposal": tx.state["proposal"],
        "input": tx.state["input"],
        "output": str(final.resolve()),
        "history": tx.state["history"] + ["COMMIT"],
    }
    (tx.path / "receipt.json").write_text(
        json.dumps(receipt, ensure_ascii=False, indent=1), encoding="utf-8")
    tx.state["committed"] = True
    tx.advance("COMMIT"); tx.save()
    out.unlink(missing_ok=True)
    return emit(envelope(rid, "transaction.commit", "ok", 0, tx=tx.state["transactionId"],
                         record={"output": str(final), "receipt": str(tx.path / "receipt.json")},
                         receipt=receipt))


def op_rollback(a, tx: Tx) -> int:
    rid = a.request_id
    wo = tx.state.get("workingOutput")
    removed = False
    if wo and Path(wo).is_file():
        Path(wo).unlink()
        removed = True
    intact = tx.input_unchanged()
    tx.state["workingOutput"] = None
    tx.state["validated"] = False
    tx.advance("ROLLBACK"); tx.save()
    return emit(envelope(rid, "transaction.rollback", "ok", 0, tx=tx.state["transactionId"],
                         record={"discardedWorkingOutput": removed,
                                 "originalIntact": intact,
                                 "inputSha256": tx.state["inputSha256"]}))


def load_receipt(path: Path, rid: str, tx_id: str | None = None):
    if not path.is_file():
        return None, envelope(rid, "transaction.replay", "error", 2001,
                              record={"reason": f"영수증이 없다: {path}"})
    try:
        receipt = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return None, envelope(rid, "transaction.replay", "error", 2000,
                              record={"reason": f"영수증 JSON 오류: {exc}"})
    required = {"transactionId", "input", "inputSha256", "output", "outputSha256", "proposal"}
    missing = sorted(required - set(receipt)) if isinstance(receipt, dict) else []
    if not isinstance(receipt, dict) or missing:
        return None, envelope(rid, "transaction.replay", "error", 2000,
                              record={"reason": f"영수증 필수 필드 없음: {missing if isinstance(receipt, dict) else '객체 아님'}"})
    if tx_id is not None and receipt["transactionId"] != tx_id:
        return None, envelope(rid, "transaction.replay", "error", 2000,
                              tx=tx_id, record={"reason": "영수증 transactionId 가 현재 트랜잭션과 다르다"})
    proposal = receipt["proposal"]
    if not isinstance(proposal, dict):
        return None, envelope(rid, "transaction.replay", "error", 2000,
                              record={"reason": "영수증 proposal 은 객체여야 한다"})
    op = proposal.get("op")
    if not isinstance(op, str) or op not in MUTATING_OPS:
        return None, envelope(rid, "transaction.replay", "error", 2000,
                              record={"reason": f"영수증 proposal.op 가 지원되지 않는다: {op!r}"})
    invalid = validate_proposal(op, proposal.get("params"))
    if invalid:
        return None, envelope(rid, "transaction.replay", "error", 2000,
                              record={"reason": f"영수증 proposal 이 유효하지 않다: {invalid}"})
    return receipt, None


def op_replay(a, tx: Tx | None = None) -> int:
    """영수증만으로 재실행해 같은 산출 해시가 나오는지 판정한다."""
    rid = a.request_id
    rp = Path(a.receipt) if getattr(a, "receipt", None) else tx.path / "receipt.json"
    r, error = load_receipt(rp, rid, tx.state["transactionId"] if tx else None)
    if error:
        return emit(error)
    bin_path = getattr(a, "bin", None) or (tx.state["bin"] if tx else None) or shutil.which("rhwp")
    if not bin_path:
        return emit(envelope(rid, "transaction.replay", "error", 2000,
                             record={"reason": "rhwp 바이너리를 찾을 수 없다"}))
    src = Path(r["input"])
    if not src.is_file():
        return emit(envelope(rid, "transaction.replay", "error", 2001,
                             record={"reason": f"입력 문서가 없다: {src}"}))
    if sha256_file(src) != r["inputSha256"]:
        return emit(envelope(rid, "transaction.replay", "verdict", 3000,
                             record={"reason": "입력 해시가 영수증과 다르다",
                                     "expected": r["inputSha256"], "observed": sha256_file(src)}))

    import tempfile
    with tempfile.TemporaryDirectory(prefix="datp_replay_") as td:
        out = Path(td) / ("replayed" + src.suffix)
        params = r["proposal"]["params"]
        p = rhwp(bin_path, ["edit", "replace-text", str(src),
                            "--find", params["find"], "--replace", params["replace"],
                            "-o", str(out), "--json"])
        if not p or p.returncode != 0 or not out.is_file():
            return emit(envelope(rid, "transaction.replay", "error", 1000,
                                 tx=tx.state["transactionId"] if tx else r["transactionId"],
                                 record={"exit": p.returncode if p else None}, retryable=True))
        observed = sha256_file(out)

    reproduced = observed == r["outputSha256"]
    env = envelope(rid, "transaction.replay",
                   "ok" if reproduced else "verdict",
                   0 if reproduced else 3000,
                   tx=r["transactionId"],
                   record={"reproduced": reproduced,
                           "expectedOutputSha256": r["outputSha256"],
                           "observedOutputSha256": observed,
                           "operationSha256": r.get("operationSha256")})
    if tx:
        tx.advance("REPLAY")
        tx.save()
    return emit(env)


def op_verify(a, tx: Tx) -> int:
    """확정 영수증과 실제 입출력의 해시를 대조하고 VERIFY 상태를 남긴다."""
    rid = a.request_id
    receipt, error = load_receipt(tx.path / "receipt.json", rid, tx.state["transactionId"])
    if error:
        error["operation"] = "transaction.verify"
        error["transaction_id"] = tx.state["transactionId"]
        return emit(error)
    failures = []
    for field in ("input", "output"):
        path = Path(receipt[field])
        digest_field = f"{field}Sha256"
        if not path.is_file():
            failures.append(f"{field} 파일이 없다: {path}")
        elif sha256_file(path) != receipt[digest_field]:
            failures.append(f"{field} 해시가 영수증과 다르다")
    tx.advance("VERIFY")
    tx.save()
    return emit(envelope(rid, "transaction.verify", "ok" if not failures else "verdict",
                         0 if not failures else 3000, tx=tx.state["transactionId"],
                         record={"verified": not failures, "failures": failures,
                                 "receipt": str(tx.path / "receipt.json")}))


def main(argv=None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--request-id", default=None, help="재시도 시 같은 값을 쓴다(멱등 키)")
    ap.add_argument("--tx-dir", default="txs", help="트랜잭션 상태 폴더")
    sub = ap.add_subparsers(dest="cmd", required=True)

    b = sub.add_parser("begin"); b.add_argument("--doc", required=True)
    b.add_argument("--bin"); b.add_argument("--agent", default="unknown-agent")
    r = sub.add_parser("read"); r.add_argument("--tx", required=True)
    s = sub.add_parser("select"); s.add_argument("--tx", required=True)
    s.add_argument("--query", required=True); s.add_argument("--expect-one", action="store_true")
    pr = sub.add_parser("propose"); pr.add_argument("--tx", required=True)
    pr.add_argument("--op", required=True); pr.add_argument("--params", required=True)
    for name in ("modify", "validate", "diff", "rollback"):
        x = sub.add_parser(name); x.add_argument("--tx", required=True)
    c = sub.add_parser("commit"); c.add_argument("--tx", required=True)
    c.add_argument("-o", "--output", required=True); c.add_argument("--overwrite", action="store_true")
    rp = sub.add_parser("replay")
    replay_input = rp.add_mutually_exclusive_group(required=True)
    replay_input.add_argument("--receipt")
    replay_input.add_argument("--tx")
    rp.add_argument("--bin")
    v = sub.add_parser("verify"); v.add_argument("--tx", required=True)

    a = ap.parse_args(argv)
    if not a.request_id:
        a.request_id = "req_" + uuid.uuid4().hex[:12]

    if a.cmd == "begin":
        return op_begin(a)
    if a.cmd == "replay" and a.receipt:
        return op_replay(a)

    tx = load_tx(Path(a.tx_dir), a.tx)
    if tx is None:
        return emit(envelope(a.request_id, f"transaction.{a.cmd}", "error", 2000,
                             record={"reason": f"트랜잭션을 찾을 수 없다: {a.tx}"}))
    op = {"read": "READ", "select": "SELECT", "propose": "PROPOSE", "modify": "MODIFY",
          "validate": "VALIDATE", "diff": "DIFF", "commit": "COMMIT",
          "rollback": "ROLLBACK", "replay": "REPLAY", "verify": "VERIFY"}[a.cmd]
    blocked = guard(tx, op, a.request_id)
    if blocked:
        return emit(blocked)
    return {"read": op_read, "select": op_select, "propose": op_propose, "modify": op_modify,
            "validate": op_validate, "diff": op_diff, "commit": op_commit,
            "rollback": op_rollback, "replay": op_replay, "verify": op_verify}[a.cmd](a, tx)


if __name__ == "__main__":
    raise SystemExit(main())
