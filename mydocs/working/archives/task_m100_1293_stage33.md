# Task 1293 Stage 33: 구분선 없음 TAC 그림 묶음 단 흐름 보정

## 배경

Stage24는 전체 sweep 기준점이었다. 모든 target의 SVG/PDF/render tree page count는 1:1이었지만,
renderer `LAYOUT_OVERFLOW`가 남아 있어 미주 기능 완료로 볼 수 없었다.

Stage31/32에서 `2024-11-practice-no-separator-above20-between20-below20`의 잔여 문제를 다시
분리했다.

- `pi=593` 계열은 실제 그림 bbox와 후행 title tail 판정이 분리된 문제였고, Stage31에서
  `Shape` overflow 오탐은 줄었다.
- `pi=613`은 실제 TAC 그림 bbox가 frame 밖으로 나가는 문제다.
- 하지만 `pi=613`만 다음 단으로 넘기면 page count가 `24/23`으로 증가한다.
- compare PNG 기준으로 p13/p14의 단별 내용 순서가 PDF와 크게 달라, 개별 overflow가 아니라
  no-separator 미주 block의 단 흐름 drift로 봐야 한다.

## 공식 의미 기준

한컴 도움말의 미주 모양 설명은 세 여백을 분리한다.

- `구분선 위`: 본문과 미주 구분선 사이의 간격
- `구분선 아래`: 미주 구분선과 미주 내용 사이의 간격
- `미주 사이`: 앞 번호 미주 내용과 다음 번호 미주 내용 사이의 간격

`구분선 넣기`가 꺼진 샘플에서도 HWP 원본에는 `구분선 위/아래/미주 사이` 값이 남아 있고,
현재 rhwp도 blank `EndnoteSeparator` item으로 위/아래 여백을 소비한다. 따라서 이번 단계는
숫자 보정이 아니라, 구분선이 보이지 않는 큰 미주 block에서 이후 미주 문단의 vpos/그림 bbox를
어떻게 단 흐름에 반영할지 확인한다.

## 대상

- target: `2024-11-practice-no-separator-above20-between20-below20`
- sample: `samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp`
- 비교 페이지:
  - p13: 우측 단 말미 `pi=593` 뒤 `문20`
  - p14: 좌측 단 `문21` TAC 그림 반복 `pi=607/609/611/613`

## 수정 후보

1. `pi=613` 개별 tail 강제 넘김은 폐기한다.
   - Stage31에서 page count가 `24/23`으로 증가했다.
2. no-separator + 큰 `미주 사이` + 큰 `구분선 위/아래` block에서 TAC picture-only 반복 묶음이
   단 하단에 실제 bbox overflow를 만들면, 다음 미주 문단 하나가 아니라 해당 그림 반복 묶음의
   시작 조건을 보수적으로 본다.
3. 보이는 구분선 target과 `above20-between0-below20` target은 회귀가 없어야 한다.

## 구현 결과

### 폐기한 후보

- `구분선 없음 + 큰 separator block`에서 모든 새 미주 제목 stale vpos를 65% 단 높이까지 cap하면
  p14/p15 흐름은 PDF에 가까워졌지만 전체 page count가 `22/23`으로 줄었다.
- 따라서 전역 threshold 보정은 폐기했다.

### 적용한 보정

1. no-separator 큰 미주 block에서 현재 단에 TAC picture-only 미주 묶음이 이미 있는 경우에만
   새 미주 제목의 stale forward vpos cap을 65% 단 높이까지 허용했다.
   - p14 우측 단은 `used=582.4px`로 여유가 있는데 `문22` 시작 `pi=632`를 raw vpos span
     약 493px로 보아 다음 페이지로 넘기고 있었다.
   - TAC 그림 묶음 뒤의 저장 vpos는 그림 반복 재배치 흔적이므로, 공식 `미주 사이` gap(`20mm`)과
     제목 높이를 기준으로 cap하면 PDF처럼 p14 우측 단 하단에 `문22`가 들어온다.
2. 같은 no-separator 큰 미주 block에서 단 마지막 쪽 일반 미주 tail도 저장 vpos 기준 y가 frame 밖을
   가리키면 typeset에서 먼저 다음 단/쪽으로 넘기도록 했다.
   - p19 `pi=880/881`, p22 `pi=1010~1012`는 순차 current height로는 fit처럼 보였지만
     render `HeightCursor`는 저장 vpos 기준으로 frame 밖에 배치했다.
   - 새 미주 제목 전용이던 vpos-outside 판정을 일반 tail에도 적용해 pagination/render 판단을 맞췄다.

## 검증 결과

- `cargo build --bin rhwp`: 통과
- `cargo test --lib compact_endnote -- --nocapture`: 28개 통과
- focused sweep:
  - 실행 결과: `output/task1293_stage33_focused_tail_guard/summary.json`
  - `2024-11-practice-no-separator-above20-between20-below20`: `23/23/23`, `overflow_lines=0`
  - `2024-11-practice-above20-between0-below20`: `21/21/21`, `overflow_lines=0`
  - `2024-11-practice-above0-between20-below2`: `22/22/22`, `overflow_lines=2`

## 남은 문제

- 이번 단계는 `구분선 없음 + 구분선위20/미주사이20/구분선아래20`의 p13~p15 단 흐름과
  후반 tail overflow를 줄이는 데 초점을 뒀다.
- `2024-11-practice-above0-between20-below2`에는 page 14 `pi=671` partial paragraph overflow
  2건이 남아 있어 task 1293 goal 완료로 볼 수 없다.
- 다음 단계에서는 visible separator가 있는 `구분선위0/미주사이20/구분선아래2` target의
  partial paragraph split과 renderer y를 별도 분석한다.

## 검증 계획

- `cargo fmt --all -- --check`
- `cargo build --bin rhwp`
- `cargo test --lib compact_endnote -- --nocapture`
- focused sweep:
  - `2024-11-practice-no-separator-above20-between20-below20`
  - `2024-11-practice-above0-between20-below2`
  - `2024-11-practice-above20-between0-below20`
- 필요 시 compare PNG p13/p14를 직접 확인한다.
