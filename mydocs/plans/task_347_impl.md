# Task #347 구현 계획서

## 1. 개요

`compute_table_x_position` / `compute_table_y_position`의 `HorzRelTo::Page` / `VertRelTo::Page` 기준 영역과 `HorzAlign::Right` 부호 오류를 정정하여 다단 문서에서 절대 좌표 표 배치가 PDF와 일치하도록 한다.

## 2. 설계 방침

- **body_area 전달 통로**: `self.current_paper_width: Cell<f64>` 와 동일한 패턴으로 `current_body_area: Cell<(f64, f64, f64, f64)>` 를 추가한다. 페이지 단위로 1회 set, `compute_table_*`에서 get. 기존 `layout_table` 시그니처는 변경하지 않는다 (호출부 9+ 곳 수정 회피).
- **시맨틱 통일**: picture_footnote 경로(`HorzRelTo::Page => body_area`, `Right => ref_x + ref_w - obj_w - h_offset`)를 정답으로 삼아 표 경로를 동일화.
- **회귀 방지**: 단단 문서에서는 `col_area = body_area` 이므로 동작 동일. 다단·절대좌표 표(글앞으로/자리차지/글뒤로)에서만 변화.

## 3. 구현 단계 (3단계)

### 단계 1 — body_area 통로 추가 (refactor, 동작 변경 없음)

**파일**: `src/renderer/layout.rs`, `src/renderer/layout/table_layout.rs`

1. `LayoutEngine`에 필드 추가:
   ```rust
   current_body_area: std::cell::Cell<(f64, f64, f64, f64)>,
   ```
   `LayoutEngine::new`에서 `(0.0, 0.0, 0.0, 0.0)`로 초기화.

2. 페이지/단 레이아웃 시작점(`current_paper_width.set(...)` 부근, layout.rs:1257)에서:
   ```rust
   let ba = &layout.body_area;
   self.current_body_area.set((ba.x, ba.y, ba.width, ba.height));
   ```

3. 이 단계에서는 `compute_table_x/y_position` 본문은 변경하지 않는다.

**검증**: `cargo build`, `cargo test`. 기존 SVG 출력 동일.

### 단계 2 — Page 기준 정정 + Right 부호 수정

**파일**: `src/renderer/layout/table_layout.rs`

`compute_table_x_position` (L894):

```rust
// 표 자체 위치 속성
let horz_rel_to = table.common.horz_rel_to;
let horz_align = table.common.horz_align;
let h_offset = hwpunit_to_px(table.common.horizontal_offset as i32, self.dpi);
let body = self.current_body_area.get();  // (x, y, w, h)
let (ref_x, ref_w) = match horz_rel_to {
    HorzRelTo::Paper => {
        let paper_w = paper_width.unwrap_or(/* 기존 fallback */);
        (0.0, paper_w)
    }
    HorzRelTo::Page => {
        // body_area 미설정(0.0) 시 기존 col_area로 폴백
        if body.2 > 0.0 { (body.0, body.2) } else { (col_area.x, col_area.width) }
    }
    HorzRelTo::Para => (col_area.x + host_margin_left, col_area.width - host_margin_left),
    _ => (col_area.x, col_area.width),
};
match horz_align {
    HorzAlign::Left | HorzAlign::Inside => ref_x + h_offset,
    HorzAlign::Center => ref_x + (ref_w - table_width).max(0.0) / 2.0 + h_offset,
    // 부호 정정: picture_footnote.rs:185와 동일하게 - h_offset
    HorzAlign::Right | HorzAlign::Outside => ref_x + (ref_w - table_width).max(0.0) - h_offset,
}
```

`compute_table_y_position` (L965) — `VertRelTo::Page` 분기에서:

```rust
let body = self.current_body_area.get();
let (ref_y, ref_h) = match vert_rel_to {
    VertRelTo::Page => {
        if body.3 > 0.0 { (body.1, body.3) } else { (col_area.y, col_area.height) }
    }
    VertRelTo::Para => (anchor_y, col_area.height - (anchor_y - col_area.y).max(0.0)),
    VertRelTo::Paper => (0.0, page_h_approx),
};
```

**검증**:
- `cargo build --release`
- `rhwp export-svg samples/exam_eng.hwp -o output/svg/exam_eng/`
- p.2 시각 확인 — "이제 듣기…" 박스가 좌측 단 하단에 위치, 우측 단 상단 여백 해소
- `cargo test` 전체 회귀
- `rhwp ir-diff samples/<멀티-단 샘플>.hwpx samples/<...>.hwp` 회귀 없음 확인 (가용 샘플 한해)

### 단계 3 — 회귀 샘플 + 보고서

1. `output/re/` 또는 단계별 보고서에 p.2 SVG 첨부
2. `mydocs/working/task_347_stage{1,2}.md` 작성 (단계 1·2 완료 시점)
3. `mydocs/working/task_347_stage3.md` (이 단계 완료)
4. `mydocs/report/task_347_report.md` 최종 보고
5. `mydocs/orders/20260426.md` 갱신

## 4. 영향 범위 / 회귀 매트릭스

| 케이스 | 변화 | 회귀 |
|--------|------|------|
| 단단 문서, HorzRelTo::Page 표 | col_area = body_area → 동일 | 없음 |
| 다단 문서, HorzRelTo::Column 표 | 분기 미변경 | 없음 |
| 다단 문서, HorzRelTo::Page + Left/Center | ref 영역만 body로 변경 | 시각 차이 가능 (정정) |
| 다단 문서, HorzRelTo::Page + Right | ref 영역 + 부호 정정 | 시각 차이 가능 (정정) |
| HorzRelTo::Para / Paper | 미변경 | 없음 |
| TAC(글자처럼) 표 | 별도 분기 (L915) | 없음 |
| 중첩 표 (depth > 0) | 별도 분기 (L951) | 없음 |

## 5. 위험 및 완화

- **위험**: body_area가 0인 경로(테스트 단위 호출, 바탕쪽 등)에서 잘못된 계산 가능.
  - **완화**: `body.2 > 0.0`/`body.3 > 0.0` 가드로 기존 `col_area` 폴백 유지.
- **위험**: VertRelTo::Page 정정으로 기존 다단 문서 표 위치 변동 가능.
  - **완화**: 단계 2 검증 시 `samples/` 내 다단 샘플(특히 절대좌표 표 포함)을 추출해 시각 비교. 차이 발생 시 PDF와 비교하여 정정 방향이 맞는지 확인.

## 6. 산출물 체크리스트

- [ ] `src/renderer/layout.rs` — `current_body_area` 필드 + set 위치
- [ ] `src/renderer/layout/table_layout.rs` — L942/L949/L994 정정
- [ ] 시각 검증: `output/svg/exam_eng/exam_eng_002.svg` PDF와 일치
- [ ] `cargo test` 전체 통과
- [ ] 보고서 3종 (stage1, stage2, stage3 또는 통합)
- [ ] `mydocs/orders/20260426.md` 갱신
