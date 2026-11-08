# #8174 v2-S53 — frame 내부 개체·저장 줄 판정의 요청 예산 연결

Issue: #8174. 2026-10-27, 기준 `e21fe48c92`.

S52의 frame 진입점 뒤에서 일반 경로로 남던 인라인 개체 입력과 저장 줄 판정을 같은
Storage에 연결했다. 기존 통합 진행 승인 범위다. **줄 구성·점유·원점·컷·캐시 수용 조건은
바꾸지 않았으며 일반 공개 호출은 같은 구현의 Unbounded wrapper를 사용한다.**

## 실제 생산·소비 경로

| 경계 | 연결과 보존한 계약 |
| --- | --- |
| 인라인 입력 | [project](../../src/renderer/composer/line_breaking/inline_frame_input.rs)의 문자·슬롯·문단 복사·field 값·출력 backing과 개체 위치 목록을 같은 요청으로 만든다. 원시 UTF-16/가시 scalar 축, 빈 글자 경계·개체 순서·여백 한 번 계상은 유지한다. |
| 인라인 표 실측 | [점유 측정](../../src/renderer/composer/line_breaking.rs#L3302)이 기존 `with_session_edited(true)` 측정기에 같은 Storage를 넘긴다. 표/캡션 실측과 바깥 여백을 포함하는 기존 크기 계약을 유지한다. 표 선언 폭이 0인 경우의 공통 열 폭 fallback도 생성 전에 예약한다. |
| frame fill | [제어 목록](../../src/renderer/composer/inline_control_flow.rs#L15)의 scalar/UTF-16 위치 수집, TAC 표 확장·stable sort scratch, 개체 슬롯 복사와 [필드 토큰](../../src/renderer/composer/line_breaking.rs#L3982)의 두 폭 배열·결과 증가를 예약한다. 일반 control 흐름의 표 제외와 표 전용 NO_LS 경로를 유지한다. |
| run 메트릭 | 공통 line-width owner가 기존 run 순서와 `sum`을 사용하되 style 생성 실패에서 순회를 중단하고 원 오류를 반환한다. TextStyle/LatinSlotMetrics·tab backing의 structural copy는 소유 문자열·배열/Box를 예약하고 immutable Arc는 공유한다. exhaustive 필드 패턴으로 누락을 검사한다. |
| 저장 줄 판정 | [stale](../../src/renderer/composer/reflow_helpers.inc.rs#L729)·underfill과 마지막 낱말의 임시 문자열/메트릭을 예약한다. 기존 과밀/과소충전·선언 폭·개체/개행·마스킹·정렬 조건과 조기 종료 순서는 유지한다. |
| 리더 | [판정](../../src/renderer/composer/reflow_helpers.inc.rs#L510)과 [trim](../../src/renderer/composer/reflow_helpers.inc.rs#L408)이 같은 채움 span을 사용한다. 문자/구간·이분 탐색의 후보 문자열과 style을 먼저 예약하며 원문 text를 보존하고 display_text의 채움만 바꾸는 기존 계약은 유지한다. |
| 저장 꼬리/마커 | [soft-wrap trial](../../src/renderer/composer/stored_word_tail.rs#L213)과 marker frame이 같은 요청으로 문단 문자/스타일·frame 사본·기존 fill·space metric projection을 소비한다. 실제 쪽/단 슬롯·명시 개행·저장 들여쓰기와 physical flag 보존 조건은 유지한다. |
| overflow memo | [동일 셀 폭의 memo](../../src/renderer/composer/reflow_helpers.inc.rs#L216)는 cache hit에서 재측정을 생략한다. miss 때만 같은 요청으로 폭을 재고 map 증가를 예약하며, 기존 true 결과의 fresh 재래핑은 매번 수행한다. pointer/폭 key와 session clear 계약은 유지한다. |

한 값의 연결은 `원시 개체 위치/표 실측 → 같은 frame의 inline token 폭/높이 → 기존 fill의
행 구간 → projection/composition → HeightMeasurer의 문단·행 높이 → 기존 pagination의 fit와
실제 배치`다. 새 요구 높이·앵커·분할 조건을 만들지 않았다. stored-row 판정도 같은 문단/셀
상자와 기존 실측을 소비하며 실패를 false/None으로 바꾸지 않는다. 새 렌더링 fixture/golden이나
좌표·쪽수 기대값은 추가·완화하지 않았다.

## Native 검증과 실패 대조

원시 명령·제품/source hash·로그는 ignored `output/pr-review/issue8174/v2-s53-20261027/`에
보존했다. `production-freeze.json`과 `checks.json`, `native-producer.json`을 연결했다.
Native는 root opt-level=1 / codegen-units=64의 기능용 제품이며 최종 성능 제품이 아니다.

| 항목 | 실제 결과 |
| --- | --- |
| 새 실제 PDF 예산 경계 | 같은 source·cap의 **S52 2 FAIL / 후보 2 PASS** |
| 공개 요청·저장·wire 계약 | **35 PASS**, 실제 실패 유지·동일 요청 재사용 거절·새 요청 복구 |
| Core·높이 측정·typeset·pagination | **624 PASS / 기존 8 ignored**, 사용자 지정 sample16 제외 유지 |
| composer·float placement·table/paragraph model | **244 PASS** |
| 기존 serializer/adapter·Allocation | **662 + 9 = 671 PASS**. Core와 일부 중복이므로 고유 개수로 합산하지 않음 |
| 기존 frame 관련 integration | **17 PASS**: #7160 TAC 3개·#6802 리더 5개·#8105 번호 꼬리 3개·#7702 condense 꼬리 6개. 원본 검사/기대값은 유지 |
| 실제 HWP/HWPX 11개 | header 11개·정상 ZIP 10개·기존 양식 거절 1개의 **22개 파일 bytes 동일** |
| PDF 왕복 | 17개·49쪽, **34개 저장/98쪽 재열기·상세 3개·선택 쪽 추가 4개** 유지 |
| 저장 결과 | 일반/상세 **37개 파일 bytes가 S52와 동일** |
| CLI 실패·복구 | 출력/진단/잔여 예산 × HWP/HWPX **6 PASS**, 기존 파일/stdout 보존·정상 후속 요청 |
| 정적·수집 | Native/WASM Clippy `-D warnings`, feature-off·fmt·suite 배정 계약 **23 PASS** |
| 절편 source unit 정책 | HEAD 대비 **4,134 tests / 304 modules**, source-side 증가 없음 |

기존 양식 거절은 `hwpers_test4_complex_table.hwp`의
`FORM_ACTION_INVALID: invalid or excessive form action`이며 정상 ZIP 11개로 세지 않는다.

새 budget 검사는 S52의 Native64 초기 decoded 실측값 + 64 bytes를 입력 cap으로 지정한다.
같은 검사 source에서 S52는 Core를 반환해 검사에 실패하고 후보는 초기 pagination 요청에서
한도 초과를 반환한다. 실패 뒤 카운터 한도·실패 유지와 새 기본 요청의 동일 쪽수/HWPX bytes도
검사했다. 같은 숫자의 WASM 경계 검사나 실제 heap/RSS 측정으로 해석하지 않는다.

| 실제 입력 | S52 초기 decoded | 후보 초기 decoded | 새 cap |
| --- | ---: | ---: | ---: |
| CFF 대조군 | 1,626,516 | 1,626,516 | 해당 없음 |
| cell-nested 대조군 | 1,823,204 | 1,824,434 | 해당 없음 |
| multiline-cell | 7,606,720 | 7,607,095 | 7,606,784 |
| paragraph-origins | 7,640,617 | 7,640,947 | 7,640,681 |

`core-budget-before.tsv`·`core-budget-after.tsv`·`core-budget-producer.json`,
`run-budget-before-producer.json`과 공개 계약 로그에 source/library hash를 기록했다.
추가 Native 17건은 기존 자동 배정 원본을 같은 최종 library에 직접 연결해 실행했다.
입력·fixture·기대값과 CI 호출은 변경하지 않았다. 이 실행을 그 HWP 문서 전체의 한컴 시각
일치 검증으로 표시하지 않으며 아래 실제 PDF 전쪽 검증과 구분한다.

## 현재 최적화 WASM·Studio·전쪽 시각 검증

현재 source를 Rust 1.93.1·locked release/codegen-units=16으로 빌드하고 wasm-opt까지
완료했다. root pkg의 JS/WASM과 Studio public의 실행 bytes가 같고, 검증용 immutable
`candidate-wasm/`을 보존했다. 이전 제품은 `before-wasm/`에 보존했다.

- WASM **25,208,798 bytes**, SHA-256
  `3f020e7ee79b133e413f4de0a989827606ecace76cb3b330b414d2872b5f2be7`.
- 실제 Worker **249 assertions / 14 Workers**(일반 11개·메모리 실패 3개). 실제 12쪽 전쪽
  렌더와 양쪽 저장/재열기, 출력/codec 한도·입력 오류·취소·timeout·Memory.grow 실패 뒤
  활성 문서 보존·정상 요청 복구를 통과했다.
- Studio는 기존 HWP가 열린 상태에서 실패 3개·취소 1개 뒤 generation·파일명·쪽수·draft·
  저장 draft·handle·recents를 보존했다. 이후 실제 사용자 PDF 15쪽 가져오기·표시와
  HWP/HWPX 저장을 통과했고 pageerror는 0개였다.
- Worker가 제공받고 Studio가 실제 로드한 WASM의 hash가 위 제품과 같다.
  `candidate-wasm-producer.json`·`wasm-checks.json`, `browser/browser-result.json`·
  `t02-studio.json`에 출처·명령·결과를 연결했다.

S52 대비 WASM 크기는 **135,514 bytes / 0.54047% 증가**했다. JS glue는 bytes 동일하며
타입 선언은 InitOutput member 순서만 다르고 나머지 선언 bytes·member/서명은 같다.
크기를 실행 시간/RSS 증가율로 해석하지 않으며 최종 성능 확정은 T08에 남는다.

원본 PDF를 독립 기준으로 Native/fresh WASM **각각 17개·49쪽**의 전쪽 TSV를 산출했다.

| 범위 | Native 최저 raw | fresh WASM 최저 raw |
| --- | ---: | ---: |
| 전쪽 17개·49쪽 | **98.34664%** | **98.79246%** |
| CFF·11개 본문/셀 PDF·이미지 clip/batch | 100.00000% | 100.00000% |
| 텍스트 clip 3쪽 | 99.88074% | 100.00000% |
| Type3 대표 11쪽 | 99.97900% | 99.98717% |
| 실제 사용자 PDF 15쪽 | 99.13508% | 99.11167% |

90% 미만·측정 불가·누락·쪽수 불일치는 없다. **각 runtime의 실제 페이지 PNG 49개 전부가
S52와 bytes 동일**하다. `native-page-byte-equivalence.json`·`wasm-page-byte-equivalence.json`에
전후 파일·hash를 기록했다. 단순 점수 동등과 실제 출력 bytes 동등을 구분한다.

새 CFF 1쪽·Type3 11쪽/최저값인 3쪽·cell-nested/multiline-cell 1쪽에 runtime별
compare·standalone overlay·review PNG를 만들었다. 열 쪽의 review와 양쪽 Type3 11쪽의
standalone overlay를 직접 판독했다. CFF 기호/ligature/간격, Type3의 표·그림/캡션·앞뒤 본문,
중첩 셀의 A/B/C/D·앞뒤 내용과 여러 줄 셀의 소속·포함·외곽을 대조했다. 새 누락·겹침은
관측하지 않았고 그림 세선/문자 획의 기존 래스터 차이는 남는다.

원시 TSV는 `visual/candidate/{native,wasm}/<문서>/silhouette.tsv`, 새 PNG는
`full-page-reviews/`에 있다. `candidate-visual-summary.json`·`visual-evidence-audit.json`·
`direct-review.json`에 전쪽 수·최저값·hash·실제 직접 판독 범위를 연결했다.
Worker의 고정 `visualJudgment` 문구와 silhouette-only의 `not_evaluated`는 자동 API/TSV 검사가
시각 승인 자체가 아님을 나타내며 위 직접 판독 결과와 구분한다. 최신 upstream/devel과의
전쪽 Native/fresh WASM 대조·2%p 후퇴 판정은 T09에 남는다.

## 남은 범위와 판정

- T04: KoPub face 판정·marker geometry 등 일부 일반 helper, kerning/shaping 측정·cache와
  frame metrics의 하위 producer, 나머지 object-only/wrap 측정 경계.
- T04: 정규화·TypesetEngine/Paginator 내부 state·page-field 재분할, table-frame 저장의
  cell-unit/index cache·content-loss 내부 수집.
- T05의 잔여 WASM/Studio 복사·전송·동기 재열기, T06–T09의 JPEG/의존성·최종 포장/소비·
  전체 회귀/성능·이전 source unit 이관과 최신 base/기능 보존·PR 준비.

이번에는 실제 frame 내부 생성·판정 경로와 전파를 보강했으며 T04 전체 완료나 전체 할당
상한 증명은 아니다. 계상은 논리적 payload 작업량이고 `parserAllocationBoundsProven=false`와
host 책임·제품 한도는 유지한다. 문서 메타데이터는 기존 오류 20개·새 오류 0개다.
원격 push·PR 생성·registry 업로드나 토큰 열람은 수행하지 않았다.
