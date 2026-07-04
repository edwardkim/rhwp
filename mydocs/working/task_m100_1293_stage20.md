# Task 1293 Stage 20: 큰 구분선 p10~p12 미주 흐름 drift 분석

## 목적

Stage19에서 `2024-11-practice-above20-between0-below20`의 쪽수와 major sweep 후보는
정리됐지만, PDF와 비교하면 p10~p12의 실제 미주 흐름이 아직 다르다. 이 단계에서는
수식 overlap 오탐이 아니라 실제 pagination drift로 남은 부분을 분석하고, 공식 미주
모양 공통 로직으로 해결 가능한 보정점을 찾는다.

## 기준 산출물

- `output/task1293_stage19_final_all_check/summary.json`
- `output/task1293_stage19_final_all_check/2024-11-practice-above20-between0-below20/compare/compare_010.png`
- `output/task1293_stage19_final_all_check/2024-11-practice-above20-between0-below20/compare/compare_011.png`
- `output/task1293_stage19_final_all_check/2024-11-practice-above20-between0-below20/compare/compare_012.png`
- `output/task1293_stage19_final_all_check/2024-11-practice-above20-between0-below20/render_tree/render_tree_010.json`
- `output/task1293_stage19_final_all_check/2024-11-practice-above20-between0-below20/render_tree/render_tree_011.json`
- `output/task1293_stage19_final_all_check/2024-11-practice-above20-between0-below20/render_tree/render_tree_012.json`

## 현재 관찰

- p10
  - rhwp 오른쪽 column에서 문6 tail/문10 시작 위치가 PDF와 다르다.
  - page count는 맞지만 column 내부 y 흐름이 PDF보다 압축/이동되어 있다.
- p11
  - rhwp 왼쪽 column 하단에 문12 그림/다음 내용이 PDF와 다른 위치에 들어간다.
  - 오른쪽 column의 문13/문14 위치도 PDF보다 위쪽으로 당겨진다.
- p12
  - PDF는 p11에서 넘어온 그림 tail이 상단에 남아 있고 그 아래 문15가 시작한다.
  - rhwp는 p12 첫 column이 문15부터 시작해 앞쪽 TAC 그림/풀이 tail 이동 정책이 다르다.

## 분석 가설

- 큰 구분선 미주에서 새 미주 첫 문단 head/tail 정책만으로는 충분하지 않다.
- TAC 그림/도형이 포함된 local vpos rewind 문단은 실제 ink 높이, 저장 vpos, column
  남은 영역을 함께 보아 현재 column에 둘지 다음 column/page로 넘길지 결정해야 한다.
- 단순히 “하단 근처 TAC 그림을 다음 page로 넘김”으로 처리하면 21쪽 문서가 22쪽으로
  늘어나는 회귀가 있었으므로, 공통 로직은 page count와 p10~p12 흐름을 동시에 만족해야 한다.

## 원인 확인

- `dump-pages -p 10` 기준 p11 왼쪽 단 하단의 문제 지점은 `pi=537`이다.
- `pi=537`은 텍스트가 비어 있지만 `그림 tac=false wrap=TopAndBottom vert=Para`를 가진
  미주 문단이다.
- 기존 fit 판정은 `para_has_visible_text_or_equation` 중심이라 해당 문단을 12px 수준의
  빈 문단으로 보고 현재 단에 남겼다.
- 실제 SVG render tree에서는 `pi=537`의 그림 bbox가 p11 왼쪽 단 하단 밖으로 내려가고,
  PDF 기준으로는 p11 오른쪽 단 상단으로 넘어가야 한다.
- `pi=537`을 넘기면 p12에서 `문16` 시작 전 저장 vpos가 크게 앞으로 튀는 stale forward
  vpos가 생긴다. 큰 구분선이면서 `미주 사이=0`인 compact 흐름에서는 이 값을 그대로 쓰면
  22쪽으로 늘어나므로, 단에 여유가 있을 때 formatter 높이로 cap해야 한다.

## 검증 계획

- `dump-pages -p 9/10/11`로 p10~p12의 미주 paragraph/shape 분포를 확인한다.
- `render_tree_010/011/012.json`에서 TAC 그림/도형 bbox와 para index를 추적한다.
- `compare_010/011/012.png`를 PDF 기준으로 직접 비교한다.
- 수정 후 최소 검증:
  - `cargo fmt --all -- --check`
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between0-below20 --out output/task1293_stage20_target_check --rhwp-bin target/debug/rhwp`
  - 회귀 target: `2024-09-below20-above20`, `2024-11-practice-shape987`,
    `2024-11-practice-no-separator-above20-between20-below20`

## 구현

- `src/renderer/typeset.rs`
  - non-TAC 그림/도형 전용 미주 문단을 visible payload로 인정한다.
  - 텍스트/수식이 없는 non-TAC 그림/도형 문단은 `common.height + margin`을 fit 높이에
    반영해 단 하단에 억지로 남기지 않는다.
  - `pi=537` 같은 객체 전용 문단은 다음 단으로 넘기되, mixed 텍스트+그림 문단은 기존
    vpos 흐름을 유지해 과도한 쪽수 증가를 막는다.
  - 큰 구분선 문서에서도 `미주 사이=0`인 compact 흐름이고 현재 단에 여유가 있을 때는
    새 미주 시작의 stale forward vpos를 formatter 높이로 cap한다.
- `scripts/task1274_visual_sweep.py`
  - 큰 그림/표/수식 덩어리의 y drift를 감시하는 `large_ink_region_drift` 지표를 추가했다.
  - red/line drift가 있는 페이지에서 큰 ink region의 개수나 중심 y가 PDF와 크게 다르면
    후보로 표시한다.

## 검증 결과

- `cargo build --bin rhwp`: 통과
- `cargo fmt --all -- --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 52개 통과
- Target sweep:
  - `output/task1293_stage20_target_after_cap_narrow/summary.json`
  - `2024-11-practice-above20-between0-below20`: SVG/PDF/render tree `21/21/21`
  - frame/equation/title/order 후보 없음
- Regression sweep:
  - `output/task1293_stage20_regression_2024_09_below20_above20/summary.json`
    - SVG/PDF/render tree `23/23/23`, frame/equation/title/order 후보 없음
  - `output/task1293_stage20_regression_shape987/summary.json`
    - SVG/PDF/render tree `21/21/21`, frame/equation/title/order 후보 없음
  - `output/task1293_stage20_regression_no_separator_after_cap_narrow/summary.json`
    - SVG/PDF/render tree `23/23/23`, frame/equation/title/order 후보 없음

## 남은 후보

- target sweep의 red/line/large drift 후보는 남아 있다.
- p11/p12 compare 기준으로 p11 객체 전용 그림 하단 overflow와 p12 문16 이후 큰 빈 단 문제는
  개선됐지만, 큰 ink region drift는 후속 stage에서 계속 줄여야 한다.
