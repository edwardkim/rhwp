---
kind: review-implementation
status: completed-local-pending-push-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4330 메인터너 보정 구현·검토 계획

## history와 변경 경계

| 순서 | commit | 소유 | 상태·내용 |
| --- | --- | --- | --- |
| 1 | `240fab24c333e66e6b0e709556464182cb47011f` | contributor | schema registry 구현·정책·계약 테스트 |
| 2 | `b54615026b97187050851c4b00c127e48911be64` | contributor | 지식 지도 정렬, 원 PR source head |
| 3 | `3f60b28f685c66366d855f849e8d67e689b06387` | maintainer | runtime `CARGO_BIN_EXE_rhwp` 우선 보정 |
| 4 | `cc5cf2a49edd6308bac2c8e42fb5a1c653c33b9c` | maintainer | 1차 active review·실행 기록 |
| 5 | `e95fa010688a346f9299b757083a37fd2f9e7294` | maintainer | `schemaVersion` insert 우회 3곳과 scanner 사각지대 보정 |
| 6 | 이 문서 commit | maintainer | 후속 finding·검증·rollback 기록; code 동작 변경 없음 |

가시성 branch `review/kevin9327-20260810-pr4330`은 정확한 source head에서 시작했다. 메인터너
code 보정은 contributor source 뒤의 별도 single-parent commit으로만 추가했고 contributor commit은
재작성하지 않았다.

## 단계

### Stage 1 — source 고정과 차단점 확인 (완료)

- `origin/pr/4330`을 `b54615026b97187050851c4b00c127e48911be64`로 고정했다.
- `origin/devel` `e48fe86947fbf9a44b1b98c7037150751af541ab`이 source의 조상임을 확인했다.
- PR 신규 CLI 테스트 전수를 확인해 compile 시점 실행 파일 직접 사용이 한 파일에 있음을 확인했다.

### Stage 2 — 메인터너 code 보정 (완료)

- 런타임 `CARGO_BIN_EXE_rhwp`를 우선하는 `rhwp_bin()` helper를 추가했다.
- compile 시점 `env!`는 일반 Cargo 실행을 위한 fallback으로만 유지했다.
- `3f60b28f685c66366d855f849e8d67e689b06387`로 별도 commit해 contributor attribution을 보존했다.
- 통합 후보 대사에서 객체 리터럴만 잡던 scanner가 `Map::insert` 버전 리터럴 세 곳을 놓치는 것을
  확인했다. 세 생성부를 `ENVELOPE_SCHEMA_VERSION`에서 파생하고, multiline insert와 인덱스 대입까지
  statement 단위로 탐지하는 mutation 회귀를 추가했다.
- `e95fa010688a346f9299b757083a37fd2f9e7294`로 두 번째 code 보정을 분리했다.

### Stage 3 — focused 검증과 active 기록 (완료)

- 강화된 schema registry 통합 테스트 5건, agent manifest·JSONL summary 실행 회귀 각 1건,
  변경 Rust 파일 rustfmt, correction diff check를 통과했다.
- `pr_4330_review.md`와 이 구현 기록을 code commit 뒤의 별도 문서 commit으로 남긴다.
- 렌더 경로 변경이 없어 시각 검증은 선택하지 않았다.

### Stage 4 — 원격 후보 갱신과 CI (승인 대기)

1. 작업지시자가 별도로 push를 승인한다.
2. push 직전 GitHub PR head·contributor source branch SHA·local source 시작 SHA의 일치를 확인한다.
3. LFS 대상 사전 판독과 dry-run을 거쳐 maintainer code·문서 commit을 contributor source branch에
   선형으로 push한다.
4. code·test 보정이 포함됐으므로 review-only fast-pass를 쓰지 않고 최신 head Full CI를 기다린다.

### Stage 5 — review·merge·후속 처리 (별도 승인 대기)

- 최신 head의 required checks와 mergeability를 다시 확인한다.
- 작업지시자 승인 뒤에만 GitHub review/comment와 merge를 수행한다.
- merge 뒤 기록 archive, devel 반영, 관련 이슈 상태와 branch/worktree 정리는 merge 후속 절차를 따른다.

## rollback

- 아직 push 전이므로 원격 contributor history에는 영향이 없다.
- 보정 방향을 취소해도 contributor commit을 amend·rebase하지 않는다. 가시성 branch에서 최신 문서
  commit → `e95fa010` → `cc5cf2a4` → `3f60b28f` 순서로 새 `git revert` commit을 만들거나,
  작업지시자 승인 뒤 로컬 branch를 폐기한다.
- push 뒤 문제가 발견되면 force-push나 history rewrite 대신 correction·문서 commit을 역순으로
  revert하고 최신 CI를 다시 받는다.

이 계획은 로컬 보정과 검증만 승인된 상태를 기록한다. push, GitHub review/comment, merge, close 권한을
포함하지 않는다.
