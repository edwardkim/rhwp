# 빈 문단 + TAC Picture 이중 emit 회귀

## 결함 패턴

빈 문단 (`text == ""`) + `treat_as_char (TAC) = true` Picture 컨트롤이 두 경로에서 동시에 ImageNode 를 emit:

1. **`paragraph_layout.rs`** (line 2008-2048) — 빈 runs + TAC offsets 분기
2. **`layout.rs::layout_shape_item`** (line 2554-2604) — Task #347 의 빈 문단 분기

## 증상

- SVG 에 같은 이미지가 두 번 출현
- y 좌표 약 2.67px 차이 (paragraph_layout 의 baseline 기준 vs layout_shape_item 의 para_start_y 기준 차이)
- x / width / height / base64 데이터 동일

## 회귀 이력

### Task #376 (2026-04-27, @planet6897, commit `45419a2`)

**결함 발견**: `2025년 기부·답례품 실적 보고서_최종형태 확정.hwpx` p7 의 pi=51 차트 이중 emit.

**정정 (A안)**:
1. `paragraph_layout` Picture emit 후 `set_inline_shape_position` 호출
2. `layout_shape_item` 의 빈 문단 분기에 `get_inline_shape_position` 가드 추가 — 등록된 경우 push 스킵

**문제**: commit `45419a2` 가 devel 에 머지되지 않음. 이슈는 close 됐지만 코드는 임시 브랜치 `pr-360-head` 에만 존재 → 회귀 그대로 남음.

### Task #418 (2026-04-28, 재정정)

**증상 발견**: `samples/hwpspec.hwp` 20 페이지 (s2:pi=83/86/89, 모두 빈 문단 + TAC=true Picture) 이중 출력 — `<image>` 6 개 (3 쌍).

**정정**: Task #376 의 정정 코드를 정확히 재적용 + 회귀 테스트 (`tests/issue_418.rs`) 추가.

## 정정 코드

### `src/renderer/layout/paragraph_layout.rs` (Picture push 후)

```rust
line_node.children.push(img_node);
// [Task #418/#376] layout_shape_item 의 Task #347 분기 (빈 문단 + TAC Picture
// 직접 emit) 와 이중 렌더링되지 않도록 인라인 위치를 등록한다.
tree.set_inline_shape_position(
    section_index, para_index, tac_ci, img_x, img_y,
);
img_x += tac_w;
```

### `src/renderer/layout.rs::layout_shape_item` (빈 문단 분기)

```rust
let has_real_text = para.text.chars()
    .any(|c| c > '\u{001F}' && c != '\u{FFFC}');
let already_registered = tree.get_inline_shape_position(
    page_content.section_index, para_index, control_index,
).is_some();
if !has_real_text && !already_registered {
    // ImageNode 생성 + push + set_inline_shape_position
    // ... result_y = pic_y + pic_h;
} else if !has_real_text && already_registered {
    // paragraph_layout 가 이미 emit — push 스킵, result_y 만 갱신
    result_y = pic_y + pic_h;
}
```

## 회귀 방지

### 단위 / 통합 테스트

`tests/issue_418.rs`:
```rust
#[test]
fn hwpspec_page20_no_duplicate_image_emit() {
    let svg = doc.render_page_svg_native(19).expect("...");
    let count = svg.matches("<image").count();
    assert_eq!(count, 3, "기대 3, 실제 {count}");
}
```

### 영역 수정 시 점검 절차

`paragraph_layout.rs` 또는 `layout.rs::layout_shape_item` 수정 시:

1. 본 트러블슈팅 문서 정독
2. 두 경로의 emit 동기화 (`set_inline_shape_position` ↔ `get_inline_shape_position`) 보존 확인
3. 회귀 테스트 (`cargo test --test issue_418`) 통과 확인

### close 절차 강화

이슈 close 전 정정 commit 의 devel 머지 검증 (메모리 `feedback_close_issue_verify_merged.md`):

```bash
git merge-base --is-ancestor <commit-hash> devel && echo "OK" || echo "MISSING"
```

`MISSING` 결과면 이슈 close 보류 + 머지 + push 후 close.

## 영향 받는 케이스

- 빈 문단 (`text == ""` 또는 `\u{FFFC}` 만) + TAC=true Picture
- `cell_ctx.is_none()` (셀 외부) + `all_runs_empty`
- TopAndBottom / BehindText / 기타 wrap 무관

## 영향 없는 케이스

| 케이스 | 사유 |
|--------|------|
| 텍스트 + TAC Picture | `has_real_text == true` 라 layout_shape_item 분기 진입 안 함 |
| TAC=false Picture | paragraph_layout 본 분기 진입 안 함, layout_shape_item `if pic.common.treat_as_char` 진입 안 함 |
| Equation TAC | paragraph_layout 의 Equation 분기 (line 2133) 가 이미 set 호출 중 |
| 셀 내부 TAC Picture | paragraph_layout 본 분기 (`cell_ctx.is_none()`) 진입 안 함, table_layout 가 처리 |

## 참고

- 이슈: [#376](https://github.com/edwardkim/rhwp/issues/376) (CLOSED, 머지 누락), [#418](https://github.com/edwardkim/rhwp/issues/418)
- 회귀 origin commit: [`45419a2`](https://github.com/edwardkim/rhwp/commit/45419a2) (Task #376, devel 미머지)
- 재정정 task: #418 (2026-04-28)
- 관련 task: #347 (TAC 그림 z-order), #287 (빈 runs TAC 수식 인라인), #301 (z-table 셀 수식 이중 렌더링)
