# PR #1702 리뷰 문서

## 개요

| 항목 | 내용 |
|------|------|
| PR | #1702 `Task #1664: cargo cache save를 trusted branch로 제한` |
| 작성자 | @postmelee |
| 경로 | collaborator self-merge 후보 |
| base | `devel` |
| head | `postmelee:task1664-ci-cache-workflow` |
| 문서 작성 시점 참고 head | `924b9b4e80fe46a663d4af0e66542144dee78517` |
| 관련 이슈 | #1664, #1668 |
| labels | 문서 작성 시점 참고값: 없음 |
| milestone | 문서 작성 시점 참고값: 없음 |

이 PR은 외부 contributor PR이 아니라 collaborator 본인 PR이다. 따라서
`mydocs/manual/pr_review_workflow.md` 8장 collaborator self-merge 후보 예외 경로를 적용한다.

## 변경 범위

PR #1702는 `.github/workflows/ci.yml` 단일 파일 변경이다.

- `CI / Build & Test` job의 cargo cache step을 `actions/cache@v5` 단일 step에서
  `actions/cache/restore@v5` + `actions/cache/save@v5` 분리 구조로 변경한다.
- restore는 모든 run에서 유지한다.
- save는 `push` 이벤트 중 `refs/heads/devel` 또는 `refs/heads/main`에서만 허용한다.
- exact cache hit이면 save를 다시 시도하지 않는다.

범위 밖 항목:

- `Cargo.toml`, `tests/`, `tests/golden_svg/` 변경 없음
- profile 변경 없음
- job 병렬화 없음
- third-party Rust cache action 도입 없음
- CodeQL Rust analyze workflow의 `actions/cache@v5` cache step 변경 없음

## 범위 명확화 반영

1차 검토에서 "PR cache save 차단" 표현이 CodeQL Rust analyze cache까지 포함하는 것처럼 읽힐 수 있음을
확인했다.

PR body와 #1664 이슈 코멘트에서 다음 경계가 보강되었다.

- #1702의 restore-only 적용 범위는 `CI / Build & Test` job의 cargo registry/build cache다.
- CodeQL Rust analyze workflow의 cache step은 이번 PR에서 변경하지 않는다.
- CodeQL cache 정책은 #1667 또는 별도 후속 PR에서 measurement 기준을 분리해 검토한다.

수정 후 동일한 blocking finding은 남지 않았다.

## 검증

| 항목 | 결과 | 비고 |
|------|------|------|
| PR head 동기화 | 통과 | `task1664-ci-cache-workflow`를 `origin/task1664-ci-cache-workflow` 최신 head로 fast-forward |
| 변경 범위 | 통과 | `upstream/devel...HEAD` 기준 `.github/workflows/ci.yml` 1개 파일 |
| `git diff --check upstream/devel...HEAD` | 통과 | whitespace 문제 없음 |
| `actionlint .github/workflows/ci.yml` | 통과 | workflow 문법 및 expression lint 오류 없음 |
| PR body 범위 명확화 | 통과 | `Build & Test` cargo cache 기준임을 명시 |
| #1664 이슈 코멘트 보강 | 통과 | CodeQL cache 범위 제외 및 후속 검토 경계 명시 |
| GitHub Actions | 통과 | 문서 작성 시점 참고값: head `924b9b4e` 기준 `Build & Test`, CodeQL 계열 성공 |

문서 작성 시점 참고 GitHub Actions:

- `CI preflight`: success
- `CodeQL preflight`: success
- `Build & Test`: success
- `Analyze (javascript-typescript)`: success
- `Analyze (python)`: success
- `Analyze (rust)`: success
- `CodeQL`: success
- `WASM Build`: skipped

## 측정 확인

#1664 코멘트 `issuecomment-4850424031`에 before/after 측정 보고가 보강되었다.

핵심 확인값:

- PR run에서 `Save cargo registry & build cache` step은 `skipped`
- `Restore cargo registry & build cache`는 유지
- latest after run `28493187002` 기준 read-only / reservation / save 실패 경고 없음
- `Build & Test`, build, native-skia, lib test, integration test, clippy step 실행
- `tests/*.rs` 162개 대응과 issue 계열 회귀 가드 131/131 확인
- P50/P90은 표본 수와 exact-hit/fallback 조건 혼재로 공식 산출 보류

## 리스크 및 후속

- #1664의 1차 목표는 `Build & Test` cargo cache 기준 PR save 차단과 trusted branch save 정책 정착이다.
- fallback cache 조건에서는 `rhwp` crate compile 비용이 계속 발생한다. 이는 #1666 profile 검토와 #1667 cache
  전략 비교의 후속 기준선으로 넘기는 것이 타당하다.
- merge 후 `devel` push run에서 trusted branch save 경로를 확인해야 한다. exact cache hit이면 push run에서도
  save가 skipped 될 수 있으며, 이 경우는 실패가 아니다.
- PR metadata는 문서 작성 시점에 labels/milestone/assignee가 비어 있다. #1664 이슈와 맞춰 `ci`,
  `enhancement`, `v1.0.0`, @postmelee 정렬이 가능하지만, 이 문서 커밋에서는 GitHub metadata 상태 변경을
  수행하지 않는다.

## 판단

현재 PR #1702는 #1664에서 합의한 코드 PR 범위를 지키고 있으며, 기존 review open question이던 CodeQL cache
범위도 PR body와 #1664 코멘트에서 명확히 분리되었다.

권고:

- review 문서 push 후 PR diff에 `mydocs/pr/archives/pr_1702_review.md`와
  `mydocs/pr/archives/pr_1702_review_impl.md` 포함 여부 확인
- 문서 전용 후속 커밋 기준 fast-pass 또는 latest checks 상태 확인
- merge 직전 최신 head SHA, mergeability, checks 상태 재확인
- 작업지시자 승인 후 merge 가능
- #1664 이슈 close는 PR #1702 merge 및 `devel` push run 확인 후 별도 승인으로 판단
