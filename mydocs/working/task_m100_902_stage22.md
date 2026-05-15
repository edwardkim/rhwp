# Task #902 Stage 22 보고서 — font escapement state 추적

**Stage**: 22 / 25+ (v2 점진 포팅)
**상태**: 완료 (state 추적, rotation rendering 은 follow-up)

## 1. 변경 영역

- `src/wmf/converter/raster/state.rs` — `FontInfo` 에 `escapement: i16`, `orientation: i16` 필드 추가
- `src/wmf/converter/raster/player.rs` — `create_font_indirect` 가 escapement/orientation 보존

## 2. 한계 (follow-up)

본 stage 는 escapement state 만 추적. 실제 text rotation rendering 은 미구현:
- escapement != 0 시 glyph bitmap 을 tiny-skia Transform 으로 회전 + blit 필요
- 회전된 bounding box 계산 + clip 처리
- LO 의 EmulateAntiAlias 기능 (회전 시 alpha 변화)

샘플 sample16 의 사용 빈도 낮음 (대다수 텍스트가 escapement=0). 향후 회전 텍스트 사용 sample 발견 시 정밀화.

## 3. 검증

```
cargo build --release           — Finished
cargo test --release --all-targets — (1412 passed)
```

## 4. 다음 단계

Stage 23: WASM 폰트 임베딩 (브라우저 quality 향상 — rhwp-studio 영향)
