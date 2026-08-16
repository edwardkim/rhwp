#!/usr/bin/env python3
"""git sparse-checkout 프리셋 — 작업 종류별 디렉터리 세트를 한 번에 적용한다.

## 왜 있는가

이 저장소는 1.4GB+ 규모이고, 일부 개발 환경은 대용량 git pack 전송이 구조적으로
불안정하다(`tools/gh_noclone.py` 참고 — `fetch-pack: invalid index-pack output`,
실측 전송 속도 약 1MB/s). 그래서 `git clone`/전체 `checkout` 대신
`git sparse-checkout add <경로...>` 로 필요한 하위 디렉터리만 그때그때 받는데,
같은 작업 종류(파서 회귀, 렌더 검증, 문서 작업, 에이전트 도구 작업 …)를 할 때마다
같은 디렉터리 조합을 손으로 다시 타이핑하게 된다 — 2026-08-16 세션에서 이 조합을
하루 동안 여러 번 반복했다.

이 도구는 그 반복을 없앤다: 작업 종류(`--task`) 하나로 검증된 디렉터리 세트를
한 번에 추가한다. **추가만 한다** — `git sparse-checkout add` 는 이미 받은 것을
지우지 않으므로, 프리셋을 잘못 골라도 되돌릴 필요가 없다(다만 디스크·전송량은
늘어난다. 줄이려면 `git sparse-checkout set <원하는 경로...>` 를 직접 쓴다 — 이
도구는 `set` 을 대신 실행하지 않는다).

## 프리셋이 정답이 아닌 이유

프리셋은 "이 종류의 작업엔 보통 이게 필요하더라"는 실측 경험칙이지, 빌드가 검증한
최소 집합이 아니다. 모자라면 프리셋 위에 그때그때
`git sparse-checkout add <추가 경로>` 를 얹으면 된다 — 이 도구는 그 출발점을
빠르게 줄 뿐이다. 최상위 디렉터리 목록은 `gh api repos/edwardkim/rhwp/contents?ref=devel`
로 다시 확인할 수 있다(새 디렉터리가 생기면 이 목록이 낡을 수 있다).

## 쓰는 법

    python3 tools/sparse_clone_hint.py --list                  # 프리셋 목록 + 근거
    python3 tools/sparse_clone_hint.py --task parser            # 미리보기(기본, 실행 안 함)
    python3 tools/sparse_clone_hint.py --task parser --apply    # 실제로 add 실행
    python3 tools/sparse_clone_hint.py --task parser,studio --apply   # 여러 프리셋 합치기
    python3 tools/sparse_clone_hint.py --task parser --json     # 기계 판독용 미리보기

Python 3 표준 라이브러리만 사용한다(agent-toolkit 관례, `mydocs/manual/agent_toolkit_cli.md`
와 같은 계열).

종료 코드: 0 성공(미리보기 또는 적용 완료) / 1 git 실행 실패
/ 2 사용법 오류(알 수 없는 프리셋 이름, `--task` 누락 등)
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
SCHEMA_VERSION = "1.0"

# 모든 프리셋에 공통으로 얹는 최소 진입점 — CLAUDE.md 의 "로딩 순서"가 가리키는
# 뿌리 파일들과 에이전트 설정이다. 전부 작아서 항상 받아도 부담이 없다.
ALWAYS = [
    ".claude",
    "AGENTS.md",
    "CLAUDE.md",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "rustfmt.toml",
]

# 근거: 2026-08-16 세션 실측 + `gh api repos/edwardkim/rhwp/contents?ref=devel` 로 확인한
# 최상위 디렉터리 목록. 각 프리셋의 "note" 는 왜 그 조합인지를 남긴다 — 이름만으로는
# 다음 에이전트가 "표본이 왜 필요했지"를 알 수 없다.
PRESETS: dict[str, dict] = {
    "core-rust": {
        "paths": ["src", "tests"],
        "note": "엔진 코어 컴파일·단위테스트. 표본·자산 없이 순수 로직 작업할 때.",
    },
    "parser": {
        "paths": ["src", "tests", "samples", "saved", "mydocs/tech", "mydocs/manual"],
        "note": (
            "파서 회귀 — 실제 HWP/HWPX 표본(samples)과 저장 왕복 픽스처(saved), "
            "그리고 파서 아키텍처/포맷 문서(mydocs/tech)·CLI 계약(mydocs/manual)."
        ),
    },
    "renderer": {
        "paths": ["src", "tests", "samples", "saved", "assets", "ttfs", "mydocs/tech", "tools"],
        "note": (
            "레이아웃·렌더 회귀 — 표본 + 글꼴(ttfs)·자산(assets) + 설계 문서, "
            "그리고 tools/ 의 렌더 게이트류(render_page_gate.py, compare_page_bbox.py 등)."
        ),
    },
    "visual-regression": {
        "paths": [
            "src", "tests", "samples", "saved", "assets", "ttfs",
            "pdf", "pdf-2020", "pdf-large", "tools",
        ],
        "note": (
            "한컴 PDF 오라클 대조까지 포함하는 무거운 프리셋(pdf/pdf-2020/pdf-large 는 "
            "바이너리 코퍼스) — 필요할 때만 명시적으로 고른다. 일반 렌더 작업은 renderer 로 충분."
        ),
    },
    "cli-tools": {
        "paths": ["tools", "mydocs/manual", "mydocs/tech/agent_org", ".github", "src"],
        "note": (
            "tools/ 밑 에이전트 도구 개발(이 스크립트류) — 플레이북·부서표를 함께 받는다. "
            "빌드된 rhwp 바이너리를 서브프로세스로 부르는 도구가 많아 src 도 포함."
        ),
    },
    "docs-only": {
        "paths": ["mydocs", "docs", "README.md", "README_EN.md", "llms.txt"],
        "note": "코드 없이 문서만 — 텍스트 위주라 가볍다.",
    },
    "studio": {
        "paths": ["rhwp-studio", "rhwp-shared", "assets", "mydocs/manual", "src"],
        "note": "웹 스튜디오 UI 작업 — 공유 컴포넌트(rhwp-shared)와 UI 관례 문서 포함.",
    },
    "browser-ext": {
        "paths": ["rhwp-chrome", "rhwp-firefox", "rhwp-safari", "rhwp-vscode", "rhwp-shared"],
        "note": "브라우저 확장/VSCode 확장 — 확장 4종 + 공유 코드.",
    },
    "bindings": {
        "paths": ["bindings", "npm", "typescript", "src", "tests"],
        "note": "언어 바인딩(Native·npm·TS 선언) 작업 — 코어 엔진과 함께.",
    },
    "fuzz": {
        "paths": ["fuzz", "src", "tests"],
        "note": "퍼징 타겟 작업.",
    },
    "gym": {
        "paths": ["gym", "tools", "mydocs/tech/agent_org", "mydocs/tech/agent_roadmap"],
        "note": "에이전트 배치·훈련 과제(gym) — 부서표·로드맵·배치 도구(tools/agent_dispatch.py 등) 포함.",
    },
    "ci-ops": {
        "paths": [".github", "scripts", "mydocs/manual"],
        "note": "GitHub Actions·워크플로·CI 스크립트 운영.",
    },
}


def get_current() -> list[str] | None:
    """현재 sparse-checkout 목록. sparse 가 아니거나 git 이 없으면 None."""
    try:
        r = subprocess.run(
            ["git", "sparse-checkout", "list"],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=20,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired, OSError):
        return None
    if r.returncode != 0:
        return None
    return [line.strip() for line in r.stdout.splitlines() if line.strip()]


def resolve_paths(tasks: list[str]) -> tuple[list[str], list[str]]:
    """ALWAYS + 프리셋 합집합(순서 보존, 중복 제거). unknown 프리셋 이름도 함께 반환."""
    unknown = [t for t in tasks if t not in PRESETS]
    seen: dict[str, None] = {}
    for p in ALWAYS:
        seen.setdefault(p, None)
    for t in tasks:
        if t in PRESETS:
            for p in PRESETS[t]["paths"]:
                seen.setdefault(p, None)
    return list(seen.keys()), unknown


def print_list(as_json: bool) -> None:
    if as_json:
        payload = {
            "schemaVersion": SCHEMA_VERSION,
            "always": ALWAYS,
            "presets": {
                name: {"paths": info["paths"], "note": info["note"]}
                for name, info in PRESETS.items()
            },
        }
        print(json.dumps(payload, ensure_ascii=False))
        return
    print(f"항상 포함: {' '.join(ALWAYS)}\n")
    for name, info in PRESETS.items():
        print(f"{name}")
        print(f"  경로: {' '.join(info['paths'])}")
        print(f"  근거: {info['note']}")
        print()


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--task",
        help="쉼표로 구분한 프리셋 이름(예: parser 또는 parser,studio). --list 로 목록 확인",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="미리보기만 하지 않고 실제로 git sparse-checkout add 를 실행한다",
    )
    parser.add_argument("--list", action="store_true", help="프리셋 목록과 경로·근거를 출력")
    parser.add_argument("--json", action="store_true", help="기계 판독용 JSON 출력")
    return parser


def main(argv: list[str] | None = None) -> int:
    for stream in (sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(encoding="utf-8", errors="replace")

    parser = build_parser()
    args = parser.parse_args(argv)

    if args.list:
        print_list(args.json)
        return 0

    if not args.task:
        print("오류: --task <프리셋[,프리셋...]> 또는 --list 가 필요합니다.", file=sys.stderr)
        return 2

    tasks = [t.strip() for t in args.task.split(",") if t.strip()]
    paths, unknown = resolve_paths(tasks)
    if unknown:
        print(
            f"오류: 알 수 없는 프리셋 - {', '.join(unknown)} "
            f"(사용 가능: {', '.join(PRESETS)})",
            file=sys.stderr,
        )
        return 2

    current = get_current()
    new_paths = [p for p in paths if current is None or p not in current]
    # --skip-checks: 프리셋에는 디렉터리(mydocs, src…)와 뿌리 파일(README.md…)이
    # 섞여 있다. cone 모드의 기본 add 는 "디렉터리만" 요구해 파일 인자에서
    # `fatal: '<file>' is not a directory` 로 죽는다(실측, 2026-08-16) — 파일도
    # 받아들이게 하는 공식 우회가 이 플래그다. 존재하지 않는 경로를 실수로 줘도
    # 그 경로는 그냥 아무 것도 체크아웃하지 않을 뿐이라 위험하지 않다.
    cmd = ["git", "sparse-checkout", "add", "--skip-checks", *paths]

    if current is None:
        # sparse-checkout 이 아직 초기화되지 않았을 수 있다 — 자동으로 cone 모드를
        # 켜지 않는다(전체 checkout 을 sparse 로 바꾸는 것은 이 도구의 책임 밖이다).
        init_hint = "git sparse-checkout init --cone && " + " ".join(cmd)
        if args.json:
            print(json.dumps({
                "schemaVersion": SCHEMA_VERSION,
                "tasks": tasks,
                "paths": paths,
                "applied": False,
                "reason": "sparse-checkout 미확인(초기화 안 됐거나 git 실행 실패)",
                "command": init_hint,
            }, ensure_ascii=False))
        else:
            print("현재 sparse-checkout 상태를 확인할 수 없습니다(초기화 안 됐을 수 있음).",
                  file=sys.stderr)
            print(f"먼저 초기화가 필요하면: {init_hint}", file=sys.stderr)
        return 0

    if args.apply:
        r = subprocess.run(
            cmd, cwd=REPO_ROOT, capture_output=True, text=True,
            encoding="utf-8", errors="replace", timeout=600,
        )
        if r.returncode != 0:
            print(r.stderr, file=sys.stderr)
            return 1
        after = get_current() or []
        if args.json:
            print(json.dumps({
                "schemaVersion": SCHEMA_VERSION,
                "tasks": tasks,
                "paths": paths,
                "newPaths": new_paths,
                "applied": True,
                "current": after,
            }, ensure_ascii=False))
        else:
            if new_paths:
                print(f"추가됨: {' '.join(new_paths)}")
            else:
                print("이미 전부 받아져 있습니다 — 추가된 경로 없음.")
            print(f"현재 sparse-checkout: {' '.join(after)}")
        return 0

    # 미리보기(기본값) — 아무 것도 바꾸지 않는다.
    if args.json:
        print(json.dumps({
            "schemaVersion": SCHEMA_VERSION,
            "tasks": tasks,
            "paths": paths,
            "newPaths": new_paths,
            "applied": False,
            "command": " ".join(cmd),
        }, ensure_ascii=False))
    else:
        if new_paths:
            print(f"새로 받을 경로: {' '.join(new_paths)}")
        else:
            print("이미 전부 받아져 있습니다 — 실행해도 추가되는 경로 없음.")
        print(f"실행할 명령: {' '.join(cmd)}")
        print("(--apply 를 붙이면 바로 실행합니다)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
