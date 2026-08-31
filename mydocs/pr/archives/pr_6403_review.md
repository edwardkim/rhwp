---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6403
author: kevin9327
---

# PR #6403 review - HWP3 char-shape 경계 보존

## Metadata

- 원 PR: [#6403](https://github.com/edwardkim/rhwp/pull/6403), source head
  `31717eea3e3d59e5a57798d899bd1aa0b49542f4`.
- 작성자: `kevin9327`; external collaborator 경로로 reviewer `jangster77`를 요청했다.
- 2026-08-30 고정 시점에 Open, non-draft, CI green인 head를 최신 `upstream/devel`
  위 검토 branch에 누적 cherry-pick했다. merge 전에는 head와 required check를 재확인한다.

## 변경과 검토

- HWP3 원본의 `char_shapes` 경계를 HWPX 저장 후 재파싱할 때 보존한다.
- #5251의 FFFC 축을 `issue_265`의 실제 문단으로 제한하고, 정상화 뒤의 잔존 IR 표본과
  빈 줄 회귀 검사를 고정한다.
- 저장소 포맷 경계와 parser/serializer 회귀만 바꾸며, HWP/HWPX 기준 fixture나 renderer
  조판을 변경하지 않는다. 이 PR 자체에는 visual sweep 산출 의무가 없다.

## 검증과 권고

- 통합 branch의 full nextest는 fixed `target/pr-review`에서 `8772 passed, 43 skipped`
  (430.908초, exit 0)로 완료했다. `fmt`, native/WASM/workspace all-target clippy,
  workspace build, manifest check도 보정 후 다시 통과했다.
- source head의 기존 CI green 상태와 HWP3/HWPX 경계 회귀 test를 함께 검토했다.

**수용.** HWP3 원본의 style-run 경계를 의도치 않게 재분할하는 회귀를 막는 범위로 수용한다.
