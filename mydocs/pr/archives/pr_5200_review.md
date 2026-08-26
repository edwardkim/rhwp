---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5200 검토 - 표를 감싼 0길이 누름틀의 FIELD_END 위치

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#5200](https://github.com/edwardkim/rhwp/pull/5200) |
| 작성자 | @planet6897 |
| 원 source head | `6c82885ab59cff3f9e9f562897c7e134582b5066` |
| 기준 devel | `e79f113080ead96c391391d211a0a64fa8398378` |
| 가시성 검토 branch | `review/planet6897-20260818-r1` |
| 검토 head | `d10030fc8` |
| 원 PR 상태 | `OPEN` / non-draft |

## 검토 결과

HWP5에서 표를 감싼 0길이 누름틀의 `FIELD_END`가 표·개체 뒤에 닫히도록 하는 변경과
`issue5162_field_wraps_table.hwpx` 회귀 fixture를 최신 devel 위에 누적 적용했다. 해당
순서가 HWP5 roundtrip 결과에 반영되는 focused test와 전체 회귀에서 확인된다.

차단 결함은 발견하지 못했다. 원격 source branch는 수정하지 않았다.

## 검증

- focused 관련 테스트 포함 전체 nextest: `7219 passed, 38 skipped, 11 slow`
- `cargo fmt --all -- --check`, manifest/unit-tier check: 통과
- root·WASM·workspace clippy `-D warnings`: 통과
- `git diff --check`: 통과

## 판단

로컬 통합 검토 기준 수용 가능하다. 원격 통합 PR 생성과 원 PR 후속 처리는 작업지시자 승인 후
진행한다.
