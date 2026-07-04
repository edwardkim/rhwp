# Task 1293 Stage 39: Stage38 이후 잔여 LAYOUT_OVERFLOW 재분류

## 배경

Stage38에서 `2024-11-practice-above20-between7-below2`의 `pi=882` TAC 그래프가
PDF/Hancom처럼 19쪽 좌상단으로 이동했다. frame overflow 후보는 사라졌지만,
focused sweep의 renderer `LAYOUT_OVERFLOW` 로그는 아직 남아 있다.

## 목적

Stage38 이후 남은 `LAYOUT_OVERFLOW` 로그가 실제 하단 bleed인지, render tree/frame
비교에서는 허용 가능한 오탐인지 재분류하고, 실제 문제인 경우 공통 미주 흐름 조건으로
수정한다.

## 우선 분석 대상

`output/task1293_stage38_focused_final` 기준:

| target | overflow_lines | 대표 위치 |
|---|---:|---|
| `2024-11-practice-above20-between7-below2` | 8 | page 18 `pi=922~925`, page 19 `pi=975` |
| `2024-11-practice-shape987` | 9 | page 11 `pi=571`, page 16 `pi=818~820` |
| `2024-11-practice-above0-between0-below0` | 14 | page 9/10/12/13/16/17 다수 |

이번 stage에서는 Stage38의 직접 후속인 `above20-between7-below2`를 먼저 본다.

## 확인 계획

1. `compare_019.png`, `annotated_019.png`, `render_tree_019.json`에서 `pi=922~925`의 실제 bbox를 확인한다.
2. `compare_020.png`, `annotated_020.png`, `render_tree_020.json`에서 `pi=975`를 확인한다.
3. `dump-pages -p 18`, `dump-pages -p 19`로 pagination 배치와 renderer overflow 로그를 대조한다.
4. 실제 frame 밖 bleed이면 해당 TAC/문단 tail 묶음을 다음 단/쪽으로 보내고, 오탐이면 overflow 판정 기준을 정정한다.

## 현재 상태

## 분석 결과

Stage38 결과에서 `2024-11-practice-above20-between7-below2`의 page 19를 확인하니,
`pi=922~925` 자체가 단순 render tree/frame 오탐은 아니었다. page 19의 우측 단 하단에
문27 tail이 과하게 남아 있었고, 그 앞 원인은 page 19 좌측 단에서 문25 제목(`pi=903`)을
다음 단으로 밀어낸 것이었다.

`dump-pages -p 18` 비교:

- Stage38 이후: page 19 좌측 단은 문24(`pi=899~902`)에서 끝나고, 문25(`pi=903`)가 우측 단
  맨 위로 이동했다.
- 한컴/PDF와 stage39 보정 후: page 19 좌측 단이 문25 제목과 짧은 본문(`pi=903~906`)까지
  포함하고, 우측 단은 문25 후속(`pi=907`)부터 시작한다.

기존 로직은 기본 `미주 사이` + 보이는 구분선 조합에서도 `large head` 계열 조건이 새 문항 제목을
너무 빨리 다음 단으로 보냈다. 이 경우 문항 제목 한 줄(`en_fit <= 24px`)이 현재 단에 충분히
들어가고, 현재 단이 하단에 몰린 상태가 아니면 한컴처럼 제목 tail을 현재 단에 남겨야 한다.

## 수정

- `src/renderer/typeset.rs`
  - `allow_default_question_title_tail`에 보이는 구분선 + 기본 미주 사이 + 다음 단이 있는 경우의
    새 문항 제목 유지 조건을 추가했다.
  - 조건은 제목 한 줄이 현재 단에 들어가고(`st.current_height + en_fit <= available - 40px`),
    현재 단 높이가 전체 단 높이의 85% 미만일 때만 적용한다. 하단에 실제로 몰린 제목은 기존
    advance 로직을 유지한다.

## 검증

- 단일 대상 sweep:
  - 명령: `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between7-below2 --out output/task1293_stage39_target1 --rhwp-bin target/debug/rhwp`
  - page count: SVG/PDF/render tree `21/21/21`
  - renderer `LAYOUT_OVERFLOW`: `8 -> 0`
  - `frame_overflow_pages`, `question_title_text_overlap_pages`, `line_order_overlap_pages`,
    `equation_text_overlap_pages`: 모두 비어 있음
  - `compare_019.png`, `compare_020.png`에서 page 19 문25/문26/문27 흐름이 PDF 기준에 맞게
    재배치됨을 확인했다.

- focused sweep:
  - 명령:
    `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between7-below2 --target 2024-11-practice-above20-between0-below20 --target 2024-11-practice-shape987 --target 2024-11-practice-above0-between0-below0 --out output/task1293_stage39_focused --rhwp-bin target/debug/rhwp`

| target | page count | Stage38 overflow | Stage39 overflow | 핵심 후보 |
|---|---:|---:|---:|---|
| `2024-11-practice-above20-between7-below2` | 21/21/21 | 8 | 0 | frame/title/order/equation 후보 없음 |
| `2024-11-practice-above20-between0-below20` | 21/21/21 | 0 | 0 | frame/title/order/equation 후보 없음 |
| `2024-11-practice-shape987` | 21/21/21 | 9 | 9 | frame/title/order/equation 후보 없음 |
| `2024-11-practice-above0-between0-below0` | 21/21/21 | 14 | 14 | frame/title/order/equation 후보 없음 |

- `cargo test --lib compact_endnote -- --nocapture`: 28 passed
- `cargo fmt --all -- --check`: passed
- `git diff --check`: passed

## 남은 작업

Stage39는 Stage38의 직접 후속 target만 정리했다. 다음 stage에서는 focused sweep에 여전히 남은
다음 renderer overflow를 계속 분석한다.

- `2024-11-practice-shape987`
  - page 11 `pi=571`
  - page 16 `pi=818~820`
- `2024-11-practice-above0-between0-below0`
  - page 9 `pi=510`
  - page 10 `pi=537~538`
  - page 12 `pi=616`
  - page 13 `pi=691`, `pi=693`
  - page 16 `pi=853`
  - page 17 `pi=914`
