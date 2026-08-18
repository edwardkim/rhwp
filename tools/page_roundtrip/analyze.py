#!/usr/bin/env python3
"""쪽수 드리프트 귀속. pages(before) != pages(after) 를 원인 축으로 나눈다.

#4882 축: 각주 subList 전 줄 vertpos=0 이 재파싱에서 쌓임 (215→223).
#4056 / #5128 은 이 모듈이 고치지 않는다 — 카탈로그 축으로만 표시한다.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Iterable, Sequence

from note_probe import NoteRecord

ISSUE_4882 = 4882
ISSUE_4056 = 4056
ISSUE_5128 = 5128

# 다른 좌석. 이 모듈은 판정만 하고 수정하지 않는다.
FOREIGN_SEATS = {
    ISSUE_4056: "다중 secd / planet #5253 — M05-6 범위 밖",
    ISSUE_5128: "한글문서파일형식 69→68 — 다른 좌석",
}

PINNED_4882_PATHS = (
    "section[0] paragraph[421]/ctrl[0]fn.p[0]",
    "section[0] paragraph[728]/ctrl[0]tbl.cell[3].p[0]/ctrl[0]fn.p[0]",
    "section[0] paragraph[1372]/ctrl[0]fn.p[0]",
    "section[0] paragraph[1832]/ctrl[0]tbl.cell[0].p[3]/ctrl[0]fn.p[0]",
    "section[0] paragraph[1865]/ctrl[0]fn.p[0]",
)

PINNED_4882_IR = (
    "[1].vertpos: expected=0 actual=1172",
    "[2].vertpos: expected=0 actual=2344",
    "[2].vertpos: expected=0 actual=2344",
    "[1].vertpos: expected=0 actual=1172",
    "[1].vertpos: expected=0 actual=1172",
)


@dataclass
class Axis:
    name: str
    issue: int | None
    in_scope: bool
    evidence: list[str] = field(default_factory=list)
    weight: int = 0

    def to_json(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "issue": self.issue,
            "inScope": self.in_scope,
            "weight": self.weight,
            "evidence": list(self.evidence),
        }


@dataclass
class DriftReport:
    doc: str
    before: int | None
    after: int | None
    axes: list[Axis] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)

    @property
    def delta(self) -> int | None:
        if self.before is None or self.after is None:
            return None
        return self.after - self.before

    @property
    def primary(self) -> str:
        live = [a for a in self.axes if a.weight > 0]
        if not live:
            return "unknown"
        live.sort(key=lambda a: a.weight, reverse=True)
        return live[0].name

    def to_json(self) -> dict[str, Any]:
        return {
            "schemaVersion": 1,
            "kind": "pageRoundtripDrift",
            "doc": self.doc,
            "pagesBefore": self.before,
            "pagesAfter": self.after,
            "delta": self.delta,
            "primary": self.primary,
            "axes": [a.to_json() for a in self.axes],
            "notes": list(self.notes),
        }


def axis_note_zero_vpos(notes: Sequence[NoteRecord]) -> Axis:
    hits = [n for n in notes if n.has_hwp5_zero_pattern]
    ev = []
    for n in hits[:12]:
        for p in n.paragraphs:
            if p.all_zero_vpos:
                ev.append(f"{n.path_hint} p[{p.para_index}] vpos={p.vpos}")
    return Axis(
        name="hwp5_note_zero_vpos",
        issue=ISSUE_4882,
        in_scope=True,
        evidence=ev,
        weight=len(hits) * 2,
    )


def axis_hangul_artifact(notes: Sequence[NoteRecord]) -> Axis:
    hits = [n for n in notes if any(p.trailing_zero_after_nonzero for p in n.paragraphs)]
    ev = [n.path_hint for n in hits[:8]]
    return Axis(
        name="hangul_hwpx_note_artifact",
        issue=1692,
        in_scope=False,
        evidence=ev,
        weight=len(hits),
    )


def axis_foreign_seat(issue: int | None) -> Axis | None:
    if issue not in FOREIGN_SEATS:
        return None
    return Axis(
        name=f"foreign_seat_{issue}",
        issue=issue,
        in_scope=False,
        evidence=[FOREIGN_SEATS[issue]],
        weight=10,
    )


def axis_page_delta(before: int | None, after: int | None) -> Axis:
    if before is None or after is None:
        return Axis("pages_unknown", None, True, ["쪽수를 읽지 못했다"], 0)
    delta = after - before
    ev = [f"before={before} after={after} delta={delta:+d}"]
    weight = abs(delta)
    name = "pages_match" if delta == 0 else "pages_mismatch"
    return Axis(name=name, issue=ISSUE_4882 if delta == 8 else None, in_scope=True, evidence=ev, weight=weight)


def classify_ir_diff(text: str) -> str:
    lower = text.lower()
    if "fn.p" in lower and "vertpos" in lower:
        return "hwp5_note_zero_vpos"
    if "en.p" in lower and "vertpos" in lower:
        return "hangul_hwpx_note_artifact"
    if "char_shape" in lower or "charshape" in lower:
        return "char_shapes_out_of_scope"
    if "ole" in lower or "shape" in lower:
        return "ole_shape_out_of_scope"
    if "secd" in lower or "secpr" in lower:
        return "foreign_seat_4056"
    return "other_ir"


def attach_ir_diffs(report: DriftReport, diffs: Iterable[str]) -> None:
    buckets: dict[str, list[str]] = {}
    for raw in diffs:
        text = raw.strip()
        if not text:
            continue
        key = classify_ir_diff(text)
        buckets.setdefault(key, []).append(text)
    for name, items in buckets.items():
        in_scope = name == "hwp5_note_zero_vpos"
        issue = ISSUE_4882 if in_scope else None
        if name.startswith("foreign_seat_"):
            try:
                issue = int(name.rsplit("_", 1)[1])
            except ValueError:
                issue = None
        report.axes.append(
            Axis(
                name=f"ir:{name}",
                issue=issue,
                in_scope=in_scope,
                evidence=items[:20],
                weight=len(items) * 3,
            )
        )


def analyze(
    *,
    doc: str,
    before: int | None,
    after: int | None,
    notes: Sequence[NoteRecord] | None = None,
    ir_diffs: Sequence[str] | None = None,
    issue: int | None = None,
) -> DriftReport:
    report = DriftReport(doc=doc, before=before, after=after)
    report.axes.append(axis_page_delta(before, after))
    if notes:
        report.axes.append(axis_note_zero_vpos(notes))
        report.axes.append(axis_hangul_artifact(notes))
    if ir_diffs:
        attach_ir_diffs(report, ir_diffs)
    foreign = axis_foreign_seat(issue)
    if foreign:
        report.axes.append(foreign)
        report.notes.append(f"이 문서는 다른 좌석 이슈 #{issue} — 고치지 않는다")
    if before == 215 and after == 223:
        report.notes.append("기계 판정 #4882 원본 봉투: 215→223")
    if before is not None and after is not None and before == after:
        report.notes.append("쪽수 등식 성립")
    return report


def expected_fail_reason(issue: int | None, before: int | None, after: int | None) -> str:
    if issue == ISSUE_4882:
        return "HWPX 내보내기 전후 쪽수 불일치 (215→223) — 각주 subList vertpos=0 합성"
    if issue == ISSUE_4056:
        return "HWPX 내보내기 전후 쪽수 불일치 (4→1) — 다중 secd, 이 PR 범위 밖"
    if issue == ISSUE_5128:
        return "HWPX 내보내기 전후 쪽수 불일치 (69→68) — 다른 좌석"
    if before is None or after is None:
        return "쪽수 미측정"
    return f"HWPX 내보내기 전후 쪽수 불일치 ({before}→{after})"
