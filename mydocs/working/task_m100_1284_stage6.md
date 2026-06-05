# task 1284 stage6: 22쪽 문28 drift 분석

## 배경

- stage5에서 page21 오른쪽 단 문25/문26 drift와 q26 tail overflow를 해소했다.
- 커밋 후 남은 `2024-09-between20` sweep 후보 중 page22 문28은 rhwp가 PDF보다 약 75.5px 높게 시작한다.
- page22 문27 시작은 rhwp y=214.3, PDF y=214.5로 이미 거의 일치하므로, 문제는 page22 왼쪽 단의 문27 내부 높이 누적에서 발생한 것으로 보인다.

## 현재 sweep 후보

- 문27:
  - rhwp page22 y=214.3
  - PDF page22 y=214.5
  - drift 거의 없음
- 문28:
  - rhwp page22 y=781.4
  - PDF page22 y=856.9
  - `-75.5px` 높음

## 분석 계획

- page22 render tree의 `pi=1087..1106` bbox를 PDF bbox와 비교한다.
- 문27 제목은 맞으므로, 문27 본문/그림/빈 문단 중 어디서 PDF보다 높이를 덜 쓰는지 찾는다.
- 특히 문27 내부 TAC 그림 또는 빈 Shape host가 page22 왼쪽 단에서 높이 예약을 누락하는지 확인한다.

## 원인

### page22 문28 drift

- `pi=1089`는 문27 본문 뒤의 TAC 그림-only 문단이다.
- 기존 `HeightCursor`는 `vpos_rewind && curr_tac_picture_only`이면 직전 문단의 종류와 무관하게 저장 vpos로 되감았다.
- 이 문단은 직전 `pi=1088`에 실제 설명 텍스트가 있으므로 한컴/PDF처럼 순차 흐름 뒤에 그림 높이를 예약해야 한다.
- 하지만 되감기 때문에 그림이 문27 제목 근처로 당겨지고, 이후 `pi=1090`부터 문28까지 약 75px 높게 배치됐다.

### page23 문29 tail overflow

- 문28 drift를 고치면 page23의 기존 잠복 문제가 드러난다.
- `pi=1159/1160`은 문29 마지막 "정사영의 넓이" tail인데, 기존 pagination은 왼쪽 단 하단에 둔다.
- 실제 render `HeightCursor`에서는 TAC 그림/수식으로 lazy base가 보정되어 `pi=1159`가 y=1090.3, `pi=1160`이 y=1108.3에 그려져 frame 하단 1092.3px을 넘었다.
- PDF/한컴 기준은 `pi=1159/1160`을 오른쪽 단 상단에 넘긴 뒤 `pi=1161`, 문30으로 이어진다.

## 수정

- `src/renderer/height_cursor.rs`
  - TAC 그림-only 문단의 vpos rewind를 직전 문단에 visible text가 없을 때만 허용했다.
  - 문27 본문 뒤 TAC 그림은 순차 흐름으로 배치되어 문28 시작이 PDF 기준으로 내려간다.
- `src/renderer/typeset.rs`
  - 큰 `미주 사이` 다단 흐름에서 제목이 아닌 tail 문단이 실제 render 위치 기준으로 frame 하단을 넘을 위험이 있으면 다음 단으로 advance한다.
  - 직접 vpos 예측이 TAC lazy base를 과소평가하는 경우를 위해, 직전 수식-only 문단 뒤 한 줄 텍스트 tail이 하단 위험 밴드에 들어오면 다음 단으로 넘긴다.
  - 이 보정은 `ep_idx > 0`, 다음 단이 있는 다단, 큰 `미주 사이`, local/internal rewind 없음, visible/equation tail 조건으로 제한했다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - page22 문28 제목 y≈856.5, page23 문29 tail `pi=1159/1160` 오른쪽 단 상단 이월, page23 문30 y≈248을 회귀 테스트로 추가했다.

## 검증

- `cargo fmt --check` 통과.
- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20 -- --nocapture` 통과: 4 passed.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과: 55 passed.
- `cargo build` 통과.
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
  - `frame=[]`
  - `question=[18, 21]`
  - page22/page23의 문28/문30 question drift 후보 제거.
  - `rg -n "LAYOUT_OVERFLOW" output/task1274/2024-09-between20` 결과 없음.

## 남은 후보

- 같은 sweep에서 남은 후보는 기존 residual이다.
  - `red=[11, 17, 18, 20, 21]`
  - `line=[4, 7, 8, 10, 12, 14, 15, 16, 20, 21]`
  - `question=[18, 21]`
- stage6 범위에서는 새로 확인한 page22 문28 drift와 page23 q29 tail overflow를 우선 해소했다.
