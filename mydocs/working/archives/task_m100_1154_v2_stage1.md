# Task M100 #1154 v2 — Stage 1 완료 보고서

## 구현 내용

### Backend: overlay JSON 에 crop 필드 추가

`src/document_core/queries/rendering.rs` `write_overlay_image`:

```rust
write_json_str(buf, wrap_str(wrap));

if let Some((left, top, right, bottom)) = image.crop {
    let _ = write!(
        buf,
        ",\"crop\":{{\"left\":{},\"top\":{},\"right\":{},\"bottom\":{}}}",
        left, top, right, bottom
    );
}
```

기존 layer tree 메인 JSON (`paint/json.rs::PaintOp::Image`) 의 crop 직렬화 형태와 정확히 동일.

### Frontend: OverlayImageInfo 확장 + createOverlayLayer wrapper

`rhwp-studio/src/view/page-renderer.ts`:

1. **타입**: `OverlayImageInfo` 에 `crop?: { left; top; right; bottom }` 추가 (HWPUNIT).
2. **폴백**: `toOverlayInfo` (PageLayerTree JSON 직접 파싱 폴백 경로) 도 `op.crop` 전달.
3. **렌더링**: `createOverlayLayer` 분기:
   - crop 이 의미 있는 경우 (`right > left` 그리고 `bottom > top`):
     - sxPx = crop.left/75, syPx = crop.top/75, swPx, shPx 계산.
     - scaleX = dw/swPx, scaleY = dh/shPx.
     - wrapper div: `position: absolute`, bbox 크기, `overflow: hidden`.
     - 내부 `<img>`: `position: absolute`, `(-sx*scaleX, -sy*scaleY)`, `width/height = naturalWidth*scaleX / naturalHeight*scaleY` (onload 시점 확정).
   - crop 없으면 기존 직접 bbox 배치 유지.
4. CSS filter / mixBlendMode / opacity 는 `<img>` 자체에 그대로 적용 — wrapper 가 단순 clip 역할.

## 검증

| 검증 | 결과 |
|---|---|
| 페이지 2 박스 18 시각 (page.screenshot, overlay 포함) | 윈도우 chrome frame 단일 정상 표시, SVG export 와 시각 일치 ✓ |
| 페이지 2 layer JSON crop 필드 존재 | image #2 (LOWER): crop=(0, 0, 189900, 120958), image #3 (UPPER): crop=(0, 105958, 189900, 138540) ✓ |
| overlay 렌더 결과의 bbox/scale 일관성 | image #2 bbox.height=219.587 (commit 8ee17fd4 clip 적용), wrapper 가 source rect 0~120958 HU 만 표시 ✓ |
| 페이지 2 SVG export (변경 영향 없음 확인) | 8ee17fd4 commit 시점과 동일 ✓ |
| cargo test --release --lib | 1432 passed / 0 failed (Rust 변경은 JSON 출력 1 줄 추가, 기존 테스트 영향 없음) |
| cargo clippy --release --lib -- -D warnings | clean |
| npx tsc --noEmit | clean |

## 다음

Stage 2 — 페이지 4 박스 27 (flow layer 큰 PNG 비동기 디코드) 안전망 추가.
