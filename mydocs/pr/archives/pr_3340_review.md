# PR #3340 검토 기록 — base64 0.23

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
current head: 22847a07ed2497cd8514f425b7d4363d40e67349
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3340](https://github.com/edwardkim/rhwp/pull/3340) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (source 통합 전 review request) |
| labels / milestone | `dependencies`, `rust` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +10/-4, 2 files, 1 commit |
| head | `22847a07ed2497cd8514f425b7d4363d40e67349` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | Rust Route B 통합에 수용 |

metadata와 CI는 문서 작성 시점 참고값이다. 최종 판단 전 최신 integration PR head에서 다시 확인한다.

## 변경과 통합

- `Cargo.toml`의 직접 의존성 `base64` 요구사항을 `0.22`에서 `0.23`으로 올리고 lockfile을 갱신한다.
- source `22847a07ed2497cd8514f425b7d4363d40e67349`를 `git cherry-pick -x`로 적용해
  integration `37293b256a564fe98a1f94ca7b404c2bcde2773e`을 만들었다.
- `cargo tree -i base64@0.23.0`에서 새 버전이 `rhwp`의 직접 의존성으로 해소됨을 확인했다.
- `resvg`/`usvg` 계열의 전이 의존성은 계속 `base64 0.22.1`을 사용하므로 lockfile에는 두 버전이
  공존한다. 이는 Cargo의 정상적인 version resolution이며 직접 의존성의 적용 누락이 아니다.
- 원 author, `Signed-off-by`, source SHA provenance를 integration commit에 보존했다.

## 검증과 리스크

- `base64`는 문서 리소스, SVG/HTML/Web Canvas renderer, OLE/EMF, clipboard, CLI, WASM API 등에서
  encode/decode에 사용된다. 현재 코드는 `Engine` API를 사용하며 source 수정 없이 0.23으로 컴파일된다.
- 원 PR 최신 head의 CI, CodeQL, Canvas visual diff는 문서 작성 시점 모두 성공이다.
- 누적 Rust candidate에서 다음을 통과했다.
  - `CARGO_INCREMENTAL=0 cargo fmt --check`
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
  - `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`
  - `git diff --check upstream/devel...HEAD`
- 전체 integration test에는 SVG snapshot, visual roundtrip baseline, base64 decode/encode 관련 회귀가
  포함된다.
- source, fixture, renderer 로직 자체는 바뀌지 않아 별도 수동 visual sweep은 수행하지 않았다.

**권고**: #3340은 직접 merge하지 않고 Rust integration PR로 대체한다. 최신 integration head의
full CI와 작업지시자 승인을 merge 조건으로 두고, merge 뒤 별도 승인으로 원 PR을 close한다.
