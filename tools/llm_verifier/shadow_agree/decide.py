"""Joint pass only when two *different* mechanical commands both pass.

Axis (issue #5510):

    (check_a, check_b, a_pass, b_pass) -> expected_joint / expected_verdict_class

This is not V-abstain (two fields fighting inside one envelope) and not
V-repeat (the same command run K times). One side passing is a disagree.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Mapping

try:
    from .checks import MechanicalCheck, check_by_id, same_command
except ImportError:
    from checks import MechanicalCheck, check_by_id, same_command

CLAIM_ID = "V-shadow"
SCHEMA_VERSION = "1.0"
KIND = "shadowAgreement"

JOINT_PASS = "JOINT_PASS"
SHADOW_A_ONLY = "SHADOW_A_ONLY"
SHADOW_B_ONLY = "SHADOW_B_ONLY"
JOINT_BOTH_FAIL = "JOINT_BOTH_FAIL"
SAME_CHECK_NOT_SHADOW = "SAME_CHECK_NOT_SHADOW"

VERDICT_CLASSES: tuple[str, ...] = (
    JOINT_PASS,
    SHADOW_A_ONLY,
    SHADOW_B_ONLY,
    JOINT_BOTH_FAIL,
    SAME_CHECK_NOT_SHADOW,
)

HONEST_CLAIMS: dict[str, str] = {
    JOINT_PASS: (
        "서로 다른 기계 명령 두 개가 동시에 합격했다. "
        "한쪽만의 합격이 아니므로 shadow agreement 로 합격한다."
    ),
    SHADOW_A_ONLY: (
        "검사 A 만 합격했다. 한 명령의 합격은 합의가 아니므로 불합격이다."
    ),
    SHADOW_B_ONLY: (
        "검사 B 만 합격했다. 한 명령의 합격은 합의가 아니므로 불합격이다."
    ),
    JOINT_BOTH_FAIL: (
        "서로 다른 명령 두 개가 모두 불합격이다. 합의 합격이 아니다."
    ),
    SAME_CHECK_NOT_SHADOW: (
        "같은 기계 명령을 두 칸에 넣었다. 이것은 그림자 합의가 아니다 "
        "(V-repeat 의 같은 산출 반복도, V-abstain 의 한 봉투 모순도 아니다)."
    ),
}


@dataclass(frozen=True)
class DecisionInputs:
    check_a: str
    check_b: str
    a_pass: bool
    b_pass: bool

    @classmethod
    def from_mapping(cls, row: Mapping[str, Any]) -> "DecisionInputs":
        from .schema import parse_bool

        return cls(
            check_a=str(row["check_a"]),
            check_b=str(row["check_b"]),
            a_pass=parse_bool(row["a_pass"]),
            b_pass=parse_bool(row["b_pass"]),
        )


@dataclass(frozen=True)
class Decision:
    verdict_class: str
    expected_joint: bool
    honest_claim: str
    check_a: MechanicalCheck
    check_b: MechanicalCheck
    a_pass: bool
    b_pass: bool
    distinct_commands: bool
    tree_path: tuple[str, ...]
    notes: tuple[str, ...] = field(default_factory=tuple)

    def to_json(self) -> dict[str, Any]:
        return {
            "schemaVersion": SCHEMA_VERSION,
            "kind": KIND,
            "claim": CLAIM_ID,
            "verdictClass": self.verdict_class,
            "expectedJoint": self.expected_joint,
            "honestClaim": self.honest_claim,
            "distinctCommands": self.distinct_commands,
            "checkA": {
                "id": self.check_a.check_id,
                "command": self.check_a.command,
                "commandKey": self.check_a.command_key,
                "passField": self.check_a.pass_field,
                "pass": self.a_pass,
            },
            "checkB": {
                "id": self.check_b.check_id,
                "command": self.check_b.command,
                "commandKey": self.check_b.command_key,
                "passField": self.check_b.pass_field,
                "pass": self.b_pass,
            },
            "treePath": list(self.tree_path),
            "notes": list(self.notes),
            "notAbstain": True,
            "notRepeat": True,
        }


def decide(check_a: str, check_b: str, a_pass: bool, b_pass: bool) -> Decision:
    left = check_by_id(check_a)
    right = check_by_id(check_b)
    path: list[str] = [f"check_a={left.check_id}", f"check_b={right.check_id}"]
    if same_command(check_a, check_b):
        path.append("same_command=true")
        return Decision(
            verdict_class=SAME_CHECK_NOT_SHADOW,
            expected_joint=False,
            honest_claim=HONEST_CLAIMS[SAME_CHECK_NOT_SHADOW],
            check_a=left,
            check_b=right,
            a_pass=a_pass,
            b_pass=b_pass,
            distinct_commands=False,
            tree_path=tuple(path),
            notes=("same command is not a two-command agreement",),
        )
    path.append("same_command=false")
    path.append(f"a_pass={int(a_pass)}")
    path.append(f"b_pass={int(b_pass)}")
    if a_pass and b_pass:
        verdict = JOINT_PASS
        joint = True
    elif a_pass and not b_pass:
        verdict = SHADOW_A_ONLY
        joint = False
    elif b_pass and not a_pass:
        verdict = SHADOW_B_ONLY
        joint = False
    else:
        verdict = JOINT_BOTH_FAIL
        joint = False
    return Decision(
        verdict_class=verdict,
        expected_joint=joint,
        honest_claim=HONEST_CLAIMS[verdict],
        check_a=left,
        check_b=right,
        a_pass=a_pass,
        b_pass=b_pass,
        distinct_commands=True,
        tree_path=tuple(path),
    )


def decide_inputs(inputs: DecisionInputs) -> Decision:
    return decide(inputs.check_a, inputs.check_b, inputs.a_pass, inputs.b_pass)


def decide_row(row: Mapping[str, Any]) -> Decision:
    return decide_inputs(DecisionInputs.from_mapping(row))
