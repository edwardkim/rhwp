# Task M100 #1379 3단계 완료 보고서 — 글상자 경로 공유 직렬화 전환 + rect/drawText 보존 + 셀·글상자 colPr 방출

- 구현계획서: `mydocs/plans/task_m100_1379_impl.md` 3단계 (+ 2단계 승인 사항 ① colPr 포함)
- 브랜치: `local/task1379`

## 1. 수행 내역

### 1.1 글상자(drawText) 직렬화를 본문 공유 경로로 전환 (`src/serializer/hwpx/shape.rs`)

- `write_draw_text_paragraph`(텍스트-only 자체 방출)를 삭제하고, 셀 경로(2단계)와
  동일한 **`render_paragraph_parts()` 공유 경로**로 교체
  - 컨트롤 슬롯(글상자 내 pic·중첩 표·각주 등) 방출 + run 분할 + char_shapes 경계 보존
  - lineseg IR 보존/fallback, `ctx.next_para_id()` 채번 동승
- `section.rs`의 `render_text_runs`(구 경로 유일 사용처)는 제거

### 1.2 drawText 속성 보존

- `textDirection`: **VERTICAL/VERTICALALL 구분 보존** — `list_attr` 하위 3비트만으로는
  구분 불가하므로 모델에 `TextBox.vertical_all: bool` 추가, 파서(parse_draw_text)에서
  원문 문자열로 세팅, serializer 역방출
- `vertAlign`: `TextBox.vertical_align`(Top/Center/Bottom) 역매핑
- 고정 속성 `lastWidth`/`name=""`/`editable="0"` (tbox-v-flow-01 실측 순서)
- `textMargin`은 **subList 뒤** 방출 (footnote-tbox-01 실측 순서) — 네 여백 모두 0이면
  원본 부재로 간주하여 미방출 (tbox-v-flow-01 정합)

### 1.3 write_rect 하위 요소 보존 (이전: sz/pos/outMargin만 방출)

원본 실측(tbox-v-flow-01) 자식 순서대로 전면 보강:

`offset → orgSz → curSz → flip → rotationInfo → renderingInfo → lineShape →
fillBrush → shadow → drawText → hc:pt0~pt3 → sz → pos → outMargin → shapeComment`

| 요소 | 역매핑 근거 (파서 함수) | 비고 |
|------|------------------------|------|
| renderingInfo | `parse_rendering_info` | raw_rendering 디코드(cnt+trans+sca/rot 행렬), 실패 시 identity 3행렬 fallback. 비정수 값 f32 Display(예: "1.579917") |
| lineShape | `parse_line_shape_attr` | style/endCap/headfill/tailfill/headSz/tailSz 비트 역산. NONE(0x40)은 파서 자체 비가역(endCap이 비트 6-9 덮어씀) → SOLID 복원 |
| fillBrush | `parse_shape_fill_brush` | None 미방출 / Solid→winBrush / Gradient→gradation / Image→imgBrush. winBrush alpha는 파서가 0~1 분수 경로만 가져 분수 방출 |
| shadow | `parse_shape_shadow_attr` | 전 필드 0이면 미방출. alpha는 정수 방출(파서에 >1.0 정수 경로 존재) |
| hc:pt0~3 | `RectangleShape.x_coords/y_coords` | 꼭짓점 4점 |
| shapeComment | `common.description` | 빈 문자열이면 미방출 |

- rect 루트 속성도 전면 보강: `numberingType`(신규 모델 필드)/`groupLevel`/`instid`/`ratio` 등
- `write_pos`의 `flowWithText`/`allowOverlap` 하드코딩("1"/"0")을 **IR 값으로 교체**
  — tbox-v-flow-01 원본이 `flowWithText="0" allowOverlap="1"`로 기존 하드코딩과 반대였음

### 1.4 모델/파서 보강

- `src/model/shape.rs`: `ObjectNumberingType` enum(None/Picture/Table/Equation) +
  `CommonObjAttr.numbering_type` + `TextBox.vertical_all` 추가
- `src/parser/hwpx/section.rs`: `numberingType`/`instid`/`ratio` 적재(parse_object_element_attrs),
  `textDirection` VERTICALALL 구분 적재(parse_draw_text)
- 기존 struct 리터럴 2곳(document_core object_ops/common_obj_attr_writer 테스트)에 신규 필드 초기화 추가

### 1.5 셀·글상자 colPr 인라인 방출 (2단계 승인 사항 ①, 본문 경로 불변)

- `SerializeContext.sub_list_depth: u32` 신설 — `table.rs write_sub_list`와
  `shape.rs write_draw_text`에서 ±1
- `section.rs render_runs`: `sub_list_depth > 0`일 때만 `Control::ColumnDef`를 슬롯에 포함,
  `render_control_slot`에 depth 가드 arm 추가 → `render_col_pr_ctrl()` 방출
  - `<hp:ctrl><hp:colPr id="" type= layout= colCount= sameSz= sameGap=/></hp:ctrl>`
    (exam-kor-3p 실측 형식), separator_type≠0이면 `<hp:colLine type width="N mm" color>` 자식
- **본문 경로(depth 0)는 불변** — colPr는 종전대로 섹션 템플릿에서만 처리 (mod.rs:349 테스트 보호)

### 1.6 테스트 7건 추가

`shape.rs` 5건:

| 테스트 | 검증 |
|--------|------|
| `task1379_drawtext_paragraph_emits_picture_control` | 글상자 내 hp:pic 방출 |
| `task1379_drawtext_vertical_direction_preserved` | VERTICALALL/VERTICAL 구분 + vertAlign CENTER |
| `task1379_rect_emits_pts_and_element_order` | 실측 자식 순서 단조증가 + shapeComment 보존 |
| `task1379_rect_line_fill_shadow_attrs` | lineShape 전속성/winBrush/shadow 역매핑 문자열 |
| `task1379_tbox_v_flow_01_roundtrip_preserves_textbox` | 대표 샘플 재파싱 후 vertical_all/문단수/텍스트/numbering_type/pt/pos + scaMatrix "1.579917" f32 정밀도 보존 |

`table.rs` 2건:

| 테스트 | 검증 |
|--------|------|
| `task1379_cell_column_def_emits_col_pr` | 셀 내 colPr 방출 (실측 형식 전문 일치) |
| `task1379_cell_column_def_col_line_emitted_when_separator` | separator 있을 때 hp:colLine 방출 |

### 1.7 해소 측정 (baseline_check 동등 임시 테스트 — 측정 후 삭제)

**XFAIL_1378_RECURSIVE 13건 + XFAIL_1379_CONTROLS 16건 = 29건 중 해소 25건 / 잔존 4건**

| 결과 | 샘플 | 비고 |
|------|------|------|
| PASS 25건 | 2단계 해소 14건 + **3단계 신규 해소 11건**: 2024-2Q, 2025-2Q, hwpx-h-02, exam-kor-3p, exam-kor-4p, k-water-rfp (colPr·글상자), 2025-1Q, footnote-tbox-01, hwpx-h-03, hy-001, hy-002 (글상자) | 4단계 일괄 승격 대상 |
| FAIL | 143E433F503322BD33 | 파서 autoNum 폭 비일관 — **#1382** (예정대로 잔존) |
| FAIL | exam_kor(31), exam_social-p1(27), issue_1133(17) | borderFillIDRef 미등록 — **#1384** (승인대로 분리, 예정대로 잔존) |

계획서의 3단계 해소 목표(글상자 5건 + colPr 5건 + k-water-rfp) **전건 해소 확인**.
잔존 4건은 전부 본 타스크 범위 밖 — xfail 승격 + 사유 갱신(#1382/#1384 귀속)은
구현계획서대로 4단계 일괄 수행 (현재 `xfail_*_still_fail` 가드가 해소분에서 red인
상태는 계획상 허용, 완료 조건은 `cargo test --lib` 그린).

## 2. 검증

- `cargo test --lib` **1704 passed** (hwpx serializer 146 포함, 0 failed)
- `cargo fmt --check`(수정 파일 9건) / `cargo clippy --lib --tests` 경고 0 통과

## 3. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/serializer/hwpx/shape.rs` | write_rect 전면 보강 + write_draw_text 공유 경로 전환 + 역매핑 헬퍼 + 테스트 5건 |
| `src/serializer/hwpx/section.rs` | render_text_runs 제거 + colPr depth 가드 방출(render_col_pr_ctrl) |
| `src/serializer/hwpx/table.rs` | write_sub_list depth ±1 + colPr 테스트 2건 |
| `src/serializer/hwpx/context.rs` | `sub_list_depth` 필드 |
| `src/model/shape.rs` | ObjectNumberingType + numbering_type + vertical_all |
| `src/parser/hwpx/section.rs` | numberingType/instid/ratio + VERTICALALL 적재 |
| `src/document_core/commands/object_ops.rs` | TextBox 리터럴 vertical_all 초기화 |
| `src/document_core/converters/common_obj_attr_writer.rs` | 테스트 리터럴 numbering_type 초기화 |

## 4. 승인 요청 사항

1. 3단계 완료 승인
2. 4단계 착수 — xfail 일괄 승격(25건) + #1382/#1384 귀속 사유 갱신 + `--batch` 전수 +
   SVG 비교 + 한컴 시각 판정 요청(tbox-v-flow-01, ta-pic-001-r) + 매뉴얼 갱신 +
   최종 보고서 + CI급 검증
