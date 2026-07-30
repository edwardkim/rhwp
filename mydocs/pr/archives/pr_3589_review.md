---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-30
---

# PR #3589 리뷰 — 표·연결선 레코드 0 패딩 한컴 정합 (#3570)

- PR: [#3589](https://github.com/edwardkim/rhwp/pull/3589) / 작성자: planet6897
- 역할: maintainer 일반 경로 + local_validation (4.3 Rust parser/serializer 행)
- 규모: 4 files, +230/−17 — serializer/control.rs, hwpx_to_hwp.rs(계약 제거), object_ops/table.rs, 회귀 테스트(+206)

## 검증 기록

| 검증 | 결과 |
| --- | --- |
| 충돌 simulation (devel merge) | clean |
| focused 2계약 | 2 passed (표 zone 종료·연결선 4바이트) |
| 384쪽 편람 변환 | 성공 (8.87MB) |
| **한컴 2020 개방 스모크** (MCP) | **status success · run_status 0 · 재저장 9.28MB** — 저장 계약 되돌림의 실물 판정 |
| release-test 전체 / fmt / clippy | 374 ok · 통과 · 경고 0 |
| PR head CI | 전 check green |

## 최종 권고

**merge 권고.**
