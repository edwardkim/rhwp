# PR #2493 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2493](https://github.com/edwardkim/rhwp/pull/2493) |
| 작성자 / base | kevin9327 / `devel` |
| 검토자 | @jangster77 (검토 전 지정) |
| 규모 / 검토 스냅샷 | 2026-07-20 GitHub 조회: +3/-0, 1 file, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 범위 | `.hml` 파일을 HWP Viewer로 여는 VS Code selector 등록 |
| 판단 | 누적 통합 PR에 수용 |

## 변경 범위와 통합
- PR 본문은 VS Code 확장자가 `.hml`을 HWP Viewer 문서로 인식하도록 filenamePattern을 추가한다.
- PR 코멘트는 검토 시점에 없었다.
- 기여자 원 변경 `574ea21cf`을 적용했다.

## 렌더 영향 판정
- VS Code 문서 selector 등록이며 renderer 출력 변경이 아니다.

## 검증
- 누적 통합 브랜치에서 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`, `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `wasm-pack build --target web --out-dir pkg`를 통과했다.
- VS Code `package.json` JSON parse와 누적 WASM build를 통과했다.

- headless hml-open.check E2E에서 HML 문서가 실제 canvas에 1페이지로 보이고 HML 저장·재열기와 기존 HWP 재열기까지 통과했다. VS Code selector 자체는 package metadata 검증 범위로 유지한다.

## 리스크와 권고
- HML 지원 노출은 [#2495](https://github.com/edwardkim/rhwp/pull/2495), [#2511](https://github.com/edwardkim/rhwp/pull/2511)의 설명·브라우저 등록과 함께 정합성을 유지한다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
