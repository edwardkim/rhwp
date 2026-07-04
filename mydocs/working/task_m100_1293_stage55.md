# Task 1293 Stage 55: no-separator 문항 흐름 drift 원인 수정

## 목적

Stage54에서 `question_marker_flow_drift`를 추가해
`2024-11-practice-no-separator-above20-between20-below20`의 구조적 문항 흐름 차이를 자동으로
감지했다. 이번 단계에서는 첫 강한 drift 구간을 실제 typeset 계산식에서 수정한다.

## 대상

- sample: `samples/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp`
- reference PDF: `pdf/3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.pdf`
- latest sweep: `output/task1293_stage54_qflow_no_separator`
- current qflow 후보: `[18, 20, 21, 22, 23]`

## 현재 관찰

- page count는 23/23/23으로 맞다.
- `compare_020.png`: PDF는 문23/문24를 왼쪽 단 하단에 남기지만 rhwp는 문23부터 오른쪽 단에 배치한다.
- `compare_021.png`: rhwp는 문27/문28 흐름이고 PDF는 문28 continuation/문29 흐름이다.
- 따라서 21쪽은 결과 지점이고, 원인은 18~20쪽 사이의 미주 paragraph 경계 누적이다.

## 분석 계획

1. render tree에서 page 18~21의 문항 title `pi`와 y/column 위치를 추출한다.
2. PDF page text와 비교해 첫 divergence가 어느 문항인지 고정한다.
3. `typeset.rs`의 no-separator + large between-notes 조건에서 해당 경계의
   `en_fit`, `total_advance_fit`, `vpos_offset`, `advance_for_fit`, `advance_for_new_endnote`를 추적한다.
4. 증상별 y 보정이 아니라 공식 미주 모양 의미에 맞게, 구분선 없음/미주 사이/구분선 위아래의 소비
   모델을 좁혀 수정한다.

## 검증 계획

- focused:
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- targeted visual:
  - `cargo build --bin rhwp`
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20 --out output/task1293_stage55_no_separator --rhwp-bin target/debug/rhwp`
- qflow 후보가 줄어드는지 확인한다. 단순 page count 유지가 아니라 `compare_020.png`,
  `compare_021.png` 직접 확인을 포함한다.

## 수정 내용

### 1. 구분선 없음 문항 제목의 단 하단 유지

`src/renderer/typeset.rs`에서 `large_separator_block && !has_visible_endnote_separator`인 새 미주 번호
문단이 현재 단 하단에 제목 한 줄과 `미주 사이` gap까지 들어갈 수 있으면, stale forward vpos cap과
`advance_for_new_endnote` 강제 넘김에서 제외했다.

- 기준 현상: `pi=899` 문24 제목이 `cur=891.3`, `available=1001.6`, `미주 사이≈75.6px` 조건에서
  제목은 현재 단에 들어갈 수 있는데도 일반 threshold로 오른쪽 단으로 넘어갔다.
- 변경 후: page 20 단0에 `pi=895` 문23뿐 아니라 `pi=899` 문24 제목까지 남는다.
- 한계: PDF는 문24 본문 일부도 왼쪽 단 하단에 남으므로, 문25 tail 이전 누적 높이 차이는 다음
  단계에서 계속 분석한다.

### 2. 구분선 없음 TAC 그림 되감김 높이 소비

TAC 그림/도형만 있는 미주 문단이 저장 vpos상 앞 문단 옆으로 되감기는 경우, 보이는 구분선 compact
gap에서만 순차 높이를 소비하던 조건을 `large_separator_block && !has_visible_endnote_separator`에도
적용했다.

- 기준 현상: `pi=996` TAC 그림 문단은 renderer에서는 그림 높이만큼 아래 내용을 밀지만,
  pagination은 `current_height`를 유지해 page 22 tail이 frame 아래로 내려갈 수 있었다.
- 변경 후: targeted sweep에서 renderer `LAYOUT_OVERFLOW` 로그가 0건이 된다.

## 검증 결과

- `cargo fmt --all -- --check`
  - 통과
- `cargo build --bin rhwp`
  - 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 52개 전부 통과
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20 --out output/task1293_stage55_no_separator_final --rhwp-bin target/debug/rhwp`
  - page count: `23/23/23`
  - `overflow_lines`: 0
  - `frame_overflow_pages`: []
  - `question_title_text_overlap_pages`: []
  - `line_order_overlap_pages`: []
  - `equation_text_overlap_pages`: []
  - `question_marker_flow_drift_pages`: `[18, 22]`

## 직접 확인

- `output/task1293_stage55_no_separator_final/2024-11-practice-no-separator-above20-between20-below20/compare/compare_020.png`
  - 개선: rhwp page 20 왼쪽 단 하단에 문24 제목이 남는다.
  - 잔여: PDF는 문24 제목과 본문 일부가 왼쪽 단 하단에 남는데, rhwp는 문24 본문(`pi=900` 이후)을
    오른쪽 단으로 넘긴다.
- `compare_018.png`
  - 잔여: PDF는 문26 이어쓰기부터 시작하고 문27이 중간에서 시작하지만, rhwp는 문26 제목이 page 18에
    남아 있어 첫 drift가 계속된다.
- `compare_022.png`
  - 잔여: 문30 큰 그림 tail의 분배는 overflow는 사라졌지만 PDF와 세부 흐름이 아직 다르다.

## 다음 단계

Stage56에서는 `question_marker_flow_drift_pages`에 남은 `[18, 22]`를 별도로 분석한다.

- page 17~18의 문25 tail `pi=783`, `pi=784`와 문26 제목 `pi=785` 경계
  - 현재 `pi=783`은 `cur=987.8`, `en_fit=16.4`로 약 2~3px 초과해 다음 쪽으로 넘어간다.
  - 하지만 문26 제목까지 현재 쪽에 맞추려면 단순 bottom bleed가 아니라 문23~문25 누적 높이 원인을
    줄여야 한다.
- page 22의 문30 큰 그림 tail
  - `LAYOUT_OVERFLOW`는 0이 되었으나, PDF와 비교하면 세부 문단 분배가 아직 다르다.
