"""에이전트 프레임 메타 가드 — 틀의 불변식을 강제한다 (#4726).

## 왜 필요한가

rhwp 에이전트 시스템은 여러 하위체계(gym·로드맵·표준·조직·planner·MCP)로
자란다. 각각은 자기 가드가 있지만, **틀 전체의 불변식**을 지키는 것이 없었다:

> 프레임에 등재된 모든 하위체계는 CI 가드를 하나 이상 가진다.

이 메타 가드가 그 불변식을 매 CI 로 강제한다. 그래서 확장은 **틀 안에서만**
일어난다 — 가드 없는 하위체계는 프레임에 못 들어오고, 머지된 하위체계는 핵심
파일·가드가 실재해야 한다. 세부는 생태계가 채우되, 프레임을 벗어나지 못한다.

## 무엇을 검사하나 (커밋된 것만 — 바이너리 불요)

1. `frame.json` 스키마: kind·invariant·subsystems·openSlots·legitimacy.
2. **불변식**: 모든 하위체계가 `guards` 를 하나 이상 선언한다(가드 없는 등재 금지).
3. `status: merged` 하위체계: keyFile 과 가드 중 하나 이상이 저장소에 실재한다.
4. `status: in-flight` 하위체계: `pr` 번호(정수)가 기록돼 있다.

사용:
  python tools/frame_guard.py           # 검증 — 리포트 + exit 0/1
  python tools/frame_guard.py --json      # 리포트를 JSON 으로
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
FRAME = REPO_ROOT / "mydocs" / "tech" / "agent_frame" / "frame.json"


def check() -> list[str]:
    problems: list[str] = []
    if not FRAME.is_file():
        return [f"프레임 레지스트리 없음: {FRAME.relative_to(REPO_ROOT)}"]
    try:
        frame = json.loads(FRAME.read_text(encoding="utf-8"))
    except ValueError as e:
        return [f"frame.json 파싱 실패: {e}"]

    for key in ("kind", "invariant", "subsystems", "openSlots", "legitimacy"):
        if key not in frame:
            problems.append(f"frame.json 필수 키 없음: {key}")
    if frame.get("kind") != "agentFrame":
        problems.append("kind 가 agentFrame 이 아니다")

    seen = set()
    for sub in frame.get("subsystems", []):
        sid = sub.get("id", "?")
        for key in ("id", "name", "status", "guards"):
            if not sub.get(key):
                problems.append(f"[{sid}] 하위체계에 {key} 가 비었다")
        if sid in seen:
            problems.append(f"[{sid}] 하위체계 id 중복")
        seen.add(sid)
        # 불변식 — 가드 없는 하위체계는 프레임에 들어올 수 없다.
        guards = sub.get("guards") or []
        if not guards:
            problems.append(f"[{sid}] 가드가 없다 — 프레임 불변식 위반(가드 없는 등재 금지)")
        status = sub.get("status")
        if status == "merged":
            key_ok = bool(sub.get("keyFile")) and (REPO_ROOT / sub["keyFile"]).exists()
            guard_ok = any((REPO_ROOT / g).exists() for g in guards)
            if not key_ok:
                problems.append(f"[{sid}] merged 인데 keyFile 이 실재하지 않는다: {sub.get('keyFile')}")
            if not guard_ok:
                problems.append(f"[{sid}] merged 인데 선언한 가드가 하나도 실재하지 않는다: {guards}")
        elif status == "in-flight":
            if not isinstance(sub.get("pr"), int):
                problems.append(f"[{sid}] in-flight 인데 pr 번호(정수)가 없다")
        else:
            problems.append(f"[{sid}] status 는 merged|in-flight 여야 한다: {status}")
    return problems


def main() -> int:
    ap = argparse.ArgumentParser(description="에이전트 프레임 메타 가드 (#4726)")
    ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    problems = check()
    n = 0
    if FRAME.is_file():
        try:
            n = len(json.loads(FRAME.read_text(encoding="utf-8")).get("subsystems", []))
        except ValueError:
            pass
    if a.json:
        sys.stdout.write(json.dumps(
            {"kind": "agentFrameReport", "schemaVersion": "1.0",
             "subsystems": n, "problems": problems, "intact": not problems},
            ensure_ascii=False, indent=2) + "\n")
    else:
        print(f"에이전트 프레임 — 하위체계 {n}개 불변식 검사")
        if problems:
            for p in problems:
                print(f"  X {p}")
            print(f"→ 불변식 위반 {len(problems)}건 (틀이 깨졌다)")
        else:
            print("→ 전부 통과 — 모든 하위체계가 가드를 가지고, 머지분은 실재한다(틀 온전)")
    return 0 if not problems else 1


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
