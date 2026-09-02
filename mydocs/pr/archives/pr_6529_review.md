# PR #6529 검토 - TAC object stored break wins

- 검토일: 2026-08-31
- 작성자: `planet6897`
- base: `devel` (`upstream/devel@887b4ce15`로 rebase)
- 원 PR head: `02ccd4bb70884ae3cb6726e69b8a6f907fe46a75`
- 통합 commit: `baa8179bb`
- 상태: [통합 PR #6537](https://github.com/edwardkim/rhwp/pull/6537) 병합 완료 (`1636910809ce9d1a394b30144fff19cc5fc32826`)

## 범위

- 권위 있는 stored break가 남아 있을 때 측정 폭 기반 조기 wrapping보다 그 break를 우선한다.
- `issue6180/156745974_tac_object_line_spacing.hwpx` fixture와 회귀 테스트를 추가한다.

## 검토 결과

- #6528의 stored break 해석을 전제로, 권위 있는 break가 존재할 때 width-driven wrapping을 중단하도록 범위를 한정했다.
- 목표 회귀 테스트 `issue_6180_tac_object_stored_break_wins`는 `release-test`에서 종료 코드 `0`으로 통과했다.
- #6529 원 PR에는 #6528 commit이 조상으로 포함돼 있어 중복 cherry-pick 없이 #6528 다음에 unique commit만 적용했다.
- Hancom 2020 기준 PDF와 p7 직접 비교를 완료했다. 자동 위험 신호는 `0`건이며, TAC object 아래의 행 간격과 표 경계가 기준 구조를 유지했다.
- 시각 증적: [p7 review 패널](assets/pr_6529_issue6180_p7_review.png)
- 기준 PDF: `pdf/pr_6529_issue6180_p7_2020.pdf`, SHA-256 `d9539968517428f21dc18628ff45fa55b5b64aaf11250ef33f74af097d460589`
- visual sweep: pixel match `86.01365%`, ink match `13.0958%`; 글꼴 rasterizer 차이는 있으나 flagged page 없음.

## 공통 검증

- Rust format, native/WASM/workspace/all-target Clippy, workspace build 통과
- 전체 `nextest` 종료 코드 `0`

## 병합 조건

- 원격 병합 또는 통합 PR 게시 직전에 원 PR head와 CI green 상태를 다시 확인한다.

## Merge 후 contributor PR comment 계획

- 대상: [#6529](https://github.com/edwardkim/rhwp/pull/6529)와 관련 issue #6180.
- 선행 조건: 통합 PR의 merge SHA가 `upstream/devel`에 포함되고 p7 review asset이 실제 merge commit에 존재할 것.
- 내용: 통합 PR·merge SHA, focused regression과 전체 nextest, Hancom 2020 p7 sweep의 flagged `0/1` 및 pixel match `86.01365%`, 사람 검토 결론, asset direct link를 남긴다.
- issue가 OPEN이면 merge 반영과 검증 증적을 comment로 남긴 뒤 close 여부를 확인한다.
