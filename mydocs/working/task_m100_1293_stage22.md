# Task 1293 Stage 22: compact 미주 partial split 렌더 y 정합

## 목적

Stage21에서 `betweenNotes=0` 새 문항 제목의 큰 vpos gap은 줄였지만, target sweep에는
여전히 renderer 내부 overflow가 남아 있다. 이번 단계에서는 문단을 단순히 다음 단/쪽으로
미는 threshold 조절을 피하고, partial split 후보를 실제 renderer line y와 맞추는 방향으로
수정한다.

## 기준 산출물

- `output/task1293_stage21_title_gap_final_sweep/summary.json`
- `output/task1293_stage21_title_gap_final_sweep/2024-11-practice-above20-between0-below20/compare/compare_015.png`
- `output/task1293_stage21_title_gap_final_sweep/2024-11-practice-above20-between0-below20/compare/compare_018.png`
- `output/task1293_stage21_title_gap_final_sweep/2024-11-practice-above20-between0-below20/compare/compare_020.png`

## 잔여 문제

- p15(display) `pi=753`, partial paragraph `lines=0..5` 중 line 1~4가 frame 아래로 overflow한다.
- p18(display) `pi=875/876`, 최대 `40.5px` overflow.
- p20(display) `pi=966/967`, 최대 `46.2px` overflow.

## Stage21에서 폐기한 방향

- `pi=753` 전체를 다음 쪽으로 넘기면 p15 overflow는 사라지지만, p16에서 문24/문25 묶음이
  다시 overflow되어 한컴/PDF 흐름과 멀어진다.
- 0mm 새 미주 제목 threshold를 낮추면 더 앞쪽 문단이 p16으로 밀려 새 overflow가 생긴다.
- 따라서 다음 수정은 advance threshold가 아니라 partial split이 실제 renderer y를 모르는 문제를
  직접 다룬다.

## 구현 방향

1. `typeset.rs`에서 compact endnote partial split 후보를 만들 때, renderer와 동일한
   `HeightCursor` 기준 다음 문단 시작 y를 추정할 수 있는지 확인한다.
2. 추정이 불안정하면 우선 `render_tree`/`LAYOUT_OVERFLOW`의 재현 케이스를 기준으로,
   partial paragraph line count가 frame 안에 들어가는지 사전 검증하는 보조 함수를 둔다.
3. p15 `pi=753`은 첫 줄 시작 y가 frame bottom 근방이므로, 단순 line count fit이 아니라
   “현재 단에 남길 수 없는 partial head”로 판정되어야 한다. 단, 이때 p16 연쇄 overflow가
   생기지 않도록 내부 rewind 문단의 실제 formatter 높이 누적도 함께 점검한다.

## 구현 결과

- `HeightCursor`를 endnote typeset fit 직전에 직접 호출하는 방향은 효과가 없었다.
  renderer의 `stale_forward` 보정은 현재 y를 바꾸지 않고, 문제는 이전 미주 문단들의
  누적 높이와 저장된 internal rewind split이 서로 다른 기준으로 쓰이는 데 있었다.
- p15(display) `pi=753`은 단 하단에서 formatter fit split이 `Some(4)`로 잡히지만,
  저장 `LINE_SEG`의 internal rewind split이 우선되어 `lines=0..5`가 남았다.
- 단 하단(`current_height > available * 0.90`)에서 internal rewind가 있고 formatter fit split이
  4줄 이상인 장문 tail인 경우에는 저장 rewind split보다 1줄 fit split을 우선한다. 이때 p15
  케이스는 `lines=0..1`만 현재 단에 남기고 나머지를 다음 쪽으로 넘겨 frame overflow를
  제거한다.
- 같은 guard가 p13(display) `pi=572` 같은 3줄 split 케이스에 적용되면 뒤쪽 TAC 그림 문단
  `pi=629/630`이 새 overflow를 만들었다. 따라서 guard는 formatter fit split이 4줄 이상이고
  현재 문단이 TAC 그림/도형 문단이 아닐 때로 제한한다.
- p18(display) `pi=875/876`, p20(display) `pi=966/967`은 split 문제가 아니라 앞쪽 내부
  rewind/단일 줄 누적 차이로 FullParagraph가 단 하단에 남는 별도 유형이다. p15와 같은
  guard로 함께 처리하면 p15 흐름이 다시 깨져 다음 스테이지로 분리한다.

## 검증 계획

- `cargo fmt --all -- --check`
- `cargo test compact_endnote_zero_between_question_title_caps_forward_gap --lib`
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between0-below20 --out output/task1293_stage22_target_sweep --rhwp-bin target/debug/rhwp`

## 현재 검증 메모

- `cargo fmt --all -- --check`: 통과.
- `cargo test compact_endnote_zero_between_question_title_caps_forward_gap --lib`: 통과.
- `cargo build --bin rhwp`: 통과.
- target sweep:
  - 명령: `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between0-below20 --out output/task1293_stage22_target_sweep --rhwp-bin target/debug/rhwp`
  - SVG/render-tree/PDF 페이지 수: `21/21/21`.
  - 자동 flag: `19/21`, `frame_overflow_pages=[]`.
  - Stage21 대비 p15 `pi=753` overflow 로그가 사라짐.
- 직접 재현:
  - p13(display): `LAYOUT_OVERFLOW` 없음. Stage22 guard 회귀 후보였던 `pi=629/630`을 확인함.
  - p15(display): `LAYOUT_OVERFLOW` 없음. `dump-pages -p 14`에서 `pi=753`이
    `PartialParagraph lines=0..1`로 줄어듦.
  - p18(display): `pi=875/876` overflow 잔여, 최대 `40.5px`.
  - p20(display): `pi=966/967` overflow 잔여, 최대 `46.2px`.
