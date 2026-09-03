"""[#5281] gym profiles 프로파일 계약.

gym/profiles/*.json 전 파일이 스키마·팩 참조·파일명=id 를 지키고,
family / starter / editor / publisher / operator / boss / maintainer
일곱 자리가 문서와 같은 묶음을 선언하는지 고정한다.

프로파일은 pack 을 고르는 도구이지 점수를 뭉치는 도구가 아니다
(gym/core/runner.py score_all). 이 시험은 그 선택 계약을 파일만으로
고정한다. 바이너리·네트워크를 부르지 않는다.

이 PR 은 schema.py / audit.py / certify.py / report.py / tutorial / PARK
를 고치지 않는다. maintainer.json 도 여기서 고치지 않는다 — 다른
열린 PR 이 pack 을 더하면 그쪽에서 정렬·추가한다. 이 시험은
현재 브랜치의 packs/ 와 profiles/ 가 서로 맞는지, 일곱 자리의
선언이 문서와 같은지만 본다.

정본 문서: gym/docs/profiles.md
작업 기록: mydocs/working/gym_profiles.md
"""

from __future__ import annotations

import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


REPO_ROOT = Path(__file__).resolve().parents[2]
GYM = REPO_ROOT / "gym"
PACKS = GYM / "packs"
PROFILES = GYM / "profiles"
DOCS_PROFILES = GYM / "docs" / "profiles.md"
WORKING_PROFILES = REPO_ROOT / "mydocs" / "working" / "gym_profiles.md"
SCORE_PY = GYM / "score.py"
AUDIT_PY = GYM / "tools" / "audit.py"

SCHEMA_VERSION = "1.0"
PROFILE_KIND = "gymProfile"

# 일곱 자리 — 파일명·id·문서 표가 같은 집합이어야 한다.
NAMED_PROFILE_IDS = (
    "family",
    "starter",
    "editor",
    "publisher",
    "operator",
    "boss",
    "maintainer",
)

# 각 자리의 pack 묶음. family~boss/operator 는 고정 계약이다.
# maintainer 의 전체 목록은 현재 packs/ 와 맞춰 검사하되, 다른 PR 이
# pack 을 더하면 그쪽 maintainer.json 이 따라간다. 여기서는
# "일곱 자리의 핵심 묶음" 과 "다른 자리가 고른 pack 은 maintainer 에
# 들어 있다" 를 고정한다.
NAMED_PACKS = {
    "family": ("casual-rides",),
    "starter": ("core-cli", "self-description"),
    "editor": ("core-cli", "text-editing", "table-editing", "objects-media"),
    "publisher": ("serialization", "layout-rendering", "security"),
    "operator": ("corpus-diagnostics", "automation"),
    "boss": ("expert-challenges",),
}

# 사람이 읽는 제목·설명의 핵심 어휘. 문장 전체가 바뀌어도 역할이
# 남는지 확인한다. 제목을 바꾸면 문서 표와 같이 고친다.
NAMED_TITLE_TOKENS = {
    "family": ("가족",),
    "starter": ("입문",),
    "editor": ("편집",),
    "publisher": ("배포",),
    "operator": ("운영",),
    "boss": ("보스",),
    "maintainer": ("메인테이너",),
}

NAMED_DESCRIPTION_TOKENS = {
    "family": ("입문", "부모님"),
    "starter": ("처음",),
    "editor": ("편집",),
    "publisher": ("배포",),
    "operator": ("진단",),
    "boss": ("고난도",),
    "maintainer": ("전",),
}

# 허용하는 최상위 키. 새 키를 넣으려면 문서와 이 집합을 같이 고친다.
ALLOWED_TOP_LEVEL_KEYS = {
    "schemaVersion",
    "kind",
    "id",
    "title",
    "description",
    "packs",
}

# 문서가 반드시 품어야 하는 표제어. 절을 옮겨도 단어는 남긴다.
DOC_REQUIRED_TOKENS = (
    "family",
    "starter",
    "editor",
    "publisher",
    "operator",
    "boss",
    "maintainer",
    "gymProfile",
    "schemaVersion",
    "--profile",
    "casual-rides",
    "core-cli",
    "self-description",
    "text-editing",
    "table-editing",
    "objects-media",
    "serialization",
    "layout-rendering",
    "security",
    "corpus-diagnostics",
    "automation",
    "expert-challenges",
    "점수를 뭉치는 도구가 아니다",
    "maintainer",
)

WORKING_REQUIRED_TOKENS = (
    "#5281",
    "feat/gym-profiles-hardening",
    "family",
    "starter",
    "editor",
    "publisher",
    "operator",
    "boss",
    "maintainer",
    "test_gym_profiles.py",
    "gym/docs/profiles.md",
    "schema.py",
    "audit.py",
    "maintainer.json",
)


def _ensure_repo_on_path():
    root = str(REPO_ROOT)
    if root not in sys.path:
        sys.path.insert(0, root)


def load_schema():
    _ensure_repo_on_path()
    from gym.core import schema

    return schema


def load_runner():
    _ensure_repo_on_path()
    from gym.core import runner

    return runner


def read_text(path):
    data = Path(path).read_bytes()
    return data, data.decode("utf-8")


def read_json(path):
    raw, text = read_text(path)
    return raw, text, json.loads(text)


def profile_paths():
    return sorted(PROFILES.glob("*.json"))


def pack_ids():
    return sorted(p.name for p in PACKS.iterdir() if (p / "pack.json").is_file())


def load_all_profiles():
    out = {}
    for path in profile_paths():
        _raw, _text, doc = read_json(path)
        out[path.stem] = {"path": path, "doc": doc}
    return out


def write_json(path, doc):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with io.open(path, "w", encoding="utf-8", newline="\n") as fh:
        json.dump(doc, fh, ensure_ascii=False, indent=2)
        fh.write("\n")


def minimal_profile(pid="probe", packs=None, **extra):
    body = {
        "schemaVersion": SCHEMA_VERSION,
        "kind": PROFILE_KIND,
        "id": pid,
        "title": pid,
        "description": "픽스처",
        "packs": list(packs) if packs is not None else ["core-cli"],
    }
    body.update(extra)
    return body


class ProfileInventoryTests(unittest.TestCase):
    """profiles/ 아래 파일이 일곱 자리와 1:1 이다."""

    def test_profiles_directory_exists(self):
        self.assertTrue(PROFILES.is_dir(), f"없음: {PROFILES}")

    def test_every_profile_file_is_json(self):
        files = list(PROFILES.iterdir())
        self.assertTrue(files, "profiles/ 가 비었다")
        leftovers = [p.name for p in files if p.is_file() and p.suffix != ".json"]
        self.assertEqual(leftovers, [], f"json 아닌 파일: {leftovers}")

    def test_named_seven_files_exist(self):
        names = {p.stem for p in profile_paths()}
        missing = [pid for pid in NAMED_PROFILE_IDS if pid not in names]
        self.assertEqual(missing, [], f"빠진 프로파일: {missing}")

    def test_no_unexpected_profile_files(self):
        names = {p.stem for p in profile_paths()}
        extra = sorted(names - set(NAMED_PROFILE_IDS))
        self.assertEqual(
            extra,
            [],
            "새 프로파일 파일을 넣었으면 NAMED_PROFILE_IDS 와 "
            "gym/docs/profiles.md 를 같이 고쳐라: " + ", ".join(extra),
        )

    def test_profile_count_is_seven(self):
        self.assertEqual(len(profile_paths()), len(NAMED_PROFILE_IDS))

    def test_profile_filenames_are_sorted_unique(self):
        stems = [p.stem for p in profile_paths()]
        self.assertEqual(stems, sorted(set(stems)))


class ProfileJsonShapeTests(unittest.TestCase):
    """모든 프로파일 파일의 뼈대 — kind·버전·id·packs."""

    def test_every_file_is_utf8_object(self):
        for path in profile_paths():
            raw, text, doc = read_json(path)
            self.assertFalse(raw.startswith(b"\xef\xbb\xbf"), f"BOM: {path.name}")
            self.assertIsInstance(doc, dict, path.name)
            self.assertTrue(text.endswith("\n"), f"끝 개행 없음: {path.name}")
            self.assertNotIn("\r\n", text, f"CRLF: {path.name}")

    def test_kind_and_schema_version(self):
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            self.assertEqual(doc.get("kind"), PROFILE_KIND, path.name)
            self.assertEqual(doc.get("schemaVersion"), SCHEMA_VERSION, path.name)

    def test_id_matches_filename(self):
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            self.assertEqual(doc.get("id"), path.stem, path.name)

    def test_title_and_description_are_nonempty_strings(self):
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            title = doc.get("title")
            desc = doc.get("description")
            self.assertIsInstance(title, str, path.name)
            self.assertIsInstance(desc, str, path.name)
            self.assertTrue(title.strip(), f"빈 title: {path.name}")
            self.assertTrue(desc.strip(), f"빈 description: {path.name}")

    def test_packs_is_nonempty_list_of_strings(self):
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            packs = doc.get("packs")
            self.assertIsInstance(packs, list, path.name)
            self.assertTrue(packs, f"packs 가 비었다: {path.name}")
            for pid in packs:
                self.assertIsInstance(pid, str, f"{path.name}: {pid!r}")
                self.assertTrue(pid.strip(), f"{path.name}: 빈 pack id")
                self.assertEqual(pid, pid.strip(), f"{path.name}: 공백 포함 {pid!r}")
                self.assertNotIn("/", pid, path.name)
                self.assertNotIn("\\", pid, path.name)
                self.assertNotIn(" ", pid, path.name)

    def test_no_duplicate_packs_inside_a_profile(self):
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            packs = doc["packs"]
            self.assertEqual(
                packs,
                list(dict.fromkeys(packs)),
                f"중복 pack: {path.name}",
            )

    def test_top_level_keys_are_known(self):
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            unknown = sorted(set(doc) - ALLOWED_TOP_LEVEL_KEYS)
            self.assertEqual(
                unknown,
                [],
                f"{path.name} 에 문서화되지 않은 키: {unknown}",
            )

    def test_json_indent_is_two_spaces(self):
        for path in profile_paths():
            _raw, text, doc = read_json(path)
            expected = json.dumps(doc, ensure_ascii=False, indent=2) + "\n"
            self.assertEqual(text, expected, f"indent/키 순서 불일치: {path.name}")


class ProfilePackRefTests(unittest.TestCase):
    """선언한 pack 은 packs/<id>/pack.json 이 있어야 한다."""

    def test_every_pack_ref_exists(self):
        known = set(pack_ids())
        missing = []
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            for pid in doc.get("packs") or []:
                if pid not in known:
                    missing.append(f"{path.stem}:{pid}")
        self.assertEqual(missing, [], f"없는 pack 참조: {missing}")

    def test_schema_validate_profile_accepts_every_file(self):
        schema = load_schema()
        known = set(pack_ids())
        errors = []
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            schema.validate_profile(doc, known, errors)
        self.assertEqual(errors, [], "\n".join(errors))

    def test_pack_json_id_matches_folder(self):
        for pid in pack_ids():
            manifest = json.loads((PACKS / pid / "pack.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest.get("id"), pid)

    def test_every_named_pack_is_still_a_real_pack(self):
        known = set(pack_ids())
        missing = []
        for profile_id, packs in NAMED_PACKS.items():
            for pid in packs:
                if pid not in known:
                    missing.append(f"{profile_id}:{pid}")
        self.assertEqual(missing, [], f"카탈로그 pack 이 사라졌다: {missing}")


class NamedProfileContractTests(unittest.TestCase):
    """일곱 자리의 고정 묶음."""

    def test_family_packs(self):
        _raw, _text, doc = read_json(PROFILES / "family.json")
        self.assertEqual(tuple(doc["packs"]), NAMED_PACKS["family"])

    def test_starter_packs(self):
        _raw, _text, doc = read_json(PROFILES / "starter.json")
        self.assertEqual(tuple(doc["packs"]), NAMED_PACKS["starter"])

    def test_editor_packs(self):
        _raw, _text, doc = read_json(PROFILES / "editor.json")
        self.assertEqual(tuple(doc["packs"]), NAMED_PACKS["editor"])

    def test_publisher_packs(self):
        _raw, _text, doc = read_json(PROFILES / "publisher.json")
        self.assertEqual(tuple(doc["packs"]), NAMED_PACKS["publisher"])

    def test_operator_packs(self):
        _raw, _text, doc = read_json(PROFILES / "operator.json")
        self.assertEqual(tuple(doc["packs"]), NAMED_PACKS["operator"])

    def test_boss_packs(self):
        _raw, _text, doc = read_json(PROFILES / "boss.json")
        self.assertEqual(tuple(doc["packs"]), NAMED_PACKS["boss"])

    def test_named_pack_catalog_matches_files(self):
        for pid, expected in NAMED_PACKS.items():
            _raw, _text, doc = read_json(PROFILES / f"{pid}.json")
            self.assertEqual(tuple(doc["packs"]), expected, pid)

    def test_named_titles_keep_role_tokens(self):
        for pid, tokens in NAMED_TITLE_TOKENS.items():
            _raw, _text, doc = read_json(PROFILES / f"{pid}.json")
            title = doc["title"]
            for token in tokens:
                self.assertIn(token, title, f"{pid} title 에 {token!r} 없음")

    def test_named_descriptions_keep_role_tokens(self):
        for pid, tokens in NAMED_DESCRIPTION_TOKENS.items():
            _raw, _text, doc = read_json(PROFILES / f"{pid}.json")
            desc = doc["description"]
            for token in tokens:
                self.assertIn(token, desc, f"{pid} description 에 {token!r} 없음")

    def test_family_is_only_casual_rides(self):
        _raw, _text, doc = read_json(PROFILES / "family.json")
        self.assertEqual(doc["packs"], ["casual-rides"])
        self.assertEqual(len(doc["packs"]), 1)

    def test_boss_is_only_expert_challenges(self):
        _raw, _text, doc = read_json(PROFILES / "boss.json")
        self.assertEqual(doc["packs"], ["expert-challenges"])
        self.assertEqual(len(doc["packs"]), 1)

    def test_starter_is_the_smallest_tool_course(self):
        _raw, _text, doc = read_json(PROFILES / "starter.json")
        self.assertEqual(set(doc["packs"]), {"core-cli", "self-description"})

    def test_editor_covers_document_mutation_axes(self):
        _raw, _text, doc = read_json(PROFILES / "editor.json")
        self.assertIn("text-editing", doc["packs"])
        self.assertIn("table-editing", doc["packs"])
        self.assertIn("objects-media", doc["packs"])
        self.assertIn("core-cli", doc["packs"])

    def test_publisher_covers_export_and_safety(self):
        _raw, _text, doc = read_json(PROFILES / "publisher.json")
        self.assertIn("serialization", doc["packs"])
        self.assertIn("layout-rendering", doc["packs"])
        self.assertIn("security", doc["packs"])

    def test_operator_covers_sweep_and_automation(self):
        _raw, _text, doc = read_json(PROFILES / "operator.json")
        self.assertIn("corpus-diagnostics", doc["packs"])
        self.assertIn("automation", doc["packs"])


class MaintainerContractTests(unittest.TestCase):
    """maintainer 는 전 표면. 이 PR 은 그 파일을 고치지 않는다."""

    def test_maintainer_file_exists(self):
        self.assertTrue((PROFILES / "maintainer.json").is_file())

    def test_maintainer_packs_are_sorted(self):
        _raw, _text, doc = read_json(PROFILES / "maintainer.json")
        self.assertEqual(doc["packs"], sorted(doc["packs"]))

    def test_maintainer_packs_all_exist(self):
        known = set(pack_ids())
        _raw, _text, doc = read_json(PROFILES / "maintainer.json")
        missing = [pid for pid in doc["packs"] if pid not in known]
        self.assertEqual(missing, [], f"maintainer 가 없는 pack 을 가리킨다: {missing}")

    def test_maintainer_covers_every_other_profile_pack(self):
        _raw, _text, maintainer = read_json(PROFILES / "maintainer.json")
        covered = set(maintainer["packs"])
        holes = []
        for path in profile_paths():
            if path.stem == "maintainer":
                continue
            _raw, _text, doc = read_json(path)
            for pid in doc["packs"]:
                if pid not in covered:
                    holes.append(f"{path.stem}:{pid}")
        self.assertEqual(
            holes,
            [],
            "다른 자리가 고른 pack 이 maintainer 에 없다: " + ", ".join(holes),
        )

    def test_maintainer_covers_named_catalog(self):
        _raw, _text, maintainer = read_json(PROFILES / "maintainer.json")
        covered = set(maintainer["packs"])
        for profile_id, packs in NAMED_PACKS.items():
            missing = [pid for pid in packs if pid not in covered]
            self.assertEqual(missing, [], f"{profile_id} → maintainer 구멍: {missing}")

    def test_maintainer_includes_every_pack_on_this_branch(self):
        """현재 브랜치 packs/ 전부가 maintainer 에 들어 있는가.

        다른 열린 PR 이 pack 을 더하면 그쪽에서 maintainer.json 을
        정렬·추가한다. 이 시험은 이 브랜치의 스냅샷만 본다. 새 pack 을
        이 PR 에 넣지 않았으므로 파일을 고울 이유가 없다.
        """
        _raw, _text, maintainer = read_json(PROFILES / "maintainer.json")
        self.assertEqual(sorted(maintainer["packs"]), pack_ids())

    def test_maintainer_is_strict_superset_of_role_profiles(self):
        _raw, _text, maintainer = read_json(PROFILES / "maintainer.json")
        role_union = set()
        for packs in NAMED_PACKS.values():
            role_union.update(packs)
        extra = set(maintainer["packs"]) - role_union
        self.assertTrue(
            extra,
            "maintainer 가 역할 여섯 자리의 합과 같으면 전 표면이 아니다",
        )

    def test_maintainer_does_not_list_unknown_future_ids(self):
        known = set(pack_ids())
        _raw, _text, maintainer = read_json(PROFILES / "maintainer.json")
        self.assertTrue(set(maintainer["packs"]) <= known)


class ProfileOverlapTests(unittest.TestCase):
    """자리가 서로 어떤 관계를 갖는가 — 놀이공원 지도의 겹침."""

    def _packs(self, pid):
        _raw, _text, doc = read_json(PROFILES / f"{pid}.json")
        return list(doc["packs"])

    def test_family_and_boss_are_disjoint(self):
        family = set(self._packs("family"))
        boss = set(self._packs("boss"))
        self.assertFalse(family & boss, "입문존과 보스존이 겹치면 안 된다")

    def test_family_is_not_subset_of_starter(self):
        family = set(self._packs("family"))
        starter = set(self._packs("starter"))
        self.assertFalse(
            family <= starter,
            "가족 코스는 입문 도구 코스와 다른 축이다",
        )

    def test_starter_core_is_in_editor(self):
        starter = set(self._packs("starter"))
        editor = set(self._packs("editor"))
        self.assertIn("core-cli", starter & editor)

    def test_editor_and_publisher_share_no_required_pack(self):
        editor = set(self._packs("editor"))
        publisher = set(self._packs("publisher"))
        self.assertFalse(
            editor & publisher,
            "편집자와 배포자는 서로 다른 능력 축을 고른다",
        )

    def test_operator_and_publisher_are_disjoint(self):
        operator = set(self._packs("operator"))
        publisher = set(self._packs("publisher"))
        self.assertFalse(operator & publisher)

    def test_operator_and_family_are_disjoint(self):
        self.assertFalse(set(self._packs("operator")) & set(self._packs("family")))

    def test_boss_not_in_role_courses_except_maintainer(self):
        boss = set(self._packs("boss"))
        for pid in ("family", "starter", "editor", "publisher", "operator"):
            overlap = boss & set(self._packs(pid))
            self.assertFalse(overlap, f"{pid} 가 보스 pack 을 품었다: {overlap}")

    def test_role_union_is_smaller_than_maintainer(self):
        role = set()
        for pid in ("family", "starter", "editor", "publisher", "operator", "boss"):
            role.update(self._packs(pid))
        maintainer = set(self._packs("maintainer"))
        self.assertTrue(role < maintainer)

    def test_no_two_role_profiles_are_identical(self):
        seen = {}
        for pid in NAMED_PROFILE_IDS:
            key = tuple(self._packs(pid))
            self.assertNotIn(key, seen, f"{pid} 와 {seen.get(key)} 가 같은 묶음")
            seen[key] = pid


class LoadProfileTests(unittest.TestCase):
    """runner.load_profile 은 파일을 그대로 돌려준다. 엔진은 고치지 않는다."""

    def test_load_profile_returns_each_named_file(self):
        runner = load_runner()
        for pid in NAMED_PROFILE_IDS:
            loaded = runner.load_profile(pid)
            _raw, _text, disk = read_json(PROFILES / f"{pid}.json")
            self.assertEqual(loaded, disk, pid)
            self.assertEqual(loaded["id"], pid)

    def test_load_profile_missing_raises(self):
        runner = load_runner()
        with self.assertRaises(runner.ScoreRunnerError) as caught:
            runner.load_profile("does-not-exist-5281")
        self.assertEqual(caught.exception.kind, "missing-profile")

    def test_score_all_uses_profile_pack_list(self):
        """profile_id가 있으면 실제 채점 대상이 프로파일 목록이 된다."""
        runner = load_runner()

        def fake_pack(pack_id, *_args):
            return {"id": pack_id, "status": "unavailable", "score": None, "max": 0}

        with (
            mock.patch.object(
                runner,
                "load_profile",
                return_value={"id": "selected", "packs": ["pack-b", "pack-a"]},
            ) as load_profile,
            mock.patch.object(runner, "score_pack", side_effect=fake_pack),
            mock.patch.object(runner, "safe_runner_identity", return_value={}),
            mock.patch.object(runner, "safe_known_commands", return_value=None),
            mock.patch.object(runner, "bin_is_missing", return_value=False),
        ):
            card = runner.score_all(
                "unused-submissions",
                "unused-rhwp",
                pack_ids=["ignored-pack"],
                profile_id="selected",
            )

        load_profile.assert_called_once_with("selected")
        self.assertEqual([pack["id"] for pack in card["packs"]], ["pack-b", "pack-a"])

    def test_score_py_exposes_profile_flag(self):
        text = SCORE_PY.read_text(encoding="utf-8")
        self.assertIn("--profile", text)
        self.assertIn("profile_id=a.profile", text)


class ValidateProfileNegativeTests(unittest.TestCase):
    """schema.validate_profile 의 거절 칸 — 엔진을 고치지 않고 현재 계약만 고정."""

    def setUp(self):
        self.schema = load_schema()
        self.known = {"core-cli", "security", "casual-rides"}

    def _errors(self, profile):
        errors = []
        self.schema.validate_profile(profile, self.known, errors)
        return errors

    def test_wrong_kind_is_rejected(self):
        doc = minimal_profile(kind="gymPack")
        self.assertTrue(any("kind" in e for e in self._errors(doc)))

    def test_empty_packs_is_rejected(self):
        doc = minimal_profile(packs=[])
        self.assertTrue(any("packs" in e for e in self._errors(doc)))

    def test_missing_packs_is_rejected(self):
        doc = minimal_profile()
        del doc["packs"]
        self.assertTrue(any("packs" in e for e in self._errors(doc)))

    def test_unknown_pack_ref_is_rejected(self):
        doc = minimal_profile(packs=["no-such-pack"])
        errors = self._errors(doc)
        self.assertTrue(any("없는 pack" in e for e in errors), errors)

    def test_known_pack_is_accepted(self):
        doc = minimal_profile(packs=["core-cli"])
        self.assertEqual(self._errors(doc), [])

    def test_mixed_known_and_unknown(self):
        doc = minimal_profile(packs=["core-cli", "ghost-pack"])
        errors = self._errors(doc)
        self.assertTrue(any("ghost-pack" in e for e in errors), errors)

    def test_none_profile_id_still_reports_where(self):
        errors = self._errors({"kind": PROFILE_KIND, "packs": []})
        self.assertTrue(any("profiles/" in e for e in errors), errors)


class ProfileHygieneTests(unittest.TestCase):
    """파일 위생 — BOM 없음, LF, 키 순서, 정렬."""

    def test_named_role_packs_keep_declared_order(self):
        """역할 여섯 자리는 사람이 읽는 순서를 유지한다.

        family/boss 는 하나라 정렬과 같고, starter/editor/publisher/operator
        는 문서에 적은 순서를 따른다. 알파벳 강제하지 않는다.
        """
        for pid, expected in NAMED_PACKS.items():
            _raw, _text, doc = read_json(PROFILES / f"{pid}.json")
            self.assertEqual(tuple(doc["packs"]), expected, pid)

    def test_maintainer_only_is_alphabetically_sorted(self):
        _raw, _text, doc = read_json(PROFILES / "maintainer.json")
        self.assertEqual(doc["packs"], sorted(doc["packs"]))

    def test_no_trailing_whitespace_on_lines(self):
        for path in profile_paths():
            _raw, text = read_text(path)
            for idx, line in enumerate(text.split("\n"), 1):
                if line.endswith("\r"):
                    self.fail(f"{path.name}:{idx} CR")
                self.assertEqual(line, line.rstrip(" \t"), f"{path.name}:{idx} 끝 공백")

    def test_ids_are_safe_single_path_component(self):
        for path in profile_paths():
            self.assertEqual(path.stem, path.name[: -len(path.suffix)])
            self.assertNotIn("..", path.stem)
            self.assertRegex(path.stem, r"^[a-z][a-z0-9-]*$")


class DocsContractTests(unittest.TestCase):
    """정본·작업 기록이 일곱 자리와 같은 표를 말한다."""

    def test_canonical_doc_exists(self):
        self.assertTrue(DOCS_PROFILES.is_file(), f"없음: {DOCS_PROFILES}")

    def test_working_doc_exists(self):
        self.assertTrue(WORKING_PROFILES.is_file(), f"없음: {WORKING_PROFILES}")

    def test_canonical_doc_frontmatter(self):
        _raw, text = read_text(DOCS_PROFILES)
        self.assertTrue(text.startswith("---\n"), "frontmatter 없음")
        self.assertIn("kind: guide", text)
        self.assertIn("canonical: gym/docs/profiles.md", text)
        self.assertIn("status: active", text)

    def test_working_doc_frontmatter(self):
        _raw, text = read_text(WORKING_PROFILES)
        self.assertTrue(text.startswith("---\n"))
        self.assertIn("kind: working", text)
        self.assertIn("canonical: mydocs/working/gym_profiles.md", text)

    def test_canonical_doc_has_required_tokens(self):
        _raw, text = read_text(DOCS_PROFILES)
        missing = [tok for tok in DOC_REQUIRED_TOKENS if tok not in text]
        self.assertEqual(missing, [], f"정본 문서 누락: {missing}")

    def test_working_doc_has_required_tokens(self):
        _raw, text = read_text(WORKING_PROFILES)
        missing = [tok for tok in WORKING_REQUIRED_TOKENS if tok not in text]
        self.assertEqual(missing, [], f"작업 기록 누락: {missing}")

    def test_docs_are_utf8_lf_no_bom(self):
        for path in (DOCS_PROFILES, WORKING_PROFILES):
            raw, text = read_text(path)
            self.assertFalse(raw.startswith(b"\xef\xbb\xbf"), path.name)
            self.assertNotIn("\r\n", text, path.name)
            self.assertTrue(text.endswith("\n"), path.name)

    def test_canonical_doc_lists_each_named_pack_set(self):
        _raw, text = read_text(DOCS_PROFILES)
        for pid, packs in NAMED_PACKS.items():
            self.assertIn(f"`{pid}`", text, pid)
            for pack in packs:
                self.assertIn(f"`{pack}`", text, f"{pid}/{pack}")

    def test_canonical_doc_points_at_working_doc(self):
        _raw, text = read_text(DOCS_PROFILES)
        self.assertIn("mydocs/working/gym_profiles.md", text)

    def test_working_doc_points_at_canonical_doc(self):
        _raw, text = read_text(WORKING_PROFILES)
        self.assertIn("gym/docs/profiles.md", text)

    def test_docs_do_not_claim_a_new_cli(self):
        for path in (DOCS_PROFILES, WORKING_PROFILES):
            _raw, text = read_text(path)
            self.assertNotIn("python gym/profiles.py", text, path.name)
            self.assertNotIn("--new-profile-flag", text, path.name)

    def test_docs_mention_score_profile_flag(self):
        _raw, text = read_text(DOCS_PROFILES)
        self.assertIn("python gym/score.py --agent", text)
        self.assertIn("--profile", text)

    def test_docs_state_no_schema_edit(self):
        _raw, text = read_text(WORKING_PROFILES)
        self.assertIn("schema.py", text)
        self.assertIn("고치지 않", text)


class ScoreAndAuditSurfaceTests(unittest.TestCase):
    """기존 진입점·감사기는 그대로 두고, 프로파일 표면만 확인한다."""

    def test_score_help_text_mentions_profile(self):
        text = SCORE_PY.read_text(encoding="utf-8")
        self.assertIn('help="pack 묶음 프로파일 id"', text)

    def test_audit_tool_still_imports(self):
        spec_path = AUDIT_PY
        self.assertTrue(spec_path.is_file())
        import importlib.util

        spec = importlib.util.spec_from_file_location("gym_audit_5281", spec_path)
        self.assertIsNotNone(spec)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        self.assertTrue(callable(module.audit))

    def test_audit_real_repo_still_ok(self):
        import importlib.util

        spec = importlib.util.spec_from_file_location("gym_audit_5281b", AUDIT_PY)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        report = module.audit(str(GYM))
        self.assertTrue(report["ok"], report)

    def test_schema_validate_profile_symbol_unchanged(self):
        schema = load_schema()
        self.assertTrue(callable(schema.validate_profile))
        self.assertEqual(schema.PROFILE_KIND, PROFILE_KIND)
        self.assertEqual(schema.SCHEMA_VERSION, SCHEMA_VERSION)


class ProfileCatalogTableTests(unittest.TestCase):
    """문서의 역할 표와 코드 카탈로그가 같은 말을 한다."""

    def test_catalog_keys_are_the_six_role_profiles(self):
        self.assertEqual(
            set(NAMED_PACKS),
            {"family", "starter", "editor", "publisher", "operator", "boss"},
        )

    def test_catalog_values_are_tuples(self):
        for pid, packs in NAMED_PACKS.items():
            self.assertIsInstance(packs, tuple, pid)
            self.assertTrue(packs, pid)

    def test_docs_repeat_catalog_rows(self):
        _raw, text = read_text(DOCS_PROFILES)
        rows = {
            "family": "casual-rides",
            "starter": "core-cli",
            "editor": "text-editing",
            "publisher": "serialization",
            "operator": "corpus-diagnostics",
            "boss": "expert-challenges",
        }
        for pid, pack in rows.items():
            self.assertIn(pid, text)
            self.assertIn(pack, text)


class ProfileTempFixtureTests(unittest.TestCase):
    """디스크의 실제 프로파일은 건드리지 않고, 임시 파일로 경계만 본다."""

    def test_roundtrip_minimal_profile(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "probe.json")
            doc = minimal_profile()
            write_json(path, doc)
            loaded = json.loads(Path(path).read_text(encoding="utf-8"))
            self.assertEqual(loaded, doc)

    def test_validate_accepts_written_fixture(self):
        schema = load_schema()
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "probe.json")
            doc = minimal_profile(packs=["core-cli"])
            write_json(path, doc)
            errors = []
            schema.validate_profile(
                json.loads(Path(path).read_text(encoding="utf-8")),
                {"core-cli"},
                errors,
            )
            self.assertEqual(errors, [])


class ProfileIdUniquenessTests(unittest.TestCase):
    def test_ids_unique_across_files(self):
        ids = []
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            ids.append(doc["id"])
        self.assertEqual(len(ids), len(set(ids)))

    def test_ids_equal_named_list(self):
        ids = []
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            ids.append(doc["id"])
        self.assertEqual(sorted(ids), sorted(NAMED_PROFILE_IDS))


class ProfileDescriptionQualityTests(unittest.TestCase):
    """설명이 한 글자가 아니고, 자리의 역할을 가리킨다."""

    def test_descriptions_are_longer_than_a_token(self):
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            self.assertGreaterEqual(len(doc["description"]), 8, path.name)

    def test_titles_are_short(self):
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            self.assertLessEqual(len(doc["title"]), 40, path.name)

    def test_family_mentions_together(self):
        _raw, _text, doc = read_json(PROFILES / "family.json")
        self.assertTrue("함께" in doc["description"] or "부모님" in doc["description"])

    def test_maintainer_mentions_full_surface(self):
        _raw, _text, doc = read_json(PROFILES / "maintainer.json")
        self.assertTrue(
            "전" in doc["description"] or "완주" in doc["description"],
            doc["description"],
        )


class ProfileDoesNotSelectScoreAggregationTests(unittest.TestCase):
    """프로파일은 선택기다. 점수 합산 규칙이 프로파일 JSON 에 없다."""

    def test_no_weights_key(self):
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            self.assertNotIn("weights", doc, path.name)
            self.assertNotIn("score", doc, path.name)
            self.assertNotIn("max", doc, path.name)

    def test_no_task_allowlist(self):
        for path in profile_paths():
            _raw, _text, doc = read_json(path)
            self.assertNotIn("tasks", doc, path.name)
            self.assertNotIn("exclude", doc, path.name)


# pack → 역할 자리. maintainer 는 전 pack 이라 여기 적지 않는다.
# 표에 없는 pack 은 전 표면에서만 고른다.
PACK_ROLE_OWNERS = {
    "casual-rides": ("family",),
    "core-cli": ("starter", "editor"),
    "self-description": ("starter",),
    "text-editing": ("editor",),
    "table-editing": ("editor",),
    "objects-media": ("editor",),
    "serialization": ("publisher",),
    "layout-rendering": ("publisher",),
    "security": ("publisher",),
    "corpus-diagnostics": ("operator",),
    "automation": ("operator",),
    "expert-challenges": ("boss",),
}

ROLE_ONLY_PACKS = tuple(PACK_ROLE_OWNERS)
MAINTAINER_ONLY_HINTS = (
    "extraction",
    "batch-ops",
    "render-tree",
    "studio-e2e",
    "table-csv",
)


def _owners_of(pack_id):
    owners = []
    for stem in NAMED_PROFILE_IDS:
        if stem == "maintainer":
            continue
        _raw, _text, doc = read_json(PROFILES / f"{stem}.json")
        if pack_id in doc["packs"]:
            owners.append(stem)
    return tuple(owners)


class PackMembershipMatrixTests(unittest.TestCase):
    """정본 4절 행렬을 파일에서 다시 계산한다."""

    def test_every_role_owned_pack_matches_catalog(self):
        for pack_id, expected in PACK_ROLE_OWNERS.items():
            self.assertEqual(_owners_of(pack_id), expected, pack_id)

    def test_casual_rides_only_family(self):
        self.assertEqual(_owners_of("casual-rides"), ("family",))

    def test_core_cli_starter_and_editor(self):
        self.assertEqual(_owners_of("core-cli"), ("starter", "editor"))

    def test_self_description_only_starter(self):
        self.assertEqual(_owners_of("self-description"), ("starter",))

    def test_text_editing_only_editor(self):
        self.assertEqual(_owners_of("text-editing"), ("editor",))

    def test_table_editing_only_editor(self):
        self.assertEqual(_owners_of("table-editing"), ("editor",))

    def test_objects_media_only_editor(self):
        self.assertEqual(_owners_of("objects-media"), ("editor",))

    def test_serialization_only_publisher(self):
        self.assertEqual(_owners_of("serialization"), ("publisher",))

    def test_layout_rendering_only_publisher(self):
        self.assertEqual(_owners_of("layout-rendering"), ("publisher",))

    def test_security_only_publisher(self):
        self.assertEqual(_owners_of("security"), ("publisher",))

    def test_corpus_diagnostics_only_operator(self):
        self.assertEqual(_owners_of("corpus-diagnostics"), ("operator",))

    def test_automation_only_operator(self):
        self.assertEqual(_owners_of("automation"), ("operator",))

    def test_expert_challenges_only_boss(self):
        self.assertEqual(_owners_of("expert-challenges"), ("boss",))

    def test_extraction_not_in_role_profiles(self):
        self.assertEqual(_owners_of("extraction"), ())

    def test_batch_ops_not_in_role_profiles(self):
        self.assertEqual(_owners_of("batch-ops"), ())

    def test_render_tree_not_in_role_profiles(self):
        self.assertEqual(_owners_of("render-tree"), ())

    def test_studio_e2e_not_in_role_profiles(self):
        self.assertEqual(_owners_of("studio-e2e"), ())

    def test_table_csv_not_in_role_profiles(self):
        self.assertEqual(_owners_of("table-csv"), ())

    def test_maintainer_only_hints_exist_or_are_absent_together(self):
        known = set(pack_ids())
        _raw, _text, maintainer = read_json(PROFILES / "maintainer.json")
        covered = set(maintainer["packs"])
        for pid in MAINTAINER_ONLY_HINTS:
            if pid in known:
                self.assertIn(pid, covered, pid)
                self.assertEqual(_owners_of(pid), (), pid)

    def test_every_existing_pack_is_either_role_owned_or_maintainer_only(self):
        known = set(pack_ids())
        role_owned = set(PACK_ROLE_OWNERS)
        leftover = sorted(known - role_owned)
        for pid in leftover:
            self.assertEqual(_owners_of(pid), (), pid)

    def test_catalog_does_not_name_missing_packs(self):
        known = set(pack_ids())
        missing = [pid for pid in PACK_ROLE_OWNERS if pid not in known]
        self.assertEqual(missing, [])


class PerProfileFileTests(unittest.TestCase):
    """파일 하나를 한 메서드가 끝까지 읽는다."""

    def _check(self, pid, title_token, packs):
        path = PROFILES / f"{pid}.json"
        raw, text, doc = read_json(path)
        self.assertFalse(raw.startswith(b"\xef\xbb\xbf"), pid)
        self.assertEqual(doc["id"], pid)
        self.assertEqual(doc["kind"], PROFILE_KIND)
        self.assertEqual(doc["schemaVersion"], SCHEMA_VERSION)
        self.assertIn(title_token, doc["title"])
        self.assertEqual(tuple(doc["packs"]), packs)
        self.assertTrue(text.endswith("\n"), pid)
        errors = []
        load_schema().validate_profile(doc, set(pack_ids()), errors)
        self.assertEqual(errors, [], f"{pid}: {errors}")
        loaded = load_runner().load_profile(pid)
        self.assertEqual(loaded, doc)

    def test_file_family(self):
        self._check("family", "가족", NAMED_PACKS["family"])

    def test_file_starter(self):
        self._check("starter", "입문", NAMED_PACKS["starter"])

    def test_file_editor(self):
        self._check("editor", "편집", NAMED_PACKS["editor"])

    def test_file_publisher(self):
        self._check("publisher", "배포", NAMED_PACKS["publisher"])

    def test_file_operator(self):
        self._check("operator", "운영", NAMED_PACKS["operator"])

    def test_file_boss(self):
        self._check("boss", "보스", NAMED_PACKS["boss"])

    def test_file_maintainer(self):
        path = PROFILES / "maintainer.json"
        _raw, _text, doc = read_json(path)
        self.assertEqual(doc["id"], "maintainer")
        self.assertIn("메인테이너", doc["title"])
        self.assertEqual(doc["packs"], sorted(doc["packs"]))
        self.assertEqual(doc["packs"], pack_ids())
        errors = []
        load_schema().validate_profile(doc, set(pack_ids()), errors)
        self.assertEqual(errors, [])


class DocsSectionTests(unittest.TestCase):
    """정본 절 제목이 사라지지 않게 한다."""

    REQUIRED_HEADINGS = (
        "# gym 프로파일 계약",
        "## 한 줄 결론",
        "## 1. 왜 프로파일인가",
        "## 2. 스키마",
        "## 3. 일곱 자리",
        "### 3.1 `family`",
        "### 3.2 `starter`",
        "### 3.3 `editor`",
        "### 3.4 `publisher`",
        "### 3.5 `operator`",
        "### 3.6 `boss`",
        "### 3.7 `maintainer`",
        "## 4. 자리 × pack 행렬",
        "## 5. 불변식",
        "## 6. 사용",
        "## 8. 새 프로파일을 넣는 법",
        "## 9. 새 pack 을 자리에 넣는 법",
        "## 10. 실패 칸",
        "## 11. 다른 기둥과 나눈 일",
    )

    def test_canonical_keeps_contract_headings(self):
        _raw, text = read_text(DOCS_PROFILES)
        missing = [h for h in self.REQUIRED_HEADINGS if h not in text]
        self.assertEqual(missing, [], f"절 제목 누락: {missing}")

    def test_working_keeps_decision_headings(self):
        _raw, text = read_text(WORKING_PROFILES)
        for heading in (
            "## 1. 결론",
            "## 4. 고친 파일 / 안 고친 파일",
            "## 5. 결정 로그",
            "## 6. 시험 지도",
            "## 8. 이웃 PR 충돌 회피",
            "feat/gym-profiles-hardening",
        ):
            self.assertIn(heading, text, heading)

    def test_canonical_mentions_no_new_cli(self):
        _raw, text = read_text(DOCS_PROFILES)
        self.assertIn("새 CLI 는 없다", text)

    def test_canonical_mentions_does_not_edit_schema(self):
        _raw, text = read_text(DOCS_PROFILES)
        self.assertIn("schema.py", text)
        self.assertIn("이 기둥이 그 함수를 고치지 않는다", text)

    def test_working_lists_three_added_paths(self):
        _raw, text = read_text(WORKING_PROFILES)
        self.assertIn("gym/docs/profiles.md", text)
        self.assertIn("mydocs/working/gym_profiles.md", text)
        self.assertIn("scripts/tests/test_gym_profiles.py", text)

    def test_matrix_rows_use_bullet_marker(self):
        _raw, text = read_text(DOCS_PROFILES)
        self.assertIn("| `casual-rides` | ● |", text)
        self.assertIn("| `expert-challenges` |", text)


class ScoreAllProfileSelectionTests(unittest.TestCase):
    """score_all의 profile 선택 순서를 실행 동작으로 고정한다."""

    def test_runner_source_order(self):
        runner = load_runner()
        events = []

        def fake_profile(profile_id):
            events.append(("load-profile", profile_id))
            return {"id": profile_id, "packs": ["profile-pack"]}

        def fake_pack(pack_id, *_args):
            events.append(("score-pack", pack_id))
            return {"id": pack_id, "status": "unavailable", "score": None, "max": 0}

        with (
            mock.patch.object(runner, "load_profile", side_effect=fake_profile),
            mock.patch.object(runner, "score_pack", side_effect=fake_pack),
            mock.patch.object(runner, "safe_runner_identity", return_value={}),
            mock.patch.object(runner, "safe_known_commands", return_value=None),
            mock.patch.object(runner, "bin_is_missing", return_value=False),
        ):
            card = runner.score_all(
                "unused-submissions",
                "unused-rhwp",
                pack_ids=["caller-pack"],
                profile_id="role",
            )

        self.assertEqual(events, [("load-profile", "role"), ("score-pack", "profile-pack")])
        self.assertEqual([pack["id"] for pack in card["packs"]], ["profile-pack"])

    def test_score_card_keeps_profile_field(self):
        text = (GYM / "core" / "runner.py").read_text(encoding="utf-8")
        self.assertIn('"profile": profile_id', text)

    def test_score_py_does_not_invent_list_flag(self):
        text = SCORE_PY.read_text(encoding="utf-8")
        self.assertNotIn("--list-profiles", text)
        self.assertNotIn("--profile-json", text)


class ValidateProfileMoreNegativeTests(unittest.TestCase):
    def setUp(self):
        self.schema = load_schema()
        self.known = set(pack_ids()) | {"core-cli", "security"}

    def _errors(self, doc):
        errors = []
        self.schema.validate_profile(doc, self.known, errors)
        return errors

    def test_empty_string_pack_is_unknown(self):
        errors = self._errors(minimal_profile(packs=[""]))
        self.assertTrue(errors)

    def test_none_kind(self):
        doc = minimal_profile()
        doc["kind"] = None
        self.assertTrue(any("kind" in e for e in self._errors(doc)))

    def test_multiple_unknown_packs_all_reported(self):
        doc = minimal_profile(packs=["ghost-a", "ghost-b"])
        errors = self._errors(doc)
        joined = "\n".join(errors)
        self.assertIn("ghost-a", joined)
        self.assertIn("ghost-b", joined)

    def test_real_named_profiles_against_real_pack_ids(self):
        errors = []
        for pid in NAMED_PROFILE_IDS:
            _raw, _text, doc = read_json(PROFILES / f"{pid}.json")
            self.schema.validate_profile(doc, set(pack_ids()), errors)
        self.assertEqual(errors, [])


class ProfileSourceCommentTests(unittest.TestCase):
    """시험 파일 머리글이 이슈 번호를 잃지 않게 한다."""

    def test_module_docstring_mentions_issue(self):
        text = Path(__file__).read_text(encoding="utf-8")
        self.assertIn("#5281", text)
        self.assertIn("family", text)
        self.assertIn("operator", text)
        self.assertIn("점수를 뭉치는 도구가 아니다", text)

    def test_forbidden_files_are_named_in_docstring(self):
        text = Path(__file__).read_text(encoding="utf-8")
        for name in ("schema.py", "audit.py", "certify.py", "report.py", "PARK"):
            self.assertIn(name, text, name)


class PackAxisTableDocTests(unittest.TestCase):
    """정본 31절이 현재 pack id 를 빠뜨리지 않는지. title 은 잠그지 않는다."""

    def test_canonical_mentions_every_current_pack_id(self):
        _raw, text = read_text(DOCS_PROFILES)
        missing = [pid for pid in pack_ids() if f"`{pid}`" not in text]
        self.assertEqual(missing, [], f"정본이 pack id 를 빼먹었다: {missing}")

    def test_canonical_has_troubleshooting_tokens(self):
        _raw, text = read_text(DOCS_PROFILES)
        for token in (
            "없는 pack 참조",
            "indent/키 순서 불일치",
            "FileNotFoundError",
            "고르지 않은 pack 은 점수가 아니라 부재",
        ):
            self.assertIn(token, text, token)

    def test_working_records_title_not_locked(self):
        _raw, text = read_text(WORKING_PROFILES)
        self.assertIn("title 문자열은 시험이", text)

    def test_working_has_reviewer_grep_block(self):
        _raw, text = read_text(WORKING_PROFILES)
        self.assertIn("git diff --name-only upstream/devel", text)
        self.assertIn("gym/core/schema.py", text)
        self.assertIn("수락 시나리오", text)


if __name__ == "__main__":
    unittest.main()
