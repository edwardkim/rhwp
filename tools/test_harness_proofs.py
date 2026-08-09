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
        "commands": [{} for _ in range(command_count)],
        "exitCodes": {},
        "jsonContract": {},
    }


class CommandSurfaceContractTests(unittest.TestCase):
    def test_exact_documented_command_count_passes(self) -> None:
        ok, detail = HARNESS_PROOFS.command_surface_contract(
            capabilities(HARNESS_PROOFS.EXPECTED_COMMAND_COUNT)
        )

        self.assertTrue(ok, detail)
        self.assertIn("expected=68", detail)

    def test_missing_or_extra_commands_fail(self) -> None:
        for count in (67, 69):
            with self.subTest(count=count):
                ok, detail = HARNESS_PROOFS.command_surface_contract(capabilities(count))

                self.assertFalse(ok, detail)
                self.assertIn(f"commands={count}", detail)

    def test_contract_fields_remain_required(self) -> None:
        caps = capabilities(HARNESS_PROOFS.EXPECTED_COMMAND_COUNT)
        del caps["jsonContract"]

        ok, detail = HARNESS_PROOFS.command_surface_contract(caps)

        self.assertFalse(ok, detail)
        self.assertIn("jsonContract=False", detail)


if __name__ == "__main__":
    unittest.main()
