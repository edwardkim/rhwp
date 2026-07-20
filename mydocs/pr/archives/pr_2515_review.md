# PR #2515 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2515](https://github.com/edwardkim/rhwp/pull/2515) |
| 작성자 / base | dependabot[bot] / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +27/-16, 1 file, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | `serde` 1.0.228 → 1.0.229 lockfile 갱신 |
| 판단 | Dependabot 누적 통합 PR에 수용 |

## 변경 범위와 통합

- PR 본문은 serde 1.0.229의 syn 3 대응을 안내한다. 검토 시점 PR 코멘트는 없었다.
- 원 커밋 `660c56aec`을 최신 `upstream/devel` 위에 충돌 없이 적용했고, 통합 브랜치 적용 커밋은 `cd4f1969f`다.
- `serde`, `serde_core`, `serde_derive`가 1.0.229로 갱신되고, derive 경로에만 `syn 3.0.1`이 추가된다. 기존 proc-macro 소비자는 `syn 2.0.114`를 계속 사용한다.

## 렌더 영향 판정

- Rust 직렬화 라이브러리의 lockfile patch update이며 renderer·layout 정책을 직접 변경하지 않는다. visual sweep 대상이 아니다.

## 검증

- `cargo tree -i serde@1.0.229`로 `rhwp`와 `wasm-bindgen-test`의 새 serde 해소를 확인했다.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 통과.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과.
- `git diff --check upstream/devel...HEAD` 통과.

## 리스크와 권고

- derive macro의 `syn` major가 분리되어 추가되므로 전체 Rust compile/test를 검증 근거로 사용한다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤 merge한다.
