# 구현 계획서 — Task M100 #418

## 이슈

[#418](https://github.com/edwardkim/rhwp/issues/418) — `samples/hwpspec.hwp` 20 페이지 이미지 이중 출력 (PR 처리 후 회귀)

## 변경 위치

### 변경 1 — `src/renderer/layout/paragraph_layout.rs:2042`

빈 문단 + TAC Picture 분기에서 `set_inline_shape_position` 호출 추가.

**현재**:
```rust
                                line_node.children.push(img_node);
                                img_x += tac_w;
                            }
                        }
                    }
```

**정정 후**:
```rust
                                line_node.children.push(img_node);
                                // [Task #418/#376] layout_shape_item 의 Task #347 분기 (빈 문단 +
                                // TAC Picture 직접 emit) 와 이중 렌더링되지 않도록 인라인 위치를
                                // 등록한다. layout_shape_item 은 등록된 경우 스킵한다.
                                tree.set_inline_shape_position(
                                    section_index, para_index, tac_ci, img_x, img_y,
                                );
                                img_x += tac_w;
                            }
                        }
                    }
```

### 변경 2 — `src/renderer/layout.rs:2554`

`layout_shape_item` 의 빈 문단 + TAC Picture 분기에 `get_inline_shape_position` 가드 추가.

**현재 (line 2552-2597)**:
```rust
let has_real_text = para.text.chars()
    .any(|c| c > '\u{001F}' && c != '\u{FFFC}');
if !has_real_text {
    let bin_data_id = pic.image_attr.bin_data_id;
    let image_data = find_bin_data(bin_data_content, bin_data_id)
        .map(|c| c.data.clone());
    // ... ImageNode 생성 ...
    if let Some(pos) = insert_pos {
        col_node.children.insert(pos, img_node);
    } else {
        col_node.children.push(img_node);
    }
    tree.set_inline_shape_position(
        page_content.section_index, para_index, control_index, pic_x, pic_y,
    );
    result_y = pic_y + pic_h;
}
```

**정정 후**:
```rust
let has_real_text = para.text.chars()
    .any(|c| c > '\u{001F}' && c != '\u{FFFC}');
// [Task #418/#376] paragraph_layout 의 빈 문단 + TAC Picture 분기에서 이미 ImageNode 가
// emit 되어 inline_shape_position 이 등록된 경우, ImageNode push 를 스킵하고
// result_y 만 갱신하여 이중 emit 을 방지한다.
let already_registered = tree.get_inline_shape_position(
    page_content.section_index, para_index, control_index,
).is_some();
if !has_real_text && !already_registered {
    let bin_data_id = pic.image_attr.bin_data_id;
    let image_data = find_bin_data(bin_data_content, bin_data_id)
        .map(|c| c.data.clone());
    // ... ImageNode 생성 ...
    if let Some(pos) = insert_pos {
        col_node.children.insert(pos, img_node);
    } else {
        col_node.children.push(img_node);
    }
    tree.set_inline_shape_position(
        page_content.section_index, para_index, control_index, pic_x, pic_y,
    );
    result_y = pic_y + pic_h;
} else if !has_real_text && already_registered {
    // 이미 paragraph_layout 에서 emit 됨 — result_y 만 갱신
    result_y = pic_y + pic_h;
}
```

## API 시그니처 확인

`src/renderer/render_tree.rs:723,728`:

```rust
pub fn set_inline_shape_position(&mut self, sec: usize, para: usize, ctrl: usize, x: f64, y: f64);
pub fn get_inline_shape_position(&self, sec: usize, para: usize, ctrl: usize) -> Option<(f64, f64)>;
```

## Stage 1 — `paragraph_layout.rs` 정정

### 변경

`paragraph_layout.rs:2042` 다음에 `tree.set_inline_shape_position(...)` 호출 추가.

### 검증

- `cargo build --lib` 통과
- `cargo test --lib` 무회귀 (1023 → 1023)
- `cargo run --release --bin rhwp -- export-svg samples/hwpspec.hwp -p 19 -o /tmp/stage1/`
  - **이 시점에서는 아직 layout.rs 정정 전이라 6 → 6 (변화 없음 예상)** — paragraph_layout 가 set 만 호출, layout.rs 가 검사 안 함
  - 또는 paragraph_layout 가 같은 키로 덮어쓰는 형태가 될 수 있어 좌표 변화 가능성 점검

### 부수 효과 점검

`set_inline_shape_position` 가 같은 (sec, para, ctrl) 키에 이미 layout.rs 가 등록한 값이 있으면 덮어쓰는 형태인지 점검 — render_tree.rs 의 구현 확인 필요. 일반적으로 HashMap insert 라 마지막 값이 남음.

## Stage 2 — `layout.rs` 가드 추가

### 변경

`layout.rs:2554` 의 `if !has_real_text` 분기에 `get_inline_shape_position` 가드.

### 검증

- `cargo build --lib` 통과
- `cargo test --lib` 무회귀
- `cargo run --release --bin rhwp -- export-svg samples/hwpspec.hwp -p 19 -o /tmp/stage2/`
  - **`<image>` 6 → 3 으로 감소 확인**
- 페이지 표지 (Task #416 효과) 보존 확인 — `cargo run -- export-svg samples/hwpspec.hwp -p 0 ...`

## Stage 3 — 회귀 테스트 + 자동 검증 + WASM 빌드

### 신규 회귀 테스트

`src/wasm_api/tests.rs` 또는 통합 테스트로 추가:

```rust
/// Task #418: hwpspec.hwp 20 페이지의 빈 문단 + TAC Picture 이중 emit 회귀 방지
#[test]
fn issue_418_hwpspec_p20_no_duplicate_image_emit() {
    use std::path::Path;
    let path = Path::new("samples/hwpspec.hwp");
    if !path.exists() { return; }

    let bytes = std::fs::read(path).expect("hwpspec.hwp 로드");
    // 페이지 19 (0-indexed) SVG 출력 후 <image> 개수 검증
    // 정상: 3개 (pi=83, 86, 89 의 TAC Picture 각 1회)
    // 회귀: 6개 (이중 emit)
    let svg = render_page_svg(&bytes, 19).expect("SVG 출력");
    let image_count = svg.matches("<image").count();
    assert_eq!(image_count, 3,
        "회귀: 빈 문단 + TAC Picture 이중 emit (Task #376 정정 누락 회귀)");
}
```

(실제 API 시그니처는 단계 진행 중 점검 — render_page_svg 등의 정확한 호출 방식 확인 후 작성)

### 자동 검증

| 항목 | 명령 |
|------|------|
| cargo lib test | `cargo test --lib` (1023 → 1024+) |
| svg_snapshot | `cargo test --test svg_snapshot` (6/6) |
| clippy | `cargo clippy --lib -- -D warnings` |
| WASM 빌드 | `docker compose --env-file .env.docker run --rm wasm` |

### 시각 검증

| 시나리오 | 기대 |
|----------|------|
| `samples/hwpspec.hwp` 1 페이지 (Task #416 효과) | 페이지 표지 정상 (16×13 → 793×1121) |
| `samples/hwpspec.hwp` 20 페이지 | `<image>` 6 → 3, 이중 출력 해소 |
| 다른 hwp 샘플 (k-water-rfp, exam_eng 등) | 무회귀 |

작업지시자 시각 판정.

## 위험 / 주의

### 1. paragraph_layout 와 layout.rs 의 호출 순서

`layout_paragraph_with_layout` (paragraph_layout) → `layout_shape_item` 순서이므로:
- paragraph_layout 가 먼저 등록 → layout.rs 가 검사 → 정합

검증 — Stage 2 후 SVG 의 image 개수가 3 으로 감소하면 순서 정합 확인.

### 2. 다른 케이스 영향

| 케이스 | 본 변경 영향 |
|--------|------------|
| 텍스트 + TAC Picture (`has_real_text == true`) | layout.rs 분기 진입 안 함 — 영향 없음 |
| TAC=false Picture | paragraph_layout 본 분기 진입 안 함, layout.rs 의 `if pic.common.treat_as_char` 분기 진입 안 함 — 영향 없음 |
| Equation TAC | 이미 line 2133 에서 set 호출 중 — 영향 없음 |
| 셀 내부 TAC Picture (`cell_ctx.is_some()`) | paragraph_layout 본 분기 (`cell_ctx.is_none()`) 진입 안 함 — 영향 없음 |

### 3. inline_shape_position 의 semantic 변화

현재 paragraph_layout 의 다른 분기 (Equation 등) 는 set 호출. layout.rs 의 분기는 set + push. 본 변경으로 paragraph_layout Picture 도 set 호출 → 다른 곳에서 같은 키를 참조하는 코드 영향 점검:

```bash
grep -rn "get_inline_shape_position" src/
```

영향 받는 호출자가 있다면 별도 점검 (주로 후속 InFrontOfText 표 / 객체 위치 계산용).

## 산출물

| 단계 | 산출물 |
|------|--------|
| Stage 1 | `mydocs/working/task_m100_418_stage1.md` |
| Stage 2 | `mydocs/working/task_m100_418_stage2.md` |
| Stage 3 | `mydocs/working/task_m100_418_stage3.md` |
| 최종 | `mydocs/report/task_m100_418_report.md` |

## 다음 단계

본 구현 계획서 승인 → Stage 1 진행.
