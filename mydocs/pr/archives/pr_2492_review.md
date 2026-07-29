# PR #2492 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2492](https://github.com/edwardkim/rhwp/pull/2492) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +21/-0, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | Dependabot 설정의 github-actions/npm/editor/rhwp-vscode ecosystem 보완 |
| 판단 | #2491 통합본으로 수용, 원 PR은 별도 적용하지 않음 |

## 변경 범위와 통합
- PR 본문은 Dependabot 대상 ecosystem을 넓히는 단독 제안이다.
- PR 코멘트는 검토 시점에 없었다.
- 같은 설정 변경이 [#2491](https://github.com/edwardkim/rhwp/pull/2491)의 기여자 커밋 `d03ce9224`에 이미 포함되어 있어 별도 cherry-pick하지 않았다.

## 렌더 영향 판정
- Dependabot 설정만 변경하므로 visual sweep 대상이 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- #2491 통합 검증의 범위에서 YAML/빌드 검증을 함께 확인했다.

## 리스크와 권고
- 중복 적용하면 workflow 정책이 분산되므로 단일 통합본을 사용한다.
- **권고**: #2491 통합본으로 수용, 원 PR은 별도 적용하지 않음. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
