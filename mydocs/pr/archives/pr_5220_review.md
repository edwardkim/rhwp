---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5220 검토 - 제목 차례와 각주 배치

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#5220](https://github.com/edwardkim/rhwp/pull/5220) |
| 작성자 | @planet6897 |
| 원 source head | `d2366a88b796850f1e9e171592117092c5e973bf` |
| 기준 devel | `e79f113080ead96c391391d211a0a64fa8398378` |
| 가시성 검토 branch | `review/planet6897-20260818-r1` |
| 검토 head | `d10030fc8` |
| 원 PR 상태 | `OPEN` / non-draft |

## 검토 결과

제목 차례 표시 영역의 슬롯 갭을 실제 본문 높이에서 제외해 각주가 문단 맨 앞으로
밀리는 문제를 보정했다. `footnote_is_emitted_after_title_mark_and_text`와 관련 HWPX
section 직렬화 경로를 확인했으며, 전체 회귀에서도 통과했다.

차단 결함은 발견하지 못했다. 원격 source branch는 수정하지 않았다.

## 검증

- 관련 회귀 포함 전체 nextest: `7219 passed, 38 skipped, 11 slow`
- `cargo fmt --all -- --check`, manifest/unit-tier check: 통과
- root·WASM·workspace clippy `-D warnings`: 통과
- `git diff --check`: 통과

## 판단

로컬 통합 검토 기준 수용 가능하다. 원격 통합 PR 생성과 원 PR 후속 처리는 작업지시자 승인 후
진행한다.
