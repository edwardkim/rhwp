---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-30
---

# PR #3588 리뷰 — HWP3 자동번호 8 코드유닛 (#3504)

- PR: [#3588](https://github.com/edwardkim/rhwp/pull/3588) / 작성자: planet6897
- 역할: maintainer 일반 경로 + local_validation (4.3 Rust parser/serializer 행)
- 규모: 2 files, +154/−1 — parser/hwp3/mod.rs(+8) + 회귀 테스트(+146)

## 검증 기록

| 검증 | 결과 |
| --- | --- |
| 충돌 simulation (devel merge) | clean |
| focused 2계약 | 2 passed |
| **red-check** (8→1 유닛 복원) | **+213 바이트 / 말미 공백 210건 정확 재현** → 원복 후 0 |
| 실측 (SO-SUEOP HWP3 46쪽) | export-text 전문 **136,021 = 136,021 (차 0)**, 말미 공백 추가 **0건** |
| 코드 교차검증 | HWP5 파서 `pos += 16`(=8 코드유닛)와 동일 규약 확인 (body_text.rs:350) |
| release-test 전체 / fmt / clippy | 374 ok · 통과 · 경고 0 |
| PR head CI | 전 check green |

## 최종 권고

**merge 권고.**
