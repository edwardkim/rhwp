---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5202 검토 - ViewText 우선 본문 판독

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#5202](https://github.com/edwardkim/rhwp/pull/5202) |
| 작성자 | @planet6897 |
| 원 source head | `80211986ec8f099b164c12536097634b1afc85fc` |
| 기준 devel | `e79f113080ead96c391391d211a0a64fa8398378` |
| 가시성 검토 branch | `review/planet6897-20260818-r1` |
| 검토 head | `d10030fc8` |
| 원 PR 상태 | `OPEN` / non-draft |

## 검토 결과

비배포 HWP5 문서에서도 유효한 `ViewText`가 있고 첫 레코드가 문단 헤더이면 이를 우선
사용하고, 조건을 만족하지 않으면 `BodyText`로 되돌리는 판독 변경을 적용했다. 변경된
`hwp5_prefers_viewtext_over_bodytext_when_not_distribution`와 실제 HWP fixture를 포함한
전체 회귀에서 본문 선택 계약을 확인했다.

차단 결함은 발견하지 못했다. 원격 source branch는 수정하지 않았다.

## 검증

- focused 관련 테스트 포함 전체 nextest: `7219 passed, 38 skipped, 11 slow`
- `cargo fmt --all -- --check`, manifest/unit-tier check: 통과
- root·WASM·workspace clippy `-D warnings`: 통과
- `git diff --check`: 통과

## 판단

로컬 통합 검토 기준 수용 가능하다. 원격 통합 PR 생성과 원 PR 후속 처리는 작업지시자 승인 후
진행한다.
