# task 1293 stage83 - 0/0/0 미주 마지막 줄 넘김 보정

## 목적

stage82 이후 현재 sweep과 시각 비교 기준으로 남은 문제를 확인했다. WIP와 HEAD의 차이는
판단 근거로 쓰지 않고, 현재 산출물의 `compare_*.png`, `dump-pages`, render tree만 기준으로
`3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`의 후반 미주 흐름을 보정한다.

## 대상

- HWP: `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`
- PDF: `pdf/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.pdf`
- sweep target: `2024-11-practice-above0-between0-below0`

## 판단 기준

- 이번 단계는 WIP와 직전 커밋의 차이로 결론을 내리지 않는다.
- 리베이스 전후 또는 HEAD 대비 변화는 원인 후보를 좁히는 보조 정보로도 채택하지 않는다.
- 실제 판단은 현재 `sweep` 결과, `compare_*.png`, `render_tree_*.json`, `dump-pages`와
  PDF 기준 시각 비교로 한다.
- page count가 맞아도 한컴/PDF와 문항 흐름 또는 하단 tail 위치가 다르면 완료로 보지 않는다.
- PR CI 전체 테스트는 사용자 승인 전에는 수행하지 않는다.

## 현재 관찰

초기 stage83 sweep에서는 대상 샘플의 페이지 수가 맞아도 page19 하단과 page20 첫 줄이
PDF와 달랐다.

- page19 오른쪽 단 끝에서 `pi=960`의 마지막 줄 `점이다.`가 frame 하단에 걸려 있었다.
- PDF 기준으로는 이 마지막 줄이 page20 왼쪽 단 첫 줄로 넘어가야 한다.
- 기존 테스트는 page20의 `pi=961` 문단 텍스트 `이다.`를 tail로 오인할 수 있어 약했다.
  실제로는 `pi=960`의 마지막 텍스트 줄이 먼저 오고, 그 아래에 `pi=961` 수식 문단이
  와야 한다.

## 원인

`구분선 위=0 / 미주 사이=0 / 구분선 아래=0`이고 구분선이 켜진 마지막 단에서는 저장 vpos
보정과 sequential 높이 계산이 서로 어긋나는 구간이 있다. 문단 전체 sequential 높이는
들어가는 것처럼 보이지만, 실제 render tree의 마지막 줄은 frame 아래로 내려갈 수 있다.
한컴/PDF는 이 경우 문단 전체를 다음 쪽으로 밀지 않고 마지막 한 줄만 다음 쪽 첫 줄로
넘긴다.

## 수정

- `src/renderer/typeset.rs`
  - compact endnote separator profile + zero spacing + visible separator + 마지막 단 조건에서
    다중 줄 미주 문단의 마지막 한 줄만 넘기는 split 규칙을 추가했다.
  - head 부분은 현재 frame에 들어가고 tail 한 줄만 하단에 걸리는 경우
    `fmt.line_heights.len() - 1` 지점에서 `PartialParagraph`로 분할한다.
  - `pi=960 lines=0..3`은 page19에 남기고, `pi=960 lines=3..4`는 page20 첫 줄로 넘긴다.

- `tests/issue_1139_inline_picture_duplicate.rs`
  - `issue_1293_2024_zero_endnote_spacing_page19_tail_moves_to_page20`를 추가했다.
  - page19에 `PartialParagraph pi=960 lines=0..3`만 남고 `pi=961`이 남지 않는지 확인한다.
  - page20이 `pi=960 lines=3..4` -> `pi=961` -> `pi=962` -> `pi=976` 순서인지 확인한다.
  - render tree에서 `pi=960` 마지막 줄이 page20 왼쪽 단 상단에 있고, `pi=961` 수식 줄이
    그 아래에서 시작하는지 확인한다.

## 검증

- focused test:
  - `cargo test --test issue_1139_inline_picture_duplicate issue_1293_2024_zero_endnote_spacing_page19_tail_moves_to_page20`
  - 결과: 통과, 1 passed / 76 filtered out

- regression test:
  - `cargo test --test issue_1139_inline_picture_duplicate`
  - 결과: 통과, 77 passed

- build:
  - `cargo build`
  - 결과: 통과

- sweep:
  - 명령:
    `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --target 2022-09 --target 2022-10 --out output/task1293_stage83_after_tail_split_sweep --rhwp-bin target/debug/rhwp`
  - 결과 요약:
    - `2024-11-practice-above0-between0-below0`: SVG/render-tree/PDF `21/21/21`, `flagged=5/21`
    - `2022-09`: SVG/render-tree/PDF `23/23/23`, `flagged=4/23`
    - `2022-10`: SVG/render-tree/PDF `18/18/18`, `flagged=3/18`
  - 대상 샘플의 page19/page20에는 frame overflow, equation overlap, line order overlap,
    question flow drift flag가 없다.

- dump-pages:
  - page19:
    - `PartialParagraph  pi=960  lines=0..3`
    - `FullParagraph[미주]  pi=961` 없음
  - page20:
    - `PartialParagraph  pi=960  lines=3..4`
    - `FullParagraph[미주]  pi=961`
    - `FullParagraph[미주]  pi=962`
    - `FullParagraph[미주]  pi=976`

- 시각 확인:
  - `output/task1293_stage83_after_tail_split_sweep/2024-11-practice-above0-between0-below0/compare/compare_019.png`
    - page19 하단의 `점이다.` 잘림이 사라졌고 PDF처럼 해당 줄이 다음 쪽으로 넘어간다.
  - `output/task1293_stage83_after_tail_split_sweep/2024-11-practice-above0-between0-below0/compare/compare_020.png`
    - page20 왼쪽 단 첫 줄이 `점이다.`로 시작하고, 그 아래 `CP·DQ=...=12이다.` 수식 문단이
      이어진다. PDF와 같은 순서다.

## 남은 판단

대상 샘플의 stage83 문제는 현재 sweep과 시각 기준으로 해결됐다. sweep에 남은 page10~14
후보는 이 단계의 page19/page20 tail 문제와 별개이며, 다음 단계가 필요하면 새 stage에서
현재 sweep과 시각 확인 기준으로 별도 판단한다.
