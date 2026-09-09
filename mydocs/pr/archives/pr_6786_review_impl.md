---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-06
pr: 6786
issue: 6635
---

# PR #6786 collaborator 보정·통합 기록

[검토 판정과 증적](pr_6786_review.md)을 기준으로 원 contributor branch에 최소 보정과 review 기록을
순서대로 추가한다. 별도 integration PR, source history 재작성, 자동 merge는 이 경로에 포함하지 않는다.

## 승인 범위와 실행 단계

작업지시자는 보정, 검증, 보정 commit push, review 문서 push와 CI 확인까지 요청했다.
최종 GitHub Approve 리뷰는 **초안을 먼저 보여주고 사용자 승인 후 등록**하도록 경계를 명시했다.

| 단계 | 상태 | 실행 내용 / 완료 조건 |
| --- | --- | --- |
| 1. 접수·원 head 검토 | 완료 | `d0179dd0410757aea8d116ba4be7b38438e406e9`, issue #6635, 원 diff·CI·공개 검증 안내 확인 |
| 2. 격리·권한 확인 | 완료 | 승인된 review worktree, `maintainerCanModify=true`, remote/source/local 시작 SHA 일치 |
| 3. 보정 | 완료 | hidden 글자색 input의 Tab 제외, 형광펜 Escape 닫기·트리거 focus 복원 |
| 4. 보정 검증 | 완료 | 신규 회귀 red 7건, green 64건씩 두 모드, unit·responsive·build·계약·manifest 성공 |
| 5. 코드 push | 완료 | 원 두 commit 보존, `cc21183ab622d13071acfa7be3224903a048526b`를 source branch에 push |
| 6. 코드 CI | 성공 | 보정 head의 CI, CodeQL, Render Diff, Adapter, Proptest 성공 확인 |
| 7. archive 문서 | 이 trailing commit | review와 implementation 두 문서만 추가; generated 파일과 다른 작업의 변경은 제외 |
| 8. 문서 push·최신 checks | 문서 작성 뒤 수행 | LFS 판독, diff/link/merge 검증, remote dry-run, source push, 최신 head 집계 확인 |
| 9. GitHub Approve | 사용자 승인 대기 | 전체 초안과 최신 검증 상태를 보여준 뒤 명시적 승인 후 등록 |
| 10. merge·후속 정리 | 미실행 | 별도 승인, 최신 head 재확인, merge SHA 확인, issue 상태·기여자 감사·로컬 정리 |

## commit 경계와 source 보존

| SHA / 제목 | 경계 |
| --- | --- |
| `67a5d00b832589fd14ec60e7b10f577c8b949d5c` — `fix(studio): 색상 버튼의 키보드 활성화 복원 (#6635)` | 원 기여자 baba9811, 수정하지 않음 |
| `d0179dd0410757aea8d116ba4be7b38438e406e9` — `docs(e2e): #6202 회귀 테스트를 목록에 등록` | 원 기여자 baba9811, 수정하지 않음 |
| `cc21183ab622d13071acfa7be3224903a048526b` — `fix(studio): 색상 도구의 Tab 이동과 Escape 닫기 보완 (#6635)` | collaborator 코드·회귀·MANIFEST 보정 |
| 이 문서를 포함하는 trailing commit | `mydocs/pr/archives/pr_6786_review.md`, `pr_6786_review_impl.md`만 포함 |

원 source는 `baba9811/rhwp:fix/6635-color-keyboard`다. local review branch는
`codex/pr6786-color-keyboard-review`, worktree는 `/private/tmp/rhwp-pr6786-review`다.
루트 checkout의 별도 #6788 작업은 checkout·stage·수정하지 않았다.

코드 push 전 변경 경로의 Git attribute가 LFS 대상이 아님을 확인하고, `git lfs status`, `git diff --check`,
remote SHA 확인과 `GIT_LFS_SKIP_PUSH=1 git push --dry-run`을 통과했다. 실제 push 후 GitHub PR head와
remote branch가 보정 SHA로 일치했다. 문서 push도 같은 검사를 적용한다. 다른 hook은 비활성화하지 않는다.

## 보정 내용과 검증 책임

- production 보정은 HTML input attribute 하나와 dropdown에 한정한 Escape 핸들러다.
- E2E는 양방향 Tab과 trigger/색 없음/다른 색 각각에서 Escape 닫기, 포커스·선택·서식·undo 보존을 검증한다.
- 보정 전 코드에 새 회귀를 실행해 실패를 확인한 뒤 보정을 복원하고 두 모드 모두 재실행했다.
- native picker 취소의 사용자 수동 확인은 원 head 결과다. 새 Escape 보정의 자동화 결과와 구분한다.
- 보정 코드·Rust 입력·workflow를 문서 commit에서 바꾸지 않는다. latest head가 바뀌면 새 diff에 따라
  검증 범위를 다시 판단하며 과거 CI 결과를 그대로 승계하지 않는다.
- 선택적 오늘할일 파일의 source/devel add/add 위험과 공개 가이드·전역 접근성 조사 분리 사유는 review에 남겼다.

## rollback과 미완료 원격 조치

보정 회귀가 발견되면 merge를 보류하고 이 보정만 수정하는 새 commit을 만든다. 보정을 철회해야 하면
`cc21183ab622d13071acfa7be3224903a048526b`의 변경을 revert하는 별도 commit으로 처리한다.
어떤 경우에도 contributor 두 commit을 rewrite하거나 source branch를 force-push하지 않는다.

Approve 리뷰 등록, merge, PR·issue comment와 close는 실행하지 않았다. 코드 CI와 문서 checks가 모두 성공한
경우에도 사용자의 최종 리뷰 초안 승인을 기다린다. merge 후 cleanup 때 contributor fork branch는 유지하고,
검토 전용 worktree·branch·서버·산출물만 범위를 확인해 정리한다.
