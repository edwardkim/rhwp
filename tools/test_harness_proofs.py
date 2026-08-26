import importlib.util
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("harness_proofs.py")
SPEC = importlib.util.spec_from_file_location("harness_proofs", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
HARNESS_PROOFS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(HARNESS_PROOFS)


def capabilities(command_count: int) -> dict:
    return {
        "commands": [{"name": f"command-{i}"} for i in range(command_count)],
        "exitCodes": {"0": "success", "1": "runtime failure", "2": "usage error"},
        "jsonContract": {
            "stdout": "JSON data only",
            "schemaPolicy": "additive fields only",
        },
    }


class CommandSurfaceContractTests(unittest.TestCase):
    def test_documented_command_floor_passes(self) -> None:
        ok, detail = HARNESS_PROOFS.command_surface_contract(
            capabilities(HARNESS_PROOFS.EXPECTED_COMMAND_FLOOR)
        )

        self.assertTrue(ok, detail)
        self.assertIn("floor=68", detail)

    def test_count_below_documented_floor_fails(self) -> None:
        count = HARNESS_PROOFS.EXPECTED_COMMAND_FLOOR - 1
        ok, detail = HARNESS_PROOFS.command_surface_contract(capabilities(count))

        self.assertFalse(ok, detail)
        self.assertIn(f"commands={count}", detail)

    def test_count_above_documented_floor_passes(self) -> None:
        count = HARNESS_PROOFS.EXPECTED_COMMAND_FLOOR + 1
        ok, detail = HARNESS_PROOFS.command_surface_contract(capabilities(count))

        self.assertTrue(ok, detail)

    def test_commands_require_objects_nonempty_unique_names(self) -> None:
        cases = []
        not_object = capabilities(HARNESS_PROOFS.EXPECTED_COMMAND_FLOOR)
        not_object["commands"][0] = "command-0"
        cases.append((not_object, "commands[0]"))
        empty_name = capabilities(HARNESS_PROOFS.EXPECTED_COMMAND_FLOOR)
        empty_name["commands"][0]["name"] = "  "
        cases.append((empty_name, "commands[0].name"))
        duplicate = capabilities(HARNESS_PROOFS.EXPECTED_COMMAND_FLOOR)
        duplicate["commands"][-1]["name"] = duplicate["commands"][0]["name"]
        cases.append((duplicate, "중복"))

        for caps, expected_detail in cases:
            with self.subTest(expected_detail=expected_detail):
                ok, detail = HARNESS_PROOFS.command_surface_contract(caps)
                self.assertFalse(ok, detail)
                self.assertIn(expected_detail, detail)

    def test_exit_codes_require_an_object_with_core_meanings(self) -> None:
        for value in (None, {}, {"0": "", "1": "runtime failure", "2": "usage error"}):
            with self.subTest(value=value):
                caps = capabilities(HARNESS_PROOFS.EXPECTED_COMMAND_FLOOR)
                caps["exitCodes"] = value
                ok, detail = HARNESS_PROOFS.command_surface_contract(caps)
                self.assertFalse(ok, detail)
                self.assertIn("exitCodes", detail)

    def test_json_contract_requires_an_object_with_core_meanings(self) -> None:
        for value in (None, {}, {"stdout": "JSON data only", "schemaPolicy": ""}):
            with self.subTest(value=value):
                caps = capabilities(HARNESS_PROOFS.EXPECTED_COMMAND_FLOOR)
                caps["jsonContract"] = value
                ok, detail = HARNESS_PROOFS.command_surface_contract(caps)
                self.assertFalse(ok, detail)
                self.assertIn("jsonContract", detail)


class ProvenanceMarkerContractTests(unittest.TestCase):
    def test_document_derived_envelope_requires_true_and_nonempty_paths(self) -> None:
        ok, detail = HARNESS_PROOFS.provenance_marker_contract(
            {"untrustedContent": True, "untrustedFields": ["title", "metadata.author"]}
        )

        self.assertTrue(ok, detail)

    def test_false_untrusted_content_is_rejected(self) -> None:
        ok, detail = HARNESS_PROOFS.provenance_marker_contract(
            {"untrustedContent": False, "untrustedFields": ["title"]}
        )

        self.assertFalse(ok, detail)
        self.assertIn("untrustedContent=False", detail)

    def test_empty_or_malformed_untrusted_fields_are_rejected(self) -> None:
        for fields in ([], [""], [None], "title"):
            with self.subTest(fields=fields):
                ok, detail = HARNESS_PROOFS.provenance_marker_contract(
                    {"untrustedContent": True, "untrustedFields": fields}
                )
                self.assertFalse(ok, detail)
                self.assertIn("untrustedFields", detail)


if __name__ == "__main__":
    unittest.main()
