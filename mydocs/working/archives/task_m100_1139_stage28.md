# Task M100 #1139 Stage 28

## 목적

Stage27 커밋 이후 남은 `3-09월_교육_통합_2022.hwp` 20쪽 하단 렌더링 차이를 별도 스테이지로 추적한다.

## 시작 기준

- 기준 커밋: `d82781a8029b7e52278c19e8b648b9768f8c46cb` (`task 1139: Stage27 22쪽 미주 경계와 표 렌더 보정`)
- Stage27 변경은 커밋 완료했다.
- Stage28 문서는 Stage27 커밋 이후 새 변경으로 생성한다.
- Stage28 소스 수정은 작업지시자 승인 후 진행한다.

## 보고된 문제

- 작업지시자 시각 검증에서 20쪽 하단에 문제가 있다고 보고되었다.
- 첨부 비교 기준으로 20쪽 하단에서 `문27)` 시작 위치와 다음 페이지 상단으로 이어지는 흐름이 한컴오피스와 다르다.
- 20쪽 하단 문제는 Stage27에서 건드린 미주 table-only 렌더와 별개로, 미주 문단 분배/하단 fit/lineSeg vpos 되감기 처리의 잔여 문제일 가능성이 있다.

## 진행 계획

1. 20쪽과 21쪽 RHWP SVG/dump를 Stage28 기준으로 다시 추출한다.
2. `pdf/3-09월_교육_통합_2022.pdf`에서 19/20/21쪽 기준 이미지를 직접 추출하고 RHWP 19/20/21쪽 이미지와 같은 배율로 비교한다.
3. 20쪽 하단 `문27)` 주변 paragraph index, lineSeg vpos, `PartialParagraph` 분할 위치를 고정한다.
4. 21쪽 상단으로 넘어간 첫 항목이 한컴 기준보다 빠르거나 늦은지 확인한다.
5. 원인이 분리되면 회귀 테스트를 추가하고 Rust/WASM 수정 후 시각 검증 산출물을 만든다.

## 초기 분석

- Stage28 기준 20쪽 SVG export는 `LAYOUT_OVERFLOW` 로그 없이 완료되지만, 시각 비교에서 오른쪽 단 하단 `문27)` 시작이 한컴/PDF보다 아래로 밀려 페이지 테두리에 붙는다.
- `dump-pages -p 19` 기준 20쪽 오른쪽 단 하단:
  - `pi=1087`: `문27)   ③`
  - `pi=1088`: `두 점 , 에서 두 점 , 를 포함하는 밑면에 내린 수선의 발을 각각 ,`
- `dump-pages -p 20` 기준 21쪽 시작:
  - `pi=1089`: 빈 문단
  - `pi=1089 ci=0`: 큰 그림 TAC shape
- 한컴/PDF 기준과 RHWP를 나란히 보면 20쪽 `문27)` 블록의 페이지 하단 fit 또는 vpos 정규화가 아직 맞지 않는다.
- Stage27에서 추가한 `height_cursor` 제목 직후 제한 backtrack 또는 `typeset`의 후반 미주 tail 예외가 20쪽 `문27)` 하단 배치에 영향을 줬는지 우선 확인한다.

## PDF 기준 재확인

- 기준 PDF: `pdf/3-09월_교육_통합_2022.pdf`
  - `pdfinfo` 기준 23쪽 A4, Creator `Hwp 2024 13.0.0.3457`, Producer `Hancom PDF 1.3.0.550`
- PDF 19/20/21쪽과 RHWP 19/20/21쪽을 144dpi로 재추출했다.
- PDF 19쪽 오른쪽 단 하단에는 문29 풀이의 `이때, s=0일 때, t=2이므로`까지 들어간다.
- RHWP 19쪽은 `pi=1019`에서 끝나고, PDF 19쪽 하단에 들어가야 할 `pi=1020/1021` 상당 흐름이 20쪽 상단으로 넘어간다.
- 따라서 20쪽 `문27)`이 아래로 밀린 직접 원인은 20쪽 하단 단독 문제가 아니라, 19쪽 마지막 단에서 문29 후반 tail을 충분히 담지 못해 20쪽 시작점이 앞당겨진 것이다.

## PDF 23쪽 단일 비교

- 작업지시자가 "23 Page pdf 와 비교"는 23쪽 전체가 아니라 PDF의 23페이지만 비교하라는 의미라고 정정했다.
- SVG를 PNG로 변환할 때 ImageMagick을 쓰지 않고 `rsvg-convert`를 사용한다.
- 23쪽 전용 산출물:
  - PDF 기준: `output/task1139_stage28_page23_compare/pdf/page-23.png`
  - RHWP SVG: `output/task1139_stage28_page23_compare/rhwp_svg/3-09월_교육_통합_2022_023.svg`
  - RHWP PNG: `output/task1139_stage28_page23_compare/rhwp_png/page-23.png`
  - 비교 HTML: `output/task1139_stage28_page23_compare/compare_page23.html`
  - dump: `output/task1139_stage28_page23_compare/dump/page23.txt`
- 23쪽 export 로그에는 `LAYOUT_OVERFLOW`가 없다.
- 시각 비교 결과 PDF 23쪽 상단에는 그림과 문단 전반부가 표시되지만, RHWP 23쪽은 상단에 `호이다.`만 남고 그림 영역이 비어 있으며 본문 후반부가 아래로 밀린다.
- `dump-pages -p 22` 기준 RHWP 23쪽은 `pi=1175`의 `PartialParagraph`에서 시작하므로 페이지 번호 자체보다 해당 문단 내부의 그림/라인 배치가 우선 조사 대상이다.
- 추가 확인 결과 PDF 23쪽 상단 그림에 해당하는 RHWP 항목은 `pi=1175` line 10의 `Picture ci=20`이다. 기존 렌더러는 split된 partial paragraph 내부의 "빈 줄 + TAC Picture" line을 건너뛰어 그림을 누락했다.

## 구현 기록

- 19쪽 마지막 단에서 문29 풀이 tail(`pi=1020/1021`)이 20쪽으로 밀리지 않도록, 문29 후반 작은 continuation tail은 좁은 overflow 범위에서 현재 단에 남기도록 보정했다.
- `paragraph_layout`에서 빈 runs line마다 해당 line의 char 범위에 걸린 TAC만 선별해 이미지/도형을 렌더하도록 보정했다.
- 수정 후 RHWP 23쪽은 PDF처럼 상단 그래프가 23쪽에 표시되고, 뒤따르는 풀이 문단도 그래프 아래에서 시작한다.

## 산출물

- RHWP 20쪽 SVG: `output/task1139_stage28_page20/page20/3-09월_교육_통합_2022_020.svg`
- RHWP 21쪽 SVG: `output/task1139_stage28_page20/page21/3-09월_교육_통합_2022_021.svg`
- 20쪽 비교 이미지: `output/task1139_stage28_page20/compare_page20_pdf_rhwp.png`
- 20/21쪽 dump: `output/task1139_stage28_page20/dump_page20.txt`, `output/task1139_stage28_page20/dump_page21.txt`
- PDF 직접 추출 기준 산출물: `output/task1139_stage28_pdf_source/`
  - `compare_page19_pdf_rsvg.png`
  - `compare_page20_pdf_rsvg.png`
  - `compare_page21_pdf_rsvg.png`
  - `dump_page19.txt`, `dump_page20.txt`, `dump_page21.txt`
- PDF 23쪽 단일 비교 산출물: `output/task1139_stage28_page23_compare/`
  - 비교 HTML: `compare_page23.html`
  - 기준 PDF: `pdf/page-23.png`
  - 수정 후 RHWP: `rhwp_png/page-23.png`, `rhwp_svg/3-09월_교육_통합_2022_023.svg`
  - dump: `dump/page23.txt`

## 검증 기록

- `cargo fmt --all`
- `cargo build`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `cargo fmt --all --check`
- `wasm-pack build --target web --out-dir pkg`
- `git diff --check`
- PDF 23쪽 단일 비교 산출물 `output/task1139_stage28_page23_compare/compare_page23.html` 기준 작업지시자 시각 검증 완료

## 승인 상태

- 2026-05-30: 작업지시자가 Stage27 커밋 후 새 스테이지 시작과 20쪽 하단 문제 확인을 지시했다.
- 2026-05-30: Stage28 문서를 생성했다.
- 2026-05-30: 20쪽/21쪽 SVG와 dump, PDF/RHWP 비교 이미지를 추출해 `문27)` 하단 배치 문제를 확인했다.
- 2026-05-30: 작업지시자가 Stage28 해결 진행을 지시했다.
- 2026-05-30: 작업지시자가 기준 PDF 위치 `pdf/3-09월_교육_통합_2022.pdf`를 확인해 주었다.
- 2026-05-30: 작업지시자가 PDF 23쪽 단일 비교와 `rsvg-convert` 사용을 지시했다.
- 2026-05-30: 23쪽 split 미주 내부 빈 줄 TAC Picture 누락을 보정하고 회귀 테스트를 추가했다.
- 2026-05-30: 작업지시자가 23쪽 시각 검증 완료를 확인하고 현재 상황 커밋을 지시했다.
- Stage28 커밋 진행.
