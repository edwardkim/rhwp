# PR #2505 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2505](https://github.com/edwardkim/rhwp/pull/2505) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +2/-0, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | Safari source tree의 누락된 `.gitignore` 추가 |
| 판단 | 누적 통합 PR에 수용 |

## 변경 범위와 통합
- PR 본문은 Safari build 산출물이 source tree에 섞이지 않도록 `.gitignore`를 보완한다.
- PR 코멘트는 검토 시점에 없었다.
- 기여자 원 변경 `89577ed70`을 적용했다.

## 렌더 영향 판정
- source-tree ignore 규칙만 변경하므로 visual sweep 대상이 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- Safari source/build 검증은 unsigned Xcode build까지 통과했다. 서명 단계 제한은 로컬 인증서 환경에 따른 것이다.

## 리스크와 권고
- ignore 규칙은 신규 build output만 대상으로 하며 tracked source를 숨기지 않는지 diff로 확인했다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
