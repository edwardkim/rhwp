# Task #347 단계 1 완료 보고서 — body_area 통로 추가

## 변경 내역

### `src/renderer/layout.rs`

1. `LayoutEngine` 구조체에 필드 추가:
   ```rust
   /// 현재 페이지 본문 영역 (표 HorzRelTo::Page / VertRelTo::Page 위치 계산용)
   /// (x, y, width, height). 미설정 시 (0, 0, 0, 0) — 호출부에서 col_area로 폴백.
   current_body_area: std::cell::Cell<(f64, f64, f64, f64)>,
   ```
2. `LayoutEngine::new`에 초기값 `(0.0, 0.0, 0.0, 0.0)` 추가.
3. 페이지 단 레이아웃 시작점(L1257 부근)에서 `layout.body_area`를 읽어 set:
   ```rust
   let ba = &layout.body_area;
   self.current_body_area.set((ba.x, ba.y, ba.width, ba.height));
   ```

## 동작 변경

없음. `compute_table_x/y_position`은 이 단계에서 본문 미수정.

## 검증

- `cargo build --release` ✅
- `cargo test --release --lib` ✅ (1000 passed)

## 다음 단계

단계 2 — `HorzRelTo::Page` / `VertRelTo::Page` 기준 영역과 `HorzAlign::Right` 부호 정정.
