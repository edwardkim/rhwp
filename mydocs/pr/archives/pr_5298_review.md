---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5298 검토 - HWPX breakLatinWord 역매핑

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#5298](https://github.com/edwardkim/rhwp/pull/5298) |
| 작성자 | @planet6897 |
| 원 source head | `b0a05bc22e472a502db7ee96e5aa1307ba3a1bd6` |
| 기준 devel | `e79f113080ead96c391391d211a0a64fa8398378` |
| 가시성 검토 branch | `review/planet6897-20260818-r1` |
| 검토 head | `d10030fc8` |
| 원 PR 상태 | `OPEN` / non-draft |

## 검토 결과

HWPX의 `breakLatinWord` lexical 값을 HWP5 `attr1` bits 5-6에서 역매핑해 HWPX→HWP5
저장 시 줄나눔 설정이 소실되지 않도록 했다. 해당 파서·serializer 계약과 전체 roundtrip을
확인했다.

메인터너 보정으로 회귀 테스트의 standalone 주석 정렬 1줄을 rustfmt 기준에 맞췄다. 동작
변경은 없고 `cargo fmt --all -- --check`를 통과시키기 위한 형식 보정이다.

## 검증

- 관련 회귀 포함 전체 nextest: `7219 passed, 38 skipped, 11 slow`
- `cargo fmt --all -- --check`, manifest/unit-tier check: 통과
- root·WASM·workspace clippy `-D warnings`: 통과
- `git diff --check`: 통과

## 판단

로컬 통합 검토 기준 수용 가능하다. 원격 통합 PR 생성과 원 PR 후속 처리는 작업지시자 승인 후
진행한다.
