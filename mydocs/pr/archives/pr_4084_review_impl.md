---
kind: pr_review_implementation
status: review-record-pending
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4084 메인터너 보정 실행 기록

## 범위와 commit

| 순서 | SHA | 주체 | 내용 |
| --- | --- | --- | --- |
| 1 | `ae7e4db3e` | contributor | #4054 이른 분할 보고서와 초기 `pi_map_hash.py` 추가 |
| 2 | `607e2fb73` | maintainer | 실패 은닉 차단, 전체 경로 key, 원자적 TSV 교체, Python 회귀 시험 |
| 3 | `90c76c10e` | maintainer | 최신 `devel` 기준선의 자동 3-way current-base bridge |
| 4 | 이 commit | maintainer | bridge 뒤 archive review·오늘할일 trailing 기록 갱신 |

contributor 원 commit은 rewrite·amend·rebase하지 않았다. `maintainerCanModify=true`인 같은 source branch에
2를 single-parent 보정으로 쌓고, 3은 최신 base를 정확히 한 부모로 둔 허용된 bridge merge이며, 4를
그 뒤의 single-parent review-only 기록으로 쌓는다.

## 실행 단계

1. contributor head `ae7e4db3e`를 `review/planet6897-4084-20260806`에 fetch하고 최신
   `upstream/devel` `3f7d87542`와 merge simulation을 수행했다.
2. 실패한 `dump-pages` 결과를 empty TSV 행으로 남기지 않고, 실패 시 기존 TSV를 보존하도록 도구를
   보정했다. 동명 파일 충돌을 막기 위해 TSV key를 원래 경로로 바꾸고 회귀 4건을 추가했다.
3. 보정 head `607e2fb73`를 contributor fork source에 push했다. source SHA, LFS filter 미지정 상태,
   `GIT_LFS_SKIP_PUSH=1` dry-run과 실제 push를 확인했다.
4. code head의 CI·CodeQL 성공 뒤 review·오늘할일을 기준선 bridge보다 먼저 넣으면 같은 날짜 hunk가
   3-way conflict가 됨을 simulation으로 확인했다. 따라서 최신 `devel`을 정확한 한 부모로 둔
   `90c76c10e` bridge에서 code head와 upstream tree를 자동 3-way로 먼저 합쳤다. review·오늘할일은
   이 bridge 뒤 trailing commit에서 추가한다.
5. bridge 뒤 이 archive review·오늘할일 문서 commit만 source에 push한다. current head preflight가
   허용된 current-base bridge와 code candidate를 재사용하고 latest `Build & Test` aggregate가 성공하면
   merge한다. fast-pass가 실패하면 문서만 재수정하지 않고 원인을 판정한다.
6. merge 뒤 `devel` 포함 여부와 #4054의 기존 closed 상태를 확인하고, contributor 감사 comment를 남긴
   뒤 review 및 merge-simulation local branch만 정리한다. contributor fork source branch는 삭제하지 않는다.
