# Stage 2 완료 보고서 — Task #240

- 작업: SVG 렌더러 임베딩 경로에 BMP→PNG 적용
- 일자: 2026-04-22

## 수정된 임베딩 경로 (3개 지점)

1. **`src/renderer/svg.rs:161-175`** — 노드 배경 이미지 (`bg.image`)
2. **`src/renderer/svg.rs:1035-1055`** — `render_image_node` (HWP 본문 이미지)
   - 기존 WMF→SVG 분기와 동일한 패턴으로 `image/bmp` 분기 추가
3. **`src/renderer/layout/shape_layout.rs:1043-1067`** — OLE container `native_image` 폴백
   - `crate::renderer::svg::bmp_bytes_to_png_bytes` 호출

모든 분기는 실패 시 원본 BMP 유지(폴백).

## 회귀 테스트

```
cargo test --lib
test result: ok. 941 passed; 0 failed; 1 ignored
```

기존 테스트 전부 통과.

## 다음 단계
- Stage 3: 샘플 재검증 및 최종 보고서
