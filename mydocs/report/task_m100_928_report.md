# Task #928 최종 보고서

## 1. 이슈

**Issue #928**: HWP3 표 셀 내 inline 사각형 shape 주변 텍스트 중복 — `exam_kor.hwp` 5p '확산 모델' 다이어그램

**현상** (rhwp vs 한컴오피스):

| 환경 | 다이어그램 행 출력 |
|------|------------------|
| 한컴오피스 (정답지) | `(가) ⇨ [A 단계] ⇨ (나)` (3 요소) |
| rhwp 회귀 (초기) | `(가) ⇨ (가) ⇨ A 단계 ⇨ (나) ⇨ (나)` (5 요소, 텍스트 중복) |

이슈 본문상 회귀는 텍스트 중복 1건이었으나, Stage 4 시각 검증에서 잔존 회귀 2건 추가 발견:
- 사각형 내부 폰트 축소 (15.33 → 6.88px)
- ㉢ 그림 중복 emit (IR 3개 → SVG 4 image)

총 회귀 3건을 본 task 안에서 모두 해결.

## 2. 회귀 3건의 ROOT CAUSE

### 회귀 1: 텍스트 중복 emit

**위치**: `src/renderer/layout/table_layout.rs::layout_table_cells` Shape 분기 (1812~) 와 트레일링 텍스트 블록 (2158~)

Shape 분기에 `will_render_inline` 가드 누락. `layout_composed_paragraph` 의 `run_tacs` split 이 paragraph 텍스트를 inline 발행 한 뒤, 본 Shape 분기가 text_before 와 trailing text 를 다른 baseline 에 재발행 → Δy=1.39px 평행이동된 중복.

Picture 분기 (1693~) 는 같은 가드가 있어 회귀 없음. Shape 만 누락.

### 회귀 2: 사각형 내부 폰트 축소

**위치**: `src/renderer/layout/shape_layout.rs::layout_textbox_content` Task #874 #3 ratio 축소 로직 (1206-1228)

사각형 IR `orig=(2925, 975)` HU → `cur=(6518, 1983)` HU (2.2배 확대된 채 인코딩). Task #874 의 `max_ratio > 1.5` 조건으로 인라인 사각형 폰트도 `1/2.228 = 0.449` 비율 축소. 본래 의도는 `shortcut.hwp` 마스터 페이지 글상자 (254pt 강제 fit) 케이스로, 인라인 도형은 범위 밖.

### 회귀 3: ㉢ 그림 중복 emit

**위치**: `src/renderer/layout/table_layout.rs` Picture/Shape 가드 boundary 조건

기존 가드 `abs_pos < line.char_start + line_chars` 가 paragraph 마지막 위치 (`abs_pos == line_chars`) 에 있는 inline 컨트롤을 통과시키지 못함. `paragraph_layout.rs` 의 `run_tacs` filter 는 `is_last_run && *pos == run_char_end` 도 포함하여 inline emit → 가드 미통과로 table_layout 의 manual emit 도 발생 → 중복.

exam_kor p[2] (㉠㉡㉢) 의 ctrl[2] (㉢): abs_pos=14, line_chars=14 → 가드 미통과.

## 3. Fix 구현

### Fix 1: Shape 분기 `will_render_inline` 가드 (회귀 1)

`table_layout.rs:1812~` Shape 분기에 Picture 패턴 가드 추가. 가드 통과 시 text_before 발행 및 prev_tac_text_pos 갱신 스킵, 도형 자체는 `get_inline_shape_position` 좌표로 `layout_cell_shape` 호출.

### Fix 2: 인라인 도형 폰트 축소 비활성 (회귀 2)

`layout_textbox_content` 시그니처에 `parent_treat_as_char: bool` 인자 추가. 4 호출자 (Rectangle/Ellipse/Polyline/Curve) 모두 `shape.common().treat_as_char` 전달. ratio 축소 조건에 `&& !parent_treat_as_char` 가드.

shortcut.hwp 마스터 페이지 글상자 (tac=false) 케이스는 기존 동작 유지.

### Fix 3: 가드 시그널 정합 (회귀 3)

Picture/Shape 가드를 `tac_controls + line_chars` 기반 → `tree.get_inline_shape_position(...).is_some()` 기반으로 변경.

`paragraph_layout` 은 inline emit 시 항상 `set_inline_shape_position` 호출 (`paragraph_layout.rs:2019, 2412` 등). 가드를 등록 여부로 판정하면 boundary 조건 무관하게 정확. Equation/Table 분기 (`table_layout.rs:1986, 2079`) 가 이미 같은 패턴 사용 — 일관성 확보.

시도 1 (`<=` 경계 포함) 은 `hwp-img-001.hwp` 4 그림 → 2 그림 회귀 발생 (paragraph_layout 이 cell_ctx 분기로 emit 안 한 케이스도 가드 true 로 만들어 manual emit 누락). `inline_shape_position` 기반으로 변경하여 해소.

## 4. 검증 결과

### 시각 정합 (한컴 2022 PDF `pdf/exam_kor-2022.pdf` 5쪽)

- ✅ 다이어그램 `(가) ⇨ [A 단계] ⇨ (나)` 3 요소 단일 baseline
- ✅ 사각형 outline x=292.88 y=408.70 w=86.92 h=26.45 (두 ⇨ 사이 정확 위치)
- ✅ "A 단계" 내부 텍스트 font 15.33px (정상)
- ✅ ㉠㉡㉢ 그림 3개 정확 배치 (181, 307, 432)
- ✅ 작업지시자 시각 검증 완료 ("이상없음")

### 자동 회귀

| 테스트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ 1275 unit passed, 0 failed |
| `cargo test --test svg_snapshot` | ✅ 8/8 통과 (KTX/aift/복학원서/exam_kor 6p 등) |
| `test_task76_img_001_four_pictures` | ✅ hwp-img-001.hwp 4 그림 정확 (회귀 차단) |

## 5. 변경 파일 통계

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/table_layout.rs` | Shape 분기 가드 추가, Picture/Shape 가드 시그널 정합 |
| `src/renderer/layout/shape_layout.rs` | `layout_textbox_content` 시그니처 + ratio 가드 |

CLAUDE.md 의 "HWP3 전용 분기는 src/parser/hwp3/" 규칙은 본 task 에 미해당 — 회귀 3건 모두 공통 렌더러의 가드 누락/boundary 조건/범위 과대 문제.

## 6. 컨트롤 분기 가드 패턴 정리 (회귀 차단 인프라)

`table_layout.rs::layout_table_cells` 의 컨트롤 분기 가드 패턴 일관성 확보:

| 분기 | 가드 시그널 | 패턴 출처 |
|------|------------|----------|
| Picture | `tree.get_inline_shape_position` | 본 task (이전: tac_controls 기반) |
| Shape | `tree.get_inline_shape_position` | 본 task (이전: 가드 없음) |
| Equation | `tree.get_inline_shape_position` | Task #287 #301 (라인 1986) |
| Table | `tree.get_inline_shape_position` | (라인 2079) |

→ 모든 분기가 `paragraph_layout` 의 inline emit 실제 동작 (set_inline_shape_position) 과 정합.

## 7. 잔존 가능 회귀 (본 task 비범위)

- **다중 inline shape 혼재 케이스**: Picture/Shape 가드가 일부 true / 일부 false 일 때 트레일링 텍스트 블록 (`table_layout.rs:2158-2231`) 의 부분 진입 가능성. 본 task 의 single-shape 케이스에서는 prev_tac_text_pos 가드 (가드 안에 둠) 로 차단되나, 혼재 케이스는 가설적. 관측 사례 없음.
- **HWP5/HWPX 동일 패턴**: 본문은 HWP3 샘플에서만 검증. HWP5/HWPX 동일 케이스 (셀 내 inline 사각형 + tac+TopAndBottom) 가 존재할 가능성 있으나 미관측.

발견 시 별도 이슈로 분리.

## 8. Stage 진행 요약

| Stage | 결과 | 핵심 |
|-------|------|------|
| 1 | ✅ | 정밀 재현 + 텍스트 중복 ROOT CAUSE (y=421.73 / y=423.12 두 baseline) |
| 2 | ✅ | 코드 trace — Shape 분기 가드 누락 확정 |
| 3 | ✅ | Fix 1 구현 (Shape 분기 가드) |
| 4 | ⚠️ | 시각 검증 — 텍스트 중복 해소 ✅, 잔존 회귀 2건 발견 |
| 5 | ✅ | Fix 2 + Fix 3 구현 (사각형 폰트 + ㉢ 그림 중복) |
| 6 | ✅ | 최종 보고 + 사용자 시각 검증 완료 |

## 9. Commit 이력

| Commit | 단계 |
|--------|------|
| 8c2899e3 | Stage 1: 정밀 재현 + ROOT CAUSE 측정 |
| (Stage 2 미커밋) | (Stage 1 직후 분석) |
| 738d63fd | Stage 3: Shape 분기 will_render_inline 가드 (Fix 1) |
| (Stage 4 보고 + impl_v2 + Stage 5 fix 통합 커밋) | Stage 5: 사각형 폰트 + ㉢ 그림 중복 fix (Fix 2 + Fix 3) |
| (본 보고서 + 오늘할일) | Stage 6: 최종 정리 |
