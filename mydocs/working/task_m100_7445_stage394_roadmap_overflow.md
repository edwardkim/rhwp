# 단계 394: 로드맵 43쪽 행 소속과 본문 넘침

## 수정 전 분석

- 입력: `samples/issue5941/1490000-201600081_roadmap_research.hwp`.
- 기준: 기존 독립 Print PDF `pdf/issue7382-regression-review/1490000-201600081_roadmap_research-2020.pdf` 302쪽.
- 정확한 base `21ed64071`는 299쪽, 현재 후보는 302쪽입니다.
- 현재 본문 넘침 partition 4는 이 문서의 신규 1건 때문에 실패합니다.
- 원문 0.902 표의 4행 셀 14는 앞 문단의 저장 vpos 12780 뒤 마지막 문단이 0으로 돌아갑니다.
  PDF 43쪽에는 앞 일곱 문단, 44쪽에는 마지막 ‘양질의 시간제 비중 증가’ 문단이 있습니다.
- 현재 scanner는 첫 조각에 0..5행을 통째 수용합니다. 저장 프레임의 초과 허용이
  첫 조각 379.2px를 잔여 374.8px보다 크게 받았으며 실제 본문 밖 4.4px가 생겼습니다.
- 통째 수용 조건은 제한된 두 줄/문단 내부 저장 프레임만 확인하여 이 문단 사이의 경계를 놓칩니다.

## 수정 계약과 소비 경로

원본 HWP5 저장 조판의 비편집 RowBreak 행에서 유효한 CellUnit의 저장 물리 경계가 있으면
통째 행 경로를 건너뛰고 기존 row-cut walker가 같은 CellUnit 경계를 소비하게 합니다.
직접 HWPX의 writer-local reset 판정, 편집/재조판 입력, 일반 단조로운 행은 그대로 둡니다.
좌표 공차 확대나 측정 행 높이 절삭으로 넘침을 숨기지 않습니다.

`CellUnit.hard_break_before → row_block_has_internal_hard_break와 첫 source frame의 행 끝 → whole_row_fits의 판정 →
advance_row_cut / split_end_cut → emit_table_fragment → layout_partial_table`를 확인합니다.
본문 문단과 국가 표의 소속을 검사하는 기존 로드맵 회귀 두 개는 수정 전 통과했습니다.
새 회귀를 추가하거나 baseline을 바꾸지 않습니다.

## 검증 상태

수정 전 Native base/head 전체 TSV와 43쪽 review PNG 산출 중입니다.
원문·PDF의 전체 비교와 수정 후 기존 회귀, Native/fresh WASM 결과를 확인한 뒤 결과를 추가합니다.

### 첫 시도와 보완

첫 시도는 `stored_frame_break_before`의 컷을 확인했지만 기존 실패가 유지되었습니다.
그 플래그의 일반 판정은 셀 로컬 줄이 본문 절반 높이에 도달한 프레임을 대상으로 하므로,
43쪽 아래에서 시작하는 짧은 첫 프레임은 해당하지 않았습니다.
두 번째 시도는 저장 첫 object frame이 끝나는 행과 같은 행의 기존 CellUnit hard-break를
함께 확인합니다. 편집되지 않은 원본 HWP5, 비재조판, 단순 글 문단 조건을 유지합니다.
원시 vpos reset 하나만으로 임의의 로컬 프레임을 쪽 경계로 승격하지 않습니다.

## 두 번째 수정 결과

- 기존 본문 넘침 partition 4: 수정 전 FAIL, 두 번째 수정 후 PASS.
- 국가 표의 원본 행 소속 검사 두 개: 수정 전·후 모두 PASS.
- 합계 3 PASS / 0 FAIL, 실행 2.591초. 기대값과 baseline 변경, 새 검사 추가 없음.
- 정확한 devel 299쪽 / 독립 PDF 302쪽 불일치로 base 전체 TSV 생성은 보류되었습니다.
  base 43쪽 review는 67.93691%이며 이를 302쪽 전수 통과로 표현하지 않습니다.
- 수정 후 전체 Native/fresh WASM TSV와 실제 PNG는 다음 검증 단계에서 확인합니다.
  현재 커밋은 전체 PR 제출 가능 판정이 아닙니다.
- 정본 증적: `mydocs/pr/assets/issue7445/completed-20261009/roadmap-source-boundary-validation.json`.
