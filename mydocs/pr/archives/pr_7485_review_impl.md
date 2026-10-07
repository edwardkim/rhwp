# PR #7485 collaborator 보정·검토 기록

## 대상과 승인

- source: `z0rimo/rhwp:feat/7477-find-hit-count`, 원 head `7da9c6ca3ee48f26c16e6b882e7cc9353bf541f6`.
- @z0rimo 원 contributor commit·작성자·credit을 보존했고 rebase/amend/force-push하지 않았다.
- 작업지시자가 2026-09-30 보정 code/test·review/impl·캡처를 같은 PR에 push하고 두 코멘트 게시, 최신 CI 성공 뒤 Approval까지 승인했다. merge·issue close는 승인되지 않았다.
- 기존 승인된 격리 작업공간의 `review/z0rimo-20260930` branch에서 연속 작업했다. 사용자 활성 checkout과 원본 `pr7485` 복사본은 변경하지 않았다.

## Commit과 검증

| 범위 | Commit 또는 기록 |
| --- | --- |
| 원 contributor 기여 | `7da9c6ca3ee48f26c16e6b882e7cc9353bf541f6` · feat(studio): show find match count |
| 보정 code/test | `73f27d3fa70b2a00bdf014eec814dbb52f7edc0c` · fix(studio): refresh find counts and handle Korean IME activation |
| 별도 review/impl·PNG·색상 관측값 | 이 문서를 포함하는 trailing documentation commit. 최종 SHA는 실제 Approval에 기록 |
| 캡처 시점 최종 patch SHA-256 | `199f078b7b6e91d27e36e7604383571e969e1f44b6428e8cedd2cc1c030c8341`. 이후 E2E MANIFEST 두 행만 추가 |

열린 창의 개수 갱신·history 검색 dedup, 토큰·상태 행과 첫 순환 안내, 한글 조합 첫 활성화와 짧은 버튼 표현을 보정했다. 실제 Mac 다음 첫 클릭 확인과 격리 Chrome 양방향 회귀 검사를 구분했다. 원 PR 신규 count 결함, 기존 UI 동작, 최초 보정안의 중복 검색을 구분했다.

전체 Studio `npm test` 1,812 passed·2 skipped·0 failed, 타입 검사, focused32·두 E2E, production frontend build와 실제 UI 캡처를 완료했다. production 성능 비교는 중복 제거 단계 `9fc538a8…`의 기존 자료를 재사용했고 이후 IME·배치 보정의 동일 production 재벤치마크는 실행하지 않았다. 자세한 입력·방법·경계는 [review](pr_7485_review.md)에 기록했다.

## 실행 단계와 대기 조건

1. remote/source/worktree 시작 SHA와 maintainerCanModify=true를 확인했다.
2. code/test를 별도 commit으로 기록했고, review/impl·PNG를 별도 documentation commit으로 작성했다.
3. push 직전에 remote SHA를 다시 확인하고, 변경 경로의 LFS filter와 `git lfs status`를 판독한다. LFS 대상이 없으면 처음부터 `GIT_LFS_SKIP_PUSH=1` dry-run/push만 사용한다. 다른 hook은 유지한다.
4. 승인된 현재 source branch에만 push하고 remote ref·PR head·local HEAD 일치를 확인한다. 사용자 다른 PR branch와 별도 docs PR에는 올리지 않는다.
5. 공개 이미지 commit SHA로 고정한 링크를 채워 최초 검토→로컬 보정의 두 Conversation 코멘트를 게시하고 API로 한글·실제 줄바꿈·URL을 재조회한다.
6. code/test 보정이 있으므로 docs-only fast-pass를 적용하지 않고 최신 head의 관련 CI를 확인한다. 성공 후 실제 correction·최종 head·CI·검토 기록을 명시해 Approval을 게시한다.
7. merge·issue close·후속 comment·cleanup은 별도 범위다. 사용자 수동 테스트용 localhost 서버는 유지하며 환경 종료 시 사라질 수 있다.

이 문서는 작성 시점의 완료된 로컬 검증과 대기 중 원격 조치를 구분한다. PR head가 바뀌면 기존 결과를 새 contributor 변경에 자동 적용하지 않는다. 게시된 collaborator 변경을 되돌릴 필요가 있으면 새 commit을 사용하고 contributor history를 rewrite하지 않는다.
