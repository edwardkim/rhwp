# task 1274 stage20: 2024-09 미주사이20 잔여 overflow 재확인

## 배경

- PR 준비 중 사용자 시각 확인에서 `3-09월_교육_통합_2024-미주사이20.hwp`가 아직 한컴과 다르게 보이는 구간이 추가 확인되었다.
- page 13/24 하단 왼쪽 열에서 `문15)` 풀이 꼬리가 page frame 아래로 내려가며 잘리는 현상이 보인다.
- page 22/24 오른쪽 열의 `문26)` 풀이도 하단에서 overflow 되어 다음 page 상단 흐름과 맞지 않는다.
- 추가로 `3-10월_교육_통합_2022.hwp` page 11/18에서도 오른쪽 열 `문20)` 풀이 꼬리가 page frame 아래로 내려가며, 다음 page 상단 `문22)` 흐름과 맞지 않는 현상이 보인다.
- 같은 파일 page 16/18에서도 하단 `문30)` 꼬리가 page frame 아래로 내려가며, 다음 page로 넘어가야 할 줄이 현재 page 하단에 남는 현상이 보인다.

## 판단 기준

- 두 문제 모두 특정 문항 번호에 대한 예외가 아니라, 미주/분단/쪽 하단에서 남은 높이가 부족할 때 다음 열 또는 다음 쪽으로 넘기는 공통 pagination 판단으로 처리해야 한다.
- 기존 stage에서 완화한 vpos rewind, split paragraph, 빈 host paragraph 처리 로직이 실제 page 하단 여유를 과대평가하지 않는지 다시 확인한다.
- 시각 sweep summary의 `overflow_lines`가 비어 있더라도, 실제 PNG/PDF 비교에서 page frame 아래로 내려간 텍스트가 있으면 미해결로 본다.

## 조사 대상

- `3-09월_교육_통합_2024-미주사이20.hwp`
  - page 13/24: 왼쪽 열 `문15)` 하단 tail overflow
  - page 22/24: 오른쪽 열 `문26)` 하단 tail overflow
- `3-10월_교육_통합_2022.hwp`
  - page 11/18: 오른쪽 열 `문20)` 하단 tail overflow
  - page 12/18: `문22)` 시작 위치와 page 11 overflow 처리 결과 비교
  - page 16/18: `문30)` 하단 tail overflow
  - page 17/18: `문30)` tail의 다음 page 시작 위치 비교

## 검증 계획

- `scripts/task1274_visual_sweep.py --target 2024-09-between20`로 SVG/PDF/compare 산출물을 재생성한다.
- `scripts/task1274_visual_sweep.py --target 2022-10`로 SVG/PDF/compare 산출물을 재생성한다.
- page 13, page 22, page 11, page 16의 rhwp PNG와 한컴 PDF 기준 PNG를 직접 비교한다.
- 반복 구현 중에는 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 중심으로 확인한다.
- 수정 후 `2024-09-between20` visual sweep에서 page frame 하단 overflow가 사라졌는지 확인한다.

## 조사 결과

### `3-09월_교육_통합_2024-미주사이20.hwp`

- page 13/24의 `문15)` 꼬리는 미주 사이가 20mm인 compact endnote 흐름에서 남은 높이에 한 줄만 들어간다고 판단해, 문단을 1줄만 현재 쪽 하단에 split하는 경로가 원인이었다.
- 이 파일은 한 줄만 page frame 하단으로 보내면 다음 줄과의 실제 높이를 확보하지 못해 한컴/PDF와 달리 frame 아래로 밀린다.
- `typeset.rs`에서 기본 미주 사이 간격이 아닌 compact endnote 흐름의 `split_endnote_to_fit == 1` 및 `internal_rewind_split == Some(1)`을 막아, 한 줄짜리 꼬리를 page 하단에 억지로 남기지 않도록 정리했다.
- page 21/24 하단의 `문26)`도 같은 "남은 높이 과대평가" 계열이다. 최신 sweep에서는 `문26)` 제목과 첫 풀이가 page 21 frame 안에 남고, page 22는 이어지는 풀이가 frame 안에서 시작한다.

### `3-10월_교육_통합_2022.hwp`

- page 11/18의 `문20)` 하단 수식은 텍스트가 없는 TAC 수식-only 문단이다. 기존 공통 vpos 보정은 현재 수식 문단의 실제 line advance까지 포함해 하단 frame 여유를 다시 계산하지 않아 수식 꼬리가 page frame 아래로 내려갈 수 있었다.
- `height_cursor.rs`에 textless TAC equation-only 문단을 판별하는 공통 helper를 추가하고, compact endnote 하단에서 현재 수식 문단의 line advance를 기준으로 page frame 안에 남길 수 있는 y 위치를 다시 계산하도록 했다.
- page 16/18의 `문30)`은 제목 문단의 저장 vpos가 page 하단에서 약간 위로 되감기는데, 기존 `injected_between_notes` 분기가 그 결과를 다시 `y_offset`으로 되돌려 첫 본문 줄이 frame 밖으로 밀렸다.
- `height_cursor.rs`에서 page 하단의 미주 제목 되감기(`compact_endnote_title_bottom_backtrack`)를 먼저 인정하고, 해당 경우에는 `injected_between_notes` 되돌림을 적용하지 않도록 해 `문30) 12`와 첫 줄 `함수 g(x)의 한 부정적분은 G(x)라 하자.`가 page 16 하단에 함께 보이도록 했다.

## 검증 결과

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 51개 통과.
  - 추가 확인: `3-10월_교육_통합_2022.hwp` page 11 `문20)` 하단 수식이 frame 안에 남는지 검사.
  - 추가 확인: `3-10월_교육_통합_2022.hwp` page 16 `문30)` 제목과 첫 본문 줄이 frame 안에 함께 남는지 검사.
- `python3 scripts/task1274_visual_sweep.py --target 2022-10`
  - SVG 18쪽, PDF 18쪽.
  - `output/task1274/2022-10/compare/compare_011.png`: page 11 `문20)` 하단 수식이 frame 안에 남는다.
  - `output/task1274/2022-10/compare/compare_016.png`: page 16 `문30) 12`와 첫 본문 줄이 한컴 PDF 기준처럼 page 하단에 같이 보인다.
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
  - SVG 24쪽, PDF 24쪽.
  - `output/task1274/2024-09-between20/compare/compare_013.png`: page 13 `문15)` 꼬리가 frame 아래로 내려가지 않는다.
  - `output/task1274/2024-09-between20/compare/compare_021.png`, `compare_022.png`: page 21 `문26)` 시작과 page 22 이어지는 풀이가 frame 안에서 이어진다.
- WASM 빌드는 작업지시자가 수동으로 수행하는 범위라 이번 stage 검증 명령으로 실행하지 않았다.
