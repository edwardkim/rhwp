# 최종 결과보고 — #1842 CellBreak 초대형 표 부재-LINE_SEG 셀 라인높이 팽창

브랜치 `fix/1842-cellbreak-synthetic-em` (base: `fix/2063` — issue2063 픽스처 공유). #2063에서 인계된 과분할(+51).

## 1. 근본 원인 (PDF 그라운드트루스 + 계측 확정)

`21914299 화성시 [별표2]`(5,277행×10열 CellBreak) 데이터 셀은 **저장 LINE_SEG 부재**(`line_segs.is_empty()`). composer(composer.rs:527)가 placeholder `line_height=400` 로 합성 → `corrected_line_height(5.33, 폰트 13.33, 160%)` 가 `raw_lh<max_fs` 라 **`max_fs×1.6=21.33px`** 반환 → 행 **25px**.

한글 2022는 동일 셀을 **폰트 em(13.33)+pad = 17px** 로 렌더(line-spacing 160%는 줄 사이 간격이지 단행 박스 높이가 아님).

**측정 대조** (한글 PDF 162p vs rhwp 213p, 동일 page 80, A4 가로):
| | 행높이 | 쪽당 행수 |
|---|---|---|
| 한글 | median **17.07px** | 41 |
| rhwp(수정 전) | median **25.07px** | 29 |

## 2. 수정

부재-LINE_SEG 셀 라인높이를 **폰트 em(max_fs)** 으로 (hwp3 synthetic 선례 `corrected_line_height_for_variant_synthetic` 확장). **CellBreak 표 한정** — RowBreak 규제영향분석서(76076 등)는 현행 ×1.6 이 공식 PDF 쪽수와 정합(#1891)이라 제외.

- 적용: measure(height_measurer 3사이트) + cut(table_layout cell_units) — `matches!(table.page_break, CellBreak)` 게이트.
- 판별자: `line_segs.is_empty() && !text.is_empty() && CellBreak`.
- `src/renderer/height_measurer.rs`, `src/renderer/layout/table_layout.rs`.

## 3. 검증

| 항목 | 결과 |
|---|---|
| 21914299 (CellBreak) 페이지 | **213 → 159** (한글 162, −3) |
| 렌더 행높이 (PDF page 80) | **17.07px, 41행/쪽** (한글 정합) |
| 76076 (RowBreak) #1891 | **82 불변** (공식 PDF, 회귀 0) |
| 전체 테스트 | **2922 passed / 0 failed** |
| 신규 회귀 테스트 | `issue_1842_cellbreak_synthetic_lineheight_em_not_inflated` |
| clippy | clean |

## 4. 잔여

−3쪽(159 vs 162)은 빈 셀의 저장 cell.height(20px) floor(한글 content 17px) 2차 잔차 — 별건. 본 수정은 지배 기전(부재-LINE_SEG ×1.6 팽창) 해소.

## 5. 판별자 정제 이력 (#1926 방법론)

1차 일반화(모든 synthetic → em)는 **76076(RowBreak) 82→80 회귀**(#1891 반례)로 검출 → **CellBreak 한정**으로 정제. 반례가 판별 기준을 좁힌 사례.
