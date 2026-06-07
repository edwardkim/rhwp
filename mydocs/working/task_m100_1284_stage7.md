# task 1284 stage7: page17-18 문28 tail 이월 과다 분석

## 배경

- stage6 커밋 후 `2024-09-between20` focused sweep 결과:
  - `frame=[]`
  - `question=[18, 21]`
  - page22 문28 drift와 page23 q29 tail overflow는 해소됐다.
- 남은 question drift 중 page18은 문29/문30/문23이 모두 PDF보다 70px 이상 낮게 시작한다.

## 현재 후보

- page18:
  - 문29: rhwp y=483.3, PDF y=404.2, drift `+79.1px`
  - 문30: rhwp y=447.3, PDF y=375.4, drift `+71.9px`
  - 문23: rhwp y=969.8, PDF y=891.4, drift `+78.4px`
- page21:
  - 문30: rhwp y=268.6, PDF y=215.0, drift `+53.6px`

## 1차 시각 판단

- `compare_018.png`에서 rhwp는 page18 문29 앞에 남은 문28 풀이 tail이 PDF보다 많다.
- `compare_017.png`에서 rhwp page17 오른쪽 단 문27/문28 시작이 PDF보다 약 33px 낮다.
- 이 때문에 page17에 들어가야 할 문28 풀이 일부가 page18로 이월되고, page18의 문29/문30/문23 전체가 아래로 밀린다.

## 분석 계획

- page17 오른쪽 단의 문27/문28 문단 bbox와 dump item을 확인한다.
- `pi=875..899` 구간의 TAC 수식/빈 문단/그림 여부와 `HeightCursor` vpos 보정을 확인한다.
- page17 문28 시작을 PDF 기준 y≈779.6 근처로 끌어올리면 page18 문29 시작이 함께 PDF 기준 y≈404.2로 맞는지 확인한다.

## 검증 예정

- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20 -- --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `cargo build`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`

## 진행 기록

- stage6 커밋 직후 focused sweep은 `frame=[]`, `question=[18, 21]`였다.
- page17 오른쪽 단 문27/문28은 PDF보다 약 33px 낮았다.
  - `pi=875` 문27: rhwp y=387.3, PDF y=353.9
  - `pi=887` 문28: rhwp y=812.8, PDF y=779.6
- `HeightCursor`는 `pi=875`에서 이미 pagination 공통 gap 위치(`result=348.59`)를 계산했지만,
  layout의 `should_preserve_endnote_title_gap`가 full 20mm gap으로 다시 내리고 있었다.
- `HeightCursor::last_compacted_endnote_title_gap` 플래그를 추가해, 저장 end_y보다 충분히 위로
  compact한 새 미주 제목은 layout gap preserve에서 제외하도록 했다.
- 이 변경 후 page17 문27/문28은 각각 y=348.6, y=774.1로 PDF 기준에 가까워졌다.

## 폐기한 실험

- 큰 미주 사이 boundary에서 직전 문단 `line_spacing`을 pagination 공통 몫으로 줄이는 typeset 실험은 폐기했다.
- targeted test는 통과했지만 focused sweep이 `question=[10, 18, 21]`로 악화되고 page21 overflow 로그가 생겼다.
- 원인: page13 문16처럼 이미 맞던 boundary도 같은 후보로 잡혀, pagination current-height를 전역으로 바꾸면 정상 gap이 깨진다.

## 현재 남은 문제

- focused sweep은 다시 `frame=[]`, `question=[18, 21]` 상태다.
- page18은 문29/문30/문23이 각각 PDF보다 약 72~79px 낮다.
  - 문29 `pi=900`: rhwp y=483.3, PDF y=404.2
  - 문30 `pi=928`: rhwp y=447.3, PDF y=375.4
  - 문23 `pi=935`: rhwp y=969.8, PDF y=891.4
- page21 문30 `pi=1025`는 rhwp y=268.6, PDF y=215.0으로 약 53.6px 낮다.
- 이 두 후보는 render/layout gap preserve만으로는 해결되지 않고, 이전 미주 tail의 pagination 분기/누적 높이 판단을 별도 stage로 좁혀야 한다.
