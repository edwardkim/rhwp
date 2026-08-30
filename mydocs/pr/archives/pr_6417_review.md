---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6417
author: kevin9327
---

# PR #6417 review - dump-pages 진단 높이 정합

## Metadata

- 원 PR: [#6417](https://github.com/edwardkim/rhwp/pull/6417), source head
  `7db64aa334bbe377a7ee55a25842bd678b51db8a`.
- 작성자: `kevin9327`; external collaborator reviewer `jangster77` 요청 완료.
- 고정 시점에 Open, non-draft, CI green인 source를 latest `upstream/devel` 위 통합 검토
  branch에 conflict 없이 적용했다.

## 변경과 검토

- `dump-pages`가 실제 production typeset과 동일한 높이를 보고하도록 renderer 진단 경로를
  맞춘다.
- #4628의 p83 높이 기대값과 주변 diagnostic test가 production 출력에 맞게 갱신된다.
- 진단 출력과 test expectation의 정합 변경이며, HWP/HWPX fixture 또는 visual output 계약을
  직접 바꾸지 않는다. visual sweep은 적용 대상이 아니다.

## 검증과 권고

- source CI green 및 통합 branch full nextest `8772 passed, 43 skipped` (430.908초,
  exit 0)를 확인했다. `fmt`, native/WASM/workspace all-target clippy, workspace build,
  manifest check도 보정 후 다시 통과했다.
- #6417 뒤에 contributor가 기록한 diagnostic 예상값 조정 사유를 확인했고, 변경이 renderer
  production 조판을 덮어쓰는 방식이 아니라 dump-pages 관측값을 맞추는 범위임을 확인했다.

**수용.** diagnostic contract 정합 변경으로 수용한다.
