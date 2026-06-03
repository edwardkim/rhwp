# Task #347 단계 2 완료 보고서 — Page 기준 영역 / Right 부호 정정

## 변경 내역

### `src/renderer/layout/table_layout.rs`

#### `compute_table_x_position` (L894 부근)

```rust
HorzRelTo::Page => {
    // Task #347: 본문 영역(body_area) 기준. 미설정 시 col_area 폴백.
    let body = self.current_body_area.get();
    if body.2 > 0.0 { (body.0, body.2) } else { (col_area.x, col_area.width) }
}
```

`HorzAlign::Right` / `Outside` 부호 정정:

```rust
// Task #347: picture_footnote.rs:185와 동일하게 - h_offset.
HorzAlign::Right | HorzAlign::Outside => ref_x + (ref_w - table_width).max(0.0) - h_offset,
```

#### `compute_table_y_position` (L965 부근)

```rust
crate::model::shape::VertRelTo::Page => {
    // Task #347: 본문 영역(body_area) 기준. 미설정 시 col_area 폴백.
    let body = self.current_body_area.get();
    if body.3 > 0.0 { (body.1, body.3) } else { (col_area.y, col_area.height) }
}
```

## 시각 검증

`samples/exam_eng.hwp` 페이지 2:

- **수정 전**: "이제 듣기 문제가 끝났습니다…" 박스가 우하단(컬럼 1 영역)으로 잘못 배치, 우측 단 상단에 불필요한 여백.
- **수정 후**: 박스가 좌측 단 하단에 정확히 위치, 우측 단 상단 여백 해소. PDF와 일치.

수정 후 이미지: [task_347_exam_eng_p2_after.png](task_347_exam_eng_p2_after.png)

## 회귀 검증

- `cargo build --release` ✅
- `cargo test --release` ✅ (전체 1047+ passed, 0 failed)

## 영향 범위 (재확인)

| 케이스 | 변화 | 회귀 |
|--------|------|------|
| 단단 문서 / HorzRelTo::Page 표 | body_area = body 영역과 동일 | 없음 |
| 다단 / HorzRelTo::Column | 분기 미변경 | 없음 |
| 다단 / HorzRelTo::Page (Left/Center/Right) | body_area 기준 + Right 부호 정정 | 의도된 정정 |
| HorzRelTo::Para / Paper | 미변경 | 없음 |
| TAC 표 / 중첩 표 | 별도 분기 | 없음 |

## 다음 단계

단계 3 — 최종 보고서 작성 + orders 갱신.
