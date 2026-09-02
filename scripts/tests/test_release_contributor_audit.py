from __future__ import annotations

import unittest

from scripts.release_contributor_audit import (
    archive_pr_number,
    build_ledger,
    candidate_document,
    extract_coauthors,
    extract_cherry_pick_sources,
    extract_pr_references,
    graphql_number_query,
    infer_github_handle,
    resolve_github_metadata,
)


class ReleaseContributorAuditTest(unittest.TestCase):
    def test_extracts_unique_references_and_coauthors(self) -> None:
        message = """Merge pull request #42

Refs #7 and #42. token#99 is not a GitHub reference.

Co-authored-by: Alice Example <1+alice@users.noreply.github.com>
Co-authored-by: Bob Example <bob@example.com>
"""
        self.assertEqual(extract_pr_references(message), [7, 42])
        self.assertEqual(
            extract_coauthors(message),
            [
                {
                    "name": "Alice Example",
                    "email": "1+alice@users.noreply.github.com",
                },
                {"name": "Bob Example", "email": "bob@example.com"},
            ],
        )
        self.assertEqual(
            extract_cherry_pick_sources(
                "(cherry picked from commit " + "A" * 40 + ")\n"
                "cherry picked from commit " + "a" * 40
            ),
            ["a" * 40],
        )

    def test_infers_github_noreply_handles(self) -> None:
        self.assertEqual(
            infer_github_handle("123+planet6897@users.noreply.github.com"),
            "planet6897",
        )
        self.assertEqual(
            infer_github_handle("kevin9327@users.noreply.github.com"), "kevin9327"
        )
        self.assertIsNone(infer_github_handle("person@example.com"))

    def test_extracts_pr_number_from_archive_variants(self) -> None:
        self.assertEqual(archive_pr_number("mydocs/pr/archives/pr_6583_review.md"), 6583)
        self.assertEqual(
            archive_pr_number("mydocs/pr/archives/pr_6564_review_impl.md"), 6564
        )
        self.assertIsNone(archive_pr_number("mydocs/orders/20260902.md"))

    def test_candidate_document_keeps_evidence_and_identity_roles(self) -> None:
        candidates = candidate_document(
            base_sha="a" * 40,
            head_sha="b" * 40,
            commits=[
                {
                    "sha": "1" * 40,
                    "authorName": "Alice",
                    "authorEmail": "1+alice@users.noreply.github.com",
                    "message": (
                        "fix: example (#42)\n\n"
                        "Co-authored-by: Bob <bob@example.com>"
                    ),
                },
                {
                    "sha": "2" * 40,
                    "authorName": "Alice",
                    "authorEmail": "1+alice@users.noreply.github.com",
                    "message": "Refs #7",
                },
            ],
            archive_paths=["mydocs/pr/archives/pr_42_review.md"],
        )
        self.assertEqual(candidates["range"]["commitCount"], 2)
        self.assertEqual(
            candidates["prCandidates"],
            [
                {"number": 7, "evidence": [f"commit:{'2' * 40}"]},
                {
                    "number": 42,
                    "evidence": [
                        "archive:mydocs/pr/archives/pr_42_review.md",
                        f"commit:{'1' * 40}",
                    ],
                },
            ],
        )
        by_name = {item["name"]: item for item in candidates["gitIdentities"]}
        self.assertEqual(by_name["Alice"]["roles"], ["author"])
        self.assertEqual(by_name["Alice"]["inferredHandle"], "alice")
        self.assertEqual(
            by_name["Alice"]["authorCommitShas"], ["1" * 40, "2" * 40]
        )
        self.assertEqual(by_name["Bob"]["roles"], ["coauthor"])
        self.assertEqual(by_name["Bob"]["authorCommitShas"], [])

    def test_github_metadata_resolves_number_types_and_primary_authors(self) -> None:
        candidates = candidate_document(
            base_sha="a" * 40,
            head_sha="b" * 40,
            commits=[
                {
                    "sha": "1" * 40,
                    "authorName": "Direct",
                    "authorEmail": "direct@example.com",
                    "message": "Refs #7 and #42",
                },
                {
                    "sha": "2" * 40,
                    "authorName": "Second",
                    "authorEmail": "second@example.com",
                    "message": "Refs #99",
                },
            ],
            archive_paths=[],
        )

        def fake_gh(args: list[str]) -> bytes:
            if args[0:2] == ["api", "graphql"]:
                query = args[3].removeprefix("query=")
                self.assertIn("item_42", query)
                return b'''{"data":{"repository":{
                  "item_7":{"__typename":"Issue","author":{"login":"reporter"}},
                  "item_42":{"__typename":"PullRequest","author":{"login":"author"},
                    "baseRefName":"devel","headRefName":"task","state":"MERGED",
                    "merged":true,"mergedAt":"2026-09-01T00:00:00Z",
                    "mergeCommit":{"oid":"1111111111111111111111111111111111111111"}},
                  "item_99":null}}}'''
            sha = args[1].rsplit("/", 1)[-1]
            author = "direct" if sha == "1" * 40 else None
            return (
                '{"sha":"%s","author":%s}'
                % (sha, "null" if author is None else '{"login":"direct"}')
            ).encode()

        metadata = resolve_github_metadata(
            candidates, "edwardkim/rhwp", gh_runner=fake_gh
        )
        self.assertEqual(
            [(item["number"], item["type"]) for item in metadata["records"]],
            [(7, "Issue"), (42, "PullRequest"), (99, "Missing")],
        )
        self.assertEqual(
            metadata["commitAuthors"],
            [
                {"sha": "1" * 40, "author": "direct"},
                {"sha": "2" * 40, "author": None},
            ],
        )
        self.assertIn("issueOrPullRequest(number: 7)", graphql_number_query([7]))

    def test_ledger_separates_issues_bots_and_unresolved_identities(self) -> None:
        candidates = candidate_document(
            base_sha="a" * 40,
            head_sha="b" * 40,
            commits=[
                {
                    "sha": "1" * 40,
                    "authorName": "Alice",
                    "authorEmail": "1+alice@users.noreply.github.com",
                    "message": "Merge #42 and issue #7",
                },
                {
                    "sha": "2" * 40,
                    "authorName": "Unknown",
                    "authorEmail": "unknown@example.com",
                    "message": "direct change",
                },
            ],
            archive_paths=[],
        )
        ledger = build_ledger(
            candidates,
            {
                "records": [
                    {"number": 7, "type": "Issue", "author": "reporter"},
                    {
                        "number": 42,
                        "type": "PullRequest",
                        "author": "dependabot[bot]",
                        "mergeCommit": "1" * 40,
                    },
                ],
                "commitAuthors": [],
            },
            {"identityToHandle": {}},
        )
        self.assertEqual([item["handle"] for item in ledger["contributors"]], ["alice"])
        self.assertEqual([item["handle"] for item in ledger["bots"]], ["dependabot[bot]"])
        self.assertEqual(ledger["counts"]["issueReferences"], 1)
        self.assertEqual(ledger["counts"]["unresolvedGitIdentities"], 1)
        unresolved = ledger["unresolvedGitIdentities"][0]
        self.assertNotIn("email", unresolved)
        self.assertIn("identitySha256", unresolved)

    def test_ledger_uses_primary_commit_author_but_not_coauthor_commit(self) -> None:
        candidates = candidate_document(
            base_sha="a" * 40,
            head_sha="b" * 40,
            commits=[
                {
                    "sha": "1" * 40,
                    "authorName": "Direct",
                    "authorEmail": "direct@example.com",
                    "message": "Co-authored-by: Other <other@example.com>",
                }
            ],
            archive_paths=[],
        )
        ledger = build_ledger(
            candidates,
            {
                "records": [],
                "commitAuthors": [{"sha": "1" * 40, "author": "direct"}],
            },
            {"identityToHandle": {}},
        )
        self.assertEqual(
            [item["handle"] for item in ledger["contributors"]], ["direct"]
        )
        self.assertEqual(ledger["counts"]["unresolvedGitIdentities"], 1)

    def test_ledger_credits_closed_pr_associated_with_cherry_pick(self) -> None:
        source_sha = "f" * 40
        integrated_sha = "1" * 40
        candidates = candidate_document(
            base_sha="a" * 40,
            head_sha="b" * 40,
            commits=[
                {
                    "sha": integrated_sha,
                    "authorName": "Contributor",
                    "authorEmail": "1+contributor@users.noreply.github.com",
                    "message": f"fix\n\n(cherry picked from commit {source_sha})",
                }
            ],
            archive_paths=[],
        )
        ledger = build_ledger(
            candidates,
            {
                "records": [],
                "commitAuthors": [],
                "sourceCommitPullRequests": [
                    {
                        "sha": source_sha,
                        "resolved": True,
                        "pullRequests": [
                            {
                                "number": 77,
                                "author": "contributor",
                                "state": "CLOSED",
                                "mergedAt": None,
                            }
                        ],
                    }
                ],
            },
            {"identityToHandle": {}},
        )
        contributor = ledger["contributors"][0]
        self.assertEqual(contributor["handle"], "contributor")
        self.assertEqual(contributor["prNumbers"], [77])
        self.assertEqual(ledger["counts"]["pullRequests"], 1)
        self.assertEqual(ledger["counts"]["cherryPickSources"], 1)
        self.assertEqual(ledger["counts"]["resolvedCherryPickSourceObjects"], 1)

    def test_ledger_adds_merged_pr_missing_from_text_candidates(self) -> None:
        integrated_sha = "1" * 40
        candidates = candidate_document(
            base_sha="a" * 40,
            head_sha="b" * 40,
            commits=[
                {
                    "sha": integrated_sha,
                    "authorName": "Maintainer",
                    "authorEmail": "1+maintainer@users.noreply.github.com",
                    "message": "merge without textual PR number",
                }
            ],
            archive_paths=[],
        )
        ledger = build_ledger(
            candidates,
            {
                "records": [],
                "commitAuthors": [],
                "sourceCommitPullRequests": [],
                "mergedPullRequests": [
                    {
                        "number": 88,
                        "author": "maintainer",
                        "mergeCommit": integrated_sha,
                    }
                ],
            },
            {"identityToHandle": {}},
        )
        self.assertEqual(ledger["contributors"][0]["prNumbers"], [88])
        self.assertEqual(ledger["counts"]["baseMergedPullRequests"], 1)
        self.assertEqual(ledger["counts"]["unreferencedBaseMergedPullRequests"], 1)

    def test_overrides_merge_direct_commits_and_pr_authorship(self) -> None:
        candidates = candidate_document(
            base_sha="a" * 40,
            head_sha="b" * 40,
            commits=[
                {
                    "sha": "1" * 40,
                    "authorName": "Maintainer",
                    "authorEmail": "maintainer@example.com",
                    "message": "Integrate #99",
                }
            ],
            archive_paths=[],
        )
        ledger = build_ledger(
            candidates,
            [
                {
                    "number": 99,
                    "type": "PullRequest",
                    "author": "external",
                    "mergeCommit": "1" * 40,
                }
            ],
            {
                "identityHashToHandle": {
                    "01cfc2529c048604a1d7a70213cc3eedbe166ce0019e72c5cdfe65319b3b5479": "maintainer"
                },
                "additionalContributors": [
                    {"handle": "security-reporter", "reason": "advisory-credit"}
                ],
            },
        )
        by_handle = {item["handle"]: item for item in ledger["contributors"]}
        self.assertEqual(set(by_handle), {"external", "maintainer", "security-reporter"})
        self.assertEqual(by_handle["external"]["prNumbers"], [99])
        self.assertEqual(ledger["unresolvedGitIdentities"], [])


if __name__ == "__main__":
    unittest.main()
