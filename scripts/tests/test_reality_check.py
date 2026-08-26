"""[#4728] 외부 검증 축 계약 — 정직 불변식.

핵심 불변식: 현실 채점은 **프로젝트 견인(코어)**과 **메타-시스템 외부 채택**을
절대 뭉뚱그리지 않는다. 뭉뚱그리는 순간 이 축은 self-graded 로 전락한다. 그래서
메타 외부 채택은 프로젝트 ★·fork 를 포함하지 않고, 그 자체(준수자·참조·재현)로만
집계돼야 한다.
"""

from __future__ import annotations

import copy
import importlib.util
import json
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL = REPO_ROOT / "tools" / "reality_check.py"
SIGNALS = REPO_ROOT / "mydocs" / "tech" / "agent_frame" / "external_signals.json"


def load_tool():
    spec = importlib.util.spec_from_file_location("reality_check", TOOL)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class RealityCheckTests(unittest.TestCase):
    def test_signals_snapshot_is_well_formed(self):
        sig = json.loads(SIGNALS.read_text(encoding="utf-8"))
        self.assertIn("project", sig)
        self.assertIn("metaSystem", sig)
        self.assertTrue(sig.get("externalValidationCriteria"),
                        "외부 검증 기준이 비었다 — 무엇을 외부 검증으로 칠지 없다")

    def test_scorecard_separates_project_from_meta(self):
        """정직 불변식 — 메타 외부 채택은 프로젝트 견인을 포함하지 않는다."""
        tool = load_tool()
        sig = json.loads(SIGNALS.read_text(encoding="utf-8"))
        sig["project"]["stars"] = 1
        sig["metaSystem"].update({
            "externalConformers": 2,
            "externalReferrers": 3,
            "thirdPartyReproductions": 5,
        })
        card = tool.scorecard(sig)
        self.assertIn("projectTraction", card)
        self.assertIn("metaSystemExternalAdoption", card)
        meta = card["metaSystemExternalAdoption"]
        proj = card["projectTraction"]
        # 메타 총합은 준수자·참조·재현의 합이지 ★·fork 가 아니다. 외부 채택 수가
        # 프로젝트 별 수보다 클 수도 있으므로 양쪽 크기를 비교하지 않는다.
        self.assertEqual(meta["total"], 10)
        self.assertEqual(meta["total"],
                         meta["conformers"] + meta["referrers"] + meta["reproductions"])
        self.assertEqual(proj["stars"], 1)

    def test_meta_adoption_is_reported_honestly_not_inflated(self):
        """메타 외부 채택은 부풀리지 않는다 — 스냅샷이 0 이면 채점도 0."""
        tool = load_tool()
        sig = json.loads(SIGNALS.read_text(encoding="utf-8"))
        m = sig["metaSystem"]
        declared = (m.get("externalConformers", 0) + m.get("externalReferrers", 0)
                    + m.get("thirdPartyReproductions", 0))
        card = tool.scorecard(sig)
        self.assertEqual(card["metaSystemExternalAdoption"]["total"], declared)

    def test_verdict_names_self_graded_when_zero(self):
        tool = load_tool()
        sig = json.loads(SIGNALS.read_text(encoding="utf-8"))
        # 메타 채택 0 인 스냅샷에서 판정이 self-graded 를 숨기지 않는다.
        card = tool.scorecard(sig)
        if card["metaSystemExternalAdoption"]["total"] == 0:
            self.assertIn("self-graded", card["verdict"])

    def test_live_refresh_updates_github_npm_and_measurement_date(self):
        tool = load_tool()
        original_gh = tool._gh_api
        original_npm = tool._npm_monthly_downloads
        try:
            tool._gh_api = lambda path, _jq: (
                {"stars": 10, "forks": 2, "watchers": 3, "openIssues": 4}
                if path.startswith("repos/") and "/contributors" not in path else 5
            )
            tool._npm_monthly_downloads = lambda package: 7 if package == "@rhwp/editor" else None
            refreshed = tool.refresh_live(copy.deepcopy(json.loads(SIGNALS.read_text(encoding="utf-8"))))
        finally:
            tool._gh_api = original_gh
            tool._npm_monthly_downloads = original_npm

        self.assertEqual(refreshed["project"]["stars"], 10)
        self.assertEqual(refreshed["project"]["contributors"], 5)
        self.assertEqual(refreshed["project"]["npmMonthlyDownloads"]["@rhwp/editor"], 7)
        self.assertRegex(refreshed["measuredAt"], r"^\d{4}-\d{2}-\d{2}$")


if __name__ == "__main__":
    unittest.main()
