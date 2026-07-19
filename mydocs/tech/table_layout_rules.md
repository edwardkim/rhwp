---
kind: canonical
status: active
canonical: mydocs/tech/table_layout_rules.md
last_verified: 2026-07-16
---

# 문단 내 표의 본문배치 및 페이지 조판 규칙

## 개요

HWP에서 표(Table)는 문단의 컨트롤로 존재하며, 두 가지 속성 조합으로 조판 방식이 결정됩니다:
- **본문배치(text_wrap)**: TopAndBottom, Square, InFrontOfText, BehindText
- **글자처럼 취급(treat_as_char)**: true/false

## 조합별 조판 규칙

### 1. TopAndBottom (자리차지) + treat_as_char=false

표가 본문 흐름에서 독립적으로 위치하며, 본문 텍스트를 위/아래로 밀어냅니다.

| 속성 | 설명 |
|------|------|
| 위치 결정 | `vert_rel_to`(Paper/Page/Para) + `vert_align`(Top/Center/Bottom) + `vertical_offset` |
| 본문 영향 | `shape_reserved`로 앵커 문단의 y_offset을 표 하단 아래로 밀어냄 |
| 높이 차지 | 본문 흐름에서 **표 높이만큼 공간 차지** (shape_reserved) |
| outer_margin | 표 바깥 여백(margin.bottom 등)이 shape_reserved에 추가 |

#### 특수 케이스

**vert=Paper + 표가 body_area 위에 위치 (renders_above_body)**:
- 표가 body-clip 밖(paper_images)에 렌더링됨
- `layout_table` 반환값으로 y_offset **미갱신** (shape_reserved가 이미 처리)
- 표 아래 간격(spacing_after, line_spacing) **미추가**
- 예: exam_kor 1페이지 상단 "2025학년도 대학수학능력시험 문제지" 표

**vert=Page/Paper + valign=Bottom/Center (페이지 하단 고정)**:
- 본문 흐름과 무관 — `process_controls`에서 `PageItem::Table`로 배치 후 continue
- `paginate_table_control` 미호출 → 높이/행 분할/단 변경 없음
- 예: exam_social 문단 1.61의 "확인 사항" 표

### 2. TopAndBottom (자리차지) + treat_as_char=true

인라인 블록 표. 문단의 텍스트 흐름 안에서 배치됩니다.

| 속성 | 설명 |
|------|------|
| 위치 결정 | 문단의 텍스트 흐름에 따라 순차 배치 |
| 본문 영향 | LINE_SEG의 line_height에 표 높이가 포함됨 |
| 높이 차지 | `paginate_table_control`에서 `table_total_height`로 계산 |
| pagination | `st.current_height += table_total_height` |

#### 특수 케이스

**TAC 블록 표 (단 너비를 차지하는 인라인 표)**:
- `is_tac_table_inline()`이 false인 경우
- `paginate_table_control` → `place_table_fits`에서 pre-text, 표, post-text 순서로 배치
- 표 호스트 문단의 공백만인 PartialParagraph: 높이 추가 없이 건너뜀
  (Table PageItem에서 이미 표 높이 반영)

**TAC 인라인 표 (텍스트와 수평 배치)**:
- `is_tac_table_inline()`이 true인 경우
- `process_controls`에서 건너뜀 (LINE_SEG가 이미 높이 포함)
- 렌더링: `layout_composed_paragraph`에서 tac_controls 기반 수평 배치

**LINE_SEG lh 이중 계산 (표 앞 텍스트가 있는 경우)**:
- 첫 SEG의 th << lh이고 마지막 SEG의 lh와 동일한 패턴
- `measure_paragraph`에서 vpos 기반 높이로 보정
- 예: exam_social 10번 문항 표

### 3. Square (어울림) + treat_as_char=false

표 옆에 텍스트가 흐릅니다.

| 속성 | 설명 |
|------|------|
| 위치 결정 | `vert_rel_to` + `horz_rel_to` 기반 절대 위치 |
| 본문 영향 | 표 옆 영역에 텍스트 배치 (LINE_SEG cs/sw로 영역 정의) |
| 높이 차지 | 표 높이만큼 (어울림 영역 내) |
| 렌더링 | `layout_wrap_around_paras`에서 처리 |

### 4. InFrontOfText (글 앞으로) + treat_as_char=false

표가 본문 위에 떠있습니다. 본문 흐름에 영향 없음.

| 속성 | 설명 |
|------|------|
| 위치 결정 | `vert_rel_to` + `horz_rel_to` 기반 절대 위치 |
| 본문 영향 | **없음** — 높이 미차지 |
| pagination | `process_controls`에서 `PageItem::Shape`로 배치, continue |
| 셀 내 높이 | `measure_non_inline_controls_height`에서 **제외** |

### 5. BehindText (글 뒤로) + treat_as_char=false

표가 본문 뒤에 깔립니다. 본문 흐름에 영향 없음.

| 속성 | 설명 |
|------|------|
| 위치 결정 | `vert_rel_to` + `horz_rel_to` 기반 절대 위치 |
| 본문 영향 | **없음** — 높이 미차지 |
| pagination | `process_controls`에서 `PageItem::Shape`로 배치, continue |

## 좌표계

| 기준 | 설명 | 사용처 |
|------|------|--------|
| Paper | 용지 좌상단 (0,0) | `vert_rel_to=Paper` 표, 바탕쪽 |
| Page | body_area 좌상단 | `vert_rel_to=Page` 표, `horz_rel_to=Page` |
| Column | 현재 단 영역 | `horz_rel_to=Column` |
| Para | 앵커 문단 위치 | `vert_rel_to=Para` |

### 좌표계 주의사항

- `compute_table_x_position`에서 `HorzRelTo::Paper`일 때 용지 너비가 필요하지만,
  `col_area`(단 영역)만 전달됨 → `col_area.x * 2 + col_area.width`로 추정
- 2단 레이아웃에서 col_area.width가 단 너비이므로, 표 너비 기반 추정 로직 추가
- **백로그 B-001**: paper_area/body_area를 명시적 전달하여 추정 제거

## shape_reserved 메커니즘

TopAndBottom 개체가 본문 시작 위치를 밀어내는 메커니즘.

1. `calculate_shape_reserved_heights`: 각 단의 PageItem에서 TopAndBottom 개체 검색
2. `calc_shape_bottom_y`: 개체의 하단 y 좌표 계산 (margin.bottom 포함)
3. `calculate_body_wide_shape_reserved`: body_area 전체에 걸치는 개체 → 모든 단에 적용
4. `build_single_column`에서 y_offset을 shape_reserved 하단으로 초기화

### 제한사항

- `common.height`가 실제 렌더링 높이보다 작을 수 있음 (셀 콘텐츠 확장)
- `measure_non_inline_controls_height`에서 InFrontOfText/BehindText 제외 필요
- pagination의 `current_height` 누적이 vpos보다 과대 → **백로그 B-002**

## 관련 코드 파일

| 파일 | 역할 |
|------|------|
| `pagination/engine.rs` | 페이지/단 분할, 표 배치 결정 |
| `layout.rs` | 렌더 트리 생성, shape_reserved, renders_above_body |
| `layout/table_layout.rs` | 표 위치/크기 계산, 셀 레이아웃 |
| `layout/shape_layout.rs` | shape_reserved 계산, 개체 위치 |
| `height_measurer.rs` | 표/문단 높이 사전 측정 |
| `layout/paragraph_layout.rs` | 문단 렌더링, 문단 테두리/배경 |
