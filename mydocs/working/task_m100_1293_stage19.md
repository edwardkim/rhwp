# Task 1293 Stage 19: 큰 구분선 단 경계 회복과 p13 수식 후보 분석

## 목적

Stage18 이후 `2024-11-practice-above20-between0-below20`에서 큰 구분선 미주 단 경계
판단이 흔들리면 PDF 기준 21쪽 정합이 깨지는 것을 확인했다. 이 단계에서는 큰 구분선
미주 단 경계 판단을 보정해 21쪽을 유지하고, Stage18 최종 sweep에서 남았던 p13 수식
겹침 후보가 실제 렌더링 문제인지 확인한 뒤 sweep 검출기를 보정한다.

## 기준 산출물

- `output/task1293_stage18_all_detector_check/summary.json`
- `output/task1293_stage18_all_detector_check/2024-11-practice-above20-between0-below20/analysis/metrics.json`
- `output/task1293_stage18_all_detector_check/2024-11-practice-above20-between0-below20/analysis/annotated_013.png`
- `output/task1293_stage18_all_detector_check/2024-11-practice-above20-between0-below20/render_tree/render_tree_013.json`

## 현재 후보

- SVG/PDF/render-tree 쪽수는 21/21/21로 일치한다.
- `frame_overflow_pages=[]`, `question_title_text_overlap_pages=[]`, `line_order_overlap_pages=[]`.
- 남은 major 후보:
  - `equation_text_overlap_pages=[13]`
  - 후보 bbox:
    - equation path `root/6/0/26/2`
    - text path `root/6/1/16/0`
    - text=`(ⅵ) `
    - overlap ratio `0.95`
    - overlap px `12.0`

## 분석 결과

- `samples/3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20.hwp`는 큰 구분선
  단 경계 조건이 조금만 어긋나도 20쪽 또는 22쪽으로 흔들린다.
- 큰 구분선 블록에서 새 미주 첫 문단을 column 하단에 둘지 판단할 때, 마지막 column과
  첫 column을 같은 기준으로 다루면 한쪽에서는 제목 tail이 끼어들고 다른 쪽에서는
  들어갈 수 있는 head tail까지 다음 column으로 밀린다.
- 마지막 column에서는 큰 구분선 새 미주 제목 tail을 억지로 split하지 않고, 첫 column에서는
  실제 높이상 들어갈 수 있는 head tail을 허용해야 PDF 쪽수와 흐름이 유지된다.
- `output/task1293_stage18_all_detector_check/2024-11-practice-above20-between0-below20/rhwp_png/rhwp_013.png`에서
  후보 영역을 직접 잘라 확인했다.
- 실제 화면에서는 왼쪽 단 하단 수식 ink와 오른쪽 단 첫 텍스트 `(ⅵ)` ink가 겹치지 않았다.
- 오탐 원인은 render-tree 수식 bbox가 실제 ink보다 넓고, 기존 detector가 bbox끼리의
  교차만으로 수식/텍스트 겹침을 판단했기 때문이다.
- 같은 샘플의 p10~p12는 PDF와 흐름 차이가 남아 있지만, 이는 수식 ink overlap이 아니라
  큰 구분선 미주의 단 경계/pagination 정합 문제이다. 이 단계에서는 검출기의 수식
  겹침 오탐만 분리하고, 남은 레이아웃 drift는 다음 stage에서 별도 보정한다.
- 따라서 renderer 쪽은 큰 구분선 단 경계의 새 미주 head/tail 판단만 보정하고,
  p13 수식 후보는 sweep 쪽에서 실제 RHWP PNG ink 교차를 함께 보도록 보정한다.

## 수정 내용

- `src/renderer/typeset.rs`
  - 큰 구분선 블록의 마지막 column에서 새 미주 첫 문단이 frame을 넘는 경우 own-vpos-span
    fit만으로 현재 column에 남기지 않도록 했다.
  - 마지막 column 하단의 한 줄짜리 새 미주 제목 split은 금지해 제목만 하단에 끼는
    흐름을 방지했다.
  - 첫 column에서는 실제 `en_fit`이 남은 영역에 들어가면 큰 구분선 새 미주 head tail을
    보존해 불필요한 다음 column advance를 피했다.
- `scripts/task1274_visual_sweep.py`
  - render-tree 수식/텍스트 bbox 후보를 먼저 거른 뒤 RHWP PNG에서 실제 ink bbox를 다시 계산한다.
  - 같은 `TextLine` 내부의 수식/텍스트는 정상 inline 흐름으로 보아 제외한다.
  - 양단 column sibling 경계에서 왼쪽 수식 bbox가 오른쪽 column 시작 x까지 넓게 잡히는
    경우를 column boundary 오탐으로 제외한다.
  - 인접 flow line 사이에서 같은 흐름의 수식/텍스트 bbox가 살짝 닿는 후보를 제외한다.
  - 후보 JSON에 `equation_ink_bbox`, `text_ink_bbox`를 추가해 오탐/실제 겹침을 구분할 수 있게 했다.

## 검증 결과

- `python3 -m py_compile scripts/task1274_visual_sweep.py`
- `cargo build --bin rhwp`
- `cargo fmt --all -- --check`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 52 passed
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between0-below20 --out output/task1293_stage19_final_p13_check --rhwp-bin target/debug/rhwp`
  - SVG/PDF/render-tree 쪽수 `21/21/21`
  - `frame_overflow_pages=[]`
  - `equation_text_overlap_pages=[]`
  - `question_title_text_overlap_pages=[]`
  - `line_order_overlap_pages=[]`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-below20-above20 --out output/task1293_stage19_final_p18_check --rhwp-bin target/debug/rhwp`
  - SVG/PDF/render-tree 쪽수 `23/23/23`
  - `frame_overflow_pages=[]`
  - `equation_text_overlap_pages=[]`
  - `question_title_text_overlap_pages=[]`
  - `line_order_overlap_pages=[]`
- `python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage19_final_all_check --rhwp-bin target/debug/rhwp`
  - 15개 대상 모두 SVG/PDF/render-tree 쪽수가 1:1로 일치한다.
  - `frame_overflow_pages`, `equation_text_overlap_pages`,
    `question_title_text_overlap_pages`, `line_order_overlap_pages`가 모두 비었다.
  - red/line drift 후보는 남아 있으며, 다음 stage에서 실제 시각 흐름 보정 입력으로 사용한다.

## 다음 stage로 분리할 내용

- `2024-11-practice-above20-between0-below20` p10~p12는 PDF와 여전히 시각 흐름이 다르다.
  - p10은 오른쪽 column의 문6 tail/문10 시작 위치가 PDF와 다르다.
  - p11은 문12 그림 tail과 문13/문14 흐름 위치가 PDF와 다르다.
  - p12는 PDF 기준 p11에서 넘어온 그림 tail이 상단에 있어야 하는데 rhwp는 문15부터 시작한다.
- 원인은 수식 bbox 겹침이 아니라 큰 구분선 미주에서 TAC 그림/도형 rewind와 새 미주
  시작 위치를 column 경계에서 어떻게 분리할지의 pagination 정책 문제로 본다.
