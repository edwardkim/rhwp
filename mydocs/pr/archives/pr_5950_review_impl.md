---
kind: pr-review-implementation
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5950 correction·self-review 구현 기록

## 기준과 commit 계보

| 단계 | commit | 판정 |
| --- | --- | --- |
| 최초 승인·PR head | `4a7c0f431` | unit-tier가 신규 source test support 6개를 거부 |
| collaborator 보정 | `745660467` | 원격 이력에 보존, 제품 source 안 수기 oracle은 여전히 부적합 |
| 근본 정정 | `fc2194b2c` | source oracle 제거, 공개 API integration 2건으로 이동 |
| W7-R4 기록 | `bbfd3ad6d` | 전체 로컬 재검증·본문 정정 뒤 Full CI 통과 |
| self-review tail | 이 문서와 `pr_5950_review.md`를 포함할 후속 commit | `mydocs/` 한정 fast-pass 대상 |

base는 `upstream/devel@343ed2c013606319b6418dd8c637c5e04047e304`이며 code candidate가 이를 이미
포함한다. 최신 base와의 merge simulation은 충돌 없이 통과했다.

## correction stage

1. 최초 CI 오류를 lint nuisance가 아니라 이중 authority 탐지로 귀속했다.
2. W1 역사 baseline, W3 current-source 계측, W6 metric lineage와 W7 runtime authority를 분리했다.
3. collaborator commit은 재작성하지 않고 부모로 보존했다.
4. 최종 tree에서는 신규 source helper·unit test를 제거하고 `tests/cases/` 원본만 남겼다.
5. prepared review worktree에서 generated harness를 만들고 integration·fmt·전체 회귀를 검증했으며,
   파생 suite·manifest·Cargo target은 source PR에 포함하지 않았다.
6. PR 본문의 integration 경계와 4,221·87/87·8,200 수치를 현재 head에 맞게 고치고 Draft를 복원했다.

## 검증·rollback 경계

- code candidate `bbfd3ad6d`는 Full CI와 별도 required workflow가 모두 녹색이다.
- 현재 후속 commit은 archive review·implementation 기록과 오늘할일만 변경한다. source, test, fixture,
  workflow, baseline과 asset은 바꾸지 않는다.
- trailing fast-pass가 candidate SHA·same repository·single-parent·허용 경로를 증명하지 못하면 Full CI로
  fallback하고 그 결과를 기다린다.
- 추가 code 보정이 필요해지면 review-only tail 위에 섞지 않고 새 code commit으로 분리해 전체 gate를
  다시 실행한다.
- `fc2194b2c`만 revert하면 부적합한 collaborator oracle이 되살아나므로 부분 rollback은 허용하지 않는다.
  정정이 수용되지 않으면 PR을 merge하지 않고 #4966 계획 단계로 되돌린다.

## 남은 승인 게이트

1. self-review trailing commit의 원격 push 승인과 fast-pass 확인
2. 최신 head의 required aggregate와 `MERGEABLE/CLEAN` 재확인
3. 메인테이너의 Draft 해제 승인
4. 메인테이너의 정상 merge commit 방식 병합 승인
5. merge 뒤 #4966 close, parent #4960 W7 상태, devel sync와 branch 정리
