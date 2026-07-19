#!/usr/bin/env python3
"""저장소 내부 Markdown 상대 링크를 검사한다.

외부 URL과 문서 내 앵커는 이 도구의 범위 밖이다. 파일 이동 전에 내부 경로가
깨지지 않았는지 확인하는 용도로 사용한다.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import unquote, urlsplit


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PATHS = (
    "AGENTS.md",
    "CLAUDE.md",
    "README.md",
    "README_EN.md",
    "CONTRIBUTING.md",
    "mydocs/README.md",
    "mydocs/manual",
    "mydocs/tech",
    "mydocs/troubleshootings",
)
FENCE_RE = re.compile(r"^\s*(`{3,}|~{3,})")
INLINE_LINK_RE = re.compile(
    r"!?\[[^\]]*\]\(\s*(?:<([^>]+)>|([^\s)]+))(?:\s+[^)]*)?\s*\)"
)
REFERENCE_LINK_RE = re.compile(r"^\s*\[[^\]]+\]:\s*(?:<([^>]+)>|([^\s]+))")


@dataclass(frozen=True)
class BrokenLink:
    source: Path
    line: int
    destination: str
    resolved: Path


@dataclass(frozen=True)
class ForbiddenLink:
    source: Path
    line: int
    destination: str
    resolved: Path


@dataclass(frozen=True)
class RedirectReference:
    source: Path
    line: int
    redirect_path: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="저장소 내부 Markdown 상대 링크의 대상 파일 존재 여부를 검사합니다."
    )
    parser.add_argument(
        "paths",
        nargs="*",
        default=list(DEFAULT_PATHS),
        help=(
            "검사할 저장소 상대 파일 또는 디렉터리 "
            "(기본: 루트 안내 문서와 mydocs/manual·tech·troubleshootings)"
        ),
    )
    parser.add_argument(
        "--forbid-path",
        action="append",
        default=[],
        metavar="PATH",
        help="새 참조를 금지할 저장소 상대 경로. 문서 이동 뒤 이전 경로를 검사할 때 반복 지정한다.",
    )
    parser.add_argument(
        "--forbid-scan-path",
        action="append",
        default=[],
        metavar="PATH",
        help=(
            "금지 경로 참조만 검사할 저장소 상대 파일 또는 디렉터리. "
            "지정하지 않으면 기본 링크 검사 문서와 같은 범위를 사용한다."
        ),
    )
    parser.add_argument(
        "--changed-from",
        metavar="REF",
        help=(
            "REF와 현재 작업 트리 사이에서 추가·수정된 Markdown도 링크 검사에 포함한다. "
            "redirect 재참조 검사는 이 범위의 변경 파일만 대상으로 한다."
        ),
    )
    parser.add_argument(
        "--forbid-redirect-references",
        action="store_true",
        help=(
            "redirect stub에서 이전 경로를 동적으로 수집해 변경 코드·문서의 재참조를 거부한다. "
            "--changed-from이 없으면 전체 추적 파일을 검사한다."
        ),
    )
    return parser.parse_args()


def iter_markdown_files(raw_paths: list[str]) -> list[Path]:
    files: set[Path] = set()
    for raw_path in raw_paths:
        candidate = (REPOSITORY_ROOT / raw_path).resolve()
        try:
            candidate.relative_to(REPOSITORY_ROOT)
        except ValueError as error:
            raise SystemExit(f"저장소 밖 경로는 검사할 수 없습니다: {raw_path}") from error

        if candidate.is_file():
            if candidate.suffix.lower() == ".md":
                files.add(candidate)
            continue
        if candidate.is_dir():
            files.update(path.resolve() for path in candidate.rglob("*.md") if path.is_file())
            continue
        raise SystemExit(f"검사 경로가 없습니다: {raw_path}")
    return sorted(files)


def git_paths(*args: str) -> set[Path]:
    result = subprocess.run(
        ["git", *args],
        cwd=REPOSITORY_ROOT,
        check=True,
        stdout=subprocess.PIPE,
    )
    return {
        (REPOSITORY_ROOT / raw_path).resolve()
        for raw_path in result.stdout.decode("utf-8").split("\0")
        if raw_path
    }


def changed_files(reference: str) -> set[Path]:
    merge_base = subprocess.run(
        ["git", "merge-base", reference, "HEAD"],
        cwd=REPOSITORY_ROOT,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    ).stdout.strip()
    changed = git_paths(
        "diff", "--name-only", "-z", "--diff-filter=ACMR", merge_base, "--"
    )
    changed.update(git_paths("ls-files", "-z", "--others", "--exclude-standard"))
    return {path for path in changed if path.is_file()}


def tracked_files() -> set[Path]:
    files = git_paths("ls-files", "-z")
    files.update(git_paths("ls-files", "-z", "--others", "--exclude-standard"))
    return {path for path in files if path.is_file()}


def redirect_mapping() -> dict[Path, str]:
    redirects: dict[Path, str] = {}
    for relative_root in ("mydocs/manual", "mydocs/tech"):
        root = REPOSITORY_ROOT / relative_root
        for path in root.rglob("*.md"):
            lines = path.read_text(encoding="utf-8").splitlines()
            if not any(line.strip() == "# 이동됨" for line in lines[:12]):
                continue
            canonical = next(
                (
                    line.split(":", 1)[1].strip()
                    for line in lines[:12]
                    if line.startswith("canonical:")
                ),
                "",
            )
            if not canonical:
                raise SystemExit(f"redirect stub의 canonical이 없습니다: {display_path(path)}")
            redirects[path.resolve()] = canonical
    return redirects


def destinations_in_markdown(source: Path) -> list[tuple[int, str]]:
    destinations: list[tuple[int, str]] = []
    in_fence = False
    fence_marker = ""
    for line_number, line in enumerate(source.read_text(encoding="utf-8").splitlines(), start=1):
        fence_match = FENCE_RE.match(line)
        if fence_match:
            marker = fence_match.group(1)
            if not in_fence:
                in_fence = True
                fence_marker = marker[0]
            elif marker[0] == fence_marker:
                in_fence = False
            continue
        if in_fence:
            continue

        for match in INLINE_LINK_RE.finditer(line):
            destinations.append((line_number, match.group(1) or match.group(2)))
        reference_match = REFERENCE_LINK_RE.match(line)
        if reference_match:
            destinations.append((line_number, reference_match.group(1) or reference_match.group(2)))
    return destinations


def resolve_local_destination(source: Path, destination: str) -> Path | None:
    parsed = urlsplit(destination)
    if parsed.scheme or parsed.netloc:
        return None

    path_text = unquote(parsed.path)
    if not path_text:
        return None
    if path_text.startswith("/"):
        resolved = REPOSITORY_ROOT / path_text.lstrip("/")
    else:
        resolved = source.parent / path_text
    return resolved.resolve()


def normalize_forbidden_paths(raw_paths: list[str]) -> set[Path]:
    normalized: set[Path] = set()
    for raw_path in raw_paths:
        candidate = (REPOSITORY_ROOT / raw_path).resolve()
        try:
            candidate.relative_to(REPOSITORY_ROOT)
        except ValueError as error:
            raise SystemExit(f"저장소 밖 금지 경로는 지정할 수 없습니다: {raw_path}") from error
        normalized.add(candidate)
    return normalized


def collect_broken_links(markdown_files: list[Path]) -> list[BrokenLink]:
    broken: list[BrokenLink] = []
    for source in markdown_files:
        for line, destination in destinations_in_markdown(source):
            resolved = resolve_local_destination(source, destination)
            if resolved is None:
                continue
            try:
                resolved.relative_to(REPOSITORY_ROOT)
            except ValueError:
                broken.append(BrokenLink(source, line, destination, resolved))
                continue
            if resolved.exists():
                continue
            broken.append(BrokenLink(source, line, destination, resolved))
    return broken


def collect_forbidden_links(
    markdown_files: list[Path], forbidden_paths: set[Path]
) -> list[ForbiddenLink]:
    forbidden: list[ForbiddenLink] = []
    for source in markdown_files:
        for line, destination in destinations_in_markdown(source):
            resolved = resolve_local_destination(source, destination)
            if resolved is not None and resolved in forbidden_paths:
                forbidden.append(ForbiddenLink(source, line, destination, resolved))
    return forbidden


def collect_redirect_references(
    scan_files: set[Path], redirects: dict[Path, str]
) -> list[RedirectReference]:
    references: list[RedirectReference] = []
    redirect_paths = {display_path(path) for path in redirects}
    for source in sorted(scan_files):
        if source in redirects:
            continue
        try:
            lines = source.read_text(encoding="utf-8").splitlines()
        except (OSError, UnicodeDecodeError):
            continue
        for line_number, line in enumerate(lines, start=1):
            for redirect_path in redirect_paths:
                if redirect_path in line:
                    references.append(
                        RedirectReference(source, line_number, redirect_path)
                    )
    return references


def display_path(path: Path) -> str:
    try:
        return str(path.relative_to(REPOSITORY_ROOT))
    except ValueError:
        return str(path)


def main() -> int:
    args = parse_args()
    changed = changed_files(args.changed_from) if args.changed_from else set()
    markdown_files = sorted(
        set(iter_markdown_files(args.paths))
        | {path for path in changed if path.suffix.lower() == ".md"}
    )
    forbidden_scan_files = (
        iter_markdown_files(args.forbid_scan_path)
        if args.forbid_scan_path
        else markdown_files
    )
    redirects = redirect_mapping() if args.forbid_redirect_references else {}
    forbidden_paths = normalize_forbidden_paths(args.forbid_path)
    forbidden_paths.update(redirects)
    broken_links = collect_broken_links(markdown_files)
    forbidden_links = collect_forbidden_links(
        forbidden_scan_files, forbidden_paths
    )
    redirect_references = collect_redirect_references(
        changed if args.changed_from else tracked_files(), redirects
    )

    print(f"검사 문서: {len(markdown_files)}개")
    if args.changed_from:
        print(f"변경 파일: {len(changed)}개 ({args.changed_from} 기준)")
    if forbidden_scan_files != markdown_files:
        print(f"금지 경로 검사 문서: {len(forbidden_scan_files)}개")
    if redirects:
        print(f"redirect stub: {len(redirects)}개")
    if not broken_links and not forbidden_links and not redirect_references:
        print("내부 Markdown 상대 링크: 이상 없음")
        return 0

    if broken_links:
        print(f"깨진 내부 Markdown 상대 링크: {len(broken_links)}건")
    for broken in broken_links:
        print(
            f"- {display_path(broken.source)}:{broken.line}: "
            f"{broken.destination} -> {display_path(broken.resolved)}",
            file=sys.stderr,
        )
    if forbidden_links:
        print(f"금지된 이전 경로 참조: {len(forbidden_links)}건")
    for forbidden in forbidden_links:
        print(
            f"- {display_path(forbidden.source)}:{forbidden.line}: "
            f"{forbidden.destination} -> {display_path(forbidden.resolved)}",
            file=sys.stderr,
        )
    if redirect_references:
        print(f"redirect 이전 경로 문자열 재참조: {len(redirect_references)}건")
    for reference in redirect_references:
        print(
            f"- {display_path(reference.source)}:{reference.line}: "
            f"{reference.redirect_path}",
            file=sys.stderr,
        )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
