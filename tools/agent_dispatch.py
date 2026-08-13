"""에이전트 자동 배치기 — 접수 이력서 → 부서 배정 + 첫 과제 + 승진 경로 (#4720).

## 무엇인가

큰 기업의 시스템이 스스로 굴러가듯, 새 에이전트가 오면 사람 손 배정 없이 이
도구가 **어느 부서에서 무엇부터 하는지**를 자동으로 정해준다. 접수(이력서) →
자동 배치 → (에이전트가 산출) → 검증된 산출로 자동 승진.

배치는 실재 자산 위에서 돈다: 부서표(`departments.json`)·gym 과제·검증 사다리.
아무 진실도 지어내지 않는다 — 배정된 과제는 실재하고, 승진 기준은 검증된 산출이다.

## 접수 이력서(intake manifest) 형식

```json
{
  "agent": "너의-이름",
  "tools": ["claude-code", "mcp"],
  "targetDepartment": "editing",   // 또는 "any"/생략 → 접수처
  "awsLevel": "AW-L1"              // 현재 직급(생략 시 신입)
}
```

사용:
  python tools/agent_dispatch.py --manifest intake.json
  echo '{"agent":"claude","targetDepartment":"verification"}' | python tools/agent_dispatch.py
  python tools/agent_dispatch.py --agent claude --department editing --json
"""

from __future__ import annotations

import argparse
import io
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
ORG = REPO_ROOT / "mydocs" / "tech" / "agent_org" / "departments.json"
PACKS = REPO_ROOT / "gym" / "packs"


def load_org():
    return json.loads(ORG.read_text(encoding="utf-8"))


def _first_task(pack_id: str) -> str | None:
    tdir = PACKS / pack_id / "tasks"
    tasks = sorted(p.stem for p in tdir.glob("*.json")) if tdir.is_dir() else []
    return f"{pack_id}/{tasks[0]}" if tasks else None


def pick_department(org: dict, target: str | None) -> dict:
    depts = {d["id"]: d for d in org["departments"]}
    if target and target != "any" and target in depts:
        return depts[target]
    # 미지정/any → 접수처(온보딩). 접수처가 없으면 첫 부서.
    return depts.get("reception", org["departments"][0])


def entry_task(dept: dict) -> str | None:
    """부서의 입사 과제 — 선언값이 실재하면 그것, 아니면 소유 pack 첫 과제."""
    declared = dept.get("entryTask")
    if declared:
        pid, _, tid = declared.partition("/")
        if (PACKS / pid / "tasks" / f"{tid}.json").is_file():
            return declared
    for pid in dept.get("packs", []):
        got = _first_task(pid)
        if got:
            return got
    return None


def next_rung(org: dict, aws_level: str | None) -> dict | None:
    ladder = org["careerLadder"]
    if not aws_level:
        return ladder[0]
    for i, rung in enumerate(ladder):
        if rung["aws"] == aws_level:
            return ladder[i + 1] if i + 1 < len(ladder) else None
    return ladder[0]


def dispatch(manifest: dict) -> dict:
    org = load_org()
    dept = pick_department(org, manifest.get("targetDepartment"))
    task = entry_task(dept)
    current = manifest.get("awsLevel")
    cur_rung = next((r for r in org["careerLadder"] if r["aws"] == current), None)
    promote = next_rung(org, current)
    # 직급을 신고하지 않았으면 지원자 — 첫 승진(AW-L1 영수증)으로 신입이 된다.
    current_level = cur_rung["level"] if cur_rung else "지원자"

    assignment = {
        "kind": "agentAssignment", "schemaVersion": "1.0",
        "agent": manifest.get("agent", "익명-에이전트"),
        "department": {"id": dept["id"], "name": dept["name"], "mission": dept["mission"]},
        "currentLevel": current_level,
        "startTask": task,
        "profile": dept.get("profile"),
        "commands": [],
        "promotion": None,
    }
    if task:
        assignment["commands"] = [
            f"python gym/score.py --agent {assignment['agent']} --pack {task.split('/')[0]}",
            # 작업을 증명으로 — AW-L1 영수증
            f"rhwp replay <너의 계획> --capsule work.capsule.json --json",
        ]
    elif dept.get("service"):
        assignment["commands"] = [f"# 서비스 부서 — 도구: {dept.get('tool')}"]
    if promote:
        assignment["promotion"] = {
            "toLevel": promote["level"], "toAws": promote["aws"],
            "when": promote["promoteWhen"],
        }
    return assignment


def render(a: dict) -> str:
    lines = [
        f"┌─ 배치 결과 — {a['agent']}",
        f"│  부서: {a['department']['name']} ({a['department']['id']})",
        f"│  사명: {a['department']['mission']}",
        f"│  직급: {a['currentLevel']}",
    ]
    if a.get("startTask"):
        lines.append(f"│  입사 과제: {a['startTask']}" + (f" (프로파일 {a['profile']})" if a.get("profile") else ""))
    for c in a.get("commands", []):
        lines.append(f"│    $ {c}")
    if a.get("promotion"):
        p = a["promotion"]
        lines.append(f"│  다음 승진: {p['toLevel']} ({p['toAws']}) — {p['when']}")
    lines.append("└─ 시스템이 자동으로 배치했다. 준거는 검증 사다리다.")
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser(description="에이전트 자동 배치기 (#4720)")
    ap.add_argument("--manifest", help="접수 이력서 JSON 파일")
    ap.add_argument("--agent", help="이름(이력서 대신 인라인)")
    ap.add_argument("--department", help="목표 부서 id (또는 any)")
    ap.add_argument("--aws-level", help="현재 AWS 직급 (예: AW-L1)")
    ap.add_argument("--json", action="store_true", help="결과를 JSON 으로")
    a = ap.parse_args()

    if a.manifest:
        manifest = json.loads(Path(a.manifest).read_text(encoding="utf-8"))
    elif not sys.stdin.isatty():
        raw = sys.stdin.read().strip()
        manifest = json.loads(raw) if raw else {}
    else:
        manifest = {}
    if a.agent:
        manifest["agent"] = a.agent
    if a.department:
        manifest["targetDepartment"] = a.department
    if a.aws_level:
        manifest["awsLevel"] = a.aws_level

    assignment = dispatch(manifest)
    if a.json:
        sys.stdout.write(json.dumps(assignment, ensure_ascii=False, indent=2) + "\n")
    else:
        print(render(assignment))
    return 0


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")  # type: ignore[attr-defined]
        except Exception:
            pass
    sys.exit(main())
