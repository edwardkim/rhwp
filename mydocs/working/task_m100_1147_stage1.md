# Task #1147 Stage 1 완료 보고서 — HWPX TopAndBottom 표 host_spacing 보정

수행계획서: [task_m100_1147.md](../plans/task_m100_1147.md) · 구현계획서: [task_m100_1147_impl.md](../plans/task_m100_1147_impl.md)

## 1. 변경 내용

### src/renderer/typeset.rs

**format_table() 의 host_spacing 산식** (`2627-2671` 부근):

1. **트리거 조건 추가** (HWPX 원본 한정):
   ```
   is_topbottom_empty_anchor_hwpx
     = is_hwpx_source && !is_tac
     && matches!(text_wrap, TextWrap::TopAndBottom)
     && para.text.is_empty()
   ```

2. **host_line_spacing 억제**: HWPX 빈 앵커 트리거 충족 시 0.0 (기존 `last seg.line_spacing` 가산 분기는 그대로 유지, 다른 케이스는 무변경)

3. **spacing_before 억제**: 동일 트리거 충족 + `!is_column_top` 시 `before = outer_top` (sb 제외)

**TypesetState 신규 필드 `is_hwpx_source: bool`** (`140-142`):
- `TypesetState::new()` 기본값 false (`317`)
- `typeset_section_with_variant()` 신규 인자로 받아 state 에 주입 (`576, 612`)
- `typeset_section()` (shortcut) 는 false 전달 (`555`)
- `format_table()` 호출처 (`2911`) 에서 `st.is_hwpx_source` 전달

### src/document_core/queries/rendering.rs

- `typeset_section_with_variant()` 호출에 `matches!(self.source_format, FileFormat::Hwpx)` 전달 (`1983`)

## 2. 검증 결과

### 본 페이지 (HWPX, page_num=5)

| 항목 | 변경 전 | 변경 후 |
|------|---------|---------|
| `items` | 7 | **8** (대상 한 줄 문단 포함) |
| `used (px)` | 931.2 | 931.5 |
| `hwp_used (px)` | — (비교 미발생) | 943.9 |
| `diff (px)` | — | **-12.4** (안전 영역) |
| pi=N (한 줄 문단) 배치 | 다음 페이지 | **본 페이지 하단** ✓ |

`TABLE_DRIFT` pi=126: `host_sp=24.7` → `host_sp=0.0` (sb + host_line_spacing 모두 0)

### 회귀 — hwpspec.hwp (HWP5)

- `task1086_hwpspec_page_count_matches_hancom_office`: **PASS** (178 페이지 유지)
- HWP5 는 `is_hwpx_source=false` 라 트리거 미발동 → 기존 동작 보존

### 전체 cargo test (release)

- 1411 lib + 모든 integration 통과 (FAILED 0)

### HWPX 회귀 영향 (aift.hwpx, 비공개 샘플 외 다른 HWPX)

- `aift.hwpx`: 드리프트 분포 거의 무변화 (mean -3.18 → -3.46, min/max 동일)
- 본 fix 는 빈 앵커 + TopAndBottom 패턴 한정이라 광범위 영향 없음

## 3. Stage 2 처리 결정

구현계획서상 Stage 2 (TAC 표 `fmt.height_for_fit` 보정) 는 **Stage 1 만으로 본 페이지 overflow 해소되지 않는 경우** 진행 조건이었음. Stage 1 만으로 본 페이지 해소 + 회귀 없음 확인 → Stage 2 본 타스크에서 미진행.

별도 latent 이슈 (TAC 1x1 표 단독 페이지 +42.9px 드리프트) 는 후속 이슈로 분리 검토.

## 4. Stage 3 / 4 진행

- Stage 3 (Golden SVG 회귀 분석): cargo test 통과로 핵심 회귀 없음 확인. 추가 SVG 시각 비교는 Stage 4 와 통합.
- Stage 4: 본 페이지 SVG 재출력 + 시각 정합 + 정리 + 최종 보고서.
