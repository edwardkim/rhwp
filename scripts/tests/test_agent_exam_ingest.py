"""[#5319] rhwp-exam-ingest 스킬 고도화 계약.

실 에이전트가 PDF/이미지/MD/DOCX 시험지를 기존 ingest 경로로
HWPX 로 바꾸도록 문서·픽스처·헬퍼 dry 경로가 같은 단어를 쓰는지
파일만으로 고정한다.

새 CLI 를 시험하지 않는다. DocumentCore exam_paper 를 시험하지 않는다.
gym/ 을 열지 않는다. 네트워크를 부르지 않는다.
poppler/ImageMagick 이 없는 환경에서도 통과해야 한다.

정본: .claude/skills/rhwp-exam-ingest/
작업 기록: mydocs/working/archives/agent_exam_ingest.md
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
import unittest
from pathlib import Path


REPO = Path(__file__).resolve().parents[2]
SKILL = REPO / ".claude" / "skills" / "rhwp-exam-ingest"
SKILL_MD = SKILL / "SKILL.md"
REFS = SKILL / "references"
EXAMPLES = SKILL / "examples"
FIXTURES = SKILL / "fixtures"
HELPERS = SKILL / "helpers"
ENVS = FIXTURES / "envelopes"
SCHEMAS = FIXTURES / "schemas"
MATS = FIXTURES / "matrices"
TRANS = FIXTURES / "transcripts"
HFIX = FIXTURES / "helpers"
WORKING = REPO / "mydocs" / "working" / "archives" / "agent_exam_ingest.md"
CANON_SCHEMA = REPO / "tools" / "rhwp-ingest" / "schema" / "ingest_schema_v1.json"

ISSUE = 5319

REFERENCE_NAMES = (
    "00_tree.md",
    "01_input_normalize.md",
    "02_pdf_to_pngs.md",
    "03_extract_docx.md",
    "04_image_passthrough.md",
    "05_md_image_refs.md",
    "06_ingest_schema_v1.md",
    "07_passages_questions.md",
    "08_stem_blocks_boxed.md",
    "09_media_placement.md",
    "10_auto_number.md",
    "11_crop_bbox.md",
    "12_build_from_ingest.md",
    "13_check_deps.md",
    "14_failure_envelopes.md",
    "15_known_limits.md",
    "16_pitfalls.md",
    "17_sample_transcripts.md",
    "18_verify_gate.md",
    "19_intent_matrix.md",
    "20_exit_codes.md",
    "README.md",
)

SIBLING_SKILLS = (
    "rhwp-form-fill",
    "rhwp-table-exchange",
    "rhwp-onboarding",
    "rhwp-safe-edit",
    "rhwp-doc-triage",
)

INVENTED_COMMANDS = (
    "exam-from-pdf",
    "ingest-exam",
    "build-exam",
    "hwp_doc_exam",
    "rhwp crop-image",
    "rhwp pdf-to-png",
    "rhwp import-md",
    "edit exam",
)

SKILL_TOKENS = (
    "새 CLI",
    "gym",
    "pdf_to_pngs.sh",
    "extract_docx.py",
    "crop_image.sh",
    "check_deps.sh",
    "build-from-ingest",
    "--media-dir",
    "auto_number",
    "passages",
    "stem_blocks",
    "boxed",
    "between",
    "above",
    "below",
    "inline",
    "DEP_MISS_POPPLER",
    "DEP_MISS_IMAGEMAGICK",
    "DEP_MISS_PYTHON_DOCX",
    "Picture",
    "references/06_ingest_schema_v1.md",
    "fixtures/catalog.json",
)

WORKING_TOKENS = (
    "#5319",
    "rhwp-exam-ingest",
    "build-from-ingest",
    "pdf_to_pngs.sh",
    "auto_number",
    "gym",
    "5000",
    "exam_paper",
    "--media-dir",
)

REF_TOKENS = {
    "00_tree.md": ("pdf_to_pngs.sh", "build-from-ingest", "exam-from-pdf", "살아 있는 동사"),
    "01_input_normalize.md": ("page_001.png", "--media-dir", "dry-run", "PDF_SRC_MISSING"),
    "02_pdf_to_pngs.md": ("pdftoppm", "page_001.png", "PDF_MISS_TOOLS", "10#$n"),
    "03_extract_docx.md": ("python-docx", "zip-regex-fallback", "DOCX_SRC_MISSING", "word/media"),
    "04_image_passthrough.md": ("패스스루", "crop_image.sh", "픽셀", "F10"),
    "05_md_image_refs.md": ("![alt](path)", "media[].id", "auto_number", "F07"),
    "06_ingest_schema_v1.md": ("deny_unknown_fields", "version", "StemBlock", "#3358"),
    "07_passages_questions.md": ("passage_ref", "passages[]", "한 번만", "①"),
    "08_stem_blocks_boxed.md": ("<보기>", "blocks", "boxed", "text"),
    "09_media_placement.md": ("between", "above", "below", "inline"),
    "10_auto_number.md": ("auto_number", "false", "true", "2. 2."),
    "11_crop_bbox.md": ("10진 정수", "CROP_BBOX", "--dry-run", "+repage"),
    "12_build_from_ingest.md": ("--media-dir", "-o", "export-text", "발명"),
    "13_check_deps.md": ("DEP_MISS_POPPLER", "DEP_MISS_IMAGEMAGICK", "DEP_MISS_PYTHON_DOCX", "--json"),
    "14_failure_envelopes.md": ("schemaVersion", "CROP_OK", "PDF_MISS_TOOLS", "exit"),
    "15_known_limits.md": ("#182", "수식", "표", "exam_paper"),
    "16_pitfalls.md": ("auto_number", "deny_unknown", "page-1.png", "OCR"),
    "17_sample_transcripts.md": ("T01", "T06", "T08", "build-from-ingest"),
    "18_verify_gate.md": ("export-text", "dump", "N. N.", "unzip"),
    "19_intent_matrix.md": ("I001", "exam-from-pdf", "F16", "발화"),
    "20_exit_codes.md": ("#2707", "exit 4", "check_deps.sh", "build-from-ingest"),
}

DEP_CODES = (
    "DEP_MISS_RHWP",
    "DEP_MISS_IMAGEMAGICK",
    "DEP_MISS_POPPLER",
    "DEP_MISS_PYTHON_DOCX",
)

PLACEMENTS = ("between", "above", "below", "inline")


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def load_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


class AgentExamIngestSkillTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.skill = read(SKILL_MD)
        cls.catalog = load_json(FIXTURES / "catalog.json")
        cls.index = load_json(FIXTURES / "skill_index.json")
        cls.intents = load_json(MATS / "intent_matrix.json")
        cls.working = read(WORKING)

    def test_skill_front_matter_and_not_gym(self):
        self.assertTrue(self.skill.startswith("---\n"))
        self.assertIn("name: rhwp-exam-ingest", self.skill)
        self.assertNotRegex(self.skill, r"(?m)^gym/")
        self.assertIn("gym 이 아니고", self.skill)
        self.assertIn("새 CLI", self.skill)
        self.assertIn("exam_paper", self.skill)

    def test_skill_tokens(self):
        for needle in SKILL_TOKENS:
            self.assertIn(needle, self.skill, f"SKILL.md 에 없음: {needle}")

    def test_reference_docs_exist_and_long_enough(self):
        for name in REFERENCE_NAMES:
            path = REFS / name
            self.assertTrue(path.is_file(), name)
            body = read(path)
            self.assertGreater(len(body), 400, f"{name} 가 너무 짧다")

    def test_index_lists_same_references(self):
        listed = self.index["references"]
        for name in REFERENCE_NAMES:
            self.assertIn(name, listed, name)
        self.assertEqual(self.catalog["references"], listed)

    def test_not_gym_and_no_new_cli(self):
        self.assertTrue(self.index["notGym"])
        self.assertTrue(self.index["noNewCli"])
        self.assertTrue(self.index["noNewExamPaperLogic"])
        self.assertEqual(self.index["issue"], ISSUE)
        self.assertEqual(self.catalog["issue"], ISSUE)
        self.assertTrue(self.catalog["notGym"])
        self.assertTrue(self.catalog["noNewCli"])
        self.assertEqual(self.catalog["existingCommand"], "build-from-ingest")

    def test_forbidden_peer_skills_exist_but_are_not_rewritten(self):
        for slug in SIBLING_SKILLS:
            self.assertIn(slug, self.index["forbiddenSkillsTouch"])
            peer = REPO / ".claude" / "skills" / slug / "SKILL.md"
            self.assertTrue(peer.is_file(), slug)

    def test_no_invented_commands_in_markdown(self):
        blobs = [self.skill, self.working]
        for name in REFERENCE_NAMES:
            blobs.append(read(REFS / name))
        for path in EXAMPLES.glob("*.md"):
            blobs.append(read(path))
        joined = "\n".join(blobs)
        for bad in INVENTED_COMMANDS:
            # 00_tree / 19_intent 는 금지 목록으로 이름을 언급한다.
            # '없다' 또는 '발명' 과 같은 줄에 있어야 한다.
            for m in re.finditer(re.escape(bad), joined):
                start = max(0, m.start() - 80)
                end = min(len(joined), m.end() + 80)
                ctx = joined[start:end]
                self.assertTrue(
                    any(w in ctx for w in ("없", "발명", "금지", "아님", "아니다")),
                    f"발명 명령이 긍정적으로 쓰임: {bad} … {ctx!r}",
                )

    def test_ref_tokens(self):
        for name, tokens in REF_TOKENS.items():
            body = read(REFS / name)
            for tok in tokens:
                self.assertIn(tok, body, f"{name} 에 없음: {tok}")

    def test_working_doc(self):
        for tok in WORKING_TOKENS:
            self.assertIn(tok, self.working, tok)
        self.assertIn("feat/agent-exam-ingest", self.working)

    def test_intent_matrix_size_and_schema(self):
        rows = self.intents["intents"]
        self.assertGreaterEqual(len(rows), 80)
        self.assertEqual(self.intents["count"], len(rows))
        self.assertEqual(self.intents["issue"], ISSUE)
        ids = set()
        for row in rows:
            self.assertRegex(row["id"], r"^I\d{3}$")
            self.assertTrue(row["utterance"])
            self.assertTrue(row["command"])
            self.assertTrue(row["reference"].endswith(".md"))
            self.assertRegex(row["stop"], r"^F\d{2}$")
            self.assertTrue(row["notGym"])
            self.assertNotIn(row["id"], ids)
            ids.add(row["id"])
            for bad in INVENTED_COMMANDS:
                if bad in row["command"]:
                    self.assertTrue(
                        any(w in row["command"] for w in ("없", "금지", "거절")),
                        row,
                    )

    def test_intent_matrix_documented(self):
        md = read(REFS / "19_intent_matrix.md")
        for row in self.intents["intents"][:30]:
            self.assertIn(row["id"], md)

    def test_stop_rules_match_skill(self):
        stops = load_json(MATS / "stop_rules.json")
        ids = [r["id"] for r in stops["rules"]]
        self.assertGreaterEqual(len(ids), 19)
        for rid in ids:
            self.assertIn(rid, self.skill, f"SKILL 정지 표에 {rid} 없음")

    def test_placement_matrix_and_schema_files(self):
        mat = load_json(MATS / "placement.json")
        self.assertEqual(mat["enum"], list(PLACEMENTS))
        self.assertEqual(mat["default"], "between")
        for plc in PLACEMENTS:
            path = SCHEMAS / f"valid_media_{plc}.json"
            doc = load_json(path)
            media = doc["questions"][0]["media"][0]
            self.assertEqual(media["placement"], plc)
            img = [
                b
                for b in doc["questions"][0]["stem_blocks"]
                if b.get("type") == "image"
            ][0]
            self.assertEqual(img["placement"], plc)

    def test_auto_number_policy_fixtures(self):
        tru = load_json(SCHEMAS / "valid_auto_number_true.json")
        fal = load_json(SCHEMAS / "valid_auto_number_false.json")
        self.assertTrue(tru["questions"][0]["auto_number"])
        self.assertFalse(fal["questions"][0]["auto_number"])
        self.assertTrue(
            fal["questions"][0]["stem"].startswith("2. "),
            "false 표본은 stem 에 번호를 이미 가진다",
        )
        self.assertFalse(tru["questions"][0]["stem"][:3].split(".")[0].isdigit() and tru["questions"][0]["stem"][1:3] == ". ")
        mat = load_json(MATS / "auto_number.json")
        self.assertTrue(mat["default"])
        dup = [r for r in mat["rows"] if r.get("avoid")]
        self.assertEqual(len(dup), 1)
        self.assertIn("3. 3.", dup[0]["printed"])

    def test_valid_schemas_look_like_v1(self):
        canon = load_json(CANON_SCHEMA)
        self.assertEqual(canon["properties"]["version"]["const"], "1")
        for path in SCHEMAS.glob("valid_*.json"):
            doc = load_json(path)
            self.assertEqual(doc["version"], "1", path.name)
            self.assertTrue(doc["questions"], path.name)
            for qq in doc["questions"]:
                self.assertGreaterEqual(qq["number"], 1, path.name)
                self.assertIn("stem", qq)
                self.assertTrue(qq["choices"], path.name)
                for ch in qq["choices"]:
                    self.assertIn("label", ch)
                    self.assertIn("text", ch)
                for key in ("answer", "latex", "equation", "table", "score"):
                    self.assertNotIn(key, qq, path.name)
            for key in ("answer_key", "latex", "debug"):
                self.assertNotIn(key, doc, path.name)

    def test_invalid_schemas_are_marked(self):
        expected = {
            "invalid_missing_version.json": lambda d: "version" not in d,
            "invalid_bad_version.json": lambda d: d.get("version") != "1",
            "invalid_missing_questions.json": lambda d: "questions" not in d,
            "invalid_unknown_field.json": lambda d: "answer_key" in d,
            "invalid_auto_number_type.json": lambda d: not isinstance(
                d["questions"][0].get("auto_number"), bool
            ),
            "invalid_boxed_text_field.json": lambda d: any(
                b.get("type") == "boxed" and "text" in b
                for b in d["questions"][0]["stem_blocks"]
            ),
            "invalid_unknown_block_type.json": lambda d: any(
                b.get("type") == "latex" for b in d["questions"][0]["stem_blocks"]
            ),
            "invalid_question_number_zero.json": lambda d: d["questions"][0]["number"]
            == 0,
        }
        for name, pred in expected.items():
            doc = load_json(SCHEMAS / name)
            self.assertTrue(pred(doc), name)

    def test_shared_passage_ids_line_up(self):
        doc = load_json(SCHEMAS / "valid_shared_passage.json")
        ids = {p["id"] for p in doc["passages"]}
        self.assertIn("p1-3", ids)
        refs = {q.get("passage_ref") for q in doc["questions"]}
        self.assertTrue(refs <= ids)
        self.assertGreaterEqual(len(doc["questions"]), 3)

    def test_boxed_valid_has_blocks_not_text(self):
        doc = load_json(SCHEMAS / "valid_boxed_bogi.json")
        boxed = [
            b
            for b in doc["questions"][0]["stem_blocks"]
            if b.get("type") == "boxed"
        ]
        self.assertEqual(len(boxed), 1)
        self.assertIn("blocks", boxed[0])
        self.assertNotIn("text", boxed[0])
        self.assertEqual(boxed[0]["title"], "<보기>")

    def test_dep_envelopes(self):
        mapping = {
            "check_deps_miss_poppler.json": "DEP_MISS_POPPLER",
            "check_deps_miss_imagemagick.json": "DEP_MISS_IMAGEMAGICK",
            "check_deps_miss_python_docx.json": "DEP_MISS_PYTHON_DOCX",
            "check_deps_miss_rhwp.json": "DEP_MISS_RHWP",
        }
        for name, code in mapping.items():
            env = load_json(ENVS / name)
            self.assertEqual(env["helper"], "check_deps.sh")
            codes = [e["code"] for e in env["envelopes"]]
            self.assertIn(code, codes, name)
            if code in ("DEP_MISS_POPPLER", "DEP_MISS_PYTHON_DOCX"):
                self.assertTrue(env["ok"], f"{code} 는 필수 실패가 아님")
            else:
                self.assertFalse(env["ok"], name)
        ok = load_json(ENVS / "check_deps_ok.json")
        self.assertTrue(ok["ok"])
        self.assertEqual(ok["missingRequired"], [])

    def test_helper_contract_files(self):
        crop = load_json(HFIX / "crop_bbox_contract.json")
        self.assertIn("--dry-run", crop["usage"])
        self.assertEqual(crop["min_wh"], 1)
        self.assertIn("CROP_BBOX_NOT_UINT", crop["codes"])
        pdf = load_json(HFIX / "pdf_to_pngs_contract.json")
        self.assertEqual(pdf["dpiDefault"], 300)
        self.assertEqual(pdf["dpiRange"], [72, 600])
        self.assertEqual(pdf["pagePattern"], "page_%03d.png")
        docx = load_json(HFIX / "extract_docx_contract.json")
        self.assertFalse(docx["fallbackIsFailure"])
        deps = load_json(HFIX / "check_deps_matrix.json")
        for code in DEP_CODES:
            self.assertIn(code, deps["codes"])

    def test_helper_scripts_advertise_dry_paths(self):
        pdf = read(HELPERS / "pdf_to_pngs.sh")
        crop = read(HELPERS / "crop_image.sh")
        docx = read(HELPERS / "extract_docx.py")
        deps = read(HELPERS / "check_deps.sh")
        for blob, needles in (
            (pdf, ("--dry-run", "--json", "PDF_MISS_TOOLS", "page_%03d.png", "72", "600")),
            (crop, ("--dry-run", "--json", "CROP_BBOX_NOT_UINT", "+repage", "CROP_MISS_IMAGEMAGICK")),
            (docx, ("--dry-run", "--json", "DOCX_SRC_MISSING", "zip-regex-fallback", "python-docx")),
            (deps, ("--json", "DEP_MISS_POPPLER", "DEP_MISS_IMAGEMAGICK", "DEP_MISS_PYTHON_DOCX")),
        ):
            for n in needles:
                self.assertIn(n, blob, n)

    def test_helper_python_dry_path_missing_file(self):
        script = HELPERS / "extract_docx.py"
        proc = subprocess.run(
            [sys.executable, str(script), "--json", "--dry-run", "no-such.docx", "out"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 1)
        data = json.loads(proc.stdout)
        self.assertFalse(data["ok"])
        self.assertEqual(data["code"], "DOCX_SRC_MISSING")
        self.assertEqual(data["helper"], "extract_docx.py")

    def test_helper_python_usage(self):
        script = HELPERS / "extract_docx.py"
        proc = subprocess.run(
            [sys.executable, str(script), "--json"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 1)
        data = json.loads(proc.stdout)
        self.assertEqual(data["code"], "DOCX_ARGS")

    def test_examples_exist_and_point_at_existing_cli(self):
        names = [p.name for p in EXAMPLES.glob("*.md")]
        self.assertIn("README.md", names)
        walks = [n for n in names if n != "README.md"]
        self.assertGreaterEqual(len(walks), 20)
        for name in walks:
            body = read(EXAMPLES / name)
            self.assertIn("build-from-ingest", body)
            self.assertIn("gym", body)
            self.assertGreater(len(body), 200, name)

    def test_transcripts_not_gym(self):
        files = list(TRANS.glob("*.json"))
        self.assertGreaterEqual(len(files), 6)
        for path in files:
            obj = load_json(path)
            self.assertTrue(obj["notGym"], path.name)
            self.assertEqual(obj["issue"], ISSUE)
            self.assertTrue(obj["steps"])

    def test_known_limits_matrix(self):
        mat = load_json(MATS / "known_limits.json")
        ids = {r["id"] for r in mat["limits"]}
        self.assertEqual(ids, {"L-picture", "L-equation", "L-table"})
        pic = [r for r in mat["limits"] if r["id"] == "L-picture"][0]
        self.assertEqual(pic["issue"], 182)
        self.assertIn("writer", pic["not"])

    def test_exit_code_matrix_mentions_helper_codes(self):
        mat = load_json(MATS / "exit_codes.json")
        self.assertIn("PDF_MISS_TOOLS", mat["pdf_to_pngs"]["2"])
        self.assertIn("CROP_BBOX", mat["crop_image"]["4"])
        self.assertEqual(mat["rhwp"]["2"], "사용법")

    def test_no_gym_tree_in_skill_dir(self):
        for path in SKILL.rglob("*"):
            rel = path.relative_to(SKILL).as_posix()
            self.assertFalse(rel.startswith("gym"), rel)
            self.assertNotIn("/gym/", f"/{rel}")

    def test_fixture_dir_is_only_under_skill(self):
        shadow = REPO / "tests" / "fixtures" / "agent_exam_ingest"
        self.assertFalse(shadow.exists(), "픽스처는 스킬 fixtures 한 곳만")

    def test_catalog_lists_generated_files(self):
        for name in self.catalog["schemas"]:
            self.assertTrue((SCHEMAS / name).is_file(), name)
        for name in self.catalog["envelopes"]:
            self.assertTrue((ENVS / name).is_file(), name)
        for name in self.catalog["transcripts"]:
            self.assertTrue((TRANS / name).is_file(), name)

    def test_mock_30_is_real_exam_shape(self):
        doc = load_json(SCHEMAS / "valid_mock_30.json")
        self.assertEqual(len(doc["questions"]), 30)
        nums = [q["number"] for q in doc["questions"]]
        self.assertEqual(nums, list(range(1, 31)))
        self.assertEqual(doc["header_text"], "전국연합학력평가")
        self.assertEqual(doc["form_label"], "홀수형")

    def test_md_sample_has_image_ref(self):
        text = read(FIXTURES / "md" / "sample_exam.md")
        self.assertRegex(text, r"!\[[^\]]*\]\([^)]+\)")
        self.assertIn("## 1.", text)
        self.assertIn("①", text)

    def test_loops_cover_three_inputs(self):
        pdf = load_json(FIXTURES / "loops" / "pdf_success.json")
        crop = load_json(FIXTURES / "loops" / "media_crop.json")
        docx = load_json(FIXTURES / "loops" / "docx_fallback.json")
        self.assertIn("pdf_to_pngs", pdf["steps"])
        self.assertTrue(docx["pythonDocxMissingIsOk"])
        self.assertIn("CROP_BBOX_NOT_UINT", crop["stopOn"])

    def test_canonical_schema_placements(self):
        canon = load_json(CANON_SCHEMA)
        enum = canon["definitions"]["Placement"]["enum"]
        self.assertEqual(enum, list(PLACEMENTS))
        self.assertIn("auto_number", canon["definitions"]["Question"]["properties"])
        self.assertIn("passages", canon["properties"])
        boxed = [
            v
            for v in canon["definitions"]["StemBlock"]["oneOf"]
            if v["properties"]["type"]["const"] == "boxed"
        ]
        self.assertEqual(len(boxed), 1)

    def test_build_envelope_does_not_invent_verify_flag(self):
        missing = load_json(ENVS / "build_missing_o.json")
        self.assertEqual(missing["exit"], 2)
        self.assertEqual(missing["command"], "build-from-ingest")
        self.assertNotIn("--verify", json.dumps(missing))

    def test_gen_pack_issue_constant(self):
        gen = read(REFS / "_gen_pack.py")
        self.assertIn("ISSUE = 5319", gen)
        self.assertIn("noNewExamPaperLogic", gen)
        self.assertIn("build-from-ingest", gen)


if __name__ == "__main__":
    unittest.main()
