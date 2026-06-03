# Task #249: 구현계획서

> 구현계획서 | 2026-04-22
> Issue: [#249](https://github.com/edwardkim/rhwp/issues/249)

---

## 단계 1: PUA 심볼 문자 렌더링

### 수정 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/svg.rs` | `draw_text`에 `map_pua_bullet_char()` 호출 추가 |
| `src/renderer/web_canvas.rs` | `draw_text`에 `map_pua_bullet_char()` 호출 추가 |
| `src/renderer/html.rs` | 텍스트 렌더링에 `map_pua_bullet_char()` 호출 추가 |

### 구현 내용

```rust
// 각 렌더러 draw_text 내
let ch = map_pua_bullet_char(ch);  // PUA → 유니코드 표준 문자 변환
```

PUA 변환 매핑 (Wingdings 기준):
- U+F028~F02F: ⇩⇧⇦⇨ 등 화살표
- U+F0A7: ● 도형
- U+F0FC: ✔ 체크마크
- 기타 U+F000~F0FF 범위

### 완료 기준
- 793개 테스트 전체 통과
- Visual Diff: PUA 문자가 포함된 페이지에서 □ → 정상 심볼 표시

---

## 단계 2: 문단 border_fill margin 반영

### 수정 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs` | border_fill rect에 margin 적용 |

### 구현 내용

```rust
// paragraph_layout.rs: border_fill rect 계산
let border_rect = Rect {
    x: para_x + margin_left,
    y: border_y,
    width: para_width - margin_left - margin_right,
    height: border_height,
};
```

### 완료 기준
- 793개 테스트 전체 통과
- Visual Diff: 문단 테두리 박스가 텍스트 영역과 일치

---

## 단계 3: 표 외곽 테두리 fallback + clip_rect 개선

### 수정 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/table_layout.rs` | border_fill_id fallback 로직 추가, 셀 커버 영역 제외 |
| `src/renderer/layout.rs` | clip_rect를 콘텐츠 레이아웃 후 확정 |

### 구현 내용

**table_layout.rs:**
```rust
// border_fill_id가 설정된 경우 외곽 테두리 그리기
if let Some(border_fill_id) = table.border_fill_id {
    // 셀로 커버된 영역을 제외한 외곽 영역에만 fallback 적용
    draw_table_outer_border(border_fill_id, uncovered_rects);
}
```

**layout.rs:**
```rust
// clip_rect: 콘텐츠 레이아웃 후 자식 노드 bbox를 반영하여 확정
if self.clip_enabled.get() {
    let mut clip = body_bbox;
    expand_clip(&mut clip, &body_node);  // 자식 노드 bbox 재귀 반영
    body_node.set_clip_rect(Some(clip));
}
```

### 완료 기준
- 793개 테스트 전체 통과
- Visual Diff: 표 외곽 테두리가 한컴 렌더링과 일치
- 표 외곽 테두리가 body_area clip으로 잘리지 않음
