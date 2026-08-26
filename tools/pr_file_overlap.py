#!/usr/bin/env python3
"""열린 PR 사이의 파일 충돌 사전 감지 — 손대기 전에 "이미 누가 만지고 있나"를 본다.

## 왜 있는가

여러 에이전트가 동시에 이 저장소에 기여할 때, 서로 다른 이슈를 잡은 두 PR이
우연히 **같은 파일**을 건드리면 나중에 머지하는 쪽이 충돌·재작업을 떠안는다.
`tools/agent_preflight.py` 의 큐 규율 검사(§11)는 이미 "동일 이슈 중복 PR"과
"미할당 착수"를 잡아 주지만, 그 검사는 **의도적으로** 파일 겹침 추정을 쓰지
않는다 — 스크립트 안 주석 원문:

    "근거는 선언만 쓴다 … 파일 겹침 추정은 쓰지 않는다(오탐이 정상 작업을 막는다)"

즉 "다른 이슈인데 같은 파일을 건드리는 PR들"은 지금 아무 도구도 보지 않는
사각지대다. 이 도구는 그 사각지대만 메운다 — agent_preflight 의 하드 게이트를
대체하지 않고, **참고용 조회 도구**로 옆에 둔다(기본은 정보 출력, 종료 코드에
영향 없음. 명시적으로 `--fail-on-overlap` 을 줘야 게이트가 된다).

## 무엇을 하나

1. `gh pr list --json files` **한 번**으로 열린 PR 전체의 변경 파일 목록을 받는다
   (PR마다 별도 API 호출 없음 — 저장소 규모와 무관하게 빠르다).
2. 파일 경로 → PR 목록 역인덱스를 만들어, 2개 이상의 PR이 같은 경로를 건드리면
   보고한다.
3. 이제 막 작업을 시작하려는 파일 목록을 주면(`--check <경로...>`), 그 파일들만
   골라 겹침이 있는지 확인한다 — "이 파일 손대기 전에 charging 이 이미 있나" 체크.

## 쓰는 법

    python3 tools/pr_file_overlap.py                              # 전체 겹침 스캔
    python3 tools/pr_file_overlap.py --json
    python3 tools/pr_file_overlap.py --check src/main.rs mydocs/x.md   # 시작 전 체크
    python3 tools/pr_file_overlap.py --check src/main.rs --fail-on-overlap  # 게이트로 쓸 때
    python3 tools/pr_file_overlap.py --repo kevin9327/rhwp         # 다른 저장소로

인증은 `gh` CLI(`gh auth status`)에 위임한다. Python 3 표준 라이브러리만
사용한다(agent-toolkit 관례) — `gh` 는 서브프로세스로만 부른다.

## 알아둘 것 — 오탐/누락

- **정적 스냅샷**이다. 호출 시점 이후 새 PR·새 커밋은 반영되지 않는다 — 매번
  다시 부르는 것이 계약이다(agent_preflight 의 네트워크 검사와 같은 태도).
- 파일 **경로 문자열**이 같으면 겹침으로 본다. 같은 파일의 다른 줄을 건드리는
  PR은 실제로 머지 충돌이 안 날 수도 있다(반대도 성립: 경로가 달라도 같은
  섹션을 참조하는 문서가 의미상 충돌할 수 있다) — 최종 판단은 사람/에이전트 몫이다.
- fork PR은 `headRefName` 만 보이고 어느 fork 인지는 `gh pr view <N>` 으로 따로
  확인해야 한다(이 도구는 fork 소유자를 추적하지 않는다).

종료 코드: 0 정상 조회(겹침 유무와 무관, `--fail-on-overlap` 미지정 시)
/ 1 gh 실행·네트워크 실패 / 2 사용법 오류
/ 3 겹침 발견 + `--fail-on-overlap` 지정(오류가 아니라 검증 판정 — 이 저장소의
  exit 3 관례, `mydocs/manual/cli_commands.md` 의 종료 코드 절 참고)
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys

DEFAULT_REPO = "edwardkim/rhwp"
SCHEMA_VERSION = "1.0"


def fetch_open_prs(repo: str, limit: int) -> list[dict]:
    r = subprocess.run(
        [
            "gh", "pr", "list",
            "--repo", repo,
            "--state", "open",
            "--limit", str(limit),
            "--json", "number,title,headRefName,author,isDraft,files",
        ],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=120,
    )
    if r.returncode != 0:
        raise RuntimeError(r.stderr.strip() or "gh pr list 실패")
    return json.loads(r.stdout or "[]")


def build_index(prs: list[dict]) -> dict[str, list[dict]]:
    """파일 경로 -> 그 파일을 건드리는 PR 요약 목록."""
    index: dict[str, list[dict]] = {}
    for pr in prs:
        summary = {
            "number": pr["number"],
            "title": pr.get("title", ""),
            "headRefName": pr.get("headRefName", ""),
            "author": (pr.get("author") or {}).get("login", ""),
            "isDraft": bool(pr.get("isDraft")),
        }
        for f in pr.get("files") or []:
            path = f.get("path")
            if not path:
                continue
            index.setdefault(path, []).append(summary)
    return index


def find_overlaps(index: dict[str, list[dict]]) -> dict[str, list[dict]]:
    return {path: prs for path, prs in index.items() if len(prs) >= 2}


def print_human(overlaps: dict[str, list[dict]], scope_label: str) -> None:
    if not overlaps:
        print(f"{scope_label}: 겹침 없음.")
        return
    print(f"{scope_label}: {len(overlaps)}개 파일에서 열린 PR이 겹칩니다.")
    for path, prs in sorted(overlaps.items()):
        who = ", ".join(f"#{p['number']}({p['author'] or '?'}) {p['headRefName']}" for p in prs)
        print(f"  {path}")
        print(f"    -> {who}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--repo", default=DEFAULT_REPO, help="owner/repo (기본 %(default)s)")
    parser.add_argument("--limit", type=int, default=200, help="조회할 열린 PR 상한")
    parser.add_argument(
        "--check",
        nargs="+",
        metavar="PATH",
        help="이 파일들만 겹침 여부를 확인한다(작업 시작 전 체크). 생략하면 전체 스캔",
    )
    parser.add_argument("--json", action="store_true", help="기계 판독용 JSON 출력")
    parser.add_argument(
        "--fail-on-overlap",
        action="store_true",
        help="겹침이 있으면 exit 3 (기본은 정보 출력만, 종료 코드에 영향 없음)",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    for stream in (sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(encoding="utf-8", errors="replace")

    parser = build_parser()
    args = parser.parse_args(argv)

    if args.limit <= 0:
        print("오류: --limit 은 1 이상이어야 합니다.", file=sys.stderr)
        return 2

    try:
        prs = fetch_open_prs(args.repo, args.limit)
    except RuntimeError as e:
        print(f"오류: {e}", file=sys.stderr)
        return 1
    except (subprocess.TimeoutExpired, OSError, json.JSONDecodeError) as e:
        print(f"오류: gh 호출 실패 - {e}", file=sys.stderr)
        return 1

    index = build_index(prs)

    if args.check:
        checked = {}
        for path in args.check:
            hits = index.get(path, [])
            if len(hits) >= 2:
                checked[path] = hits
        scope_overlaps = checked
        scope_label = f"체크한 {len(args.check)}개 파일"
        checked_all = {path: index.get(path, []) for path in args.check}
    else:
        scope_overlaps = find_overlaps(index)
        scope_label = f"열린 PR {len(prs)}건 전체"
        checked_all = None

    if args.json:
        payload = {
            "schemaVersion": SCHEMA_VERSION,
            "repo": args.repo,
            "openPrCount": len(prs),
            "overlaps": {
                path: [{"number": p["number"], "title": p["title"],
                         "headRefName": p["headRefName"], "author": p["author"],
                         "isDraft": p["isDraft"]} for p in prs_]
                for path, prs_ in scope_overlaps.items()
            },
        }
        if checked_all is not None:
            payload["checked"] = {
                path: [{"number": p["number"], "headRefName": p["headRefName"]} for p in prs_]
                for path, prs_ in checked_all.items()
            }
        print(json.dumps(payload, ensure_ascii=False))
    else:
        print_human(scope_overlaps, scope_label)

    if args.fail_on_overlap and scope_overlaps:
        return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
