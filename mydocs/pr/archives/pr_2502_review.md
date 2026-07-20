# PR #2502 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2502](https://github.com/edwardkim/rhwp/pull/2502) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +0/-1, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | Safari background의 미사용 상수 제거 |
| 판단 | Safari 보정 통합본으로 수용, 원 PR은 별도 적용하지 않음 |

## 변경 범위와 통합
- PR 본문은 Safari background에서 더 이상 참조되지 않는 `MAX_FILE_SIZE` 상수를 제거한다.
- PR 코멘트는 검토 시점에 없었다.
- 동일한 제거가 Safari HML 원격 로드 보안 보정 `7304b385a`에 이미 포함되어 있어 별도 cherry-pick하지 않았다.

## 렌더 영향 판정
- dead-code 제거이므로 visual sweep 대상이 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- Safari shared signature 검사 회귀와 Chrome/Firefox production build, unsigned Safari Xcode build를 함께 확인했다.

## 리스크와 권고
- 중복 삭제를 피하고 보안 게이트 보정이 포함된 단일 Safari 변경으로 관리한다.
- **권고**: Safari 보정 통합본으로 수용, 원 PR은 별도 적용하지 않음. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
