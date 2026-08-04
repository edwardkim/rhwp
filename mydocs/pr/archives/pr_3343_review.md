# PR #3343 검토 기록 — snafu 0.9.2

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
current head: 87a111eedb77be5c95bde209bbe39607348a98ab
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3343](https://github.com/edwardkim/rhwp/pull/3343) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (source 통합 전 review request) |
| labels / milestone | `dependencies`, `rust` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +4/-4, 1 file, 1 commit |
| head | `87a111eedb77be5c95bde209bbe39607348a98ab` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | Rust Route B 통합에 수용 |

metadata와 CI는 문서 작성 시점 참고값이다. 최종 판단 전 최신 integration PR head에서 다시 확인한다.

## 변경과 통합

- `Cargo.lock`의 `snafu`와 `snafu-derive`를 0.9.1에서 0.9.2로 갱신한다.
- `Cargo.toml`의 직접 의존성 요구사항 `snafu = "0.9.0"`은 0.9.2를 허용하므로 manifest 수정은 없다.
- source `87a111eedb77be5c95bde209bbe39607348a98ab`를 `git cherry-pick -x`로 적용해
  integration `c4a0bd7fcc13ccbf5c05065c50f4576f7b331915`를 만들었다.
- `cargo tree -i snafu@0.9.2`에서 새 lockfile 해소가 `rhwp`의 직접 의존 경로임을 확인했다.
- 원 author, `Signed-off-by`, source SHA provenance를 integration commit에 보존했다.

## 검증과 리스크

- `snafu` derive와 `ResultExt`는 HWP3 parser와 WMF converter의 오류 타입·컨텍스트에 사용된다.
- patch update이며 source API 보정 없이 전체 target이 컴파일되고 관련 parser/converter 테스트가
  통과했다.
- 원 PR 최신 head의 CI, CodeQL, Canvas visual diff는 문서 작성 시점 모두 성공이다.
- 누적 Rust candidate에서 다음을 통과했다.
  - `CARGO_INCREMENTAL=0 cargo fmt --check`
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
  - `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`
  - `git diff --check upstream/devel...HEAD`
- source, fixture, renderer 로직 자체는 바뀌지 않아 별도 수동 visual sweep은 수행하지 않았다.

**권고**: #3343은 직접 merge하지 않고 Rust integration PR로 대체한다. 최신 integration head의
full CI와 작업지시자 승인을 merge 조건으로 두고, merge 뒤 별도 승인으로 원 PR을 close한다.
