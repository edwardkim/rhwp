---
kind: pr_review_implementation
status: maintainer-correction-ready-for-push
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4187 메인터너 보정 이행 기록

## 고정 기준

- contributor source head: `84b0aae11f33c220a0e172a6eddd120cc064f713`
- collaborator visibility branch: `review/planet6897-20260809-r2`
- collaborator code/test/doc correction: `9ffc8009594315299349a1e53568e377038c5238`
- 문서 작성 시점 `upstream/devel`: `828eabc19a4953a684e05d523a614256dae28b26`
- 현재 source head는 force-update 전의 이전 보정을 포함하지 않았으므로, contributor commit을 재작성하지
  않고 그 뒤에 collaborator commit만 추가한다.

## 완료한 단계

1. contributor source head와 PR head, source repository, `maintainerCanModify=true`를 확인했다.
2. contributor source와 최신 `upstream/devel`의 merge tree를 계산해 충돌 없이 병합됨을 확인했다.
3. 이전에 누락된 package export 계약과 cross-platform harness 보정을 source head 위에서 복원했다.
4. package 소비 계약, Python harness 계약, 원장 검사와 Linux WASM self-check 44개 시나리오를 실행해
   통과했다.
5. `mydocs/orders/20260809.md`는 source에 없는 반면 최신 devel에 unrelated 기록으로 존재하므로 추가하지
   않았다. 이 archive 검토 기록 두 파일만 trailing commit으로 추가한다.

## commit 분리

| 순서 | commit | 역할 |
| --- | --- | --- |
| 1 | `84b0aae11` | contributor 원 P1~P3 구현과 44개 시나리오 |
| 2 | `9ffc80095` | collaborator cross-platform 및 package contract 보정 |
| 3 | 이 문서 commit | 검토 결과와 이행 절차 기록 |

## 남은 순서

1. 문서 commit 후 contributor 원격 ref, PR head SHA, local source 기준 SHA가 모두 `84b0aae`인지 재확인한다.
2. LFS 대상 여부를 판독한다. 이번 변경이 LFS 대상이 아니면 `GIT_LFS_SKIP_PUSH=1` dry-run을 수행한다.
3. 작업지시자의 push 승인 뒤 `planet6897/pr/devel-hwpctrl-p1p3`에 code 보정과 review 기록을 push한다.
4. 원격 head가 local HEAD와 일치하는지 확인하고, code/test 보정이 포함됐으므로 fast-pass 없이 최신 Full CI,
   CodeQL, Render Diff 및 mergeable 상태를 확인한다.
5. 작업지시자의 merge 승인 뒤 merge SHA를 확인한다. PR 본문의 관련 참조는
   [#4178](https://github.com/edwardkim/rhwp/pull/4178)뿐이고 close 키워드는 없으므로, 이 PR을 근거로
   issue close를 수행하지 않는다.
6. merge 후 `upstream/devel` 동기화, source branch/visibility worktree와 review 전용 target 정리를 수행한다.

## rollback

원 contributor commit은 수정하지 않는다. 보정이 원인인 회귀가 확인되면 `9ffc80095`와 이 문서 commit만
`git revert`하여 contributor source `84b0aae`로 되돌린다. contributor branch의 rebase, amend, reset,
force-push는 사용하지 않는다.
