# PR #3340 umbrella 사전 판단 보고서 — Rust 의존성 2건

> 이 문서는 PR #3340/#3343을 통합한 Route B candidate의 pre-merge 판단이다.
> remote PR 생성·최신 CI 성공·merge·원 PR close는 아직 완료되지 않았다.

## 판단 요약

| 항목 | 내용 |
|---|---|
| 대상 | [#3340](https://github.com/edwardkim/rhwp/pull/3340), [#3343](https://github.com/edwardkim/rhwp/pull/3343) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| route | Route B — source commit cherry-pick integration PR |
| 기준선 | `upstream/devel@e7dffced399e45685ae746bd2ea21d37542ea95e` |
| local code candidate | `c4a0bd7fcc13ccbf5c05065c50f4576f7b331915` |
| 권고 | 원 PR 직접 merge 대신 Rust integration candidate를 수용 |
| 최종 조건 | 최신 integration PR head full CI 성공과 작업지시자 merge 승인 |

## 변경 축

| dependency | version | 실제 변경 | 주요 사용 범위 |
|---|---|---|---|
| `base64` | 0.22.1 → 0.23.0 | `Cargo.toml`, `Cargo.lock` | renderer, document resources, clipboard, CLI, WASM |
| `snafu` / `snafu-derive` | 0.9.1 → 0.9.2 | `Cargo.lock` | HWP3 parser, WMF converter 오류 처리 |

최종 diff는 2 files, 14 additions/8 deletions이며 Rust source와 fixture는 바뀌지 않는다.

## source provenance와 credit

| 원 PR | source commit | integration commit |
|---|---|---|
| #3340 | `22847a07ed2497cd8514f425b7d4363d40e67349` | `37293b256a564fe98a1f94ca7b404c2bcde2773e` |
| #3343 | `87a111eedb77be5c95bde209bbe39607348a98ab` | `c4a0bd7fcc13ccbf5c05065c50f4576f7b331915` |

두 commit은 `git cherry-pick -x`로 적용해 `dependabot[bot]` author, `Signed-off-by`, source SHA를
보존했다. 충돌, squash, contributor commit rewrite는 없었다.

## 검증

| 검증 | 결과 |
|---|---|
| 누적 cherry-pick | 2/2 충돌 없음 |
| `CARGO_INCREMENTAL=0 cargo fmt --check` | 성공 |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | 전체 성공, 실패 0 |
| `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` | 성공, 경고 0 |
| dependency tree | base64 0.23.0과 snafu 0.9.2 모두 `rhwp` 직접 경로 확인 |
| `git diff --check` | 성공 |
| 원 PR 최신 head CI | 2건 모두 full CI, CodeQL, Canvas visual diff 성공 |

Route B의 authoritative CI는 review 문서까지 포함한 새 integration PR의 최신 head 결과다.

## 리스크 판정

### base64 0.23

`base64` 호출은 renderer와 import/export에 넓게 분포한다. 현재 사용 범위는 `Engine` 기반
encode/decode API이며 source 보정 없이 전체 target이 컴파일됐다. 전체 integration test의 SVG snapshot,
visual roundtrip, CLI, clipboard, resource 회귀가 통과했다.

`resvg`/`usvg` 전이 경로는 계속 `base64 0.22.1`을 사용해 lockfile에 0.22.1과 0.23.0이 공존한다.
`cargo tree`에서 `rhwp` 직접 경로가 0.23.0을 가리키므로 적용 누락은 아니다.

### snafu 0.9.2

manifest의 `snafu = "0.9.0"` 범위 안에서 lockfile만 patch 해소한다. HWP3 parser와 WMF converter의
derive/context 사용은 source 변경 없이 컴파일되고 전체 테스트와 Clippy를 통과했다.

### 시각 영향

renderer 로직, fixture, baseline은 바뀌지 않았다. 전체 integration test의 SVG snapshot과 visual
roundtrip baseline, 원 PR의 Canvas visual diff가 성공했으므로 별도 수동 visual sweep은 생략했다.

## 문서와 remote 반영 계획

- 원 PR별 review: [#3340](pr_3340_review.md), [#3343](pr_3343_review.md)
- 실행 순서와 승인 gate: [umbrella implementation 계획](pr_3340_review_impl.md)
- 원 PR은 `maintainerCanModify=false`이므로 source head에 commit을 push하지 않는다.
- 승인된 `review/dependabot-rust-20260726`을 origin에 push하고 `devel` 대상 Draft PR을 생성한다.
- PR body에 두 `Supersedes`, source/integration mapping, credit, 검증과 최신 CI gate를 기록한다.

## Merge 전 조건

1. docs commit까지 포함한 integration PR 최신 head SHA를 확인한다.
2. `CI preflight`, `Lint`, `Build & Test`, CodeQL, Render Diff 등 최신 relevant checks가 성공해야 한다.
3. review 문서가 integration PR diff에 포함되어야 한다.
4. integration PR이 mergeable 상태여야 한다.
5. 작업지시자가 GitHub review/merge를 별도로 승인해야 한다.

Rust production dependency 변경이므로 review-only fast-pass를 적용하지 않는다.

## Merge 뒤 확인 계획

- integration PR merge commit과 merge 시각을 다시 읽는다.
- 별도 승인 뒤 각 원 PR에 integration PR 번호, merge commit, source/integration mapping, CI 요약,
  Dependabot credit 보존을 설명하고 superseded 상태로 close한다.
- 두 원 PR에는 linked issue가 없다.

**사전 권고**: source provenance를 보존하고 dependency manifest/lockfile만 변경한 Route B candidate를
Draft PR 생성 후보로 수용한다. merge와 원 PR 후속 처리는 최신 integration CI와 별도 승인 뒤 수행한다.
