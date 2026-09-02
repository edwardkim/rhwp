"""[#5311] rhwp-bulk-pipeline 스킬·픽스처 계약.

실 에이전트가 폴더의 HWP/HWPX 를 batch 로 처리할 때 쓰는 규약이
기존 CLI 표면(info / export-text / export-structure / export-tables /
fields / search / extract-data / convert / fill)을 벗어나지 않는지,
gym 과 새 CLI 를 끌어들이지 않았는지를 바이너리 없이 커밋된 파일만으로
검사한다.
"""

from __future__ import annotations

import importlib.util
import json
import unittest
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".claude" / "skills" / "rhwp-bulk-pipeline"
REF = SKILL / "references"
FIXT = SKILL / "fixtures"
EX = SKILL / "examples"
GEN = REF / "_gen_pack.py"
WORKING = REPO / "mydocs" / "working" / "agent_bulk_pipeline.md"

INVENTED_COMMANDS = [
    "batch merge",
    "batch export-markdown",
    "batch thumbnail",
    "batch redact",
    "batch mail-merge",
    "hwp_batch_convert_write",
]


def load_gen():
    spec = importlib.util.spec_from_file_location("rhwp_bulk_pipeline_gen", GEN)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(name: str):
    return json.loads((FIXT / name).read_text(encoding="utf-8"))


class AgentBulkPipelineSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.gen = load_gen()
        cls.skill = read(SKILL / "SKILL.md")
        cls.idx = load_json("skill_index.json")
        cls.tree = load_json("tree.json")
        cls.stops = load_json("stop_rules.json")
        cls.axes = load_json("axes.json")
        cls.intents = load_json("intent_matrix.json")
        cls.journeys = load_json("journeys.json")
        cls.gate = load_json("recipe9_gate.json")
        cls.env = load_json("envelopes.json")
        cls.exits = load_json("exit_codes.json")
        cls.pw = load_json("password_reject.json")
        cls.conv = load_json("convert_names.json")
        cls.fill = load_json("fill_contract.json")
        cls.traces = load_json("traces_index.json")

    def test_skill_front_matter_and_not_gym(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: rhwp-bulk-pipeline", self.skill)
        self.assertNotRegex(self.skill, r"(?m)^gym/")
        self.assertIn("gym 이 아니고", self.skill)
        self.assertIn("새 CLI", self.skill)

    def test_required_topics(self):
        for needle in (
            "batch info",
            "batch export-text",
            "batch extract-data",
            "batch convert",
            "batch fill",
            "NDJSON",
            "exitClass",
            "N=성공+실패",
            "--password",
            "--query",
            "--out-dir",
            "references/15_no_global_password.md",
            "references/16_convert_name_reservation.md",
            "references/17_fill_not_stdin.md",
            "references/18_exit_aggregation.md",
        ):
            self.assertIn(needle, self.skill, needle)

    def test_no_invented_commands(self):
        # 금지 목록·발화 행렬은 발명 이름을 인용할 수 있다.
        # 요청→명령 표가 그걸 처방으로 내리면 실패.
        table = self.skill.split("## 요청 → 명령", 1)[1].split("## 정지 규칙", 1)[0]
        for cmd in INVENTED_COMMANDS:
            self.assertNotIn(cmd, table, cmd)
        for axis in ("info", "export-text", "extract-data", "convert", "fill"):
            self.assertIn(f"batch {axis}", table)

    def test_references_exist_and_long_enough(self):
        refs = self.idx["references"]
        self.assertGreaterEqual(len(refs), 30)
        for name in refs:
            path = REF / name
            self.assertTrue(path.is_file(), name)
            self.assertGreater(path.stat().st_size, 200, name)

    def test_examples_exist(self):
        for name in self.idx["examples"]:
            path = EX / name
            self.assertTrue(path.is_file(), name)

    def test_schema_issue(self):
        for name in (
            "skill_index.json",
            "tree.json",
            "stop_rules.json",
            "gate.json",
            "journeys.json",
        ):
            v = load_json(name)
            self.assertEqual(v["schemaVersion"], "1.0", name)
            self.assertEqual(v["issue"], 5311, name)
            self.assertTrue(v["notGym"], name)
            self.assertTrue(v["noNewCli"], name)

    def test_nine_axes(self):
        ids = [a["id"] for a in self.axes["axes"]]
        self.assertEqual(len(ids), 9)
        self.assertIn("extract-data", ids)
        self.assertIn("convert", ids)
        self.assertIn("fill", ids)
        fill = next(a for a in self.axes["axes"] if a["id"] == "fill")
        self.assertFalse(fill["stdin"])

    def test_journeys_stop_ids(self):
        ids = {r["id"] for r in self.stops["rules"]}
        items = self.journeys["journeys"]
        self.assertGreaterEqual(len(items), 80)
        for j in items:
            self.assertIn(j["stop"], ids, j["id"])
            self.assertTrue(j["notGym"])
            self.assertGreater(len(j["steps"]), 0)

    def test_intents(self):
        items = self.intents["intents"]
        self.assertGreaterEqual(len(items), 100)
        utter = "\n".join(i["utterance"] for i in items)
        self.assertIn("메일머지", utter)
        self.assertTrue(any("password" in i["command"] or "비밀번호" in i["utterance"] for i in items))

    def test_recipe9_gate(self):
        self.assertEqual(self.gate["input"], 5)
        self.assertEqual(self.gate["success"], 4)
        self.assertEqual(self.gate["failure"], 1)
        self.assertEqual(self.gate["exit"], 1)
        recs = load_json("export_text_rows.json")["rows"]
        self.assertEqual(len(recs), 5)
        self.assertEqual(sum(1 for r in recs if "error" in r), 1)
        self.assertEqual(recs[-1]["exitClass"], "runtime")

    def test_password_reject(self):
        self.assertEqual(self.pw["exit"], 2)
        self.assertFalse(self.pw["consumesStdin"])
        self.assertIn("--password", self.pw["rejectedFlags"])

    def test_convert_names(self):
        self.assertTrue(self.conv["reserveBeforeWrite"])
        self.assertFalse(self.conv["partialWrite"])
        self.assertEqual(self.conv["exitOnCollision"], 2)
        self.assertTrue(self.conv["mcpExcluded"])

    def test_fill_contract(self):
        self.assertTrue(self.fill["stdinIsNotFileList"])
        self.assertTrue(self.fill["dryRunStillNeedsOutDir"])
        self.assertEqual(self.fill["emptyCsvExit"], 2)

    def test_exit_codes(self):
        codes = {r["code"] for r in self.exits["aggregation"]}
        self.assertEqual(codes, {0, 1, 2, 3, 4})

    def test_transcripts_ndjson(self):
        for t in self.traces["traces"]:
            path = SKILL / t["transcript"]
            self.assertTrue(path.is_file(), path)
            text = path.read_text(encoding="utf-8")
            if t["exit"] == 2:
                self.assertEqual(text.strip(), "")
                continue
            for line in text.splitlines():
                if not line.strip():
                    continue
                rec = json.loads(line)
                self.assertEqual(rec["schemaVersion"], "1.0")

    def test_order_preserved_in_t02(self):
        paths = [ln for ln in read(EX / "lists" / "recipe9.txt").splitlines() if ln]
        recs = [
            json.loads(ln)
            for ln in read(EX / "transcripts" / "T02.ndjson").splitlines()
            if ln
        ]
        self.assertEqual([r["source"] for r in recs], paths)

    def test_extract_limit_per_doc(self):
        recs = load_json("extract_rows.json")["rows"]
        first = next(r for r in recs if "국립국어원" in r["source"])
        self.assertEqual(first["itemCount"], 3)
        self.assertEqual(first["totalItemCount"], 297)
        self.assertTrue(first["truncated"])

    def test_working_doc(self):
        text = read(WORKING)
        self.assertIn("#5311", text)
        self.assertIn("rhwp-bulk-pipeline", text)

    def test_gen_issue_constant(self):
        self.assertEqual(self.gen.ISSUE, 5311)
        self.assertEqual(self.gen.SCHEMA, "1.0")
        self.assertFalse(self.gen.AXES[-1]["stdin"])

    def test_forbidden_trees(self):
        self.assertIn("gym/", self.idx["forbiddenTrees"])


if __name__ == "__main__":
    unittest.main()
