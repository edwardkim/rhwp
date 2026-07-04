# PR #1877 처리 계획 - #1665 Build & Test Native Skia 병렬 분리

## 대상

- PR: #1877
- 작성자: @postmelee
- 관련 이슈: #1665, #1668
- 처리 경로: collaborator self-merge 후보 예외 경로
- 코드 검증 기준 SHA: `34f714bcab6d9c2e975cce9c8ebf03a6fda77bf5`
- 처리 판단: 문서 후속 커밋 push 후 최신 PR head 기준 CI/fast-pass와 merge state를 재확인하고, 작업지시자
  승인 후 merge 여부를 결정한다.

## 커밋 구성

원 코드/작업 커밋:

| SHA | 제목 | 비고 |
|-----|------|------|
| `bdde0886f39a7bfe6bb78b261c135edfe7bdddc4` | `docs: plan #1665 build test parallel split` | 수행계획서 추가 |
| `f3597b0caf89ab3196e8eae6ad5b1fa37964f5f4` | `docs: add #1665 implementation plan` | 구현계획서 추가 |
| `34f714bcab6d9c2e975cce9c8ebf03a6fda77bf5` | `ci: split native skia test job` | workflow 구조 변경 및 Stage 1 보고서 |

후속 문서 커밋:

| 항목 | 내용 |
|------|------|
| 오늘할일 보강 | `mydocs/orders/20260704.md`의 #1665 진행 상태에 PR review 문서 작성과 merge 승인 대기 반영 |
| review 문서 | `mydocs/pr/archives/pr_1877_review.md` |
| 처리 계획 | `mydocs/pr/archives/pr_1877_review_impl.md` |
| 변경 범위 | `mydocs/**` 문서만 변경 |

후속 문서 커밋은 이 문서를 포함하므로 문서 안에 자기 자신의 최종 SHA를 고정 기록하지 않는다. Push 후
`gh pr view 1877 --json headRefOid,statusCheckRollup,mergeable,mergeStateStatus`로 최신값을 확인한다.

## Stage 1. PR 메타 정렬

완료.

- labels: `enhancement`, `ci`
- milestone: `v1.0.0`
- assignee: @postmelee
- review request: @edwardkim
- base: `devel`
- head: `postmelee:task-1665-build-test-parallel-plan`
- 문서 작성 시점 참고 상태: draft false, `MERGEABLE`, `CLEAN`

## Stage 2. 변경 내용 검토

완료.

검토한 핵심 변경:

- `.github/workflows/ci.yml`
  - `Build default-feature tests` worker 추가
  - `Native Skia tests` worker 추가
  - 기존 `Build & Test` 이름은 aggregate gate로 유지
  - aggregate gate가 preflight, default-feature worker, native-skia worker 결과를 확인

보존 확인:

- PR 경로 default-feature test 명령 유지
- Native Skia test 명령 유지
- `Build & Test` check 이름 유지
- cache key/path 유지
- PR restore-only 정책 유지
- trusted branch cache save writer는 `Build default-feature tests` 하나로 제한
- `tests/**`, `tests/golden_svg/**`, sample 변경 없음
- renderer/layout/paint 변경 없음

검토 중 확인한 보정 권고:

- PR 본문에는 build 명령이 `--no-default-features --features test-utils`로 적혀 있으나, 실제 workflow는 기존
  default-feature `cargo build --profile release-test --verbose`를 유지한다. Merge 전 PR 본문 보정이 필요하다.

## Stage 3. 로컬 정적 검증

완료.

- `ruby -e "require 'yaml'; YAML.load_file('.github/workflows/ci.yml'); puts 'yaml ok'"`: 통과
- `actionlint .github/workflows/ci.yml`: 통과
- `git diff --check upstream/devel...HEAD`: 통과
- `git diff --check`: 통과

YAML parse 중 로컬 Ruby `ffi` gem 확장 경고가 출력됐지만 파싱 결과는 `yaml ok`였다.

## Stage 4. GitHub Actions 확인

완료.

코드 검증 기준 SHA `34f714bcab6d9c2e975cce9c8ebf03a6fda77bf5` 기준:

- CI preflight: success
- Build default-feature tests: success
- Native Skia tests: success
- Build & Test: success
- WASM Build: skipped
- Render Diff preflight: success
- Canvas visual diff: success
- CodeQL preflight: success
- Analyze (python): success
- Analyze (javascript-typescript): success
- Analyze (rust): success
- CodeQL: success

CI 로그에서 확인한 핵심 결과:

- preflight: `fast_pass=false reason=no-trailing-review-only-commits`
- `Build default-feature tests`: `running 2081 tests`, `2075 passed; 0 failed; 6 ignored`
- `Native Skia tests`: `running 48 tests`, `48 passed; 0 failed; 0 ignored`
- aggregate `Build & Test`: `preflight=success`, `build_default=success`, `native_skia=success`
- cache restore: 두 worker 모두 동일 cargo cache key restore 성공
- PR cache save: `Build default-feature tests`의 save step skipped, `Native Skia tests`에는 save step 없음
- cache reservation/read-only/save failure 경고 없음

## Stage 5. 후속 문서 커밋

진행 계획:

1. `mydocs/orders/20260704.md`의 #1665 행을 PR review 문서 작성 및 merge 승인 대기 상태로 보강
2. `mydocs/pr/archives/pr_1877_review.md` 작성
3. `mydocs/pr/archives/pr_1877_review_impl.md` 작성
4. staging 범위가 위 세 파일로 제한됐는지 확인
5. 문서 커밋 생성
6. PR head branch `postmelee:task-1665-build-test-parallel-plan`로 push
7. PR diff에 review 문서 2건과 오늘할일 보강이 포함됐는지 확인

## Stage 6. 문서 커밋 push 후 확인

문서 커밋 push 후 다음을 확인한다.

- 최신 PR head SHA가 문서 커밋으로 변경됐는지
- PR diff에 다음 파일이 포함됐는지
  - `mydocs/pr/archives/pr_1877_review.md`
  - `mydocs/pr/archives/pr_1877_review_impl.md`
  - `mydocs/orders/20260704.md`
- 후속 문서 커밋이 `mydocs/**`만 변경하는 single-parent commit인지
- 직전 코드 검증 기준 SHA의 relevant checks가 success/skipped 상태인지
- latest PR head 기준 GitHub Actions 또는 후속 기록 fast-pass 결과가 merge 가능 상태인지

## Stage 7. PR 본문 보정

Merge 전 PR 본문의 `Build default-feature tests` 명령 설명을 실제 workflow와 맞춘다.

보정 대상:

- 현재 설명: `cargo build --profile release-test --no-default-features --features test-utils`
- 실제 workflow: `cargo build --profile release-test --verbose`

이 보정은 workflow 코드를 바꾸는 것이 아니라 PR 설명의 검증 명령 추적성을 맞추는 작업이다.

## Stage 8. merge 전 대기 조건

문서 커밋 push 후 즉시 merge하지 않는다. 다음 조건을 모두 확인한 뒤 작업지시자 승인으로 넘어간다.

- latest PR head 기준 GitHub Actions 또는 후속 기록 fast-pass 결과가 merge 가능 상태
- `mergeable` / `mergeStateStatus` 최신값 재확인
- PR diff에 review 문서와 오늘할일 보강이 포함됨
- PR 본문 명령 설명이 실제 workflow와 일치함
- 작업지시자 최종 merge 승인

## Stage 9. merge 후 후속 확인 계획

merge 후에는 다음을 확인한다.

- PR merge commit과 mergedAt
- `devel` push CI run에서 aggregate `Build & Test`, `Build default-feature tests`, `Native Skia tests` 결과
- trusted branch cache save writer가 하나뿐인지
- Native Skia job의 산출물이 별도 runner에서 생성된 뒤 저장되지 않는 한계가 실제 Native Skia compile 시간과
  runner-minutes에 미치는 영향
- Native Skia compile 시간이 계속 크면 별도 native-skia cache key 또는 별도 target dir 도입을 후속으로 검토
- #1665, #1668 상태
- #1665는 `Refs` 형식이라 auto-close되지 않는 것이 정상이다. merge 후 measurement/report 정리가 남으면 open
  유지한다.
- #1668은 tracking issue이므로 open 유지한다.
