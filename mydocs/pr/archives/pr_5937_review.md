---
kind: pr-review
status: accepted-pending-integration-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5937 검토 - HML 출처 PAGE_BORDER_FILL 정규화 (#5933)

## 접수와 범위

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5937](https://github.com/edwardkim/rhwp/pull/5937) / [@planet6897](https://github.com/planet6897) |
| base / source head | `devel` / `b1d07c8421ba0f7a3a3198c76bcdd02599caf254` |
| 규모 | 3 files, +123 / -1, 1 commit |
| 접수 상태 | non-draft, reviewer `@jangster77` 지정, 작성 시점 `MERGEABLE/CLEAN` |

HML 출처 HWP 저장 시 구역당 `PAGE_BORDER_FILL`을 한컴 호환 3개로 정규화해 본문이 사라지는
serializer 결함을 고친다. source commit 1/1이 통합 후보에 적용됐다.

## 검증과 잔여 한계

- source head의 check는 18 success, 1 neutral, 4 skipped, failure 0이다.
- `issue_5933_hml_page_border_fill` 공개 계약 test와 통합 code candidate 전체 nextest 8,201 passed가
  정규화 경로를 검증한다. 현 head의 merge-tree·공백·fmt·unit-tier도 통과했다.
- 원 PR의 한컴 실측 결론은 HML 출처 저장본의 본문 보존이지만, 사용 가능한 source diff에는 해당 원본
  HML과 기준 PDF가 포함되지 않았다. 따라서 이 기록은 공개 계약과 CI를 근거로 수용 권고하며, 비공개
  원본의 시각 결과를 일반 공개 증적처럼 주장하지 않는다.

## 판정

**수용 권고.** HML adapter의 누락된 정규화만 좁게 보완했고 공개 회귀와 source CI가 통과했다. 통합 PR
최신 CI와 작업지시자 승인이 merge 전 조건이다. 비공개 원본의 추가 fidelity 차이는 별도 이슈로 분리한다.
