# Stage 2 — 코드 수정 완료 보고서

**Task**: #577 — 셀 내부 단독 TopAndBottom 이미지 1라인 오프셋
**브랜치**: `local/task577`
**단계**: 2/4 (코드 수정)

---

## 1. 변경 요약

`src/renderer/layout/table_layout.rs:1624..1648` 의 비-TAC Picture 분기에 anchor_y 도입.

### Diff (요지)

```rust
} else {
    // 비-인라인(자리차지/글뒤로/글앞으로) 이미지:
    let pic_w = ...;
    let pic_h = ...;
+   // [Task #577] TopAndBottom + vert_rel_to=Para 인 셀 내부 이미지는
+   // anchor 라인이 이미지에 의해 displaced 되므로, layout_composed_paragraph
+   // 가 advance 시킨 para_y 가 아닌 anchor 시점(para_y_before_compose)을 기준
+   // 으로 해야 cell-clip 영역 내부에 정확히 배치된다.
+   let anchor_y = if matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
+                  && matches!(pic.common.vert_rel_to, VertRelTo::Para)
+   { para_y_before_compose } else { para_y };
    let cell_area = LayoutRect {
-       y: para_y,
-       height: (inner_area.height - (para_y - inner_area.y)).max(0.0),
+       y: anchor_y,
+       height: (inner_area.height - (anchor_y - inner_area.y)).max(0.0),
        ..inner_area
    };
    let (pic_x, pic_y) = self.compute_object_position(
        &pic.common, pic_w, pic_h,
        &cell_area, &inner_area, &inner_area, &inner_area,
-       para_y, para_alignment,
+       anchor_y, para_alignment,
    );
```

`para_y += pic_h;` (단락 종료 후 다음 단락 시작점 산출)는 무변경. anchor_y 변경은 좌표 계산만 영향.

## 2. 빌드 / 테스트

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ 성공 |
| `cargo test --release --lib` | ✅ 1125 passed; 0 failed; 2 ignored |
| `cargo clippy --release -- -D warnings` | ⚠ 사전 존재 에러 2건 (`src/renderer/layout.rs:313-314` doc_lazy_continuation) — 본 타스크 변경과 무관, 변경 전 동일하게 발생 (stash 후 재실행으로 확인). 본 PR 범위 외 |

## 3. 리스크 점검

- 수정 위치는 `table_layout.rs` 셀 처리 루프의 비-TAC Picture 분기 한 곳.
- `picture_footnote.rs::compute_object_position` 자체는 무변경 → 본문(셀 외) 경로 회귀 영향 없음.
- 조건 `text_wrap==TopAndBottom AND vert_rel_to==Para` 외에는 기존 동작(`para_y`) 그대로.

## 4. 다음 단계

Stage 3 — `exam_science.hwp` 1페이지 보기 5개 좌표 재측정 + `mel-001.hwp` 회귀 SVG diff 비교.

## 5. 승인 요청

Stage 2 결과 검토 후 Stage 3 진행 승인 부탁드립니다.
