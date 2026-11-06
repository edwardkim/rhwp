# 통합 PR #8683 원격 검증·병합 완료 기록

- 기여자: hyunmyungJaneLee. 원 PR [#8515](https://github.com/edwardkim/rhwp/pull/8515)·[#8599](https://github.com/edwardkim/rhwp/pull/8599)·[#8608](https://github.com/edwardkim/rhwp/pull/8608)·[#8631](https://github.com/edwardkim/rhwp/pull/8631)·[#8656](https://github.com/edwardkim/rhwp/pull/8656).
- [통합 PR #8683](https://github.com/edwardkim/rhwp/pull/8683)의 검증 head `71052fccc6fc5090fa94ca9dd013be0ff5984ff7`, base `5716a226b562b7300193dd0b573ded9cd71ac890`.
- merge [`c80f355faa2bd810eda0724ec606027e7bbc019f`](https://github.com/edwardkim/rhwp/commit/c80f355faa2bd810eda0724ec606027e7bbc019f), 2026-11-06 14:39:03 KST. 원 기여 SHA 5개 보존·원 PR 5건의 간접 MERGED를 API로 확인했다. merge의 제품 src tree는 검증한 `c1ea19fbc94dd6aac77d4368755b6da5a9544e42`와 같다.

## GitHub CI와 self-review

| 검증 | 실제 실행 | 결과 |
| --- | --- | --- |
| Build & Test·Native·Lint·Frontend·Archive A–E | [42166096972](https://github.com/edwardkim/rhwp/actions/runs/42166096972) | success |
| CodeQL 4개 언어·집계 | [42166096800](https://github.com/edwardkim/rhwp/actions/runs/42166096800) | success |
| Render Diff | [42166096920](https://github.com/edwardkim/rhwp/actions/runs/42166096920) | success |
| Adapter inter-diff | [42166097004](https://github.com/edwardkim/rhwp/actions/runs/42166097004) | success |
| Proptest roundtrip | [42166096803](https://github.com/edwardkim/rhwp/actions/runs/42166096803) | success |
| 최종 CI Impact Policy 감사 | [42169452466](https://github.com/edwardkim/rhwp/actions/runs/42169452466) | success·status 게시 확인 |

Archive A/B/C/D/E 실제 실행은 각각 3,801/2,018/1,835/2,596/2,308 passed, 합계 **12,558 passed·기존 45 skipped**다. [실제 로그의 집계](ci-archive-actual-results.json)와 [전체 check·상태](github-final-validation.json)에 기록했다. 로컬 **12,752 passed**, Native 공식 **4,064 passed**, 집중 **27 passed**는 [로컬 최종 검증](final-validation.md)과 별개다. workflow 재실행·취소 없이 첫 실행에서 모두 성공했다.

[self-review](self-review-final.json)에서는 문단 원점의 생산·소비와 떠 있는 paint/흐름 분리, signed offset 범위, 격자의 Storage 예산·실패 전파, Link 슬롯 순번과 soft-edge 꼬리의 저장 계약을 재검토했다. 새 차단 사항은 없다. 보안 봇 #478·#483·#484는 이전 #8659에도 있던 선택적 진단 로그이며 해당 파일은 base와 같다. [직접 판독 결과](security-bot-review.json)에 근거를 남겼고 경고 분류는 변경하지 않았다.

## 승인된 예외와 후속 순서

#8599의 기존 그림 뒤 본문 겹침에 한정한 수용 예외를 적용했다. 새 회귀 예외가 아니며 [#8579](https://github.com/edwardkim/rhwp/issues/8579)는 **OPEN 유지**다. #8513·#8584·#8594·#8593은 해결 범위를 확인했고, 이 운영 기록 시점에는 OPEN이다. 기록 push·최종 devel 동기화 뒤 close와 근거 댓글을 게시한다. #8593의 크기·alpha 복원과 blur/모든 효과 지원은 구분한다.

원 PR별 댓글은 각각의 archive 계획과 comment-pr-*-draft.md를 따른다. 원 재현물 7쪽·Native/WASM의 review/overlay 28개를 이 merge SHA로 고정해 HTTP 200·image/png·SHA-256 일치를 확인했다. 운영 기록·devel 동기화 뒤 게시하고 API 본문과 실제 GitHub 화면을 확인한다.

후속 문서는 **maintainer 직접 반영 옵션 M**으로 archive 5개·공통 assets·오늘할일만 한 commit에 반영한다. 코드 PR은 workflow/CI 정책 변경을 포함하지 않아 별도 기록 PR을 강제하는 옵션 2 대상이 아니다. 후속 처리가 끝나면 작업 전용 branch·두 worktree·fetch ref를 제거하고 기본 작업공간의 output 전체와 target/pr-review/debug 전체를 정리한다. contributor fork와 다른 작업의 worktree는 보존한다.

## 병합 후 duration 갱신

[Refresh nextest target duration data](https://github.com/edwardkim/rhwp/actions/runs/42169674541)는 success다. 이 PR의 실제 B–E 측정 artifact 4개를 수집했고 `ci-metrics/nextest-target-durations`에 commit `e581547d281f744262fd8f06bb3d99e66dbeb2a0`을 게시했다. [실제 수집·게시 결과](duration-refresh-result.json)를 확인했다. 병합 후 CI·CodeQL·Adapter·Proptest·Oracle 검증을 시작하거나 재실행하지 않았다.
