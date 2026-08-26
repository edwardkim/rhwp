# M01-5 한컴 오라클 편입 · 다중버전 쪽수 불일치

- 클레임: `M01-5`
- 측정: `pypdf_page_count` (픽셀 차이는 `out_of_scope`)
- 파일 425 / stem 395 / 다중버전 stem 9
- 쪽수 불일치 1 · 쪽수 일치 8 · 미완(쪽수 미측정) 0
- 측정 425 · 미측정 0

## 트리 편입

| 트리 | 존재 | 기본 한글 버전 | 파일 | 측정 | 미측정 |
| --- | --- | --- | ---: | ---: | ---: |
| `pdf/` | 예 | 2022 | 406 | 406 | 0 |
| `pdf-2020/` | 예 | 2020 | 1 | 1 | 0 |
| `pdf-large/` | 예 | 없음 | 18 | 18 | 0 |

## pdf-2020/ · pdf-large/ 편입 목록

### `pdf-2020/` (1건)

| 경로 | stem | 한글 버전 | 출처 | 쪽수 | 상태 |
| --- | --- | --- | --- | ---: | --- |
| `pdf-2020/pr-1674-2020.pdf` | `pr-1674` | 2020 | explicit | 35 | measured |

### `pdf-large/` (18건)

| 경로 | stem | 한글 버전 | 출처 | 쪽수 | 상태 |
| --- | --- | --- | --- | ---: | --- |
| `pdf-large/3-09월_교육_통합_2024-구분선아래20-2024.pdf` | `3-09월_교육_통합_2024-구분선아래20` | 2024 | explicit | 23 | measured |
| `pdf-large/3-09월_교육_통합_2024-미주사이20-2024.pdf` | `3-09월_교육_통합_2024-미주사이20` | 2024 | explicit | 24 | measured |
| `pdf-large/hwpx/143E433F503322BD33.pdf` | `143E433F503322BD33` | — | unknown | 1 | measured |
| `pdf-large/hwpx/2026_oss_rst.pdf` | `2026_oss_rst` | — | unknown | 6 | measured |
| `pdf-large/hwpx/[2027] 온새미로 1 본교재.pdf` | `[2027] 온새미로 1 본교재` | — | unknown | 46 | measured |
| `pdf-large/hwpx/el-school-001.pdf` | `el-school-001` | — | unknown | 1 | measured |
| `pdf-large/hwpx/eq-002.pdf` | `eq-002` | — | unknown | 1 | measured |
| `pdf-large/hwpx/footnote-tbox-01.pdf` | `footnote-tbox-01` | — | unknown | 1 | measured |
| `pdf-large/hwpx/hcar-001.pdf` | `hcar-001` | — | unknown | 6 | measured |
| `pdf-large/hwpx/hy-001.pdf` | `hy-001` | — | unknown | 2 | measured |
| `pdf-large/hwpx/hy-002.pdf` | `hy-002` | — | unknown | 2 | measured |
| `pdf-large/hwpx/issue_1133.pdf` | `issue_1133` | — | unknown | 3 | measured |
| `pdf-large/hwpx/k-water-rfp.pdf` | `k-water-rfp` | — | unknown | 27 | measured |
| `pdf-large/hwpx/math-001.pdf` | `math-001` | — | unknown | 1 | measured |
| `pdf-large/hwpx/shape-001.pdf` | `shape-001` | — | unknown | 1 | measured |
| `pdf-large/hwpx/ta-pic-001-r.pdf` | `ta-pic-001-r` | — | unknown | 1 | measured |
| `pdf-large/hwpx/tb-org-02.pdf` | `tb-org-02` | — | unknown | 1 | measured |
| `pdf-large/issue2006/1790387_prep_final_report-2022.pdf` | `1790387_prep_final_report` | 2022 | explicit | 146 | measured |


## 쪽수 불일치 (다중버전)

| stem | 버전별 쪽수(실측) | 최소 | 최대 | 파일 수 |
| --- | --- | ---: | ---: | ---: |
| `2025 행정업무운영 편람(최종)` | 2010=388; 2020=383; 2024=383 | 383 | 388 | 8 |

버전별 파일:

### `2025 행정업무운영 편람(최종)`

- 2010 (쪽 388)
  - `pdf/2025 행정업무운영 편람(최종)-2010-kopub.pdf`
- 2020 (쪽 383,383,383,383,383,383)
  - `pdf/2025 행정업무운영 편람(최종)-2020-kopub.pdf`
  - `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf`
  - `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2020.pdf`
  - `pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2020.pdf`
  - `pdf/stage242-hwp2020-page-count/2025 행정업무운영 편람(최종)-2020.pdf`
  - `pdf/stage242-hwp2020-page-count/hwpx/2025 행정업무운영 편람(최종)-2020.pdf`
- 2024 (쪽 383)
  - `pdf/2025 행정업무운영 편람(최종)-2024.pdf`


## 다중버전 · 쪽수 일치

같은 stem 에 한글 버전이 둘 이상이고, **잰 쪽수는 같다**. 시각(픽셀) 일치로 읽지 말 것.

| stem | 버전 | 쪽수 | 파일 수 |
| --- | --- | ---: | ---: |
| `SO-SUEOP` | 2022,2024 | 46 | 2 |
| `hwp3-sample16-hwp5` | 2020,2022 | 64 | 2 |
| `hwpx_sample2` | 2020,2024 | 29 | 2 |
| `issue1949_giant_cell_nested_tables_perf` | 2020,2024 | 115 | 2 |
| `k-water-rfp` | 2022,2024 | 27 | 3 |
| `none_table_declared_fits` | 2020,2022 | 2 | 2 |
| `pr-1674` | 2020,2024 | 35 | 2 |
| `saved_single_line_spacing_after` | 2020,2022 | 1 | 2 |


## 다중버전 · 쪽수 미완

없음.

## 정직 한계

- 쪽수는 pypdf `len(reader.pages)` 만 사용한다.
- 픽셀/렌더 차이는 측정하지 않았고, 불일치로 세지 않는다.
- LFS 포인터·비PDF·pypdf 실패는 `page_count=null` + 상태 코드다.
- `scripts/visual_sweep.py` 는 이 도구가 수정하지 않는다.
