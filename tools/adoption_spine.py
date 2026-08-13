"""채택 척추 가드 — 에이전트 작업 표준(AWS)이 여러 표면에서 정합한가 (#4715).

## 왜 필요한가

트랙 L(채택 중력)은 "모든 표면이 같은 축으로 이어지는가"를 묻는다. 표면은
늘어나는데(규약 파일 22종·gym·로드맵·표준 정본 2종) 그 이어짐을 사람이 손으로
지키면 조용히 끊긴다 — 규약 파일 하나를 고치며 표준 링크를 빠뜨리면 그 도구의
에이전트는 축을 못 만난다. 이 가드는 **표면이 표준을 일관되게 가리키는지**를 매
CI 마다 확인한다. 파일 작업이 곧 척추 유지가 되게 하는 능동 장치다.

## 무엇을 검사하나 (바이너리 불요 — 커밋된 문서만 본다)

1. 기계용 정본(json)이 스키마를 지킨다: standard·version·levels 5개(AW-L1..L5)·
   surfaces·legitimacy.
2. 사람용 정본(md)과 기계용 정본이 **같은 레벨**을 말한다(AW-L1..L5 전부 등장).
3. json 이 선언한 surfaces 가 전부 실재한다(파일, `#anchor` 는 파일까지 검사).
4. AGENTS.md 작업 증빙 절이 표준을 가리킨다(척추의 뿌리).
5. 트랙 L 문서가 표준을 가리킨다(로드맵 정합).

사용:
  python tools/adoption_spine.py          # 검증 — 리포트 + exit 0/1
  python tools/adoption_spine.py --json    # 리포트를 JSON 으로
"""

from __future__ import annotations

import argparse
import io
import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
STD_JSON = REPO_ROOT / "mydocs" / "tech" / "standards" / "agent_work_standard.json"
STD_MD = REPO_ROOT / "mydocs" / "tech" / "standards" / "agent_work_standard.md"

LEVEL_IDS = ["AW-L1", "AW-L2", "AW-L3", "AW-L4", "AW-L5"]
MARKDOWN_LINK_RE = re.compile(r"\[[^\]]+\]\(([^)\s]+)(?:\s+[^)]*)?\)")
MARKDOWN_HEADING_RE = re.compile(r"^#{1,6}\s+(.+?)(?:\s+#+)?\s*$")


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def _points_to_standard(surface: Path, text: str) -> bool:
    """surface의 실제 Markdown 링크가 사람용 정본을 가리키는지 확인한다."""
    standard = STD_MD.resolve()
    for destination in MARKDOWN_LINK_RE.findall(text):
        destination = destination.strip("<>").split("#", 1)[0]
        if not destination or "://" in destination or destination.startswith("#"):
            continue
        if (surface.parent / destination).resolve() == standard:
            return True
    return False


def _github_heading_anchor(heading: str) -> str:
    """이 저장소에서 쓰는 일반 Markdown heading의 GitHub anchor를 만든다."""
    return "".join(
        char.lower() if char.isalnum() else char if char in "- " else ""
        for char in heading
    ).replace(" ", "-")


def _has_declared_anchor(text: str, anchor: str) -> bool:
    for line in text.splitlines():
        match = MARKDOWN_HEADING_RE.match(line)
        if match and _github_heading_anchor(match.group(1)) == anchor:
            return True
    return False


def check() -> list[str]:
    """정합 위반 목록을 반환한다(빈 목록 = 척추 온전)."""
    problems: list[str] = []

    if not STD_JSON.is_file():
        return [f"기계용 정본 없음: {STD_JSON.relative_to(REPO_ROOT)}"]
    try:
        spec = json.loads(_read(STD_JSON))
    except ValueError as e:
        return [f"기계용 정본 JSON 파싱 실패: {e}"]

    # 1) 스키마
    for key in ("standard", "abbrev", "version", "levels", "surfaces", "legitimacy"):
        if key not in spec:
            problems.append(f"기계용 정본에 필수 키 없음: {key}")
    levels = spec.get("levels", [])
    if not isinstance(levels, list):
        problems.append("기계용 정본의 levels 가 목록이 아니다")
        levels = []
    got_ids = [lvl.get("id") if isinstance(lvl, dict) else None for lvl in levels]
    if got_ids != LEVEL_IDS:
        problems.append(f"레벨 id 가 {LEVEL_IDS} 가 아니다: {got_ids}")
    for lvl in levels:
        if not isinstance(lvl, dict):
            problems.append(f"레벨 항목이 객체가 아니다: {lvl!r}")
            continue
        for key in ("id", "name", "requires", "referenceCommand"):
            if not lvl.get(key):
                problems.append(f"레벨 {lvl.get('id')} 에 {key} 가 비었다")

    # 2) 사람용 ↔ 기계용 정합 — md 가 버전·전 레벨을 말하는가
    if not STD_MD.is_file():
        problems.append(f"사람용 정본 없음: {STD_MD.relative_to(REPO_ROOT)}")
    else:
        md = _read(STD_MD)
        version = spec.get("version", "")
        if version and version not in md:
            problems.append(f"사람용 정본이 버전 {version} 을 말하지 않는다")
        for lid in LEVEL_IDS:
            if lid not in md:
                problems.append(f"사람용 정본에 레벨 {lid} 서술이 없다")

    # 3) 선언한 surfaces 가 실재하고 실제 정본 링크·선언 anchor를 지키는가
    surfaces = spec.get("surfaces", [])
    if not isinstance(surfaces, list):
        problems.append("기계용 정본의 surfaces 가 목록이 아니다")
        surfaces = []
    for surf in surfaces:
        if not isinstance(surf, str):
            problems.append(f"표면 경로가 문자열이 아니다: {surf!r}")
            continue
        rel, separator, anchor = surf.partition("#")
        surface = REPO_ROOT / rel
        if not surface.is_file():
            problems.append(f"표준이 선언한 표면이 실재하지 않는다: {surf}")
            continue
        content = _read(surface)
        if surface.resolve() != STD_MD.resolve() and not _points_to_standard(surface, content):
            problems.append(f"표면이 사람용 정본을 가리키지 않는다: {surf}")
        if separator and not _has_declared_anchor(content, anchor):
            problems.append(f"표면에 선언한 anchor가 없다: {surf}")

    return problems


def main() -> int:
    ap = argparse.ArgumentParser(description="채택 척추 가드 (AWS 표면 정합)")
    ap.add_argument("--json", action="store_true", help="리포트를 JSON 으로")
    a = ap.parse_args()

    problems = check()
    surfaces = []
    if STD_JSON.is_file():
        try:
            surfaces = json.loads(_read(STD_JSON)).get("surfaces", [])
        except ValueError:
            pass

    if a.json:
        report = {"kind": "adoptionSpineReport", "schemaVersion": "1.0",
                  "surfaces": len(surfaces), "problems": problems,
                  "intact": not problems}
        sys.stdout.write(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    else:
        print(f"채택 척추 — 표면 {len(surfaces)}개 정합 검사")
        if problems:
            for p in problems:
                print(f"  X {p}")
            print(f"→ 정합 위반 {len(problems)}건 (척추 끊김)")
        else:
            print("→ 전부 통과 — 표준이 전 표면에서 일관되게 가리켜진다(척추 온전)")
    return 0 if not problems else 1


if __name__ == "__main__":
    # Windows 콘솔 한글 안전 출력
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
