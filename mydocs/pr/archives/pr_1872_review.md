# PR #1872 리뷰 - #1667 Build & Test 기본 test step 정리

## 메타

| 항목 | 내용 |
|------|------|
| PR | #1872 |
| 제목 | `Task #1667: Build & Test 기본 test step 정리` |
| 작성자 | @postmelee |
| base | `devel` |
| head | `postmelee:task-1667-dirty-rhwp-analysis` |
| 검토 경로 | collaborator self-merge 후보 예외 경로 |
| 관련 이슈 | #1667, #1668 |
| labels | `ci`, `enhancement` |
| milestone | `v1.0.0` |
| assignee | @postmelee |
| review request | 없음. PR 작성자와 현재 reviewer가 같아 self-review request는 적용하지 않는다. |
| 코드 검증 기준 SHA | `3f987ecf5e3aa35c1fc7bab4bca093e0b23f6c52` |
| 규모 | 문서 작성 시점 참고값: 9 files, +782 / -24 |
| 상태 | 문서 작성 시점 참고값: draft false, `MERGEABLE`, `CLEAN` |

이 문서는 merge 완료 보고서가 아니라 self-merge 후보의 merge 전 검토 기록이다. 이 문서와
`pr_1872_review_impl.md`, 오늘할일 정리는 후속 `mydocs/**` 문서 커밋으로 PR head에 포함한다. 따라서
문서 커밋 push 후 최신 PR head SHA는 위 코드 검증 기준 SHA와 달라진다.

## 변경 요약

Build & Test job의 기본 feature test step을 하나로 정리한다.

- 기존 `Run integration tests`의 명령인 `cargo test --tests`는 유지한다.
- step 이름을 실제 역할에 맞춰 `Run default-feature tests`로 바꾼다.
- 별도 `Run lib tests`의 `cargo test --lib` step을 제거한다.
- `Native Skia tests`는 `native-skia skia` feature set이 다르므로 유지한다.
- cache action, cache key/path, branch protection, required check 이름은 변경하지 않는다.

핵심 해석은 현재 저장소 Cargo target 구성에서 `cargo test --tests`가 integration test executable과
기본 feature `rhwp` lib test executable을 함께 포함한다는 것이다. 따라서 `Run lib tests`는 coverage를
추가하지 않고 같은 lib test harness를 다시 실행하는 중복 step으로 판단했다.

## 변경 범위

코드/CI 변경:

- `.github/workflows/ci.yml`
  - `Run default-feature tests` 추가
  - `Run lib tests` 제거
  - `Run integration tests` 이름 제거
  - `cargo test --profile release-test --tests --verbose` / `cargo test --release --tests --verbose` 명령 유지

운영 문서:

- `mydocs/orders/20260703.md`
- `mydocs/orders/20260704.md`
- `mydocs/plans/task_m100_1667_v3.md`
- `mydocs/plans/task_m100_1667_impl_v3.md`
- `mydocs/plans/task_m100_1667_impl_v4.md`
- `mydocs/working/task_m100_1667_stage3.md`
- `mydocs/working/task_m100_1667_stage4.md`
- `mydocs/working/task_m100_1667_stage5.md`

후속 문서 커밋에서 추가/정리한 운영 기록:

- `mydocs/orders/20260704.md`
  - #1873 merge 완료 기록을 `PR 처리` 표에 두고, `공통 - 운영 작업` 표에는 #1667 행만 남긴다.
- `mydocs/pr/archives/pr_1872_review.md`
- `mydocs/pr/archives/pr_1872_review_impl.md`

## 관련 이슈

- #1667 `[CI] Rust cache 전략 개선: actions/cache 유지 vs Swatinem/rust-cache 검토`
  - 상태: open (문서 작성 시점 참고값)
  - labels: `enhancement`, `ci`
  - milestone: `v1.0.0`
- #1668 `[Tracking/RFC] CI Build & Test 실행 시간 단계별 단축`
  - 상태: open (문서 작성 시점 참고값)
  - labels: `enhancement`, `discussions`, `ci`
  - milestone: `v1.0.0`

PR 본문은 `Refs #1667`, `Refs #1668` 형식이다. Merge만으로 issue auto-close가 발생하지 않는다. #1667은
후속 측정값과 최종 cache 전략 정리 여부에 따라 close 판단이 필요하고, #1668은 tracking issue이므로 후속
sub-issue 진행 상황에 맞춰 open 유지 여부를 별도로 판단한다.

## 로컬 검증

코드 검증 기준 SHA `3f987ecf5e3aa35c1fc7bab4bca093e0b23f6c52` 기준으로 확인했다.

```bash
actionlint .github/workflows/ci.yml
ruby -e "require 'yaml'; YAML.load_file('.github/workflows/ci.yml'); puts 'yaml ok'"
git diff --check upstream/devel...HEAD
cargo test --profile release-test --tests --no-run --message-format=json
cargo test --profile release-test --lib --no-run --message-format=json
git merge-tree --write-tree upstream/devel HEAD
```

결과:

- `actionlint .github/workflows/ci.yml`: 통과
- YAML parse: `yaml ok` 확인. 로컬 Ruby `ffi` gem 확장 경고는 YAML 파싱 실패가 아니었다.
- `git diff --check upstream/devel...HEAD`: 통과
- merge simulation: 충돌 없이 tree 생성
- `--tests --no-run`: executable artifact 187개
  - `test` 182개
  - `bin` 4개
  - `cdylib+rlib` 1개 (`rhwp`)
- `--lib --no-run`: executable artifact 1개 (`rhwp` `cdylib+rlib`)
- `--lib`에만 존재하는 고유 target: 0개

따라서 현재 저장소 구조에서는 `cargo test --tests`가 기본 feature lib test executable을 포함하고,
`cargo test --lib` 별도 step은 고유 실행 target을 추가하지 않는 것으로 확인했다.

## GitHub Actions

코드 검증 기준 SHA `3f987ecf5e3aa35c1fc7bab4bca093e0b23f6c52` 기준 relevant checks:

- CI preflight: success
- Build & Test: success
- WASM Build: skipped
- Render Diff preflight: success
- Canvas visual diff: success
- CodeQL preflight: success
- Analyze (python): success
- Analyze (javascript-typescript): success
- Analyze (rust): success
- CodeQL: success

`Build & Test` job log 확인:

- `Run default-feature tests` step 실행 확인
- `Run lib tests` 문자열 0회
- `Run integration tests` 문자열 0회
- `cargo test --profile release-test --tests --verbose` 1회
- `running 2081 tests` 1회
- `Native Skia tests`는 별도 step으로 유지, `running 48 tests` 확인

후속 문서 커밋은 `mydocs/**`만 변경한다. 직전 코드 검증 기준 SHA의 relevant checks가 success/skipped 상태이므로,
문서 커밋 push 후에는 후속 기록 fast-pass 조건을 기대할 수 있다. 다만 branch protection에서 pending/failing
check가 남으면 merge하지 않고 최신 PR head 기준 상태를 재확인해야 한다.

## 시각 검증

해당 없음.

이 PR은 CI workflow와 운영 문서 변경이며 renderer, layout, paint, sample, golden fixture를 변경하지 않는다.

## 리뷰 결과

Blocking finding 없음.

`Run default-feature tests`는 기존 `Run integration tests`의 실질 명령을 유지하면서 기본 feature lib test
harness까지 한 번에 실행하는 이름으로 정리됐다. 로컬 Cargo target 비교와 GitHub Actions 로그 모두 이 판단을
뒷받침한다.

초기 검토 중 발견한 문서 정리 사항은 후속 문서 커밋에서 처리했다.

- `mydocs/orders/20260704.md`의 #1873 merge 완료 기록을 `PR 처리` 표로 이동
- `공통 - 운영 작업` 표에는 #1667 진행 행만 유지

## merge 전 재확인 조건

merge 전에는 다음을 다시 확인한다.

- 문서 후속 커밋 push 후 최신 PR head SHA
- latest head 기준 GitHub Actions 또는 후속 기록 fast-pass 결과
- `mergeable` / `mergeStateStatus`
- PR diff에 `mydocs/pr/archives/pr_1872_review.md`와 `mydocs/pr/archives/pr_1872_review_impl.md`가 포함됐는지
- 작업지시자 최종 merge 승인

## issue close 계획

- #1667: PR 본문이 `Refs` 형식이므로 auto-close 대상이 아니다. merge 후에도 측정값 정리와 cache 전략 판단이
  남아 있으면 open 유지한다.
- #1668: tracking issue이므로 #1665 등 후속 sub-issue 진행 상황을 보고 open 유지 여부를 판단한다.
