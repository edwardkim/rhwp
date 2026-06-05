# task 1284 stage8: page18/page21 문항 drift pagination 분기 분석

## 배경

- stage7 커밋: `task 1284: 미주 제목 compact gap preserve 보정`
- stage7 focused sweep 결과는 `frame=[]`, `question=[18, 21]`이다.
- stage7은 page17 오른쪽 단 문27/문28 제목 gap을 PDF 기준에 가깝게 보정했지만,
  page18/page21의 큰 문항 drift는 남았다.

## 현재 남은 후보

- `3-09월_교육_통합_2024-미주사이20.hwp`
- page18:
  - 문29 `pi=900`: rhwp y=483.3, PDF y=404.2, drift `+79.1px`
  - 문30 `pi=928`: rhwp y=447.3, PDF y=375.4, drift `+71.9px`
  - 문23 `pi=935`: rhwp y=969.8, PDF y=891.4, drift `+78.4px`
- page21:
  - 문30 `pi=1025`: rhwp y=268.6, PDF y=215.0, drift `+53.6px`

## 분석 방향

- render/layout gap preserve만으로는 page18/page21 후보가 움직이지 않았다.
- page18은 문29 앞의 문28 tail(`pi=894..899`)이 PDF보다 긴 위치를 차지한다.
- page21은 문30 앞의 문29 tail(`pi=1021..1024`)이 PDF보다 긴 위치를 차지한다.
- 두 케이스 모두 이전 미주 tail의 pagination 분기/누적 높이 판단이 다음 문항 시작을 밀고 있는지 확인한다.

## 구현 판단

- pagination은 page18의 `pi=900/928/935`, page21의 `pi=1025`를 이미 같은 쪽/단에 남기고 있었다.
- 실제 drift는 render 단계에서 직전 미주 tail의 마지막 `line_spacing=5669HU(20mm)`가 paragraph advance로 먼저 소비된 뒤,
  다음 문항 제목이 그 위치에서 시작하면서 발생했다.
- 단순히 모든 20mm gap을 접으면 page13 문16, page21 문24처럼 한컴/PDF에서 full gap을 유지해야 하는 정상 케이스가 깨진다.
- 따라서 다음 조건을 모두 만족할 때만 공통 보정을 적용했다.
  - 현재 항목이 compact endnote의 문항 제목이다.
  - 직전 항목의 실제 content bottom과 현재 y 사이에 20mm급 gap이 이미 소비되어 있다.
  - 직전 tail이 단일 treat-as-char 수식 중심이다.
  - page/column 앞쪽의 late question(`문29`, `문30`) 경계다.
- 저장 vpos가 크게 튄 경우(page21 문30)는 20mm 전체를 제거하지 않고 기본 흐름 몫(약 7mm)을 남겨 문30을 PDF y=215px 근처로 맞췄다.

## 중간 확인

- `cargo run --quiet --bin rhwp -- export-render-tree ... --page 17`
  - page18 문29: rhwp y=`407.7`, PDF y=`404.2`, delta `+3.5`
  - page18 문30: rhwp y=`371.7`, PDF y=`375.4`, delta `-3.7`
  - page18 문23: rhwp y=`894.2`, PDF y=`891.4`, delta `+2.8`
- `cargo run --quiet --bin rhwp -- export-render-tree ... --page 20`
  - page21 문30: rhwp y=`219.4`, PDF y=`215.0`, delta `+4.4`
  - page21 문24/25/26 기존 기준은 유지됐다.
- 보호 확인
  - page10 문8~문12는 기존 PDF 기준 범위 유지.
  - page13 문15~문18은 기존 PDF 기준 범위 유지.
  - page23 문29/문30은 기존 PDF 기준 범위 유지.

## 검증 결과

- `cargo fmt --check`: 통과.
- `cargo test --test issue_1139_inline_picture_duplicate issue_1274_2022_oct_page16_question30_title_keeps_first_line -- --nocapture`: 통과.
  - 기본 7mm 미주 사이 문서의 page16 문30 하단 tail 회귀가 없음을 확인했다.
- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20 -- --nocapture`: 5개 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 56개 통과.
- `cargo build`: 통과.
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`: 완료.
  - `frame_overflow_pages=[]`
  - `question_marker_drift_pages=[]`
  - `question_title_text_overlap_pages=[]`
  - `line_order_overlap_pages=[]`
  - page18/page21의 기존 문항 drift 후보는 사라졌다.
