# #8174 v2-S50 — run·중첩 XML과 Core 호출 경계의 통합 보강

Issue: #8174. 2026-10-26 착수, 2026-10-27 검증. 기준 `c66cad462e`.

사용자의 「함께 처리하는 것이 더 좋은 방법으로 보입니다」에 따라 run/lineseg와 중첩 XML,
Core pagination·표 프레임 저장 호출 경계를 같은 변경·검증에 포함했다. 내부 보정마다 승인을
다시 요청하지 않았다. **Core 내부 producer까지 모두 연결한 것은 아니며 T04는 진행 중이다.**

## 반영한 전달 경로

- section/master-page → paragraph parts → run splitter/문자모양 경계·markpen·field 위치·
  LineSeg 참조/재배치 배열·문자열 증가에 같은 Storage를 전달한다. 문단별 consume-once 상태와
  subList 깊이는 실패 전파 전에 복구한다. 중첩 표·캡션·텍스트 상자도 이 문단 경로를 사용한다.
- 표·그림·도형 writer의 XML 이벤트·숫자·escape·속성 배열과 그림 변환 행렬의 binary/문자열
  backing을 생성 전에 예약한다. 행렬 binary는 기존 encoder를 사용한다. 기존 semantic 오류를
  경고/빈 결과로 처리하던 분기에서도 요청 실패는 먼저 확인해 밖으로 전파한다.
- 표 grid/row size의 재구축과 row-grid sidecar의 fingerprint·projection·JSON writer에 같은
  Storage를 전달한다. 표 projection 복사는 직렬화에서 제외된 문단 runtime 필드도 먼저 계상한다.
- `from_parsed_document_with_storage` → `paginate_with_storage` → `paginate_pass_with_storage`로
  같은 요청을 전달한다. orchestration이 소유하는 section/cache/map 배열, 측정 표의 깊은 복사,
  미주 복사와 머리말·꼬리말 상속/재선택의 복사를 생성 전에 예약한다. 실패한 초기 Core는 반환하지 않는다.
- 파생 구조의 내부 `CloneWithStorage`는 소유 String/Vec/집합과 공유 Arc/Rc를 구분한다.
  구조 필드의 exhaustive pattern으로 새 소유 필드가 추가되면 정책을 다시 확인하게 한다.
  일반 Unbounded 호출은 기존 `Clone`을 직접 사용한다. 이 정책이 아직 호출되지 않는 내부
  clone까지 예약한 것으로 해석하지 않는다.
- `try_prepare_hwp_export_snapshot` → `writeback_reflowed_table_frames_with_storage`는 저장
  계획·컷 집합·host 줄 목록을 같은 요청으로 생성한다. row-cut 순서·물리 시작점·줄 소속 helper의
  **수집 결과**도 예약한다. helper가 소비하는 `cell_units` 본체의 생성 비용은 아래 잔여다.

배치 규칙과 컷·요구 높이의 계산은 바꾸지 않았다. 표 저장은 기존
`PartialTable.start_cut → row_cut_cell_order → physical_start_after_leading_spacers →
cell_unit_line_anchors → CellFrameWriteback → snapshot LineSeg.vertical_pos`의 같은 결과를
소비한다. block/nested-row cursor의 unmappable 분기와 원래 줄 간격·후속 내용 보존 규칙도 유지한다.
새로운 좌표 핀·쪽수 기대값·fixture/golden은 추가하거나 완화하지 않았다.

## 실제 검증

증적은 ignored `output/pr-review/issue8174/v2-s50-20261026/`에 보존했다.
`production-freeze.json`의 source와 `native-producer.json`의 제품 hash, `checks.json`의 명령·결과를
대조했다. Native는 root opt-level=1 / codegen-units=64의 기능 검증 제품이며 성능 측정 제품이 아니다.

| 항목 | 결과 |
| --- | --- |
| 새 경계 검사 3개 | 같은 test source·S49 Native library에서 **3 FAIL**, 후보에서 **3 PASS** |
| 공개 요청·저장·wire 계약 | 기존 25개 + run/lineseg·pagination·중첩 표 4개 = **29 PASS** |
| Core·높이 측정·typeset·pagination 집중 회귀 | **624 PASS / 기존 8 ignored**. sample16을 읽는 기존 `page_border_fill_reference_sources_keep_paper_and_page_basis`는 사용자 제외를 유지해 실행하지 않음 |
| 기존 serializer/adapter·Allocation | **662 + 9 = 671 PASS**. 위 Core 필터와 겹치는 검사가 있어 합산 고유 개수로 표시하지 않음 |
| 실제 HWP/HWPX 11개 | header 11개·정상 ZIP 10개·기존 양식 거절 1개의 **22개 파일 bytes 동일** |
| PDF 17개·49쪽 | **34개 저장/98쪽 재열기**, 상세 진단 3개와 선택 쪽 추가 4개 왕복 유지 |
| PDF 저장 파일 | 일반/상세 **37개 bytes가 S49와 동일** |
| CLI 실패·복구 | HWP/HWPX의 출력·진단·잔여 예산 **6 PASS**, 기존 파일/stdout 보존·새 요청 복구 |
| 새 Native 시각 선행 | CFF **100.00000%**, Type3 11쪽 **99.97900%**, compare/overlay/review PNG 6개의 bytes도 S49와 동일 |
| 정적·수집 검사 | Native/WASM Clippy `-D warnings`·feature-off·fmt 통과, suite 배정 계약 **23 PASS** |
| 절편 source unit 정책 | HEAD 대비 **4,134 tests / 304 modules**, source-side 증가 없음 |

기존 양식 거절은 `hwpers_test4_complex_table.hwp`의
`FORM_ACTION_INVALID: invalid or excessive form action`이다. 정상 저장 11건으로 세지 않았다.

새 CFF review/standalone overlay에서 글자·기호·ligature·간격을, Type3 11쪽에서 26–30번 행·
괘선·그림 9/10·캡션·앞뒤 내용의 포함과 순서를 직접 대조했다. 기존 미세 래스터 차이를 유지하며
새 누락·겹침은 관측하지 않았다. 이 두 쪽의 확인은 최종 전쪽 Native/fresh WASM 검증의 대체가 아니다.

새 정식 검사는 `tests/cases/issue_8174_pdf_session_lifetime.rs`에 두고 실제 Native library에
연결해 실행했다. S49에서는 run/lineseg가 section 조립에서 늦게 실패하고 pagination 한도는
composition 경계에서만 확인됐다. 후보는 해당 run/lineseg와 pagination에서 거절한다. 같은
정상 작업 실측값을 기준으로 한도 산식을 적용했으며 오류 뒤 실패 유지·한도 준수·live IR/일반
저장 bytes 보존·새 요청 복구를 확인했다. 중첩 표도 실패 뒤 같은 계약을 검사했다.
이 입력은 host가 수정한 IR의 자원 검사이며 독립 한컴 문서의 시각 fixture가 아니다.
`run-budget-before-producer.json`의 test source hash는 후보에서 실행한 source와 같다.

## 통합 작업에 남은 경계

현재 예약한 orchestration의 결과 복사를 내부 producer의 예약으로 표시하지 않는다.
다음 실제 호출 경로를 같은 T04 통합 작업에서 계속 보강해야 한다.

- `paginate_pass_with_storage`가 호출하는 `compute_render_normalized`, `HeightMeasurer`의
  측정·중첩 `compose_paragraph_in_context`/frame reflow, TypesetEngine/Paginator 내부 state 증가.
- `settle_body_page_fields`의 반복 페이지 분할, 표 프레임 helper가 소비하는 `cell_units`/index
  cache 생성, content-loss 보고 등의 나머지 내부 수집. PDF가 생성하지 않는 control의 일반
  semantic producer와 실제 PDF 요청 경로를 구분해 확인한다.
- T05의 WASM/Studio 복사·동기 재열기와 T06–T09의 JPEG·의존성·포장·전체 회귀·전쪽 시각/성능·
  base/PR 준비, 기존 source unit 전체 이관.

논리적 payload 작업량은 프로세스 전체 heap/RSS·실행 시간의 상한 증명이 아니다.
제품 기본/host 한도를 높이지 않았고 `parserAllocationBoundsProven=false`를 유지한다.
현재 source의 최적화 fresh WASM·Studio 실행은 아직 없으며 S37 제품을 현재 결과로 사용하지 않는다.
원격 push·PR 생성·registry 게시는 수행하지 않았다.
