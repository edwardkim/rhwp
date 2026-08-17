---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #4974 검토 - HWPX 말미 zero-width 글자모양 경계

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4974](https://github.com/edwardkim/rhwp/pull/4974) |
| 작성자 / source | @planet6897 / `fix/3532-trailing-boundary` |
| 원 source head | `2cb99453e6f8c96658a2697dc1044c1ed4068e51` |
| 기준 devel | `8d4fb781c2f253f4a9993938f51e6bf415d8488e` |
| 가시성 검토 branch | `review/nondraft-prs-20260817` |
| local 적용 commit | `a3cb16995`, `081ea59d8` |
| 원 PR 상태 참고값 | `OPEN` / `MERGEABLE`; 실패·대기 check 없음 |

본문 끝 zero-width 글자모양 경계를 말미 컨트롤보다 먼저 적용해, HWPX 저장 뒤에도 trailing
charshape 경계와 IR 검증 표본의 의미를 보존한다. 최신 `devel`의 integration suite 규약에 맞춰
fixture를 `tests/cases/`로 등록하고 생성 manifest를 갱신하는 메인터너 보정 `a2907c573`을 추가했다.

## 검증과 판단

| 범위 | 결과 |
| --- | --- |
| 전용 fixture | `issue_3532_trailing_charshape_boundary` 1 passed |
| 누적 Rust | release-test nextest 6,529 passed, 38 skipped, 7 slow, 334.412초 |
| 품질 | fmt, clippy, doctest, manifest/unit-tier, diff 검사 통과 |
| 원 source CI | Build & Test 및 CodeQL 성공 |

fixture가 최신 suite manifest에 포함되어 있으며 재배치 외 내용 변경은 없다. **통합 수용 권고.**
