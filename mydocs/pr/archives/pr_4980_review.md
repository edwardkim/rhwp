---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #4980 검토 - 표·차트 CSV 왕복 도구 온톨로지 등록

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4980](https://github.com/edwardkim/rhwp/pull/4980) |
| 작성자 / source | @kevin9327 / `feat/rhwp-desk-csv-roundtrip-tools` |
| 원 source head | `920929ca5469d7a978b6b0a02f41fcafa6d8e9d9` |
| 기준 devel | `8d4fb781c2f253f4a9993938f51e6bf415d8488e` |
| 가시성 검토 branch | `review/nondraft-prs-20260817` |
| local 적용 commit | `cbe7dd422` (선행 #4975, #4978, #4979 적용 뒤) |
| 원 PR 상태 참고값 | `OPEN` / `MERGEABLE`; 실패·대기 check 없음 |

표와 차트의 CSV 내보내기·가져오기를 서로 섞지 않는 별도 온톨로지 경로로 등록한다. 표 검색은 셀 채우기와
CSV 내보내기로, 차트 경로는 차트 데이터 흐름으로만 이어져 잘못된 후속 도구 선택을 막는다.

## 검증과 판단

| 범위 | 결과 |
| --- | --- |
| ontology unit | 표/차트 경계, chart CSV 닫힘, leaf 판정 관련 desk unit 통과 |
| rhwp-desk | Rust 34 passed, fmt, clippy, build, JS 구문 검사 통과 |
| 누적 Rust | release-test nextest 6,529 passed, 38 skipped |
| 원 source CI | Build & Test 및 CodeQL 성공 |

데이터 유형별 후속 작업 그래프가 분리되어 있다. **통합 수용 권고.**
