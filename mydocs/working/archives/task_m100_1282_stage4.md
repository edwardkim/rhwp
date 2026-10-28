# Task 1282 Stage 4 — 최종 검증 정리

## 목적

Stage 2 Rust 보정과 Stage 3 rhwp-studio 드래그 회귀 테스트를 최종 산출물 관점에서
정리한다.

## 검증 결과

통과:

```text
cargo fmt --check
cargo test --test issue_1282_rotated_cell_picture_resize -- --nocapture
cargo test --test issue_1279_picture_rotation_save
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && node e2e/table-picture-resize-1282.test.mjs --mode=headless
```

## 결론

- `samples/ta-pic-001-r.hwp`의 회전된 표 셀 picture를 크게 리사이즈해도 owner cell height가
  picture 표시 높이와 padding 이상으로 증가한다.
- export/reparse 후에도 증가한 cell height가 유지된다.
- rhwp-studio 실제 마우스 드래그 경로에서도 `cellPath`가 보존되고, cell height 증가와 bbox 중심 안정성이 확인됐다.
- #1279의 회전 저장/라운드트립 회귀 테스트는 그대로 통과했다.

## 후속 분리

- 드래그 결과에서 음수 offset은 기존 Studio 경로와 동일하게 unsigned u32 값으로 노출될 수 있다.
  이번 이슈의 렌더/리사이즈 정합은 통과했으므로, 속성 UI 표시 정규화가 필요하면 별도 이슈로 분리한다.
