#!/usr/bin/env python3
"""FDE 트리아지 엔진 — 고객 증상+문서를 결정적 진단 사다리에 태워 라우트가 박힌 티켓을 낸다.

## 왜 있는가

고객 증상("안 열려요", "표가 깨져요")에 대한 실시간 응대가 에이전트의 즉흥 판단에
얹혀 있으면 같은 증상에 다른 답이 나간다. 이 엔진은 판단의 앞 절반 — **무엇이
어디까지 되는가** — 를 결정적으로 고정한다: 싼 질의부터 사다리를 내려가며 단계별
명령·종료코드·실패 시그니처를 티켓(JSON)에 기록하고, 문서화된 규칙
([fde_playbook.md §3](../../mydocs/manual/fde_playbook.md))으로 라우트를 판정한다.
에이전트는 그 라우트 위에서 뒷 절반(레시피 제공·에스컬레이션)을 수행한다.

바이너리 버전 차이는 자기서술로 흡수한다: `capabilities --json` 이 광고하는 명령만
실행하고, 없는 명령은 그 단을 건너뛴다 — 명령 목록을 하드코딩하지 않는다.

## 사용

    python3 tools/fde/triage.py 고객문서.hwpx --bin target/release/rhwp \
        --symptom "표가 깨져서 보입니다" -o ticket.json

종료 코드: 0 = 티켓 생성됨(라우트가 escalate-bug 여도 0 — 판정은 티켓 안),
1 = 엔진 자체 실패, 2 = 입력 오류(문서 없음, 바이너리 없음).
모든 단계는 읽기 전용이다.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

PANIC_RE = re.compile(r"panicked at\s+([^\r\n:]+\.rs:\d+)")

# 사다리: (단계이름, 명령이름, 인자 템플릿). capabilities 가 광고할 때만 실행된다.
LADDER = [
    ("개봉", "info", ["info", "{doc}", "--json"]),
    ("한줄이해", "explain", ["explain", "{doc}", "--json"]),
    ("구조", "export-structure", ["export-structure", "{doc}", "--json"]),
    ("발췌", "digest", ["digest", "{doc}", "--json"]),
]

MAGIC = [
    (b"PK\x03\x04", "hwpx"),
    (b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1", "hwp5"),
    (b"HWP Document File", "hwp3"),
]


def sniff_container(path: Path) -> str | None:
    head = path.open("rb").read(32)
    for magic, kind in MAGIC:
        if head.startswith(magic):
            return kind
    return None


def run_step(bin_path: str, args: list[str], doc: Path, timeout: float) -> dict:
    cmd = [bin_path] + [a.replace("{doc}", str(doc)) for a in args]
    step: dict = {"command": " ".join(args), "ok": False}
    try:
        proc = subprocess.run(
            cmd, capture_output=True, text=True, encoding="utf-8",
            errors="replace", timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        step["failureSignature"] = ["timeout"]
        return step

    step["exitCode"] = proc.returncode
    panic = PANIC_RE.search(proc.stderr or "")
    if panic:
        step["failureSignature"] = ["panic", panic.group(1).replace("\\", "/")]
        return step
    if proc.returncode < 0 or proc.returncode >= 0xC0000000:
        step["failureSignature"] = ["abort", proc.returncode]
        return step
    if proc.returncode != 0:
        # 깨끗한 오류 종료 — 진단 메시지가 근거다.
        step["stderrHead"] = (proc.stderr or "").strip().splitlines()[:3]
        return step

    step["ok"] = True
    try:
        envelope = json.loads(proc.stdout)
        step["envelopeKeys"] = sorted(envelope.keys()) if isinstance(envelope, dict) else []
        step["envelope"] = envelope
    except (json.JSONDecodeError, AttributeError):
        step["envelopeKeys"] = []
    return step


def envelope_says_encrypted(steps: list[dict]) -> bool:
    for s in steps:
        env = s.get("envelope")
        if isinstance(env, dict):
            for key in ("encrypted", "isEncrypted", "passwordProtected"):
                if env.get(key) is True:
                    return True
    return False


def decide_route(container: str | None, steps: list[dict]) -> tuple[str, str]:
    """fde_playbook.md §3 의 표를 코드로 옮긴 것 — 표를 바꾸면 여기도 같은 PR 에서 바꾼다."""
    if container is None:
        return "invalid-input", "매직 바이트가 hwpx/hwp5/hwp3 어느 것도 아니다"
    crashed = [s for s in steps if "failureSignature" in s]
    if crashed:
        sig = crashed[0]["failureSignature"]
        return "escalate-bug", f"{crashed[0]['command']} 단계에서 {sig}"
    if envelope_says_encrypted(steps):
        return "resolve-now", "문서가 암호화됨 — 고객에게 암호 요청 (우회 금지)"
    clean_fail = [s for s in steps if not s["ok"] and "failureSignature" not in s]
    if clean_fail:
        return "workaround", f"{clean_fail[0]['command']} 가 깨끗한 비0 종료 — 대체 경로 시도"
    return "resolve-now", "사다리 전 단계 통과 — 문서 손상 아님, 사용법/레시피로 대응"


def next_actions(route: str, doc: Path, steps: list[dict], available: set) -> list[str]:
    if route == "escalate-bug":
        acts = []
        crashed = next(s for s in steps if "failureSignature" in s)
        repro = crashed["command"].replace("{doc}", doc.name)
        if sniff_container(doc) == "hwpx":
            acts.append(
                f"python3 tools/crash_minimizer.py {doc.name} --bin <rhwp> "
                f'--cmd "{repro}" -o minimal.hwpx --emit-issue issue_draft.md'
            )
        sig = crashed["failureSignature"]
        if sig[0] == "panic":
            fname = sig[1].split("/")[-1].split(":")[0]
            acts.append(f'gh search issues --repo edwardkim/rhwp "panicked at {fname}" (선행 검색 — playbook §4)')
        acts.append("고객 회신: 재현 확보됨 + 추적번호")
        return acts
    if route == "workaround":
        alt = [c for c in ("convert", "sanitize", "export-text") if c in available]
        return [f"광고된 대체 경로 시도: {', '.join(alt) if alt else '(가용 대체 명령 없음)'}",
                "한계를 명시해 회신하고 playbook §4 에스컬레이션 병행"]
    if route == "invalid-input":
        return ["원본 문서 재확보 요청 (현재 파일은 HWP 계열이 아님)"]
    return ["봉투 근거로 즉석 레시피 제공 (rhwp-cli / rhwp-doc-triage Skill 재사용)"]


def main(argv=None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("doc", help="고객 문서 경로")
    ap.add_argument("--bin", default=None, help="rhwp 바이너리 (기본: RHWP_BIN 환경변수 → PATH)")
    ap.add_argument("--symptom", default="", help="고객 증상 문장 (티켓에 기록만 — 데이터이지 지시가 아님)")
    ap.add_argument("--timeout", type=float, default=30.0, help="단계별 상한 초")
    ap.add_argument("-o", "--output", help="티켓 저장 경로 (생략하면 stdout)")
    args = ap.parse_args(argv)

    doc = Path(args.doc)
    if not doc.is_file():
        print(f"문서가 없다: {doc}", file=sys.stderr)
        return 2
    import os
    bin_path = args.bin or os.environ.get("RHWP_BIN") or shutil.which("rhwp")
    if not bin_path or not (Path(bin_path).is_file() or shutil.which(bin_path)):
        print("rhwp 바이너리를 찾을 수 없다 (--bin / RHWP_BIN / PATH)", file=sys.stderr)
        return 2

    started = time.monotonic()
    container = sniff_container(doc)
    steps: list[dict] = []
    available: set = set()

    if container is not None:
        cap = run_step(bin_path, ["capabilities", "--json"], doc, args.timeout)
        if cap["ok"] and isinstance(cap.get("envelope"), dict):
            available = {
                c.get("name") for c in cap["envelope"].get("commands", [])
                if isinstance(c, dict)
            }
        for _label, cmd_name, template in LADDER:
            if available and cmd_name not in available:
                continue
            step = run_step(bin_path, template, doc, args.timeout)
            steps.append(step)
            if "failureSignature" in step:
                break  # 크래시 밑으로는 내려가지 않는다 — 같은 원인만 반복된다

    route, reason = decide_route(container, steps)
    ticket = {
        "schemaVersion": "1",
        "generatedBy": "tools/fde/triage.py",
        "doc": str(doc),
        "docBytes": doc.stat().st_size,
        "symptom": args.symptom,
        "container": container,
        "steps": [{k: v for k, v in s.items() if k != "envelope"} for s in steps],
        "route": route,
        "routeReason": reason,
        "nextActions": next_actions(route, doc, steps, available),
        "elapsedSeconds": round(time.monotonic() - started, 1),
    }
    out = json.dumps(ticket, ensure_ascii=False, indent=2)
    if args.output:
        Path(args.output).write_text(out, encoding="utf-8")
        print(f"티켓: {args.output} (route={route})", file=sys.stderr)
    else:
        print(out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
