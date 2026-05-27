# Task #1147 최종 결과 보고서 — HWPX TopAndBottom 표 host_spacing 보정

- **이슈**: https://github.com/edwardkim/rhwp/issues/1147
- **브랜치**: `local/task1147`
- **수행계획서**: [task_m100_1147.md](../plans/task_m100_1147.md)
- **구현계획서**: [task_m100_1147_impl.md](../plans/task_m100_1147_impl.md)
- **Stage 1 보고서**: [task_m100_1147_stage1.md](../working/task_m100_1147_stage1.md)

## 1. 문제 요약

HWPX 원본 본문 페이지에서 마지막 한 줄 문단이 다음 페이지로 강제 이월되는 현상.

`dump-pages` 진단상 wrap=TopAndBottom 비-TAC 표 (빈 앵커 문단) 가 단독으로 +27.6px 과잉 가산되어 페이지 잔여 공간을 잠식 → 한 줄 문단 overflow.

## 2. 근본 원인

`src/renderer/typeset.rs` 의 `format_table()` 이 HWPX 원본의 빈 앵커 문단에서도 다음 두 가지를 가산:

1. **spacing_before** — HWPX 의 빈 앵커 LINE_SEG vpos 는 직전 문단 종료 vpos 와 동일 (갭 0). PS.spacing_before 를 별도 가산하면 +sb 만큼 어긋남.
2. **host_line_spacing** — 빈 앵커 문단에 본문 줄이 없는데도 last seg.line_spacing 을 표 다음 갭으로 가산하여 +leading 만큼 어긋남.

HWP5/HWP3 는 LINE_SEG 인코딩이 달라 (sb·leading 이 vpos 안에 흡수) 기존 가산이 한컴 오피스 출력과 정합. **HWPX 만의 시멘틱 차이로 발생하는 문제**.

## 3. 변경 내용

### src/renderer/typeset.rs

- `TypesetState.is_hwpx_source: bool` 신설 (기본 false)
- `format_table()` 의 host_spacing 산식에 HWPX 한정 트리거 추가:
  ```
  is_topbottom_empty_anchor_hwpx
    = is_hwpx_source && !is_tac
    && matches!(text_wrap, TextWrap::TopAndBottom)
    && para.text.is_empty()
  ```
- 트리거 충족 시 `host_line_spacing = 0`, `before = outer_top` (sb 제외, `!is_column_top` 가드 유지)
- `typeset_section_with_variant()` 시그니처에 `is_hwpx_source: bool` 추가, `format_table()` 호출 시 전달

### src/document_core/queries/rendering.rs

- `typeset_section_with_variant()` 호출 시 `matches!(self.source_format, FileFormat::Hwpx)` 전달

## 4. 검증 결과

### 본 페이지 (HWPX, page_num=5)

| 항목 | 변경 전 | 변경 후 |
|------|---------|---------|
| `items` | 7 | **8** ✓ |
| `used (px)` | 931.2 | 931.5 |
| `hwp_used (px)` | (비교 미발생) | 943.9 |
| `diff (px)` | — | -12.4 (안전) |
| 한 줄 문단 배치 | 다음 페이지 | **본 페이지 하단 (권위 PDF 정합)** ✓ |

SVG 재출력 시각 확인 — `_008.svg` 가 pi=120…127 포함, `_009.svg` 는 pi=128 (`5. ...`) 으로 시작.

### 회귀 — hwpspec.hwp (HWP5)

- `task1086_hwpspec_page_count_matches_hancom_office`: **PASS** (178 페이지 유지)
- HWP5 는 `is_hwpx_source=false` 라 트리거 미발동

### 전체 cargo test (release)

- lib (1411) + integration 전수 통과, **FAILED 0**

### 다른 HWPX 회귀 영향 (aift.hwpx)

- 드리프트 분포 거의 무변화 (mean -3.18 → -3.46, min/max 동일)
- 빈 앵커 + TopAndBottom 패턴 한정이라 광범위 영향 없음

## 5. 남은 / 후속 이슈

본 타스크에서 다루지 않은 latent 이슈:

- **TAC (treat_as_char) 1x1 표 단독 페이지 +42.9px 드리프트**
  - 구현계획서 Stage 2 후보였으나 Stage 1 만으로 본 페이지 해소되어 미진행
  - `typeset.rs:3201-3206` 의 `fmt.height_for_fit` 가 LINE_SEG.lh 안에 표 본체를 이미 포함하는데도 PS.spacing_before 를 추가 가산
  - 별도 이슈 등록 검토 필요

- **`hwp_used` 비교 음수 드리프트 (-5 ~ -13)** (대부분 페이지)
  - 본문 텍스트 라인 측정 누락 가능성 — 본 타스크 범위 외

## 6. 커밋 / 머지 단계

- 단계 1 커밋: `Task #1147: Stage 1 — HWPX TopAndBottom 표 host_spacing 보정` (`8d3063dc`)
- 최종 보고서 커밋 예정
- `local/task1147` → `local/devel` 머지 → `devel` 머지 (작업지시자 승인 후)
