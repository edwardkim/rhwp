# Task #1261 Stage2 계획 - 2024 9월 문8 겹침 원인 조사

## 배경

Stage1에서 `samples/3-10월_교육_통합_2022.hwp` 5쪽 `문28)` 조건 박스 겹침을 수정하고 커밋했다.
작업지시자는 다음 자료를 기준으로 2024년 9월 문8 겹침 원인을 다음 스테이지에서 확인하라고 지시했다.

- 한컴 정답지 PDF: `pdf-large/3-09월_교육_통합_2024-미주사이20-2024.pdf`
- 작업지시자 화면 기준 위치: `3-09월_교육_통합_2024-미주사이20.hwp` 10쪽 부근 `문8)`

## 목표

1. `문8)` 겹침이 Stage1의 TAC Shape 높이 문제와 같은 원인인지, 별도 수식/줄높이/다단/미주 흐름 문제인지 분리한다.
2. 한컴 정답지 PDF와 rhwp 렌더링 좌표를 비교해 겹침이 시작되는 문단과 라인 인덱스를 찾는다.
3. 구현 전 단계에서 수정 후보와 회귀 테스트 후보를 문서화한다.

## 조사 절차

- 기준 HWP/HWPX 샘플 파일 위치를 확인한다.
- `dump-pages`로 rhwp 기준 10쪽 전후의 `문8)` 문단과 컨트롤 인덱스를 찾는다.
- `dump`로 대상 문단의 `ParaShape`, `LINE_SEG`, 수식 컨트롤, 글자처럼 취급되는 객체 속성을 확인한다.
- 한컴 정답지 PDF에서 같은 영역을 이미지로 추출해 rhwp SVG/PNG와 비교한다.
- 겹침 원인이 확인되면 수정 전 구현 계획과 검증 명령을 별도 기록하고 작업지시자 승인을 받는다.

## 주의

- 이 스테이지에서는 원인 조사와 문서화만 수행한다.
- 소스 수정은 작업지시자 승인 후 진행한다.
- `pdf-large/`의 PDF 파일은 기준 자료로 사용하되, 별도 지시가 없으면 커밋하지 않는다.

## 1차 조사 결과

### 재현/비교 산출물

- rhwp SVG: `output/task1261_stage2_2024_page10/3-09월_교육_통합_2024-미주사이20_010.svg`
- rhwp PNG: `output/task1261_stage2_2024_page10_compare/rhwp_page10.png`
- 한컴 PDF PNG: `output/task1261_stage2_2024_page10_compare/hancom_page-10.png`

한컴 PDF 기준 10쪽 왼쪽 단에서는 문7 마지막 수식 묶음 아래에 충분한 여백이 있고, 그 아래에서 `문8） ①`이 시작한다.
rhwp 기준 SVG/PNG에서는 직전 분수 수식 묶음이 `문8） ①` 제목 영역까지 내려와 겹친다.

### 페이지네이션 상태

`dump-pages samples/3-09월_교육_통합_2024-미주사이20.hwp -p 9` 기준:

- 10쪽 왼쪽 단은 `pi=517..540`으로 구성된다.
- `pi=522`는 문7의 마지막 큰 수식/빈 미주 문단이다.
- `pi=523`은 `문8）   ①` 제목 문단이다.
- 기존 회귀 조건처럼 문8 자체는 9쪽이 아니라 10쪽에서 시작한다.

따라서 이번 겹침은 문8의 페이지 진입 위치 문제가 아니라, 10쪽 첫 단 내부에서 `pi=522` 렌더 후 `pi=523` 시작 y가 위로 되감기는 문제다.

### 렌더러 로그

`RHWP_DEBUG_TAC_CURSOR=1` 결과:

```text
TAC_CURSOR  FullPara pi=522 y_in=260.6 y_out=437.4 dy=176.8 was_tac=false
TAC_CURSOR  FullPara pi=523 y_in=371.8 y_out=389.9 dy=18.0 was_tac=false
```

`pi=522`는 실제로 `y=437.4`까지 렌더되었는데, 다음 `pi=523`은 `y=371.8`에서 시작한다.
즉 다음 문항 제목이 직전 수식 문단의 실제 렌더 하단보다 약 65px 위로 되감긴다.

`RHWP_VPOS_DEBUG=1` 결과 `pi=523`에서 `compact_new_note=true`가 켜진다.
`HeightCursor::vpos_adjust()`의 compact endnote 새 문제 제목 cap이 자연 목표 `end_y=480.29` 대신
직전 수식 줄의 `line_spacing`을 뺀 추정 하단 `prev_content_bottom_y + 10px`를 사용하면서 `371.8`로 낮춘다.

### 원인 판단

원인은 Stage1의 TAC Shape 높이 문제가 아니다.

현재 원인은 `src/renderer/height_cursor.rs`의 compact endnote 새 문제 제목 보정이 직전 display 수식의 보이는 하단을
`y_offset - prev_line_spacing`으로 추정하는 데 있다. 문7 마지막 수식처럼 분수식이 큰 줄에서는 실제 수식 글리프가 이 추정 하단보다 더 아래까지 내려오므로,
`문8）` 제목이 실제 수식 위에 겹쳐 배치된다.

### 구현 후보

가장 좁은 수정 후보:

- `layout.rs`에는 직전 문단 렌더 후 실제 콘텐츠 하단을 담는 `last_item_content_bottom`이 이미 있다.
- 다음 item의 `vpos_adjust()`를 호출한 뒤, 현재 item이 compact endnote 새 문제 제목이고 결과 y가 직전 실제 콘텐츠 하단보다 작으면
  `last_item_content_bottom + 최소 gap` 이상으로 클램프한다.
- 기존 `height_cursor.rs`의 `문13`류 소-gap/backtrack 정책은 유지하되, 실제 렌더 콘텐츠를 침범하는 되감김만 차단한다.

수정 전 검증 후보:

- `3-09월_교육_통합_2024-미주사이20.hwp` 10쪽에서 `pi=523` 시작 y가 `pi=522` 실제 콘텐츠 하단보다 아래인지 단언한다.
- 기존 `issue_1139_exam_2022_page_count_matches_hancom_after_endnotes`와 `issue_1139_inline_picture_duplicate` 전체 테스트를 반드시 재실행한다.

## 구현

`src/renderer/layout.rs`에서 compact endnote 새 문항 제목의 vpos 되감김을 실제 직전 콘텐츠 하단 기준으로 제한했다.

- `HeightCursor::vpos_adjust()` 자체의 저장 vpos 정책은 유지했다.
- 렌더러가 이미 기록하는 `last_item_content_bottom`을 사용해, 새 미주 문항 제목이 직전 실제 콘텐츠 하단보다 위로 올라가는 경우만 보정했다.
- 적용 조건은 다음으로 좁혔다.
  - 현재 항목이 미주 새 문항 제목이다.
  - 직전 항목이 실제 텍스트 또는 글자처럼 취급되는 수식/그림/도형/표를 가진다.
  - 직전 유효 `LINE_SEG.line_height`가 큰 display 수식 줄이다.
  - lazy-base vpos 보정 흐름이다.
- 이 조건으로 page-base safe backtrack, 빈 spacer 뒤 문항, 의도적 소-gap 케이스는 그대로 둔다.

`tests/issue_1139_inline_picture_duplicate.rs`에 다음 회귀 테스트를 추가했다.

- `issue_1261_2024_sep_page10_question8_stays_below_previous_equation`
  - `samples/3-09월_교육_통합_2024-미주사이20.hwp` 10쪽에서 `pi=523` 문8 제목이 `pi=522` 실제 콘텐츠 하단보다 아래에서 시작하는지 단언한다.

## 구현 후 확인

재현 로그 기준:

```text
TAC_CURSOR  FullPara pi=522 y_in=260.6 y_out=437.4 dy=176.8 was_tac=false
TAC_CURSOR  FullPara pi=523 y_in=447.4 y_out=465.5 dy=18.0 was_tac=false
```

기존 문제였던 `pi=523 y_in=371.8` 되감김이 사라지고, 문8 제목이 직전 수식 하단 아래에서 시작한다.

시각 산출물:

- rhwp SVG: `output/task1261_stage2_final_page10/3-09월_교육_통합_2024-미주사이20_010.svg`
- rhwp PNG: `output/task1261_stage2_final_page10_compare/rhwp_page10.png`

PNG 확인 결과 문8 제목은 직전 분수 수식 묶음과 분리되어 렌더된다.

자동 검증:

- `cargo fmt -- --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 45개 통과
- `cargo test --lib height_cursor -- --nocapture`: 31개 통과
- `git diff --check`: 통과

WASM/Studio 시각 검증:

- 작업지시자가 수동으로 `wasm-pack build --target web --out-dir pkg`와 `localhost:7700` 시각 검증을 수행하기로 했다.
- Codex는 `pkg/package.json` 등 `pkg/` 산출물을 수정 대상으로 포함하지 않는다.
- 2026-06-03 작업지시자가 수동 WASM/Studio 화면에서 문8 겹침 해소를 확인했다.
- 작업지시자 지시에 따라 `pdf-large/`의 2024년 9월 기준 PDF를 커밋 범위에 포함한다.

## 남은 확인

- 10쪽 오른쪽 단 하단에는 기존 `LAYOUT_OVERFLOW` 진단 로그가 남는다. 이번 Stage2는 왼쪽 단 문8 겹침 원인만 좁게 수정했으므로, 오른쪽 단 하단 overflow는 별도 후속 판단 대상으로 남긴다.
