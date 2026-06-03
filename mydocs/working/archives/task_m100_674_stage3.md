# Task #674 Stage 3 단계별 보고서 — 광범위 회귀 sweep + 최종 검증

## 1. 광범위 페이지네이션 회귀 sweep

`samples/` 폴더 전체 187 fixture 페이지 수 BEFORE/AFTER 비교:

| 영역 | 결과 |
|------|------|
| BEFORE (task672) | 187 fixtures / **2013 pages** |
| AFTER (task674) | 187 fixtures / **2013 pages** |
| **차이** | **0** ✅ |

**회귀 위험 영역 완전 좁힘** — 본 task 정정 (calc_para_lines_height 의 corrected_line_height 적용) 이 다른 fixture 페이지네이션에 영향 0.

## 2. 결정적 검증

| 검증 영역 | 결과 |
|----------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ **6/6** |
| `cargo test --release --test issue_546` | ✅ **1/1** |
| `cargo test --release --test issue_554` | ✅ **12/12** |
| `cargo clippy --release` | ✅ 0 warnings |

## 3. 시각 판정 게이트웨이 통과

`samples/계획서.hwp` 1페이지 표 (PNG 시각 확인, rsvg-convert -z 4):

| 셀 | BEFORE (task672) | AFTER (task674) |
|----|-----------------|-----------------|
| [13] r=3,c=1 "탈레스 HSM 관리 시스템 및 REST API" | 2줄 정상 | 2줄 정상 (회귀 0) ✅ |
| [21] r=5,c=1 "목적" | 2줄 (마지막 줄 클립) | **3줄 모두 표시** ✅ |
| [52] r=13,c=3 "특허 취득" | 2 paragraph | **3 paragraph 모두 표시** ✅ |
| 다른 셀 | — | 회귀 0 ✅ |

**Task #671/#672/#674 시리즈 정정 완료** — `samples/계획서.hwp` 1페이지 표 시각 결함 완전 해소.

## 4. Task #671 ~ #674 시리즈 본질 영역 정합

| Task | 본질 영역 | 정정 위치 |
|------|----------|-----------|
| #671 | 셀 paragraph line_segs 부재 → compose_lines 단일 ComposedLine 압축 | composer.rs (recompose_for_cell_width) + 6개 호출 위치 |
| #672 | TAC 표 비례 축소 (작은 차이도 발동) | height_measurer.rs:822 임계값 가드 |
| #674 | calc_para_lines_height corrected_line_height 누락 | table_layout.rs:746 시그니처 + 보정 |

## 5. Stage 3 완료 — 최종 보고서 작성 영역 진입

본 단계별 보고서 + 결정적 검증 + 광범위 sweep 회귀 0 + 시각 판정 ★ 통과 → **최종 결과 보고서** (`mydocs/report/task_m100_674_report.md`) 작성 진입.
