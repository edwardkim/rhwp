# Task #671 Stage 3 단계별 보고서 — 광범위 회귀 sweep + 최종 검증

## 1. 광범위 페이지네이션 회귀 sweep

`samples/` 폴더 전체 187 fixture 페이지 수 BEFORE/AFTER 비교:

| 영역 | 결과 |
|------|------|
| BEFORE (devel) | 187 fixtures / **2013 pages** |
| AFTER (Task #671) | 187 fixtures / **2013 pages** |
| **차이** | **0** ✅ |

**회귀 위험 영역 완전 좁힘** — 본 task 정정이 다른 fixture 의 페이지네이션에 영향 0.

## 2. 핵심 fixture 페이지 수 (개별 검증)

| 파일 | BEFORE | AFTER | 차이 |
|------|--------|-------|------|
| aift.hwp | 77 | 77 | 0 ✅ |
| exam_kor.hwp | 20 | 20 | 0 ✅ |
| exam_math.hwp | 20 | 20 | 0 ✅ |
| exam_science.hwp | 4 | 4 | 0 ✅ |
| synam-001.hwp | 35 | 35 | 0 ✅ |
| footnote-01.hwp | 6 | 6 | 0 ✅ |
| hwp3-sample.hwp | 16 | 16 | 0 ✅ |
| hwp3-sample4.hwp | 36 | 36 | 0 ✅ |
| hwp3-sample5.hwp | 64 | 64 | 0 ✅ |
| **계획서.hwp** (권위) | **1** | **1** | **0** ✅ |

## 3. 결정적 검증 (최종)

| 검증 영역 | 결과 |
|----------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ **6/6** |
| `cargo test --release --test issue_546` | ✅ **1/1** |
| `cargo test --release --test issue_554` | ✅ **12/12** |
| `cargo clippy --release` | ✅ 0 warnings |

## 4. 시각 판정 (작업지시자 게이트웨이)

### 4.1 본 task #671 본질 영역 — 줄겹침 해소 ✅

`samples/계획서.hwp` 1페이지 표 (PNG 시각 확인):

| 영역 | BEFORE (정정 전) | AFTER (정정 후) |
|------|-----------------|-----------------|
| 셀 [13] r=3,c=1 "탈레스 HSM 관리 시스템 및 REST API" | 1줄 압축 (글자 겹침) | **2줄 정상 분리** ✅ |
| 셀 [21] r=5,c=1 "탈레스 HSM 을 관리하기위한 CCC..." | 1줄 압축 (글자 겹침) | **3줄 분리** ✅ (단 마지막 줄 부분 클립 — Issue #672) |

본 task 의 본질 영역 (셀 paragraph line_segs 부재 → 줄겹침) **정정 완료** ✅.

### 4.2 잔존 결함 — 별도 Issue #672

본 task 정정 후 노출된 잔존 클립 영역:

- 셀 [21] 3번째 줄 마지막 부분 클립 (~0.88px)
- 셀 [52] 3번째 paragraph 클립

**본질 진단**: `height_measurer.rs:822-830` TAC 표 (treat_as_char=true) 비례 축소 메커니즘 — 측정 row_heights 합 > common.height 시 모든 row_heights 비례 축소 → 셀 콘텐츠 클립.

**별도 Issue 등록**: [#672](https://github.com/edwardkim/rhwp/issues/672) — "TAC 표 비례 축소 시 셀 콘텐츠 클립 — common.height vs measured row_heights 불일치"

`feedback_hancom_compat_specific_over_general` + 회귀 위험 좁힘 영역 정합 — 본 task #671 본질 영역 (line_segs 부재) 과 다른 본질 (TAC 표 비례 축소) 분리 처리.

## 5. 코드 변경 사항 정리

### 5.1 신규 함수 (`src/renderer/composer.rs`)

- `recompose_for_cell_width(composed, para, cell_inner_width_px, styles)` — 진입점
- `split_composed_line_by_width(line, max_width_px, styles)` — 헬퍼

### 5.2 호출 위치 (6곳)

| 파일:줄 | 역할 |
|---------|------|
| `composer.rs` | 신규 함수 추가 |
| `table_layout.rs:1226-1234` | 셀 layout 렌더링 경로 |
| `table_layout.rs:614/678` (caller) + `:700` (callee 시그니처) | resolve_row_heights 측정 fallback |
| `table_partial.rs:94, 358` | 분할 표 측정 + layout |
| `height_measurer.rs:527, 712` | MeasuredTable 핵심 측정 |

### 5.3 회귀 위험 영역 좁힘 (3중 가드)

`recompose_for_cell_width` 내부:

1. `para.line_segs.is_empty()` — 한컴 인코딩 부재만
2. `composed.lines.len() == 1` — fallback 단일 ComposedLine 만
3. 측정 폭 > `cell_inner_width_px` — 너비 안에 들어가면 분할 불필요

3 중 가드 미충족 시 `composed` 무변경 → **회귀 0 보장**.

## 6. 작업지시자 시각 판정 결과

본 task 의 본질 영역 (Issue #671 제목: "표 셀 내부 paragraph 줄바꿈 시 다중 LINE_SEG 줄 겹침") 통과 ✅:

- 셀 [13] / [21] 줄겹침 해소
- 다른 셀 영역 회귀 0
- 광범위 sweep 페이지 수 차이 0

잔존 영역 (Issue #672 영역) 은 본 task 본질 영역 외.

## 7. Stage 3 완료 — 최종 보고서 작성 영역 진입

본 단계별 보고서 + 결정적 검증 + 광범위 sweep 회귀 0 + 시각 판정 통과 + 잔존 결함 별도 Issue 분리 → **최종 결과 보고서** (`mydocs/report/task_m100_671_report.md`) 작성 진입.
