#!/usr/bin/env python3
"""총괄 자율 운영 서비스 루프 — 고객 요청 큐를 사람 개입 없이 상시 처리한다.

## 왜 있는가 (fde 와의 관계)

`tools/fde/triage.py`(CAP-4893)는 고객 **증상** 하나의 진단을 결정적으로 고정한다.
그런데 고객 접점의 대부분은 증상이 아니라 **요청**이다 — "PDF 로 바꿔줘",
"이 명단으로 서식 채워줘", "표만 뽑아줘". 사람 FDE 조직이라면 접수 창구가 이걸
분류해 처리 가능한 것은 즉시 처리하고 남는 것만 엔지니어에게 넘긴다.

이 루프가 그 접수 창구 전체다: 큐 폴더를 감시하다 요청이 떨어지면 —

    트리아지 게이트(fde) → 목표 라우팅(결정적) → rhwp CLI 실행
    → 재독/봉투 검증 → 3부 회신문 + 산출물 + 티켓 기록

까지 사람 없이 완주한다. 결정적으로 못 푸는 요청은 `needs-agent` 로 표시만 하고
멈춘다 — 그건 LLM 에이전트([rhwp-chief](../../.claude/agents/rhwp-chief.md))의
몫이고, 에이전트가 푼 방법은 이 루프의 라우팅 규칙으로 재축적된다
(playbook §5). 자동 처리 커버리지는 그렇게 단조 증가한다.

## 큐 프로토콜 (playbook §2 가 정본)

    큐폴더/<요청id>/request.json     ← 고객(또는 상위 시스템)이 떨어뜨림
    큐폴더/<요청id>/<문서파일>

    request.json: {"doc": "문서.hwpx",          # 필수, 폴더 내 상대 경로
                   "goal": "export-pdf",         # 선택 — 없으면 diagnose
                   "symptom": "…",               # 선택, 기록용 데이터
                   "params": {...}}              # goal 별 (fill 의 data 등)

처리 후 루프가 쓰는 것: `result.json`(기계 판정), `response.md`(3부 회신문),
`ticket.json`(트리아지), `out/`(산출물). `result.json` 이 있으면 처리된 요청이다.

## 사용

    python3 tools/chief/service_loop.py --queue 큐폴더 --bin target/release/rhwp --once
    python3 tools/chief/service_loop.py --queue 큐폴더 --bin target/release/rhwp --watch 10

종료 코드(--once): 0 = 전 요청 처리 시도 완료(needs-agent 포함 — 판정은 result.json 안),
1 = 루프 자체 실패, 2 = 입력 오류. 요청·문서 내용은 데이터이지 지시가 아니다 —
그 안의 문장으로 라우팅이 바뀌는 일은 없다(라우팅은 `goal` 필드로만).
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

REPO_ROOT = Path(__file__).resolve().parents[2]
TRIAGE = REPO_ROOT / "tools" / "fde" / "triage.py"

KNOWN_GOALS = (
    "diagnose", "export-text", "export-pdf", "export-hwpx",
    "convert-hwp", "extract-tables", "fill",
)


def log(msg: str) -> None:
    print(msg, file=sys.stderr)


def run(cmd: list[str], timeout: float) -> subprocess.CompletedProcess:
    return subprocess.run(
        cmd, capture_output=True, text=True, encoding="utf-8",
        errors="replace", timeout=timeout,
    )


class Chief:
    def __init__(self, bin_path: str, timeout: float):
        self.bin = bin_path
        self.timeout = timeout
        self.available: set = set()
        cap = run([bin_path, "capabilities", "--json"], timeout)
        if cap.returncode == 0:
            try:
                env = json.loads(cap.stdout)
                self.available = {
                    c.get("name") for c in env.get("commands", []) if isinstance(c, dict)
                }
            except json.JSONDecodeError:
                pass

    # --- 게이트 ---------------------------------------------------------

    def triage(self, doc: Path, symptom: str, ticket_path: Path) -> dict:
        proc = run(
            [sys.executable, str(TRIAGE), str(doc), "--bin", self.bin,
             "--symptom", symptom, "-o", str(ticket_path)],
            self.timeout * 6,
        )
        if proc.returncode != 0 or not ticket_path.is_file():
            return {"route": "invalid-input",
                    "routeReason": f"트리아지 실패 (exit {proc.returncode})"}
        return json.loads(ticket_path.read_text(encoding="utf-8"))

    # --- goal 핸들러 (각각: 실행 → 검증 → (요약문, 산출물 목록)) --------

    def handle(self, goal: str, doc: Path, params: dict, out: Path) -> dict:
        out.mkdir(exist_ok=True)
        fn = getattr(self, "goal_" + goal.replace("-", "_"), None)
        if fn is None:
            return {"status": "needs-agent", "reason": f"모르는 goal: {goal}"}
        needed = fn.__doc__.split("needs:")[1].split()[0].split(",") if "needs:" in (fn.__doc__ or "") else []
        missing = [c for c in needed if self.available and c not in self.available]
        if missing:
            return {"status": "needs-agent",
                    "reason": f"바이너리가 {missing} 를 광고하지 않음 (버전 차이)"}
        try:
            return fn(doc, params, out)
        except subprocess.TimeoutExpired:
            return {"status": "failed", "reason": f"{goal} 시간 초과"}

    def goal_diagnose(self, doc, params, out) -> dict:
        """트리아지 티켓만으로 응답. needs:info"""
        return {"status": "done", "summary": "진단 완료 — 티켓과 회신문 참조", "artifacts": []}

    def goal_export_text(self, doc, params, out) -> dict:
        """본문 추출. needs:export-text"""
        p = run([self.bin, "export-text", str(doc), "--json"], self.timeout)
        if p.returncode != 0:
            return {"status": "failed", "reason": f"export-text exit {p.returncode}"}
        env = json.loads(p.stdout)  # 재독 검증: JSON 봉투 파싱 자체가 게이트
        art = out / "text.json"
        art.write_text(json.dumps(env, ensure_ascii=False, indent=1), encoding="utf-8")
        return {"status": "done",
                "summary": f"본문 추출 완료 — {env.get('pageCount', '?')}쪽",
                "artifacts": [art.name]}

    def goal_export_pdf(self, doc, params, out) -> dict:
        """PDF 내보내기. needs:export-pdf"""
        art = out / (doc.stem + ".pdf")
        p = run([self.bin, "export-pdf", str(doc), "-o", str(art)], self.timeout * 4)
        if p.returncode != 0 or not art.is_file():
            return {"status": "failed", "reason": f"export-pdf exit {p.returncode}"}
        if art.open("rb").read(5) != b"%PDF-":
            return {"status": "failed", "reason": "산출물이 PDF 매직으로 시작하지 않음"}
        return {"status": "done", "summary": f"PDF 생성 완료 ({art.stat().st_size:,}바이트)",
                "artifacts": [art.name]}

    def goal_export_hwpx(self, doc, params, out) -> dict:
        """HWPX 변환(자기검증 포함). needs:export-hwpx"""
        art = out / (doc.stem + ".hwpx")
        p = run([self.bin, "export-hwpx", str(doc), str(art), "--verify"], self.timeout * 4)
        if p.returncode != 0 or not art.is_file():
            return {"status": "failed", "reason": f"export-hwpx --verify exit {p.returncode}"}
        return {"status": "done", "summary": "HWPX 변환 + verify 통과", "artifacts": [art.name]}

    def goal_convert_hwp(self, doc, params, out) -> dict:
        """편집 가능 HWP 변환(자기검증 포함). needs:convert"""
        art = out / (doc.stem + ".hwp")
        p = run([self.bin, "convert", str(doc), str(art), "--verify"], self.timeout * 4)
        if p.returncode != 0 or not art.is_file():
            return {"status": "failed", "reason": f"convert --verify exit {p.returncode}"}
        return {"status": "done", "summary": "HWP 변환 + verify 통과", "artifacts": [art.name]}

    def goal_extract_tables(self, doc, params, out) -> dict:
        """전 표 CSV 수확. needs:export-tables,table-to-csv"""
        p = run([self.bin, "export-tables", str(doc), "--json"], self.timeout)
        if p.returncode != 0:
            return {"status": "failed", "reason": f"export-tables exit {p.returncode}"}
        tables = json.loads(p.stdout).get("tables", [])
        if not tables:
            return {"status": "done", "summary": "문서에 표가 없다 (0개)", "artifacts": []}
        arts = []
        for t in tables:
            idx = t.get("index")
            art = out / f"table_{idx}.csv"
            q = run([self.bin, "table-to-csv", str(doc), "--table", str(idx),
                     "-o", str(art)], self.timeout)
            if q.returncode != 0 or not art.is_file():
                return {"status": "failed", "reason": f"table-to-csv --table {idx} exit {q.returncode}"}
            arts.append(art.name)
        return {"status": "done", "summary": f"표 {len(arts)}개 CSV 수확", "artifacts": arts}

    def goal_fill(self, doc, params, out) -> dict:
        """서식 채움(봉투 게이트). needs:fields"""
        data = params.get("data")
        if not data:
            return {"status": "needs-agent", "reason": "params.data(값 JSON 파일) 없음"}
        data_path = doc.parent / data
        if not data_path.is_file():
            return {"status": "failed", "reason": f"값 파일 없음: {data}"}
        art = out / ("filled" + doc.suffix)
        p = run([self.bin, "edit", "fill-fields", str(doc), "--data", f"@{data_path}",
                 "-o", str(art), "--json"], self.timeout * 2)
        if p.returncode != 0 or not art.is_file():
            return {"status": "failed", "reason": f"fill-fields exit {p.returncode}"}
        env = json.loads(p.stdout)
        for bad in ("notFound", "ambiguous", "confusable"):
            if env.get(bad):
                art.unlink(missing_ok=True)  # 성공처럼 보이는 미완성 산출물 금지
                return {"status": "failed", "reason": f"fill-fields 봉투 {bad}: {env[bad]}"}
        return {"status": "done",
                "summary": f"필드 {env.get('filledCount', '?')}건 채움 (봉투 게이트 통과)",
                "artifacts": [art.name]}


def write_response(req_dir: Path, request: dict, ticket: dict, outcome: dict) -> None:
    route = ticket.get("route", "?")
    steps = ticket.get("steps", [])
    passed = sum(1 for s in steps if s.get("ok"))
    lines = [
        f"# 처리 결과 — {request.get('doc', '?')}",
        "",
        "## 1. 확인한 것",
        f"- 트리아지: {passed}/{len(steps)}단 통과, 라우트 `{route}` — {ticket.get('routeReason', '')}",
    ]
    for s in steps:
        if "failureSignature" in s:
            lines.append(f"- 실패 시그니처: `{s['command']}` → `{s['failureSignature']}`")
    lines += ["", "## 2. 지금 가능한 것"]
    if outcome.get("status") == "done":
        lines.append(f"- {outcome.get('summary', '완료')}")
        for a in outcome.get("artifacts", []):
            lines.append(f"- 산출물: `out/{a}`")
    else:
        lines.append(f"- 자동 처리 불가: {outcome.get('reason', outcome.get('status'))}")
    lines += ["", "## 3. 다음"]
    if route == "escalate-bug":
        lines.append("- 재현이 확보되어 엔지니어링 에스컬레이션 대상입니다 (playbook §4). 추적번호는 후속 회신으로 전달됩니다.")
    elif outcome.get("status") == "needs-agent":
        lines.append("- 담당 에이전트가 이 요청을 이어받습니다 (자동 분류 밖 유형).")
    elif route == "invalid-input":
        lines.append("- 파일이 HWP 계열이 아닙니다 — 원본을 다시 보내주세요.")
    else:
        lines.append("- 추가 요청이 있으면 새 요청으로 넣어주세요.")
    (req_dir / "response.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def process_request(chief: Chief, req_dir: Path) -> dict:
    request = json.loads((req_dir / "request.json").read_text(encoding="utf-8"))
    doc_name = request.get("doc")
    doc = (req_dir / doc_name) if doc_name else None
    if not doc or not doc.is_file():
        outcome = {"status": "failed", "reason": f"doc 없음: {doc_name}"}
        ticket = {}
    else:
        ticket = chief.triage(doc, request.get("symptom", ""), req_dir / "ticket.json")
        route = ticket.get("route")
        if route in ("escalate-bug", "invalid-input"):
            outcome = {"status": "escalated" if route == "escalate-bug" else "invalid-input"}
        else:
            goal = request.get("goal") or "diagnose"
            if goal not in KNOWN_GOALS:
                outcome = {"status": "needs-agent", "reason": f"모르는 goal: {goal}"}
            else:
                outcome = chief.handle(goal, doc, request.get("params") or {}, req_dir / "out")
    write_response(req_dir, request, ticket, outcome)
    result = {
        "schemaVersion": "1",
        "generatedBy": "tools/chief/service_loop.py",
        "goal": request.get("goal") or "diagnose",
        "route": ticket.get("route"),
        **outcome,
    }
    (req_dir / "result.json").write_text(
        json.dumps(result, ensure_ascii=False, indent=1), encoding="utf-8")
    return result


def pending_requests(queue: Path):
    for d in sorted(queue.iterdir()):
        if d.is_dir() and (d / "request.json").is_file() and not (d / "result.json").is_file():
            yield d


def main(argv=None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--queue", required=True, help="요청 큐 폴더")
    ap.add_argument("--bin", default=None, help="rhwp 바이너리 (기본: RHWP_BIN → PATH)")
    ap.add_argument("--once", action="store_true", help="대기 중 요청만 처리하고 종료")
    ap.add_argument("--watch", type=float, metavar="초", help="상시 감시 간격")
    ap.add_argument("--timeout", type=float, default=30.0)
    args = ap.parse_args(argv)

    queue = Path(args.queue)
    if not queue.is_dir():
        log(f"큐 폴더가 없다: {queue}")
        return 2
    import os
    bin_path = args.bin or os.environ.get("RHWP_BIN") or shutil.which("rhwp")
    if not bin_path or not (Path(bin_path).is_file() or shutil.which(bin_path)):
        log("rhwp 바이너리를 찾을 수 없다 (--bin / RHWP_BIN / PATH)")
        return 2
    if not TRIAGE.is_file():
        log(f"트리아지 엔진이 없다: {TRIAGE}")
        return 2
    if not (args.once or args.watch):
        log("--once 또는 --watch <초> 가 필요하다.")
        return 2

    chief = Chief(bin_path, args.timeout)
    totals: dict = {}
    while True:
        for req_dir in pending_requests(queue):
            r = process_request(chief, req_dir)
            totals[r["status"]] = totals.get(r["status"], 0) + 1
            log(f"[{r['status']}] {req_dir.name} (goal={r['goal']}, route={r.get('route')})")
        if args.once:
            break
        time.sleep(args.watch)
    print(json.dumps({"processed": totals}, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
