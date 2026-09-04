# 구현계획서 — #3309 공통 green candidate 탐색

## candidate 구간

PR commit을 최신순으로 읽는다. 허용된 trailing review-only single-parent commit은 후보 목록에 넣는다.
그 뒤 current base를 parent로 포함한 2-parent review-only Update branch merge가 나오면 그것도 후보에 넣고
중단한다. 다른 merge 형태는 기존처럼 full CI fallback이다. review-only 구간 뒤 비문서 commit이 있으면
그 commit도 종전 호환 candidate로 목록 끝에 넣는다.

candidate 목록은 최신순이다. 각 SHA에 대해 다음을 확인한다.

| 조건 | 처리 |
| --- | --- |
| current base가 candidate의 ancestor | 해당 SHA의 기존 PR workflow/check 조회 |
| workflow/check가 아직 없거나 진행 중 | 더 이전 candidate를 계속 탐색 |
| 가장 최근 완료 candidate가 실패 | full CI fallback |
| green aggregate 및 workflow identity 충족 | fast-pass candidate로 채택 |
| 허용 경로 밖 변경·base 불일치·비허용 merge | 즉시 full CI fallback |

이 규칙으로 #3304의 `bcff621`은 `2042ee0`보다 한 단계 이전이지만, 이후 변경이 review-only이고
current base를 포함한 가장 최근 green head이므로 재사용된다. 반면 code 변경, stale base, 실패한 required
check는 우회하지 않는다.

## 파일별 변경

- `.github/workflows/ci.yml`: Build & Test check와 fork fallback workflow job을 후보별로 탐색한다.
- `.github/workflows/codeql.yml`: 세 analysis check를 같은 후보별 규칙으로 검증한다.
- `.github/workflows/render-diff.yml`: Canvas visual diff, PR/head/base identity를 후보별로 검증한다.
- `mydocs/manual/pr_review/review_only_fast_pass.md`: 일반 green PR head 재사용 및 fallback 조건을 갱신한다.
- `mydocs/manual/pr_review/multi_pr_update_branch.md`: stale run 확인 직후 force-cancel API를 쓰도록 기본 절차를
  명확히 한다.

세 workflow의 허용 경로·merge topology·candidate 순서는 동일하게 유지한다. workflow inline JavaScript를
각 파일의 기존 verification 방식에 맞게 최소한으로 확장한다.
