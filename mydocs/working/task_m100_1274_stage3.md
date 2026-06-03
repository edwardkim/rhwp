# Task 1274 Stage 3

## 대상

- 문서: `samples/3-11월_실전_통합_2022.hwp`
- PDF: `pdf/3-11월_실전_통합_2022.pdf`
- 페이지: 11쪽, 0-index `p=10`
- 로그: `LAYOUT_OVERFLOW_DRAW section=0 pi=553 line=7 y=1106.7 col_bottom=1092.3 overflow=14.4px`

## 배경

Stage2에서 같은 페이지의 `pi=537` 빈 non-TAC 그림 host phantom overflow를 제거했다.
재생성한 `2022-11-practice` manifest에서는 `pi=537`의 1530px overflow가 사라졌고, 11쪽에 남은 주요 후보는 `pi=553`의 14.4px overflow다.

## 1차 관찰

`dump-pages -p 10` 기준 `pi=553`은 오른쪽 단 마지막 항목이며 `PartialParagraph lines=0..8`로 배치된다.
표시된 vpos 범위는 `238574..214356`으로 내부 vpos 되감김을 갖는다.

## 진행 계획

1. `pi=553`의 렌더 라인과 한컴 PDF의 11쪽/12쪽 분기 위치를 비교한다.
2. 이 overflow가 실제 내용 초과인지, 마지막 줄 trailing/partial split 오탐인지 구분한다.
3. 수정이 필요하면 문항 번호 전용 보정이 아니라 partial paragraph 하단/내부 vpos 되감김 공통 로직으로 처리한다.
4. 검증은 진행 중에는 `tests/issue_1139_inline_picture_duplicate.rs`만 사용하고, CI급 전체 테스트는 전체 작업 마지막에만 수행한다.

## 분석 결과

- 11쪽 오른쪽 단 끝의 `pi=553`은 `PartialParagraph lines=0..8`이고, 12쪽 첫머리는 `lines=8..11`로 이어진다.
- PDF도 11쪽 끝에 `iv) x>1일 때` 줄을 남기고, 다음 식은 12쪽 첫머리에서 시작한다.
- 따라서 `pi=553`을 다음 쪽으로 넘기거나 split line 수를 줄이면 한컴/PDF 분기가 깨진다.
- 문제는 실제 페이지 밖 초과가 아니라 compact 미주 하단에서 줄 박스가 본문 하단을 약간 넘지만 페이지 테두리 안에는 남는 small bleed를 `LAYOUT_OVERFLOW`로 기록한 오탐이다.

## 수정

- `src/renderer/layout.rs`
  - `is_tolerated_endnote_column_bottom_bleed` 공통 헬퍼를 추가했다.
  - compact 미주 하단의 작은 bleed가 단의 마지막 항목이면 항목 단위 `LAYOUT_OVERFLOW`로 기록하지 않도록 했다.
- `src/renderer/layout/paragraph_layout.rs`
  - 같은 기준을 줄 단위 `LAYOUT_OVERFLOW_DRAW`에도 적용했다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - 11쪽 `pi=553 lines=0..8`, 12쪽 `pi=553 lines=8..11` 분기가 유지되는지 확인하는 회귀 테스트를 추가했다.

## 검증

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 48개 테스트 통과.
  - 진행 중 테스트 범위 제한에 맞춰 전체 CI급 테스트는 실행하지 않았다.
- `cargo build --bin rhwp`
  - 네이티브 SVG export용 바이너리 빌드 통과.
- `python3 scripts/task1274_visual_sweep.py --target 2022-11-practice`
  - SVG/PDF/비교 PNG: 21/21/21.
  - `pi=553` 관련 `LAYOUT_OVERFLOW_DRAW`/`LAYOUT_OVERFLOW` 로그 제거 확인.
  - 남은 manifest overflow 후보: `pi=0` 2건.

## 시각 확인

- `output/task1274/2022-11-practice/compare/compare_011.png`
  - 11쪽 `pi=553` 마지막 `iv) x>1일 때` 줄은 PDF와 동일하게 11쪽 끝에 남는다.
- `output/task1274/2022-11-practice/compare/compare_012.png`
  - 12쪽 첫머리도 `pi=553` 나머지 줄에서 시작한다.
  - 다만 12쪽 이후 좌/우 단 분기 차이가 여전히 남아 있어 다음 stage에서 별도 분석한다.
