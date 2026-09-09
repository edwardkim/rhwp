#!/usr/bin/env python3
"""Release package publish가 승인된 exact tag source를 사용하는지 검증한다."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any, Mapping


SHA_PATTERN = re.compile(r"^[0-9a-f]{40}$")


def _error(errors: list[str], condition: bool, code: str) -> None:
    if not condition:
        errors.append(code)


def evaluate_release_source(context: Mapping[str, Any]) -> dict[str, Any]:
    """정규화된 입력을 순수 함수로 검증해 결정적 verdict를 반환한다."""

    mode = str(context.get("mode", ""))
    event_name = str(context.get("eventName", ""))
    ref = str(context.get("ref", ""))
    ref_type = str(context.get("refType", ""))
    ref_name = str(context.get("refName", ""))
    github_sha = str(context.get("githubSha", "")).lower()
    checkout_sha = str(context.get("checkoutSha", "")).lower()
    tag_sha_value = context.get("tagSha")
    tag_sha = str(tag_sha_value).lower() if tag_sha_value else None
    versions_value = context.get("versions", {})
    versions = dict(versions_value) if isinstance(versions_value, Mapping) else {}
    release_value = context.get("release")
    release = dict(release_value) if isinstance(release_value, Mapping) else None

    errors: list[str] = []
    _error(errors, mode in {"verify", "publish"}, "invalid-mode")
    _error(errors, bool(SHA_PATTERN.fullmatch(github_sha)), "invalid-github-sha")
    _error(errors, bool(SHA_PATTERN.fullmatch(checkout_sha)), "invalid-checkout-sha")
    _error(errors, github_sha == checkout_sha, "checkout-sha-mismatch")

    cargo_version = str(versions.get("cargo", ""))
    expected_tag = f"v{cargo_version}" if cargo_version else ""
    _error(errors, bool(cargo_version), "cargo-version-missing")
    for package_name in ("npmEditor", "vscode"):
        _error(
            errors,
            str(versions.get(package_name, "")) == cargo_version,
            f"version-mismatch:{package_name}",
        )

    if mode == "publish":
        _error(
            errors,
            event_name in {"push", "workflow_dispatch"},
            "publish-event-not-allowed",
        )
        _error(errors, ref_type == "tag", "publish-ref-not-tag")
        _error(errors, ref == f"refs/tags/{ref_name}", "publish-ref-name-mismatch")
        _error(errors, ref_name == expected_tag, "publish-tag-version-mismatch")
        _error(errors, tag_sha is not None, "tag-sha-missing")
        _error(errors, tag_sha == github_sha, "tag-sha-mismatch")
        _error(errors, release is not None, "release-metadata-missing")
        if release is not None:
            _error(
                errors,
                str(release.get("tag_name", "")) == ref_name,
                "release-tag-mismatch",
            )
            _error(errors, release.get("draft") is False, "release-is-draft")
            _error(errors, release.get("prerelease") is False, "release-is-prerelease")
            _error(errors, bool(release.get("published_at")), "release-not-published")

    return {
        "schemaVersion": 1,
        "accepted": not errors,
        "mode": mode,
        "eventName": event_name,
        "ref": ref,
        "refType": ref_type,
        "refName": ref_name,
        "githubSha": github_sha,
        "checkoutSha": checkout_sha,
        "tagSha": tag_sha,
        "expectedTag": expected_tag,
        "versions": versions,
        "errors": errors,
    }


def _git_commit(repo_root: Path, revision: str) -> str | None:
    result = subprocess.run(
        ["git", "rev-parse", f"{revision}^{{commit}}"],
        cwd=repo_root,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        return None
    value = result.stdout.strip().lower()
    return value if SHA_PATTERN.fullmatch(value) else None


def _package_version(repo_root: Path, relative_path: str) -> str:
    data = json.loads((repo_root / relative_path).read_text(encoding="utf-8"))
    return str(data["version"])


def _versions(repo_root: Path) -> dict[str, str]:
    cargo = tomllib.loads((repo_root / "Cargo.toml").read_text(encoding="utf-8"))
    return {
        "cargo": str(cargo["package"]["version"]),
        "npmEditor": _package_version(repo_root, "npm/editor/package.json"),
        "vscode": _package_version(repo_root, "rhwp-vscode/package.json"),
    }


def _release_metadata(path: str | None) -> dict[str, Any] | None:
    if path is None:
        return None
    if path == "-":
        return json.load(sys.stdin)
    return json.loads(Path(path).read_text(encoding="utf-8"))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--mode", required=True, choices=("verify", "publish"))
    parser.add_argument("--event-name", required=True)
    parser.add_argument("--ref", required=True)
    parser.add_argument("--ref-type", required=True)
    parser.add_argument("--ref-name", required=True)
    parser.add_argument("--github-sha", required=True)
    parser.add_argument("--release-json")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    repo_root = Path(args.repo_root).resolve()
    checkout_sha = _git_commit(repo_root, "HEAD")
    tag_sha = (
        _git_commit(repo_root, f"refs/tags/{args.ref_name}")
        if args.mode == "publish"
        else None
    )
    context = {
        "mode": args.mode,
        "eventName": args.event_name,
        "ref": args.ref,
        "refType": args.ref_type,
        "refName": args.ref_name,
        "githubSha": args.github_sha,
        "checkoutSha": checkout_sha,
        "tagSha": tag_sha,
        "versions": _versions(repo_root),
        "release": _release_metadata(args.release_json),
    }
    verdict = evaluate_release_source(context)
    print(json.dumps(verdict, ensure_ascii=False, sort_keys=True))
    return 0 if verdict["accepted"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
