# PR #8825 검토

## 최종 판정

**승인 — 로컬 검증 완료, 승인된 통합 경로로 원격 처리 완료.** 원 head의 기존 CI 상태와 새 통합 head에 필요한 GitHub CI를 구분한다.

- 원 PR: [#8825](https://github.com/edwardkim/rhwp/pull/8825), dependabot[bot], devel 대상, non-draft.
- 변경: chore(deps): bump crc32fast from 1.5.3 to 1.5.4.
- 고정 원 head: `a705ee565a3985b560db58167e70b5678285ddd6`. reviewer edwardkim 지정·API 확인 뒤 상세 조사·fetch를 진행했다.
- 기준 devel: `024d610633ede166ebcbb915f9c90e7a9d0649d8`. 결합 검증 source: `2320351d2325a41d900e91f4e07f559d4a924a7d`. 누적 merge에 충돌이 없고 원 head는 현재 branch의 조상으로 보존됐다.
- 추가 하네스·고지 보정 commit은 `42f82843ec5c957e4e06c414e7f9d0ade7cf052b`다. [실제 보정과 검증 source의 동일성](../assets/dependabot-20261109/source-review.md)을 별도로 확인했다.
- 역할: maintainer_general, intake_and_review·local_validation·multi_pr_update_branch·rework_and_exceptions·visual_fixture_evidence·post_merge.

원 commit을 보존한 결합 graph에서 실제 소비 범위의 필수 로컬 검증을 완료했다.

적용 검증은 Rust focused·전체 회귀·Native 공식 3종·fresh WASM과 실제 출력다. [의존성별 실제 소비·원 CI 원인](../assets/dependabot-20261109/dependency-review.md)의 해당 행과 [공통 실행 원장](../assets/dependabot-20261109/README.md), [원 head 최신 CI](../assets/dependabot-20261109/source-review.md)를 따른다. 공유 검증을 19번 실행한 것으로 집계하지 않는다.

조판 규칙·geometry·pagination 분기 자체를 바꾸는 구현은 없다. 실제 출력 영향이 가능한 의존성은 Native/fresh WASM 각 9쪽 전쪽 TSV·직접 확대 판독과 편람 Native 383쪽에서 검증했다. 새 fixture·golden·절대 좌표 핀·기대값 변경이 없다. Action과 build 도구 갱신은 backend 조판 개선을 주장하지 않는다.

승인 뒤 단일 통합 PR에 원 기여를 보존해 push하고 정확한 head CI·self-review·최신 base merge-tree를 확인한 뒤 merge한다. 원 PR별 완료 상태를 확인하고, 옵션 2의 별도 후속 기록 PR·devel 동기화 뒤 결과 댓글과 이번 전용 작업공간·output·미사용 debug 정리를 수행한다. 원 CI의 실패·취소는 보존하며 승인 없는 재실행·분류 변경은 하지 않는다.

## 원격 처리 결과

[통합 PR #8840](https://github.com/edwardkim/rhwp/pull/8840)의 제출 head `91453509c9fb10d24bc8fb318b5437f2e80692fa`를 검증했고 [merge `153d5aede886077772349569e900fd770adddc93`](https://github.com/edwardkim/rhwp/commit/153d5aede886077772349569e900fd770adddc93)로 수용했다. 사용자 원격 승인 뒤 실행했으며 [self-review](https://github.com/edwardkim/rhwp/pull/8840#pullrequestreview-5822605316)를 공개했다. 원 기여 head 19개를 모두 보존했다.

- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/42573393012): success, attempt 1, head `91453509c9fb10d24bc8fb318b5437f2e80692fa`.
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/42573392969): success, attempt 1, head `91453509c9fb10d24bc8fb318b5437f2e80692fa`.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42573392806): success, attempt 1, head `91453509c9fb10d24bc8fb318b5437f2e80692fa`.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42573392794): success, attempt 1, head `91453509c9fb10d24bc8fb318b5437f2e80692fa`.
- [CI](https://github.com/edwardkim/rhwp/actions/runs/42573392890): success, attempt 1, head `91453509c9fb10d24bc8fb318b5437f2e80692fa`.
- CI Impact Policy: success.

후속 기록은 CI 변경 PR의 옵션 2로 별도 review-only PR에 반영한다. 원 코드 PR에는 trailing commit·추가 CI를 만들지 않았다. 원 PR 19건 모두 MERGED임을 확인했고 결과 댓글과 종료 정리는 기록 병합 뒤 수행하며 최종 실행 결과는 해당 PR 댓글과 `target/pr-review/cleanup-pr8840-20261109.json`에서 확인한다. closing issue는 없다.

## Merge 후 contributor PR comment 계획

- 대상: [원 PR #8825](https://github.com/edwardkim/rhwp/pull/8825). [댓글 초안](../assets/dependabot-20261109/comment-drafts/pr-8825-accepted.md)에 실제 통합 PR·merge·CI·증거 SHA를 채운다.
- 원 기여 head는 통합 head `91453509c9fb10d24bc8fb318b5437f2e80692fa`의 조상이다. 원 CI의 실패·취소와 새 head CI 성공을 분리하고, 공동 회귀 13,005·Native 4,077·Studio 2,778건을 원 PR마다 반복한 것으로 세지 않는다.
- 원 PR은 의존성 갱신이며 원 문서 배치 수정 주장이 없다. 실제 소비의 호환 대조군 `bitmap-channel` 1쪽을 표시한다. Native Skia/fresh WASM Canvas2D 각각의 review·overlay 네 PNG를 사용한다. 전체 9쪽의 두 backend raw 최저 100.00%·base/후보 byte 동일과 확대 판독을 기록한다.
  - [댓글의 native review](../assets/dependabot-20261109/visual/candidate/native/bitmap-channel/review/review_001.png).
  - [댓글의 native overlay](../assets/dependabot-20261109/visual/candidate/native/bitmap-channel/overlay/overlay_001.png).
  - [댓글의 wasm review](../assets/dependabot-20261109/visual/candidate/wasm/bitmap-channel/review/review_001.png).
  - [댓글의 wasm overlay](../assets/dependabot-20261109/visual/candidate/wasm/bitmap-channel/overlay/overlay_001.png).
- 게시 직전 실제 증거 commit의 HTTP 응답·PNG 형식·SHA-256을 Git 원본과 대조한다. 게시 후 API 본문·한글·링크·이미지를 확인하며 동일 marker 댓글은 중복하지 않는다. 남은 작업은 이 결과 댓글과 종료 정리다.
