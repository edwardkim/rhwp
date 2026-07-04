# Task 1293 Stage 79: 전체 미주 sweep 잔여 후보 재분류

## 목적

Stage78에서 `구분선위0/미주사이20/구분선아래2` target의 page20 tail 흐름과
qflow 오탐을 해소했다. 이제 구현 계획서의 visual 검증 범위에 맞춰 전체 sweep을 다시
돌리고, 남은 후보가 실제 미주 기능 결함인지 sweep 한계/허용 drift인지 분류한다.

## 확인 항목

- 전체 target의 SVG/PDF/render tree page count가 기준과 맞는지 확인한다.
- `frame_overflow`, `equation_text_overlap`, `question_title_text_overlap`, `line_order_overlap`은
  실제 렌더 결함 후보로 우선 본다.
- `question_marker_flow_drift`는 Stage78에서 보수화했으므로, 남으면 실제 문항 흐름 차이로 본다.
- 단순 `line_band_drift`, `large_ink_region_drift`는 compare PNG를 열어 허용 drift인지 분리한다.

## 검증 계획

1. `python3 scripts/task1274_visual_sweep.py --target all`을 새 output 경로로 실행한다.
2. `summary.json`에서 page count와 강한 후보 목록을 추린다.
3. 실제 결함 후보가 있으면 다음 스테이지에서 원인 분석 후 수정한다.
4. 후보가 sweep 허용 drift뿐이면 Stage79 결과로 남기고, PR 전 전체 CI 검증은 작업지시자
   승인 후에만 실행한다.

## 검증 결과

- 실행 명령:
  `python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage79_full_sweep --rhwp-bin target/debug/rhwp`
- 전체 target의 SVG/PDF/render tree page count는 모두 일치했다.
- `frame_overflow`, `equation_text_overlap`, `question_title_text_overlap`, `line_order_overlap`,
  `endnote_separator_gap_drift` 후보는 없었다.
- 강한 qflow 후보는 1건 남았다.
  - target: `2024-11-practice-above20-between0-below20`
  - sample: `samples/3-11월_실전_통합_2024-구분선위20미주사이0구분선아래20.hwp`
  - page: 18
  - 한컴/PDF: 18쪽 왼쪽 단은 문30 tail이 이어지고, 오른쪽 단에는 문23~문25가 배치된다.
  - RHWP: 18쪽 왼쪽 단부터 문30이 크게 시작해 문23~문25 흐름이 빠져 있다.
  - 산출물:
    `output/task1293_stage79_full_sweep/2024-11-practice-above20-between0-below20/compare/compare_018.png`

## 판단

- Stage78의 `구분선위0/미주사이20/구분선아래2` 잔여 qflow는 해결되었다.
- 남은 문제는 `구분선 위=20mm`, `미주 사이=0mm`, `구분선 아래=20mm` profile에서
  page/column 경계가 한컴보다 다르게 잡히는 별도 케이스이다.
- 다음 스테이지에서는 page17~18 전환에서 `separator_above/below`가 미주 시작 영역과
  다음 단 흐름에 어떻게 반영되는지 `lineSegArray -> line_seg`와 render tree를 기준으로
  좁혀본다.
