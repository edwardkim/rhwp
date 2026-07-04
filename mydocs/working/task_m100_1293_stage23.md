# Task 1293 Stage 23: compact 미주 FullParagraph 하단 overflow 분석

## 목적

Stage22에서 p15(display) `pi=753` partial split overflow는 제거했다. target sweep에는
p18(display) `pi=875/876`, p20(display) `pi=966/967` FullParagraph overflow가 남아 있다.
이번 단계는 split 후보가 아니라 단 하단에 단일/소형 FullParagraph가 남는 누적 오차 유형을
분리해 원인을 확인한다.

## 기준

- 기준 sweep: `output/task1293_stage22_target_sweep/summary.json`
- 대상 문서: `samples/3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20.hwp`
- 잔여 overflow:
  - p18(display): `pi=875/876`, 최대 `40.5px`
  - p20(display): `pi=966/967`, 최대 `46.2px`

## 작업 방향

1. `dump-pages`와 `LAYOUT_OVERFLOW` 로그로 p18/p20의 직전 internal rewind 또는 TAC 그림
   누적 차이를 찾는다.
2. Stage22처럼 앞쪽 흐름을 크게 흔드는 전역 advance threshold는 피한다.
3. FullParagraph가 frame 밖으로 남는 경우에는 해당 문단 직전의 실제 renderer y와
   typeset 누적 높이 차이를 줄이는 조건을 우선 검토한다.

## 구현 결과

- p18(display) `pi=875/876`은 p853 내부 rewind 문단 이후 누적 y가 renderer보다 작아진 뒤,
  왼쪽 단 하단에 1줄 FullParagraph가 남으면서 overflow했다.
- p20(display) `pi=966/967`도 왼쪽 단 하단 96% 이후의 소형 텍스트 FullParagraph가
  renderer에서는 frame 밖에서 시작하는 같은 tail 유형이었다.
- p11(display)도 p510 내부 rewind 문단이 141.9px formatter 높이를 44.5px fit 기준으로만
  누적하면서 뒤쪽 p535/p536이 frame 밖에 남는 같은 계열이었다.
- compact 미주의 내부 rewind 문단이 TAC 그림/도형이 아니고 단 초중반(`available * 0.45`
  이전)에 있으며 formatter 전체 높이가 vpos metric보다 40px 이상 크면 전체 높이를 누적한다.
- compact 미주에서 오른쪽 단으로 넘길 수 있는 왼쪽 단이고, 현재 문단이 TAC 그림/도형이나
  rewind 문단이 아니며, 1~2줄 텍스트 tail이 `available * 0.96` 이후에 놓이면
  `height_for_fit`만 믿지 않고 다음 단으로 넘긴다.
- Stage22에서 발견한 p15(display) `pi=753`은 p707 내부 rewind 누적 보정 후 formatter fit
  split이 1줄로 바뀌므로, 단 하단 97% 이후의 `split=1` internal rewind도 저장 rewind split보다
  fit split을 우선하도록 유지했다.
- Stage22에서 발견한 p13(display) `pi=629/630` TAC 그림 회귀와 p15(display) `pi=753`
  partial split 회귀를 함께 직접 확인했다.

## 검증 계획

- `cargo fmt --all -- --check`
- `cargo test compact_endnote_zero_between_question_title_caps_forward_gap --lib`
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between0-below20 --out output/task1293_stage23_target_sweep --rhwp-bin target/debug/rhwp`

## 현재 검증 메모

- `cargo fmt --all -- --check`: 통과.
- `cargo test compact_endnote_zero_between_question_title_caps_forward_gap --lib`: 통과.
- `cargo build --bin rhwp`: 통과.
- target sweep:
  - 명령: `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between0-below20 --out output/task1293_stage23_target_sweep --rhwp-bin target/debug/rhwp`
  - SVG/render-tree/PDF 페이지 수: `21/21/21`.
  - 자동 flag: `19/21`, `frame_overflow_pages=[]`.
  - `overflow_lines`: `0`.
- 직접 재현:
  - p11(display): `LAYOUT_OVERFLOW` 없음.
  - p13(display): `LAYOUT_OVERFLOW` 없음.
  - p15(display): `LAYOUT_OVERFLOW` 없음.
  - p18(display): `LAYOUT_OVERFLOW` 없음.
  - p20(display): `LAYOUT_OVERFLOW` 없음.
