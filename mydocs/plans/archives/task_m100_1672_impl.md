# Task M100 #1672 구현 계획서 — RowBreak 표 페이지네이션 과대 분할 조사

## 구현 방침

우선 페이지 수 차이를 만드는 표 분할 단위를 좁힌 뒤, RowBreak 표 처리에 국소 수정한다.

현재 관찰상 기본 경로 416쪽 중 최종 발행정보 페이지가 PDF 383쪽과 대응한다. 따라서 페이지 순서 자체가 깨진 것이 아니라 중간에서 33쪽이 추가 생성된다. 후반부에는 `PartialTable` continuation 페이지가 집중되어 있으므로 표 분할 경로를 1차 수정 대상으로 둔다.

2026-06-30 중간 구현 후에는 HWP 388쪽, HWPX 390쪽까지 줄었다. 아직 한컴 PDF 기준 383쪽에는 도달하지 않았으므로, 이번 변경은 최종 정합 완료가 아니라 누적 과대 페이지네이션을 줄인 중간 단계로 다룬다.

## 조사 대상 코드

`src/renderer/typeset.rs`

- `typeset_table` 계열
- `split_table_rows` 계열
- `TABLE_SPLIT_AVAIL` 디버그 로그 주변
- `start_cut` / `end_cut` / `is_block_split` 전파

`src/renderer/height_measurer.rs`

- `MeasuredTable`
- `remaining_content_for_row`
- `is_row_splittable`
- `find_break_row`

`src/renderer/layout/table_layout.rs`

- `advance_row_cut`
- `advance_row_block_cut`
- `block_fragment_height`
- RowBreak 표의 cell unit 구성

`src/renderer/pagination/engine.rs`

- 구형 Paginator의 `split_table_rows`
- TypesetEngine과 비교할 기준 구현

## 구현 단계

### Stage 1 — 문제 표 inventory

다음 정보를 출력/수집한다.

- `PartialTable` 발생 페이지별 `(section, page, pi, ci, rows, start_cut, end_cut)`
- Paginator 대비 TypesetEngine에서 페이지가 더 늘어나는 표 목록
- `used=0.0px` continuation 페이지가 생기는 표 목록
- 문제 표의 셀 단위 `vpos=0` 리셋, 빈 문단, RowBreak attr 조합

산출은 `output/poc/task1672/` 아래에 둔다.

### Stage 2 — 최소 정정 후보 검증

후보를 좁혀 하나씩 검증한다.

1. RowBreak 셀 내부 `vpos=0` 리셋을 hard break로 보는 조건 완화
2. 빈 또는 시각적으로 비가시인 cell unit이 continuation 페이지를 독립 생성하지 않도록 가드
3. `start_cut`/`end_cut`이 진행하지 않는 continuation 반복 차단
4. TypesetEngine과 Paginator의 RowBreak first-fragment 예산 차이 보정

수정은 샘플명 조건 없이 문서 구조 조건으로 제한한다.

현재 반영된 후보:

- RowBreak 표의 hard-break 완화를 모든 RowBreak 표에 적용했다.
- 텍스트 문단의 `height_for_fit`에 저장 `LINE_SEG` vpos span 상한을 적용했다. 컨트롤이 있는 문단은 #1156 회귀 위험 때문에 제외했다.
- `LAYOUT_DRIFT_SAFETY_PX`를 0px 로 낮췄다.
- RowBreak 마지막 빈 spacer 행이 제한 범위 안에서 overflow 하면 별도 꼬리 페이지 대신 직전 조각에 흡수한다.
- 1단 구역 끝의 빈 단나누기 문단은 후속 단/밴드가 없으면 건너뛴다.
- 글앞/글뒤 비-TAC overlay 표 뒤의 빈 guide 문단은 다단 flow 높이를 소비하지 않는다.
- 단일단 일반 문단의 flow advance 에도 저장 `LINE_SEG` vpos 상한을 제한적으로 적용한다. `spacing_after`가 있거나, RowBreak 표 바로 주변이 아닌 `spacing_before` 전용 문단만 대상이다.
- landscape RowBreak continuation 은 반복 머리행이 있고 짧은 꼬리 행/whole-row 가 소폭 넘치는 경우에만 직전 조각 흡수를 허용한다. portrait RowBreak 표까지 넓히면 `issue_rowbreak_chart_overlap` 회귀가 발생해 제외했다.

남은 차이 추적:

- 구역 10 Q&A: 시작 지점이 PDF 279쪽 대비 rhwp 281쪽으로 +2쪽, 후반은 PDF 307쪽 대비 rhwp 310쪽으로 +3쪽.
- 구역 11 부록 법령 비교 RowBreak 표: `부록` 시작이 PDF 309쪽 대비 rhwp 312쪽, 후반부 법령 페이지가 PDF 378~382쪽 대비 rhwp 383~387쪽.
- 구역 12 별표/별지 양식: `[별표 4]` 후반과 `별지 제8호서식`에서 PDF 대비 +4~+5쪽이 남는다.
- 최종 HWP 기준 잔여 차이는 +5쪽, HWPX 기준 잔여 차이는 +7쪽이다.

### Stage 3 — 회귀 테스트 추가

테스트 파일 후보:

- `tests/issue_1672_admin_manual_pagination.rs`

테스트 내용:

- HWP 페이지 수 383 단언
- HWPX 페이지 수 383 단언
- 필요 시 후반부 최종 발행정보 텍스트가 마지막 페이지에 있는지 확인
- 과도한 `PartialTable` continuation 페이지 수에 대한 보조 단언

현 단계에서는 아직 목표 383쪽에 도달하지 않았으므로 위 fixture 테스트는 추가하지 않는다. 대신 기존 RowBreak 회귀 테스트로 변경의 blast radius 를 먼저 확인한다.

### Stage 4 — 검증

우선 focused 검증:

```bash
cargo test --profile release-test --test issue_1488_rowbreak_empty_overlay_pages
cargo test --profile release-test --test issue_rowbreak_chart_overlap
cargo test --profile release-test --test issue_1156_rowbreak_fragment_fit
git diff --check
```

목표 383쪽 도달 후에는 `issue_1672_admin_manual_pagination` fixture 테스트를 추가해 함께 실행한다.

PR 준비 단계에서는 작업지시자 승인 후 macOS 로컬 검증 기준에 맞춰 범위를 넓힌다.

2026-06-30 현재 focused 검증 대상:

```bash
cargo test --profile release-test --test issue_1156_rowbreak_fragment_fit -- --nocapture
cargo test --profile release-test --test issue_1488_rowbreak_empty_overlay_pages -- --nocapture
cargo test --profile release-test --test issue_rowbreak_chart_overlap -- --nocapture
git diff --check
```

2026-06-30 검증 결과:

- `cargo build --release`: 통과
- `cargo test --profile release-test --test issue_1156_rowbreak_fragment_fit -- --nocapture`: 통과, 3 passed
- `cargo test --profile release-test --test issue_1488_rowbreak_empty_overlay_pages -- --nocapture`: 통과, 1 passed
- `cargo test --profile release-test --test issue_rowbreak_chart_overlap -- --nocapture`: 통과, 20 passed
- `git diff --check`: 통과
- 릴리스 바이너리 기준 페이지 수: HWP 388쪽, HWPX 390쪽

## 승인/보류 사항

- 2026-06-30 작업지시자가 승인해 Stage 2 소스 수정을 진행했다.
- 회귀 fixture 로 현재 미추적 `samples/2025 행정업무운영 편람(최종).hwp`, `samples/2025 행정업무운영 편람(최종).hwpx`를 포함할지는 아직 보류한다.
- PDF oracle 저장소 포함 여부도 파일 크기와 기존 정책을 보고 별도 판단한다.
