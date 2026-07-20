# PR #2503 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2503](https://github.com/edwardkim/rhwp/pull/2503) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +1/-0, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | `@rhwp/editor`의 funding metadata 추가 |
| 판단 | #2504 통합본으로 수용, 원 PR은 별도 적용하지 않음 |

## 변경 범위와 통합
- PR 본문은 npm editor package에 funding 필드를 추가하는 단독 metadata 제안이다.
- PR 코멘트는 검토 시점에 없었다.
- 같은 funding 보강이 [#2504](https://github.com/edwardkim/rhwp/pull/2504)의 기여자 변경 `a81b51776`에 포함되어 있어 별도 cherry-pick하지 않았다.

## 렌더 영향 판정
- npm metadata만 변경하므로 visual sweep 대상이 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- #2504의 `npm test` 18/18 및 `npm pack --dry-run` 결과로 package metadata의 publish 경로를 함께 확인했다.

## 리스크와 권고
- 중복 field 적용을 피하고 README 보강과 한 커밋으로 유지한다.
- **권고**: #2504 통합본으로 수용, 원 PR은 별도 적용하지 않음. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
