# Task 1274 Stage 16

## 대상

- `samples/3-09월_교육_통합_2022.hwp`
- 비교 기준: `pdf/3-09월_교육_통합_2022.pdf`
- 위치: 18쪽 좌측 하단 문26)

## 발견

- 문26) 하단의 큰 inline 수식이 같은 줄의 실제 높이에 충분히 반영되지 않는다.
- 그 결과 다음 텍스트 `그러므로 구하는 ...` 줄이 수식 아래로 내려가지 못하고 수식 glyph와 겹친다.
- Stage15 스윕은 페이지 수와 overflow 로그만 확인했기 때문에, 페이지 내부 줄 높이 부족으로 생기는 겹침을 감지하지 못했다.

## 분석 방향

- 문26) 수식 컨트롤 bbox와 뒤따르는 텍스트 bbox를 render tree에서 직접 비교한다.
- 해당 문단의 line segment 높이, inline control 높이, line advance 계산 경로를 확인한다.
- 문26 전용 좌표 보정보다 본문/미주 빈 TAC 수식 줄의 공통 baseline 정렬로 해결한다.

## 원인

- 문26) 하단 수식은 `pi=948`의 텍스트 없는 TAC 수식 문단이고, 다음 문단 `pi=949`가 `그러므로 구하는 ...` 텍스트다.
- `EquationNode.bbox.height`는 HWP 저장 영역을 사용하지만 SVG/Canvas/Skia 렌더러는 수식을 세로로 늘리지 않고 `layout_box.height` 그대로 그린다.
- 기존 본문/미주 빈 TAC 수식 줄은 `eq_y = y`로 배치되어 큰 루트/분수 수식이 줄 아래쪽으로 내려앉았고, 다음 문단의 텍스트 line box와 시각적으로 겹쳤다.

## 수정

- `layout_composed_paragraph`의 두 inline equation 렌더 경로에서, 본문/미주 빈 TAC 수식 줄도 텍스트 혼합 수식과 동일하게 baseline 기준으로 y를 정렬한다.
- 다음 문단 y advance를 크게 늘리는 방식은 페이지 하단 텍스트를 밀어내므로 적용하지 않았다.
- 회귀 테스트는 render tree bbox가 아니라 실제 렌더 높이인 `EquationNode.layout_box.height` 기준으로 수식 하단을 비교한다.

## 검증 계획

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2022-09`
- 필요 시 `python3 scripts/task1274_visual_sweep.py`

## 검증 결과

- `cargo test --test issue_1139_inline_picture_duplicate issue_1274_2022_sep_page18_question26_equation_paragraph_reserves_height -- --nocapture`: 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 49개 통과.
- `cargo fmt --check`: 통과.
- `cargo build --bin rhwp`: 통과.
- `python3 scripts/task1274_visual_sweep.py --target 2022-09`: SVG/PDF/compare 23/23/23, overflow 0건.
- `python3 scripts/task1274_visual_sweep.py`:
  - `2022-09`: SVG/PDF/compare 23/23/23, overflow 0건.
  - `2023-09`: SVG/PDF/compare 20/20/20, overflow 0건.
  - `2024-09-below20`: SVG/PDF/compare 23/23/23, overflow 0건.
  - `2024-09-between20`: SVG/PDF/compare 24/24/24, overflow 0건.
  - `2022-10`: SVG/PDF/compare 18/18/18, overflow 0건.
  - `2022-11-practice`: SVG/PDF/compare 21/21/21, overflow 0건.
