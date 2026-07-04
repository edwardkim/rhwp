# Task 1293 Stage 34: 구분선 위 0 / 미주 사이 20 / 구분선 아래 2 partial overflow 분석

## 배경

Stage33에서 `2024-11-practice-no-separator-above20-between20-below20`의 page count와
renderer overflow는 정리했다. 남은 focused sweep 후보는 보이는 구분선이 있는
`2024-11-practice-above0-between20-below2`의 page 14 `pi=671` partial paragraph overflow
2건이다.

이번 단계는 구분선이 보이는 샘플에서 공식 `미주 모양` 값이 렌더 흐름에 어떻게 소비되는지
다시 확인한다.

- `구분선 위`: 0mm
- `미주 사이`: 20mm
- `구분선 아래`: 2mm

## 대상

- target: `2024-11-practice-above0-between20-below2`
- sample: `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp`
- 남은 overflow:
  - `LAYOUT_OVERFLOW_DRAW: section=0 pi=671 line=2 y=1140.5 col_bottom=1092.3 overflow=48.2px`
  - `LAYOUT_OVERFLOW: page=14, sec=0, col=0, para=671, type=PartialParagraph, first=false, y=1140.5, bottom=1092.3, overflow=48.2px`

## 분석 계획

1. `dump-pages`와 render tree로 page 13~14의 `pi=671` 분할 위치를 확인한다.
2. `dump`로 `pi=671`의 LINE_SEG vpos/line height와 문단 텍스트를 확인한다.
3. Stage33에서 정리한 no-separator 흐름과 달리, 보이는 구분선에서는 `구분선 아래`와
   `미주 사이`가 같은 단 하단에서 중복 소비되는지 분리한다.
4. 수정은 공식 의미에 맞게 `미주 사이`는 번호 사이 gap, `구분선 아래`는 separator 아래 첫 미주
   내용 전 gap으로만 쓰이도록 좁힌다.

## 검증 계획

- `cargo build --bin rhwp`
- `cargo test --lib compact_endnote -- --nocapture`
- focused sweep:
  - `2024-11-practice-above0-between20-below2`
  - `2024-11-practice-no-separator-above20-between20-below20`
  - `2024-11-practice-above20-between0-below20`

## 상태

- 분석 및 보정 완료.

## 분석 결과

- `dump-pages -p 14` 기준 `pi=671`은 원래 page 15 좌측 단에 `lines=0..3`, 우측 단에
  `lines=3..4`로 분할되었다.
- render tree 기준 좌측 단의 3번째 줄은 `y=1112.9`, `h=27.6`으로 body bottom
  `1092.3`을 넘어 `LAYOUT_OVERFLOW`가 발생했다.
- 같은 문단은 내부 LINE_SEG vpos가 `line1`에서 되감기는 구조다.
- 일반 `current_height + line_advance` 기준으로는 3줄이 들어가는 것처럼 보이지만,
  renderer는 보이는 구분선 + 큰 `미주 사이` 경계의 저장 vpos/gap을 반영해 마지막 포함 줄을
  pagination보다 낮게 그린다.

## 폐기한 후보

- `미주 사이` gap을 vpos offset뿐 아니라 `current_height`에도 전역 가산하는 후보는 폐기했다.
- 이 후보는 `pi=671` 위치는 바꾸지만 전체 page count를 `22/22`에서 `24/22`로 늘려 한컴 기준과
  맞지 않았다.

## 적용한 보정

- 보이는 구분선 + 큰 `미주 사이` + 내부 vpos rewind + 단 하단 90% 이후의 미주 partial split에만
  적용한다.
- `split_endnote_to_fit` 후보가 2줄 이상이면 마지막 포함 줄 하나를 다음 단으로 보낸다.
- 결과적으로 `pi=671`은 좌측 단 `lines=0..2`, 우측 단 `lines=2..4`로 분할된다.

## 검증 결과

- `cargo build --bin rhwp`: 통과
- `cargo test --lib compact_endnote -- --nocapture`: 28개 통과
- focused sweep:
  - 실행 결과: `output/task1293_stage34_focused/summary.json`
  - `2024-11-practice-above0-between20-below2`: `22/22/22`, `overflow_lines=0`
  - `2024-11-practice-no-separator-above20-between20-below20`: `23/23/23`, `overflow_lines=0`
  - `2024-11-practice-above20-between0-below20`: `21/21/21`, `overflow_lines=0`
  - 세 target 모두 `frame_overflow_pages`, `question_title_text_overlap_pages`,
    `line_order_overlap_pages`, `equation_text_overlap_pages`가 비어 있다.

## 남은 판단

- Stage24 전체 sweep에는 아직 다른 target의 `LAYOUT_OVERFLOW`가 남아 있었다.
- 이번 단계는 focused 세 target의 회귀를 정리한 것이므로, 다음 단계에서 전체 sweep을 다시 돌려
  남은 미주 target을 재선정한다.
