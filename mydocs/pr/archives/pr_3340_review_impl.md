# PR #3340 umbrella implementation 계획 — Rust 의존성 2건

## 목적과 상태

- 대상: Dependabot PR [#3340](https://github.com/edwardkim/rhwp/pull/3340),
  [#3343](https://github.com/edwardkim/rhwp/pull/3343)
- 역할: 두 원 PR을 직접 merge하지 않고 최신 `upstream/devel` 기반 Route B integration PR로 대체한다.
- 현재 상태: source cherry-pick과 local Rust 검증을 완료하고 review 문서를 작성하는 단계다.
- 이 문서는 integration PR merge 전 실행 계획이다. remote push, PR 생성, 최신 CI 성공, merge,
  원 PR close를 완료 사실로 기록하지 않는다.

## 라우팅과 기준선

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
integration branch: review/dependabot-rust-20260726
base: upstream/devel@e7dffced399e45685ae746bd2ea21d37542ea95e
verified local code head: c4a0bd7fcc13ccbf5c05065c50f4576f7b331915
```

두 원 PR은 모두 `maintainerCanModify=false`이므로 Dependabot head에는 commit을 push하지 않는다.
원 PR별 reviewer를 source 통합 전에 `@postmelee`로 지정했다. 기존 `dependencies`, `rust` labels를
유지하고 milestone은 만들지 않았다. Dependabot bot은 assignable actor가 아니어서 assignee는 비워 두었다.

## source와 integration commit

| 순서 | 원 PR | source commit | integration commit | 변경 |
|---|---|---|---|---|
| 1 | #3340 | `22847a07ed2497cd8514f425b7d4363d40e67349` | `37293b256a564fe98a1f94ca7b404c2bcde2773e` | base64 0.22.1 → 0.23.0 |
| 2 | #3343 | `87a111eedb77be5c95bde209bbe39607348a98ab` | `c4a0bd7fcc13ccbf5c05065c50f4576f7b331915` | snafu/snafu-derive 0.9.1 → 0.9.2 |

두 source commit은 `git cherry-pick -x`로 적용했다. author는 `dependabot[bot]`으로 유지되고
`Signed-off-by`와 source SHA provenance가 commit message에 보존된다. 충돌이나 collaborator code
보정은 없었다.

## 변경 범위

- 최종 diff는 `Cargo.toml`, `Cargo.lock` 2개 파일의 14 additions/8 deletions다.
- `base64` 직접 요구사항을 `0.22`에서 `0.23`으로 올린다.
- `snafu` manifest 요구사항은 그대로 두고 lockfile 해소만 0.9.2로 갱신한다.
- Rust source, fixture, renderer, frontend, workflow는 변경하지 않는다.

## 단계

| 단계 | 상태 | 작업 |
|---|---|---|
| 0. metadata 정렬 | 완료 | 두 원 PR reviewer 지정, labels/milestone 확인 |
| 1. Route B branch | 완료 | 최신 `upstream/devel`에서 integration branch 생성 |
| 2. source 통합 | 완료 | 두 source commit을 번호순 `-x` cherry-pick |
| 3. local verification | 완료 | fmt, 전체 integration test, 전체 clippy, dependency tree, diff check |
| 4. review 문서 | 현재 단계 | 원 PR별 review와 umbrella report를 별도 docs commit으로 작성 |
| 5. remote integration PR | 승인됨·미실행 | origin push와 `devel` 대상 Draft PR 생성 |
| 6. authoritative CI/review | 미실행 | 최신 integration head full CI 확인 |
| 7. merge/후속 | 별도 승인 필요 | integration merge, 원 PR comment/close |

## 검증 결과와 해석

- `CARGO_INCREMENTAL=0 cargo fmt --check`: 성공.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 전체 성공, 실패 0.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 성공, 경고 0.
- `git diff --check upstream/devel...HEAD`: 성공.
- `cargo tree -i base64@0.23.0`: `base64 0.23.0 -> rhwp`.
- `cargo tree -i snafu@0.9.2`: `snafu 0.9.2 -> rhwp`.
- 원 PR 두 건의 최신 head full CI와 CodeQL, Canvas visual diff는 문서 작성 시점 성공이다.

`base64`는 renderer와 serialization 경로에 널리 사용되지만 기존 `Engine` API 코드가 수정 없이
컴파일되고 전체 snapshot/roundtrip test가 통과했다. `resvg`/`usvg`의 전이 의존성은 lockfile에서
`base64 0.22.1`을 계속 사용한다. `snafu`는 HWP3/WMF 오류 경로의 patch 해소다.

source와 fixture가 바뀌지 않았고 전체 test에 SVG snapshot과 visual roundtrip baseline이 포함되므로
별도 수동 visual sweep은 불필요하다고 판정했다. Route B의 authoritative gate는 docs commit까지 포함한
integration PR 최신 head CI다.

## Remote PR 계획과 승인 경계

작업지시자는 branch push와 Draft PR 생성을 승인했다. PR body에는 다음을 포함한다.

- 두 원 PR의 `Supersedes #...`
- source/integration commit mapping과 Dependabot credit 보존
- manifest/lockfile 변경과 두 버전 base64 공존 설명
- local fmt/test/clippy/diff check와 원 PR CI
- 최신 integration PR head full CI gate
- integration PR merge 뒤 별도 승인으로 원 PR을 close한다는 명시

연결된 issue가 없어 `Closes #...`는 넣지 않는다. merge, 원 PR comment/close는 별도 승인이 필요하다.
