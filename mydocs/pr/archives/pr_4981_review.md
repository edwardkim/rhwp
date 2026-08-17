---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #4981 검토 - 워터마크 검증 축 추가

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4981](https://github.com/edwardkim/rhwp/pull/4981) |
| 작성자 / source | @kevin9327 / `feat/rhwp-desk-watermark-axis` |
| 원 source head | `bc43d392d946020b777f8763caf73a99289ee0f0` |
| 기준 devel | `8d4fb781c2f253f4a9993938f51e6bf415d8488e` |
| 가시성 검토 branch | `review/nondraft-prs-20260817` |
| local 적용 commit | `3e04b6408` (선행 #4975, #4978, #4979, #4980 적용 뒤) |
| 원 PR 상태 참고값 | `OPEN` / `MERGEABLE`; 실패·대기 check 없음 |

검증 결과를 IR·page·render에 watermark까지 더한 네 축으로 확장한다. 메인터너 보정 `ffe221908`은
batch UI와 main UI가 서로 다른 축 목록을 갖지 않도록 `VERIFY_AXES` 단일 상수를 사용하게 했다.

## 검증과 판단

| 범위 | 결과 |
| --- | --- |
| ontology/unit | 네 검증 축은 leaf이며 다음 도구를 제안하지 않는 계약 통과 |
| rhwp-desk | Rust 34 passed, fmt, clippy, build, JS 구문 검사 통과 |
| 누적 Rust | release-test nextest 6,529 passed, 38 skipped |
| 원 source CI | Build & Test 및 CodeQL 성공 |

표시와 온톨로지가 같은 네 축을 참조하므로 축 누락·불일치를 막는다. **통합 수용 권고.**
