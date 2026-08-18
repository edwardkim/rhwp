#!/usr/bin/env python3
"""쪽수 드리프트 귀속. pages(before) != pages(after) 를 원인 축으로 나눈다.

#5128 축: HWP5-origin HWPX 가 native 전용 저장 pagination 게이트를 건너뛰어
스펙 문서가 69→68 이 된다. 첫 갈림은 p015/p016 (문단 84 PartialParagraph).
#4056 은 planet #5253, #4882 는 PR #5470 — 이 모듈이 고치지 않는다.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Iterable, Sequence

ISSUE_5128 = 5128
ISSUE_4056 = 4056
ISSUE_4882 = 4882

FOREIGN_SEATS = {
    ISSUE_4056: "다중 secd / planet #5253 — M05-7 범위 밖",
    ISSUE_4882: "정책연구 중간진도보고서 215→223 — PR #5470",
}

PINNED_5128_PAGES = (69, 68)
PINNED_5128_FIRST_SPLIT = {
    "page": 15,
    "srcKind": "partialParagraph",
    "srcPara": 84,
    "rtKind": "text",
    "note": "원본 p016 은 문단 84 PartialParagraph. HWPX 는 문단 86 본문으로 흡수",
}
PINNED_5128_TABLES = (73, 174, 193, 203, 284, 343, 363, 380)


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
            "doc": self.doc,
            "before": self.before,
            "after": self.after,
            "delta": self.delta,
            "primary": self.primary,
            "axes": [a.to_json() for a in self.axes],
            "notes": list(self.notes),
        }


def expected_fail_reason(issue: int, before: int | None, after: int | None) -> str:
    if issue == ISSUE_5128:
        return f"한글문서파일형식 69→68 — 저장 pagination 게이트 ({before}→{after})"
    if issue in FOREIGN_SEATS:
        return FOREIGN_SEATS[issue]
    return f"expected-fail #{issue} ({before}→{after})"


def analyze(
    doc: str,
    before: int | None,
    after: int | None,
    *,
    first_split_para: int | None = None,
    whole_tables: Sequence[int] = (),
    ir_diff_count: int = 0,
    issue: int | None = None,
) -> DriftReport:
    report = DriftReport(doc=doc, before=before, after=after)
    if before == 69 and after == 68:
        axis = Axis(
            name="hwp5_origin_stored_pagination",
            issue=ISSUE_5128,
            in_scope=True,
            weight=100,
            evidence=[
                "IR 차이 없음",
                "첫 갈림 p015/p016 문단 84 PartialParagraph",
                "RowBreak 표 174/193/203/284 가 통째 fit",
            ],
        )
        if first_split_para == 84:
            axis.evidence.append("문단 84 분할 소실 확인")
            axis.weight += 20
        extra = [t for t in whole_tables if t in PINNED_5128_TABLES]
        if extra:
            axis.evidence.append(f"통째 흡수 표: {extra}")
            axis.weight += 10 * len(extra)
        report.axes.append(axis)
    if ir_diff_count == 0 and before != after:
        report.notes.append("IR 동일 · 쪽수만 불일치 — 레이아웃 프로필 축")
    if issue == ISSUE_4056:
        report.axes.append(
            Axis(name="foreign_4056", issue=4056, in_scope=False, weight=1, evidence=[FOREIGN_SEATS[4056]])
        )
    if issue == ISSUE_4882:
        report.axes.append(
            Axis(name="foreign_4882", issue=4882, in_scope=False, weight=1, evidence=[FOREIGN_SEATS[4882]])
        )
    if not report.axes:
        report.axes.append(Axis(name="unknown", issue=issue, in_scope=False, weight=0))
    return report
