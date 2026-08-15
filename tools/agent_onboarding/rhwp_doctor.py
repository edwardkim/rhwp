#!/usr/bin/env python3
"""rhwp_doctor.py — 에이전트 제로프릭션 온보딩 닥터 + 부트스트랩.

한 명령으로 "rhwp 를 처음 보는 에이전트"가 다음 넷을 한 번에 끝낸다:

  1. 바이너리 위치·버전 확인 (PATH → target/release → 없으면 빌드 명령 안내)
  2. 번들 샘플로 읽기 전용 자가검증 (info / export-text 구조 출력 확인)
  3. 붙여넣기용 .mcp.json 스니펫 방출 (rhwp mcp-serve)
  4. "첫 5분" 레시피 지도 (실존 스킬·레시피만 인용)

설계 규약(저장소 철학과 일치):
  - 판정은 데이터다: 통과를 절대 위조하지 않는다. 못 돌린 검사는 SKIPPED/FAIL 로
    이유와 함께 정직하게 보고한다.
  - 매달리지 않는다: 바이너리가 없으면 긴 빌드를 강제하지 않고 빌드 명령만 찍고
    종료 코드로 신호한다. 모든 하위 프로세스는 타임아웃으로 감싼다.
  - 순수 Python 3 표준 라이브러리만 사용한다(외부 의존성 0). 반복 실행에 안전하다.

종료 코드(에이전트 계약):
  0  모든 임계 검사 통과 — 바로 붙여도 됨
  1  임계 검사 실패 — 바이너리는 있으나 버전/자가검증이 깨짐
  2  사용법 오류 — 잘못된 인자, --write 덮어쓰기 거부(--force 없이)
  3  바이너리 미발견 — 아직 빌드 안 됨(조치: 아래 빌드 명령 실행)

--json 을 주면 stdout 에는 기계 판독용 리포트 JSON 하나만 나가고, 사람용 텍스트는
전부 stderr 로 간다(에이전트가 stdout 을 그대로 파싱한다).
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

SCHEMA_VERSION = "1.0"
BUILD_COMMAND = "cargo build --release --bin rhwp"
# 하위 프로세스 상한(초) — 어떤 검사도 매달리지 않게 한다.
VERSION_TIMEOUT = 20
SELFTEST_TIMEOUT = 45

# 자가검증에 쓸 "정상 문서" 후보(첫 존재 파일 선택). 병리적 픽스처가 아니라
# 평범한 문서만 고른다 — 자가검증이 실패하면 그건 진짜 신호여야 한다.
SAMPLE_CANDIDATES = [
    "samples/basic/english.hwp",
    "samples/basic/KTX.hwp",
    "samples/basic/BookReview.hwp",
    "samples/2022년 국립국어원 업무계획.hwp",
    "samples/2022년 국립국어원 업무계획.hwpx",
]

# 첫 5분 레시피 지도 — 브리프가 지정한 5대 고가치 과제.
# 각 항목의 skill/recipe 경로는 런타임에 실존을 검증해 인용한다(없으면 정직하게 표시).
RECIPES = [
    {
        "task": "문서 트리아지 — 처음 보는 문서를 컨텍스트 아끼며 파악",
        "command": 'rhwp digest "<파일>" --json',
        "skill": "rhwp-doc-triage",
        "recipe": None,
    },
    {
        "task": "표 추출 — 병합 보존 격자 / CSV 왕복",
        "command": 'rhwp export-tables "<파일>" --json',
        "skill": "rhwp-table-exchange",
        "recipe": "mydocs/manual/recipes/02_table_csv_roundtrip.md",
    },
    {
        "task": "서식 채우기 — 누름틀 조사 후 값 채워 제출본 생성",
        "command": 'rhwp fields "<파일>" --json  →  rhwp edit fill-fields "<파일>" --data @row.json -o out.hwp --json',
        "skill": "rhwp-form-fill",
        "recipe": "mydocs/manual/recipes/01_fill_form_and_submit.md",
    },
    {
        "task": "보안 스윕 — 배포 전/수신 후 주입·은닉·유니코드 점검",
        "command": 'rhwp inspect injection "<파일>" --json',
        "skill": "rhwp-security-sweep",
        "recipe": "mydocs/manual/recipes/10_security_sweep_before_share.md",
    },
    {
        "task": "작업 영수증 — 산출물을 3-해시로 증명·재현 검증",
        "command": "rhwp replay --plan-json '{\"planVersion\":\"1.0\",...}' --json",
        "skill": "rhwp-work-receipt",
        "recipe": None,
    },
]

PASS, FAIL, SKIP = "PASS", "FAIL", "SKIP"


# --------------------------------------------------------------------------- #
# 순수 로직(바이너리 불요) — 가드 테스트가 여기를 겨눈다.
# --------------------------------------------------------------------------- #
def default_repo_root() -> Path:
    """이 스크립트 위치(tools/agent_onboarding/x.py)에서 저장소 루트를 유도한다."""
    return Path(__file__).resolve().parents[2]


def build_mcp_snippet(command: str, args=None):
    """붙여넣기용 .mcp.json 딕셔너리를 만든다.

    command 은 PATH 에 rhwp 가 있으면 "rhwp", 아니면 바이너리 절대 경로다
    (mcp_integration_guide.md: "PATH 에 없으면 command 에 절대 경로를 쓴다").
    """
    if args is None:
        args = ["mcp-serve"]
    return {"mcpServers": {"rhwp": {"command": command, "args": list(args)}}}


def aggregate(checks, binary_found: bool):
    """검사 목록 → (ok, exit_code). 순수 함수(가드 테스트 대상).

    ok 는 임계 검사가 하나도 실패/스킵되지 않았을 때만 True.
    exit_code: 0 정상 / 3 바이너리 미발견 / 1 임계 실패.
    """
    critical = [c for c in checks if c.get("critical")]
    all_pass = all(c["status"] == PASS for c in critical)
    if not binary_found:
        return False, 3
    if all_pass:
        return True, 0
    return False, 1


def resolve_recipe_map(repo_root: Path):
    """RECIPES 를 실존 검증과 함께 해석한다. 없는 스킬/레시피는 정직하게 표시."""
    out = []
    for r in RECIPES:
        skill_rel = f".claude/skills/{r['skill']}"
        skill_exists = (repo_root / skill_rel / "SKILL.md").is_file()
        recipe_rel = r["recipe"]
        recipe_exists = bool(recipe_rel) and (repo_root / recipe_rel).is_file()
        out.append(
            {
                "task": r["task"],
                "command": r["command"],
                "skill": r["skill"],
                "skillPath": skill_rel,
                "skillExists": skill_exists,
                "recipe": recipe_rel,
                "recipeExists": recipe_exists,
            }
        )
    return out


def pick_sample(repo_root: Path, override: str | None):
    """자가검증용 샘플 경로를 고른다(override 우선, 아니면 후보 중 첫 존재)."""
    if override:
        p = Path(override)
        return p if p.is_file() else None
    for rel in SAMPLE_CANDIDATES:
        p = repo_root / rel
        if p.is_file():
            return p
    return None


# --------------------------------------------------------------------------- #
# 바이너리 조달·실행
# --------------------------------------------------------------------------- #
def find_binary(repo_root: Path, override: str | None):
    """rhwp 바이너리를 찾는다. 반환: (path|None, source, on_path)."""
    if override:
        p = Path(override)
        if p.is_file():
            return p, "--rhwp", False
        return None, "--rhwp(미발견)", False
    on_path = shutil.which("rhwp")
    if on_path:
        return Path(on_path), "PATH", True
    exe = "rhwp.exe" if os.name == "nt" else "rhwp"
    cand = repo_root / "target" / "release" / exe
    if cand.is_file():
        return cand, "target/release", False
    return None, "(미발견)", False


def _run(binary: Path, args, timeout: int):
    """rhwp 를 실행하고 (exit, stdout_str, stderr_str) 반환. 타임아웃/오류는 예외로 던진다.

    Windows cp949 로케일에서도 UTF-8 JSON 이 깨지지 않도록 bytes 로 받아 직접 디코드한다.
    """
    proc = subprocess.run(
        [str(binary), *args],
        capture_output=True,
        timeout=timeout,
        check=False,
    )
    out = proc.stdout.decode("utf-8", errors="replace")
    err = proc.stderr.decode("utf-8", errors="replace")
    return proc.returncode, out, err


def check_version(binary: Path):
    cmd = "rhwp --version"
    try:
        code, out, err = _run(binary, ["--version"], VERSION_TIMEOUT)
    except subprocess.TimeoutExpired:
        return _mk("version", "바이너리 버전", FAIL, cmd, f"{VERSION_TIMEOUT}s 내 무응답(타임아웃)", True)
    except OSError as e:
        return _mk("version", "바이너리 버전", FAIL, cmd, f"실행 불가: {e}", True)
    text = (out or err).strip()
    if code == 0 and text:
        return _mk("version", "바이너리 버전", PASS, cmd, text.splitlines()[0], True, version=text.splitlines()[0])
    return _mk("version", "바이너리 버전", FAIL, cmd, f"exit={code}, 출력='{text[:80]}'", True)


def check_info(binary: Path, sample: Path):
    cmd = f'rhwp info "{sample}" --json'
    try:
        code, out, err = _run(binary, ["info", str(sample), "--json"], SELFTEST_TIMEOUT)
    except subprocess.TimeoutExpired:
        return _mk("selftest-info", "자가검증: info", FAIL, cmd, f"{SELFTEST_TIMEOUT}s 내 무응답(타임아웃)", True)
    except OSError as e:
        return _mk("selftest-info", "자가검증: info", FAIL, cmd, f"실행 불가: {e}", True)
    if code != 0:
        return _mk("selftest-info", "자가검증: info", FAIL, cmd, f"exit={code}: {(err or out).strip()[:120]}", True)
    try:
        obj = json.loads(out)
    except json.JSONDecodeError as e:
        return _mk("selftest-info", "자가검증: info", FAIL, cmd, f"JSON 파싱 실패: {e}", True)
    if not isinstance(obj, dict) or "format" not in obj or "pageCount" not in obj:
        return _mk("selftest-info", "자가검증: info", FAIL, cmd, "구조 출력에 format/pageCount 없음", True)
    detail = f"format={obj.get('format')}, pageCount={obj.get('pageCount')}, version={obj.get('version')}"
    return _mk("selftest-info", "자가검증: info", PASS, cmd, detail, True)


def check_export_text(binary: Path, sample: Path):
    cmd = f'rhwp export-text "{sample}" --json --max-chars 2000'
    try:
        code, out, err = _run(
            binary, ["export-text", str(sample), "--json", "--max-chars", "2000"], SELFTEST_TIMEOUT
        )
    except subprocess.TimeoutExpired:
        return _mk("selftest-export-text", "자가검증: export-text", FAIL, cmd, f"{SELFTEST_TIMEOUT}s 내 무응답(타임아웃)", True)
    except OSError as e:
        return _mk("selftest-export-text", "자가검증: export-text", FAIL, cmd, f"실행 불가: {e}", True)
    if code != 0:
        return _mk("selftest-export-text", "자가검증: export-text", FAIL, cmd, f"exit={code}: {(err or out).strip()[:120]}", True)
    try:
        obj = json.loads(out)
    except json.JSONDecodeError as e:
        return _mk("selftest-export-text", "자가검증: export-text", FAIL, cmd, f"JSON 파싱 실패: {e}", True)
    pages = obj.get("pages") if isinstance(obj, dict) else None
    if not isinstance(pages, list) or len(pages) < 1:
        return _mk("selftest-export-text", "자가검증: export-text", FAIL, cmd, "pages 배열이 비었거나 없음", True)
    chars = sum(len(p.get("text", "")) for p in pages if isinstance(p, dict))
    detail = f"pageCount={obj.get('pageCount')}, pages={len(pages)}, 본문문자={chars}"
    return _mk("selftest-export-text", "자가검증: export-text", PASS, cmd, detail, True)


def _mk(cid, title, status, command, detail, critical, version=None):
    d = {"id": cid, "title": title, "status": status, "command": command, "detail": detail, "critical": critical}
    if version is not None:
        d["version"] = version
    return d


# --------------------------------------------------------------------------- #
# 출력
# --------------------------------------------------------------------------- #
def render_human(report, out):
    p = lambda *a: print(*a, file=out)
    b = report["binary"]
    p("rhwp doctor — 에이전트 제로프릭션 온보딩 점검")
    p(f"repo: {report['repoRoot']}")
    p("")
    p("[1] 바이너리 위치·버전")
    if b["found"]:
        p(f"  [PASS] rhwp 발견: {b['path']}  (source: {b['source']})")
    else:
        p(f"  [FAIL] rhwp 미발견 — 아직 빌드 안 됨. 저장소 루트에서 실행:")
        p(f"           {report['buildCommand']}")
    for c in report["checks"]:
        p(f"  [{c['status']}] {c['title']}: {c['command']}")
        if c["detail"]:
            p(f"           → {c['detail']}")
    p("")
    p(f"[2] 붙여넣기용 .mcp.json  (호스트 프로젝트 루트에 두거나 mcpServers 키를 병합)")
    for line in json.dumps(report["mcpJson"], ensure_ascii=False, indent=2).splitlines():
        p(f"  {line}")
    if report.get("mcpJsonWritten"):
        p(f"  → 기록함: {report['mcpJsonWritten']}")
    p("")
    p("[3] 첫 5분 레시피 지도 (실존 스킬·레시피만 인용)")
    for r in report["recipes"]:
        sflag = "OK" if r["skillExists"] else "missing"
        p(f"  · {r['task']}")
        p(f"      명령: {r['command']}")
        p(f"      스킬: {r['skill']} [{sflag}]  ({r['skillPath']})")
        if r["recipe"]:
            rflag = "OK" if r["recipeExists"] else "missing"
            p(f"      레시피: {r['recipe']} [{rflag}]")
    p("")
    verdict = "정상 — 바로 붙여도 됩니다" if report["ok"] else "미완 — 위 FAIL/빌드 안내를 먼저 처리하세요"
    p(f"판정: {verdict}  (exit={report['exitCode']})")


def _force_utf8_streams():
    """stdout/stderr 를 UTF-8 로 맞춘다. Windows 콘솔(cp949)에서도 한글·em-dash·
    UTF-8 JSON 이 깨지지 않게 한다. 에이전트는 어차피 stdout 을 UTF-8 로 파싱한다."""
    for stream in (sys.stdout, sys.stderr):
        reconfigure = getattr(stream, "reconfigure", None)
        if reconfigure is not None:
            try:
                reconfigure(encoding="utf-8", errors="replace")
            except (ValueError, OSError):
                pass


def main(argv=None) -> int:
    _force_utf8_streams()
    ap = argparse.ArgumentParser(
        prog="rhwp_doctor.py",
        description="rhwp 에이전트 온보딩 닥터 — 바이너리 검증 + 자가검증 + .mcp.json + 레시피 지도",
    )
    ap.add_argument("--json", action="store_true", help="기계 판독용 리포트 JSON 을 stdout 으로")
    ap.add_argument("--write", metavar="PATH", help=".mcp.json 스니펫을 이 경로에 기록(기존 파일은 --force 필요)")
    ap.add_argument("--force", action="store_true", help="--write 시 기존 파일 덮어쓰기 허용")
    ap.add_argument("--rhwp", metavar="PATH", help="rhwp 바이너리 경로를 직접 지정")
    ap.add_argument("--sample", metavar="PATH", help="자가검증에 쓸 샘플 문서 경로")
    ap.add_argument("--repo-root", metavar="PATH", help="저장소 루트(기본: 스크립트 위치에서 유도)")
    args = ap.parse_args(argv)

    # --json 모드: 사람용 텍스트는 stderr, stdout 은 순수 JSON 만.
    human_out = sys.stderr if args.json else sys.stdout

    repo_root = Path(args.repo_root).resolve() if args.repo_root else default_repo_root()
    binary, source, on_path = find_binary(repo_root, args.rhwp)

    checks = []
    if binary is not None:
        checks.append(check_version(binary))
        sample = pick_sample(repo_root, args.sample)
        if sample is None:
            note = "샘플 문서를 찾지 못함(samples/ 없음). --sample 로 지정하세요."
            checks.append(_mk("selftest-info", "자가검증: info", SKIP, "rhwp info <샘플> --json", note, True))
            checks.append(_mk("selftest-export-text", "자가검증: export-text", SKIP, "rhwp export-text <샘플> --json", note, True))
        else:
            checks.append(check_info(binary, sample))
            checks.append(check_export_text(binary, sample))
    else:
        sample = None

    # .mcp.json 스니펫(바이너리 유무와 무관하게 방출 — 문서 산출물).
    if binary is not None and not on_path:
        snippet = build_mcp_snippet(str(binary))
    else:
        snippet = build_mcp_snippet("rhwp")

    ok, exit_code = aggregate(checks, binary is not None)

    report = {
        "schemaVersion": SCHEMA_VERSION,
        "tool": "rhwp_doctor",
        "ok": ok,
        "exitCode": exit_code,
        "repoRoot": str(repo_root),
        "binary": {
            "found": binary is not None,
            "path": str(binary) if binary else None,
            "source": source,
            "onPath": on_path,
            "version": next((c.get("version") for c in checks if c.get("id") == "version" and c.get("version")), None),
        },
        "sample": str(sample) if sample else None,
        "checks": checks,
        "mcpJson": snippet,
        "mcpJsonWritten": None,
        "recipes": resolve_recipe_map(repo_root),
        "buildCommand": BUILD_COMMAND,
    }

    # --write 처리(덮어쓰기 보호).
    if args.write:
        target = Path(args.write)
        if target.exists() and not args.force:
            print(f"경고: {target} 가 이미 있어 기록하지 않았습니다. 덮어쓰려면 --force 를 주세요.", file=sys.stderr)
            _emit(report, args.json, human_out)
            return 2
        try:
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(json.dumps(snippet, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            report["mcpJsonWritten"] = str(target)
        except OSError as e:
            print(f"경고: {target} 기록 실패: {e}", file=sys.stderr)
            _emit(report, args.json, human_out)
            return 2

    _emit(report, args.json, human_out)
    return exit_code


def _emit(report, as_json, human_out):
    if as_json:
        print(json.dumps(report, ensure_ascii=False, indent=2))
    else:
        render_human(report, human_out)


if __name__ == "__main__":
    sys.exit(main())
