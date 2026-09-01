# PR #6578 검토 기록 - 빈 줄 TAC Picture 상단 배치

## 대상과 범위

- PR: [#6578](https://github.com/edwardkim/rhwp/pull/6578)
- base: `upstream/devel` `89db03103db0a37f41ad2270a922c028ce20e7d2`
- 최종 검토 head: `53282aa5e859c509a56aa7206e00a88aba9c5fcb`
- 원본 contributor head: `c1f3c7bf1c0f8347681681e9e92ad650ea14d7f4`
- 범위: 저장 lineseg보다 작은 TAC Picture가 baseline 바닥 기준으로 내려가는 #6575 배치 결함과 그 회귀 검증

## 원본 PR 검토와 메인터너 보정

원본 head는 한 `Picture` baseline 경로에 일반적인 `raw_lh - pic_h > 4px` 줄 상단 규칙을 추가했다. 이 방식은 캡션을 포함한 실제 개체 상자 높이를 고려하지 않고, 동일 배치 경로 전체에도 일관되게 적용되지 않았다. 회귀 테스트도 폭이 같은 여러 그림이 생기면 마지막 그림을 검증할 수 있었다.

메인터너 보정은 다음을 반영했다.

- Top/Bottom 캡션의 간격과 문단 높이를 포함한 `tac_object_box_height_px`를 baseline 정렬에 사용한다.
- 동일 TAC Picture baseline 경로가 같은 개체 상자 규칙을 사용하도록 맞춘다.
- `issue_6575_tac_picture_line_top`이 폭 `557.25px` 대상 그림을 정확히 하나 찾는지 단언한다.
- 축소 HWPX fixture `samples/issue6575/tac_picture_line_top.hwpx`를 PR에 포함한다.

최종 PR diff는 남은 baseline 경로에서 `pic_h` 대신 개체 상자 높이를 사용하고, fixture 기반 회귀 테스트를 추가하는 범위다.

## 검증

`53282aa5e` rebase head에서 다음을 완료했다.

- Rust fmt, native/WASM Clippy, workspace build, Rust test-suite manifest 검증 통과
- #6575 focused regression, #5789, #1898 회귀 통과
- Native Skia 라이브러리 `3946 passed, 0 failed, 13 ignored` 및 관련 renderer 회귀 6건 통과
- WASM package build 통과
- full nextest `8925 passed, 0 failed, 46 skipped`

Hancom Office 2020 기준 PDF와 rhwp SVG PDF의 전 8페이지 visual sweep도 수행했다. 물리 페이지 5의 캡션과 이후 표 흐름에는 별도 renderer 수직 차이가 남지만, 현재 PR의 TAC Picture 개체 상자 보정 범위를 넘는 기존 과제로 분리한다. 이 PR의 직접 변경을 거절하는 근거로 사용하지 않는다. 로컬 증적은 `pdf/pr_6578_rebased_sweep_20260902/`에 보관한다.

## 판정과 다음 단계

- 판정: `메인터너 보정 됨 수용 가능`
- 병합 전 조건: 갱신된 PR head의 필수 CI, Native Skia, Render Diff, CodeQL, test archive가 모두 green이고 mergeability를 다시 확인한다.
- 현재 상태: PR head CI 진행 중이므로 merge하지 않는다.
- 조건 충족 뒤: admin merge 후 merge SHA, `devel` 반영, PR/issue 기록, contributor branch와 검토 산출물 정리를 절차에 따라 수행한다.

## Merge 후 contributor PR comment 계획

게시 gate는 원 코드 PR의 merge SHA 확인, `upstream/devel` fast-forward, #6575 close 상태 확인, 그리고 실제 CI 결과 재확인 뒤다. merge 전에는 comment를 게시하지 않는다.

- 대상 PR: [#6578](https://github.com/edwardkim/rhwp/pull/6578)
- 관련 issue: [#6575](https://github.com/edwardkim/rhwp/issues/6575)
- PR comment: 감사, [#6578](https://github.com/edwardkim/rhwp/pull/6578)의 merge 사실과 실제 merge commit direct link, Build & Test·Lint·Native Skia·Render Diff·CodeQL·Proptest의 최신 head 성공, 이 문서의 local 검증 요약, 남은 PR 범위 후속 작업 유무를 기록한다.
- issue comment: `upstream/devel` 반영 뒤 #6575의 auto-close 상태와 기존 maintainer 기록을 확인한다. maintainer 기록이 없으면 같은 merge commit과 검증 근거를 남긴다. issue가 OPEN이면 작업지시자 승인 뒤에만 수동 close한다.
- 시각 증적: 이 PR의 수용 판단은 fixture 기반 TAC Picture 회귀와 code 경로 보정에 근거한다. 로컬 `pdf/pr_6578_rebased_sweep_20260902/`는 devel asset이 아니며, 페이지 5의 별도 renderer 차이도 PR merge 판단 근거가 아니므로 PR/issue comment에 raw URL·이미지를 넣지 않는다.

게시 직전 실제 결과로 아래 초안의 placeholder를 채운다. 미완료 check 또는 devel에 없는 asset을 성공·link로 기록하지 않는다.

~~~markdown
검토 및 머지 완료했습니다. 감사합니다.

- merge: [#6578](https://github.com/edwardkim/rhwp/pull/6578) -> [<merge-sha>](https://github.com/edwardkim/rhwp/commit/<merge-sha>)
- CI: Build & Test, Lint, Native Skia tests, Render Diff, CodeQL, Proptest roundtrip의 최신 head 성공 확인
- 로컬 검증: full nextest 8925 passed, Native Skia 3946 passed, WASM package build 완료
- 후속 작업: #6575 auto-close 상태 <OPEN/CLOSED> 확인; PR 범위 내 추가 작업 없음
~~~
