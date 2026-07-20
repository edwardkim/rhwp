# PR #2496 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2496](https://github.com/edwardkim/rhwp/pull/2496) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +6/-2, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | `@rhwp/editor`의 Node engines와 publish 파일 목록 정합성 |
| 판단 | 누적 통합 PR에 수용 |

## 변경 범위와 통합
- PR 본문은 npm editor package가 지원 Node 범위를 선언하고 README를 배포 패키지에 포함하도록 보완한다.
- PR 코멘트는 검토 시점에 없었다.
- 기여자 원 변경 `bfc27616f`을 적용했다.

## 렌더 영향 판정
- npm package metadata만 변경하므로 visual sweep 대상이 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- `npm test` 18/18 통과, `npm pack --dry-run`에서 README를 포함한 예상 5개 파일을 확인했고 JSON parse를 통과했다.

## 리스크와 권고
- 지원 Node 범위를 `>=18`으로 명시해 설치 실패를 조기에 드러내는 범위다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
