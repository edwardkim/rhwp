# Stage 2 완료 보고서 — Task M100 #418

## 작업 내용

`src/renderer/layout.rs:2554` 의 `if !has_real_text` 분기에 `get_inline_shape_position` 가드 추가.

### 변경

```diff
                         let has_real_text = para.text.chars()
                             .any(|c| c > '\u{001F}' && c != '\u{FFFC}');
+                        // [Task #418/#376] paragraph_layout 의 빈 문단 + TAC Picture 분기에서
+                        // 이미 ImageNode 가 emit 되어 inline_shape_position 이 등록된 경우,
+                        // 여기서 또 push 하면 이중 emit 이 된다. 등록된 경우 push 를 스킵하고
+                        // result_y 만 갱신한다.
+                        let already_registered = tree.get_inline_shape_position(
+                            page_content.section_index, para_index, control_index,
+                        ).is_some();
-                        if !has_real_text {
+                        if !has_real_text && !already_registered {
                             let bin_data_id = pic.image_attr.bin_data_id;
                             ...
                             result_y = pic_y + pic_h;
+                        } else if !has_real_text && already_registered {
+                            // [Task #418/#376] paragraph_layout 가 이미 emit 함 — push 스킵, result_y 만 갱신
+                            result_y = pic_y + pic_h;
                         }
```

## 검증

| 항목 | 결과 |
|------|------|
| `cargo build --lib` | ✅ 통과 |
| `cargo test --lib` | ✅ **1023 passed**, 0 failed (무회귀) |
| `cargo test --test svg_snapshot` | ✅ **6/6 passed** (다른 샘플 무회귀) |
| **hwpspec p20 SVG image 개수** | ✅ **6 → 3** (정상화) |
| Task #416 효과 (p1 페이지 표지) | ✅ 보존 (`<image width="793.72"`) |

### hwpspec p20 image y 좌표 변화

| 정정 전 (6 개) | 정정 후 (3 개) |
|--------------|--------------|
| 445.43 (paragraph_layout) | **445.43** (paragraph_layout) |
| 442.76 (layout.rs 중복) | (제거) |
| 604.09 (paragraph_layout) | **604.09** (paragraph_layout) |
| 601.43 (layout.rs 중복) | (제거) |
| 741.43 (paragraph_layout) | **741.43** (paragraph_layout) |
| 738.76 (layout.rs 중복) | (제거) |

paragraph_layout 가 emit 한 좌표만 남음. layout.rs 의 중복 emit (약 2.67px 위 좌표) 정확히 제거.

## 영향

- `samples/hwpspec.hwp` 20 페이지의 빈 문단 + TAC Picture 이중 출력 해소
- 다른 샘플 무회귀 (svg_snapshot 6/6 + cargo test --lib 1023)
- Task #416 페이지 표지 효과 보존

## 다음 단계

Stage 3 — 회귀 테스트 추가 + 자동 검증 종합 + WASM 빌드 + 시각 판정.

## 산출물

- 변경 파일: `src/renderer/layout.rs`
- 시각 산출물: `output/svg/issue-418-fixed/hwpspec_020.svg`
- 본 보고서: `mydocs/working/task_m100_418_stage2.md`
