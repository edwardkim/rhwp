# Task M100 #500 최종 결과 보고서

| 항목 | 내용 |
|------|------|
| 이슈 | [#500](https://github.com/edwardkim/rhwp/issues/500) |
| 마일스톤 | M100 (v1.0.0) |
| 브랜치 | `local/task500` |
| 상태 | 완료 |

## 1. 증상

`samples/exam_science.hwp` 페이지 2 7번 박스 안 paragraph p[1] 의 ctrl[1] 사각형 (tac=true, wrap=TopAndBottom, w=4724 h=1716, text_box="㉠") 이 paragraph 첫 줄(ls[0])에 그려져 PDF (둘째 줄 ls[1] "[㉠] 이다.") 와 어긋남.

## 2. 근본 원인

`src/renderer/layout/table_layout.rs` 의 `Control::Shape` `treat_as_char` 분기 (라인 ~1683) 가 사각형 좌표를 `inline_x, para_y_before_compose` 로 고정. multi-line paragraph 에서 사각형이 ls[1]+ 에 있을 때 첫 줄 기준 좌표라 잘못된 위치.

같은 함수의 `Control::Picture` 분기는 `target_line` 산출 + 줄 변경 시 `inline_x/tac_img_y` 리셋 로직을 갖고 multi-line 정상 처리. Shape 분기에 동일 로직 부재.

## 3. 수정

### `src/renderer/layout/table_layout.rs` Shape 분기

Picture 분기와 동일한 target_line/리셋 로직 적용:
- `target_line` 산출 (`composed.tac_controls` 에서 ctrl_idx 의 abs_pos 로 줄 식별)
- `target_line > current_tac_line` 시 `inline_x/tac_img_y` 리셋
- `shape_area.y` 와 `layout_cell_shape` 의 para_y 인자를 `tac_img_y` 로 변경

`current_tac_line, tac_img_y, inline_x, tac_line_widths` 모두 outer scope (`for ctrl_idx, ctrl in para.controls`) 변수이므로 Picture 와 Shape 가 공유.

## 4. 검증

### 4-1. 핵심 회귀 케이스 (exam_science p2 7번 박스)

| 측정 | Before | After |
|------|--------|-------|
| 사각형 y | 206.74 (= para_y_before_compose) | **249.68** (= ls[1] 시작) |
| 사각형 x | 104.07 | **97.07** (ls[1] 시작) |

PDF 시각 비교 (`samples/pdf/hwp2022/exam_science.pdf` p2 7번 박스):
- ls[0]: "분자당 구성 원자 수가 3인 분자의 분자 모양은 모두"
- ls[1]: "[㉠] 이다." ← 사각형 위치 ✓ PDF 정합

### 4-2. 단위 + 통합 테스트

- `cargo test --release --lib`: **1103 passed; 0 failed; 1 ignored**
- `cargo test --release --tests`: 모든 통합 테스트 통과

### 4-3. 광범위 회귀 (7 샘플 SVG byte diff)

| 샘플 | total | same | diff |
|------|-------|------|------|
| exam_kor | 20 | 20 | 0 |
| exam_eng | 8 | 8 | 0 |
| **exam_science** | **4** | **3** | **1 (의도)** |
| exam_math | 20 | 20 | 0 |
| synam-001 | 35 | 35 | 0 |
| aift | 77 | 77 | 0 |
| 2010-01-06 | 6 | 6 | 0 |

**170 페이지 중 의도된 1 페이지 정정 외 모두 byte 동일** — 회귀 0건.

## 5. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/table_layout.rs` | Shape 분기 (라인 ~1597) 에 target_line/리셋 로직 추가 + shape_area/layout_cell_shape 의 y 좌표를 tac_img_y 로 변경 |

## 6. 영향 범위

| 케이스 | 영향 |
|--------|------|
| 셀 안 paragraph 의 인라인 Shape (treat_as_char), multi-line paragraph 의 ls[1]+ 위치 | **정정** (ls 좌표 기준 정확화) |
| 셀 안 paragraph 의 인라인 Shape, single-line 또는 ls[0] 위치 | 변화 없음 (target_line=0, tac_img_y=para_y_before_compose 유지) |
| 셀 안 Picture, Shape 비-treat_as_char | 변화 없음 |
| 본문 (셀 외부) 의 Shape | 변화 없음 (다른 코드 경로) |

## 7. 후속

- 본 task 의 정정으로 #500 본질 결함 해결
- #496 (보류) 는 본질이 다름 (`layout_inline_table_paragraph` multi-row 표 + multi-line 텍스트 처리 한계) — 본 task 와 별개로 유지

## 8. 요약

- exam_science p2 7번 박스 ㉠ 사각형 ls[1] 정상 위치 ✓
- PDF 시각 일치 ✓
- 회귀 없음 (단위 1103 + 통합 + 7 샘플 169/170 byte 동일) ✓
