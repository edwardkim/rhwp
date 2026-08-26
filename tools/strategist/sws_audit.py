#!/usr/bin/env python3
"""SWS/1.0 감사기 — 전략 산출물을 되짚어 채점한다 (표준 정본: mydocs/tech/standards/).

## 왜 있는가

전략 보고서의 문장은 지금까지 **검증 대상이 아니었다**. "이 주장은 어느 원자료에서
왔는가"를 물으면 저자를 다시 불러야 했고, 그래서 컨설팅 산출물의 품질은 신뢰의
문제였지 검증의 문제가 아니었다.

이 감사기는 [SWS/1.0](../../mydocs/tech/standards/strategy_work_standard.md) 의
5단 레벨을 기계로 판정한다. 핵심은 **체크리스트가 아니라 재독**이다: SW-L1 은
"근거에 좌표 필드가 있는가"를 보는 게 아니라, 그 좌표로 실제 문서를 다시 검색해
**인용이 거기서 그대로 나오는지** 확인한다. 지어낸 인용·표류한 좌표는 여기서 죽는다.

도구 무관이다. rhwp 로 만든 산출물이 아니어도, 표준의 공개 포맷으로 주장·근거를
노출하기만 하면 채점된다 — 사람이 쓴 보고서도 마찬가지다. 낮은 점수는 저자의
역량 판정이 아니라 포맷이 좌표를 싣지 못했다는 사실의 기록이다(표준 §경계).

## 사용

    # 자기 채점 (재독 검증 포함)
    python3 tools/strategist/sws_audit.py deliverable.json --bin target/release/rhwp --level L1

    # 도달 가능한 최고 레벨 판정
    python3 tools/strategist/sws_audit.py deliverable.json --bin <rhwp> --json

    # 재독 없이 형식만 (문서 없이 포맷 점검할 때 — L1 은 '미검증'으로 남는다)
    python3 tools/strategist/sws_audit.py deliverable.json --no-reread

    # 표준 두 정본(json/md)이 같은 레벨을 말하는지 (척추 자기검사)
    python3 tools/strategist/sws_audit.py --self-check

종료 코드: 0 = 요청 레벨 충족 / 3 = 미충족(**판정이지 오류가 아니다**) /
2 = 입력 오류 / 1 = 실행 실패. 이 저장소의 "exit 3 = 판정" 관례를 따른다.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from datetime import date
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
STD_JSON = REPO_ROOT / "mydocs" / "tech" / "standards" / "strategy_work_standard.json"
STD_MD = REPO_ROOT / "mydocs" / "tech" / "standards" / "strategy_work_standard.md"
LEVEL_IDS = ["SW-L1", "SW-L2", "SW-L3", "SW-L4", "SW-L5"]
VALID_VERDICTS = {"survived", "weakened", "refuted"}


def log(msg: str) -> None:
    print(msg, file=sys.stderr)


# --- 척추 자기검사 (AWS/1.0 의 adoption_spine 과 같은 취지) ------------------

def self_check() -> int:
    problems = []
    if not STD_JSON.is_file() or not STD_MD.is_file():
        log("표준 정본 파일이 없다.")
        return 2
    std = json.loads(STD_JSON.read_text(encoding="utf-8"))
    md = STD_MD.read_text(encoding="utf-8")

    ids = [lv.get("id") for lv in std.get("levels", [])]
    if ids != LEVEL_IDS:
        problems.append(f"기계 정본 레벨이 {LEVEL_IDS} 가 아니다: {ids}")
    for lid in LEVEL_IDS:
        if lid not in md:
            problems.append(f"사람 정본에 {lid} 가 없다")
    for key in ("standard", "version", "measures", "legitimacy", "surfaces"):
        if not std.get(key):
            problems.append(f"기계 정본에 {key} 가 없다")
    for surface in std.get("surfaces", []):
        if not (REPO_ROOT / surface).exists():
            problems.append(f"선언한 surface 가 실재하지 않는다: {surface}")

    for p in problems:
        log(f"- {p}")
    log(f"자기검사: 문제 {len(problems)}건")
    return 0 if not problems else 3


# --- 재독 검증 (SW-L1 의 핵심) ----------------------------------------------

class Rereader:
    """근거의 인용을 원문에서 실제로 다시 찾아 좌표까지 대조한다."""

    def __init__(self, bin_path: str | None, base: Path, timeout: float = 30.0):
        self.bin = bin_path
        self.base = base
        self.timeout = timeout
        self.cache: dict = {}

    def verify(self, ev: dict) -> tuple[bool, str]:
        if not self.bin:
            return False, "재독 안 함(--no-reread 또는 바이너리 없음)"
        doc = (self.base / ev["file"]).resolve()
        if not doc.is_file():
            return False, f"문서가 없다: {ev['file']}"
        quote = ev.get("quote", "").strip()
        if not quote:
            return False, "인용이 비어 있다"
        # 검색어는 인용 앞부분으로 — 긴 인용은 문단 경계·조판 문자로 완전일치가 깨진다.
        needle = quote[:40]
        key = (str(doc), needle)
        if key not in self.cache:
            try:
                proc = subprocess.run(
                    [self.bin, "search", str(doc), "--json", "--", needle],
                    capture_output=True, text=True, encoding="utf-8",
                    errors="replace", timeout=self.timeout,
                )
            except subprocess.TimeoutExpired:
                return False, "search 시간 초과"
            if proc.returncode != 0:
                return False, f"search exit {proc.returncode}"
            try:
                self.cache[key] = json.loads(proc.stdout).get("matches", [])
            except json.JSONDecodeError:
                return False, "search 봉투가 JSON 이 아니다"
        matches = self.cache[key]
        if not matches:
            return False, f"원문에서 인용을 찾지 못했다: {needle!r}"

        loc = ev.get("locator") or {}
        # 좌표가 선언됐으면 매치 중 하나와 일치해야 한다 — 표류한 좌표를 잡는다.
        for axis in ("page", "paragraph", "section"):
            if axis in loc and loc[axis] is not None:
                if not any(m.get(axis) == loc[axis] for m in matches):
                    found = sorted({m.get(axis) for m in matches if axis in m})
                    return False, f"{axis} 좌표 불일치: 선언 {loc[axis]}, 실제 매치 {found}"
        return True, f"재독 확인 (매치 {len(matches)}건)"


# --- 레벨 판정 ---------------------------------------------------------------

def check_l1(d: dict, rr: Rereader) -> list[str]:
    fails = []
    ev_by_id = {e.get("id"): e for e in d.get("evidence", [])}
    claims = d.get("claims", [])
    if not claims:
        return ["주장이 하나도 없다"]
    for c in claims:
        cid = c.get("id", "?")
        refs = c.get("evidence") or []
        if not refs:
            fails.append(f"{cid}: 근거에 연결되지 않은 주장")
            continue
        for ref in refs:
            ev = ev_by_id.get(ref)
            if ev is None:
                fails.append(f"{cid}: 근거 대장에 없는 근거 id {ref}")
                continue
            if not ev.get("file") or not ev.get("quote"):
                fails.append(f"{ref}: file 또는 quote 누락")
                continue
            if not (ev.get("locator") or {}):
                fails.append(f"{ref}: 좌표(locator) 없음")
                continue
            ok, why = rr.verify(ev)
            if not ok:
                fails.append(f"{ref}: {why}")
    return fails


def check_l2(d: dict) -> list[str]:
    fails = []
    corpus = d.get("corpus") or {}
    declared = set(corpus.get("declared") or [])
    read = set(corpus.get("read") or [])
    unreadable = corpus.get("unreadable") or []
    if not declared:
        return ["코퍼스가 선언되지 않았다 (corpus.declared)"]
    if not read and not unreadable:
        fails.append("읽은 문서도 못 읽은 문서도 기록되지 않았다")
    if read - declared:
        fails.append(f"선언되지 않은 문서를 읽었다: {sorted(read - declared)}")
    for u in unreadable:
        if not isinstance(u, dict) or not u.get("path") or not u.get("reason"):
            fails.append(f"unreadable 항목에 path/reason 누락: {u}")
    unread_paths = {u.get("path") for u in unreadable if isinstance(u, dict)}
    missing = declared - read - unread_paths
    if missing:
        fails.append(f"선언됐으나 읽지도 사유도 없는 문서: {sorted(missing)}")
    for e in d.get("evidence", []):
        if e.get("file") and e["file"] not in read:
            fails.append(f"{e.get('id')}: 읽었다고 선언되지 않은 문서에서 온 근거 ({e['file']})")
    return fails


def check_l3(d: dict) -> list[str]:
    fails = []
    ev_ids = {e.get("id") for e in d.get("evidence", [])}
    for c in d.get("claims", []):
        cid = c.get("id", "?")
        ch = c.get("challenge") or {}
        if not ch:
            fails.append(f"{cid}: 반증 시도 기록 없음 (challenge)")
            continue
        if not (ch.get("searched") or "").strip():
            fails.append(f"{cid}: 실행한 반대 근거 질의(challenge.searched)가 비어 있다")
        verdict = ch.get("verdict")
        if verdict not in VALID_VERDICTS:
            fails.append(f"{cid}: verdict 가 {sorted(VALID_VERDICTS)} 중 하나가 아니다 ({verdict})")
        elif verdict == "refuted":
            fails.append(f"{cid}: 반증된 주장이 산출물에 남아 있다 — 삭제하거나 강등하라")
        for ce in ch.get("counterEvidence") or []:
            if ce not in ev_ids:
                fails.append(f"{cid}: 근거 대장에 없는 반대 근거 id {ce}")
    return fails


def check_l4(d: dict, today: date) -> list[str]:
    fails = []
    for c in d.get("claims", []):
        cid = c.get("id", "?")
        if not (c.get("falsifier") or "").strip():
            fails.append(f"{cid}: falsifier 없음 — 틀릴 조건이 없는 주장은 검증 대상이 아니다")
        conf = c.get("confidence")
        if not isinstance(conf, (int, float)) or not 0.0 <= float(conf) <= 1.0:
            fails.append(f"{cid}: confidence 가 0~1 범위의 수가 아니다 ({conf})")
        rb = c.get("resolveBy")
        if rb:
            try:
                due = date.fromisoformat(rb)
            except ValueError:
                fails.append(f"{cid}: resolveBy 가 ISO 날짜가 아니다 ({rb})")
                continue
            if due < today and not (c.get("resolvedOutcome") or "").strip():
                fails.append(f"{cid}: 판정 기한({rb})이 지났는데 실제 결과(resolvedOutcome)가 없다")
    return fails


def check_l5(d: dict, base: Path) -> list[str]:
    fails = []
    receipt = d.get("receipt") or {}
    capsule = receipt.get("capsule")
    if not capsule:
        return ["영수증(receipt.capsule)이 없다 — AWS/1.0 AW-L1 접합 필요"]
    if not (base / capsule).is_file():
        fails.append(f"영수증 캡슐 파일이 실재하지 않는다: {capsule}")
    return fails


def audit(d: dict, rr: Rereader, base: Path, today: date) -> dict:
    results = {
        "SW-L1": check_l1(d, rr),
        "SW-L2": check_l2(d),
        "SW-L3": check_l3(d),
        "SW-L4": check_l4(d, today),
        "SW-L5": check_l5(d, base),
    }
    attained = None
    for lid in LEVEL_IDS:  # 누적 — 앞 레벨이 깨지면 뒤는 세지 않는다
        if results[lid]:
            break
        attained = lid
    return {"attained": attained, "findings": results}


def main(argv=None) -> int:
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("deliverable", nargs="?", help="산출물 JSON (표준 공개 포맷)")
    ap.add_argument("--bin", default=None, help="rhwp 바이너리 (재독 검증용)")
    ap.add_argument("--base", default=None, help="문서 경로 기준 폴더 (기본: 산출물 파일 위치)")
    ap.add_argument("--level", choices=["L1", "L2", "L3", "L4", "L5"],
                    help="이 레벨 충족 여부로 종료 코드 판정 (생략 시 도달 레벨만 보고)")
    ap.add_argument("--no-reread", action="store_true", help="재독 검증 생략 (형식만)")
    ap.add_argument("--self-check", action="store_true", help="표준 두 정본 정합 검사")
    ap.add_argument("--json", action="store_true", help="기계용 리포트")
    ap.add_argument("--today", default=None, help="기한 판정 기준일 (ISO, 테스트용)")
    args = ap.parse_args(argv)

    if args.self_check:
        return self_check()
    if not args.deliverable:
        log("산출물 JSON 경로가 필요하다 (또는 --self-check).")
        return 2

    path = Path(args.deliverable)
    if not path.is_file():
        log(f"산출물이 없다: {path}")
        return 2
    try:
        d = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        log(f"산출물이 JSON 이 아니다: {e}")
        return 2

    base = Path(args.base) if args.base else path.parent
    bin_path = None
    if not args.no_reread:
        import os
        bin_path = args.bin or os.environ.get("RHWP_BIN") or shutil.which("rhwp")
        if bin_path and not (Path(bin_path).is_file() or shutil.which(bin_path)):
            log(f"바이너리를 찾을 수 없다: {bin_path}")
            return 2
    today = date.fromisoformat(args.today) if args.today else date.today()

    report = audit(d, Rereader(bin_path, base), base, today)
    report["standard"] = "SWS/1.0"
    report["deliverable"] = str(path)
    report["rereadVerified"] = bool(bin_path)

    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=1))
    else:
        print(f"SWS/1.0 감사 — {path.name}")
        print(f"도달 레벨: {report['attained'] or '(SW-L1 미달)'}"
              + ("" if bin_path else "  [재독 미검증]"))
        for lid in LEVEL_IDS:
            fails = report["findings"][lid]
            print(f"  {lid}: {'통과' if not fails else f'{len(fails)}건 미충족'}")
            for f in fails[:10]:
                print(f"    - {f}")
            if len(fails) > 10:
                print(f"    … 외 {len(fails) - 10}건")

    if args.level:
        want = "SW-" + args.level
        if report["findings"][want] or (
            LEVEL_IDS.index(want) > (LEVEL_IDS.index(report["attained"]) if report["attained"] else -1)
        ):
            return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
