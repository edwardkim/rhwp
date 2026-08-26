---
kind: implementation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 76 — 76076 p35--p36 RowBreak rowspan fragment boundary

## 입력 계약

Stage 75는 실제 `HYGothic-Medium` font를 Mac·두 Ubuntu host에 설치하고
`한양중고딕 → HY중고딕` alias를 renderer에 명시했지만,
`76076_regulatory_analysis.hwp` p33--p36의 RHWP raster가 Stage 74와
pixel-identical임을 확인했다. 따라서 font 선택은 이 구간의 PDF fidelity 결함을 설명하지
못한다.

공식 기준은 `samples/issue1891/76076_regulatory_analysis-2024.pdf`이고, 비교 대상은
동일 HWP의 RHWP SVG raster다. Stage 75 p34 review에서 RHWP 우측 대형 table cell은 PDF와
비교해 line density·fragment height가 크게 달라져 아래 table/row의 page owner도 연쇄적으로
다르다. 자동 sweep의 후보 수 0은 이 결함이 검출되지 않았다는 뜻일 뿐 정합성 통과가 아니다.

## 분석 질문

1. p35에서 PDF와 RHWP가 서로 다른 table row/partial fragment를 page owner로 선택하는가?
2. owner가 다르면 fragment continuation의 remaining-height·break predicate 중 어떤 값이
   최초로 diverge하는가?
3. 수정 후 p35--p36을 한 묶음으로 다시 비교해 page 35만 맞추고 다음 page를 악화시키지
   않는가?

## 확정 분석 (2026-08-08)

초기 이름과 달리 이 경계의 직접 원인은 중첩 표가 아니다. `dump --section 0 --para 347`의
외부 `RowBreak` 표(31×17)에서 다음 행이 최초로 갈린다.

- row 12 `주요내용`: 선언 높이 7,410 HU = 98.8px, 실제 내용은 한 줄(약 23.3px).
- p35의 row 12 직전 잔여: 74.7px. RHWP는 `rowspan_touched[12]`와
  `MeasuredTable::is_row_splittable(12) == false`를 이유로 행 전체를 p36으로 이월했다.
- 한컴 2024 PDF는 p35 하단(y≈736pt)에 `주요내용`을 그리고, p36에는 그 행의 **빈 하단
  밴드만** 남긴 뒤 `11.영향평가 여부`를 시작한다. 즉 p36에 `주요내용` 텍스트를 재방출하면
  안 되지만, 그 행의 나머지 테두리도 제거하면 안 된다.

`typeset.rs::scan_block_table_split_rows`의 기존 `RSPAN_STOP`이 `advance_row_cut`의
"내용 완전 소비" 가능성을 보기 전에 멈춘 것이 원인이다. 모든 rowspan 행을 느슨하게
분할하지 않고, **이전 행에서 시작한 rowspan이 닿고, 중첩 표가 없으며, 현재 쪽 잔여 안에
실제 내용 전체가 들어가는 RowBreak 행**에만 밴드 경로를 허용한다.

## 구현과 1차 증거

- `PageItem::PartialTable`와 continuation cursor에 마지막 행 높이와 다음 쪽의 빈 tail
  높이를 명시적으로 보존했다. 따라서 p35에는 `주요내용` + 74.7px 밴드가, p36에는
  내용 없는 약 24.1px tail 밴드가 렌더된다.
- 새 focused regression `tests/issue_3820_rowbreak_rowspan_band.rs`는 p35의
  `주요내용` y≈979px, p36의 텍스트 부재와 `11.영향평가` y≈108px를 고정한다.
- 180 DPI direct raster 비교에서 p36 표의 시작 blank band와 `11.영향평가`의 시작 위치가
  PDF와 정렬됨을 확인했다. 이 확인은 **p35의 별도 중첩 표 누락 후보를 해결했다는 뜻은
  아니다**. p35 row 8 내부 표가 RHWP에서 빠지는 현상은 후속 분리 분석 대상이다.
- 같은 direct comparison에서 p36의 표 구조는 이제 PDF와 같은 continuation owner를
  사용하지만, p35--p36 전체가 pixel-identical한 상태는 아니다. p35의 row 8 내부 표와
  그에 따른 text/ink 차이는 그대로 남아 있어, 자동 sweep의 `flagged_page_count=0`을
  fidelity 통과로 해석하지 않는다.

## 증적과 방법

- Stage 75 direct sweep: `mydocs/pr/assets/task_m100_3820_stage75_hanyang_font_environment/`.
- RHWP render tree의 p33--p36 table/row/cell bounds를 추출해 page별 owner 및 fragment
  continuation을 표로 만든다.
- PDF는 `pdftotext -bbox-layout`와 180 DPI raster를 함께 사용한다. PDF text bbox만으로
  table 구조를 추정하지 않고 border·ink band와 교차 확인한다.
- root cause가 확인되기 전에는 cell padding·line-height를 전역 보정하지 않는다. 이 문서는
  Stage 75의 font 가설이 기각된 뒤의 구조 분석 checkpoint다.
- PDF direct evidence: [p35 review](../pr/assets/task_m100_3820_stage76_rowbreak_rowspan_band/review_035.png),
  [p36 review](../pr/assets/task_m100_3820_stage76_rowbreak_rowspan_band/review_036.png),
  [p35 side-by-side](../pr/assets/task_m100_3820_stage76_rowbreak_rowspan_band/compare_035.png),
  [p36 side-by-side](../pr/assets/task_m100_3820_stage76_rowbreak_rowspan_band/compare_036.png),
  [summary](../pr/assets/task_m100_3820_stage76_rowbreak_rowspan_band/summary.json).

## 완료 조건

1. focused regression, issue1891 page-count, overflow-cell gate를 실행한다. 실행 결과는
   모두 통과했다. overflow-cell은 678개 sample(스킵 3), nonzero 17개 문서, 총 691줄로
   baseline 증가가 없었다.
2. p35 row 8 내부 표 누락 여부를 별도 stage에서 분석한다. 이 stage의 RowBreak tail
   수정과 섞어 원인을 숨기지 않는다.

## 완료 기록

- `cargo test --profile release-test --test issue_3820_rowbreak_rowspan_band -- --nocapture`: 통과.
- `cargo test --profile release-test --test issue_1891 -- --nocapture`: 통과.
- `cargo test --profile release-test --test overflow_cell_baseline -- --nocapture`: 통과.
- `cargo test --profile release-test --tests`: 통과 (exit 0).
- `cargo clippy --profile release-test --all-targets -- -D warnings`: 통과.
- `cargo fmt --check`, `git diff --check`: 아래 커밋 직전에 재실행한다.
