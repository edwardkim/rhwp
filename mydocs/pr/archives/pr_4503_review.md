---
kind: pr-review
status: completed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4503 검토 기록

## 결론

- 메인터너 보정 `2d2f42524`를 포함하면 수용 가능하다. HWPX `<hp:parameters>`를 구조로 읽어
  HWP5 왕복 뒤 field command가 하나로 축소되는 것을 막는다.
- 최신 contributor head `1f8cdb8d3`는 `devel` 대상, non-draft, `MERGEABLE`, required check 성공이었다.

## 누적 검토와 검증

- source의 구현·되돌림·stage 기록 세 commit을 `-x` 체리픽했다. 검증되지 않은 HWP5 `CTRL_DATA`
  item ID `0x4010`을 되돌린 contributor 판단을 유지했다.
- #4462와의 교차에서 새 `shape_children` caption 분기가 이전 parser 이름을 호출했다. 메인터너 보정은
  검증된 `parse_caption` 공용 parser를 호출하게 하는 최소 정합화다.
- HWPX structured parameter 보존과 HWP5 roundtrip focused test 6건, 누적 `release-test` 5,645건이
  통과했다.

## 범위

- HWPX 구조 보존을 보완하며, 실증되지 않은 HWP5 확장 바이너리 슬롯을 발명하지 않는다.
