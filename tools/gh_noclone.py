#!/usr/bin/env python3
"""로컬 clone/checkout 없이 GitHub API만으로 파일을 읽고 고치는 도구.

## 왜 있는가

이 저장소는 ~1.4GB 규모다. 이 개발 환경(2026-08-16 기준, bg 이 PC)은 대용량
git pack 전송이 구조적으로 불안정해서 `git clone`/`git fetch`(blob 포함)가
`fatal: fetch-pack: invalid index-pack output` 로 반복 실패한다 — 실측
전송 속도는 약 1MB/s 이고, 그 자체가 깨진 게 아니라 git/curl 쪽의 짧은
타임아웃이 느린 전송을 조기에 죽여서 나는 오류다(진짜 원인 진단은
`mydocs/troubleshootings/` 참고, 없으면 새로 기록할 것).

반면 GitHub REST Contents/Git API 는 개별 파일 단위 요청이라 크기가 작고,
이 환경에서 안정적으로 성공한다(오늘 세션에서 PR #4887·#4888 의 `cargo fmt`
수정을 이 방식으로 로컬 clone 전혀 없이 push 까지 마쳤다).

**언제 이 도구를 써야 하는가**: 몇 개 파일만 읽거나 고치면 되는 작업
(포맷 수정, 오타, 작은 로직 조각, 새 파일 추가)인데 로컬 clone 이 느리거나
실패할 때. **언제 못 쓰는가**: 여러 파일에 걸친 구조적 병합·충돌 해결·
`cargo build`/`cargo test` 로 실제 컴파일 검증이 필요한 변경 — 그런 작업은
`git sparse-checkout`(`tools/sparse_clone_hint.md` 참고)으로 필요한 하위
디렉터리만 좁혀서 받는 쪽이 낫다. 이 도구는 그 좁히기가 오히려 배보다 배꼽인
"파일 한두 개" 규모를 위한 것이다.

## 하위 명령

    python3 tools/gh_noclone.py read <경로> --ref <브랜치> [--repo owner/name]
    python3 tools/gh_noclone.py write <경로> --ref <브랜치> --file <로컬파일> \
        --message <커밋메시지> [--repo owner/name] [--create]
    python3 tools/gh_noclone.py ci-status [--repo owner/name] [--author 이름]
    python3 tools/gh_noclone.py ci-log <run-id> [--repo owner/name] [--failed-only]

인증은 `gh` CLI(`gh auth status`)에 위임한다 — 토큰을 이 스크립트에 넣지 않는다.

Python 3 표준 라이브러리만 사용한다(agent-toolkit 관례와 동일). `gh` 는
서브프로세스로만 호출한다 — REST 인증을 직접 구현하지 않는다.
"""

from __future__ import annotations

import argparse
import base64
import json
import subprocess
import sys


DEFAULT_REPO_UPSTREAM = "edwardkim/rhwp"


def run_gh(args: list[str], input_text: str | None = None) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["gh", *args],
        input=input_text,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )


def cmd_read(args: argparse.Namespace) -> int:
    result = run_gh(
        [
            "api",
            f"repos/{args.repo}/contents/{args.path}",
            "-f",
            f"ref={args.ref}",
            "--jq",
            ".content",
        ]
    )
    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        return 1
    content = base64.b64decode(result.stdout.strip()).decode("utf-8", errors="replace")
    if args.out:
        with open(args.out, "w", encoding="utf-8", newline="") as f:
            f.write(content)
        print(f"{args.out} 에 저장됨 ({len(content)}바이트)", file=sys.stderr)
    else:
        sys.stdout.write(content)
    return 0


def cmd_write(args: argparse.Namespace) -> int:
    with open(args.file, "rb") as f:
        content_b64 = base64.b64encode(f.read()).decode("ascii")

    api_args = [
        "api",
        f"repos/{args.repo}/contents/{args.path}",
        "-X",
        "PUT",
        "-f",
        f"message={args.message}",
        "-f",
        f"content={content_b64}",
        "-f",
        f"branch={args.ref}",
    ]

    if not args.create:
        sha_result = run_gh(
            [
                "api",
                f"repos/{args.repo}/contents/{args.path}",
                "-f",
                f"ref={args.ref}",
                "--jq",
                ".sha",
            ]
        )
        if sha_result.returncode != 0:
            print(
                "기존 파일의 sha 를 못 가져왔다 — 새 파일이면 --create 를 붙여라.",
                file=sys.stderr,
            )
            print(sha_result.stderr, file=sys.stderr)
            return 1
        sha = sha_result.stdout.strip()
        api_args += ["-f", f"sha={sha}"]

    result = run_gh(api_args)
    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        return 1
    commit_sha = json.loads(result.stdout).get("commit", {}).get("sha", "?")
    print(f"커밋됨: {commit_sha} ({args.repo}@{args.ref}:{args.path})", file=sys.stderr)
    return 0


def cmd_ci_status(args: argparse.Namespace) -> int:
    fields = "number,title,headRefName,isDraft,mergeable,statusCheckRollup"
    gh_args = [
        "pr",
        "list",
        "--repo",
        args.repo,
        "--state",
        "open",
        "--json",
        fields,
    ]
    if args.author:
        gh_args += ["--author", args.author]
    result = run_gh(gh_args)
    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        return 1

    prs = json.loads(result.stdout)
    any_failure = False
    for pr in prs:
        checks = pr.get("statusCheckRollup") or []
        failed = [
            c
            for c in checks
            if c.get("conclusion") == "FAILURE" and c.get("__typename") == "CheckRun"
        ]
        pending = [c for c in checks if c.get("status") not in (None, "COMPLETED")]
        status = "FAILURE" if failed else ("PENDING" if pending else "OK")
        if status != "OK":
            any_failure = True
        mergeable = pr.get("mergeable") or "?"
        print(
            f"#{pr['number']:<6} {status:<8} "
            f"{'DRAFT' if pr.get('isDraft') else 'OPEN ':<6} "
            f"{mergeable:<12} "
            f"{pr['headRefName']:<40} {pr['title']}"
        )
        for c in failed:
            print(f"        ✗ {c.get('workflowName')} / {c.get('name')} — {c.get('detailsUrl')}")

    if args.json:
        print(json.dumps(prs, ensure_ascii=False))

    return 0 if not (args.fail_on_red and any_failure) else 2


def cmd_ci_log(args: argparse.Namespace) -> int:
    gh_args = ["run", "view", args.run_id, "--repo", args.repo]
    gh_args += ["--log-failed"] if args.failed_only else ["--log"]
    result = run_gh(gh_args)
    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        return 1
    # 타임스탬프·ANSI 색 코드를 걷어내고 본문만 남긴다 — 실패 원인을 빨리 읽기 위함.
    for line in result.stdout.splitlines():
        parts = line.split("\t", 2)
        text = parts[2] if len(parts) == 3 else line
        text = text.split("Z ", 1)[-1] if text[:4].isdigit() else text
        print(text)
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--repo", default=DEFAULT_REPO_UPSTREAM, help="owner/repo")
    sub = parser.add_subparsers(dest="command", required=True)

    p_read = sub.add_parser("read", help="파일 내용을 읽는다 (clone 불필요)")
    p_read.add_argument("path")
    p_read.add_argument("--ref", required=True)
    p_read.add_argument("--out", help="저장할 로컬 경로 (생략하면 stdout)")
    p_read.set_defaults(func=cmd_read)

    p_write = sub.add_parser("write", help="파일을 커밋한다 (clone 불필요)")
    p_write.add_argument("path")
    p_write.add_argument("--ref", required=True)
    p_write.add_argument("--file", required=True, help="커밋할 로컬 파일")
    p_write.add_argument("--message", required=True)
    p_write.add_argument("--create", action="store_true", help="새 파일 생성 (기존 sha 조회 생략)")
    p_write.set_defaults(func=cmd_write)

    p_status = sub.add_parser("ci-status", help="open PR들의 CI 상태 요약")
    p_status.add_argument("--author")
    p_status.add_argument("--json", action="store_true", help="원본 JSON도 함께 출력")
    p_status.add_argument(
        "--fail-on-red", action="store_true", help="FAILURE/PENDING 이 있으면 exit 2"
    )
    p_status.set_defaults(func=cmd_ci_status)

    p_log = sub.add_parser("ci-log", help="워크플로 실행 로그 조회")
    p_log.add_argument("run_id")
    p_log.add_argument("--failed-only", action="store_true", default=True)
    p_log.set_defaults(func=cmd_ci_log)

    return parser


def main(argv: list[str] | None = None) -> int:
    # Windows 콘솔 기본 코드페이지(cp949 등)는 한글 제목의 em dash 같은 문자에서
    # UnicodeEncodeError 를 낸다 — tools/cli-autodiscovery/autodiscover.py 와 동일한 수정.
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")

    parser = build_parser()
    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
