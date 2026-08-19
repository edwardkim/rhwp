"""#5511 CLI catalog와 agent preflight의 help 가시성 계보 회귀 테스트."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MODULE_PATH = REPO_ROOT / "tools" / "agent_preflight.py"


def load_module():
    spec = importlib.util.spec_from_file_location("agent_preflight_surface_test", MODULE_PATH)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class HelpHiddenCatalogTests(unittest.TestCase):
    def test_repository_catalog_is_the_help_hidden_authority(self):
        module = load_module()
        report = module.Report()

        hidden = module.load_help_hidden(REPO_ROOT, report)

        self.assertEqual(hidden, {"core-pages", "dump-extents", "measure-width"})
        self.assertEqual(report.skipped, [])

    def test_public_command_before_hidden_command_is_not_misattributed(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as tmp:
            repo = Path(tmp)
            catalog = repo / module.CLI_CATALOG
            catalog.parent.mkdir(parents=True)
            catalog.write_text(
                '''
                const COMMANDS: &[CommandSpec] = &[
                    spec(
                        "public-command",
                        Category::Query,
                        Visibility::Public,
                        None,
                        true,
                        false,
                        true,
                    ),
                    spec(
                        "hidden-command",
                        Category::Diagnostic,
                        Visibility::Hidden("internal probe"),
                        None,
                        false,
                        false,
                        false,
                    ),
                ];
                ''',
                encoding="utf-8",
            )
            report = module.Report()

            hidden = module.load_help_hidden(repo, report)

        self.assertEqual(hidden, {"hidden-command"})
        self.assertEqual(report.skipped, [])


if __name__ == "__main__":
    unittest.main()
