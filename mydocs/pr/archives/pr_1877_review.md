# PR #1877 리뷰 - #1665 Build & Test Native Skia 병렬 분리

## 메타

| 항목 | 내용 |
|------|------|
| PR | #1877 |
| 제목 | `Task #1665: Build & Test Native Skia 병렬 분리` |
| 작성자 | @postmelee |
| base | `devel` |
| head | `postmelee:task-1665-build-test-parallel-plan` |
| 검토 경로 | collaborator self-merge 후보 예외 경로 |
| 관련 이슈 | #1665, #1668 |
| labels | `enhancement`, `ci` |
| milestone | `v1.0.0` |
| assignee | @postmelee |
| review request | @edwardkim |
| 코드 검증 기준 SHA | `34f714bcab6d9c2e975cce9c8ebf03a6fda77bf5` |
| 규모 | 문서 작성 시점 참고값: 5 files, +810 / -14 |
| 상태 | 문서 작성 시점 참고값: draft false, `MERGEABLE`, `CLEAN` |

이 문서는 merge 완료 보고서가 아니라 self-merge 후보의 merge 전 검토 기록이다. 이 문서와
`pr_1877_review_impl.md`, 오늘할일 보강은 후속 `mydocs/**` 문서 커밋으로 PR head에 포함한다. 따라서 문서
커밋 push 후 최신 PR head SHA는 위 코드 검증 기준 SHA와 달라진다.

## 변경 요약

`CI / Build & Test`의 긴 직렬 경로 중 Native Skia 검증만 1차로 별도 worker job으로 분리한다.

- `build-default-feature-tests`
  - 기존 기본 feature 경로를 담당한다.
  - format check, Build, default-feature tests, WASM target check, Clippy를 실행한다.
  - 기존 cargo cache key/path를 유지한다.
  - trusted branch cache save writer를 이 job 하나로 제한한다.
- `native-skia-tests`
  - native Skia runtime package 설치와 `cargo test --features native-skia skia --lib`를 담당한다.
  - cargo cache restore-only로 동작한다.
  - save step을 두지 않는다.
- `build-and-test`
  - job id와 `name: Build & Test`를 유지한다.
  - 직접 build/test를 수행하지 않고 `preflight`, `Build default-feature tests`, `Native Skia tests` 결과를
    집계하는 aggregate gate가 된다.
  - fast-pass PR에서는 worker jobs가 skipped 되어도 preflight 승인 결과를 근거로 success가 된다.
  - 일반 PR/push에서는 두 worker job이 모두 success여야 success가 된다.

이번 PR은 job 배치를 바꾸는 변경이며 test 명령, cache key/path, golden/sample, renderer/layout 코드는 변경하지
않는다.

## 변경 범위

코드/CI 변경:

- `.github/workflows/ci.yml`
  - 기존 `build-and-test` job을 `build-default-feature-tests` worker로 이동
  - `native-skia-tests` worker 추가
  - `build-and-test`를 aggregate gate로 재정의
  - `wasm-build` 조건과 동작은 유지

운영 문서:

- `mydocs/orders/20260704.md`
- `mydocs/plans/task_m100_1665.md`
- `mydocs/plans/task_m100_1665_impl.md`
- `mydocs/working/task_m100_1665_stage1.md`

후속 문서 커밋에서 추가/보강할 운영 기록:

- `mydocs/orders/20260704.md`
- `mydocs/pr/archives/pr_1877_review.md`
- `mydocs/pr/archives/pr_1877_review_impl.md`

## 관련 이슈

- #1665 `[CI] Build & Test job 병렬 분리 설계`
  - 상태: open (문서 작성 시점 참고값)
  - labels: `enhancement`, `ci`
  - milestone: `v1.0.0`
- #1668 `[Tracking/RFC] CI Build & Test 실행 시간 단계별 단축`
  - 상태: open (문서 작성 시점 참고값)
  - tracking/RFC 이슈

PR 본문은 `Refs #1665`, `Parent tracking: #1668` 형식이다. Merge만으로 issue auto-close가 발생하지 않는다.
#1665는 merge 후 `devel` push 관측과 measurement/report 문서 반영이 남으면 open 유지가 자연스럽다. #1668은
tracking issue이므로 후속 sub-issue 진행 상황에 맞춰 별도 판단한다.

## 로컬 검증

코드 검증 기준 SHA `34f714bcab6d9c2e975cce9c8ebf03a6fda77bf5` 기준으로 확인했다.

```bash
ruby -e "require 'yaml'; YAML.load_file('.github/workflows/ci.yml'); puts 'yaml ok'"
actionlint .github/workflows/ci.yml
git diff --check upstream/devel...HEAD
git diff --check
```

결과:

- YAML parse: `yaml ok` 확인. 로컬 Ruby `ffi` gem 확장 경고는 YAML 파싱 실패가 아니었다.
- `actionlint .github/workflows/ci.yml`: 통과
- `git diff --check upstream/devel...HEAD`: 통과
- `git diff --check`: 통과
- merge base: `upstream/devel`의 `0a741f9e2ac84777c048cb0ff63b302e085a2553`

## GitHub Actions

코드 검증 기준 SHA `34f714bcab6d9c2e975cce9c8ebf03a6fda77bf5` 기준 relevant checks:

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

CI run:

- <https://github.com/edwardkim/rhwp/actions/runs/28678383547>
- PR event head SHA: `34f714bcab6d9c2e975cce9c8ebf03a6fda77bf5`

핵심 로그 확인:

- preflight: `fast_pass=false reason=no-trailing-review-only-commits`
- `Build default-feature tests`
  - cache size: `~1561 MB (1637296893 B)`
  - cache key: `Linux-cargo-6a1af67968af2b829f31637cb42371573b1fc279c0b7634dc63557a90d4227c2`
  - Build profile: `release-test`, `event=pull_request`
  - default-feature tests: `running 2081 tests`
  - result: `2075 passed; 0 failed; 6 ignored`
  - PR run에서 `Save cargo registry & build cache` skipped
- `Native Skia tests`
  - cache size: `~1561 MB (1637296893 B)`
  - cache key: `Linux-cargo-6a1af67968af2b829f31637cb42371573b1fc279c0b7634dc63557a90d4227c2`
  - profile: `release-test`, `event=pull_request`
  - native Skia tests: `running 48 tests`
  - result: `48 passed; 0 failed; 0 ignored; 2080 filtered out`
  - save step 없음
- aggregate `Build & Test`
  - `preflight=success`
  - `fast_pass=false`
  - `build_default=success`
  - `native_skia=success`

전체 CI run 로그에서 `Cache reservation failed`, `read only`, `read-only`, `Failed to save`, `Unable to reserve`
문구는 확인되지 않았다.

## 한계

이번 PR은 cache save writer를 `Build default-feature tests` 하나로 제한한다. 변경 전 단일 `Build & Test`
job에서는 trusted branch cache save 시점에 default-feature 산출물과 Native Skia 산출물이 같은 `target/`에
함께 포함될 수 있었다. 변경 후에는 Native Skia job의 `target/` 산출물이 별도 runner에서 생성되고 저장되지
않는다.

이 선택은 같은 cargo cache key에 여러 job이 동시에 save하면서 cache reservation 경합이나 read-only 경고가
재발하는 것을 피하기 위한 보수적 설계다. 대신 Native Skia job은 exact restore 이후에도 필요한 산출물을 다시
빌드할 수 있고, runner-minutes 또는 Native Skia job 시간이 기대보다 줄지 않을 수 있다.

따라서 merge 후 `devel` push와 후속 PR run에서 Native Skia compile 시간, cache hit 여부, queue 대기,
runner-minutes를 관측한 뒤 별도 native-skia cache key 또는 별도 target dir 도입 여부를 판단한다.

## 시각 검증

해당 없음.

이 PR은 GitHub Actions workflow와 운영 문서 변경이며 renderer, layout, paint, wasm render output, samples,
golden fixture를 변경하지 않는다. 따라서 `visual_sweep_guide.md` 대상이 아니다.

## 검토 중 확인한 사항

### PR 본문 명령 예시 불일치

PR 본문의 `Build default-feature tests` 설명에는 build step이
`cargo build --profile release-test --no-default-features --features test-utils`로 적혀 있다. 실제 workflow는
기존 default-feature build 경로인 `cargo build --profile release-test --verbose`를 유지한다.

이 불일치는 workflow 동작 자체의 blocker는 아니지만, PR 설명이 실제 검증 명령과 다르므로 merge 전 PR 본문을
보정하는 것이 좋다.

## 리뷰 결과

Blocking finding 없음.

Native Skia 검증 분리는 기존 `Build & Test` required check 이름을 aggregate gate로 보존하면서, default-feature
경로와 native-skia 경로를 병렬 실행하도록 구성됐다. GitHub Actions에서 두 worker job과 aggregate `Build & Test`
모두 성공했고, default-feature test count와 Native Skia test count도 기존 기대값과 일치한다. PR cache save
정책도 유지됐다.

단, PR 본문 명령 예시 불일치는 merge 전 보정 권고 사항으로 남긴다.

## merge 전 재확인 조건

merge 전에는 다음을 다시 확인한다.

- 문서 후속 커밋 push 후 최신 PR head SHA
- latest head 기준 GitHub Actions 또는 후속 기록 fast-pass 결과
- `mergeable` / `mergeStateStatus`
- PR diff에 `mydocs/pr/archives/pr_1877_review.md`와 `mydocs/pr/archives/pr_1877_review_impl.md`가 포함됐는지
- PR 본문 명령 예시 불일치 보정 여부
- 작업지시자 최종 merge 승인

## issue close 계획

- #1665: PR 본문이 `Refs #1665` 형식이므로 auto-close 대상이 아니다. merge 후에도 `devel` push 관측과
  measurement/report 문서 정리가 남아 있으면 open 유지한다.
- #1668: tracking issue이므로 open 유지한다.
