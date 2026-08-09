---
kind: pr-review
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4283 검토 - zero span 셀 분할 방어

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4283](https://github.com/edwardkim/rhwp/pull/4283) / `kevin9327` |
| 관련 이슈 | #4280 |
| base / source head | `devel` / `9f92c4afd74e167f9e4010d71648258ca2ec2175` |
| 누적 적용 | `1b6de4389` |
| 접수 참고 상태 | MERGEABLE / CLEAN, 2 files, +25 / -0 |

## 판정

**수용 권고.** HML 입력의 명시적 span 0은 `unwrap_or(1)`을 우회하며, 기존 split path는
빈 vector index 또는 0 나누기로 이어졌다. 변경은 split 진입에서 손상 span을 거부해 두 패닉
경로를 함께 막는다. table focused 79건과 전체 Rust 회귀 5,499건이 통과했다.

모델 편집 경로만 바뀌며 renderer/fixture 변경은 없다. 실제 merge 전 최신 source head와
required checks를 재확인한다. 공통 누적 검증은 [통합 검토 계획](pr_4282_review_impl.md)에 있다.
