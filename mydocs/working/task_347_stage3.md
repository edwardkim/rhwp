# Task #347 단계 3 완료 보고서 — 스코프 확장 (TAC 그림 렌더 + InFrontOfText 표 push-down)

## 배경

단계 2 완료 후 작업지시자 시각 검토에서 두 가지 추가 결함 확인:

1. **page 2 우측 박스 내부 텍스트가 너무 아래로 그려짐** — 박스 데코레이션 그림과 본문 사이 약 70px(=18.5mm) 공백
2. **page 4 우측 Q27/Q28 박스 프레임 누락 + 내용 겹침** — 박스 그림이 그려지지 않고 후속 InFrontOfText 표·문단들이 위로 올라붙어 충돌

작업지시자 지시에 따라 #347 스코프를 확장하여 동일 이슈로 처리.

## 추가 수정 1 — 글뒤로/글앞으로 그림 y_offset 진행 차단

**파일**: `src/renderer/layout/picture_footnote.rs::layout_body_picture`

**증상 원인**: pi=104(p2 우측)의 두 번째 글뒤로 그림(108×18.5mm)이 본문 흐름 점유가 아닌데도 `y_offset += pic_height`를 적용 → 후속 인라인 TAC 표(박스 본체)가 70px 아래로 밀림.

**수정**:
```rust
match (picture.common.vert_rel_to, picture.common.text_wrap) {
    (VertRelTo::Para, TextWrap::BehindText | TextWrap::InFrontOfText) => y_offset,
    (VertRelTo::Para, _) => y_offset + total_height,
    (VertRelTo::Page | VertRelTo::Paper, _) => y_offset,
}
```

## 추가 수정 2 — 인라인 TAC 그림 직접 렌더 분기 추가

**파일**: `src/renderer/layout.rs::layout_shape_item`

**증상 원인**: pi=181(p4 Q27)처럼 호스트 문단에 텍스트가 없고 TAC 그림(박스 프레임 시각)만 있는 경우, `FullParagraph` PageItem이 발행되지 않아 `paragraph_layout`이 호출되지 않고 → 인라인 TAC 그림이 영영 렌더되지 않음. 결과: 박스 프레임 그림 누락 + 인라인 그림이 점유해야 할 114mm 공간 미예약 → 후속 InFrontOfText 표·문단들이 같은 y에 piling.

**수정**: TAC Picture 분기에 다음 추가
```rust
let has_real_text = para.text.chars()
    .any(|c| c > '\u{001F}' && c != '\u{FFFC}');
if !has_real_text {
    // 직접 ImageNode 생성 + col_node에 push
    // set_inline_shape_position 등록 (후속 InFrontOfText 객체 para_y 기준)
    // result_y = pic_y + pic_h
}
```

호스트 문단에 실제 텍스트가 있는 경우 `paragraph_layout`이 인라인 그림을 렌더하므로 이 분기 미진입 (이중 렌더 방지).

## 추가 수정 3 — InFrontOfText/BehindText 표의 절대 위치 보존

**파일**: `src/renderer/layout/table_layout.rs::compute_table_y_position`

**증상 원인**: `vert=Para` 분기에서 `raw_y.max(y_start)`로 모든 wrap 모드를 본문 흐름 아래로 강제 이동. TopAndBottom은 자리 차지이므로 push-down이 필요하지만, 글뒤로/글앞으로는 절대 위치 오버레이여서 push-down 시 박스가 본문 흐름 따라 아래로 밀려남.

수정 1로 인라인 TAC 그림이 result_y를 그림 하단까지 진행 → 후속 InFrontOfText 표가 호출될 때 y_start = 그림 하단 → 기존 `raw_y.max(y_start)`로 강제 이동되는 문제 발현.

**수정**:
```rust
let pushed = if matches!(table_text_wrap, TextWrap::TopAndBottom) {
    raw_y.max(y_start)
} else {
    raw_y  // 글뒤로/글앞으로: 절대 위치 유지
};
pushed.clamp(body_top, body_bottom.max(body_top))
```

## 추가 수정 4 — InFrontOfText 표 z-order 보존 (TAC 그림이 표 위 덮는 회귀 방지)

**파일**: `src/renderer/layout.rs::layout_shape_item`

**증상 원인**: pi=189(p4 Q28)는 컨트롤 순서가 `[표(InFrontOfText), 그림(TAC)]`이라 PageItem 처리 순서대로 렌더하면 표가 먼저 그려지고 인라인 TAC 그림이 나중에 그려져 그림이 표 텍스트를 덮음 → 박스 본문 텍스트가 박스 프레임 그림에 가려져 보이지 않음.

**수정**: 인라인 TAC 그림을 col_node에 push할 때, 같은 문단에 이미 렌더된 Table 노드가 있으면 그 **앞에** insert하여 z-order(그림 → 표) 보존.

```rust
let insert_pos = col_node.children.iter().position(|c| {
    matches!(&c.node_type, RenderNodeType::Table(t)
        if t.para_index == Some(para_index))
});
if let Some(pos) = insert_pos {
    col_node.children.insert(pos, img_node);
} else {
    col_node.children.push(img_node);
}
```

## 시각 검증

- **p2 우측**: "Dear Rosydale City Marathon Racers" 박스 안 텍스트가 PDF처럼 박스 상단 직하 위치 (`task_347_exam_eng_p2_after.png`)
- **p4 우측 Q27**: Adamville City Pass Card 박스 프레임 + 내용 정상 (`task_347_exam_eng_p4_after.png`)
- **p4 우측 Q28**: Lockwood Snow Festival 박스 프레임 + 내용 (제목, When & Where, Special Activities, Transportation 등) 모두 정상 (`task_347_exam_eng_p4_q28_after.png`)
- **p1, 3, 5, 6, 7, 8**: 시각 회귀 없음

## 회귀 검증

- `cargo test --release` ✅ 1047+ passed, 0 failed
- 다른 샘플 (`biz_plan.hwp`, `aift.hwp`, `equation-lim.hwp`) export-svg 빌드 회귀 없음

## 영향 범위 (확장 후)

| 케이스 | 변화 | 회귀 |
|--------|------|------|
| 글뒤로/글앞으로 그림 + Para | y_offset 진행 차단 | 의도된 정정 |
| TAC 그림 + 텍스트 없는 문단 | 직접 렌더 추가 | 의도된 신규 동작 |
| TAC 그림 + 텍스트 있는 문단 | 분기 미진입 (paragraph_layout 처리) | 없음 |
| InFrontOfText/BehindText Para 표 | y_start push-down 미적용 | 의도된 정정 |
| TopAndBottom Para 표 | y_start push-down 유지 | 없음 |
| 단단 / TAC 표 / 중첩 표 | 미변경 | 없음 |
