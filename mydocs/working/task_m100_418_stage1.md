# Stage 1 완료 보고서 — Task M100 #418

## 작업 내용

`src/renderer/layout/paragraph_layout.rs:2042` 다음에 `set_inline_shape_position` 호출 추가.

### 변경

```diff
                                     line_node.children.push(img_node);
+                                    // [Task #418/#376] layout_shape_item 의 Task #347 분기 (빈 문단 +
+                                    // TAC Picture 직접 emit) 와 이중 렌더링되지 않도록 인라인 위치를
+                                    // 등록한다. layout_shape_item 은 등록된 경우 push 를 스킵한다.
+                                    tree.set_inline_shape_position(
+                                        section_index, para_index, tac_ci, img_x, img_y,
+                                    );
                                     img_x += tac_w;
```

## 검증

| 항목 | 결과 |
|------|------|
| `cargo build --lib` | ✅ 통과 |
| `cargo test --lib` | ✅ **1023 passed**, 0 failed (무회귀) |
| hwpspec p20 SVG 출력 | 6 image (Stage 1 만으로는 변화 없음 — layout.rs 가 아직 검사 안 함, 예상 동작) |

## 영향

- paragraph_layout 가 빈 문단 + TAC Picture emit 후 inline_shape_position 등록
- 등록 키: (section_index, para_index, tac_ci) → (img_x, img_y)
- 이 시점에서는 layout.rs 가 검사 로직 미적용이라 여전히 이중 emit 발생 — Stage 2 에서 layout.rs 가드 추가하면 해결

## 다음 단계

Stage 2 — `layout.rs` 의 `if !has_real_text` 분기에 `get_inline_shape_position` 가드 추가.

## 산출물

- 변경 파일: `src/renderer/layout/paragraph_layout.rs`
- 본 보고서: `mydocs/working/task_m100_418_stage1.md`
