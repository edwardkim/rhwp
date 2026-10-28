#!/usr/bin/env python3
"""Validate the canonical Hancom PDF oracle repository policy."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
MAX_PDF_BYTES_EXCLUSIVE = 50 * 1024 * 1024
LFS_POINTER_HEADER = b"version https://git-lfs.github.com/spec/v1\n"
RETIRED_ORACLE_ROOTS = ("pdf-2010", "pdf-2020", "pdf-large")


@dataclass(frozen=True)
class Violation:
    path: str
    rule: str
    detail: str


def relative_path(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


def canonical_pdfs(root: Path) -> list[Path]:
    pdf_root = root / "pdf"
    if not pdf_root.is_dir():
        return []
    return sorted(
        (path for path in pdf_root.rglob("*") if path.is_file() and path.suffix.lower() == ".pdf"),
        key=lambda path: relative_path(path, root),
    )


def git_filter_values(root: Path, paths: list[Path]) -> dict[str, str]:
    if not paths:
        return {}
    payload = b"".join(
        os.fsencode(relative_path(path, root)) + b"\0"
        for path in paths
    )
    completed = subprocess.run(
        ["git", "check-attr", "-z", "--stdin", "filter"],
        cwd=root,
        input=payload,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if completed.returncode != 0:
        message = completed.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(f"git check-attr failed: {message}")
    fields = completed.stdout.split(b"\0")
    if fields and fields[-1] == b"":
        fields.pop()
    if len(fields) % 3:
        raise RuntimeError("git check-attr returned an invalid NUL-delimited record")
    return {
        os.fsdecode(fields[index]): os.fsdecode(fields[index + 2])
        for index in range(0, len(fields), 3)
    }


def indexed_lfs_pointers(root: Path) -> list[str]:
    completed = subprocess.run(
        ["git", "ls-files", "--stage", "-z", "--", "pdf"],
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if completed.returncode != 0:
        message = completed.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(f"git ls-files failed: {message}")

    entries: list[tuple[str, str]] = []
    for record in completed.stdout.split(b"\0"):
        if not record:
            continue
        metadata, raw_path = record.split(b"\t", 1)
        _mode, raw_oid, raw_stage = metadata.split(b" ")
        path = os.fsdecode(raw_path)
        if raw_stage != b"0" or not path.lower().endswith(".pdf"):
            continue
        entries.append((path, raw_oid.decode("ascii")))
    if not entries:
        return []

    unique_oids = sorted({oid for _path, oid in entries})
    sizes = subprocess.run(
        ["git", "cat-file", "--batch-check=%(objectname) %(objectsize)"],
        cwd=root,
        input=("\n".join(unique_oids) + "\n").encode("ascii"),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if sizes.returncode != 0:
        message = sizes.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(f"git cat-file --batch-check failed: {message}")
    size_by_oid = {
        line.split()[0]: int(line.split()[1])
        for line in sizes.stdout.decode("ascii").splitlines()
    }

    candidate_oids = {oid for oid, size in size_by_oid.items() if size <= 1024}
    pointer_oids: set[str] = set()
    for oid in candidate_oids:
        data = subprocess.check_output(["git", "cat-file", "blob", oid], cwd=root)
        if data.startswith(LFS_POINTER_HEADER):
            pointer_oids.add(oid)
    return sorted(path for path, oid in entries if oid in pointer_oids)


def evaluate(root: Path) -> tuple[list[Path], list[Violation]]:
    root = root.resolve()
    violations: list[Violation] = []
    for retired in RETIRED_ORACLE_ROOTS:
        path = root / retired
        if path.exists():
            violations.append(
                Violation(retired, "retired-root", "한컴 PDF 오라클은 pdf/**만 사용해야 합니다")
            )

    files = canonical_pdfs(root)
    if not (root / "pdf").is_dir():
        violations.append(Violation("pdf", "missing-root", "정본 PDF 루트가 없습니다"))
        return files, violations

    for path in files:
        rel = relative_path(path, root)
        size = path.stat().st_size
        with path.open("rb") as stream:
            header = stream.read(160)
        if header.startswith(LFS_POINTER_HEADER):
            violations.append(Violation(rel, "lfs-pointer", "작업트리가 실제 PDF가 아닌 LFS pointer입니다"))
        elif not header.startswith(b"%PDF-"):
            violations.append(Violation(rel, "pdf-magic", "파일이 %PDF- magic으로 시작하지 않습니다"))
        if size >= MAX_PDF_BYTES_EXCLUSIVE:
            violations.append(
                Violation(
                    rel,
                    "size-limit",
                    f"{size} bytes는 상한 {MAX_PDF_BYTES_EXCLUSIVE} bytes 미만이 아닙니다",
                )
            )

    filters = git_filter_values(root, files)
    for path, value in sorted(filters.items()):
        if value == "lfs":
            violations.append(Violation(path, "lfs-attribute", "Git filter=lfs가 적용됩니다"))
    for path in indexed_lfs_pointers(root):
        violations.append(Violation(path, "lfs-index-pointer", "Git index가 LFS pointer를 보관합니다"))

    violations.sort(key=lambda item: (item.path, item.rule, item.detail))
    return files, violations


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=REPOSITORY_ROOT)
    parser.add_argument("--json", action="store_true")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        files, violations = evaluate(args.repo_root)
    except RuntimeError as error:
        print(f"PDF repository policy: ERROR: {error}", file=sys.stderr)
        return 2

    payload = {
        "status": "ok" if not violations else "fail",
        "pdfCount": len(files),
        "maxPdfBytesExclusive": MAX_PDF_BYTES_EXCLUSIVE,
        "violations": [asdict(item) for item in violations],
    }
    if args.json:
        print(json.dumps(payload, ensure_ascii=False, indent=2))
    elif violations:
        print(f"PDF repository policy: FAIL ({len(violations)} violation(s))")
        for item in violations:
            print(f"- {item.path}: {item.rule}: {item.detail}")
    else:
        print(
            "PDF repository policy: OK "
            f"({len(files)} PDFs, each < {MAX_PDF_BYTES_EXCLUSIVE} bytes, no LFS pointers)"
        )
    return 1 if violations else 0


if __name__ == "__main__":
    raise SystemExit(main())
