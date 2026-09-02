"""v0.8.6 공개 릴리스 기록과 contributor ledger의 정합성 계약."""

from __future__ import annotations

import json
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
LEDGER = (
    REPO_ROOT
    / "mydocs/tech/investigations/issue-6584/release_contributor_ledger.json"
)
RELEASE_RECORDS = (
    REPO_ROOT / "CHANGELOG.md",
    REPO_ROOT / "CHANGELOG_EN.md",
    REPO_ROOT / "mydocs/working/task_m100_6584_release_notes.md",
)
START = "<!-- release-contributors:start -->"
END = "<!-- release-contributors:end -->"


def contributor_keys(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    if text.count(START) != 1 or text.count(END) != 1:
        raise AssertionError(f"{path}: contributor marker는 각각 하나여야 한다")
    block = text.split(START, maxsplit=1)[1].split(END, maxsplit=1)[0]
    keys: list[str] = []
    for line in block.splitlines():
        if not line.startswith("- "):
            continue
        credit = line.removeprefix("- ").split(maxsplit=1)[0]
        keys.append(credit.removeprefix("@"))
    return keys


class ReleaseRecordContributorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.ledger = json.loads(LEDGER.read_text(encoding="utf-8"))
        cls.expected = [item["handle"] for item in cls.ledger["contributors"]]

    def test_public_release_records_match_human_ledger_exactly(self) -> None:
        self.assertEqual(self.ledger["counts"]["people"], 20)
        self.assertEqual(len(self.expected), 20)
        for path in RELEASE_RECORDS:
            with self.subTest(path=path.relative_to(REPO_ROOT)):
                self.assertEqual(contributor_keys(path), self.expected)

    def test_public_release_records_exclude_bots(self) -> None:
        bot_handles = {item["handle"] for item in self.ledger["bots"]}
        self.assertEqual(bot_handles, {"dependabot[bot]"})
        for path in RELEASE_RECORDS:
            with self.subTest(path=path.relative_to(REPO_ROOT)):
                self.assertTrue(bot_handles.isdisjoint(contributor_keys(path)))

    def test_records_identify_v086(self) -> None:
        headings = {
            RELEASE_RECORDS[0]: "## [0.8.6] — 2026-09-02",
            RELEASE_RECORDS[1]: "## [0.8.6] — 2026-09-02",
            RELEASE_RECORDS[2]: "# rhwp v0.8.6",
        }
        for path, heading in headings.items():
            with self.subTest(path=path.relative_to(REPO_ROOT)):
                self.assertIn(heading, path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
