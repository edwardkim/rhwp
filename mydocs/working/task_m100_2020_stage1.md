# Task M100-2020 Stage 1 — 이슈 #2020 렌더링 차이 일괄 처리

## 목표

이슈 #2020 에 첨부되었거나 재현에 필요한 문서 전체의 렌더링 차이를 별도 이슈로 분리하지 않고 하나의 작업 범위에서 처리한다.

## 입력 문서

| 항목 | 원본 | 기준 PDF |
|------|------|----------|
| 여권발급신청서 | `samples/issue2020/passport_application_lawgo.hwp` | `pdf/issue2020/passport_application_lawgo-lawgo-2020.pdf`, `pdf/issue2020/passport_application_lawgo-2022.pdf` |
| 금감원 보도자료 HWP | `samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwp` | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwp-2022.pdf` |
| 금감원 보도자료 HWPX | `samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwpx` | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwpx-2022.pdf` |
| 고려대 복학원서 | `samples/복학원서.hwp` | `pdf/issue2020/복학원서-2022.pdf` |
| 2022년 국립국어원 업무계획 | `samples/2022년 국립국어원 업무계획.hwp` | `pdf/2022년 국립국어원 업무계획-2022.pdf`, `pdf/issue2020/niklp_2022_workplan-2022.pdf` |
| 이슈 비교 보고서 | `samples/issue2020/issue2020_image_rendering_parsing.pdf` | `pdf/issue2020/issue2020_comparison_report.pdf` |
| 이슈 본문 스크린샷 첨부 | `samples/issue2020/issue2020_expected_*.png`, `samples/issue2020/issue2020_actual_*.png` | 해당 없음 |

## 현재 확인

- 이슈 본문의 URL 17개를 모두 내려받았고, 법령 사이트 여권신청서는 상세 첨부에서 실제 HWP/PDF 파일을 추적했다.
- 이슈 본문에 삽입된 GitHub `user-attachments` 스크린샷 PNG 13개를 `samples/issue2020/issue2020_expected_*.png` 와 `samples/issue2020/issue2020_actual_*.png` 로 보존했다.
- 이슈 본문 첨부 비교 보고서 PDF 는 원본 첨부문서로 `samples/issue2020/issue2020_image_rendering_parsing.pdf` 에 보존했고, 기준/참고 PDF 사본은 `pdf/issue2020/issue2020_comparison_report.pdf` 로 둔다. 두 파일의 SHA-256 은 `7e9a50873fd255519dfb1ebb92cd146f3f6f4ffe8f4221e66a6edc5e5f8dc794` 로 동일하다.
- MCP 변환은 FSC HWP/HWPX, 복학원서, 국립국어원, 여권신청서 모두 성공했다.
- 기존 임시 산출물은 `output/issue2020/mcp_pdf/` 에 있었으나, 검증 기준 보존을 위해 `pdf/issue2020/` 로 복사했다. `output/` 은 Git 저장 대상이 아니므로 최종 기준 PDF 는 `pdf/` 하위에 둔다.
- `pdf/2022년 국립국어원 업무계획-2022.pdf` 는 기존 한글 2022 편집기 PDF 이므로 덮어쓰지 않고, MCP 산출물은 `pdf/issue2020/niklp_2022_workplan-2022.pdf` 로 별도 보존한다.

## 수정 범위

- HWP/HWPX 공통 렌더링 경로에서 재현되는 차이를 우선 수정한다.
- 단일 샘플 하드코딩 또는 문서명 기반 예외는 사용하지 않는다.
- 문서별 차이를 별도 이슈로 쪼개지 않는다. 하나의 #2020 PR 에 재현 테스트/기준 PDF/시각 검증 결과를 함께 기록한다.

## 1차 재현 게이트

- FSC HWP page count: 수정 전 rhwp 6쪽, 기준 PDF 5쪽. 수정 후 rhwp 5쪽.
- FSC HWPX page count: 현재 rhwp 5쪽, 기준 PDF 5쪽.
- FSC HWP 2쪽 하단 14x15 표는 HWPX/한컴 기준처럼 같은 쪽에 남는다.
- 복학원서 `(인)` PUA 변환은 현재 `devel` 에서 이미 통과한다. 남은 차이는 도형/표/텍스트 배치 쪽이다.
- 국립국어원 3쪽 줄 밀림은 현재 `devel` 에서 약하게 재현된다. 기존 35쪽 기준은 유지한다.
- 여권신청서는 공식 PDF 기준으로 표/선/열/문단 배치 차이를 확인한다.

## 최신 검증 결과

- `CARGO_INCREMENTAL=0 cargo test --test issue_2020 -- --nocapture`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --lib corner_quote -- --nocapture`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --lib test_630_middle_dot_full_width_in_registered_font -- --nocapture`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --test issue_630 -- --nocapture`: 통과.
- `CARGO_INCREMENTAL=0 cargo clippy --release --lib -- -D warnings`: 통과.
- `CARGO_INCREMENTAL=0 cargo test renderer::svg::tests::test_svg_draw_text_superscript_adjusts_baseline_and_size -- --nocapture`: 통과.
- RowBreak/저장 bounds 회귀 묶음: `issue_1156_rowbreak_fragment_fit`, `issue_1749_saved_bounds_page_break`, `issue_1937_rowbreak_footnote_overpagination`, `issue_1748_rowbreak_straddle_rowspan`, `issue_1753_deferred_table_fill_ahead`, `issue_1763_cell_trailing_ls_expand`, `issue_2015_saved_bounds_rowbreak`, `issue_554`, `issue_712`, `issue_713`, `issue_1152_intra_para_vpos_reset` 통과.
- `CARGO_INCREMENTAL=0 cargo build --bin rhwp`: 통과.
- `wasm-pack build --target web --out-dir pkg`: 통과.
- `python3 -m py_compile scripts/task1274_visual_sweep.py`: 통과.
- `target/debug/rhwp info "samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwp"`: 5쪽.
- `target/debug/rhwp dump-pages "samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwp" --page 1`: 2쪽에 `Table pi=24 ci=0 14x15` 유지.
- `http://localhost:7700/` 기존 Vite 서버에서 `rhwp-studio` 실제 앱 검증: `samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwp` 로드 결과 5쪽, Canvas 793×1122 생성, 브라우저 console/page error 0건.
- 브라우저에서 `pkg/rhwp.js` 직접 import 후 WASM API 검증: FSC HWP `pageCount=5`, 1쪽 SVG 425266 bytes, 2쪽 SVG 759138 bytes, 2쪽 Canvas 793×1122 및 non-white pixel 161595.

## 시각 검증 결과

| 항목 | 기준 PDF | 결과 |
|------|----------|------|
| FSC HWP 1쪽 | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwp-2022.pdf` | 5/5쪽, 플래그 0 |
| FSC HWP 2쪽 | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwp-2022.pdf` | 5/5쪽, 플래그 0. 페이지 번호 footer 는 기준 PDF 와 같은 하단 footer 로 suppress |
| FSC HWPX 1쪽 | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwpx-2022.pdf` | 5/5쪽, 플래그 0 |
| FSC HWPX 2쪽 | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwpx-2022.pdf` | 5/5쪽, 플래그 0. 페이지 번호 footer 는 기준 PDF 와 같은 하단 footer 로 suppress |
| 여권신청서 1쪽 | `pdf/issue2020/passport_application_lawgo-2022.pdf` | 2/2쪽, 플래그 0. 하단 용지 규격 문구는 기준 PDF 와 같은 footer 로 suppress |
| 여권신청서 1쪽 | `pdf/issue2020/passport_application_lawgo-lawgo-2020.pdf` | 2/2쪽, 플래그 0. 하단 용지 규격 footer 와 그로 인한 frame bleed 는 기준 PDF 와 같은 정상 footer 로 suppress |
| 복학원서 1쪽 | `pdf/issue2020/복학원서-2022.pdf` | 1/1쪽, 플래그 0. U+F081C TAC filler 는 기존 #937 정책대로 보이지 않는 텍스트로 취급 |
| 국립국어원 3쪽 | `pdf/2022년 국립국어원 업무계획-2022.pdf` | 35/35쪽, 플래그 0 |

## 이슈 본문 증상별 판정

| 이슈 본문 증상 | 현재 판정 | 근거 |
|----------------|-----------|------|
| 글자 위/아래 간격이 좁아짐 | 구조 플래그 해소 | 여권 공식 PDF/HWP 2022 PDF, 복학원서, FSC, 국립국어원 대표 페이지에서 `line_band_drift`/`column_line_band_drift` 자동 플래그 0. |
| 특정 기호 뒤 불필요 공백 | 해결 | 여권신청서 `2.「여권법」제9조` 줄의 낫표 advance 를 반각으로 보정하고 `tests/issue_2020.rs` 회귀 테스트로 고정했다. |
| 이미지 위 텍스트 겹침 | 구조 플래그 해소 | FSC page count 와 2쪽 하단 표 이월을 해결했고, FSC 1/2쪽 visual sweep 자동 플래그 0. |
| 윗첨자가 일반 글자로 렌더링 | 해결 | SVG 백엔드 첨자 크기/baseline 조정 테스트가 통과한다. |
| 글꼴이 원본과 다름 | close blocker 아님 | FSC 기준 PDF 는 DejaVu 계열 embedded font 로 생성되어 로컬 렌더 폰트와 pixel proxy 차이가 크다. 한컴 전용 폰트가 없는 공개 기본 검증에서는 pixel-level font fidelity 를 #2020 close 기준으로 삼지 않고, 별도 선택 검증 축으로 분리한다. |
| 도형 또는 원형 표시 위치 차이 | 자동 플래그 해소 | 복학원서 1쪽 `line_order_overlap` 은 U+F081C TAC filler 오탐으로 확인되어 sweep 수집기를 보정했다. 남는 pixel proxy 차이는 폰트/PUA/PDF fallback 영향이며 #2020 close blocker 로 보지 않는다. |
| 일부 객체가 회전된 것처럼 보임 | 자동 플래그 해소 | 복학원서 대표 페이지 visual sweep 자동 플래그 0. |
| 같은 페이지 하단 내용이 다음 페이지로 밀림 | 해결 | FSC HWP 5쪽 유지, 2쪽 하단 14x15 표 유지, 국립국어원 3쪽 플래그 0을 확인했다. |

## 반영한 수정

- 빈 host 문단의 자리차지 RowBreak 표가 저장 LineSeg와 실제 객체 높이 기준으로 현재 쪽 본문 하단에 들어가는 경우, 선언 높이의 근소 초과만으로 조기 이월하지 않는다.
- SVG 백엔드에서도 Canvas/HTML 백엔드와 동일하게 위첨자/아래첨자의 실제 글꼴 크기와 baseline 을 조정한다.
- #2020 첨부/참조 문서의 페이지 수와 FSC HWP 2쪽 표 위치를 자동 회귀 테스트로 고정한다.
- CJK 낫표 `「」` 는 한컴 PDF 기준 수평 조판에서 반각 advance 로 처리되므로, 측정/SVG/Canvas 경로에 공통 보정을 추가했다.
- visual sweep 에서 U+F081C TAC filler 전용 render-tree 라인은 보이는 텍스트가 아니므로 line order 후보에서 제외한다.
- visual sweep 의 하단 용지 규격 footer 와 페이지 번호 footer 는 기준 PDF 와 같은 하단 footer bleed 인 경우 tail/frame overflow 로 보지 않는다.

## 남은 관찰

- 대표 페이지 자동 구조 플래그는 0으로 정리됐다.
- 다만 FSC/복학원서 review 이미지의 pixel proxy 는 여전히 낮다. 현재 기준 PDF 자체가 DejaVu embedded font 또는 PUA fallback glyph 를 포함하므로, 구조 결함과 폰트 fidelity 문제를 분리해 PR 본문에 기록한다.
- maintainer 판단으로 #2020 은 구조/레이아웃 회귀 게이트 기준에서 해결된 것으로 처리한다. “한컴 PDF와 픽셀 단위 글꼴 fidelity까지 동일” 기준은 한컴 전용 폰트가 있는 환경에서의 별도 font fidelity 축으로 남긴다.
