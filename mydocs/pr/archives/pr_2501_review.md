# PR #2501 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2501](https://github.com/edwardkim/rhwp/pull/2501) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +9/-9, 4 files, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | 4개 GitHub Actions workflow의 action major version 갱신 |
| 판단 | 누적 통합 PR에 수용 |

## 변경 범위와 통합
- PR 본문은 repository에서 사용 중인 GitHub Actions의 공통 버전을 최신 정책에 맞게 일괄 갱신한다.
- PR 코멘트는 검토 시점에 없었다.
- 기여자 원 변경 `4b9dd1627`을 적용했고, [#2488](https://github.com/edwardkim/rhwp/pull/2488), [#2490](https://github.com/edwardkim/rhwp/pull/2490), [#2499](https://github.com/edwardkim/rhwp/pull/2499)의 workflow 범위를 함께 흡수했다.

## 렌더 영향 판정
- CI workflow 메타데이터만 변경하므로 visual sweep 대상이 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- 변경 workflow 4개의 YAML parse와 actionlint를 확인했다. actionlint의 shellcheck 정보성 8건은 baseline이며, 해당 규칙을 제외하면 통과한다.

## 리스크와 권고
- 기존 CI job의 shellcheck 경고는 이번 PR 원인이 아니며 별도 정리 대상으로 남긴다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
