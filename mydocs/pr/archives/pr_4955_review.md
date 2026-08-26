---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #4955 검토 - HWP5 덧말 tdut 양방향 보존

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4955](https://github.com/edwardkim/rhwp/pull/4955) |
| 작성자 / source | @planet6897 / `fix/4397-ruby-hwp5` |
| 원 source head | `9ed4d12374d34e092d95fd5a5eadaa3ad979734e` |
| 기준 devel | `418e5b191d23cf0618ce99f0cfec332c19ac1bc2` |
| 통합 branch / local 적용 | `review/non-draft-20260816` / `ac475bdac` |
| 관련 issue | #4397 |
| 작성 시점 원 PR 상태 | `OPEN` / `MERGEABLE` / `CLEAN`; merge 전 재확인 필요 |

HWPX Ruby control을 HWP5 `tdut` CTRL_HEADER payload로 직렬화하고 반대 파서를 추가해 main/sub text와 다섯
속성이 HWPX→HWP5→HWPX 왕복에서 사라지지 않도록 한다.

## 검증과 판단

신규 `issue4397_ruby_survives_hwp5_roundtrip`을 포함한 최종 release-test nextest 전체가 6,519건 통과했다.
fmt, clippy, diff 검사도 통과했다. 한글 COM으로 최종 산출물을 여는 별도 오라클 검증은 source PR에도 없었고
이번 통합 범위에서는 추가 실행하지 않았다. 스펙 기반 parser·serializer 왕복 계약은 고정됐다.
**통합 수용 권고.**
