"""ABSTAIN ON CONTRADICTION.

If envelope fields conflict, the verifier must output abstain.
It must not invent pass or fail.

This is not V-proto's classifier. V-proto maps exit 0/1/2/3/4 plus
judgment fields onto pass / io_fail / usage_fail / judgment_fail /
page_verify_fail / inconsistent. Here the closed set is only

    pass | fail | abstain

and abstain wins the moment success-leaning and fail-leaning evidence
are both present, or a named same-node / same-block contradiction fires.

Examples the issue names:

* identical:true AND hasSignal:true
* reproduced:true AND exit 3
* pageCount match AND STRUCT_MISMATCH on the same node
"""

from __future__ import annotations

from typing import Mapping

from .schema import (
    CLAIM_ID,
    Decision,
    EnvelopeFields,
    VERDICT_ABSTAIN,
    VERDICT_FAIL,
    VERDICT_PASS,
)

SCHEMA_VERSION = "1.0"

# Named rules the corpus and fixtures pin. First match is contradiction_id.
NAMED_RULES: tuple[tuple[str, str], ...] = (
    ("identical_and_has_signal", "identical=true+hasSignal=true"),
    ("reproduced_and_exit3", "reproduced=true+exit=3"),
    ("reproduced_and_exit4", "reproduced=true+exit=4"),
    ("pagecount_match_and_struct_same_node", "pageCountMatch+STRUCT_MISMATCH@same-node"),
    ("pagecount_equal_and_mismatch_flag", "pageCountA==pageCountB+pageCountMismatch=true"),
    ("pagecount_unequal_and_match_flag", "pageCountA!=pageCountB+pageCountMismatch=false"),
    ("verify_identical_and_diffcount", "verify.identical=true+verify.diffCount>0"),
    ("verify_false_and_zero_diff", "verify.identical=false+verify.diffCount=0"),
    ("identical_and_diffcount", "identical=true+diffCount>0"),
    ("identical_and_failcount", "identical=true+failCount>0"),
    ("identical_and_verify_false", "identical=true+verify.identical=false"),
    ("verdict_pass_and_failcount", "verdict=pass+failCount>0"),
    ("verdict_pass_and_identical_false", "verdict=pass+identical=false"),
    ("verdict_fail_and_clean_success", "verdict=fail+identical=true+failCount=0"),
    ("exit0_and_identical_false", "exit=0+identical=false"),
    ("exit0_and_reproduced_false", "exit=0+reproduced=false"),
    ("exit0_and_verdict_fail", "exit=0+verdict=fail"),
    ("exit0_and_pagecount_mismatch", "exit=0+pageCountMismatch=true"),
    ("exit0_and_struct_mismatch", "exit=0+STRUCT_MISMATCH"),
    ("exit3_and_verdict_pass", "exit=3+verdict=pass"),
    ("exit3_and_success_fields", "exit=3+success-fields"),
    ("exit4_and_pagecount_ok", "exit=4+pageCountMismatch=false"),
    ("has_signal_false_and_counts", "hasSignal=false+counts>0"),
    ("has_signal_true_and_zero_counts", "hasSignal=true+counts=0"),
    ("clean_true_and_findings", "clean=true+findingCount>0"),
    ("valid_true_and_failcount", "valid=true+failCount>0"),
    ("regression_true_and_status_ok", "regression=true+status=OK"),
    ("regression_false_and_status_fail", "regression=false+status=FAIL"),
    ("identical_and_regression", "identical=true+regression=true"),
    ("identical_and_clean_false", "identical=true+clean=false"),
    ("identical_and_valid_false", "identical=true+valid=false"),
    ("reproduced_and_identical_false", "reproduced=true+identical=false"),
    ("status_ok_and_failcount", "status=OK+failCount>0"),
    ("passcount_and_verdict_fail", "passCount>0+failCount=0+verdict=fail"),
)

NAMED_RULE_IDS: tuple[str, ...] = tuple(rule_id for rule_id, _ in NAMED_RULES)

_FAIL_STRUCT = frozenset({"STRUCT_MISMATCH", "PAGE_MISMATCH", "OVER", "LOAD_FAIL", "FAIL"})
_FAIL_VERDICT = frozenset({"fail", "FAIL", "invalid", "mismatch"})
_PASS_VERDICT = frozenset({"pass", "PASS"})
_OK_STATUS = frozenset({"OK", "PASS"})
_FAIL_STATUS = frozenset({"FAIL", "STRUCT_MISMATCH", "PAGE_MISMATCH", "OVER", "LOAD_FAIL"})


def _norm_status(value: str | None) -> str | None:
    if value is None:
        return None
    return value.strip()


def success_tokens(fields: EnvelopeFields) -> tuple[str, ...]:
    out: list[str] = []
    if fields.identical is True:
        out.append("identical=true")
    if fields.has_signal is False:
        out.append("hasSignal=false")
    if fields.reproduced is True:
        out.append("reproduced=true")
    if fields.page_count_mismatch is False:
        out.append("pageCountMismatch=false")
    if fields.pages_equal() is True:
        out.append("pageCountA==pageCountB")
    if _norm_status(fields.struct_status) == "PASS":
        out.append("struct=PASS")
    if fields.verify_identical is True:
        out.append("verify.identical=true")
    if fields.verify_diff_count == 0:
        out.append("verify.diffCount=0")
    if fields.diff_count == 0:
        out.append("diffCount=0")
    if fields.fail_count == 0:
        out.append("failCount=0")
    if fields.verdict is not None and fields.verdict in _PASS_VERDICT:
        out.append("verdict=pass")
    if fields.regression is False:
        out.append("regression=false")
    if _norm_status(fields.status) in _OK_STATUS:
        out.append("status=OK")
    if fields.clean is True:
        out.append("clean=true")
    if fields.valid is True:
        out.append("valid=true")
    if fields.signal_count == 0:
        out.append("signalCount=0")
    if fields.finding_count == 0:
        out.append("findingCount=0")
    if fields.overflow_count == 0:
        out.append("overflowCount=0")
    if fields.overlap_count == 0:
        out.append("overlapCount=0")
    if fields.exit == 0:
        out.append("exit=0")
    return tuple(out)


def fail_tokens(fields: EnvelopeFields) -> tuple[str, ...]:
    out: list[str] = []
    if fields.identical is False:
        out.append("identical=false")
    if fields.has_signal is True:
        out.append("hasSignal=true")
    if fields.reproduced is False:
        out.append("reproduced=false")
    if fields.page_count_mismatch is True:
        out.append("pageCountMismatch=true")
    if fields.pages_equal() is False:
        out.append("pageCountA!=pageCountB")
    status = _norm_status(fields.struct_status)
    if status in _FAIL_STRUCT:
        out.append(f"struct={status}")
    if fields.verify_identical is False:
        out.append("verify.identical=false")
    if fields.verify_diff_count is not None and fields.verify_diff_count > 0:
        out.append("verify.diffCount>0")
    if fields.diff_count is not None and fields.diff_count > 0:
        out.append("diffCount>0")
    if fields.fail_count is not None and fields.fail_count > 0:
        out.append("failCount>0")
    if fields.verdict is not None and fields.verdict in _FAIL_VERDICT:
        out.append("verdict=fail")
    if fields.regression is True:
        out.append("regression=true")
    if _norm_status(fields.status) in _FAIL_STATUS:
        out.append(f"status={_norm_status(fields.status)}")
    if fields.clean is False:
        out.append("clean=false")
    if fields.valid is False:
        out.append("valid=false")
    if fields.signal_count is not None and fields.signal_count > 0:
        out.append("signalCount>0")
    if fields.finding_count is not None and fields.finding_count > 0:
        out.append("findingCount>0")
    if fields.overflow_count is not None and fields.overflow_count > 0:
        out.append("overflowCount>0")
    if fields.overlap_count is not None and fields.overlap_count > 0:
        out.append("overlapCount>0")
    if fields.exit in (1, 2, 3, 4):
        out.append(f"exit={fields.exit}")
    return tuple(out)


def named_contradictions(
    fields: EnvelopeFields,
    success: tuple[str, ...],
    fail: tuple[str, ...],
) -> tuple[str, ...]:
    """Pin the issue examples and other same-envelope conflicts."""
    del success, fail
    found: list[str] = []

    if fields.identical is True and fields.has_signal is True:
        found.append("identical_and_has_signal")
    if fields.reproduced is True and fields.exit == 3:
        found.append("reproduced_and_exit3")
    if fields.reproduced is True and fields.exit == 4:
        found.append("reproduced_and_exit4")
    if (
        (fields.pages_equal() is True or fields.page_count_mismatch is False)
        and _norm_status(fields.struct_status) == "STRUCT_MISMATCH"
        and fields.same_struct_node()
    ):
        found.append("pagecount_match_and_struct_same_node")
    if fields.pages_equal() is True and fields.page_count_mismatch is True:
        found.append("pagecount_equal_and_mismatch_flag")
    if fields.pages_equal() is False and fields.page_count_mismatch is False:
        found.append("pagecount_unequal_and_match_flag")
    if fields.verify_identical is True and (
        fields.verify_diff_count is not None and fields.verify_diff_count > 0
    ):
        found.append("verify_identical_and_diffcount")
    if fields.verify_identical is False and fields.verify_diff_count == 0:
        found.append("verify_false_and_zero_diff")
    if fields.identical is True and fields.diff_count is not None and fields.diff_count > 0:
        found.append("identical_and_diffcount")
    if fields.identical is True and fields.fail_count is not None and fields.fail_count > 0:
        found.append("identical_and_failcount")
    if fields.identical is True and fields.verify_identical is False:
        found.append("identical_and_verify_false")
    if (
        fields.verdict is not None
        and fields.verdict in _PASS_VERDICT
        and fields.fail_count is not None
        and fields.fail_count > 0
    ):
        found.append("verdict_pass_and_failcount")
    if (
        fields.verdict is not None
        and fields.verdict in _PASS_VERDICT
        and fields.identical is False
    ):
        found.append("verdict_pass_and_identical_false")
    if (
        fields.verdict is not None
        and fields.verdict in _FAIL_VERDICT
        and fields.identical is True
        and fields.fail_count == 0
    ):
        found.append("verdict_fail_and_clean_success")
    if fields.exit == 0 and fields.identical is False:
        found.append("exit0_and_identical_false")
    if fields.exit == 0 and fields.reproduced is False:
        found.append("exit0_and_reproduced_false")
    if fields.exit == 0 and fields.verdict is not None and fields.verdict in _FAIL_VERDICT:
        found.append("exit0_and_verdict_fail")
    if fields.exit == 0 and fields.page_count_mismatch is True:
        found.append("exit0_and_pagecount_mismatch")
    if fields.exit == 0 and _norm_status(fields.struct_status) == "STRUCT_MISMATCH":
        found.append("exit0_and_struct_mismatch")
    if fields.exit == 3 and fields.verdict is not None and fields.verdict in _PASS_VERDICT:
        found.append("exit3_and_verdict_pass")
    if (
        fields.exit == 3
        and fields.identical is True
        and fields.has_signal is not True
        and (fields.fail_count is None or fields.fail_count == 0)
        and fields.reproduced is not False
    ):
        found.append("exit3_and_success_fields")
    if (
        fields.exit == 4
        and fields.page_count_mismatch is False
        and fields.identical is not False
    ):
        found.append("exit4_and_pagecount_ok")
    if fields.has_signal is False and (
        (fields.signal_count or 0) > 0
        or (fields.overflow_count or 0) > 0
        or (fields.overlap_count or 0) > 0
    ):
        found.append("has_signal_false_and_counts")
    if (
        fields.has_signal is True
        and fields.signal_count == 0
        and fields.overflow_count == 0
        and fields.overlap_count == 0
        and fields.finding_count == 0
    ):
        found.append("has_signal_true_and_zero_counts")
    if fields.clean is True and fields.finding_count is not None and fields.finding_count > 0:
        found.append("clean_true_and_findings")
    if fields.valid is True and fields.fail_count is not None and fields.fail_count > 0:
        found.append("valid_true_and_failcount")
    if fields.regression is True and _norm_status(fields.status) in _OK_STATUS:
        found.append("regression_true_and_status_ok")
    if fields.regression is False and _norm_status(fields.status) in {"FAIL"}:
        found.append("regression_false_and_status_fail")
    if fields.identical is True and fields.regression is True:
        found.append("identical_and_regression")
    if fields.identical is True and fields.clean is False:
        found.append("identical_and_clean_false")
    if fields.identical is True and fields.valid is False:
        found.append("identical_and_valid_false")
    if fields.reproduced is True and fields.identical is False:
        found.append("reproduced_and_identical_false")
    if _norm_status(fields.status) in _OK_STATUS and (
        fields.fail_count is not None and fields.fail_count > 0
    ):
        found.append("status_ok_and_failcount")
    if (
        fields.pass_count is not None
        and fields.pass_count > 0
        and fields.fail_count == 0
        and fields.verdict is not None
        and fields.verdict in _FAIL_VERDICT
    ):
        found.append("passcount_and_verdict_fail")
    return tuple(found)


def decide(fields: EnvelopeFields) -> Decision:
    success = success_tokens(fields)
    fail = fail_tokens(fields)
    named = named_contradictions(fields, success, fail)
    if named:
        return Decision(
            verdict=VERDICT_ABSTAIN,
            contradiction_id=named[0],
            contradictions=named,
            success_tokens=success,
            fail_tokens=fail,
        )
    # No named conflict: do not invent a third polarity. Fail-leaning
    # evidence that can legally sit next to page-count match (other node)
    # is fail. Otherwise pass.
    if fail:
        return Decision(
            verdict=VERDICT_FAIL,
            contradiction_id="",
            contradictions=(),
            success_tokens=success,
            fail_tokens=fail,
        )
    return Decision(
        verdict=VERDICT_PASS,
        contradiction_id="",
        contradictions=(),
        success_tokens=success,
        fail_tokens=fail,
    )


def decide_row(row: Mapping[str, object]) -> Decision:
    return decide(EnvelopeFields.from_mapping(row))


def decide_mapping(
    command: str,
    exit_code: int,
    **fields: object,
) -> Decision:
    payload = {"command": command, "exit": exit_code, **fields}
    return decide(EnvelopeFields.from_mapping(payload))


CLAIM = CLAIM_ID
