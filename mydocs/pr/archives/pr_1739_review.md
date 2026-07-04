# PR #1739 리뷰 — #1666 PR CI release-test profile 전환

- PR: #1739 `Task #1666: PR CI를 release-test profile 중심으로 전환`
- 작성자: @postmelee
- 기준 브랜치: `devel`
- PR head: `8614c95b4a46f5500ff46362af8cbe16ac9bcddd` (문서 작성 시점 참고값)
- 규모: 5 files, +598/-4
- 관련 이슈: #1666, #1668
- 검토 경로: collaborator self-merge 후보 예외 경로
- 검토 중 상태: `MERGEABLE`, `CLEAN` (merge 전 확인 시점 참고값)
- 최종 처리: merge 완료
- merge commit: `1a7a8305d765830605a4ae8f9bbb99f61febb82c`
- mergedAt: 2026-07-01 13:58:02Z
- 후속 문서 처리: PR head merge 이후 작성되어 별도 문서-only PR로 반영

## 변경 요약

PR `Build & Test` job의 Rust build/test profile을 이벤트별로 분리한다.

- `pull_request`
  - `Build`: `cargo build --profile release-test --verbose`
  - `Native Skia tests`: `cargo test --profile release-test --features native-skia skia --lib --verbose`
  - `Run lib tests`: `cargo test --profile release-test --lib --verbose`
  - `Run integration tests`: `cargo test --profile release-test --tests --verbose`
- non-PR trusted/manual event
  - 위 네 step을 `release` profile로 실행

보존한 항목:

- `Build & Test` job 이름
- cache restore/save 정책
- `Cargo.toml` profile 정의
- `tests/**`, `tests/golden_svg/**`
- job 병렬화 없음
- third-party Rust cache action 도입 없음

## 변경 범위

코드 변경:

- `.github/workflows/ci.yml`
  - 네 cargo step에 `GITHUB_EVENT_NAME` 기반 조건부 profile 분기 추가
  - 각 step에 `profile=... event=...` 로그 추가

운영 문서:

- `mydocs/orders/20260701.md`
- `mydocs/plans/task_m100_1666.md`
- `mydocs/plans/task_m100_1666_impl.md`
- `mydocs/working/task_m100_1666_stage1.md`

범위 외로 유지된 파일:

- `Cargo.toml`
- `tests/**`
- `tests/golden_svg/**`

## 관련 이슈

- #1666 `[CI] PR용 Rust profile 재검토: --release vs release-test`
  - 상태: open (문서 작성 시점 참고값)
  - milestone: `v1.0.0`
  - labels: `enhancement`, `ci`
- #1668 `[Tracking/RFC] CI Build & Test 실행 시간 단계별 단축`
  - 상태: open (문서 작성 시점 참고값)
  - milestone: `v1.0.0`
  - labels: `enhancement`, `discussions`, `ci`

PR description은 `Refs #1666, #1668` 형식이므로 merge만으로 issue auto-close가 발생하지 않는다. Merge 후 #1666은
open 상태이며 측정값 기록과 close 여부를 별도로 판단한다. #1668도 open 상태이며 tracking issue로 open 유지
여부를 확인한다.

## 로컬 검증

PR head `8614c95b4a46f5500ff46362af8cbe16ac9bcddd` 기준으로 확인했다.

```bash
git diff --check upstream/devel...HEAD
actionlint .github/workflows/ci.yml
ruby -e 'require "yaml"; YAML.load_file(".github/workflows/ci.yml"); puts "yaml parse ok"'
git diff --name-only upstream/devel...HEAD Cargo.toml tests tests/golden_svg
```

결과:

- `git diff --check`: 통과
- `actionlint .github/workflows/ci.yml`: 통과
- YAML parse: 통과
- `Cargo.toml`, `tests/**`, `tests/golden_svg/**`: 변경 없음

## merge 충돌 확인

`upstream/devel` 기준으로 PR head가 최신 base를 포함하는지 확인했다.

```bash
git merge-base --is-ancestor upstream/devel HEAD
git merge-tree --write-tree upstream/devel HEAD
```

결과:

- `upstream/devel`은 PR head의 조상
- `git merge-tree --write-tree upstream/devel HEAD`: 충돌 없이 tree 생성

## GitHub Actions

merge 전 PR head `8614c95b4a46f5500ff46362af8cbe16ac9bcddd` 기준:

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

`Build & Test` run 관측값:

- run: `28519297448`
- job: `84541003903`
- job duration: 2026-07-01 13:08:58Z - 13:19:47Z
- cache restore: exact hit, cache size 약 1561 MB
- cache save: PR event 조건상 skipped
- 오류/경고 표식: `##[error]`, `##[warning]`, cache reservation/read-only/save failure 표식 없음

profile 로그:

- `Build`: `profile=release-test event=pull_request`
- `Native Skia tests`: `profile=release-test event=pull_request`
- `Run lib tests`: `profile=release-test event=pull_request`
- `Run integration tests`: `profile=release-test event=pull_request`

cargo 완료 로그:

- `Build`: `Finished release-test ... in 1m 30s`
- `Check WASM target`: `Finished dev ... in 14.03s`
- `Native Skia tests`: `Finished release-test ... in 2m 04s`
- `Run lib tests`: `Finished release-test ... in 1m 26s`
- `Run integration tests`: `Finished release-test ... in 2m 30s`
- `Clippy`: `Finished dev ... in 24.36s`

## 시각 검증

해당 없음.

렌더러, 레이아웃, 샘플, golden fixture 변경이 없는 CI workflow 변경 PR이다.

## 리뷰 결과

Blocking finding 없음.

PR event에서는 의도대로 네 cargo step 모두 `release-test` profile을 사용했고, trusted/manual event에서는
`release` profile을 유지하도록 분기되어 있다. `Build & Test` job 이름과 cache 정책은 유지됐고,
branch protection 표면도 바뀌지 않는다.

주요 운영상 trade-off는 PR에서 release LTO/link-only 회귀를 매번 잡지 않는 대신, `devel`/`main` push 및
tag/workflow_dispatch에서 release-grade 검증을 수행하는 구조로 바뀐다는 점이다. 이 trade-off는 #1666 계획서와
구현 계획서에 기록되어 있고, merge 후 `devel` push run에서 release profile 유지 여부를 확인해야 한다.

## merge 전 재확인 결과

- PR head 최신 SHA: `8614c95b4a46f5500ff46362af8cbe16ac9bcddd`
- GitHub Actions: latest head 기준 relevant checks success/skipped
- `mergeable` / `mergeStateStatus`: `MERGEABLE` / `CLEAN` (merge 전 확인 시점 참고값)
- 작업지시자 승인 이후 merge 완료
- review 문서는 PR head merge 이후 작성되어 후속 문서-only PR로 반영

## merge 후 확인 결과와 계획

- PR #1739 merge 완료: `1a7a8305d765830605a4ae8f9bbb99f61febb82c`
- #1666: open
- #1668: open
- `devel` push run에서 네 cargo step이 `profile=release event=push`로 실행되는지 확인
- `Run integration tests`가 trusted event에서 `cargo test --release --tests --verbose`로 실행되는지 확인
- #1666에 raw measurement 기록
- #1666 close 여부 판단
- #1668 tracking issue는 후속 sub-issue가 남아 있으므로 open 유지 여부 확인

## 최종 판단

수용 및 merge 완료.

PR event에서 `release-test` profile 전환이 의도대로 작동했고, latest PR head 기준 GitHub Actions가 통과한 뒤
merge됐다. review 문서는 PR head merge 이후 작성되었으므로 별도 후속 문서-only PR로 반영한다.
