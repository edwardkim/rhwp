---
kind: review_plan
status: local-validation-passed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4361 메인터너 보정 실행 기록

## commit 경계

| 역할 | SHA | 상태 |
| --- | --- | --- |
| contributor workspace v1 | `7d3c6db171102d901875339176a7d125b8a7a285` | 보존 |
| contributor annotations CI 수리 / 원 head | `c98a3f1101ba78d8cc4d87f3c6f906ab8cf632c1` | 보존 |
| 메인터너 profile·workspace 경계 보정 | `19fad2a44142b1c3f9c9dde300c6151d245080e8` | 완료 |
| 1차 review 및 implementation 기록 | `999627e741a841e61c82b253283880e66fafaff3` | 완료 |
| 메인터너 결정적 상한·nextest 보정 | `058acbe79aa5b45e32507a53a99aaac55d9d6fc3` | 완료 |
| 후속 review 및 implementation 기록 | 본 문서와 `pr_4361_review.md`의 trailing 문서 commit | 완료 후 SHA를 Git 이력에서 확인 |

가시성 branch는 `review/kevin9327-20260810-pr4361` 하나만 사용한다. 원 contributor commits 위에
메인터너 commit과 review 문서 commit을 선형으로 쌓으며 원 contributor·기존 메인터너 commit을 다시
작성하지 않는다.

## 실행 단계

1. **접수·기준 고정 — 완료.** `origin/pr/4361` head와 `origin/devel` 조상 관계를 확인하고 원 head에서
   가시성 branch/worktree를 만들었다. GitHub reviewer assign이나 comment는 수행하지 않았다.
2. **profile gate 보정 — 완료.** 중복된 session-tool match를 제거하고 `ALL_SESSION_TOOLS`를 직접 사용해
   신규 workspace 4종을 포함한 모든 세션 도구가 같은 `session_allows` gate를 거치게 했다.
3. **workspace scan 보정 — 완료.** 링크와 비정규 파일을 제외하고 canonical root containment 및 방문
   집합으로 root 이탈·중복·cycle을 차단했다. profile direct-call과 Unix link 회귀를 추가했다.
4. **로컬 검증 — 완료.** focused release-test 11건, 변경 파일 rustfmt, diff 검사를 실행했다. Windows에서
   Unix 전용 회귀가 실행되지 않은 사실을 [검토 기록](pr_4361_review.md)에 제한 조건으로 남겼다.
5. **후속 결정성·test runner 보정 — 완료.** 모든 신규/수정 CLI test를 nextest runtime binary helper로
   통일하고, filesystem 발견 순서와 무관하게 경로순 최소 10,000개만 보존하는 bounded max-heap을 넣었다.
   root 상한 뒤 더 작은 중첩 경로가 나오는 10,001파일 회귀를 추가했다.
6. **후속 로컬 검증·기록 — 완료.** focused release-test 12건(profile 8, workspace 4), rustfmt, diff check를
   통과했다. source/test와 섞지 않고 이 두 문서를 trailing commit으로 만든다.
7. **원격 반영 — 승인 대기.** 작업지시자가 push 대상과 방식(원 PR branch update 또는 별도 maintainer
   integration branch)을 정한 뒤에만 수행한다. 현재 작업에는 push 권한 행사가 포함되지 않는다.
8. **CI와 merge — 미착수.** 새 원격 head의 required checks와 Linux symlink 계약 성공을 확인한 뒤에도
   별도의 명시적 merge 승인이 있어야 한다. 승인 전에는 review 제출, merge, close 및 contributor comment를
   수행하지 않는다.

## rollback

- 후속 기록만 철회할 때는 trailing 문서 commit만 revert한다. source/test 보정과 섞지 않는다.
- 전체 메인터너 보정을 철회할 때는 trailing 문서, `058acbe7`, `999627e7`, `19fad2a4`를 이력 역순으로
  revert한다. 그러면 branch는 원 contributor head `c98a3f11...`의 내용으로 돌아간다.
- contributor의 두 commit은 reset, amend, rebase 또는 squash하지 않는다.
- 원격 반영 전이므로 현재 rollback은 로컬 branch에서만 수행하며 GitHub 상태에는 영향이 없다.

## 권한 경계

이 기록은 merge 허가가 아니다. push, GitHub review/comment, PR 갱신, merge, close는 각각 작업지시자의
명시적 승인과 최신 상태 재확인이 필요한 별도 단계다.
