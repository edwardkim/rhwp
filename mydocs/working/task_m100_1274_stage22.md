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
