# task 1274 stage22: visual sweep 정밀 비교 지표 추가

## 배경

- 기존 `scripts/task1274_visual_sweep.py`는 SVG/PDF PNG와 좌우 비교 이미지를 생성하지만, 실제 PDF와 rhwp 결과의 차이를 수치화하지 못한다.
- PR #1277 닫힘 후 전체 재검증에서 `2024-09-between20` page 12의 실제 overflow가 contact sheet만으로는 작게 보였고, export 로그와 확대 비교를 함께 봐야 확인됐다.
- 앞으로는 overflow, 수식/텍스트 겹침, 미주 문항 간격 차이를 자동 후보로 표시해야 한다.
- 수식 겹침은 SVG/PDF 픽셀만으로는 대략 후보를 잡을 수 있지만, 정확도를 높이려면 rhwp render tree의 `Equation`/`TextRun` bbox를 직접 비교해야 한다.

## 목표

- sweep 보조 CLI와 스크립트 변경으로 다음 지표를 `manifest.json`과 `summary.json`에 기록한다.
  - page frame 밖 실제 콘텐츠 픽셀.
  - PDF/rhwp 하단 콘텐츠 위치 차이.
  - 빨간 문항 제목 marker 위치 drift.
  - 텍스트/수식 line band drift.
  - render tree JSON 기반 수식 노드와 텍스트 런의 bbox overlap 후보.
- 문제가 큰 페이지는 `analysis/` 폴더에 annotation PNG를 생성한다.
- `rhwp export-render-tree` 보조 명령으로 페이지별 render tree bbox JSON을 내보낸다.

## 구현 범위

- `src/main.rs`에 `export-render-tree` CLI 보조 명령을 추가한다.
- `scripts/task1274_visual_sweep.py`에서 render tree JSON을 생성/분석한다.
- Rust/WASM 산출물은 수정하지 않는다. 이 CLI는 native debug/export 용도이다.
- PDF 쪽은 semantic bbox가 없으므로 PNG 픽셀 기반 분석을 우선한다.
- rhwp 쪽 수식 겹침은 SVG XML 추정 대신 render tree bbox를 사용한다.

## 검증 계획

- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
  - stage21에서 확인한 page 12 overflow 후보가 자동 분석 결과에 포함되는지 확인한다.
  - `output/task1274/2024-09-between20/render_tree/render_tree_*.json` 생성과 수식 overlap 후보 기록 여부를 확인한다.
- `python3 scripts/task1274_visual_sweep.py --target 2022-10`
  - page 11/16 등 stage20 수정 페이지가 과도한 false positive 없이 기록되는지 확인한다.
- `cargo build --bin rhwp`
  - 새 native CLI 보조 명령 컴파일을 확인한다.
- `python3 -m py_compile scripts/task1274_visual_sweep.py`
  - sweep 스크립트 문법을 확인한다.
- `python3 scripts/task1274_visual_sweep.py --target all`
  - 6종 전체 summary가 생성되고 기존 page count/compare PNG 생성 흐름이 깨지지 않는지 확인한다.

## 상태

- 작업지시자 승인 후 착수.
- `src/main.rs`에 `export-render-tree` native CLI를 추가했다.
- `scripts/task1274_visual_sweep.py`가 PDF/SVG PNG 비교와 render tree bbox 분석을 함께 수행하도록 확장했다.

## 구현 결과

- `rhwp export-render-tree <파일.hwp> -o <폴더> [-p <0-based page>]` 명령을 추가했다.
  - 출력 파일은 `render_tree_001.json` 형식이다.
  - 기존 `build_page_render_tree()` 결과의 `root.to_json()`을 그대로 저장하므로 `Equation`, `TextRun`의 엔진 bbox를 직접 확인할 수 있다.
- `scripts/task1274_visual_sweep.py`는 각 대상별로 다음 산출물을 추가 생성한다.
  - `render_tree/render_tree_*.json`: 페이지별 render tree bbox JSON.
  - `analysis/metrics.json`: 페이지별 픽셀 지표와 render tree 수식 겹침 후보.
  - `analysis/flagged_pages.json`: flag가 있는 페이지만 추린 목록.
  - `analysis/annotated_*.png`: flag 페이지의 rhwp/PDF frame annotation.
- 수식 겹침 후보는 SVG 글꼴/그룹 추정이 아니라 render tree의 `Equation` bbox와 `TextRun` bbox 교차율로 기록한다.
- frame overflow는 PDF 쪽 frame 밖 픽셀 대비 rhwp 쪽 frame 밖 픽셀이 유의미하게 많을 때 flag로 올린다.
- line band drift는 단순 최대값만 보지 않고 평균/90분위 기준을 함께 적용해 false positive를 줄였다.

## 실제 검증해야 할 항목

자동 지표는 후보를 좁히는 장치이므로, 아래 항목은 compare PNG와 annotation PNG를
함께 열어 한컴/PDF 기준으로 직접 확인한다.

| 대상 | 위치 | 실제로 확인할 내용 | sweep 확인값/산출물 |
|---|---:|---|---|
| 6종 전체 | 전 페이지 | SVG/PDF/render tree 페이지 수가 모두 1:1인지 확인한다. 페이지 수가 다르면 이후 시각 비교는 의미가 없다. | `summary.json`의 `svg_pages`, `pdf_pages`, `render_tree_pages`가 각각 `23/23/23`, `20/20/20`, `23/23/23`, `24/24/24`, `18/18/18`, `21/21/21`인지 확인한다. |
| `3-09월_교육_통합_2022.hwp` | 17쪽 문29 | stage23에서 추가 확인된 항목이다. 빨간 `문29） 175` 제목과 다음 본문 `네 장의 카드를 꺼내는 경우의 수는`이 서로 겹치지 않고 PDF처럼 제목 아래로 본문이 시작하는지 확인한다. | 수정 전 render tree는 `pi=900` 제목 y=`1041.1`, `pi=901` 본문 y=`1038.7`로 역전됐다. 수정 후 `question_title_text_overlap_pages`는 빈 배열이고, `compare_017.png`, `annotated_017.png`, `render_tree_017.json`을 확인한다. |
| `3-09월_교육_통합_2022.hwp` | 18쪽 문26 | 큰 루트/분수 TAC 수식이 다음 텍스트와 겹치는지 확인한다. stage18에서 수정한 “문26 수식 겹침” 회귀 감시 항목이다. | `equation_text_overlap_pages`에 18쪽이 포함된다. `metrics.json` page 18 후보는 `text_pi=947`, text=`정사각형의 넓이는`, overlap ratio `0.317`이다. `compare_018.png`, `annotated_018.png`, `render_tree_018.json`을 확인한다. |
| `3-09월_교육_통합_2024-미주사이20.hwp` | 12쪽 | stage21에서 남은 하단 overflow가 실제로 frame 아래로 나가는지 확인한다. 수식/텍스트 bbox 후보도 같은 페이지에서 같이 본다. | `frame_overflow_pages: [12]`, `rhwp_outside_frame_pixels=29`, `pdf_outside_frame_pixels=0`. 수식 후보는 `text_pi=633`, text=`㉡, ㉢에서`, overlap ratio `0.275`. `compare_012.png`, `annotated_012.png`, `render_tree_012.json`을 확인한다. |
| `3-10월_교육_통합_2022.hwp` | 11쪽 문20 주변 | 빨간 문항 marker 위치와 수식/텍스트 bbox 겹침 후보가 실제 시각 문제인지 확인한다. stage18의 문20 주변 미주 흐름 회귀 감시 항목이다. | page 11 flags는 `red_marker_drift`, `equation_text_overlap`이다. 대표 후보는 `text_pi=586`, text=`이차식`, overlap ratio `1.0`이다. `compare_011.png`, `annotated_011.png`, `render_tree_011.json`을 확인한다. |
| `3-10월_교육_통합_2022.hwp` | 16쪽 문30 | 문30 제목과 첫 본문 줄이 overflow로 오탐되지 않는지, 빨간 marker drift 후보가 실제 배치 차이인지 확인한다. | page 16은 `red_marker_drift`만 flag되고, frame overflow와 equation overlap은 없다. `rhwp_outside_frame_pixels=5`, `content_bottom_delta_px=4.0`. `compare_016.png`, `annotated_016.png`를 확인한다. |
| `3-09월_교육_통합_2023.hwp` | 19쪽 | 원래 수정 대상은 아니지만 full sweep에서 새 frame/line 후보로 잡힌 페이지다. 실제 하단 overflow인지 frame 검출 오탐인지 확인한다. | `frame_overflow_pages`에 19쪽이 포함된다. page 19는 `rhwp_outside_frame_pixels=276`, `line_band_drift` 평균 `235.8px`이다. `compare_019.png`, `annotated_019.png`를 확인한다. |
| `3-11월_실전_통합_2022.hwp` | 12쪽 | full sweep에서 큰 frame overflow 후보가 잡힌 페이지다. 11~12쪽 partial tail/수식 흐름이 PDF와 맞는지 확인한다. | `frame_overflow_pages`에 12쪽이 포함된다. page 12는 `rhwp_outside_frame_pixels=9269`, `content_bottom_delta_px=-185.0`, 수식 후보 `text_pi=565`, text=`다음과 같고`, overlap ratio `0.239`이다. `compare_012.png`, `annotated_012.png`, `render_tree_012.json`을 확인한다. |
| `3-11월_실전_통합_2022.hwp` | 19쪽 | full sweep에서 frame/red/line 후보가 동시에 잡힌 페이지다. 실제 하단 bleed인지, marker drift가 시각적으로 허용 가능한지 확인한다. | page 19 flags는 `frame_overflow_pixels`, `red_marker_drift`, `line_band_drift`이다. `rhwp_outside_frame_pixels=60`, line drift 평균 `101.3px`. `compare_019.png`, `annotated_019.png`를 확인한다. |

## 검증 결과

- `cargo build --bin rhwp` 통과.
- `python3 -m py_compile scripts/task1274_visual_sweep.py` 통과.
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20` 통과.
  - SVG/PDF/render tree 모두 24쪽.
  - stage21 잔여 overflow였던 page 12가 `frame_overflow_pages: [12]`로 잡혔다.
  - page 12 `metrics.json`에는 `Equation`/`TextRun` overlap 후보가 기록됐다.
    - `text_pi=633`, text=`㉡, ㉢에서`
    - equation bbox `[34.0, 435.9, 62.4, 31.4]`
    - text bbox `[34.0, 427.2, 57.0, 12.0]`
    - overlap ratio `0.275`
- `python3 scripts/task1274_visual_sweep.py --target 2022-10` 통과.
  - SVG/PDF/render tree 모두 18쪽.
  - page 11은 red marker drift와 render tree 수식/text overlap 후보로 잡혔다.
- `python3 scripts/task1274_visual_sweep.py --target all` 통과.
  - `2022-09`: SVG/PDF/render tree 23/23/23쪽.
  - `2023-09`: SVG/PDF/render tree 20/20/20쪽.
  - `2024-09-below20`: SVG/PDF/render tree 23/23/23쪽.
  - `2024-09-between20`: SVG/PDF/render tree 24/24/24쪽, frame overflow page `[12]`.
  - `2022-10`: SVG/PDF/render tree 18/18/18쪽.
  - `2022-11-practice`: SVG/PDF/render tree 21/21/21쪽.

## 참고

- 자동 후보는 한컴 시각 판정을 대체하지 않는다. 특히 line band drift와 equation overlap은 검토 후보를 좁히는 보조 지표로 사용한다.
- render tree bbox 기반 겹침은 SVG/PDF 픽셀 추정보다 재현성이 높지만, `TextRun` bbox가 glyph tight bbox가 아니라 레이아웃 run bbox라는 점은 해석 시 고려해야 한다.
