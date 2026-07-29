# PR #2495 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2495](https://github.com/edwardkim/rhwp/pull/2495) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +2/-1, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | VS Code 확장 package metadata에 HML 노출 |
| 판단 | 누적 통합 PR에 수용 |

## 변경 범위와 통합
- PR 본문은 VS Code 확장 설명과 검색 키워드가 HML 지원을 드러내도록 보완한다.
- PR 코멘트는 검토 시점에 없었다.
- 기여자 원 변경 `91e655906`을 적용했다.

## 렌더 영향 판정
- 확장 marketplace metadata만 변경하므로 visual sweep 대상이 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- VS Code `package.json` JSON parse와 #2493의 selector 변경 정합성을 확인했다.

## 리스크와 권고
- 기능 변경 없이 발견성만 보강하므로 HML parser/renderer 동작을 새로 약속하지 않는다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
