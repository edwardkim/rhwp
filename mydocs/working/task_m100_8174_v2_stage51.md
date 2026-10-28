# #8174 v2-S51 — 높이 측정·중첩 composition의 요청 예산 전파

Issue: #8174. 2026-10-27, 기준 `abce97497d`.

S50이 예약한 결과의 복사에서 실제 측정 producer 쪽으로 같은 요청을 전달했다.
본문/셀/캡션의 초기 composition과 측정 배열·표 재귀·공통 열 너비/줄 소속 수집을 함께 보강했다.
기존 공개 함수는 Unbounded wrapper로 유지한다. **프레임 reflow 등 내부 잔여가 있어 T04 전체
완료가 아니며 PR 준비 완료로 표시하지 않는다.** 기존 통합 승인 범위에서 진행했다.

## 생산·소비 경로

| 경계 | 실제 연결과 예약 |
| --- | --- |
| Core → 측정 | `paginate_pass_with_storage`가 일반·빈 구역·증분 측정에 같은 Storage를 전달한다. [Core 호출](../../src/document_core/queries/rendering.rs#L5202), [측정 진입점](../../src/renderer/height_measurer.rs#L2089) |
| 문단 메트릭 | line height/spacing·TAC offset·줄별 pair의 collect/unzip·증가와 빈 줄 fallback을 생성 전에 예약한다. 일반 호출의 unzip/extend는 기존 표준 collection 경로를 쓴다. |
| 셀·중첩 표 | [재귀 측정](../../src/renderer/height_measurer.rs#L3536)이 같은 Storage로 자식 높이·행 하한/누적/병합 배열·제약/상세 셀 원장을 만든다. map/filter_map의 실패를 Result로 전달하며 0/None으로 버리지 않는다. MAX_NESTED_DEPTH와 기존 분기·컷·높이 규칙은 유지한다. |
| composition | [기존 scoped composer](../../src/renderer/composer/composition.inc.rs#L211)의 supplemental metric 수용 조건을 그대로 사용한다. 셀·캡션도 기존 fallible composer를 소비하며 다른 shaping 계약으로 교체하지 않는다. ComposedTextRun 복사는 공통 structural copy를 재사용한다. text/display_text는 예약하고 CharOverlapInfo 등의 scalar는 유지한다. |
| 파생 frame 복사 | [paragraph_with_frame_geometry](../../src/renderer/composer/frame_geometry.rs#L67)의 source 문단과 이미 생산된 LineSeg/space metric 복사, 셀 projection과 runtime 문단 필드를 먼저 예약한다. 원본 IR은 유지한다. frame 자체를 처음 생성하는 내부 reflow는 아래 잔여다. |
| 중첩 크기 전달 | [공통 nested-table owner](../../src/renderer/composer/nested_table_frames.rs#L46)의 높이 배열·문단 projection과 callback 오류를 연결했다. 이미 있던 실측·줄 소속/폭 조건은 유지하며 callback 호출 순서도 바꾸지 않는다. |
| 공통 기하 수집 | [열 너비 소유자](../../src/model/table.rs#L732)의 row/tree/ref 배열·정렬 scratch, control UTF-16 위치와 저장 줄 소속, nested group/가로 구간 및 [stored block flow](../../src/renderer/float_placement.rs#L6833)의 높이/원점 배열도 같은 Storage를 쓴다. |
| 캐시 재사용 | 증분 측정의 이전 문단/표와 baseline 행 높이 복사, 중첩 측정 cache의 entry를 예약한다. 기존 cache key·유효성·행 배분 하한은 유지한다. |

한 값의 연결은 `paragraph_frame_owner_widths → 셀 composition/현재 frame 문단 →
row_heights·기존 rowspan solver → cumulative_heights/MeasuredTable → Core의 measured_tables →
기존 TypesetEngine/Paginator의 fit와 배치`다. 새 좌표·요구 높이·컷 조건을 계산하지 않았으며
S50의 표 프레임 저장 경로와 일반/중첩 분할 규칙도 바꾸지 않았다. 새 좌표 핀·쪽수 기대값이나
fixture/golden은 추가·완화하지 않았다. 상세 원장에서 단일 중첩 표 여부만 확인하던 임시 Vec는
동일한 순서의 iterator로 대체했으며, 0개·1개·여러 개의 기존 선택 조건을 유지한다.

## 실제 검증

ignored `output/pr-review/issue8174/v2-s51-20261027/`에 source·제품 hash·명령·결과를 보존했다.
`production-freeze.json`, `native-producer.json`, `checks.json`을 대조했다. Native는 root
opt-level=1 / codegen-units=64의 기능 검증 제품이며 성능 측정 제품이 아니다.

| 항목 | 결과 |
| --- | --- |
| 새 실제 PDF 예산 경계 2개 | 같은 test source로 **S50 2 FAIL / 후보 2 PASS**, 실패 유지·새 요청 복구 |
| 공개 요청·저장·wire 계약 | 기존 29개 + 측정 경계 2개 = **31 PASS** |
| Core·높이 측정·typeset·pagination | **624 PASS / 기존 8 ignored**, sample16을 읽는 기존 검사는 사용자 제외 유지 |
| composer·float placement·table/paragraph model | **244 PASS** |
| 기존 serializer/adapter·Allocation | **662 + 9 = 671 PASS**. Core 필터와 일부 겹치므로 합산 고유 개수로 표시하지 않음 |
| 실제 HWP/HWPX 11개 | header 11개·정상 ZIP 10개·기존 양식 거절 1개의 **22개 파일 bytes 동일** |
| PDF 17개·49쪽 | **34개 저장/98쪽 재열기**, 상세 진단 3개·선택 쪽 추가 4개 왕복 유지 |
| 저장 파일 | 일반/상세 PDF 저장 파일 **37개 bytes가 S50과 동일** |
| CLI 실패·복구 | 출력/진단/잔여 예산 × HWP/HWPX **6 PASS**, 기존 파일/stdout 보존·정상 후속 요청 |
| 새 Native 시각 선행 | CFF **100.00000%**, Type3 11쪽 **99.97900%**, 새 compare/overlay/review PNG 6개의 bytes도 S50과 동일 |
| Rust·수집 검사 | Native/WASM Clippy `-D warnings`, feature-off·fmt 통과, suite 배정 계약 **23 PASS** |
| 절편 source unit 정책 | HEAD 대비 **4,134 tests / 304 modules**, source-side 증가 없음 |

기존 양식 거절은 `hwpers_test4_complex_table.hwp`의
`FORM_ACTION_INVALID: invalid or excessive form action`이며 정상 저장 11건으로 세지 않았다.

새 CFF review/standalone overlay에서 글자·기호·ligature·간격을, Type3 11쪽에서 26–30번 행·
괘선·그림 9/10·캡션·앞뒤 본문의 포함과 순서를 직접 대조했다. 기존 미세 래스터 차이를 유지하며
새 누락·겹침은 관측하지 않았다. 두 쪽의 선행 확인은 최종 전쪽 Native/fresh WASM의 대체가 아니다.

## 실패 대조와 계상 범위

S50 Native library를 먼저 보존하고 실제 `cell-nested.pdf`와 `multiline-cell.pdf`를 기본 정책으로
열어 초기 decoded 작업량을 측정했다. 그 값 + 64 bytes를 입력 정책으로 정한 같은 테스트에서
S50은 초기 Core를 반환해 실패했고, 후보는 pagination 안의 측정 중 한도 초과를 반환했다.
실패 뒤 요청 재사용 거절·카운터 한도 준수·새 기본 요청의 동일 페이지 수/HWPX bytes를 확인했다.
이는 64-bit Native 실측 cap을 쓰는 자원 회귀이며 WASM 실사용 증거로 표시하지 않는다.

| 실제 입력 | S50 초기 decoded | 후보 초기 decoded | 새 경계 검사의 cap |
| --- | ---: | ---: | ---: |
| CFF 대조군 | 1,626,396 | 1,626,516 | 해당 없음 |
| cell-nested | 1,739,954 | 1,820,188 | 1,740,018 |
| multiline-cell | 7,556,074 | 7,606,256 | 7,556,138 |
| paragraph-origins 대조군 | 7,585,075 | 7,640,385 | 해당 없음 |

`core-budget-before.tsv`·`core-budget-after.tsv`·`core-budget-producer.json`과
`run-budget-before-producer.json`에 source/library hash와 실측을 연결했다. 정식 검사는 기존
`tests/cases/issue_8174_pdf_session_lifetime.rs`에 추가했으며 Native 최종 library에 직접 연결했다.
전후 test source hash가 같다. 기존 자동 배정 경로를 유지하고 새 CI 호출은 추가하지 않았다.

숫자는 새로 계상한 논리적 payload 작업량이다. 실행 시간·실제 heap/RSS의 증가율로 해석하지
않는다. 기본/host 한도를 높이지 않았고 `parserAllocationBoundsProven=false`를 유지한다.

## 같은 통합 작업에 남은 경계

- `recompose_stored_lines_in_frame`·stored-row resolver의 내부 frame 생성/검증·projection과
  overflow/shaping cache. 기존 fallible fill/frame helper가 있으므로 같은 의미의 경로에 연결해야 한다.
- 높이 측정이 소비하는 일부 일반 helper의 선행 수집: object-only TAC picture 줄 참조,
  stored row-frame/일부 wrap geometry 등. 결과 Vec의 예약을 그 앞 producer의 예약으로 표시하지 않는다.
- `compute_render_normalized`, TypesetEngine/Paginator 내부 state, body page-field의 반복
  페이지 분할, 표 저장의 cell-unit/index cache 생성과 content-loss 내부 수집.
- T05의 WASM/Studio 복사·동기 재열기와 T06–T09의 JPEG·의존성·포장·전체 회귀·최종 전쪽
  시각/성능·base/PR 준비 및 기존 source unit 전체 이관.

현재 source의 최적화 fresh WASM·Studio 실행은 아직 없고 S37 제품을 현재 결과로 사용하지 않는다.
원격 push·PR 생성·registry 게시는 수행하지 않았다. T04 잔여도 기존 통합 승인 범위에 유지한다.
