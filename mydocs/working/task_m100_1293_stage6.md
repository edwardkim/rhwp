# Task 1293 Stage 6: 미주 vpos 되감기 문단 분할 정합

## 목적

Stage5-A에서 `구분선 위=0` 적용 경로는 정정했다. 이제 남은 문제는 한컴 공식 미주 모양 값이
읽히더라도, 미주 내용 paragraph의 LINE_SEG `vertical_pos` 되감기와 단 이월 판단이 맞지 않아
하단 overwrap/flow drift가 남는 현상이다.

## 핵심 관찰

`dump-pages` 기준 여러 샘플에서 미주 가상 문단의 vpos 범위가 역전된다.

- `2024-09-between20` 12쪽:
  - `pi=651 vpos=329160..315548`
  - `pi=662 vpos=389495..342597`
- `2024-09-below20-above20` 10쪽:
  - `pi=580 lines=0..1 vpos=247575..235638`
- `2024-09-below20-above20` 17쪽:
  - `pi=894 lines=0..3 vpos=787851..772089`
  - `pi=922 lines=0..4 vpos=837560..791903`

이는 단순한 구분선 여백 문제가 아니라, 저장 LINE_SEG가 단/페이지 이월을 암시하는데 typeset과
layout이 이를 순차 높이 또는 잘못된 split으로 처리하는 문제다.

## 공식 모델과 연결되는 판단

- `구분선 위/아래`는 미주 블록 시작부 여백이다.
- `미주 사이`는 새 번호 미주 경계의 gap이다.
- `vertical_pos` 되감기는 여백 설정값이 아니라 paragraph 내부 저장 좌표 흐름이다.

따라서 해결 방향은 `미주 사이` 숫자를 더 키워서 밀어내는 것이 아니라, 되감기 LINE_SEG 문단을
한컴처럼 단/페이지 경계에서 분할하거나 다음 단으로 보내는 것이다.

## 분석 계획

1. 되감기 문단의 line index, line height, line spacing을 `RHWP_TYPESET_DRIFT_LINES=1`로 확인한다.
2. 현재 `internal_rewind_split` 조건이 왜 해당 페이지에서 너무 늦거나 너무 이르게 작동하는지 확인한다.
3. `PartialParagraph`의 dump vpos 표시가 역전될 때 실제 render bbox가 frame 밖으로 나가는지 대조한다.
4. `미주 사이` 값과 무관한 되감기 분할 보정만 구현한다.
5. focused test와 visual sweep으로 `구분선 위=0`, `구분선아래20`, `미주사이20`, 조합 샘플을 함께 확인한다.

## 구현 내용

- `dump-pages`의 `PartialParagraph` vpos 출력 범위를 `start_line..end_line`의 exclusive 의미에 맞게
  정정했다. 이전 출력은 `end_line`을 inclusive처럼 읽어 다음 줄 vpos까지 섞었기 때문에 실제보다
  더 많은 역전처럼 보이는 경우가 있었다.
- 같은 출력에 `vpos-rewind@lineN` 마커를 추가했다. `vpos-reset@lineN`만으로는 0 리셋과 일반
  되감기를 구분하기 어려워, 남은 미주 흐름 문제를 분석하기 힘들었다.
- `typeset`의 문단 분할 후보에서 LINE_SEG 내부 되감기 후보를 fit 후보보다 우선하되, 첫 줄 직후
  되감기(`split=1`)는 기존 fit 후보가 있으면 fit 후보를 유지했다.
  - 이유: `2024-09-below20-above20` 17쪽 `pi=894`는 한컴 기준으로 line2에서 갈라져야 한다.
  - 반대로 `2023-09` 19쪽 `pi=935`는 기존 PDF 기준대로 `0..2 / 2..3` 분배가 유지되어야 한다.

## 추가 샘플 반영

작업 중 다음 3종 샘플이 추가되어 검증 세트에 포함했다.

- `samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp`
- `samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwpx`
- `pdf/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.pdf`

`dump-note-shape` 기준 HWP와 HWPX의 raw 슬롯은 다르지만 UI 의미값은 동일하게 정규화된다.

- HWP: raw `separatorMarginBottom=8.999mm`, raw `noteSpacing=6.999mm`, raw `rawUnknown=7.997mm`
- HWPX: raw `separatorMarginTop=8.999mm`, raw `noteSpacing=6.999mm`, raw `rawUnknown=7.997mm`
- UI 의미값: `구분선 위 8.999mm / 구분선 아래 6.999mm / 미주 사이 7.997mm`

`scripts/task1274_visual_sweep.py`에 `2024-11-practice-shape987` target을 추가했다.

## 검증 결과

- `cargo fmt --all -- --check`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20 --out output/task1293_stage6_between20 --rhwp-bin target/debug/rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-below20-above20 --out output/task1293_stage6_abovebelow20 --rhwp-bin target/debug/rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-shape987 --out output/task1293_stage6_2024_11_shape987 --rhwp-bin target/debug/rhwp`
- `python3 -m py_compile scripts/task1274_visual_sweep.py`

확인된 결과:

- `issue_1139_inline_picture_duplicate`: 52 passed
- `2024-09-below20-above20`: SVG/PDF/render tree 23/23/23
  - Stage4 대비 `frame_overflow_pages`에서 17쪽이 빠졌다.
  - 17쪽 `pi=894`가 `lines=0..2`, `lines=2..5`로 갈라져 `vpos-rewind@line2` 경계와 맞는다.
  - 남은 후보: frame `[9, 18, 19]`, red `[9, 10, 13, 18, 19, 22]`, line `[4, 7, 10, 11, 15, 18, 21, 22]`
- `2024-09-between20`: SVG/PDF/render tree 24/24/24
  - Stage6 변경으로 새 회귀는 확인되지 않았지만, 기존 12쪽 frame overflow와 후반 line/red drift는 남아 있다.
- `2024-11-practice-shape987`: SVG/render tree 22쪽, PDF 21쪽
  - note shape 정규화는 맞지만, 한컴 PDF보다 rhwp가 한 페이지 늦어지는 흐름 문제가 남는다.
  - `flagged=20/21`, frame `[11, 13, 17]`, red `[10..21]`, line `[9..21]`
  - 이 샘플은 다음 스테이지의 페이지 수/미주 흐름 정합 기준으로 사용한다.

## 다음 스테이지 후보

- `2024-11-practice-shape987`의 22쪽 렌더를 21쪽 PDF와 맞추는 흐름 압축/분할 원인을 분석한다.
- `2024-09-between20`의 12쪽 frame overflow와 후반 red/line drift를 같은 공식 미주 모양 모델에서
  해결 가능한지 확인한다.
