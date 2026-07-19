#!/usr/bin/env python3
"""e2e MANIFEST 대조 검사 (#2353).

로컬 실행 원칙 (CI 미편입 — check_markdown_links 와 동일 운용):

1. 양방향 대조: git tracked `rhwp-studio/e2e/*.mjs` ↔ MANIFEST 행
   - 미등재 파일 = FAIL (신규 추가 시 등재 의무 — 원장 트립와이어 방식)
   - 유령 행(파일 없음) = FAIL
2. 명명 정합: 분류 `상시` 는 `.test.mjs`, `진단` 은 `probe-`/`debug-` 접두 —
   비고 `legacy-name` 표기 행은 면제 (기존 파일 보수 원칙, 신규만 강제)
3. 배선 실재: rhwp-studio/package.json 의 e2e script 와 .github/workflows 가
   가리키는 e2e 파일이 실재하는지 검증 — 없는 파일명 실행이 공허 통과로
   집계되는 결함 재발 방지 (#2353 1단계 발견)
"""
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
E2E = ROOT / "rhwp-studio" / "e2e"
MANIFEST = E2E / "MANIFEST.md"

errors: list[str] = []


def tracked_files() -> set[str]:
    out = subprocess.run(
        ["git", "ls-files", "rhwp-studio/e2e/"],
        capture_output=True, text=True, cwd=ROOT, check=True,
    ).stdout.split()
    return {Path(f).name for f in out if f.endswith(".mjs")}


PLACEHOLDER = {"", "—", "-"}  # 빈칸/생략 기호 — 결손으로 집계 (파싱 실패 아님)

COLUMNS = ("file", "cat", "state", "use", "samples", "wiring", "note")


def manifest_rows() -> dict[str, dict]:
    """MANIFEST 표를 7열 전체로 명시 파싱한다.

    - 셀은 strip() 후 그대로 보존 — `—`(생략)·`/`(구분)·`·` 등 기호를 파괴하지
      않고 메타데이터로 유지한다 (향후 검증의 정밀도 확보).
    - `.mjs` 백틱을 포함하는데 열 수가 7이 아닌 행은 malformed 로 FAIL —
      느슨한 regex 가 행을 조용히 건너뛰어 가짜 "미등재" 오류를 만드는 것을
      방지한다.
    """
    rows: dict[str, dict] = {}
    for lineno, line in enumerate(MANIFEST.read_text(encoding="utf-8").splitlines(), 1):
        if not line.startswith("|") or "`" not in line:
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        m = re.match(r"`([^`]+\.mjs)`", cells[0]) if cells else None
        if not m:
            continue
        if len(cells) != len(COLUMNS):
            errors.append(
                f"malformed 행(L{lineno}): {cells[0]} — 열 {len(cells)}개 (기대 {len(COLUMNS)})"
            )
            continue
        name = m.group(1)
        row = dict(zip(COLUMNS, cells))
        row["file"] = name
        rows[name] = row
    return rows


def check_bidirectional(files: set[str], rows: dict[str, dict]) -> None:
    for f in sorted(files):
        if f not in rows:
            errors.append(f"미등재 파일: {f} — MANIFEST.md 에 행을 추가하세요")
    for name in sorted(rows):
        if name not in files:
            errors.append(f"유령 행: {name} — 파일이 없거나 untracked (행 제거 또는 파일 커밋)")


def check_naming(rows: dict[str, dict]) -> None:
    for name, r in sorted(rows.items()):
        if "legacy-name" in r["note"]:
            continue
        if r["cat"] == "상시" and not name.endswith(".test.mjs"):
            errors.append(f"명명 위반(상시): {name} — .test.mjs 필수 (기존 파일이면 비고 legacy-name)")
        if r["cat"] == "진단" and not (name.startswith("probe-") or name.startswith("debug-")):
            errors.append(f"명명 위반(진단): {name} — probe-/debug- 접두 필수 (기존 파일이면 legacy-name)")
        if r["cat"] not in ("상시", "진단", "유틸"):
            errors.append(f"분류 열거 위반: {name} — '{r['cat']}'")
        if r["state"] not in ("active", "hold", "deprecated"):
            errors.append(f"상태 열거 위반: {name} — '{r['state']}'")


def check_wiring(files: set[str], rows: dict[str, dict]) -> None:
    pkg = json.loads((ROOT / "rhwp-studio" / "package.json").read_text(encoding="utf-8"))
    npm_targets: dict[str, set[str]] = {}
    for script_name, cmd in pkg.get("scripts", {}).items():
        for m in re.finditer(r"e2e/([a-zA-Z0-9._-]+\.mjs)", cmd):
            npm_targets.setdefault(m.group(1), set()).add(script_name)
            if m.group(1) not in files:
                errors.append(f"배선 실재 위반: npm script '{script_name}' → e2e/{m.group(1)} 부재")
    ci_targets: set[str] = set()
    for wf in (ROOT / ".github" / "workflows").glob("*.yml"):
        for m in re.finditer(r"e2e/([a-zA-Z0-9._-]+\.mjs)", wf.read_text(encoding="utf-8")):
            ci_targets.add(m.group(1))
            if m.group(1) not in files:
                errors.append(f"배선 실재 위반: {wf.name} → e2e/{m.group(1)} 부재")

    # 배선 열 ↔ 실제 배선 교차 대조 (표가 주장하는 배선이 실재하는지, 역도 성립하는지)
    for name, r in sorted(rows.items()):
        w = r["wiring"]
        claims_npm = w.startswith("npm")
        claims_ci = "CI" in w
        if claims_npm and name not in npm_targets:
            errors.append(f"배선 열 불일치: {name} — 표는 '{w}' 주장하나 npm script 에 없음")
        if claims_ci and name not in ci_targets:
            errors.append(f"배선 열 불일치: {name} — 표는 '{w}' 주장하나 워크플로에 없음")
        if not claims_npm and not claims_ci and w not in PLACEHOLDER and w != "수동":
            errors.append(f"배선 열 열거 위반: {name} — '{w}' (npm …/CI/npm+CI/수동)")
        if name in npm_targets and not claims_npm:
            errors.append(f"배선 열 불일치: {name} — npm script({', '.join(sorted(npm_targets[name]))}) 배선인데 표는 '{w}'")
        if name in ci_targets and not claims_ci:
            errors.append(f"배선 열 불일치: {name} — 워크플로 배선인데 표는 '{w}'")


def report_gaps(rows: dict[str, dict]) -> None:
    """결손(빈칸·생략 기호) 집계 — FAIL 아님, 정보 출력."""
    empty_use = [n for n, r in sorted(rows.items()) if r["use"] in PLACEHOLDER]
    if empty_use:
        print(f"용도 결손 {len(empty_use)}건 (채움 권장): {', '.join(empty_use)}")


def main() -> int:
    files = tracked_files()
    rows = manifest_rows()
    check_bidirectional(files, rows)
    check_naming(rows)
    check_wiring(files, rows)
    report_gaps(rows)
    print(f"e2e 파일(tracked): {len(files)}개 / MANIFEST 행: {len(rows)}개")
    if errors:
        print(f"e2e manifest 오류: {len(errors)}건")
        for e in errors:
            print(f"- {e}")
        return 1
    print("e2e manifest: 이상 없음")
    return 0


if __name__ == "__main__":
    sys.exit(main())
