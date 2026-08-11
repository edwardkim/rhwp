# Task #4515 Stage 1 — LAYOUT_TABLE_OVERLAP 진단 구현

Issue: #4515 (LAYOUT_OVERFLOW 진단이 표 겹침을 검출하지 못함 — 경고 페이지와 실제 결함
페이지의 교집합 0). 결함 표본은 #4514 의 `sample1-repro.hwp`.

## 문제

`LAYOUT_OVERFLOW` 는 `build_single_column` 의 자가 검증에서 항목 하단이
`col_bottom + 2px` 를 **초과**할 때만 기록된다. #4514 처럼 겹친 표들의 하단이 본문
하단(1046.9px)으로 clamp 되는 경우 초과량이 0 이라 한 건도 잡히지 않았다 —
devel HEAD(8ea92cdad) 실측: 경고는 16·42쪽(PartialTable 31.0/6.1px)뿐, 실제 겹침
6쪽(8·12·13·22·25·29, 최대 555.5px)과 교집합 0.

## 구현

겹침은 하단 초과가 아니라 **형제 요소 y 구간 중첩**이므로 별도 진단 축을 추가했다.

- `src/renderer/layout.rs`
  - `LayoutTableOverlap` 구조체 + `LAYOUT_TABLE_OVERLAP: page=…, para_a=…, para_b=…,
    a=…~…, b=…~…, overlap=…px` stderr 경고 (기존 `LAYOUT_OVERFLOW` 채널과 동형).
  - `collect_top_level_table_spans`: 페이지 루트에서 **Page 직계 Table**(글앞/글뒤·용지
    기준 overlay z-layer)과 **Body→Column 직계 Table**(본문 흐름 표)을 한 집합으로
    수집. Cell 하위로 내려가지 않아 셀 안 중첩 표는 자연 제외(이슈의 오탐 방지 조건).
    비가시 노드 제외. — render tree 실측상 overlay 표는 Column 이 아니라 Page 직계로
    붙으므로(8쪽: pi=102·119·139 가 Page 직계, pi=99·118 이 Column 직계) 단 수준
    검사로는 도메인이 갈라진다. 페이지 조립 완료 시점(`build_render_tree` 말미,
    paper z-layer 부착 이후)에 검사하는 이유다.
  - `detect_table_overlaps`: y 시작 정렬 후 인접 쌍의 `위 표 하단 − 아래 표 상단 >
    2.0px` 판정. 임계 2px 이하는 테두리 접합 오차(이슈 46문서/491표 실측 + sample1
    20쪽 1.7px 확인).
  - 엔진에 `layout_table_overlaps` 누적 + `take_table_overlaps()` 조회·리셋.
- `src/document_core/queries/rendering.rs`
  - `DocumentCore::take_table_overlaps()` 위임 (테스트·소비자용,
    `take_overflow_cell_lines` 와 동형).

## 검증

- 단위 (`src/renderer/layout/tests.rs`):
  `detect_table_overlaps_flags_only_above_threshold` (임계 경계·정렬 복원·#4514 8쪽
  실측 좌표 3쌍), `collect_top_level_table_spans_domain` (Page 직계+Column 직계 포함,
  중첩 표·비가시 제외) — 통과.
- 통합 (`tests/issue_4515_table_overlap_diag.rs` + `samples/issue4514/sample1-repro.hwp`):
  47쪽 전 페이지에서 `take_table_overlaps()` 가 render tree JSON 독립 재계산과
  페이지·쌍 단위 일치 — 통과 (0.52s). #4514 가 수정되면 양쪽이 함께 0 이 되는
  자기일관 단언이라 픽스처 갱신이 필요 없다.
- CLI 실측 (debug bin, `export-render-tree`): 0-based 7·11·12·21·24·28쪽에서
  `LAYOUT_TABLE_OVERLAP` 8건 발생 = 이슈의 실제 결함 6쪽(1-based 8·12·13·22·25·29)과
  정확히 일치, 20쪽 1.7px 는 미보고(오탐 0). 기존 `LAYOUT_OVERFLOW` 2건(16·42) 불변.
- 관측 전용 확인: 진단 추가 전(devel HEAD release)·후(본 브랜치 debug) render tree
  JSON 47쪽 전부 바이트 동일 — 레이아웃 동작 무변경.
- `cargo clippy --lib` 경고 0. 변경 파일 rustfmt 적용.
- 미실행(승인 대기): release-test 전체, Native Skia 3종, wasm-pack build.

## 남은 것

- #4514 원인 수정은 별도 트랙 (본 진단은 검출 축만 추가). 이 진단이 게이트에 들어가면
  #4514 수정 PR 의 회귀 가드로 바로 쓸 수 있다.
