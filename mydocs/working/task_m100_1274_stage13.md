# Task 1274 Stage 13

## 대상

- Stage12 이후 남은 overflow:
  - `2024-09-between20` 21쪽 오른쪽 단 `pi=1080` 2.8px 로그
  - `2024-09-between20` 23쪽 오른쪽 단 `pi=1175` partial overflow

## 관찰

- Stage12 이후 여섯 대상 모두 PDF와 페이지 수가 일치한다.
- `pi=1080`은 2.8px overflow로, 실제 시각 배치가 하단 허용 범위인지 먼저 확인해야 한다.
- `pi=1175`만 더 이른 split으로 보내면 overflow는 사라지지만, 24쪽이 PDF보다 앞쪽 풀이를 과도하게 가져와 시각 흐름이 틀어진다.
- 실제 원인은 22쪽 오른쪽 단의 텍스트 없는 TAC 그림/도형 미주 문단(`pi=1115`)이 저장 vpos보다 순차 y 뒤에 붙어 그림 높이를 이중 가산한 것이다.
- `pi=1115`가 낮게 배치되면서 22쪽에 들어가야 할 문28 꼬리(`pi=1126..1128`)가 23쪽으로 밀렸고, 그 결과 23쪽 문29/문30 흐름과 `pi=1175` overflow가 함께 발생했다.
- `pi=1080`은 마지막 partial continuation 직전의 미주 tail에서 발생한 2.8px item-level bottom bleed이고, draw overflow는 없다.

## 목적

- 남은 `2024-09-between20` overflow를 제거한다.
- 23쪽/24쪽의 문29/문30 흐름을 PDF와 다시 맞춘다.
- Stage11/12의 p13/p14, p18/p19 개선과 24쪽 페이지 수를 유지한다.

## 검증 계획

- 진행 중 자동 테스트는 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`만 사용한다.
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
- 필요 시 전체 sweep으로 여섯 대상 페이지 수와 overflow를 다시 확인한다.
- 전체 CI급 테스트는 전체 목표 마지막에만 수행한다.

## 수정

- `height_cursor`에서 compact 미주 흐름의 텍스트 없는 TAC 그림/도형 문단이 vpos rewind를 가질 때, 큰 backward 보정을 허용했다.
- `typeset`에서도 같은 조건의 TAC 그림/도형 미주 문단은 순차 y 뒤에 높이를 더하지 않고, 저장 vpos 기준 시작 높이와 그림 높이의 최대값만 단 높이에 반영하도록 맞췄다.
- `layout`의 overflow 자가검증에서 마지막 partial continuation 직전 항목도 미주 tail로 보아, draw overflow가 없는 작은 bottom bleed는 기존 미주 하단 허용 로직으로 처리했다.

## 검증 결과

- `cargo fmt` 완료.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 48개 통과.
- `cargo build --bin rhwp` 통과.
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`:
  - SVG 24쪽 / PDF 24쪽.
  - overflow 0건.
  - 22쪽 오른쪽 단에 `pi=1126..1128`이 들어가고 23쪽은 `문29`로 시작한다.
- `python3 scripts/task1274_visual_sweep.py`:
  - `2022-09`: 23/23, overflow 0건.
  - `2023-09`: 20/20, overflow 0건.
  - `2024-09-below20`: 23/23, overflow 0건.
  - `2024-09-between20`: 24/24, overflow 0건.
  - `2022-10`: 18/18, overflow 0건.
  - `2022-11-practice`: 21/21, overflow 0건.

## 시각 확인

- 여섯 대상의 contact sheet를 모두 확인했다.
- `2024-09-between20` 22쪽/23쪽/24쪽은 개별 compare 이미지를 확대 확인했다.
  - 22쪽 오른쪽 단의 TAC 그림이 저장 vpos 기준 위치로 올라가면서 문28 꼬리(`pi=1126..1128`)가 22쪽에 남는다.
  - 23쪽은 PDF처럼 `문29`로 시작한다.
  - 24쪽은 `pi=1175` 후반 풀이와 그림 흐름이 PDF와 맞는다.
- 전체 contact sheet에서 새 수식 겹침, 미주 하단 page overflow, 페이지 수 불일치가 보이지 않는다.

## 산출물 무결성 감사

- `output/task1274/summary.json`의 각 대상별 `svg_pages`, `pdf_pages`, `compare_pages`가 일치한다.
- 각 대상별 `svg`, `rhwp_png`, `pdf_png`, `compare` 파일 개수가 manifest 페이지 수와 일치한다.
  - `2022-09`: SVG/RHWP PNG/PDF PNG/compare 23개.
  - `2023-09`: SVG/RHWP PNG/PDF PNG/compare 20개.
  - `2024-09-below20`: SVG/RHWP PNG/PDF PNG/compare 23개.
  - `2024-09-between20`: SVG/RHWP PNG/PDF PNG/compare 24개.
  - `2022-10`: SVG/RHWP PNG/PDF PNG/compare 18개.
  - `2022-11-practice`: SVG/RHWP PNG/PDF PNG/compare 21개.
- `output/task1274/**/export.log`에서 `LAYOUT_OVERFLOW`, `LAYOUT_OVERFLOW_DRAW` 검색 결과가 없다.
