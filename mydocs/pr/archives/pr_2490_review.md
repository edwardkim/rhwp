# PR #2490 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2490](https://github.com/edwardkim/rhwp/pull/2490) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +1/-1, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | `render-diff.yml`의 `upload-artifact` 버전 갱신 |
| 판단 | #2501 통합본으로 수용, 원 PR은 별도 적용하지 않음 |

## 변경 범위와 통합
- PR 본문은 render-diff workflow의 단일 `upload-artifact` action 버전을 올리는 변경이다.
- PR 코멘트는 검토 시점에 없었다.
- 원 변경은 [#2501](https://github.com/edwardkim/rhwp/pull/2501)의 4개 workflow 일괄 갱신 `4b9dd1627`에 포함되어 있어 별도 cherry-pick하지 않았다.

## 렌더 영향 판정
- CI workflow 메타데이터만 변경하므로 visual sweep 대상이 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- YAML parse 및 actionlint 결과는 #2501 검토 기록과 동일하다.

## 리스크와 권고
- 중복 적용을 피하고 단일 CI 갱신 커밋으로 version 정책을 유지한다.
- **권고**: #2501 통합본으로 수용, 원 PR은 별도 적용하지 않음. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
