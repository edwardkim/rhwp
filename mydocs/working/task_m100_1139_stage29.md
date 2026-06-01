# Task M100 #1139 Stage 29

## 목적

Stage28 커밋 이후 `3-09월_교육_통합_2023.hwp` 4쪽 `문26)` 겹침 문제를 별도 스테이지로 추적한다.

## 시작 기준

- 기준 커밋: `ea0b39c8` (`task 1139: Stage28 23쪽 split 그림 렌더 보정`)
- Stage28 변경은 커밋 완료했다.
- Stage29 문서는 Stage28 커밋 이후 새 변경으로 생성한다.
- Stage29 소스 수정은 작업지시자 승인 후 진행한다.

## 보고된 문제

- 대상 파일: `samples/3-09월_교육_통합_2023.hwp`
- 기준 PDF: `pdf/3-09월_교육_통합_2023.pdf`
- 작업지시자 시각 확인에서 4쪽 `문26)` 영역의 본문/표/보기 텍스트가 서로 겹친다고 보고되었다.
- 한컴오피스 기준 4쪽에서는 `문26)` 본문과 우측의 표가 같은 영역에서 겹치지 않고, 이어지는 5쪽 시작부도 정상 흐름을 유지한다.
- RHWP 기준 4쪽에서는 `문26)` 주변에서 표 또는 문단 높이 계산이 부족해 본문 줄과 표/보기 항목이 중첩되는 것으로 보인다.

## 진행 계획

1. 2023 HWP 4쪽과 기준 PDF 4쪽을 같은 배율로 추출한다.
2. SVG를 PNG로 변환할 때는 `rsvg-convert`를 사용한다.
3. 4쪽 `dump-pages`를 추출해 `문26)` 주변 paragraph index, table/control index, lineSeg 높이, column/page 경계를 고정한다.
4. 겹침 원인이 table-only 렌더, TAC 개체 높이, lineSeg 높이, 미주/본문 흐름 중 어디에 있는지 분리한다.
5. 원인이 확인되면 4쪽 `문26)` 겹침 방지 회귀 테스트를 추가한다.
6. Rust/WASM 수정 후 `cargo fmt --all --check`, `cargo build`, 관련 회귀 테스트, `wasm-pack build --target web --out-dir pkg`, `git diff --check`를 실행한다.
7. PDF/RHWP 4쪽 비교 산출물을 생성해 작업지시자 시각 확인을 받는다.

## 분석 결과

- 4쪽 `문26)` 겹침 위치는 왼쪽 단 하단 `pi=258`의 표준정규분포표였다.
- 해당 표는 `ci=5`, `wrap=Square`, `treat_as_char=false`, `vert=Para`, `halign=Right`인 어울림 표다.
- HWP LineSeg는 `pi=258`의 마지막 줄만 `segment_width`가 줄어들어 있어, 한컴은 표를 문단 첫 줄이 아니라 후반 줄 오른쪽에 붙인다.
- 기존 렌더러는 Square wrap 표의 세로 위치를 문단 시작 y로만 잡아 표가 위로 올라갔고, 본문 줄과 표 셀이 서로 겹쳤다.
- 본문과 선택지의 흐름은 크게 밀린 것이 아니라 표의 시각 y만 잘못된 것이므로, 표를 내려 그리되 후속 flow y는 기존처럼 유지해야 한다.

## 구현 내용

- `src/renderer/layout.rs`
  - `square_wrap_table_line_anchor_y`를 추가했다.
  - 비-TAC Square wrap 표가 문단 기준/위쪽 정렬/오른쪽 정렬이고, LineSeg 중 후반 줄의 `segment_width`가 표 폭만큼 줄어든 경우 해당 줄의 `vertical_pos`를 표의 시각 anchor y로 사용한다.
  - 표를 anchor line 위치에 그리되, flow y는 기존 문단 흐름을 유지해 선택지 위치가 불필요하게 아래로 밀리지 않도록 했다.
  - 빈 host 문단에 TAC Picture가 있고 같은 문단의 `FullParagraph` 항목이 이미 발행된 경우, `Shape` PageItem 경로에서 그림은 렌더하되 y 흐름을 다시 진행하지 않도록 했다. 이중 진행으로 12쪽/13쪽 미주 그림 뒤 문단이 하단을 넘던 현상을 제거했다.
- `src/renderer/typeset.rs`
  - compact 미주에서 TAC Picture/Shape 포함 문단의 formatter 높이가 HWP LineSeg vpos advance보다 크게 튀는 경우, HWP 저장본의 vpos advance를 우선하도록 보정했다.
  - 이 보정이 발생한 직후의 기본 미주 간격 새 문제 제목은, 앞 두 문단이 현재 단 하단에 들어갈 때만 이전 단 tail로 허용했다. 2022 10쪽 `문11)` 회귀를 막기 위해 조건을 이전 미주의 inline-object vpos overestimate 발생 직후로 제한했다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - `samples/3-09월_교육_통합_2023.hwp` 4쪽 `pi=258 ci=5` 표가 문단 첫 줄이 아니라 LineSeg가 좁아지는 후반 줄에 붙어야 한다는 회귀 테스트를 추가했다.
  - 2023 12쪽 왼쪽 단 하단에 `문14)` 제목/첫 풀이가 남고, 13쪽은 `pi=637` 그래프부터 시작해야 한다는 페이지 경계 회귀 테스트를 추가했다.

## 산출물

- 기준 PDF 4쪽: `output/task1139_stage29_page4_compare/pdf/page-4.png`
- RHWP 4쪽 SVG/PNG: `output/task1139_stage29_page4_compare/rhwp_svg/`, `output/task1139_stage29_page4_compare/rhwp_png/page-4.png`
- 디버그 오버레이: `output/task1139_stage29_page4_compare/debug_svg/`, `output/task1139_stage29_page4_compare/debug_png/page-4-debug.png`
- 비교 HTML: `output/task1139_stage29_page4_compare/compare_page4.html`
- dump: `output/task1139_stage29_page4_compare/dump/page4.txt`, `output/task1139_stage29_page4_compare/dump/page4_after.txt`

## 12쪽 추가 비교

- 작업지시자 요청으로 `pdf/3-09월_교육_통합_2023.pdf` 12쪽과 RHWP 12쪽을 추가 비교했다.
- SVG → PNG 변환은 `rsvg-convert`를 사용했다.
- 산출물:
  - 기준 PDF 12쪽: `output/task1139_stage29_page12_compare/pdf/page-12.png`
  - RHWP 12쪽 SVG/PNG: `output/task1139_stage29_page12_compare/rhwp_svg/`, `output/task1139_stage29_page12_compare/rhwp_png/page-12.png`
  - 디버그 오버레이: `output/task1139_stage29_page12_compare/debug_svg/`, `output/task1139_stage29_page12_compare/debug_png/page-12-debug.png`
  - 비교 HTML: `output/task1139_stage29_page12_compare/compare_page12.html`
  - dump: `output/task1139_stage29_page12_compare/dump/page12.txt`
- 비교 결과:
  - PDF 기준 12쪽은 왼쪽 단 하단에서 `문14)` 제목이 시작되고, 오른쪽 단 상단에는 `문14)` 그래프/풀이가 이어진다.
  - RHWP 12쪽은 오른쪽 단 상단에 아직 이전 풀이 tail(`pi=609`, `pi=610`)이 남아 있고, `문14)` 시작이 아래로 밀린다.
  - RHWP 12쪽 오른쪽 단 하단에서 `pi=625..634`가 body 하단을 넘는 `LAYOUT_OVERFLOW`를 발생시킨다.
  - 따라서 12쪽은 PDF와 시각적으로 불일치하며, 4쪽 `문26)` 보정과 별개로 미주 단 분배/페이지 경계 보정 후보로 남는다.

## 13쪽 추가 비교

- 작업지시자 요청으로 `pdf/3-09월_교육_통합_2023.pdf` 13쪽과 RHWP 13쪽을 추가 비교했다.
- SVG → PNG 변환은 `rsvg-convert`를 사용했다.
- 산출물:
  - 기준 PDF 13쪽: `output/task1139_stage29_page13_compare/pdf/page-13.png`
  - RHWP 13쪽 SVG/PNG: `output/task1139_stage29_page13_compare/rhwp_svg/`, `output/task1139_stage29_page13_compare/rhwp_png/page-13.png`
  - 디버그 오버레이: `output/task1139_stage29_page13_compare/debug_svg/`, `output/task1139_stage29_page13_compare/debug_png/page-13-debug.png`
  - 비교 HTML: `output/task1139_stage29_page13_compare/compare_page13.html`
  - dump: `output/task1139_stage29_page13_compare/dump/page13.txt`
- 비교 결과:
  - PDF 기준 13쪽은 12쪽 오른쪽 단 하단에서 이어진 그래프가 13쪽 왼쪽 단 상단에 바로 시작한다.
  - RHWP 13쪽은 첫머리에 이전 페이지 tail 텍스트가 남고, 그래프와 `문15)` 시작 위치가 PDF보다 아래로 밀린다.
  - RHWP export에서 왼쪽 단 `pi=656`, `pi=657`이 body 하단을 넘고, 오른쪽 단 `pi=695`도 하단에 약간 걸친다.
  - 따라서 13쪽도 PDF와 시각적으로 불일치하며, 12쪽에서 확인된 미주 단 분배/페이지 경계 문제의 연속 증상으로 본다.

## 12쪽/13쪽 보정 결과

- 원인 1: 12쪽 왼쪽 단 `pi=605`는 TAC 도형 포함 미주 문단이다. HWP LineSeg 기준 advance는 약 295px인데 formatter height floor가 약 445px로 잡혀 단 높이가 약 150px 과충전되었다.
- 원인 2: 직전 미주에 vpos rewind가 있으면 다음 미주 묶음을 통째로 다음 단으로 보내는 보호 규칙이 `문14)` 제목까지 오른쪽 단으로 밀었다.
- 원인 3: 빈 host 문단의 TAC Picture가 `FullParagraph` 진행 뒤 `Shape` PageItem에서 다시 y를 진행해 실제 SVG 렌더에서 dump보다 더 아래로 밀렸다.
- 보정 후 dump 기준:
  - 12쪽 왼쪽 단은 `pi=611` `문14)` 제목과 `pi=612` 첫 풀이까지 포함한다.
  - 12쪽 오른쪽 단은 `pi=613` 그래프 host부터 `pi=636` tail까지 포함한다.
  - 13쪽 왼쪽 단은 `pi=637` 그래프부터 시작하고, 이전 tail `pi=635/636`은 남지 않는다.
- 보정 후 12쪽/13쪽 SVG export와 debug overlay export에서 `LAYOUT_OVERFLOW`가 발생하지 않았다.
- 갱신 산출물:
  - 12쪽 RHWP PNG: `output/task1139_stage29_page12_compare/rhwp_png/page-12.png`
  - 12쪽 debug PNG: `output/task1139_stage29_page12_compare/debug_png/page-12-debug.png`
  - 12쪽 보정 dump: `output/task1139_stage29_page12_compare/dump/page12_after_12_13_fix.txt`
  - 13쪽 RHWP PNG: `output/task1139_stage29_page13_compare/rhwp_png/page-13.png`
  - 13쪽 debug PNG: `output/task1139_stage29_page13_compare/debug_png/page-13-debug.png`
  - 13쪽 보정 dump: `output/task1139_stage29_page13_compare/dump/page13_after_12_13_fix.txt`

## 검증 기록

- `cargo fmt --all`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1139_2023_page4_question26_square_table_uses_anchor_line -- --nocapture`: 통과
- `cargo build`: 통과
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2023.hwp -p 3 -o output/task1139_stage29_page4_compare/rhwp_svg`: 통과
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2023.hwp -p 3 --debug-overlay -o output/task1139_stage29_page4_compare/debug_svg`: 통과
- `rsvg-convert`로 4쪽 SVG/디버그 SVG를 PNG 변환: 통과
- `cargo fmt --all --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 통과, 23개 테스트
- `cargo build`: 통과
- `git diff --check`: 통과
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2023.hwp -p 11 -o output/task1139_stage29_page12_compare/rhwp_svg`: 통과, `LAYOUT_OVERFLOW` 없음
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2023.hwp -p 11 --debug-overlay -o output/task1139_stage29_page12_compare/debug_svg`: 통과, `LAYOUT_OVERFLOW` 없음
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2023.hwp -p 12 -o output/task1139_stage29_page13_compare/rhwp_svg`: 통과, `LAYOUT_OVERFLOW` 없음
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2023.hwp -p 12 --debug-overlay -o output/task1139_stage29_page13_compare/debug_svg`: 통과, `LAYOUT_OVERFLOW` 없음
- `rsvg-convert`로 12쪽/13쪽 SVG/디버그 SVG를 PNG 변환: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과. 2026-05-30 21:26 작업지시자 재실행과 Codex 재실행 모두 `Your wasm pkg is ready to publish at /Users/tsjang/Cloud/Devel/rhwp/pkg.` 확인.

## 승인 상태

- 2026-05-30: 작업지시자가 Stage28 23쪽 시각 검증 완료를 확인하고 커밋을 지시했다.
- 2026-05-30: Stage28 변경을 `ea0b39c8`로 커밋했다.
- 2026-05-30: 작업지시자가 `3-09월_교육_통합_2023.hwp` 4쪽 `문26)` 겹침 문제로 새 스테이지 시작을 지시했다.
- 2026-05-30: 작업지시자가 Stage29 문제 해결 진행을 지시했다.
- 2026-05-30: 4쪽 `문26)` Square wrap 표의 세로 anchor line 보정을 구현하고 회귀 테스트를 추가했다.
- 2026-05-30: Rust/WASM 검증과 4쪽 PDF/RHWP 비교 산출물 생성을 완료했다.
- 2026-05-30: 작업지시자가 12쪽도 PDF와 비교하라고 지시했고, 12쪽 RHWP/PDF 불일치와 오른쪽 단 overflow를 확인했다.
- 2026-05-30: 작업지시자가 13쪽도 비교하라고 지시했고, 13쪽 RHWP/PDF 불일치와 overflow 연속 증상을 확인했다.
- 2026-05-30: 작업지시자가 12쪽/13쪽 문제 해결을 지시했다.
- 2026-05-30: 12쪽/13쪽 미주 단 분배와 TAC Picture 이중 y 진행을 보정하고 비교 산출물을 갱신했다.
- 2026-05-30: Rust 관련 검증, 공식 WASM 검증 명령, SVG export 검증을 통과했다.
- 작업지시자 12쪽/13쪽 시각 승인 대기.
