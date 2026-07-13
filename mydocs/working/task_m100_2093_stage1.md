# Task #2093 Stage 1 - 원본 fixture 2쪽 과분할 분석

## 재현

수정 전 원본 `saved_single_line_spacing_after.hwpx`는 rhwp에서 2쪽이었다.

- 1쪽: pi=0 `FILL`(917.3px) + pi=1 `TAIL LINE WITH SPACING AFTER`
- 2쪽: pi=2 `PAGE2 HEAD`

같은 원본을 HWP 2020 MCP로 변환한 `pdf/task2093/saved_single_line_spacing_after-2020.pdf`는
1쪽이며 세 텍스트를 모두 포함한다.

## 원인

- `FILL`은 10pt 글자와 160% 줄간격인데 저장 `LINE_SEG`가
  `line_height=text_height=68800HU`를 가진다.
- rhwp의 `corrected_line_metrics_for_source`는 작은 저장 줄만 font metrics로
  보정하고, 이처럼 비정상적으로 큰 순수 텍스트 줄은 그대로 신뢰했다.
- 그 결과 첫 줄 하나가 917.3px을 차지해 뒤 문단이 다음 페이지로 밀렸다.
- 줄 높이만 재조판하면 뒤 문단의 저장 `vpos`와 `FILL`의
  `baseline_distance=58480HU`가 남는다. 전자는 다시 새 쪽을 만들고, 후자는 SVG와
  Canvas에서 `FILL`을 페이지 하단에 그린다.

## 구현 방향

원본 fixture와 기준 PDF는 바꾸지 않는다. `SectionDef`와 `ColumnDef`가 함께 있는 HWPX
구역 첫 순수 텍스트 줄의 저장 높이와 text height가 스타일상 가능한 줄 advance의 40배를
모두 넘는 경우만 재조판 metrics로 전환한다. 책갈피는 추가 메타데이터로 허용하되,
표·그림·글상자·필드와 정상 text height를 가진 큰 line box는 보존한다.

## 구현 및 검증 결과

- 재조판 대상 줄은 글꼴 크기와 문단 줄간격으로 `line_height`, `line_spacing`,
  `baseline`을 함께 복원한다.
- HWPX 조판 커서는 손상 줄이 남긴 후속 `vpos`를 순차 흐름으로 접고, 해당 페이지의
  near-top vpos reset을 새 쪽 증거로 오인하지 않는다.
- `target/release-test/rhwp dump-pages ...`: 1쪽에 pi=0~2 모두 배치됐다.
- `cargo test --profile release-test --test issue_2093_saved_single_line_spacing_after --test issue_2093_1192000_real_doc_pin`: 통과했다.
- `issue_2098_margin_boundary_split`(의도된 55000HU 큰 구역 첫 줄의 footer 2쪽)과
  `issue_1692` HWPX 미주 페이지 범위도 통과했다.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests --quiet`: 2200 passed,
  0 failed, 7 ignored으로 통과했다.
- `wasm-pack build --target web --out-dir pkg`: 통과했다.
- 기준 PDF와 visual sweep: SVG 1쪽, PDF 1쪽, 페이지/줄 순서/frame/tail 후보 0건.
  글꼴 fallback 차이로 `visual_accuracy_proxy_percent`는 10.51101이므로 글리프 모양
  일치 수치로 해석하지 않는다.
