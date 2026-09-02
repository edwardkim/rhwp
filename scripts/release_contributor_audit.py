#!/usr/bin/env python3
"""Build reproducible contributor evidence for an rhwp release range.

The tool deliberately separates local Git evidence from GitHub metadata:

1. ``candidates`` extracts commit identities and referenced issue/PR numbers.
2. A maintainer resolves those numbers with authenticated ``gh`` tooling.
3. ``ledger`` combines both inputs and applies explicit identity overrides.

Raw candidate data can contain public Git email addresses and must stay in an
untracked evidence directory.  The final ledger emits only SHA-256 identity
hashes, GitHub handles, PR numbers, and evidence references.
"""

from __future__ import annotations

import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
from typing import Any, Callable, Iterable, TypeVar


SCHEMA = "rhwp.release-contributor-audit.v1"
T = TypeVar("T")
PR_REFERENCE_RE = re.compile(r"(?<![\w])#([1-9][0-9]*)")
COAUTHOR_RE = re.compile(
    r"^Co-authored-by:\s*(.*?)\s*<([^>]+)>\s*$",
    re.IGNORECASE | re.MULTILINE,
)
ARCHIVE_PR_RE = re.compile(r"(?:^|/)pr_([1-9][0-9]+)(?:_|\.|$)")
NOREPLY_RE = re.compile(
    r"^(?:[0-9]+\+)?([^@]+)@users\.noreply\.github\.com$",
    re.IGNORECASE,
)
CHERRY_PICK_RE = re.compile(
    r"cherry picked from commit ([0-9a-f]{40})",
    re.IGNORECASE,
)


def run_git(repo: Path, args: list[str]) -> bytes:
    return subprocess.run(
        ["git", *args],
        cwd=repo,
        check=True,
        stdout=subprocess.PIPE,
    ).stdout


def run_gh(args: list[str]) -> bytes:
    return subprocess.run(
        ["gh", *args],
        check=True,
        stdout=subprocess.PIPE,
    ).stdout


def resolve_git_sha(repo: Path, revision: str) -> str:
    return (
        run_git(repo, ["rev-parse", "--verify", f"{revision}^{{commit}}"])
        .decode("ascii")
        .strip()
    )


def extract_pr_references(message: str) -> list[int]:
    return sorted({int(match) for match in PR_REFERENCE_RE.findall(message)})


def extract_coauthors(message: str) -> list[dict[str, str]]:
    return [
        {"name": name.strip(), "email": email.strip()}
        for name, email in COAUTHOR_RE.findall(message)
    ]


def extract_cherry_pick_sources(message: str) -> list[str]:
    return sorted({match.casefold() for match in CHERRY_PICK_RE.findall(message)})


def archive_pr_number(path: str) -> int | None:
    match = ARCHIVE_PR_RE.search(path)
    return int(match.group(1)) if match else None


def infer_github_handle(email: str) -> str | None:
    match = NOREPLY_RE.match(email.strip())
    return match.group(1) if match else None


def is_bot_handle(handle: str) -> bool:
    lowered = handle.casefold()
    return lowered.endswith("[bot]") or lowered in {"dependabot", "github-actions"}


def identity_key(name: str, email: str) -> str:
    if email.strip():
        return f"email:{email.strip().casefold()}"
    return f"name:{name.strip().casefold()}"


def identity_hash(name: str, email: str) -> str:
    key = identity_key(name, email).encode("utf-8")
    return hashlib.sha256(key).hexdigest()


def parse_git_log(raw: bytes) -> list[dict[str, str]]:
    commits: list[dict[str, str]] = []
    for raw_record in raw.split(b"\x1e"):
        raw_record = raw_record.strip(b"\r\n")
        if not raw_record:
            continue
        parts = raw_record.split(b"\x00", 3)
        if len(parts) != 4:
            raise ValueError("unexpected git log record")
        sha, name, email, message = (
            part.decode("utf-8", errors="replace").strip() for part in parts
        )
        commits.append(
            {"sha": sha, "authorName": name, "authorEmail": email, "message": message}
        )
    return commits


def candidate_document(
    *,
    base_sha: str,
    head_sha: str,
    commits: Iterable[dict[str, str]],
    archive_paths: Iterable[str],
) -> dict[str, Any]:
    pr_evidence: dict[int, set[str]] = defaultdict(set)
    identities: dict[str, dict[str, Any]] = {}
    cherry_pick_sources: dict[str, set[str]] = defaultdict(set)
    commit_count = 0

    def add_identity(name: str, email: str, role: str, sha: str) -> None:
        key = identity_key(name, email)
        item = identities.setdefault(
            key,
            {
                "name": name,
                "email": email,
                "roles": set(),
                "commitShas": set(),
                "authorCommitShas": set(),
                "coauthorCommitShas": set(),
                "inferredHandle": infer_github_handle(email),
            },
        )
        item["roles"].add(role)
        item["commitShas"].add(sha)
        item[f"{role}CommitShas"].add(sha)

    for commit in commits:
        commit_count += 1
        sha = commit["sha"]
        message = commit["message"]
        add_identity(commit["authorName"], commit["authorEmail"], "author", sha)
        for coauthor in extract_coauthors(message):
            add_identity(coauthor["name"], coauthor["email"], "coauthor", sha)
        for number in extract_pr_references(message):
            pr_evidence[number].add(f"commit:{sha}")
        for source_sha in extract_cherry_pick_sources(message):
            cherry_pick_sources[source_sha].add(sha)

    for path in archive_paths:
        number = archive_pr_number(path)
        if number is not None:
            pr_evidence[number].add(f"archive:{path}")

    return {
        "schema": SCHEMA,
        "kind": "candidates",
        "range": {
            "baseSha": base_sha,
            "headSha": head_sha,
            "commitCount": commit_count,
        },
        "prCandidates": [
            {"number": number, "evidence": sorted(evidence)}
            for number, evidence in sorted(pr_evidence.items())
        ],
        "cherryPickSources": [
            {
                "sourceSha": source_sha,
                "integratedCommitShas": sorted(integrated_shas),
            }
            for source_sha, integrated_shas in sorted(cherry_pick_sources.items())
        ],
        "gitIdentities": [
            {
                "name": item["name"],
                "email": item["email"],
                "roles": sorted(item["roles"]),
                "commitShas": sorted(item["commitShas"]),
                "authorCommitShas": sorted(item["authorCommitShas"]),
                "coauthorCommitShas": sorted(item["coauthorCommitShas"]),
                "inferredHandle": item["inferredHandle"],
            }
            for _, item in sorted(identities.items())
        ],
    }


def collect_candidates(repo: Path, base: str, head: str) -> dict[str, Any]:
    base_sha = resolve_git_sha(repo, base)
    head_sha = resolve_git_sha(repo, head)
    raw_log = run_git(
        repo,
        [
            "log",
            "--format=%H%x00%aN%x00%aE%x00%B%x1e",
            f"{base_sha}..{head_sha}",
        ],
    )
    raw_paths = run_git(
        repo,
        [
            "diff",
            "--name-only",
            "--diff-filter=A",
            "-z",
            f"{base_sha}..{head_sha}",
            "--",
            "mydocs/pr/archives",
        ],
    )
    paths = [
        path.decode("utf-8", errors="replace")
        for path in raw_paths.split(b"\x00")
        if path
    ]
    return candidate_document(
        base_sha=base_sha,
        head_sha=head_sha,
        commits=parse_git_log(raw_log),
        archive_paths=paths,
    )


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def github_records(value: Any) -> list[dict[str, Any]]:
    if isinstance(value, list):
        return value
    if isinstance(value, dict) and isinstance(value.get("records"), list):
        return value["records"]
    raise ValueError("GitHub metadata must be a list or an object with records")


def github_commit_authors(value: Any) -> dict[str, str]:
    if not isinstance(value, dict):
        return {}
    records = value.get("commitAuthors", [])
    if not isinstance(records, list):
        raise ValueError("GitHub commitAuthors must be a list")
    result: dict[str, str] = {}
    for item in records:
        handle = normalized_handle(item.get("author"))
        sha = item.get("sha")
        if isinstance(sha, str) and handle is not None:
            result[sha] = handle
    return result


def chunks(values: list[T], size: int) -> Iterable[list[T]]:
    for offset in range(0, len(values), size):
        yield values[offset : offset + size]


def graphql_number_query(numbers: list[int]) -> str:
    fields = "\n".join(
        f"""item_{number}: issueOrPullRequest(number: {number}) {{
          __typename
          ... on Issue {{ author {{ login }} }}
          ... on PullRequest {{
            author {{ login }}
            baseRefName
            headRefName
            merged
            mergedAt
            mergeCommit {{ oid }}
            state
          }}
        }}"""
        for number in numbers
    )
    return f"""query($owner: String!, $name: String!) {{
      repository(owner: $owner, name: $name) {{
        {fields}
      }}
    }}"""


def graphql_commit_query(shas: list[str]) -> str:
    fields = "\n".join(
        f"""commit_{index}: object(oid: \"{sha}\") {{
          ... on Commit {{
            oid
            associatedPullRequests(first: 20) {{
              nodes {{ number author {{ login }} state mergedAt }}
            }}
          }}
        }}"""
        for index, sha in enumerate(shas)
    )
    return f"""query($owner: String!, $name: String!) {{
      repository(owner: $owner, name: $name) {{
        {fields}
      }}
    }}"""


def resolve_github_metadata(
    candidates: dict[str, Any],
    repository: str,
    *,
    merged_base_ref: str | None = None,
    merged_search: str | None = None,
    gh_runner: Callable[[list[str]], bytes] = run_gh,
) -> dict[str, Any]:
    try:
        owner, name = repository.split("/", 1)
    except ValueError as error:
        raise ValueError("repository must use OWNER/NAME") from error
    if not owner or not name:
        raise ValueError("repository must use OWNER/NAME")
    if (merged_base_ref is None) != (merged_search is None):
        raise ValueError("merged base ref and search must be supplied together")

    numbers = sorted(int(item["number"]) for item in candidates["prCandidates"])
    records: list[dict[str, Any]] = []
    for batch in chunks(numbers, 50):
        payload = json.loads(
            gh_runner(
                [
                    "api",
                    "graphql",
                    "-f",
                    f"query={graphql_number_query(batch)}",
                    "-F",
                    f"owner={owner}",
                    "-F",
                    f"name={name}",
                ]
            ).decode("utf-8")
        )
        repository_value = payload["data"]["repository"]
        for number in batch:
            node = repository_value.get(f"item_{number}")
            if node is None:
                records.append({"number": number, "type": "Missing", "author": None})
                continue
            records.append(
                {
                    "number": number,
                    "type": node["__typename"],
                    "author": normalized_handle(node.get("author")),
                    "baseRefName": node.get("baseRefName"),
                    "headRefName": node.get("headRefName"),
                    "merged": node.get("merged"),
                    "mergedAt": node.get("mergedAt"),
                    "state": node.get("state"),
                    "mergeCommit": (
                        node.get("mergeCommit", {}).get("oid")
                        if node.get("mergeCommit")
                        else None
                    ),
                }
            )

    source_commit_pull_requests: list[dict[str, Any]] = []
    source_shas = [
        item["sourceSha"] for item in candidates.get("cherryPickSources", [])
    ]
    for batch in chunks(source_shas, 20):
        payload = json.loads(
            gh_runner(
                [
                    "api",
                    "graphql",
                    "-f",
                    f"query={graphql_commit_query(batch)}",
                    "-F",
                    f"owner={owner}",
                    "-F",
                    f"name={name}",
                ]
            ).decode("utf-8")
        )
        repository_value = payload["data"]["repository"]
        for index, sha in enumerate(batch):
            node = repository_value.get(f"commit_{index}")
            pull_requests = []
            if node is not None:
                pull_requests = [
                    {
                        "number": item["number"],
                        "author": normalized_handle(item.get("author")),
                        "state": item.get("state"),
                        "mergedAt": item.get("mergedAt"),
                    }
                    for item in node["associatedPullRequests"]["nodes"]
                ]
            source_commit_pull_requests.append(
                {
                    "sha": sha,
                    "resolved": node is not None,
                    "pullRequests": pull_requests,
                }
            )

    representative_shas = sorted(
        {
            identity["authorCommitShas"][0]
            for identity in candidates["gitIdentities"]
            if identity.get("authorCommitShas")
        }
    )
    commit_authors: list[dict[str, Any]] = []
    for sha in representative_shas:
        payload = json.loads(
            gh_runner(["api", f"repos/{repository}/commits/{sha}"]).decode("utf-8")
        )
        commit_authors.append(
            {
                "sha": sha,
                "author": normalized_handle(payload.get("author")),
            }
        )

    merged_pull_requests: list[dict[str, Any]] = []
    if merged_base_ref is not None and merged_search is not None:
        merged_value = json.loads(
            gh_runner(
                [
                    "pr",
                    "list",
                    "--repo",
                    repository,
                    "--state",
                    "merged",
                    "--base",
                    merged_base_ref,
                    "--limit",
                    "1000",
                    "--search",
                    merged_search,
                    "--json",
                    (
                        "number,title,author,labels,mergedAt,mergeCommit,url,"
                        "baseRefName,headRefName"
                    ),
                ]
            ).decode("utf-8")
        )
        if len(merged_value) >= 1000:
            raise ValueError("merged PR query reached the 1000 item safety limit")
        merged_pull_requests = [
            {
                "number": item["number"],
                "title": item["title"],
                "author": normalized_handle(item.get("author")),
                "labels": sorted(label["name"] for label in item.get("labels", [])),
                "mergedAt": item.get("mergedAt"),
                "mergeCommit": (
                    item.get("mergeCommit", {}).get("oid")
                    if item.get("mergeCommit")
                    else None
                ),
                "url": item.get("url"),
                "baseRefName": item.get("baseRefName"),
                "headRefName": item.get("headRefName"),
            }
            for item in merged_value
        ]

    return {
        "schema": SCHEMA,
        "kind": "github-metadata",
        "repository": repository,
        "range": candidates["range"],
        "records": records,
        "commitAuthors": commit_authors,
        "sourceCommitPullRequests": source_commit_pull_requests,
        "mergedPullRequestQuery": {
            "baseRefName": merged_base_ref,
            "search": merged_search,
        },
        "mergedPullRequests": merged_pull_requests,
    }


def normalized_handle(value: Any) -> str | None:
    if isinstance(value, str):
        return value.lstrip("@").strip() or None
    if isinstance(value, dict):
        login = value.get("login")
        if isinstance(login, str):
            return login.lstrip("@").strip() or None
    return None


def build_ledger(
    candidates: dict[str, Any],
    github_value: Any,
    overrides: dict[str, Any],
) -> dict[str, Any]:
    evidence_by_number = {
        int(item["number"]): list(item.get("evidence", []))
        for item in candidates["prCandidates"]
    }
    records = {int(item["number"]): item for item in github_records(github_value)}
    commit_authors = github_commit_authors(github_value)
    github_mapping = github_value if isinstance(github_value, dict) else {}
    source_commit_records = github_mapping.get("sourceCommitPullRequests", [])
    source_commit_pull_requests = {
        item["sha"]: item.get("pullRequests", [])
        for item in source_commit_records
    }
    unresolved_cherry_pick_sources = sorted(
        item["sha"] for item in source_commit_records if not item.get("resolved")
    )
    integrated_to_sources: dict[str, set[str]] = defaultdict(set)
    for item in candidates.get("cherryPickSources", []):
        for integrated_sha in item.get("integratedCommitShas", []):
            integrated_to_sources[integrated_sha].add(item["sourceSha"])
    dispositions = {
        int(number): reason
        for number, reason in overrides.get("numberDispositions", {}).items()
    }
    identity_overrides = {
        key.casefold(): value.lstrip("@")
        for key, value in overrides.get("identityToHandle", {}).items()
    }
    identity_hash_overrides = {
        key.casefold(): value.lstrip("@")
        for key, value in overrides.get("identityHashToHandle", {}).items()
    }
    identity_dispositions = {
        key.casefold(): reason
        for key, reason in overrides.get("identityDispositions", {}).items()
    }
    handle_aliases = {
        key.casefold(): value.lstrip("@")
        for key, value in overrides.get("handleAliases", {}).items()
    }

    release_commit_shas = {
        sha
        for identity in candidates["gitIdentities"]
        for sha in identity.get("commitShas", [])
    }

    contributors: dict[str, dict[str, Any]] = {}
    bots: dict[str, dict[str, Any]] = {}
    unresolved_numbers: list[int] = []
    excluded_pull_request_references: list[int] = []
    included_pull_request_numbers: set[int] = set()
    candidate_merged_pull_request_numbers: set[int] = set()
    base_merged_pull_request_numbers: set[int] = set()

    def person(handle: str) -> dict[str, Any]:
        handle = handle_aliases.get(handle.casefold(), handle)
        target = bots if is_bot_handle(handle) else contributors
        return target.setdefault(
            handle.casefold(),
            {
                "handle": handle,
                "prNumbers": set(),
                "gitIdentityHashes": set(),
                "evidence": set(),
            },
        )

    for number, evidence in sorted(evidence_by_number.items()):
        record = records.get(number)
        if record is None:
            if number not in dispositions:
                unresolved_numbers.append(number)
            continue
        if record.get("type") == "Issue":
            continue
        if record.get("type") != "PullRequest":
            if number not in dispositions:
                unresolved_numbers.append(number)
            continue
        merge_commit = record.get("mergeCommit")
        if merge_commit not in release_commit_shas:
            excluded_pull_request_references.append(number)
            continue
        handle = normalized_handle(record.get("author"))
        if handle is None:
            if number not in dispositions:
                unresolved_numbers.append(number)
            continue
        item = person(handle)
        item["prNumbers"].add(number)
        item["evidence"].update(evidence)
        included_pull_request_numbers.add(number)
        candidate_merged_pull_request_numbers.add(number)

    for record in github_mapping.get("mergedPullRequests", []):
        merge_commit = record.get("mergeCommit")
        if merge_commit not in release_commit_shas:
            continue
        handle = normalized_handle(record.get("author"))
        number = int(record["number"])
        if handle is None:
            if number not in dispositions:
                unresolved_numbers.append(number)
            continue
        item = person(handle)
        item["prNumbers"].add(number)
        item["evidence"].add(f"merge:{merge_commit}")
        included_pull_request_numbers.add(number)
        base_merged_pull_request_numbers.add(number)

    for source_sha, pull_requests in sorted(source_commit_pull_requests.items()):
        integrated_shas = sorted(
            sha
            for sha, sources in integrated_to_sources.items()
            if source_sha in sources
        )
        for record in pull_requests:
            handle = normalized_handle(record.get("author"))
            if handle is None:
                continue
            number = int(record["number"])
            item = person(handle)
            item["prNumbers"].add(number)
            item["evidence"].update(
                f"cherry-pick:{source_sha}->{sha}" for sha in integrated_shas
            )
            included_pull_request_numbers.add(number)

    excluded_pull_request_references = [
        number
        for number in excluded_pull_request_references
        if number not in included_pull_request_numbers
    ]

    unresolved_identities: list[dict[str, Any]] = []
    excluded_identities: list[dict[str, Any]] = []
    for identity in candidates["gitIdentities"]:
        name = identity.get("name", "")
        email = identity.get("email", "")
        digest = identity_hash(name, email)
        keys = [identity_key(name, email).casefold(), f"name:{name.casefold()}"]
        handle = identity_hash_overrides.get(
            digest.casefold(),
            next(
                (identity_overrides[key] for key in keys if key in identity_overrides),
                None,
            ),
        )
        if handle is None:
            handle = normalized_handle(identity.get("inferredHandle"))
        if handle is None:
            resolved_authors = {
                commit_authors[sha]
                for sha in identity.get("authorCommitShas", [])
                if sha in commit_authors
            }
            if len(resolved_authors) == 1:
                handle = resolved_authors.pop()
        disposition = identity_dispositions.get(digest.casefold())
        if disposition is not None:
            excluded_identities.append(
                {
                    "identitySha256": digest,
                    "reason": disposition,
                    "roles": sorted(identity.get("roles", [])),
                    "commitCount": len(identity.get("commitShas", [])),
                }
            )
            continue
        if handle is None:
            unresolved_identities.append(
                {
                    "identitySha256": digest,
                    "roles": sorted(identity.get("roles", [])),
                    "commitCount": len(identity.get("commitShas", [])),
                }
            )
            continue
        item = person(handle)
        item["gitIdentityHashes"].add(digest)
        item["evidence"].update(
            f"commit:{sha}" for sha in identity.get("commitShas", [])
        )

    for extra in overrides.get("additionalContributors", []):
        handle = normalized_handle(extra.get("handle"))
        if handle is None:
            raise ValueError("additional contributor requires handle")
        item = person(handle)
        reason = extra.get("reason", "explicit-override")
        item["evidence"].add(f"override:{reason}")
        extra_pr_numbers = {
            int(number) for number in extra.get("prNumbers", [])
        }
        item["prNumbers"].update(extra_pr_numbers)
        included_pull_request_numbers.update(extra_pr_numbers)

    def serialize(items: dict[str, dict[str, Any]]) -> list[dict[str, Any]]:
        return [
            {
                "handle": item["handle"],
                "prNumbers": sorted(item["prNumbers"]),
                "gitIdentityHashes": sorted(item["gitIdentityHashes"]),
                "evidence": sorted(item["evidence"]),
            }
            for _, item in sorted(items.items())
        ]

    issue_reference_count = sum(
        1 for record in records.values() if record.get("type") == "Issue"
    )
    pull_request_candidate_count = sum(
        1 for record in records.values() if record.get("type") == "PullRequest"
    )
    included_pull_request_count = len(included_pull_request_numbers)
    return {
        "schema": SCHEMA,
        "kind": "ledger",
        "range": candidates["range"],
        "counts": {
            "people": len(contributors),
            "bots": len(bots),
            "pullRequests": included_pull_request_count,
            "pullRequestCandidates": pull_request_candidate_count,
            "candidateMergedPullRequests": len(
                candidate_merged_pull_request_numbers
            ),
            "baseMergedPullRequests": len(base_merged_pull_request_numbers),
            "unreferencedBaseMergedPullRequests": len(
                base_merged_pull_request_numbers
                - candidate_merged_pull_request_numbers
            ),
            "excludedPullRequestReferences": len(excluded_pull_request_references),
            "issueReferences": issue_reference_count,
            "unresolvedNumbers": len(unresolved_numbers),
            "unresolvedGitIdentities": len(unresolved_identities),
            "excludedGitIdentities": len(excluded_identities),
            "cherryPickSources": len(source_commit_records),
            "resolvedCherryPickSourceObjects": sum(
                bool(item.get("resolved")) for item in source_commit_records
            ),
            "cherryPickSourcesWithAssociatedPrs": sum(
                bool(item.get("pullRequests")) for item in source_commit_records
            ),
        },
        "contributors": serialize(contributors),
        "bots": serialize(bots),
        "unresolvedNumbers": unresolved_numbers,
        "numberDispositions": [
            {"number": number, "reason": reason}
            for number, reason in sorted(dispositions.items())
        ],
        "excludedPullRequestReferences": excluded_pull_request_references,
        "unresolvedGitIdentities": sorted(
            unresolved_identities, key=lambda item: item["identitySha256"]
        ),
        "excludedGitIdentities": sorted(
            excluded_identities, key=lambda item: item["identitySha256"]
        ),
        "unresolvedCherryPickSources": unresolved_cherry_pick_sources,
    }


def emit_json(value: Any, output: str) -> None:
    text = json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    if output == "-":
        sys.stdout.write(text)
    else:
        Path(output).write_text(text, encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    candidates = subparsers.add_parser("candidates")
    candidates.add_argument("--repo", type=Path, default=Path.cwd())
    candidates.add_argument("--base", required=True)
    candidates.add_argument("--head", required=True)
    candidates.add_argument("--output", default="-")

    ledger = subparsers.add_parser("ledger")
    ledger.add_argument("--candidates", type=Path, required=True)
    ledger.add_argument("--github", type=Path, required=True)
    ledger.add_argument("--overrides", type=Path, required=True)
    ledger.add_argument("--output", default="-")
    ledger.add_argument("--require-resolved", action="store_true")

    github = subparsers.add_parser("github")
    github.add_argument("--candidates", type=Path, required=True)
    github.add_argument("--repository", required=True)
    github.add_argument("--merged-base-ref")
    github.add_argument("--merged-search")
    github.add_argument("--output", default="-")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.command == "candidates":
        value = collect_candidates(args.repo.resolve(), args.base, args.head)
    elif args.command == "github":
        value = resolve_github_metadata(
            load_json(args.candidates),
            args.repository,
            merged_base_ref=args.merged_base_ref,
            merged_search=args.merged_search,
        )
    else:
        value = build_ledger(
            load_json(args.candidates), load_json(args.github), load_json(args.overrides)
        )
        if args.require_resolved and (
            value["unresolvedNumbers"] or value["unresolvedGitIdentities"]
        ):
            emit_json(value, args.output)
            return 2
    emit_json(value, args.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
