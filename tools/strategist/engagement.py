#!/usr/bin/env python3
"""전략 엔게이지먼트 엔진 — 목표+코퍼스에서 근거 대장 기반 산출물 골격까지.

## 왜 있는가 (fde·chief 와의 관계)

`tools/fde/triage.py`(CAP-4893)는 고객 **증상** 하나를, `tools/chief/service_loop.py`
(CAP-4900)는 고객 **요청** 큐를 다룬다. 그 위에 남은 층이 **목표**다 — "정부과제를
수주하고 싶다", "이 사업의 다음 분기 전략 보고서가 필요하다". 사람 전략 컨설턴트의
산출물은 근거 추적이 안 되는 슬라이드로 나오지만, rhwp 의 `search`/`extract-data`
봉투는 쪽·문단·문자 오프셋 좌표를 주므로 이 층을 구조적으로 다르게 만들 수 있다:
**모든 주장이 원문 좌표로 재현 가능**한 산출물.

이 엔진은 전략을 만들지 않는다. 엔진이 보장하는 것은 세 가지뿐이다 —
수집의 전수성(코퍼스 전 문서 지도화), 근거의 좌표(봉투 값 그대로, 조작 없음),
주장-근거 연결의 기계 검증(`--validate`). 전략적 판단(무엇을 주장할지)은
에이전트([rhwp-strategist](../../.claude/agents/rhwp-strategist.md))의 몫이되,
그 주장이 산출물에 실리려면 근거 대장의 실좌표에 연결되어야 한다.

## 엔게이지먼트 프로토콜 (playbook §2 가 정본)

    engagement.json: {"objective": "…",              # 필수, 고객 목표 문장
                      "corpus": "문서폴더",           # 필수, .hwp/.hwpx 재귀 수집
                      "questions": [                  # 필수, 근거를 캘 질문들
                        "문자열" | {"id","text","keywords":[…]}
                      ],
                      "deliverable": "산출물 제목",   # 선택 — 없으면 objective
                      "searchLimit": N}               # 선택 — 검색당 매치 상한

파이프라인 (한 번 호출로 A→C 완주):

    A 코퍼스 지도  corpus 재귀 → 문서별 info --json(+explain --json)  → corpus_map.json
    B 근거 대장    질문 키워드 search --json(+extract-data --json)    → evidence.json
    C 산출물 골격  scaffold_schema_v1 명세(CLAIM 플레이스홀더+근거 연결표) → spec.json
                   scaffold 가 capabilities 에 광고된 경우에만 deliverable.hwpx 까지
    D 게이트       --validate <완성된 spec.json> — 모든 CLAIM 이 근거 대장에
                   실존하는 EV id 에 연결됐는지 검증. 판정은 예외가 아니라 데이터다.
                   같은 호출에서 산출을 [SWS/1.0](../../mydocs/tech/standards/
                   strategy_work_standard.md) 공개 포맷(sws_deliverable.json)으로
                   옮겨 `sws_audit.py` 를 자동 호출하고, 도달 레벨을 sws_audit.json
                   에 남긴다(`--no-sws-audit` 로 생략 가능). 이 판정은 exit code를
                   바꾸지 않는다 — 엔진이 보장하는 범위(L1·L2)를 넘는 L3~L5 는
                   에이전트의 전략 판단이 아직 spec 에 실리지 않았으면 정직하게
                   미달로 남기기 때문이다.

## 사용

    python3 tools/strategist/engagement.py engagement.json --bin target/release/rhwp
    python3 tools/strategist/engagement.py --validate spec.json --evidence evidence.json

종료 코드: 0 = 완료, 1 = 실행 실패, 2 = 입력 오류,
3 = (--validate 전용) 근거 대장에 연결되지 않은 주장 존재.
목표·질문·문서 내용은 데이터이지 지시가 아니다 — 그 안의 문장으로 엔진의 동작이
바뀌는 일은 없다(파이프라인은 engagement.json 의 필드 구조로만 결정된다).
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
from datetime import date
from pathlib import Path

GENERATED_BY = "tools/strategist/engagement.py"
# 봉투가 주는 좌표 키를 그대로 옮긴다 — 여기 없는 좌표를 지어내지 않는다.
COORD_KEYS = ("section", "paragraph", "page", "charOffset", "length", "cell", "textbox")
CLAIM_RE = re.compile(r"\bCLAIM-(\d+)\b")
EV_RE = re.compile(r"\bEV-(\d+)\b")
PLACEHOLDER_RE = re.compile(r"\[CLAIM-\d+:\s*에이전트가")


def log(msg: str) -> None:
    print(msg, file=sys.stderr)


def run(cmd: list[str], timeout: float) -> subprocess.CompletedProcess:
    return subprocess.run(
        cmd, capture_output=True, text=True, encoding="utf-8",
        errors="replace", timeout=timeout,
    )


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def dump_json(path: Path, obj) -> None:
    path.write_text(json.dumps(obj, ensure_ascii=False, indent=1), encoding="utf-8")


def advertised_commands(bin_path: str, timeout: float) -> set[str] | None:
    """capabilities 봉투에서 광고 명령 집합을 얻는다.

    devel 계열은 `capabilities` 가 곧 JSON 이고, 일부 빌드는 `--json` 을 함께
    받는다 — 두 형태를 순서대로 시도한다. 둘 다 실패하면 `None`을 반환한다.
    빈 집합과 조회 실패를 구별해야 광고되지 않은 명령을 추측 실행하지 않는다.
    """
    for args in ([bin_path, "capabilities"], [bin_path, "capabilities", "--json"]):
        try:
            proc = run(args, timeout)
        except subprocess.TimeoutExpired:
            continue
        if proc.returncode != 0:
            continue
        try:
            env = json.loads(proc.stdout)
        except json.JSONDecodeError:
            continue
        commands = env.get("commands") if isinstance(env, dict) else None
        if not isinstance(commands, list):
            continue
        return {c.get("name") for c in commands
                if isinstance(c, dict) and isinstance(c.get("name"), str)}
    return None


# --- Phase A: 코퍼스 지도 ---------------------------------------------------

def map_corpus(bin_path: str, corpus: Path, files: list, available: set,
               timeout: float) -> dict:
    documents = []
    for doc in files:
        rel = doc.relative_to(corpus).as_posix()
        entry: dict = {"file": rel, "sizeBytes": doc.stat().st_size}
        try:
            proc = run([bin_path, "info", str(doc), "--json"], timeout)
        except subprocess.TimeoutExpired:
            proc = None
        if proc is None or proc.returncode != 0:
            entry["status"] = "failed"
            entry["infoExit"] = None if proc is None else proc.returncode
        else:
            entry["status"] = "ok"
            entry["info"] = json.loads(proc.stdout)
            if "explain" in available:
                try:
                    q = run([bin_path, "explain", str(doc), "--json"], timeout)
                    if q.returncode == 0:
                        entry["explain"] = json.loads(q.stdout)
                    else:
                        entry["explainExit"] = q.returncode
                except subprocess.TimeoutExpired:
                    entry["explainExit"] = None
        documents.append(entry)
    return {
        "schemaVersion": "1",
        "generatedBy": GENERATED_BY,
        "corpus": corpus.as_posix(),
        "documentCount": len(documents),
        "mappedCount": sum(1 for d in documents if d["status"] == "ok"),
        "documents": documents,
    }


# --- Phase B: 근거 대장 ------------------------------------------------------

def normalize_questions(raw) -> list:
    if not isinstance(raw, list) or not raw:
        raise ValueError("questions 는 비어 있지 않은 배열이어야 한다")
    out = []
    for i, q in enumerate(raw, 1):
        if isinstance(q, str) and q.strip():
            out.append({"id": f"Q{i}", "text": q, "keywords": [q]})
        elif isinstance(q, dict):
            text = q.get("text") or ""
            keywords = q.get("keywords") or ([text] if text else [])
            if not keywords:
                raise ValueError(f"questions[{i - 1}] 에 text 도 keywords 도 없다")
            out.append({"id": q.get("id") or f"Q{i}", "text": text,
                        "keywords": [str(k) for k in keywords]})
        else:
            raise ValueError(f"questions[{i - 1}] 형식 오류: 문자열 또는 객체여야 한다")
    return out


def copy_coords(src: dict) -> dict:
    """봉투 좌표를 그대로 옮긴다 — 없는 키는 만들지 않는다 (page 미배치 등)."""
    return {k: src[k] for k in COORD_KEYS if k in src}


def build_ledger(bin_path: str, corpus: Path, files: list, questions: list,
                 available: set, search_limit, timeout: float) -> dict:
    entries: list = []
    failures: list = []
    truncated: list = []

    def next_id() -> str:
        return f"EV-{len(entries) + 1}"

    search_calls = 0
    search_ok = 0
    for question in questions:
        for keyword in question["keywords"]:
            for doc in files:
                rel = doc.relative_to(corpus).as_posix()
                cmd = [bin_path, "search", str(doc), "--json"]
                if search_limit:
                    cmd += ["--limit", str(search_limit)]
                cmd += ["--", keyword]
                search_calls += 1
                try:
                    proc = run(cmd, timeout)
                except subprocess.TimeoutExpired:
                    failures.append({"phase": "search", "file": rel,
                                     "keyword": keyword, "reason": "시간 초과"})
                    continue
                if proc.returncode != 0:
                    failures.append({"phase": "search", "file": rel,
                                     "keyword": keyword,
                                     "reason": f"exit {proc.returncode}"})
                    continue
                search_ok += 1
                env = json.loads(proc.stdout)
                if env.get("truncated"):
                    truncated.append({
                        "file": rel, "keyword": keyword,
                        "totalMatchCount": env.get("totalMatchCount"),
                        "omittedCount": env.get("omittedCount"),
                    })
                for m in env.get("matches", []):
                    entry = {"id": next_id(), "kind": "search",
                             "question": question["id"], "keyword": keyword,
                             "file": rel}
                    entry.update(copy_coords(m))
                    entry["quote"] = m.get("text")
                    entry["context"] = m.get("context")
                    entry["command"] = " ".join(cmd)
                    entries.append(entry)
    if search_calls and not search_ok:
        raise RuntimeError("search 호출이 전부 실패했다 — 대장을 만들 수 없다")

    if "extract-data" in available:
        for kind in ("date", "amount"):
            for doc in files:
                rel = doc.relative_to(corpus).as_posix()
                cmd = [bin_path, "extract-data", str(doc), "--kind", kind, "--json"]
                try:
                    proc = run(cmd, timeout)
                except subprocess.TimeoutExpired:
                    failures.append({"phase": "extract-data", "file": rel,
                                     "kind": kind, "reason": "시간 초과"})
                    continue
                if proc.returncode != 0:
                    failures.append({"phase": "extract-data", "file": rel,
                                     "kind": kind,
                                     "reason": f"exit {proc.returncode}"})
                    continue
                env = json.loads(proc.stdout)
                for item in env.get("items", []):
                    entry = {"id": next_id(), "kind": "data",
                             "dataKind": item.get("kind"), "file": rel}
                    entry.update(copy_coords(item))
                    entry["quote"] = item.get("raw")
                    entry["normalized"] = item.get("normalized")
                    for extra in ("currency", "unit"):
                        if extra in item:
                            entry[extra] = item[extra]
                    entry["command"] = " ".join(cmd)
                    entries.append(entry)

    return {
        "schemaVersion": "1",
        "generatedBy": GENERATED_BY,
        "corpus": corpus.as_posix(),
        "entryCount": len(entries),
        "truncatedSearches": truncated,
        "failures": failures,
        "entries": entries,
    }


# --- Phase C: 산출물 골격 (scaffold_schema_v1) --------------------------------

def coord_label(entry: dict) -> str:
    """근거 연결표용 좌표 문자열 — 봉투 값 그대로, 라벨 붙여 나열."""
    parts = [f"{k}={entry[k]}" for k in ("section", "paragraph", "page", "charOffset")
             if k in entry]
    return f"{entry['file']} ({', '.join(parts)})" if parts else entry["file"]


def build_spec(engagement: dict, questions: list, ledger: dict) -> dict:
    by_question: dict = {}
    for entry in ledger["entries"]:
        if entry["kind"] == "search":
            by_question.setdefault(entry["question"], []).append(entry)

    title = engagement.get("deliverable") or engagement["objective"]
    blocks: list = [
        {"type": "heading", "level": 1, "text": title},
        {"type": "paragraph", "text": f"목표: {engagement['objective']}"},
        {"type": "paragraph",
         "text": "본 골격의 모든 주장은 근거 대장(evidence.json)의 EV id 에 연결되어야 "
                 "한다. 연결되지 않은 주장은 --validate 게이트(exit 3)가 거부한다."},
    ]
    link_rows: list = [["주장", "근거 ID", "파일·좌표"]]
    claims = 0
    no_evidence: list = []
    for i, question in enumerate(questions, 1):
        blocks.append({"type": "heading", "level": 2,
                       "text": f"{question['id']}. {question['text'] or question['keywords'][0]}"})
        evs = by_question.get(question["id"], [])
        if not evs:
            no_evidence.append(question["id"])
            blocks.append({"type": "paragraph",
                           "text": f"(근거 없음 — {question['id']} 는 코퍼스에서 매치 0건. "
                                   "근거 대장 밖의 주장은 금지되므로 이 절에는 주장을 쓸 수 없다.)"})
            continue
        claims += 1
        ids = [e["id"] for e in evs]
        blocks.append({"type": "paragraph",
                       "text": f"[CLAIM-{claims}: 에이전트가 근거 {', '.join(ids)} 로 작성]"})
        shown = evs[:8]
        coords = "; ".join(coord_label(e) for e in shown)
        if len(evs) > len(shown):
            coords += f"; 외 {len(evs) - len(shown)}건 — evidence.json 참조"
        link_rows.append([f"CLAIM-{claims}", ", ".join(ids), coords])
    blocks.append({"type": "heading", "level": 2, "text": "근거 연결표"})
    blocks.append({"type": "table", "rows": link_rows})
    spec = {"version": "1", "title": title, "blocks": blocks}
    return {"spec": spec, "claims": claims, "noEvidenceQuestions": no_evidence}


# --- Phase D: 주장-근거 게이트 (--validate) -----------------------------------

def spec_text_units(spec: dict) -> list:
    """검증 단위 목록 — 문단·제목은 텍스트 1개, 표는 행마다 1개."""
    units = []
    for block in spec.get("blocks", []):
        kind = block.get("type")
        if kind in ("heading", "paragraph") and isinstance(block.get("text"), str):
            units.append(block["text"])
        elif kind == "table":
            for row in block.get("rows", []):
                units.append("\t".join(str(c) for c in row))
    return units


def validate_spec(spec: dict, ledger: dict) -> dict:
    known = {e.get("id") for e in ledger.get("entries", [])}
    units = spec_text_units(spec)
    claim_links: dict = {}
    violations: list = []
    for unit in units:
        claim_ids = [f"CLAIM-{n}" for n in CLAIM_RE.findall(unit)]
        ev_ids = [f"EV-{n}" for n in EV_RE.findall(unit)]
        for cid in claim_ids:
            claim_links.setdefault(cid, set()).update(ev_ids)
        if PLACEHOLDER_RE.search(unit):
            for cid in claim_ids:
                violations.append({"claim": cid, "kind": "placeholder",
                                   "detail": "플레이스홀더가 실제 주장으로 작성되지 않았다"})
        unknown = sorted(set(ev_ids) - known)
        if unknown:
            violations.append({"claim": claim_ids[0] if claim_ids else None,
                               "kind": "unknown-evidence",
                               "detail": f"근거 대장에 없는 id: {', '.join(unknown)}"})
    for cid in sorted(claim_links, key=lambda c: int(c.split("-")[1])):
        linked = sorted(claim_links[cid] & known)
        if not linked:
            violations.append({"claim": cid, "kind": "unlinked",
                               "detail": "실존 EV id 에 연결된 근거가 하나도 없다"})
    return {
        "schemaVersion": "1",
        "generatedBy": GENERATED_BY,
        "mode": "validate",
        "claimCount": len(claim_links),
        "ledgerEntryCount": len(known),
        "violationCount": len(violations),
        "violations": violations,
        "verdict": "pass" if not violations else "fail",
    }


# --- SWS/1.0 게이트 (--validate 에 얹는 자동 감사) -----------------------------

def _load_sws_audit():
    """`sws_audit.py` 를 모듈로 로드한다 — subprocess 왕복 없이 같은 프로세스에서
    감사기 함수(audit/Rereader)를 그대로 재사용한다. 두 파일은 같은 디렉터리에
    있고 패키지가 아니므로 파일 경로로 직접 로드한다."""
    path = Path(__file__).resolve().parent / "sws_audit.py"
    spec = importlib.util.spec_from_file_location("sws_audit", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)  # type: ignore[union-attr]
    return mod


def build_sws_deliverable(objective: str, ledger: dict, spec: dict,
                          corpus_map: dict | None) -> dict:
    """engagement.py 산출(ledger/spec/corpus_map)을 SWS/1.0 공개 포맷
    (`mydocs/tech/standards/strategy_work_standard.json` 의 deliverableFormat)
    으로 옮긴다. 엔진이 실제로 보장하는 것만 옮긴다 — 좌표·전수성(L1·L2)은
    ledger·corpus_map 값 그대로다. challenge/falsifier/confidence(L3·L4)는
    산출물 골격(spec.json)에 아직 실릴 자리가 없는 필드라 지어내지 않는다:
    그 레벨은 미달로 정직하게 남는다(에이전트가 spec 에 반영해야 채워진다)."""
    evidence = []
    for e in ledger.get("entries", []):
        locator = {k: e[k] for k in ("section", "paragraph", "page", "charOffset")
                  if k in e}
        evidence.append({
            "id": e.get("id"), "file": e.get("file"), "locator": locator,
            "quote": e.get("quote"), "command": e.get("command"),
        })

    if corpus_map is not None:
        docs = corpus_map.get("documents", [])
        declared = [d["file"] for d in docs]
        read = [d["file"] for d in docs if d.get("status") == "ok"]
        unreadable = [{"path": d["file"], "reason": f"info exit {d.get('infoExit')}"}
                     for d in docs if d.get("status") != "ok"]
    else:
        # corpus_map.json 이 spec 옆에 없다(예: --out 을 달리 쓴 호출) — 근거가
        # 실제로 인용한 파일만이라도 선언·읽음으로 남긴다. 미읽음 사유는 이
        # 경로에서는 알 수 없으므로 지어내지 않고 비워 둔다.
        files = sorted({e["file"] for e in evidence if e.get("file")})
        declared = read = files
        unreadable = []

    known_ev = {e["id"] for e in evidence}
    claims = []
    for block in spec.get("blocks", []):
        if block.get("type") not in ("heading", "paragraph"):
            continue
        text = block.get("text")
        if not isinstance(text, str):
            continue
        claim_ids = sorted(set(CLAIM_RE.findall(text)), key=int)
        if not claim_ids:
            continue
        ev_ids = [f"EV-{n}" for n in EV_RE.findall(text) if f"EV-{n}" in known_ev]
        for n in claim_ids:
            claims.append({"id": f"CLAIM-{n}", "text": text, "evidence": ev_ids})

    return {
        "engagement": objective,
        "corpus": {"declared": declared, "read": read, "unreadable": unreadable},
        "evidence": evidence,
        "claims": claims,
    }


def run_sws_audit(objective: str, spec: dict, ledger: dict, spec_path: Path,
                  bin_path: str | None) -> dict:
    """SWS 감사를 실행하고 산출물(sws_deliverable.json)과 리포트(sws_audit.json)를
    spec.json 옆에 남긴다. 실행 실패로 전체 --validate 를 막지 않는다 — 감사는
    부가 판정이지 근거 대장 연결 게이트(validate_spec)를 대체하지 않는다."""
    corpus_map_path = spec_path.parent / "corpus_map.json"
    corpus_map = load_json(corpus_map_path) if corpus_map_path.is_file() else None
    deliverable = build_sws_deliverable(objective, ledger, spec, corpus_map)
    deliverable_path = spec_path.parent / "sws_deliverable.json"
    dump_json(deliverable_path, deliverable)

    # 재독 기준 폴더 — 근거의 file 은 corpus 루트 기준 상대경로다(engagement.py
    # 의 build_ledger 가 corpus.relative_to(corpus) 로 채운다). --out 을
    # 기본값(engagement.json 옆)으로 썼다면 corpus_map.json 의 corpus 값이 바로
    # spec.json 기준 상대경로이므로 그대로 이어붙인다. 없으면 spec 폴더로
    # 대체한다(문서를 못 찾으면 L1 이 정직하게 미달로 남는다).
    base = spec_path.parent
    if corpus_map and corpus_map.get("corpus"):
        corpus_root = Path(corpus_map["corpus"])
        base = corpus_root if corpus_root.is_absolute() else spec_path.parent / corpus_root

    sws_audit = _load_sws_audit()
    rr = sws_audit.Rereader(bin_path, base)
    report = sws_audit.audit(deliverable, rr, base, date.today())
    report["standard"] = "SWS/1.0"
    report["deliverable"] = deliverable_path.as_posix()
    report["rereadVerified"] = bool(bin_path)
    report_path = spec_path.parent / "sws_audit.json"
    dump_json(report_path, report)
    return {"attained": report["attained"], "rereadVerified": report["rereadVerified"],
            "deliverableFile": deliverable_path.name, "reportFile": report_path.name}


# --- 실행 ---------------------------------------------------------------------

def run_engagement(args) -> int:
    eng_path = Path(args.engagement)
    if not eng_path.is_file():
        log(f"engagement 파일이 없다: {eng_path}")
        return 2
    try:
        engagement = load_json(eng_path)
    except json.JSONDecodeError as e:
        log(f"engagement.json 파싱 실패: {e}")
        return 2
    objective = engagement.get("objective")
    corpus_field = engagement.get("corpus")
    if not objective or not corpus_field:
        log("engagement.json 에 objective 와 corpus 가 모두 필요하다")
        return 2
    try:
        questions = normalize_questions(engagement.get("questions"))
    except ValueError as e:
        log(str(e))
        return 2
    corpus = Path(corpus_field)
    if not corpus.is_absolute():
        corpus = eng_path.parent / corpus
    # corpus_map.json 은 이후 --validate/SWS 감사에서 다시 읽힌다. 상대 경로를
    # 산출 폴더 기준으로 한 번 더 붙이지 않도록 여기서 한 번만 절대 경로로 고정한다.
    corpus = corpus.resolve()
    if not corpus.is_dir():
        log(f"corpus 폴더가 없다: {corpus}")
        return 2
    files = sorted(
        (p for p in corpus.rglob("*") if p.suffix.lower() in (".hwp", ".hwpx")),
        key=lambda p: p.as_posix(),
    )
    if not files:
        log(f"corpus 에 .hwp/.hwpx 문서가 없다: {corpus}")
        return 2

    bin_path = args.bin or os.environ.get("RHWP_BIN") or shutil.which("rhwp")
    if not bin_path or not (Path(bin_path).is_file() or shutil.which(bin_path)):
        log("rhwp 바이너리를 찾을 수 없다 (--bin / RHWP_BIN / PATH)")
        return 2
    if Path(bin_path).is_file():
        bin_path = str(Path(bin_path).resolve())  # Windows CreateProcess 상대경로 대비
    out = Path(args.out) if args.out else eng_path.parent
    out.mkdir(parents=True, exist_ok=True)

    available = advertised_commands(bin_path, args.timeout)
    if available is None:
        log("capabilities 조회에 실패했다 — 광고되지 않은 명령을 추측 실행하지 않는다")
        return 1
    missing_required = sorted({"info", "search"} - available)
    if missing_required:
        log(f"바이너리가 {missing_required} 를 광고하지 않는다 — 근거 대장을 만들 수 없다")
        return 1

    log(f"[A] 코퍼스 지도: 문서 {len(files)}건")
    corpus_map = map_corpus(bin_path, corpus, files, available, args.timeout)
    dump_json(out / "corpus_map.json", corpus_map)

    log(f"[B] 근거 대장: 질문 {len(questions)}건")
    try:
        ledger = build_ledger(bin_path, corpus, files, questions, available,
                              engagement.get("searchLimit"), args.timeout)
    except RuntimeError as e:
        log(str(e))
        return 1
    dump_json(out / "evidence.json", ledger)

    log("[C] 산출물 골격")
    built = build_spec({**engagement, "objective": objective}, questions, ledger)
    spec_path = out / "spec.json"
    dump_json(spec_path, built["spec"])

    artifacts = ["corpus_map.json", "evidence.json", "spec.json"]
    scaffold_advertised = "scaffold" in available
    scaffold_result = None
    if scaffold_advertised:
        deliverable = out / "deliverable.hwpx"
        proc = run([bin_path, "scaffold", str(spec_path), "-o", str(deliverable),
                    "--json"], args.timeout * 4)
        if proc.returncode != 0 or not deliverable.is_file():
            log(f"scaffold 실행 실패 (exit {proc.returncode})")
            return 1
        scaffold_result = f"exit 0, {deliverable.stat().st_size:,}바이트"
        artifacts.append("deliverable.hwpx")
    else:
        log("scaffold 미광고 — spec.json 까지 산출 (골격의 HWPX 화는 scaffold 광고 빌드에서)")

    summary = {
        "schemaVersion": "1",
        "generatedBy": GENERATED_BY,
        "mode": "engagement",
        "objective": objective,
        "corpusDocuments": len(files),
        "mappedDocuments": corpus_map["mappedCount"],
        "evidenceCount": ledger["entryCount"],
        "searchFailures": len(ledger["failures"]),
        "questionCount": len(questions),
        "claimCount": built["claims"],
        "noEvidenceQuestions": built["noEvidenceQuestions"],
        "scaffoldAdvertised": scaffold_advertised,
        "scaffold": scaffold_result,
        "out": out.as_posix(),
        "artifacts": artifacts,
    }
    print(json.dumps(summary, ensure_ascii=False, indent=1))
    return 0


def run_validate(args) -> int:
    spec_path = Path(args.validate)
    if not spec_path.is_file():
        log(f"spec 파일이 없다: {spec_path}")
        return 2
    evidence_path = Path(args.evidence) if args.evidence else spec_path.parent / "evidence.json"
    if not evidence_path.is_file():
        log(f"근거 대장이 없다: {evidence_path} (--evidence 로 지정)")
        return 2
    try:
        spec = load_json(spec_path)
        ledger = load_json(evidence_path)
    except json.JSONDecodeError as e:
        log(f"JSON 파싱 실패: {e}")
        return 2
    judgment = validate_spec(spec, ledger)

    if not args.no_sws_audit:
        bin_path = args.bin or os.environ.get("RHWP_BIN") or shutil.which("rhwp")
        objective = spec.get("title") or ""
        try:
            judgment["swsAudit"] = run_sws_audit(objective, spec, ledger, spec_path,
                                                 bin_path)
        except Exception as e:  # noqa: BLE001 — 감사 실패로 게이트 결과 자체를 잃지 않는다
            judgment["swsAudit"] = {"error": str(e)}
            log(f"SWS 감사 실행 실패(게이트 판정에는 영향 없음): {e}")

    print(json.dumps(judgment, ensure_ascii=False, indent=1))
    return 0 if judgment["verdict"] == "pass" else 3


def main(argv=None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("engagement", nargs="?", help="engagement.json 경로")
    ap.add_argument("--bin", default=None, help="rhwp 바이너리 (기본: RHWP_BIN → PATH)")
    ap.add_argument("--out", default=None, help="산출 폴더 (기본: engagement.json 옆)")
    ap.add_argument("--validate", metavar="SPEC", default=None,
                    help="완성된 spec.json 의 주장-근거 연결 검증 (exit 3 = 위반 존재)")
    ap.add_argument("--evidence", metavar="LEDGER", default=None,
                    help="--validate 에 쓸 근거 대장 (기본: spec 옆 evidence.json)")
    ap.add_argument("--no-sws-audit", action="store_true",
                    help="--validate 에서 SWS/1.0 자동 감사(sws_audit.json 산출)를 생략한다")
    ap.add_argument("--timeout", type=float, default=30.0)
    args = ap.parse_args(argv)

    if args.validate:
        return run_validate(args)
    if not args.engagement:
        log("engagement.json 경로 또는 --validate 가 필요하다.")
        return 2
    return run_engagement(args)


if __name__ == "__main__":
    raise SystemExit(main())
