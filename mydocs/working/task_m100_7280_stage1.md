# Task #7280 Stage 1 — 기준 계측과 문단 중심 책임 지도

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 일자: 2026-09-20
- 수행계획: [task_m100_7280.md](../plans/task_m100_7280.md)
- 상태: 1단계 완료. 구조·복잡도·정책 조사 및 기본 feature Rust 회귀 기준 확정.
- 제품 코드·테스트 원본·기준값 변경 없음. 구현계획 승인 전 구현에 착수하지 않는다.

## 작업지시자 확인 — 구조 분리와 변경 관리에 집중

이번 리팩토링은 조판 문제를 해결하는 작업이 아니다. 다양한 기여자가 조판 코드를
추가·수정·삭제할 때 책임 모듈, 입력/결과 계약, 상태 소유권, 의존 관계 및 검증 범위를
명확히 알 수 있도록 실제 코드 구조와 관리 절차를 마련하는 작업이다.

아래 책임 지도와 중복 판단 경로 조사는 이 구조를 설계하기 위한 자료다. 발견한 기존
조판 오류나 한컴과의 차이를 이번에 모두 고치는 목록으로 해석하지 않는다.
회귀·시각 검증은 고정 baseline 대비 동작 보존을 확인하며, 기존 문제는 후속 대상으로
분리하고 리팩토링에서 새로 생긴 변경은 이번 작업에서 해소한다.
구체적인 기여자 완료 기준은 [수행계획 §1](../plans/task_m100_7280.md#1-목적과-경계)을 따른다.

## 1. 기준과 증거 범위

- 고정 devel: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 작업 브랜치: `task_m100_7280`. 승인 계획 커밋: `32caa4b28285be9d7985b26dcc7d3ff03bd3d9ea`.
- 기준 검증 worktree: `/home/edward/mygithub/rhwp-review-7280-base`, 위 devel SHA의 detached HEAD.
- 검증 target: `/home/edward/mygithub/rhwp/target/pr-review`. Cargo 명령은 순차 실행한다.
- 조사 산출물: `output/7280/stage1/`. ignored 로컬 증적이며 제출 원본이 아니다.
- `#7195` 작업 브랜치 및 기존 worktree는 변경하지 않았다. 미병합 수정은 포함하지 않는다.

아래 소스 위치는 고정 SHA 기준이다. 대표 호출 경로와 상태 소유권 조사이며, 모든 분기의
조판 규칙을 검증하거나 한컴과의 시각 일치를 판정한 결과가 아니다.

## 2. 복잡도 재계측

native 기본 target에 Clippy cognitive complexity 진단 임계값 5를 적용했다.
WASM 전용 cfg, 미컴파일 feature 및 테스트 함수 전체를 포함하는 계측은 아니다.
진단 경고를 얻은 exit 0은 제출용 `-D warnings` lint 통과를 뜻하지 않는다.

```bash
CLIPPY_CONF_DIR=/home/edward/mygithub/rhwp/output/7280/stage1/clippy-config \
CARGO_BUILD_JOBS=4 cargo clippy --locked --target-dir target/pr-review \
  --message-format=json -- -W clippy::cognitive_complexity
```

- `complexity.jsonl`, `complexity.stderr`: 원시 진단.
- `summarize-complexity.mjs`, `complexity-summary.json`: primary span 중복 제거와 요약.
- 요약 대상 7개 소스는 고정 devel의 내용과 일치함을 확인했다.
- 보고된 CC 항목 1,604개, CC > 25 항목 114개, 최대 186. 전체 함수 수를 뜻하지 않는다.
- `typeset.rs` 31,937줄. 이전 `18a9fa85…` 계측 31,968줄과 기준 SHA가 다르다.

| 함수/클로저 (`src/renderer/typeset.rs`) | 시작 줄 | CC | 현재 책임 |
| --- | ---: | ---: | --- |
| `typeset_section_with_variant` | 7661 | 186 | 구역 흐름 조정·규칙 선택·상태 적용 |
| `typeset_block_table_inner` | 24508 | 129 | 표 최초 배치와 분할 세션 준비 |
| `typeset_endnote_paragraphs` | 11191 | 124 | 미주 문단 흐름·배치 |
| `scan_block_table_split_rows` | 22850 | 102 | 행/셀 컷과 수용 높이 후보 계산 |
| `typeset_table_paragraph` | 20081 | 85 | 소유 문단과 컨트롤의 배치 경로 선택 |
| `typeset_paragraph` | 17941 | 75 | 문단 fit·분할·상태 적용 |
| continuation step 내부 클로저 | 27341 | 60 | 컷·각주·페이지·커서 전이 |
| `place_table_with_text` | 22030 | 50 | 텍스트와 표의 혼합 배치 |
| `format_paragraph_for_flow` | 17360 | 30 | 문단 구성·저장 줄 정보 처리 |

복잡도만으로 추출 순서를 결정하지 않는다. 작은 변경으로 경계 계약을 보존할 수 있는지가 우선이다.

## 3. 문단 중심 IR에서 출력까지

```text
HWP parse_paragraph / HWPX parse_paragraph_element
  → Section.paragraphs → Paragraph(text, char_offsets, controls, line_segs, 서식)
       └─ Table.cells → Cell.paragraphs → 중첩 컨트롤
  → composer: ComposedParagraph / ComposedLine
  → HeightMeasurer: MeasuredParagraph / MeasuredTable / MeasuredCell
  → TypesetEngine: 문단·컨트롤 흐름 및 페이지 분할
  → PaginationResult / PageItem / 일부 경로의 InlineFlowPlan
  → LayoutEngine: RenderNode 생성 및 일부 기하 재계산
  → 출력 backend
```

이는 대표 데이터 흐름이며 단방향 의존 구조가 이미 완성됐다는 뜻은 아니다.
현재 typeset도 LayoutEngine의 셀 유닛·컷 측정 메서드를 호출한다.

### IR의 소속과 위치

- `src/model/document.rs:275`의 Section은 문단 목록을 소유하고,
  `src/model/table.rs:112`의 Cell은 다시 문단 목록을 소유한다.
- `src/model/paragraph.rs:17`의 Paragraph에 텍스트, UTF-16/control 슬롯 대응용
  `char_offsets`, 글자모양, `controls`, 저장 LineSeg 및 출처·보강 정보가 있다.
  배열들이 분리돼 있다는 이유만으로 IR에 컨트롤 삽입 위치가 없다고 판단하면 안 된다.
- `Paragraph::control_text_positions`(1806)은 offset 차이에서 삽입 위치를 구한다.
  offset 없는 fallback은 정밀도가 제한되는 경로로 별도 검토해야 한다.
  편집용 `logical_control_positions`와 줄 구성용 위치를 같은 축으로 취급하지 않는다.
- HWP 파서 `src/parser/body_text.rs:270`은 문단 헤더 아래 텍스트·글자모양·줄·컨트롤을 읽고,
  HWPX 파서 `src/parser/hwpx/section.rs:600` 이후는 문단/run/컨트롤을 공통 Paragraph로 만든다.
- composer는 `control_text_positions`와 `line_seg_text_start` 등 기존 접근자를 사용한다.
  HWPX `hwpx_axis_shift`, 저장 vpos, 조판 전용 보강 줄, 편집 후 무효화를 보존해야 한다.

**설계 시 우선순위:** 기존 IR 접근 계약을 명시하고 필요한 읽기 전용 관점을 검토한다.
새 공통 IR 스키마나 원본 저장 형식 변경부터 시작하지 않는다. 소유 문단과 실제 배치의
쪽/단/문단 기준 좌표, TAC/float 속성은 각각 유지한다.

## 4. Query / Command / 조정 책임 지도

| 현재 진입점 | 읽는 주요 입력 | 상태/결과 | 설계에서 다룰 경계 |
| --- | --- | --- | --- |
| `compose_paragraph` (`composer.rs:333`) | Paragraph 내용·저장 줄/컨트롤 관계 | ComposedParagraph | 기존 구성 결과의 의미·인덱스 계약 |
| `HeightMeasurer::measure_paragraph/measure_table_impl` | 구성 결과·서식·표/셀 | Measured 계열 | 측정 결과와 실제 배치 소비자의 일치 |
| `format_paragraph_for_flow` | 문단·구성·서식·profile/영역 | FormattedParagraph | Query 후보. 내부 Cell/RefCell 및 환경 조건을 함께 검토 |
| `scan_block_table_split_rows` | 페이지 상태 관측·행 기하·컷·가용 높이 | BlockTableRowScan | Query 후보. LayoutEngine 캐시와 의미 상태 변경을 구분 |
| `typeset_paragraph/typeset_table_paragraph` | 문단·컨트롤·현재 영역 | 후보 계산과 PageItem/커서 변경 혼재 | 계산과 확정 적용을 분리할 조정 경계 |
| `step_block_table_continuation` | 원본/유효 행 표·측정·prepared state | 컷, 각주 큐, 페이지 항목, continuation 전진 | 확정 fragment 적용과 다음 후보 계산 분리 |
| `TypesetState::advance_column/push_new_page/reset_for_new_page` | 현재 영역·각주·반복 상태 | 페이지/단 상태 변경 | Command 경계, reset 시 보존/해제 항목 명시 |
| `LayoutEngine::layout_partial_table_resolved` | PageItem 컷·원본/유효 표·측정 결과 | RenderNode 및 행 높이 재계산 | 출력 기하와 페이지 분할 결과 사이의 계약 |

`&self`라는 서명이나 `queries/`라는 디렉터리 이름만으로 순수 Query라고 분류하지 않는다.
예를 들어 `document_core/queries/rendering.rs`의 `recompose_paragraph`(4267)은 구성 결과를
갱신하며, 같은 파일의 측정·typeset 호출(5116–5255) 뒤에는 후처리도 있다.
따라서 TypesetEngine 하나만을 최종 페이지 상태 소유자로 가정할 수 없다.

### 이미 있는 상태 분리와 공통 결과

- `TypesetState`(1086): 본문 높이, 페이지/단, float 예약, 각주, 저장 좌표 등의 큰 상태.
- `TableContinuationCursor`(106): 행·컷·남은 물리 높이·각주 tail 등 진행 상태.
- `BlockTableContinuationPreparedState`(176), source/context 및
  `ResumableTablePaginationJob`(263): 준비 결과와 재개 가능한 진행 경계가 이미 있다.
  native/WASM 재개 경로를 무시하고 별도 상태 머신을 중복 신설하지 않는다.
- `FormattedParagraph`(5793), `ComposedLine`, `MeasuredTable` 등은 재사용 우선 대상이다.
- `InlineFlowPlan`은 일부 경로에서 이미 공통 결과다. `typeset/inline_flow.rs`에서
  계획을 저장하고 `layout.rs:9219`의 FullParagraph 경로가 같은 plan의 start/end/box를 소비한다.
  다만 이 경로가 모든 문단·표를 포괄하지 않으며 적용 함수에는 페이지 상태 변경도 있다.

## 5. 표 분할 결과 전달을 실제로 추적한 사례

1. `scan_block_table_split_rows`는 소비 높이, 끝 행, 끝 컷, 물리 높이 override 등을 반환한다.
2. continuation step은 결과에서 `PageItem::PartialTable`을 만든다(`typeset.rs:28322`).
   원본 문단/컨트롤, 시작/끝 행·컷, 시작 컷의 인덱스 공간, 유효 중첩 행 도메인,
   물리 높이 override를 싣는다. 이어 current_height와 continuation을 변경한다.
3. `pagination.rs:589`의 PartialTable은 이 계약을 명시한다. `is_block_split`은 legacy 복합 의미이고,
   시작 컷은 `start_cut_is_block`을 따르므로 이름만 정리하며 의미를 바꾸면 안 된다.
4. `layout/table_partial.rs:3803`은 `row_cursor_is_nested`에 따라 원본 또는 유효 중첩 표를 선택한다.
   이후 기본 행 높이를 만들고(4058 이후), 컷 범위와 rowspan/중첩 조건에 따라
   `row_cut_content_height`로 일부 높이를 다시 계산한다(4187–4233 부근).

**관찰:** 이미 컷/도메인 전달은 명시적으로 개선된 부분이 있다. 그러나 최종 행 기하 전체를
동일 결과로 전달하는 구조는 아니며, layout 쪽에 측정 결과 유지/재계산 정책이 남아 있다.
이는 책임 중첩의 근거이지 이번 조사에서 새 회귀를 재현했다는 뜻은 아니다.
리팩토링에서는 기존 재계산을 무조건 삭제하지 않고 입력·출력·반례를 먼저 고정한다.
내용 소비 범위와 내용 없는 물리 밴드의 남은 높이도 합치지 않는다.

## 6. 테스트 기준과 현재 실행

기준 worktree에서 다음 정책 검사는 모두 exit 0이다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
```

- manifest: 1,382 sources / 28 suites + 20 exceptions = 48 integration targets.
- source-side inventory: 4,205 static tests / 298 modules.
- 이 숫자는 실행·통과 건수가 아니다. 파생 suite·manifest는 review worktree에만 준비했다.

실제 기본 feature 회귀를 다음 명령으로 실행하여 exit 0을 확인했다.
결과는 `output/7280/stage1/nextest.log`다.

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  --tests --no-fail-fast
```

- rustc 1.93.1, Node v24.15.0, cargo-nextest 0.9.137. 저장소 권장 0.9.140과 다르며
  `profile.ci-duration-observation.junit.report-skipped`를 모르는 설정으로 경고했다.
  이 실행은 기본 profile의 로컬 회귀이며 CI와 도구 버전까지 동일하다고 보고하지 않는다.
- 기준 빌드 7분 57초, 실행 442.237초. **10,096건 실행 / 10,096건 통과 / 실패 0건 /
  기존 제외 50건**, 78 binaries, slow 표시 11건이다.
  nextest run ID: `b7401d53-71bd-4436-9034-f2958f495e6a`.
  장시간 실행된 `issue_2063::huge_cellbreak_table_paginates_without_quadratic_blowup`도 통과했다.
  timeout/ignore/pin은 바꾸지 않았다. 이 결과는 리팩토링 전의 동작 기준이며 한컴 피델리티 판정이 아니다.
- GitHub CI archive 분할 실행, WASM/native-Skia feature, 전체 workspace lint, 시각 검증은
  이번 결과에 포함하지 않는다. 변경 head 제출 단계에서는 적용 게이트를 별도로 실행한다.

### 추출 전에 보존할 기존 계약 예

- `tests/cases/issue_6935_row_space_start_cut_straddle.rs`: 시작/끝 컷의 인덱스 공간 분리.
  합성 입력의 계약 테스트이며 그 자체를 한컴 실물 시각 증거로 보지 않는다.
- typeset source-side의 `issue2424_table_continuation_cursor_preserves_break_state`,
  `issue2424_block_table_context_owns_step_lifecycle`: 재개 상태 수명.
- `test_measure_endnote_advance_side_effect_free`: 계산 중 상태 변화 방지.
- 기존 stored-vpos/빈 문단/page-bottom 테스트: 옮기는 구조와 규칙 의미 변경을 분리한다.

기존 `assert_pagination_match`(`typeset.rs:30557`)는 페이지 수·단 수·항목 수를 비교한다.
이 검사가 통과해도 같은 원본 내용 범위·좌표·출력까지 같다는 의미는 아니다.
구현계획에서는 이동 대상의 컷·소유권·점유 기하와 같은 의미 계약을 별도로 연결한다.
기존 테스트가 무효하다는 뜻이 아니라, 검사하는 범위 이상으로 결과를 해석하지 않는 것이다.

`rust-unit-test-tiers.mjs:423`의 기존 모듈 대응은 ID 일치 또는 Git rename과 동일 suffix다.
파일 일부 추출은 새 테스트 모듈로 인식될 수 있다. 구현계획에 private 테스트의 유지 위치와
접근 범위를 포함하며 테스트 삭제·일괄 public 노출·정책 상향으로 우회하지 않는다.

## 7. 구현 설계로 넘길 판단

1. 문단 IR 접근·좌표·저장 줄 유효성 계약을 먼저 명시하고, 기존 공통 결과를 연결한다.
2. 상태 전이와 독립적인 입력/결과 경계를 작은 단위로 정한다. 단순 파일 이동으로 끝내지 않는다.
3. 문단, 표 fragment, float, 각주·미주의 순서와 소유권을 조정 계층에 명시한다.
4. 기여자 안내는 규칙별 입력·조건·반례·구현 위치·소비자·검증으로 연결한다.
   표 규칙은 기존 `table_layout_rules.md` 정본을 사용하며 별도 중복 정본을 만들지 않는다.
   각 모듈의 책임/비책임·허용 의존·가시성과 추가·수정·삭제 시 확인할 호출부·테스트·문서를
   구현계획에 포함한다. 관리 체계가 실제 코드 경계와 대응하는지 검토한다.
5. HWPX vpos가 항상 0이라는 과거 설명 등은 현재 파서/IR 계약과 대조해야 한다.
   LineSeg 문서의 높이·줄간격과 기준 좌표 설명도 구현에 사용하기 전 모순을 해소한다.
6. 기준 테스트 결과를 확정한 후 구체 모듈·가시성·테스트 이전·추출 순서를 구현계획으로 제시한다.
   실제 규칙 오류 수정과 IR 스키마 변경은 동작 보존 추출에 끼워 넣지 않는다.

1단계 종료: 실제 회귀 결과와 조사 기록을 확정했다. 변경 문서의 상대 파일 링크 및
`git diff --check`를 확인했다. 2단계 구현계획 작성·승인이 다음 절차이며 제품 리팩토링은 아직 시작하지 않았다.
