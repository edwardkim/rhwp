"""P0 하니스의 오라클 격리·재사용 계약을 빠르게 검증한다."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path, PurePosixPath, PureWindowsPath

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from compare import classify, selected_oracle_paths
from extract_spec import verify
from oracle_version import matches_expected_version
from run_package_gate import gate_command
from scenario_spec import call_contract, platform_path_key, resolve_args
from run_gate import (
    cleanup_and_wait_for_hwp_exit,
    hwp_pids,
    new_hwp_pids,
    oracle_mode,
    stored_oracle_status,
    validate_rhwp_output,
    wait_for_hwp_exit,
)
from runner_ocx import clear_previous_outputs, discard_changes_and_quit


class HarnessContractTests(unittest.TestCase):
    def test_oracle_quit_discards_modified_document_before_closing(self) -> None:
        calls: list[tuple[str, object | None]] = []

        class FakeHwp:
            def clear(self, *, option: int) -> None:
                calls.append(("clear", option))

        class FakeCom:
            def Quit(self) -> None:
                calls.append(("quit", None))

        discard_changes_and_quit(FakeHwp(), FakeCom())

        self.assertEqual(calls, [("clear", 1), ("quit", None)])

    def test_oracle_quit_still_runs_when_discard_fails(self) -> None:
        calls: list[str] = []

        class FailingHwp:
            def clear(self, *, option: int) -> None:
                if option != 1:
                    raise AssertionError(f"unexpected discard option: {option}")
                raise RuntimeError("discard failed")

        class FakeCom:
            def Quit(self) -> None:
                calls.append("quit")

        discard_changes_and_quit(FailingHwp(), FakeCom())

        self.assertEqual(calls, ["quit"])

    def test_hancom_2022_version_spellings_use_the_same_major(self) -> None:
        self.assertTrue(matches_expected_version("12, 0, 0, 4547", "12"))
        self.assertTrue(matches_expected_version("12.0.0.4547", "12,"))
        self.assertFalse(matches_expected_version("13.0.0.1", "12"))
        self.assertFalse(matches_expected_version(None, "12"))

    def test_previous_oracle_outputs_are_removed_before_a_new_run(self) -> None:
        scenario = {"id": "field-read", "saveAs": "nested/result.hwp"}
        with tempfile.TemporaryDirectory() as directory:
            out_dir = Path(directory)
            for relative in ("field-read.returns.json", "field-read.rejected.json", "nested/result.hwp"):
                path = out_dir / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("old", encoding="utf-8")
            returns, rejected, saved = clear_previous_outputs(scenario, out_dir)
            self.assertFalse(returns.exists())
            self.assertFalse(rejected.exists())
            self.assertIsNotNone(saved)
            self.assertFalse(saved.exists())

    def test_oracle_output_cannot_escape_the_requested_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(ValueError):
                clear_previous_outputs({"id": "escape", "saveAs": "../outside.hwp"}, Path(directory))

    def test_skip_ocx_rejects_wrong_or_invalid_oracle(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "field-read.returns.json"
            path.write_text(json.dumps({"oracle": {"version": "12.0.0.4547"}}), encoding="utf-8")
            self.assertEqual(stored_oracle_status(path, "12"), "SKIPPED")
            path.write_text(json.dumps({"oracle": {"version": "13.0.0.1"}}), encoding="utf-8")
            self.assertEqual(stored_oracle_status(path, "12"), "STALE_ORACLE")
            path.write_text("not-json", encoding="utf-8")
            self.assertEqual(stored_oracle_status(path, "12"), "INVALID_ORACLE")

    def test_non_windows_defaults_to_wasm_self_check(self) -> None:
        self.assertEqual(oracle_mode("Linux", False, False, None), "wasm-self-check")
        self.assertEqual(oracle_mode("Darwin", False, False, None), "wasm-self-check")
        self.assertEqual(oracle_mode("Windows", False, False, None), "live")
        self.assertEqual(oracle_mode("Linux", False, False, Path("fixture")), "fixture")
        self.assertEqual(oracle_mode("Darwin", False, False, None, fixture=True), "fixture")

    def test_package_gate_cleans_only_windows_com_processes(self) -> None:
        self.assertIn("--cleanup-spawned", gate_command("Windows"))
        self.assertNotIn("--cleanup-spawned", gate_command("Linux"))
        self.assertNotIn("--cleanup-spawned", gate_command("Darwin"))

    def test_wasm_output_requires_successful_calls_and_saved_file(self) -> None:
        scenario = {"id": "sample", "open": "samples/input.hwp", "calls": [["GetPos", []]], "saveAs": "saved.hwp"}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scenario_path = root / "sample.json"
            scenario_path.write_text(json.dumps(scenario), encoding="utf-8")
            saved = root / "saved.hwp"
            saved.write_bytes(b"HWP")
            (root / "sample.returns.json").write_text(
                json.dumps(
                    {
                        "calls": [{"call": "Open", "value": True}, {"call": "GetPos", "value": {}}],
                        "saved": {"path": str(saved), "ok": True},
                        "fatal": None,
                    }
                ),
                encoding="utf-8",
            )
            self.assertEqual(validate_rhwp_output(scenario_path, root), "OK")
            (root / "sample.returns.json").write_text(
                json.dumps(
                    {
                        "calls": [
                            {"call": "Open", "value": True},
                            {"call": "GetPos", "error": "MissingApi"},
                        ],
                        "fatal": None,
                    }
                ),
                encoding="utf-8",
            )
            self.assertEqual(validate_rhwp_output(scenario_path, root), "CALL_ERROR")

    def _validate(self, calls: list, records: list) -> str:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "s.json").write_text(json.dumps({"id": "s", "calls": calls}), encoding="utf-8")
            (root / "s.returns.json").write_text(
                json.dumps({"calls": records, "fatal": None}), encoding="utf-8"
            )
            return validate_rhwp_output(root / "s.json", root)

    def test_declared_exception_passes_only_when_it_dies_the_declared_way(self) -> None:
        """일부러 죽는 호출을 공식 시나리오에 두려면 **미리 선언**해야 한다 (#4274 리뷰).

        선언은 면제가 아니라 계약이다 — 안 죽어도, 딴 이유로 죽어도 실패다.
        """
        declared = [["Foo", [], {"expectError": {"rhwp": "죽어야 한다", "ocx": None}}]]
        self.assertEqual(self._validate(declared, [{"call": "Foo", "error": "Error: 죽어야 한다"}]), "OK")
        self.assertEqual(self._validate(declared, [{"call": "Foo", "value": 1}]), "EXPECT_DIFF")
        self.assertEqual(self._validate(declared, [{"call": "Foo", "error": "Error: 딴 이유"}]), "CALL_ERROR")

    def test_missing_api_never_satisfies_a_declared_exception(self) -> None:
        """"아직 안 만들었다"가 예외 선언 뒤에 숨지 못하게 한다 — 리뷰가 막은 그 구멍이다."""
        declared = [["Foo", [], {"expectError": {"rhwp": "죽어야 한다", "ocx": None}}]]
        self.assertEqual(self._validate(declared, [{"call": "Foo", "error": "MissingApi: Foo"}]), "CALL_ERROR")
        with self.assertRaises(ValueError):
            call_contract(["Foo", [], {"expectError": {"rhwp": "MissingApi: Foo", "ocx": None}}])

    def test_undeclared_error_still_fails_and_expected_value_is_checked(self) -> None:
        self.assertEqual(self._validate([["Foo", []]], [{"call": "Foo", "error": "Error: x"}]), "CALL_ERROR")
        self.assertEqual(self._validate([["Foo", [], {"expect": 4}]], [{"call": "Foo", "value": 5}]), "EXPECT_DIFF")
        self.assertEqual(self._validate([["Foo", [], {"expect": 4}]], [{"call": "Foo", "value": 4}]), "OK")

    def test_scenario_paths_resolve_per_platform(self) -> None:
        """Windows 오라클 경로와 Linux 경로를 갈라 놓는다 — 하나로 쓰면 Linux 에서 뜻이 달라진다."""
        definition = {
            "paths": {"pic": {"win": "{repo}\\samples\\s1.jpg", "posix": "{repo}/samples/s1.jpg"},
                      "out": {"win": "{out}\\a.bmp", "posix": "{out}/a.bmp"}}
        }
        args = [{"$path": "pic"}, {"$path": "out"}, 0]
        # 각 OS 갈래는 그 OS의 root type으로 넓힌다. contributor 개인 홈 경로를 적으면 다른
        # Windows Oracle host에서 gate가 재현되지 않는다.
        win_repo, win_out = PureWindowsPath(r"C:\repo"), PureWindowsPath(r"C:\out")
        self.assertEqual(
            resolve_args(args, definition, platform_path_key("Windows"), win_repo, win_out),
            [r"C:\repo\samples\s1.jpg", r"C:\out\a.bmp", 0],
        )
        repo, out_dir = PurePosixPath("/repo"), PurePosixPath("/out")
        self.assertEqual(
            resolve_args(args, definition, platform_path_key("Linux"), repo, out_dir),
            ["/repo/samples/s1.jpg", "/out/a.bmp", 0],
        )
        with self.assertRaises(ValueError):
            resolve_args([{"$path": "없음"}], definition, "posix", repo, out_dir)

    def test_tracked_scenario_paths_do_not_depend_on_a_contributor_home(self) -> None:
        for path in sorted((HERE / "scenarios").glob("*.json")):
            definition = json.loads(path.read_text(encoding="utf-8"))
            for name, variants in (definition.get("paths") or {}).items():
                for platform_name in ("win", "posix"):
                    value = variants[platform_name]
                    self.assertTrue(
                        "{repo}" in value or "{out}" in value,
                        f"{path.name}:{name}:{platform_name} must use a portable root token",
                    )
                    self.assertNotIn("C:\\Users\\", value, f"{path.name}:{name}:{platform_name}")

    def test_both_sides_dying_is_not_a_match_unless_declared(self) -> None:
        """양쪽이 죽었다는 것만으로는 일치가 아니다 — `MissingApi` 를 막은 것과 같은 이유다."""
        ocx = {"call": "Foo", "error": "com_error: ..."}
        rhwp = {"call": "Foo", "error": "Error: 죽어야 한다"}
        self.assertEqual(classify(ocx, rhwp, {})[0], "ERROR_UNDECLARED")
        declared = {"expectError": {"rhwp": "죽어야 한다", "ocx": None}}
        self.assertEqual(classify(ocx, rhwp, declared)[0], "MATCH")
        # 오라클 문구를 재고 나면 그 문구까지 대조한다.
        measured = {"expectError": {"rhwp": "죽어야 한다", "ocx": "딴 문구"}}
        self.assertEqual(classify(ocx, rhwp, measured)[0], "EXPECT_DIFF")

    def test_declared_return_value_is_checked_against_the_oracle_too(self) -> None:
        """자체 검사(Linux)와 오라클 대조(Windows)가 **같은 한 값**을 본다."""
        contract = {"expect": False}
        self.assertEqual(classify({"call": "Foo", "value": False}, {"call": "Foo", "value": False}, contract)[0], "MATCH")
        self.assertEqual(classify({"call": "Foo", "value": True}, {"call": "Foo", "value": True}, contract)[0], "EXPECT_DIFF")

    def test_every_scenario_declares_a_well_formed_contract(self) -> None:
        for path in sorted((HERE / "scenarios").glob("*.json")):
            definition = json.loads(path.read_text(encoding="utf-8"))
            for call in definition.get("calls", []):
                call_contract(call)
            for name, variants in (definition.get("paths") or {}).items():
                self.assertEqual(set(variants), {"win", "posix"}, f"{path.name}:{name}")

    def test_three_way_verdicts_partition_the_disagreement_space(self) -> None:
        """3자 판정 — 어느 둘이 같은지가 곧 판정이다(compare3.classify3)."""
        from compare3 import classify3, ALL_AGREE, COM_DRIFT, IMPL_GAP, WEB_DIVERGES, ALL_DIFFER

        v = lambda x: {"call": "Foo", "value": x}  # noqa: E731
        self.assertEqual(classify3(v(1), v(1), v(1))[0], ALL_AGREE)
        # 기안기·rhwp 일치, COM 만 다름 — 프록시의 한계. rhwp 는 이미 제품과 맞다.
        self.assertEqual(classify3(v(2), v(1), v(1))[0], COM_DRIFT)
        # 두 오라클 일치, rhwp 만 다름 — 실 결함.
        self.assertEqual(classify3(v(1), v(1), v(2))[0], IMPL_GAP)
        # COM·rhwp 일치, 기안기만 다름 — 웹 계약이 갈리는 지점. 기안기가 이긴다.
        self.assertEqual(classify3(v(1), v(2), v(1))[0], WEB_DIVERGES)
        self.assertEqual(classify3(v(1), v(2), v(3))[0], ALL_DIFFER)

    def test_three_way_errors_compare_by_kind_not_by_message(self) -> None:
        """오류는 '죽었다'로만 묶는다 — 문구는 러너·플랫폼마다 달라 러너 차이가 판정을 오염시킨다."""
        from compare3 import classify3, ALL_AGREE, IMPL_GAP, WEB_DIVERGES

        v = lambda x: {"call": "Foo", "value": x}  # noqa: E731
        e = lambda m: {"call": "Foo", "error": m}  # noqa: E731
        self.assertEqual(classify3(e("com_error: A"), e("TypeError: B"), e("Error: C"))[0], ALL_AGREE)
        self.assertEqual(classify3(e("com_error: A"), e("TypeError: B"), v(1))[0], IMPL_GAP)
        self.assertEqual(classify3(v(1), e("TypeError: B"), v(1))[0], WEB_DIVERGES)

    def test_web_open_envelope_compares_only_the_contract_common_denominator(self) -> None:
        """기안기 `Open` 봉투의 `fileName` 은 서버 부여 난수 — `result` 만 판정 잣대다."""
        from compare3 import ALL_AGREE, WEB_DIVERGES, classify3, project_web

        web = {
            "call": "Open",
            "value": {"result": True, "fileName": "216e9d77.hwp", "orgName": "a.hwp", "size": 84992},
        }
        local = {"call": "Open", "value": True}
        self.assertEqual(classify3(local, project_web(web), local)[0], ALL_AGREE)
        # 성공 신호가 갈리면 봉투를 벗겨도 갈린 것으로 남는다.
        failed = {**web, "value": {**web["value"], "result": False}}
        self.assertEqual(classify3(local, project_web(failed), local)[0], WEB_DIVERGES)
        # 투영은 Open 봉투에만 닿는다 — 다른 호출·오류·비봉투 값은 그대로 지난다.
        other = {"call": "GetPos", "value": {"list": 0, "para": 0, "pos": 16}}
        self.assertEqual(project_web(other), other)
        self.assertEqual(project_web({"call": "Open", "value": True}), {"call": "Open", "value": True})
        died = {"call": "Open", "error": "TypeError: x"}
        self.assertEqual(project_web(died), died)

    def test_web_results_without_a_version_stamp_are_rejected(self) -> None:
        """스탬프(URL·측정 시각) 없는 기안기 산출물은 정답지 자격이 없다(계획서 §6.3.3)."""
        from compare3 import require_stamp

        good = {"oracle": {"url": "https://demo/", "measuredAt": "2026-08-10T00:00:00Z"}}
        require_stamp(good, Path("x"))
        for bad in ({}, {"oracle": {}}, {"oracle": {"url": "https://demo/"}}):
            with self.assertRaises(SystemExit):
                require_stamp(bad, Path("x"))

    def test_compare_only_reads_explicitly_eligible_oracles(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            out_dir = Path(directory)
            for name in ("fresh", "stale"):
                (out_dir / f"{name}.returns.json").write_text("{}", encoding="utf-8")
            self.assertEqual(
                [path.name for path in selected_oracle_paths(out_dir, ["fresh"])],
                ["fresh.returns.json"],
            )

    def test_compare_returns_failure_when_a_value_diff_exists(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            ocx_dir, rhwp_dir, verdict_dir = root / "ocx", root / "rhwp", root / "verdict"
            ocx_dir.mkdir()
            rhwp_dir.mkdir()
            oracle = {"scenario": "doc-basic", "calls": [{"call": "PageCount", "value": 1}], "saved": None}
            implementation = {"scenario": "doc-basic", "calls": [{"call": "PageCount", "value": 2}], "saved": None}
            (ocx_dir / "doc-basic.returns.json").write_text(json.dumps(oracle), encoding="utf-8")
            (rhwp_dir / "doc-basic.returns.json").write_text(json.dumps(implementation), encoding="utf-8")
            proc = subprocess.run(
                [
                    sys.executable,
                    str(HERE / "compare.py"),
                    "--ocx",
                    str(ocx_dir),
                    "--rhwp",
                    str(rhwp_dir),
                    "--out",
                    str(verdict_dir),
                ],
                capture_output=True,
                check=False,
            )
            self.assertEqual(proc.returncode, 1, proc.stdout.decode("utf-8", "replace"))

    def test_tasklist_parser_selects_only_hancom_processes(self) -> None:
        tasklist = 'Hwp.exe,101,Console,1,10 K\r\nHwpFrame.exe,202,Console,1,10 K\r\nnode.exe,303,Console,1,10 K\r\n'
        self.assertEqual(hwp_pids(tasklist), {"101", "202"})

    def test_new_process_detection_does_not_terminate_anything(self) -> None:
        from unittest.mock import patch

        tasklist = 'Hwp.exe,101,Console,1,10 K\r\nHwpFrame.exe,202,Console,1,10 K\r\n'
        with patch("run_gate.hwp_pids", return_value=hwp_pids(tasklist)):
            self.assertEqual(new_hwp_pids({"101"}), {"202"})

    def test_quit_settle_waits_for_asynchronous_hancom_exit(self) -> None:
        from unittest.mock import patch

        with patch("run_gate.new_hwp_pids", side_effect=[{"202"}, set()]), patch("run_gate.time.sleep") as sleep:
            self.assertEqual(wait_for_hwp_exit({"101"}, 10.0), set())
        sleep.assert_called_once()

    def test_forced_cleanup_also_waits_for_hancom_exit(self) -> None:
        from unittest.mock import patch

        with (
            patch("run_gate.cleanup_spawned_hwp") as cleanup,
            patch("run_gate.wait_for_hwp_exit", return_value=set()) as wait,
        ):
            self.assertEqual(cleanup_and_wait_for_hwp_exit({"101"}, 10.0), set())
        cleanup.assert_called_once_with({"101"})
        wait.assert_called_once_with({"101"}, 10.0)

    def test_tracked_spec_keeps_all_declared_parameter_set_items(self) -> None:
        spec_dir = HERE.parents[1] / "npm" / "hwpctrl-ocx" / "spec"
        api = json.loads((spec_dir / "webhwpctrl_api.json").read_text(encoding="utf-8"))["entries"]
        sets = json.loads((spec_dir / "parameter_sets.json").read_text(encoding="utf-8"))["sets"]
        actions = json.loads((spec_dir / "actions.json").read_text(encoding="utf-8"))["actions"]
        self.assertEqual(verify(api, sets, actions), [])


if __name__ == "__main__":
    # Windows 콘솔은 기본이 cp949 라, 검사 대상이 찍는 `—` 하나에 테스트가 통째로 죽었다
    # (실패가 아니라 UnicodeEncodeError 였다). 저장소의 다른 스크립트와 같은 자리를 둔다.
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")

if __name__ == "__main__":
    unittest.main()
