---
kind: canonical
status: active
canonical: mydocs/tech/typesetting_architecture.md
last_verified: 2026-09-23
---

# 조판 코드의 책임 경계와 변경 지도

## 목적과 권위

기여자가 조판 코드를 추가·수정·삭제할 위치와 소비 경로를 찾기 위한 구조 정본이다.
`TypesetEngine`의 #7280 책임 분리 결과를 설명하며, 기존 조건이 올바른 조판 규칙이라는
재승인이나 전체 렌더러의 재설계 완료를 뜻하지 않는다. 제품 확인 기준은 `7947ee45f`다.

- 포맷 해석과 공통 IR: [파서 경계](parser_architecture.md), [LineSeg 계약](document_ir_lineseg_standard.md).
- 표의 의미·좌표 규칙: [표 레이아웃 규칙](table_layout_rules.md). 여기서 중복 정의하지 않는다.
- 구현·검토 원칙: [AGENTS](../../AGENTS.md#조판-수정과-검토-원칙).
- 기여 검증: [CONTRIBUTING](../../CONTRIBUTING.md#pr-전-체크리스트).
  메인터너의 추가 검증은 [로컬 검증 4.3](../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증)을 따른다.
- 이 작업의 이동·무회귀 증거: [구현계획](../plans/task_m100_7280_impl.md),
  [R5 검증](../working/task_m100_7280_stage55.md). 과거 이슈 번호는 이력이지 규칙의 이름이 아니다.

## 문단을 출발점으로 읽기

HWP/HWPX 파서가 공통 `Document → Section → Paragraph`를 만든다. 문단은 텍스트와
컨트롤의 논리 소유자이고, 표의 셀·각주·미주도 다시 문단을 가진다. 논리 소속과 배치 기준
Paper/Page/Column/Para는 서로 다르다. 컨트롤 배열 순서만으로 텍스트 삽입 위치를 재구성하지
않으며 UTF-16/control 슬롯, 구성된 줄의 문자 인덱스, 저장 LineSeg의 축을 구별한다.

```text
공통 IR + 서식 + 기존 구성/측정 결과
  → TypesetEngine 공개 진입점
    → section: 원본 문단 순회·호출 순서
      → paragraph / controls / table / notes: 조회와 흐름 조정
        → state: 확정 항목·쪽/단 전이·예약 반영
          → PaginationResult / PageItem → layout → RenderTree → 출력 backend

표 셀·각주 안의 문단 → 기존 composer/height_measurer/layout 경로
                    (모두 section의 본문 순회로 평탄화하지 않음)
```

위 그림은 책임 흐름이다. 실제 의존은 완전한 단방향 DAG가 아니다. 표의 컷은 LayoutEngine의
셀 유닛 계산을 호출하고 미주 측정도 scratch layout을 사용한다. 아래 잔여 의존 목록을 함께 본다.

## 실제 변경 위치

경로의 공통 접두사는 `src/renderer/typeset/`다. 각 행은 책임 소유자이며 관련 파일 전수 목록은 아니다.

| 변경할 책임 | 구현 진입점 | 여기서 소유하지 않는 것 |
| --- | --- | --- |
| 공개 진입·호환 API | [`../typeset.rs`](../../src/renderer/typeset.rs) | 새 포맷 파싱·새 규칙의 편의상 집결지 |
| 구역·문단 순서 | [`section.rs`](../../src/renderer/typeset/section.rs), `section/{entry,stored_reset,tail,heading,flow,post_flow,controls}.rs` | 셀 컷·미주 줄 계산 세부 |
| 줄 소속·서식·fit/분할 | [`paragraph.rs`](../../src/renderer/typeset/paragraph.rs), `paragraph/{line_queries,context,format,metrics,flow,fit,scan,split,placement}.rs` | 원본 IR 변경, backend별 줄 소속 추측 |
| 컨트롤 순서·TAC·float·지연 배치 | [`controls/paragraph_flow.rs`](../../src/renderer/typeset/controls/paragraph_flow.rs), `order`, `stored_tac`, `tac_*`, `flow_table`, `deferred*`, `wrap_*` | 표 셀 내부 컷 알고리즘 |
| 공통 inline 흐름 적용 | [`inline_flow.rs`](../../src/renderer/typeset/inline_flow.rs), `inline_flow/plan.rs` | 지원되지 않는 입력의 새 fallback 규칙 |
| 표 준비·전체 수용 | [`table.rs`](../../src/renderer/typeset/table.rs), `host_spacing`, `block/{prepare,entry,whole_fit,host_placement}` | 페이지 상태의 직접 필드 쓰기 |
| 행/rowspan 컷 후보 | [`table/scan/runner.rs`](../../src/renderer/typeset/table/scan/runner.rs), `scan/{row,block_fit,block_fragment,source_frame,source_tail,...}` | 확정 PageItem 방출 |
| 표 조각·재개 수명 | [`table/continuation.rs`](../../src/renderer/typeset/table/continuation.rs), `continuation/{job,step,fragment}.rs`, `fragment/{budget,scan,emit}.rs` | 원본 표와 유효 행 도메인의 재추정 |
| 각주 내용·참조, 미주 구성/배치 | [`notes.rs`](../../src/renderer/typeset/notes.rs), `footnotes/{body,boundary,measure}`, `endnotes/{prepare,format,measure,fit,paragraph,emit}` | 본문 쪽 상태 직접 쓰기; 표 fragment 큐 수명 |
| 상태 소유·전이·확정 | [`state.rs`](../../src/renderer/typeset/state.rs), `state/{data,commands,transition,notes,finalize}.rs` | 새 fit 정책이나 포맷 해석 |

`section::run_section`의 실제 순서는 지연 표 선행 처리 → 명시 경계 → 저장 경계 → 빈 꼬리/어울림
흡수 → 쪽 보장·제목 보호 → 본문/표 흐름 → 앵커/vpos 후처리 → 컨트롤 → 이전 문단 인덱스다.
조기 반환은 원래 문단 루프의 건너뛰기와 대응한다. 같은 조회라도 상태 전이 전후 호출 시점을 바꾸지 않는다.
종료는 지연 그림 → 미주 → 지연 표 → 단 flush → 후행 흡수 부착 → 빈 꼬리/쪽 번호 확정 → 결과 이동이다.

## 조회·조정·상태 적용 계약

SOLID는 책임별 변경 위치·좁은 입력·가시성 경계에, CQRS는 조회와 상태 적용의 구별에 적용한다.
동적 rule engine, 등록 플러그인, 이벤트 버스나 별도 IR 복사본은 만들지 않았다.

- **Query**: 기존 참조·스칼라·불변 관측값을 받고 결과를 돌려준다. `FormattedParagraph`의
  `total_height`, `height_for_fit`, 줄 전진량은 다른 의미다. `BlockTableRowScan`의 컷과
  `consumed` 물리 점유도 합치지 않는다. Query가 진단·내부 캐시를 사용하면 수학적 순수 함수라고
  부르지 않는다. 특히 `&self`만으로 순수성을 입증하지 않는다.
- **조정**: `paragraph/flow`, `controls/paragraph_flow`, `table/continuation`, `section`은
  조회·재측정·상태 전이 순서를 연결한다. 상태 전환 후 다시 계산하는 기존 흐름을 유지한다.
  모든 단계를 하나의 새 원자적 트랜잭션으로 만들지 않는다.
- **Command**: `TypesetState`가 항목, 흐름 높이, 쪽/단, 예약, 앵커 기록을 반영한다.
  외부에는 private `data`의 불변 `Deref`만 제공하며 `DerefMut`/가변 data 접근자는 없다.
  `append_item`, `advance_flow_by`, `commit_*`, 예약 메서드를 사용한다. 결과 타입의 공개 필드와
  진행 상태의 쓰기 권한은 다르게 취급한다.
- **다른 상태 소유자**: 행 스캔의 `ScanProgress`는 scan 내부, continuation 커서는 context,
  구역의 이전 문단 인덱스는 section 지역 변수다. 모든 가변 값을 TypesetState에 몰지 않는다.
  Native drain은 완료한 `flow_state` 소유권을 복귀시키고 WASM job은 완료 경계에서 결과를 넘긴다.
- **접근 범위**: 내부 연결은 `pub(super)`/`pub(in crate::renderer::typeset)`이며 기존 공개 facade와
  crate-visible 재개 API를 유지한다. private helper를 테스트하려고 public API를 늘리지 않는다.

## 규칙과 계약을 연결하는 방법

아래 의미 ID는 이 문서의 탐색 키다. 런타임 ID·새 정책 검사기가 아니며 코드 rename을 요구하지 않는다.
이번에 분리한 대표 계약만 연결한다. 전체 조판 규칙 목록이나 모든 기존 예외의 사양 검증은 아니다.
표에서 말하는 근거가 기존 계약이면 **동작 보존 근거**이며, 독립 한컴 출력과의 일치 근거가 아니다.

| 의미 ID·근거 | 적용 / 비적용 | 입력 → 담당 결과 → 실제 소비·상태 소유 | 기존 검사 진입점 |
| --- | --- | --- | --- |
| `paragraph.line-membership` — 구성 줄의 반열린 범위, 기존 composer 계약 | 구성된 줄 / 원본 control 배열만으로 줄 추정 불가 | `ComposedParagraph` → `paragraph/line_queries` → root의 수식/TAC 판별 및 `notes/endnotes/{prepare,fit,paragraph}`; 조회 자체 상태 쓰기 없음 | [`issue_4318_endnote_last_column_frame`](../../tests/cases/issue_4318_endnote_last_column_frame.rs) 등의 소비 경로 검사. 네 helper의 모든 입력 경계에 대한 독립 단위 증거는 미검증 |
| `control.stored-tac-placement` — 저장 줄 수용과 공통 원점 보존 | 비편집 stored profile·유효 TAC 줄·측정/예산 fit / 불일치·side-wrap 입력은 fallback | Paragraph·FormattedParagraph·MeasuredTable·StoredTacPage → `controls/stored_tac::prepare/placement` → `controls/paragraph_flow` → state의 stored TAC 확정 → layout | [`issue_7150_tac_line_owner_anchor`](../../tests/cases/issue_7150_tac_line_owner_anchor.rs), [`issue_6601_inline_tac_tables_share_a_line`](../../tests/cases/issue_6601_inline_tac_tables_share_a_line.rs) |
| `control.wrap-band-lifetime` — 어울림 소유 밴드의 기록·종료 | 활성 어울림 후속 문단 / 매칭 실패·전폭 꼬리는 별도 경계 판단 | `wrap_match/absorption/tail` → `wrap_flow` → state의 밴드 확장·흡수·종료 → 다음 본문 fit | [`issue_6128_wraparound_para_flow_advance`](../../tests/cases/issue_6128_wraparound_para_flow_advance.rs), [`issue_7158_square_wrap_continuation`](../../tests/cases/issue_7158_square_wrap_continuation.rs) |
| `table.fragment-cut-domain` — 행/블록 컷의 소유 유닛 보존 | 분할 표·rowspan·유효 행 도메인 / 통째 수용은 block 경로 | MeasuredTable·원본/유효 Table·시작 컷·예산 → `scan/runner`의 BlockTableRowScan → `continuation/fragment/{scan,emit}` → state 항목/흐름 → layout/table_partial | [`issue_6935_row_space_start_cut_straddle`](../../tests/cases/issue_6935_row_space_start_cut_straddle.rs), [`issue_6803_split_start_row_rowspan_end_cut`](../../tests/cases/issue_6803_split_start_row_rowspan_end_cut.rs) |
| `table.continuation-commit` — 재개 중간과 최종 commit 수명 | 분할 job의 step/drain / 일반 비재개 API를 별도 정책으로 변경하지 않음 | prepared·cursor·flow_state → `continuation::step` → `job` 완료 시 `state::into_result` → document_core/WASM 적용 | [`wasm_api/tests.rs`](../../src/wasm_api/tests.rs)의 `issue2424_resumable_{pagination,delete}_commits_only_after_final_fragment` |
| `notes.reservation-budget` — projected 조회와 실제 예약 분리 | 각주 포함 본문/표 조각 / 미주 흐름은 endnotes | 참조 내용·구분선/간격·footer 관측 → `notes/footnotes`와 `state/notes` → 본문/표 refit·실제 예약 Command | [`typeset.rs`](../../src/renderer/typeset.rs)의 `footnote_area_reserve_uses_section_shape_metrics`; [`issue_5966_queued_table_footnote_fresh_page`](../../tests/cases/issue_5966_queued_table_footnote_fresh_page.rs)는 큐 완주/쪽수 검사이며 좌표 정합의 대체 증거는 아님 |
| `notes.endnote-query-isolation` — 측정이 확정 흐름을 바꾸지 않음 | 미주 예상 위치/전진량 / 실제 방출·단 전이는 emit | 문단·FormattedParagraph·불변 state → `endnotes/measure`의 scratch 계산 → `fit/paragraph` 판단 → `emit`와 state 명령 | [`typeset.rs`](../../src/renderer/typeset.rs)의 `test_measure_endnote_advance_side_effect_free` |
| `section.page-finalization` — 이어받기 쪽의 원 앵커 속성 처리 보존 | finalization의 페이지/머리말/쪽 번호 / 새 문서 해석 아님 | 페이지 항목·header/footer 참조 → `state/finalize` → `into_result`; `section/finalize`는 참조 수집 담당 | [`typeset.rs`](../../src/renderer/typeset.rs)의 `table_continuation_does_not_reapply_page_hide` |

분할 규칙을 바꾸면 Query의 컷만 검사하지 않는다. `시작/끝 컷·소유 유닛 → 요구 높이 → 누적
예약 → 예산 실패 이월 → PageItem → 실제 layout`을 대조한다. 같은 타입을 반환하거나 helper를
공유한다는 사실은 후속 덮어쓰기·원점 선택·클리핑까지 같다는 증거가 아니다.

## 잔여 의존과 근거 재검토 위치

| 현재 경계 | 남긴 이유·수정 시 확인 |
| --- | --- |
| `table/block/prepare` → `LayoutEngine`; `table/scan` → layout의 셀 유닛/컷 계산 | 측정·paint가 소비하는 기존 컷 정의를 보존. typeset만의 새 컷 알고리즘을 복제하지 않는다. |
| `notes/endnotes/measure` → scratch layout | 실제 미주 위치 예측 경로를 보존. 공유 layout 상태의 오염 여부와 무부수효과 계약을 함께 검증한다. |
| `typeset.rs` → `document_core::queries::rendering::body_pile_stays_on_anchor_page` | 기존 pile 판별자 공용화는 별도 설계 대상. renderer가 document_core와 완전히 독립적이라는 주장 금지. |
| `TypesetEngine`의 `Cell/RefCell` profile·저장 사다리·float 관측 | 구역 초기화/측정 순서 의존을 보존. 불변 참조라도 호출 시점 변경을 검토한다. |
| state → controls/paragraph/table 결과 타입 | 계산 결과를 적용하기 위한 타입 의존. state는 독립 도메인 crate가 아니며 넓은 읽기 StateView도 남아 있다. |
| root `typeset.rs`의 helper·기존 private 테스트 | facade만 남은 파일이 아니다. 잔존 유틸리티와 테스트를 줄 수 목표로 무분별하게 이동하지 않는다. |

**근거 확인 필요**: `paragraph/overflow`, `table/scan/landscape`,
`table/scan/source_tail/extension`, `notes/endnotes/profile` 및 root의 미주 off-canvas 허용치처럼
기존 실측/호환 예외는 이번 리팩토링에서 조건·상수·순서를 보존했을 뿐 일반 사양으로 재입증하지 않았다.
이슈 주석·통과 테스트·페이지 수만으로 새 규칙의 근거로 삼지 않는다. 재검토 시 원본 생성 방식,
독립 출력, 적용/비적용 반례와 실제 소비 경로를 확보하고 별도 의미 변경으로 다룬다.
이 목록 역시 모든 legacy 예외의 전수 감사 결과는 아니다.

## 기여자가 변경할 때

기존 PR 설명과 코드 문서에 다음 연결을 남긴다. 별도 양식이나 새 CI 서비스를 요구하지 않는다.

1. **추가**: 위 지도에서 책임 소유자를 선택하고 기존 결과를 재사용할 수 있는지 확인한다.
   의미 ID/근거, 적용·비적용 입력, 반환 결과, 상태 적용자와 최종 소비자를 적는다.
2. **수정**: 계산뿐 아니라 호출 순서, 좌표/단위, 저장/편집 경로, 후속 덮어쓰기까지 추적한다.
   변경 전 FAIL/변경 후 PASS는 의도한 원인으로 확인하고 정상 대조군을 함께 둔다.
3. **삭제·대체**: 호출부·fallback·공유 타입·기존 검사·문서의 대체 소유자를 확인한다.
   삭제한 구현과 무관한 보호 계약까지 제거하거나 기준값을 완화하지 않는다.
4. **구조 이동**: 이전/이후 위치·가시성·입출력·평가 순서·수명을 연결한다. 동작 수정은 섞지 않는다.
   출력 보존과 한컴 피델리티 통과를 구분하고, 기존 차이는 남은 차이로 보고한다.

새 회귀는 `tests/cases/` 원본에 작성한다. source-side cfg(test) 모듈/지원 항목을 늘리거나
include로 숨기지 않는다. 기존 private 테스트 위치 보존은 새 테스트를 제품 파일에 넣는 허가가 아니다.
suite 번호를 고정하지 말고 원본 파일·검사 이름을 사용하며, 파생 suite/manifest는 제출하지 않는다.
검증 명령·정책 base 고정은 CONTRIBUTING과 로컬 검증 정본을 따른다.

최종 검토에서는 **구현 주장 → 적용/비적용 경로 → 독립 기대값 → 실제 검사 항목 → 전후 관측값**을
연결한다. 시각 판정과 애매한 출력의 트리아지는 [시각 검증 거버넌스](../manual/verification/visual_verification_governance.md)를 따른다.
CC·줄 수 감소는 구조 효과의 보조 지표이지 정확성·성능·근거 확인 필요 항목의 해소 증거가 아니다.
