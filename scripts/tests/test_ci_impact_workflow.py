from __future__ import annotations

import os
import re
import subprocess
import textwrap
import unittest
from pathlib import Path


WORKFLOW_PATH = Path(__file__).resolve().parents[2] / ".github/workflows/ci.yml"
CLASSIFIER_PATH = Path(__file__).resolve().parents[1] / "ci-impact-classifier.cjs"
TESTS_DIR = Path(__file__).resolve().parents[2] / "tests"
WORKER_MARKER = "  # [#2393] 기본 테스트 병렬화"

# [#4040] 파일 전체가 native-skia 로 게이트된 integration test.
#
# default-feature worker 는 이 파일을 통째로 cfg-out 하므로, Native Skia job 이
# 명시적으로 실행하지 않으면 **어디에서도 돌지 않는다.**
#
# 판별은 양쪽 방향의 오탐을 모두 막아야 한다. 한쪽으로 좁으면 부류를 놓치고,
# 반대쪽으로 넓으면 배선할 이유가 없는 파일을 배선하라고 요구한다.
#
# - 게이트는 중첩된다 — `render_p37_direct_pdf_export.rs` 는
#   `#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]` 형태라
#   정확 일치로 좁히면 놓친다. 그래서 괄호 균형으로 술어를 잘라 본다.
# - `not(feature = "native-skia")` 는 **정반대 조건**이고 `any(feature =
#   "native-skia", target_os = "linux")` 는 Linux 에서 feature 없이도 참이다.
#   native-skia 를 끈 상태에서 술어가 반드시 거짓일 때만 이 부류로 본다.
# - 문자열·줄/블록 주석 안의 cfg 인용과 블록 안의 inner attribute 는 crate
#   게이트가 아니다. Rust 비코드 영역을 같은 길이의 공백으로 가린 뒤 최상위
#   inner attribute 만 찾는다.
_INNER_CFG_OPEN = re.compile(r"#!\[\s*cfg\s*\(")
_OUTER_CFG_OPEN = re.compile(r"#\s*\[\s*cfg\s*\(")
_RAW_STRING_OPEN = re.compile(r'(?:br|rb|r)(?P<hashes>#{0,255})"')
_CFG_TOKEN = re.compile(
    r'\s*(?:(?P<ident>[A-Za-z_][A-Za-z0-9_]*)|'
    r'(?P<string>"(?:\\.|[^"\\])*")|(?P<punct>[(),=]))'
)
_FUNCTION_DECLARATION = re.compile(
    r"(?<![A-Za-z0-9_])"
    r"(?:(?:pub(?:\s*\([^()\n]*\))?)\s+)?"
    r"(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+)?fn\s+"
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
)
_MODULE_DECLARATION = re.compile(
    r"(?<![A-Za-z0-9_])"
    r"(?:(?:pub(?:\s*\([^()\n]*\))?)\s+)?mod\s+"
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
)
_CFG_ATTR_OPEN = re.compile(r"#\s*\[\s*cfg_attr\s*\(")
_PATH_ATTRIBUTE_OPEN = re.compile(r"#\s*\[\s*path\s*=")
_TEST_ATTRIBUTE = re.compile(r"#\s*\[\s*test\s*\]")


def _mask_rust_non_code(source: str) -> str:
    """문자열과 중첩 가능 주석을 같은 길이의 공백으로 가린다."""
    masked = list(source)

    def blank(start: int, end: int) -> None:
        for offset in range(start, end):
            if masked[offset] != "\n":
                masked[offset] = " "

    index = 0
    while index < len(source):
        if source.startswith("//", index):
            end = source.find("\n", index + 2)
            end = len(source) if end == -1 else end
            blank(index, end)
            index = end
            continue

        if source.startswith("/*", index):
            depth = 1
            end = index + 2
            while end < len(source) and depth > 0:
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            blank(index, end)
            index = end
            continue

        raw = _RAW_STRING_OPEN.match(source, index)
        if raw:
            terminator = '"' + raw.group("hashes")
            end = source.find(terminator, raw.end())
            end = len(source) if end == -1 else end + len(terminator)
            blank(index, end)
            index = end
            continue

        if source[index] == '"':
            end = index + 1
            escaped = False
            while end < len(source):
                char = source[end]
                end += 1
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == '"':
                    break
            blank(index, end)
            index = end
            continue

        index += 1

    return "".join(masked)


def _brace_depths(code: str) -> list[int]:
    depths = []
    depth = 0
    for char in code:
        depths.append(depth)
        if char == "{":
            depth += 1
        elif char == "}" and depth > 0:
            depth -= 1
    return depths


def _matching_delimiter(code: str, opened: int, opening: str, closing: str) -> int | None:
    """마스킹된 Rust 코드에서 `opened`와 짝인 닫는 구분자 위치를 찾는다."""
    depth = 1
    index = opened + 1
    while index < len(code):
        if code[index] == opening:
            depth += 1
        elif code[index] == closing:
            depth -= 1
            if depth == 0:
                return index
        index += 1
    return None


def _outer_attributes_before(source: str, code: str, item_start: int) -> str:
    """item 바로 앞에 연속된 outer attribute 원문을 반환한다."""
    cursor = item_start
    attributes = []
    while True:
        while cursor > 0 and code[cursor - 1].isspace():
            cursor -= 1
        if cursor == 0 or code[cursor - 1] != "]":
            break

        close = cursor - 1
        depth = 1
        opened = close - 1
        while opened >= 0 and depth > 0:
            if code[opened] == "]":
                depth += 1
            elif code[opened] == "[":
                depth -= 1
            opened -= 1
        if depth != 0:
            break

        opened += 1
        hash_at = opened - 1
        while hash_at >= 0 and code[hash_at].isspace():
            hash_at -= 1
        if hash_at < 0 or code[hash_at] != "#":
            break
        if hash_at + 1 < len(code) and code[hash_at + 1] == "!":
            break

        attributes.append(source[hash_at:close + 1])
        cursor = hash_at

    return "\n".join(reversed(attributes))


def _item_body_range(code: str, declaration_end: int) -> tuple[int, int] | None:
    """함수 또는 inline module 선언 뒤의 body 범위를 반환한다."""
    index = declaration_end
    paren_depth = 0
    bracket_depth = 0
    while index < len(code):
        char = code[index]
        if char == "(":
            paren_depth += 1
        elif char == ")" and paren_depth > 0:
            paren_depth -= 1
        elif char == "[":
            bracket_depth += 1
        elif char == "]" and bracket_depth > 0:
            bracket_depth -= 1
        elif paren_depth == 0 and bracket_depth == 0:
            if char == ";":
                return None
            if char == "{":
                closing = _matching_delimiter(code, index, "{", "}")
                return None if closing is None else (index, closing)
        index += 1
    return None


def _cfg_predicate_at(source: str, code: str, opened: re.Match[str]) -> str | None:
    depth = 1
    index = opened.end()
    while index < len(code) and depth > 0:
        if code[index] == "(":
            depth += 1
        elif code[index] == ")":
            depth -= 1
        index += 1
    if depth != 0:
        return None
    return source[opened.end():index - 1]


def _cfg_predicates_in_attributes(attributes: str) -> list[str]:
    code = _mask_rust_non_code(attributes)
    predicates = []
    for opened in _OUTER_CFG_OPEN.finditer(code):
        predicate = _cfg_predicate_at(attributes, code, opened)
        if predicate is not None:
            predicates.append(predicate)
    return predicates


def _split_top_level_comma(source: str) -> tuple[str, str] | None:
    depth = 0
    for index, char in enumerate(source):
        if char == "(":
            depth += 1
        elif char == ")" and depth > 0:
            depth -= 1
        elif char == "," and depth == 0:
            return source[:index], source[index + 1:]
    return None


def _cfg_attr_enables_native_skia_test(attributes: str) -> bool:
    """`cfg_attr(native-skia 술어, test)`가 있는지 판정한다."""
    code = _mask_rust_non_code(attributes)
    for opened in _CFG_ATTR_OPEN.finditer(code):
        arguments = _cfg_predicate_at(attributes, code, opened)
        if arguments is None:
            continue
        split = _split_top_level_comma(arguments)
        if split is None:
            continue
        predicate, applied = split
        if (
            _requires_native_skia_enabled(predicate)
            and re.search(r"(?:^|,)\s*test\s*(?:,|$)", applied)
        ):
            return True
    return False


def _inner_cfg_predicates(source: str) -> list[str]:
    """crate 최상위 `#![cfg(...)]` 술어를 괄호 균형으로 잘라낸다."""
    code = _mask_rust_non_code(source)
    brace_depth = _brace_depths(code)

    predicates = []
    for opened in _INNER_CFG_OPEN.finditer(code):
        if brace_depth[opened.start()] != 0:
            continue
        predicate = _cfg_predicate_at(source, code, opened)
        if predicate is not None:
            predicates.append(predicate)
    return predicates


class _CfgParser:
    """이 계약에 필요한 Rust cfg meta-item의 작은 재귀 하강 parser."""

    def __init__(self, source: str) -> None:
        self.tokens = []
        index = 0
        while index < len(source):
            token = _CFG_TOKEN.match(source, index)
            if not token:
                if source[index:].strip():
                    raise ValueError(f"unsupported cfg syntax: {source[index:]!r}")
                break
            self.tokens.append(token.group("ident") or token.group("string") or token.group("punct"))
            index = token.end()
        self.index = 0

    def _take(self) -> str:
        if self.index >= len(self.tokens):
            raise ValueError("unexpected end of cfg predicate")
        token = self.tokens[self.index]
        self.index += 1
        return token

    def _accept(self, expected: str) -> bool:
        if self.index < len(self.tokens) and self.tokens[self.index] == expected:
            self.index += 1
            return True
        return False

    def parse(self) -> tuple:
        expression = self._expression()
        if self.index != len(self.tokens):
            raise ValueError(f"trailing cfg tokens: {self.tokens[self.index:]!r}")
        return expression

    def _expression(self) -> tuple:
        name = self._take()
        if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name):
            raise ValueError(f"expected cfg name, got {name!r}")

        if self._accept("="):
            value = self._take()
            if not (value.startswith('"') and value.endswith('"')):
                raise ValueError(f"expected cfg string, got {value!r}")
            if name == "feature" and value[1:-1] == "native-skia":
                return ("native-skia",)
            return ("atom", name, value)

        if not self._accept("("):
            return ("atom", name)

        arguments = []
        if not self._accept(")"):
            while True:
                arguments.append(self._expression())
                if self._accept(")"):
                    break
                if not self._accept(","):
                    raise ValueError("expected ',' or ')' in cfg predicate")
                if self._accept(")"):
                    break
        return (name, *arguments)


def _evaluate_cfg(expression: tuple, native_skia: bool) -> bool | None:
    """다른 cfg atom은 미정으로 둔 3값 평가 결과를 반환한다."""
    operator, *arguments = expression
    if operator == "native-skia":
        return native_skia
    if operator == "atom":
        return None

    values = [_evaluate_cfg(argument, native_skia) for argument in arguments]
    if operator == "all":
        if False in values:
            return False
        return True if all(value is True for value in values) else None
    if operator == "any":
        if True in values:
            return True
        return False if all(value is False for value in values) else None
    if operator == "not" and len(values) == 1:
        return None if values[0] is None else not values[0]
    return None


def _requires_native_skia_enabled(predicate: str) -> bool:
    """native-skia를 끄면 반드시 거짓이고 켜면 가능성이 생기는 술어인가."""
    try:
        expression = _CfgParser(predicate).parse()
    except ValueError:
        return False
    disabled = _evaluate_cfg(expression, native_skia=False)
    enabled = _evaluate_cfg(expression, native_skia=True)
    return disabled is False and enabled is not False


def source_is_file_gated_native_skia(source: str) -> bool:
    """소스 텍스트가 파일 전체를 native-skia **활성** 조건으로 게이트하는가."""
    return any(
        _requires_native_skia_enabled(predicate)
        for predicate in _inner_cfg_predicates(source)
    )


def file_gated_native_skia_tests() -> list[str]:
    """`tests/*.rs` 중 파일 게이트된 native-skia test 의 stem 목록."""
    return sorted(
        path.stem
        for path in TESTS_DIR.glob("*.rs")
        if source_is_file_gated_native_skia(path.read_text(encoding="utf-8"))
    )


def _function_gated_native_skia_test_names(source: str) -> list[str]:
    """혼합 crate의 native-skia 전용 test 이름을 module 내부까지 찾는다."""
    if source_is_file_gated_native_skia(source):
        return []

    code = _mask_rust_non_code(source)
    functions = [
        (function, _item_body_range(code, function.end()))
        for function in _FUNCTION_DECLARATION.finditer(code)
    ]
    function_bodies = [body for _, body in functions if body is not None]

    native_module_bodies = []
    for module in _MODULE_DECLARATION.finditer(code):
        if any(opened < module.start() < closed for opened, closed in function_bodies):
            continue
        attributes = _outer_attributes_before(source, code, module.start())
        if not any(
            _requires_native_skia_enabled(predicate)
            for predicate in _cfg_predicates_in_attributes(attributes)
        ):
            continue
        body = _item_body_range(code, module.end())
        if body is not None:
            native_module_bodies.append(body)

    names = []
    for function, _ in functions:
        if any(opened < function.start() < closed for opened, closed in function_bodies):
            continue
        attributes = _outer_attributes_before(source, code, function.start())
        is_test = bool(_TEST_ATTRIBUTE.search(attributes))
        cfg_attr_test = _cfg_attr_enables_native_skia_test(attributes)
        if not is_test and not cfg_attr_test:
            continue
        function_gate = any(
            _requires_native_skia_enabled(predicate)
            for predicate in _cfg_predicates_in_attributes(attributes)
        )
        module_gate = any(
            opened < function.start() < closed
            for opened, closed in native_module_bodies
        )
        if function_gate or module_gate or cfg_attr_test:
            names.append(function.group("name"))
    return names


def function_gated_native_skia_tests() -> list[str]:
    """`tests/*.rs`의 혼합 crate 함수 게이트 목록을 `stem::fn`으로 반환한다."""
    found = []
    for path in TESTS_DIR.glob("*.rs"):
        source = path.read_text(encoding="utf-8")
        found.extend(
            f"{path.stem}::{name}"
            for name in _function_gated_native_skia_test_names(source)
        )
    return sorted(found)


def _path_attribute_values(source: str) -> list[str]:
    """주석·문자열을 제외한 `#[path = "..."]` 값을 반환한다."""
    code = _mask_rust_non_code(source)
    found = []
    for opened in _PATH_ATTRIBUTE_OPEN.finditer(code):
        bracket = code.find("[", opened.start())
        close = _matching_delimiter(code, bracket, "[", "]")
        if close is None:
            continue
        attribute = source[opened.start():close + 1]
        matched = re.search(r'=\s*"([^"]+)"', attribute)
        if matched is not None:
            found.append(matched.group(1))
    return found


def file_gated_native_skia_support_files() -> list[str]:
    """파일 게이트 target의 `#[path]` support를 저장소 상대 경로로 반환한다."""
    repo_root = TESTS_DIR.parent.resolve()
    found = set()
    for path in TESTS_DIR.glob("*.rs"):
        source = path.read_text(encoding="utf-8")
        if not source_is_file_gated_native_skia(source):
            continue
        for relative in _path_attribute_values(source):
            resolved = (path.parent / relative).resolve()
            found.add(resolved.relative_to(repo_root).as_posix())
    return sorted(found)


class CiImpactWorkflowTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
        cls.preflight, cls.workers = cls.workflow.split(WORKER_MARKER, maxsplit=1)

    def _step(self, name: str, source: str | None = None) -> str:
        workflow = source or self.workflow
        step = workflow.split(f"      - name: {name}", maxsplit=1)[1]
        boundary = re.search(r"(?m)^(?:      - name:|  [A-Za-z0-9_-]+:)\s*", step)
        return step[: boundary.start()] if boundary else step

    def _job(self, name: str) -> str:
        match = re.search(
            rf"(?ms)^  {re.escape(name)}:\n.*?(?=^  [A-Za-z0-9_-]+:\n|\Z)",
            self.workflow,
        )
        self.assertIsNotNone(match, name)
        return match.group(0) if match else ""

    def _run_aggregate(self, **overrides: str) -> subprocess.CompletedProcess[str]:
        step = self._step("Check Build & Test worker results")
        script = textwrap.dedent(step.split("        run: |\n", maxsplit=1)[1])
        env = {
            **os.environ,
            "PREFLIGHT_RESULT": "success",
            "FAST_PASS": "false",
            "RUST_REQUIRED": "false",
            "NATIVE_SKIA_REQUIRED": "false",
            "FRONTEND_MODE": "unit",
            "IMPACT_REASON": "classified:studio-unit",
            "BUILD_SLOW_RESULT": "skipped",
            "BUILD_A_RESULT": "skipped",
            "BUILD_B_RESULT": "skipped",
            "TEST_SLOW_RESULT": "skipped",
            "TEST_REGULAR_1_RESULT": "skipped",
            "TEST_REGULAR_2_RESULT": "skipped",
            "TEST_REGULAR_3_RESULT": "skipped",
            "LINT_RESULT": "skipped",
            "NATIVE_SKIA_RESULT": "skipped",
            "FRONTEND_UNIT_RESULT": "success",
            "FRONTEND_PACKAGE_RESULT": "skipped",
            **overrides,
        }
        return subprocess.run(
            ["bash", "-e", "-o", "pipefail", "-c", script],
            check=False,
            capture_output=True,
            env=env,
            text=True,
        )

    def test_preflight_exposes_every_axis_with_fail_closed_defaults(self) -> None:
        expected_defaults = {
            "rust_required": "'true'",
            "frontend_mode": "'package'",
            "render_required": "'true'",
            "native_skia_required": "'true'",
            "codeql_languages": "'javascript-typescript,python,rust'",
            "classification_status": "'full'",
            "classifier_version": "'unavailable'",
            "impact_reason": "'fail-closed:impact-unavailable'",
            "impact_authority": "'unavailable'",
        }
        for output, default in expected_defaults.items():
            with self.subTest(output=output):
                self.assertIn(f"      {output}:", self.preflight)
                self.assertIn(default, self.preflight)

    def test_classifier_uses_pr_base_sha_without_checkout_credentials(self) -> None:
        step = self._step("Check out trusted CI impact classifier", self.preflight)
        self.assertIn(
            "ref: ${{ github.event_name == 'pull_request' "
            "&& github.event.pull_request.base.sha || github.sha }}",
            step,
        )
        self.assertIn("persist-credentials: false", step)
        self.assertIn("sparse-checkout: scripts/ci-impact-classifier.cjs", step)
        self.assertIn("sparse-checkout-cone-mode: false", step)
        self.assertIn("id: checkout-impact-classifier", step)
        self.assertIn("Classify CI impact", self.preflight)
        self.assertIn(
            "Stage 4 activates frontend_mode, render_required, rust_required, "
            "and native_skia_required",
            self.preflight,
        )
        self.assertIn("pr-base-trusted", self.preflight)
        self.assertNotIn("pr-base-trusted-shadow", self.preflight)

    def test_missing_classifier_checkout_cannot_claim_trusted_authority(self) -> None:
        self.assertIn(
            "const classifierPath = path.join(\n"
            "              workspace,\n"
            "              'scripts',\n"
            "              'ci-impact-classifier.cjs',",
            self.preflight,
        )
        self.assertIn(
            "const checkoutSucceeded = "
            "process.env.CLASSIFIER_CHECKOUT_OUTCOME === 'success'\n"
            "              && fs.existsSync(classifierPath);",
            self.preflight,
        )
        self.assertIn(
            "const authority = !checkoutSucceeded\n"
            "              ? 'unavailable'",
            self.preflight,
        )

    def test_review_only_fast_pass_does_not_pay_classifier_cost(self) -> None:
        for step_name in (
            "Check out trusted CI impact classifier",
            "Collect CI impact input",
            "Classify CI impact",
        ):
            with self.subTest(step=step_name):
                self.assertIn(
                    "if: ${{ steps.finalize.outputs.fast_pass != 'true' }}",
                    self._step(step_name, self.preflight),
                )

    def test_label_events_do_not_restart_ci_and_manual_dispatch_forces_full(self) -> None:
        self.assertIn(
            "types: [opened, reopened, synchronize]",
            self.workflow,
        )
        self.assertNotIn("labeled, unlabeled", self.workflow)
        collect = self._step("Collect CI impact input", self.preflight)
        self.assertNotIn("label.name === 'ci:full'", collect)
        self.assertIn("context.eventName === 'workflow_dispatch'", collect)
        self.assertIn("? 'manual-or-tag'", collect)

    def test_stage4_consumes_frontend_rust_and_native_axes_but_defers_codeql(self) -> None:
        self.assertIn("needs.preflight.outputs.frontend_mode", self.workers)
        for active_axis in (
            "needs.preflight.outputs.rust_required",
            "needs.preflight.outputs.native_skia_required",
        ):
            with self.subTest(axis=active_axis):
                self.assertIn(active_axis, self.workers)
        self.assertNotIn("needs.preflight.outputs.codeql_languages", self.workers)

    def test_unit_and_package_jobs_are_mutually_exclusive(self) -> None:
        unit = self._job("frontend-unit-gates")
        package = self._job("frontend-package-gates")
        self.assertIn("needs.preflight.outputs.frontend_mode == 'unit'", unit)
        self.assertIn("npx tsc --project tsconfig.ci-unit.json --noEmit", unit)
        self.assertIn("npm --prefix rhwp-studio run test", unit)
        self.assertNotIn("wasm-pack build", unit)
        self.assertIn("needs.preflight.outputs.frontend_mode == 'package'", package)
        self.assertIn("wasm-pack build --target web --dev", package)
        self.assertIn("npm --prefix rhwp-studio run test", package)
        self.assertIn("npm --prefix rhwp-studio run build", package)

    def test_rust_lint_and_archive_builders_require_rust_axis(self) -> None:
        lint = self._job("lint")
        self.assertIn("needs.preflight.outputs.rust_required == 'true'", lint)

        for job_name in (
            "build-test-archive-slow",
            "build-test-archive-a",
            "build-test-archive-b",
        ):
            with self.subTest(job=job_name):
                job = self._job(job_name)
                self.assertIn("needs.preflight.outputs.rust_required == 'true'", job)
                self.assertIn("needs.lint.result == 'success'", job)
                self.assertIn("frontend-unit-gates", job)
                self.assertIn("frontend-package-gates", job)
                self.assertIn("frontend_mode == 'none'", job)
                self.assertIn("frontend_mode == 'unit'", job)
                self.assertIn("frontend_mode == 'package'", job)

    def test_native_skia_accepts_expected_lint_state_for_each_rust_lane(self) -> None:
        native = self._job("native-skia-tests")
        self.assertIn("needs.preflight.outputs.native_skia_required == 'true'", native)
        self.assertIn("needs.preflight.outputs.rust_required == 'true'", native)
        self.assertIn("needs.lint.result == 'success'", native)
        self.assertIn("needs.preflight.outputs.rust_required == 'false'", native)
        self.assertIn("needs.lint.result == 'skipped'", native)
        self.assertIn("frontend-unit-gates", native)
        self.assertIn("frontend-package-gates", native)
        self.assertIn("frontend_mode == 'none'", native)
        self.assertIn("frontend_mode == 'unit'", native)
        self.assertIn("frontend_mode == 'package'", native)
        self.assertNotIn("build-test-archive-", native)
        self.assertNotIn("test-regular-shard", native)
        self.assertNotIn("test-slow-shard", native)

    def test_aggregate_harness_stops_at_the_next_job_boundary(self) -> None:
        step = self._step("Check Build & Test worker results")
        script = textwrap.dedent(step.split("        run: |\n", maxsplit=1)[1])
        self.assertNotIn("wasm-build:", script)
        self.assertNotIn("startsWith(github.ref", script)

    def test_native_skia_integration_targets_are_classifier_inputs(self) -> None:
        # 역방향 감시: job 이 실행하는 target 은 classifier 소유여야 한다.
        native_step = self._step("Native Skia tests")
        classifier = CLASSIFIER_PATH.read_text(encoding="utf-8")
        targets = set(re.findall(r"--test ([A-Za-z0-9_]+)", native_step))
        self.assertTrue(targets)
        for target in targets:
            with self.subTest(target=target):
                self.assertIn(f"'tests/{target}.rs'", classifier)

    def test_discovery_finds_the_known_file_gated_native_skia_tests(self) -> None:
        """발견 패턴이 망가지면 아래 테스트가 조용히 무의미해진다.

        `render_p37_direct_pdf_export` 는 `all(...)` 중첩 게이트라, 정확 일치
        패턴으로는 잡히지 않는다. 이 단언이 그 회귀를 막는다.
        """
        found = file_gated_native_skia_tests()
        for expected in [
            "cli_exit_codes_native",
            "issue_1144_native",
            "issue_2083_hide_fill_page_background",
            "issue_2292_chart_png_clip",
            "issue_2293_chart_png_text",
            "render_p37_direct_pdf_export",
        ]:
            self.assertIn(expected, found)
        # 함수 게이트 파일은 이 부류가 아니다 — 별도 축(#4132)이다.
        self.assertNotIn("issue_2225_missing_picture_placeholder", found)
        self.assertNotIn("cli_exit_codes", found)

    def test_only_documented_mixed_function_gate_remains(self) -> None:
        """[#4132] 새 함수 게이트 test를 파일 규약 밖의 예외로 만들지 않는다."""
        self.assertEqual(
            function_gated_native_skia_tests(),
            [
                "issue_2225_missing_picture_placeholder::"
                "issue_2225_export_png_defaults_to_print_equivalent_skia_profile",
            ],
        )

    def test_file_gated_native_skia_support_files_are_classifier_inputs(self) -> None:
        """[#4132] target이 공유하는 helper 변경도 Native job을 선택해야 한다."""
        support_files = file_gated_native_skia_support_files()
        for expected in [
            "tests/support/cli_exit_code_support.rs",
            "tests/support/issue_1144_support.rs",
        ]:
            self.assertIn(expected, support_files)

        classifier = CLASSIFIER_PATH.read_text(encoding="utf-8")
        missing = [
            path for path in support_files if f"'{path}'" not in classifier
        ]
        self.assertEqual(
            missing,
            [],
            "file-gated native-skia target의 #[path] support가 classifier 소유 목록에 없다",
        )

    def test_function_gate_discovery_rejects_non_tests_and_false_cfgs(self) -> None:
        detected = [
            '#[cfg(feature = "native-skia")]\n#[test]\nfn native_test() {}',
            '#[test]\n#[cfg(all(unix, feature = "native-skia"))]\nfn native_test() {}',
            (
                'mod native_probe {\n'
                '    #[cfg(feature = "native-skia")]\n'
                '    #[test]\n'
                '    fn native_test() {}\n'
                '}'
            ),
            (
                '#[cfg(feature = "native-skia")]\n'
                'mod native_probe {\n'
                '    #[test]\n'
                '    fn native_test() {}\n'
                '}'
            ),
            '#[cfg_attr(feature = "native-skia", test)]\nfn native_test() {}',
        ]
        rejected = [
            '#[cfg(not(feature = "native-skia"))]\n#[test]\nfn opposite() {}',
            '#[cfg(any(feature = "native-skia", unix))]\n#[test]\nfn optional() {}',
            '#[cfg(feature = "native-skia")]\nfn helper() {}',
            'const S: &str = r#"#[cfg(feature = "native-skia")] #[test] fn quoted() {}"#;',
            'fn nested() { #[cfg(feature = "native-skia")] #[test] fn inner() {} }',
        ]
        for source in detected:
            with self.subTest(detected=source):
                self.assertEqual(
                    _function_gated_native_skia_test_names(source),
                    ["native_test"],
                )
        for source in rejected:
            with self.subTest(rejected=source):
                self.assertEqual(_function_gated_native_skia_test_names(source), [])

    def test_function_gate_discovery_handles_attributes_without_backtracking(self) -> None:
        source = (
            "\t#[]\n" * 2_000
            + '#[doc = "a]b"]\n'
            '#[cfg(feature = "native-skia")]\n'
            '#[test]\n'
            'fn bracketed_doc() {}\n'
        )
        self.assertEqual(
            _function_gated_native_skia_test_names(source),
            ["bracketed_doc"],
        )

    def test_path_support_discovery_ignores_commented_attributes(self) -> None:
        source = '''
#![cfg(feature = "native-skia")]
// #[path = "support/commented.rs"]
#[path = "support/real.rs"]
mod support;
'''
        self.assertEqual(_path_attribute_values(source), ["support/real.rs"])

    def test_discovery_rejects_negated_gates_and_quoted_attributes(self) -> None:
        """[PR #4170 리뷰] 발견 패턴의 **반대 방향** 오탐도 막는다.

        위 테스트는 "놓치지 않는가" 만 본다. 넓은 쪽 오탐은 저장소에 해당 파일이
        생기기 전까지 드러나지 않으므로 합성 입력으로 고정한다.

        - `not(feature = "native-skia")` 는 native-skia 빌드에서 오히려 cfg-out
          되므로, 배선을 요구하면 0건짜리 target 이 생긴다.
        - 이 저장소는 한국어 `//!` 문서에 cfg 속성을 자주 인용한다. 인용은
          게이트가 아니다.
        """
        for source in [
            '#![cfg(feature = "native-skia")]',
            '#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]',
            '#![cfg(all(\n    not(target_arch = "wasm32"),\n    feature = "native-skia"\n))]',
            '#![cfg(not(not(feature = "native-skia")))]',
            '#![cfg(any(feature = "native-skia", all(feature = "native-skia", unix)))]',
        ]:
            with self.subTest(gated=source):
                self.assertTrue(source_is_file_gated_native_skia(source))

        for source in [
            '#![cfg(not(feature = "native-skia"))]',
            '#![cfg(all(not(target_arch = "wasm32"), not(feature = "native-skia")))]',
            '#![cfg(any(feature = "native-skia", target_os = "linux"))]',
            '//! `#![cfg(feature = "native-skia")]` 로 파일을 게이트한다',
            '// #![cfg(feature = "native-skia")]',
            '/* #![cfg(feature = "native-skia")] */',
            '/* outer /* #![cfg(feature = "native-skia")] */ comment */',
            'const S: &str = r##"#![cfg(feature = "native-skia")]"##;',
            'const S: &str = "#![cfg(feature = \\"native-skia\\")]";',
            'fn nested() { #![cfg(feature = "native-skia")] }',
            '#![cfg(not(target_arch = "wasm32"))]',
        ]:
            with self.subTest(not_gated=source):
                self.assertFalse(source_is_file_gated_native_skia(source))

    def test_every_file_gated_native_skia_test_is_wired(self) -> None:
        """[#4040] 파일 게이트된 native-skia test 는 job·classifier 양쪽에 있어야 한다.

        기존 `test_native_skia_integration_targets_are_classifier_inputs` 는
        **job 이 실행하는 target** 만 순회하므로, 양쪽 어디에도 없는 파일은 대조
        대상 자체가 아니라 조용히 빠진다. `issue_2083`·`issue_2292`·`issue_2293`
        이 정확히 그 경로로 새어 나갔다 — 파일 전체가 cfg-out 되어 default worker
        에서도 돌지 않고, Native job 도 실행하지 않는 상태였다.

        저장소를 직접 훑어 부류 자체를 강제한다.
        """
        native_step = self._step("Native Skia tests")
        classifier = CLASSIFIER_PATH.read_text(encoding="utf-8")
        targets = set(re.findall(r"--test ([A-Za-z0-9_]+)", native_step))

        missing_from_job = []
        missing_from_classifier = []
        for stem in file_gated_native_skia_tests():
            if stem not in targets:
                missing_from_job.append(stem)
            if f"'tests/{stem}.rs'" not in classifier:
                missing_from_classifier.append(stem)

        self.assertEqual(
            missing_from_job,
            [],
            "Native Skia job 이 실행하지 않는 파일 게이트 test 가 있다. "
            "`--test <name>` 을 release-test·release 두 경로에 추가한다.",
        )
        self.assertEqual(
            missing_from_classifier,
            [],
            "classifier 의 NATIVE_SKIA_RUST_FILES 에 없는 파일 게이트 test 가 있다. "
            "빠지면 그 파일을 고치는 PR 에서 Native Skia job 이 skip 된다.",
        )

    def test_native_skia_targets_run_in_both_profiles(self) -> None:
        """[#4040] release-test 와 release 두 경로가 같은 target 집합을 실행한다."""
        native_step = self._step("Native Skia tests")
        release_test = set(
            re.findall(r"--profile release-test --features native-skia --test ([A-Za-z0-9_]+)", native_step)
        )
        release = set(
            re.findall(r"--release --features native-skia --test ([A-Za-z0-9_]+)", native_step)
        )
        self.assertTrue(release_test)
        self.assertEqual(release_test, release)

    def test_rust_workers_wait_only_for_their_test_archive(self) -> None:
        expected_archives = {
            "test-slow-shard": "build-test-archive-slow",
            "test-regular-shard-1": "build-test-archive-a",
            "test-regular-shard-2": "build-test-archive-slow",
            "test-regular-shard-3": "build-test-archive-b",
        }
        for job_name, archive in expected_archives.items():
            with self.subTest(job=job_name):
                job = self._job(job_name)
                self.assertIn("needs.preflight.outputs.rust_required == 'true'", job)
                self.assertIn(f"needs: [preflight, {archive}]", job)
                self.assertIn(f"needs['{archive}'].result == 'success'", job)
                self.assertNotIn("native-skia-tests", job)
                self.assertNotIn("native_skia_required", job)

    def test_aggregate_validates_expected_success_and_skipped_states(self) -> None:
        aggregate = self._job("build-and-test")
        self.assertIn("- frontend-unit-gates", aggregate)
        self.assertIn("- frontend-package-gates", aggregate)
        self.assertIn("- native-skia-tests", aggregate)
        self.assertIn("RUST_REQUIRED:", aggregate)
        self.assertIn("NATIVE_SKIA_REQUIRED:", aggregate)
        self.assertIn("Rust lane expected success", aggregate)
        self.assertIn("Rust lane expected skipped", aggregate)
        self.assertIn("Native Skia lane expected success", aggregate)
        self.assertIn("Native Skia lane expected skipped", aggregate)
        self.assertIn("Unknown rust_required", aggregate)
        self.assertIn("Unknown native_skia_required", aggregate)
        self.assertIn("Frontend none lane expected skipped/skipped", aggregate)
        self.assertIn("Frontend unit lane expected success/skipped", aggregate)
        self.assertIn("Frontend package lane expected skipped/success", aggregate)
        self.assertIn("Unknown frontend mode", aggregate)

    def test_shard_count_artifacts_are_downloaded_only_for_rust_lane(self) -> None:
        aggregate = self._job("build-and-test")
        for step_name in (
            "Download shard counts",
            "Download archive expected counts",
            "Verify shard totals",
        ):
            with self.subTest(step=step_name):
                self.assertIn(
                    "needs.preflight.outputs.rust_required == 'true'",
                    self._step(step_name, aggregate),
                )

    def test_aggregate_accepts_every_supported_stage4_lane(self) -> None:
        rust_success = {
            "RUST_REQUIRED": "true",
            "LINT_RESULT": "success",
            "BUILD_SLOW_RESULT": "success",
            "BUILD_A_RESULT": "success",
            "BUILD_B_RESULT": "success",
            "TEST_SLOW_RESULT": "success",
            "TEST_REGULAR_1_RESULT": "success",
            "TEST_REGULAR_2_RESULT": "success",
            "TEST_REGULAR_3_RESULT": "success",
        }
        cases = {
            "frontend-only": {},
            "rust-non-render": {
                **rust_success,
                "FRONTEND_MODE": "none",
                "FRONTEND_UNIT_RESULT": "skipped",
            },
            "rust-render": {
                **rust_success,
                "NATIVE_SKIA_REQUIRED": "true",
                "NATIVE_SKIA_RESULT": "success",
                "FRONTEND_MODE": "none",
                "FRONTEND_UNIT_RESULT": "skipped",
            },
            "non-rust-native-input": {
                "NATIVE_SKIA_REQUIRED": "true",
                "NATIVE_SKIA_RESULT": "success",
                "FRONTEND_MODE": "package",
                "FRONTEND_UNIT_RESULT": "skipped",
                "FRONTEND_PACKAGE_RESULT": "success",
            },
        }
        for name, env in cases.items():
            with self.subTest(lane=name):
                result = self._run_aggregate(**env)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_aggregate_rejects_axis_result_mismatches(self) -> None:
        cases = {
            "unexpected-rust-worker": {"LINT_RESULT": "success"},
            "missing-native-worker": {
                "NATIVE_SKIA_REQUIRED": "true",
                "NATIVE_SKIA_RESULT": "skipped",
            },
            "unexpected-native-worker": {"NATIVE_SKIA_RESULT": "success"},
            "frontend-mismatch": {"FRONTEND_UNIT_RESULT": "skipped"},
            "unknown-rust-axis": {"RUST_REQUIRED": "maybe"},
            "unknown-native-axis": {"NATIVE_SKIA_REQUIRED": "maybe"},
        }
        for name, env in cases.items():
            with self.subTest(lane=name):
                result = self._run_aggregate(**env)
                self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_aggregate_fast_pass_still_accepts_skipped_heavy_jobs(self) -> None:
        result = self._run_aggregate(
            FAST_PASS="true",
            RUST_REQUIRED="true",
            NATIVE_SKIA_REQUIRED="true",
            FRONTEND_MODE="package",
            FRONTEND_UNIT_RESULT="skipped",
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_classifier_failures_remain_fail_closed_without_failing_preflight(self) -> None:
        for step_name in (
            "Check out trusted CI impact classifier",
            "Collect CI impact input",
            "Classify CI impact",
            "Summarize CI impact classification",
        ):
            with self.subTest(step=step_name):
                self.assertIn("continue-on-error: true", self._step(step_name, self.preflight))


if __name__ == "__main__":
    unittest.main()
