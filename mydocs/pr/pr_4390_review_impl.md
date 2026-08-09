---
kind: review-implementation
status: completed-local-pending-push-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4390 메인터너 보정 구현·검토 계획

## history와 변경 경계

| 순서 | commit | 소유 | 상태·내용 |
| --- | --- | --- | --- |
| 1 | `8aa039997d8f8bc1444c0ab42d1efb99ee666ac1` | contributor | harness scorecard와 6-proof runner 추가 |
| 2 | `02ce6d0b3840acb5c1b2569ee9154dc32832c789` | contributor | 미병합 문서 링크 정정, 원 PR source head |
| 3 | `48a80dc09a9a876165292777546ec124f195a82a` | maintainer | 정확한 68-command 판정과 회귀 테스트 |
| 4 | `7d73d692471d46c0e60f206b8a820129332069bf` | maintainer | 1차 active review·실행 기록 |
| 5 | `bc93618d678d29522ab22873322b2962a9eb736b` | maintainer | 명령·계약·출처 표지 의미 검증과 음성 회귀 |
| 6 | 이 문서 commit | maintainer | 후속 검증·잔여 위험 기록; runner 동작 변경 없음 |

가시성 branch `review/kevin9327-20260810-pr4390`은 정확한 source head에서 시작했다. 메인터너
보정은 `tools/harness_proofs.py`와 신규 `tools/test_harness_proofs.py`에 한정되며 contributor와 기존
메인터너 commit은 재작성하지 않았다.

## 단계

### Stage 1 — source 고정과 차단점 확인 (완료)

- `origin/pr/4390`을 `02ce6d0b3840acb5c1b2569ee9154dc32832c789`로 고정했다.
- `origin/devel` `e48fe86947fbf9a44b1b98c7037150751af541ab`이 source의 조상임을 확인했다.
- scorecard의 정확한 68개 주장과 runner의 `>= 50` 판정을 대조해 false-PASS 범위를 확인했다.

### Stage 2 — 메인터너 code·test 보정 (완료)

- exact command count와 계약 필드 판정을 순수 helper로 분리했다.
- 성공·과소·과다·필드 누락을 검증하는 표준 라이브러리 `unittest` 3건을 추가했다.
- `48a80dc09a9a876165292777546ec124f195a82a`로 별도 commit해 contributor attribution을 보존했다.
- 후속 검토에서 확인한 duplicate/null false-PASS와 P5 키 존재 false-PASS를 순수 helper 의미 검증으로
  닫고, 성공·음성 회귀 8건을 `bc93618d678d29522ab22873322b2962a9eb736b`로 추가했다.

### Stage 3 — focused 검증과 active 기록 (완료)

- Python unit test 8건, py_compile, 실제 harness proof 6건, correction diff check를 통과했다.
- 실물 P2가 고유 이름 68개와 핵심 exit/JSON 계약을, P5가 `true`와 실제 비어 있지 않은 경로를
  출력함을 확인했다.
- `pr_4390_review.md`와 이 구현 기록을 code·test commit 뒤의 별도 문서 commit으로 남긴다.
- 렌더 경로 변경이 없어 시각 검증은 선택하지 않았다.

### Stage 4 — 원격 후보 갱신과 CI (승인 대기)

1. 작업지시자가 별도로 push를 승인한다.
2. push 직전 GitHub PR head·contributor source branch SHA·local source 시작 SHA의 일치를 확인한다.
3. LFS 대상 사전 판독과 dry-run을 거쳐 maintainer code·test·문서 commit을 contributor source branch에
   선형으로 push한다.
4. code·test 보정이 포함됐으므로 review-only fast-pass를 쓰지 않고 최신 head의 required checks를 기다린다.
5. CI가 신규 Python test를 자동 실행하지 않으면 focused unittest와 harness proof를 merge 근거에 명시한다.

### Stage 5 — review·merge·후속 처리 (별도 승인 대기)

- 최신 head의 required checks와 mergeability를 다시 확인한다.
- 작업지시자 승인 뒤에만 GitHub review/comment와 merge를 수행한다.
- merge 뒤 기록 archive, devel 반영, 관련 이슈 상태와 branch/worktree 정리는 merge 후속 절차를 따른다.

## rollback

- 아직 push 전이므로 원격 contributor history에는 영향이 없다.
- 보정 방향을 취소해도 contributor나 기존 메인터너 commit을 amend·rebase하지 않는다. 가시성 branch에서
  trailing 문서, `bc93618d`, `7d73d692`, `48a80dc0`을 이력 역순의 새 `git revert` commit으로 되돌리거나,
  작업지시자 승인 뒤 로컬 branch를 폐기한다.
- push 뒤 문제가 발견되면 force-push나 history rewrite 대신 correction·문서 commit을 역순으로
  revert하고 최신 CI를 다시 받는다.

이 계획은 로컬 보정과 검증만 승인된 상태를 기록한다. push, GitHub review/comment, merge, close 권한을
포함하지 않는다.
