# Task M100 #1139 Stage 27

## 목적

Stage26 커밋 이후 남은 `3-09월_교육_통합_2022.hwp` 22쪽 렌더링 차이를 별도 스테이지로 추적한다.

## 시작 기준

- 기준 커밋: `f05ebe79ad213e1c5fa0149c45f3d0210af7d247` (`task 1139: Stage26 미주 분배와 속성 회귀 보정`)
- Stage26 변경은 커밋 완료했다.
- Stage27 문서는 Stage26 커밋 이후 새 변경으로 생성한다.
- Stage27 소스 수정은 작업지시자 승인 후 진행한다.

## 보고된 문제

- 작업지시자 시각 검증에서 22쪽 렌더링 자체가 한컴오피스와 다르다고 보고되었다.
- 첨부 비교 기준으로 22쪽은 단순 overflow가 아니라 상단 표/그림/미주 흐름의 전체 배치가 한컴오피스와 다르다.
- 한컴오피스 기준 22쪽은 상단 제목 표, 좌측 큰 그림, 우측 문30)/그림, 미주 흐름이 한 페이지 안에서 균형 있게 배치된다.
- RHWP 기준 22쪽은 페이지 내 본문/그림/미주 흐름 자체가 다른 위치와 크기로 배치되어 후속 페이지 흐름에도 영향을 줄 수 있다.

## 유지 판단

- Stage26의 10/12/13/17/19/20쪽 미주 분배 보정은 완료 범위로 유지한다.
- Stage15에서 남긴 9쪽 격자 원점 판단은 유지한다.
  - `--grid-origin=9mm,24mm`와 `--grid-origin=auto`의 SVG pattern 차이는 약 0.002px 수준이다.
  - 남은 미세 차이는 격자 원점 문제가 아니라 문단 내부 수식/텍스트 렌더 메트릭, 수식 TAC baseline/bbox 스케일, 문단 세로 정렬(`attr1` bit 20~21, 글꼴 기준) 처리 쪽으로 좁힌다.
- 22쪽은 위 후보와 별도로 표/그림/TAC/미주 흐름의 페이지네이션 후보를 함께 점검한다.

## 진행 계획

1. 22쪽 RHWP SVG와 한컴/PDF 기준 이미지를 다시 추출해 같은 배율의 비교 이미지를 만든다.
2. 21/22/23쪽 `dump-pages`를 다시 추출해 22쪽으로 들어온 문단, 표, 그림, 수식, 미주 흐름을 고정한다.
3. 22쪽 상단 제목 표와 좌측 큰 그림, 우측 문30)/그림의 control anchor, TAC bbox, lineSeg `vertical_pos`를 비교한다.
4. 문단 내부 수식/텍스트 렌더 메트릭과 수식 TAC baseline/bbox 스케일이 페이지 높이 계산에 미치는 영향을 분리한다.
5. 문단 세로 정렬(`attr1` bit 20~21, 글꼴 기준) 처리가 한컴오피스와 다른 문단이 있는지 확인한다.
6. 원인이 분리되면 22쪽 배치를 고정하는 회귀 테스트를 추가한다.
7. Rust/WASM 수정 후 `wasm-pack build --target web --out-dir pkg`를 실행하고, 22쪽 SVG/PDF 시각 비교 산출물을 생성해 작업지시자 확인을 받는다.

## 분석 결과

- 22쪽 렌더링 차이의 1차 원인은 22쪽 자체의 헤더/표 좌표가 아니라 20쪽에서 21쪽, 21쪽에서 22쪽으로 이어지는 미주 페이지네이션 경계였다.
- 초기 RHWP 흐름은 21쪽 하단에 `문29)` 제목(`pi=1129`)과 `[출제의도]`(`pi=1130`)가 남지 못하고 22쪽 첫머리로 밀렸다.
- 한컴/PDF 기준은 21쪽 하단에 `pi=1129`, `pi=1130`이 남고, 22쪽은 큰 구 그림이 있는 `pi=1131`부터 시작한다.
- 20쪽에서는 제목 직후 다줄 꼬리 문단(`pi=1088`)을 강제로 다음 쪽으로 나누던 기존 보정이 오히려 22쪽 전체 흐름을 늦췄다.
- 22쪽 오른쪽 단의 `[그림 1]` 누락은 그림 control 누락이 아니었다. 해당 그래프는 미주 가상 문단 `pi=1169` 안의 `Table` control인데, 미주 렌더 경로가 `Shape/Picture`만 `PageItem`으로 내보내고 `Table`을 누락하고 있었다.
- 후속 시각 검증에서 13쪽 `문20)` 변화표가 중복 출력되었다.
  - Stage27의 미주 `Table` 보정이 모든 미주 table control을 별도 `PageItem::Table`로 추가하면서, `pi=739`처럼 host `FullParagraph` 안에서 이미 렌더되는 TAC table까지 다시 배치했다.
  - 22쪽 `pi=1169`는 원문 text가 비어 있는 table-only 문단이라 별도 `PageItem::Table`이 필요하지만, 13쪽 `pi=739`는 host paragraph 경로 렌더를 유지해야 한다.

## 구현 내용

- `src/renderer/typeset.rs`
  - 기본 미주 사이 7mm 문서에서 후반 `문29)`/`문30)`은 마지막 단 하단에서도 제목과 풀이 일부가 같은 쪽에 남도록 fit/advance 예외를 좁게 적용했다.
  - 제목 직후 다줄 문단의 마지막 줄을 무조건 다음 쪽으로 보내던 `split_title_tail_near_column_bottom` 흐름을 제거했다.
  - 미주 table-only 문단은 빈 `FullParagraph`만 배치하지 않고 `PageItem::Table`로 렌더링되도록 보정했다.
  - 단, 별도 `PageItem::Table` 추가는 `text.is_empty()`인 table-only 미주 문단으로 제한해, 공백 host 문단 안에서 이미 렌더되는 TAC table 중복을 막았다.
- `src/renderer/height_cursor.rs`
  - 미주 제목 직후 다줄 꼬리 문단이 단 하단에서 작게 넘칠 때만 제한적 backtrack을 허용했다.
  - backtrack은 최대 16px로 제한해 겹침 회귀를 피한다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - 21쪽에 `pi=1129/1130`이 남고, 22쪽이 `pi=1131` 그림부터 시작하는 회귀 테스트를 추가했다.
  - 22쪽 문30 그래프 표 `pi=1169 ci=0`가 dump와 render tree에 실제 존재해야 한다는 회귀 테스트를 추가했다.
  - 13쪽 문20 변화표 `pi=739 ci=0`가 한 번만 렌더되어야 한다는 중복 회귀 테스트를 추가했다.

## 해결된 부분

- 20쪽 `pi=1088` 하단 overflow 로그가 사라졌다.
- 21쪽 하단에 한컴/PDF 기준처럼 `문29)` 제목과 `[출제의도]`가 남는다.
- 22쪽은 큰 구 그림 `pi=1131`부터 시작한다.
- 22쪽 오른쪽 단의 `[그림 1]` 그래프 표 `pi=1169`가 렌더링된다.
- 13쪽 `문20)` 변화표 `pi=739`의 중복 출력이 제거되었다.
- 20/21/22쪽 SVG 재추출에서 `LAYOUT_OVERFLOW` 로그가 발생하지 않았다.

## 남은 판단

- 22쪽의 주요 페이지 경계와 누락 그래프는 보정했다.
- 수식 glyph/baseline, 일부 텍스트 굵기/메트릭의 미세 차이는 Stage15에서 남긴 전역 후보와 같은 범위로 남긴다.
- 최종 커밋은 작업지시자의 시각 승인 후 진행한다.

## 검증 기록

- `cargo fmt --all`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 통과, 18개 테스트
- `cargo test renderer::height_cursor::tests::compact_endnote -- --nocapture`: 통과, 대상 10개 테스트
- `cargo build`: 통과
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 19 -o output/task1139_stage27_final_svg3/page20`: 통과, overflow 로그 없음
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 20 -o output/task1139_stage27_final_svg3/page21`: 통과, overflow 로그 없음
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 21 -o output/task1139_stage27_final_svg3/page22`: 통과, overflow 로그 없음
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 20`: 21쪽 `pi=1129`, `pi=1130` 확인
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 21`: 22쪽 `Shape pi=1131`, `Table pi=1169 ci=0` 확인
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 12 -o output/task1139_stage27_page13_regression/fixed_page13`: 통과, overflow 로그 없음
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 12`: 13쪽 `pi=739` host paragraph 유지, 별도 `Table pi=739 ci=0` 제거 확인
- `wasm-pack build --target web --out-dir pkg`: 통과
- `cargo fmt --all --check`: 통과
- `git diff --check`: 통과

## 산출물

- 21쪽 비교: `output/task1139_stage27_final_svg3/compare_page21_pdf_rhwp.png`
- 22쪽 비교: `output/task1139_stage27_final_svg3/compare_page22_pdf_rhwp.png`
- 20/21/22쪽 SVG와 dump: `output/task1139_stage27_final_svg3/`
- 13쪽 중복 회귀 확인/수정 산출물: `output/task1139_stage27_page13_regression/`

## 승인 상태

- 2026-05-30: 작업지시자가 Stage26 커밋 이후 Stage27 문서 생성을 지시했다.
- 2026-05-30: Stage27 문서를 생성했다.
- 2026-05-30: 작업지시자가 Stage27 문제 분석 후 해결 시작을 지시했다.
- 2026-05-30: 22쪽 미주 페이지 경계와 문30 그래프 표 누락을 보정하고 검증 산출물을 생성했다.
- 2026-05-30: 작업지시자가 13쪽 마지막 문제 변화표 중복 출력을 보고했고, table-only 미주 보정 범위를 좁혀 중복을 제거했다.
- 커밋 전 작업지시자 시각 승인 대기.
