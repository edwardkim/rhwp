# #8174 T04 완료 보고

2026-10-28. [확정한 완료 범위](../plans/task_m100_8174_t04_completion.md)의 C01–C05를
하나의 작업으로 처리했다. 기준 code는 S60 `e7d053b18f6b765ffa2b29e6ebce49e272763ff5`,
범위 확정 커밋은 `8038ac058ff3c5f89ab8952e48cfb70a22cfaba4`다.
최종 판정은 **T04 완료·필수 미연결/미검증 항목 0건**이다. C01–C05의 실제 연결과
구조적 비도달을 구분해 확인했고, 전체 검사·시각 판독·증적 대조를 통과했다. T05–T09와
최종 PR 준비는 별도 잔여다.

## 구현과 수명

PDF 정책·입력 복사·backend 추출/인식·IR에서 시작한 동일 요청을 최초 Core 정규화·조판과
첫 HWP/HWPX 저장까지 전달한다. 이번에는 일반 문단 State 항목·문단 원점·흐름 끝점·내용
하단 원장, 단 전환/분할 확정, 표 control 순서와 표 서식의 행·셀·누적 높이 복사에 연결했다.
증가 전에 fallible 예약을 하고 오류를 실제 조정자까지 전파한다. 일반 HWP/HWPX API는 같은
구현의 `Unbounded` wrapper를 유지한다. 성공 뒤 일반 편집 세션을 누적 import 예산에 묶지 않는다.

수정한 값의 실제 연결은 다음과 같다. 별도 좌표 clamp·분할 조건·높이 계산은 추가하지 않았다.

| 생산 → 수용 → 최종 소비 | 유지한 계약 |
| --- | --- |
| `section/flow.rs::place_paragraph_flow_with_storage` → `paragraph/flow.rs::place_with_storage` → `paragraph.rs`의 fit/split → `state.rs`의 항목 확정 | 같은 composed line·시작/끝 컷·기준선과 원래의 다음 단 재검사를 소비한다. 저장 원점/흐름 끝점·수용한 내용의 하단을 같은 State에 기록한다. |
| `controls/paragraph_flow.rs::place_with_storage` → `controls/order.rs` → `format_table_with_storage` → `table.rs::format_with_storage` → decoration State 항목 | control의 stable 순서, 측정 행·셀·누적 높이, 표 원점·host 소유와 뒤 본문의 기존 흐름을 유지한다. |
| `table.rs::fit_measured_for_host_with_storage` → `height_measurer.rs`의 기존 frame/선언 높이 보정 → 표 서식/State | 원래 조건 순서와 높이 결과를 유지하고 소유 복사만 예약한다. `Option`의 정상 비적용과 자원 오류를 구분한다. |

## 잔여 항목의 종결 근거

현재 항목별 판정은 [할당 감사의 현행 표](task_m100_8174_t04_allocation_audit.md#t04-최종-대조표)에
모았다. 과거 S10–S60의 「남는다」는 각 시점의 기록이며 현행 미완료 목록과 구분한다.

셀 유닛/행 색인 cache와 일부 frame·각주/미주·커닝/shaping 호출을 표본의 breakpoint 0회만으로
닫지 않았다. 다음 생성·소비 불변식을 코드에서 대조했고, 현재 공개 계약과 실행 자료로 보호한다.

1. `parser/pdf/lowering.rs`는 PDF 쪽마다 빈 본문·빈 `char_offsets`·최대 한 LineSeg인 루트
   문단 하나를 만든다. `structured_lowering.rs`의 루트 표는 Paper/Paper·InFrontOfText·
   비 TAC·`page_break=None`이다. 셀 안의 자식 표만 TAC/Para 흐름을 쓴다. 원본 페이지 선택이나
   표 진단 옵션은 이 루트 속성을 authored 본문 표로 바꾸지 않는다.
2. 공통 정규화의 TAC 수식/글상자와 Square float 조정은 해당 속성을 요구한다. PDF의 원본
   도형/그림은 InFront/Behind 및 overlap 허용으로 생성되며 저장 HWP5/HWPX 전용 분기도
   `SourceFormat::Pdf`에 적용되지 않는다. 루트 문단/표의 소유 불변식이 유지된다.
3. `typeset/controls/table_entry.rs`는 Paper overlay 표를 `oversized_multirow`에서 제외한다.
   따라서 선언 높이가 쪽보다 커도 block 표의 cell fragment 경로로 넘어가지 않는다.
   `decoration_table.rs::continuation`은 Para가 아니면 cell-unit callback 전에 반환한다.
   Paginator의 PDF fallback도 본문 분할 표 대신 overlay Shape 소유를 사용한다.
4. 첫 저장의 `writeback_reflowed_table_frames_with_storage`는 reflow 기록과 **실제 비어 있지
   않은 PartialTable 컷**을 요구한다. PDF 초기 쪽 항목은 fullParagraph/Shape이며 cell-unit
   재구성에 도달하지 않는다. 중첩 셀 측정·composition·인라인 producer는 S51–S60에서 연결한
   별도의 bounded 경로를 사용한다.
5. PDF lowerer는 Footnote/Endnote/ColumnDef/field control을 생성하지 않는다. 미주 대기열·
   page field·list head는 비어 있고 note reset의 저장 형식 조건도 PDF에서 거짓이다. 루트
   `control_line_seg_index`는 LineSeg 0/1에서 임시 배열 전에 반환한다. Square host plan도 없다.
6. PDF 글자 모양은 resolved embedded exact-font registry를 등록하지 않는다. Core 재구성은
   컨텍스트를 비운다. 커닝은 컨텍스트 확인 뒤, horizontal shaping은 두 컨텍스트 확인 뒤에
   소유 projection을 만든다. 실제 초기 17개·49쪽에서도 exact registry/list/field 값은 모두 0이다.
7. `record_computed_paragraph`·`queue_paragraph_origin`·`confirm_pending_paragraph_origins`는
   정의 외 source 호출자가 없다. private State의 해당 배열을 다른 경로가 쓰지 않는 것도 확인했다.
   이 미사용 API를 실행 중인 bounded producer라고 주장하거나 알고리즘을 바꾸지 않았다.

실제 S60/후보의 **18개 PDF·50쪽** 초기/첫 저장에서 위 소유와 항목 종류를 확인했다.
셀 cache 관측은 양쪽 모두 0회였고, 일반 HWP 양성 대조군은 각각 row scope **4,378회**·
cell-unit producer **4,210회**였다. 이는 위 코드 근거를 보조하는 실행 관측이다.
별도 trusted IR 대조군은 실제 중첩 PDF의 루트 표 높이를 용지의 1배/3배로 바꿔 재구성했고
양쪽 모두 **표 1개·Shape 3개·1쪽**으로 기존 Paper guard를 유지했다. 이 일반 host 수명의
대조군을 원본 PDF 시각 일치나 편집 후 import 예산 보장으로 확대하지 않는다.
선언 높이만으로 측정 overflow를 입증하지 않도록 별도로 셀 높이도 키웠다. 실제 공통
HeightMeasurer의 **용지 533.333px·표 3,200px**를 확인한 대조군에서도 S60/후보 모두
**표 1개·Shape 3개·1쪽**이며 PartialTable을 만들지 않았다.

처음 만든 작은 MediaBox PDF는 추론 결과 루트 표가 0개였다. 그 결과를 거대 표 양성 사례로
세지 않았고, 생성 절차·실패한 가정과 실행 로그를 보존했다. giant table의 근거는 위 소비 조건과
별도의 실제 IR guard 실행이다. 임의 HWP의 모든 cache·paint·편집 할당을 T04가 제한한다고
주장하지 않으며, 이번 종결을 위해 PDF 기능을 축소하거나 새로운 거절 조건을 추가하지 않았다.

## 실제 거절·복구와 통합 검사

새 공개 한도 검사 두 건은 동일한 검사 source·Native64 조건에서 **S60 2 FAIL → 후보 2 PASS**다.
기준 초기 계상에 16 bytes의 여유를 둔 입력 정책으로 누락을 검출한다. 오늘의 counter나 최초
거절 위치를 기대값으로 고정하지 않는다. 작은 한도 거절 → 실패 유지 → 같은 요청 재사용 거절 →
새 요청의 같은 쪽수/HWPX bytes 복구를 실행했다. 루트 소유·첫 저장 계약은 양쪽에서 PASS다.
기존 회귀·baseline/golden·보안 한도는 완화하지 않았고 source unit 모듈/개수 증가도 없다.

| 검사 | 현재 결과 |
| --- | --- |
| 공개 요청/저장/wire | 50 PASS, 그중 요청 수명 39 PASS |
| 기존 편집/메타데이터 | 52 PASS |
| 기존 frame/inline Print | 22 PASS |
| Core/두 엔진·정규화/inline/frame | 657 PASS·기존 8 ignored |
| composer/float/model·텍스트/도형 측정 | 244 + 38 PASS |
| serializer/adapter·Allocation | 662 + 9 PASS |
| 실제 저장·재열기/선택 쪽·wire | 저장 37개·header/ZIP 22개가 S60 bytes 유지 |
| 실제 CLI 제한·복구와 preflight | 6 + 6 PASS, 기존 출력 sentinel 보존 |
| Native/WASM Clippy·feature-off·fmt·suite/source unit 범위 검사 등 | 12개 실행 PASS |
| Native Skia 전체 library | root 3,891 PASS·기존 12 ignored·사용자 지정 제외 1건, 관련 workspace library 182 PASS |
| Native Skia 필수 case | missing-picture 2 PASS·direct-PDF 14 PASS |
| 전체 nextest | 11,717 PASS·기존 46 skipped, 새 실패 없음 |
| workspace build·전체 대상 Clippy·manifest·fmt | 모두 PASS. Clippy/포맷 지적 보정 후 실제 재실행 |
| 최적화 fresh WASM·실제 Worker | 249 assertions·14 Workers PASS |
| Studio 실제 브라우저 | 활성 문서/draft/recents 보존·오류/취소/복구·15쪽 표시·HWP/HWPX 저장 PASS |

`sample16`을 읽는 `page_border_fill_reference_sources_keep_paper_and_page_basis` 1건은
S50의 기존 사용자 제외를 그대로 유지했다. 새 실패를 숨기기 위한 제외는 없다.

전체 대상 Clippy의 첫 실행은 S54 `ba75e1aa8c8`이 추가한
`estimate_text_width_with_storage`가 기존 테스트 모듈 뒤에 있어
`items_after_test_module`로 실패했다. warning을 허용하지 않고 테스트 모듈을 파일 끝으로
옮겼다. 그 전후의 **프로덕션 항목과 순서·전체 테스트 본문은 구분 공백을 제외하고 bytes 동일**하다.
원 source와 정리 후 source를 각각 `production-freeze.json`/`final-source-freeze.json`에
보존하고 `test-module-order-equivalence.json`으로 대조한다. 정리 후 전체 대상 Clippy·Native
전체 library·source unit 범위·manifest/fmt를 실제 재검증했다. 앞선 전체 nextest·전쪽 렌더링을
정리 후 새로 실행한 것으로 표시하지 않으며, 동일한 생산 코드/검사 본문과 새 재검증의 근거로
연결한다. 테스트 배치만 정리했으므로 조판·할당·저장 규칙은 추가로 바뀌지 않았다.
suite 준비 뒤 fmt가 새 공개 검사 두 assertion의 줄바꿈도 지적해 정리했다. 원래/정리 후
Rust leaf token **8,593개가 동일**하며, 정리한 최종 검사 source로 S60의 **2 FAIL + 1 PASS**와
후보의 요청 수명 **39 PASS**를 다시 실행했다. 실제 실행 source hash를 함께 갱신했다.

## 출력·제품·비용의 실제 범위

Native/fresh WASM 각각 **21개·87쪽**의 전쪽 TSV와 쪽수 대조를 완료했다. 기존 17개·49쪽의
raw 최저는 **98.34664% / 98.79246%**, inline Print 4개·38쪽은 **99.66991% / 99.17555%**다.
각각 PNG **87개 모두 S60과 bytes 동일**하며 Native fallback 5개·5쪽도 100%·동일하다.
기존 완료 페이지의 98% 조건은 inline 35쪽과 distribution 3쪽의 양쪽 출력에서 유지한다.

현재 CFF·Type3 3/11쪽·중첩 표·여러 줄 셀의 review/overlay와 inline 선택 panel 16개를
직접 대조했다. 표 외곽·본문/셀 소유·중첩 전후 텍스트·그림/캡션·줄 순서와 분할 경계를 유지한다.
독립 PDF와 남아 있는 글자 굵기·가는 획 및 distribution의 일부 기호 차이는 S60과 동일하다.
점수나 PNG 동일성을 독립 PDF와 모든 글꼴이 완전히 같다는 주장으로 바꾸지 않는다.

| Native cold 입력 | S60 논리 decoded bytes | 후보 | 증가 |
| --- | ---: | ---: | ---: |
| CFF | 1,633,772 | 1,634,644 | 872 |
| 중첩 셀 | 1,832,874 | 1,834,586 | 1,712 |
| 여러 줄 셀 | 7,614,927 | 7,616,599 | 1,672 |
| 문단 원점 | 7,648,683 | 7,650,347 | 1,664 |

위 증가는 기존 소유 작업에 대한 예약 계상이다. 실제 heap/RSS·시간 증가 측정이 아니다.
Native 기능 검증 제품은 root opt-level 1·codegen-units 64이며 최종 성능 제품과 구분한다.
WASM은 실제 wasm-opt를 마친 최적화 제품이며 크기 **25,496,895 bytes**(S60 대비
**+0.42919%**), SHA-256은
`82fb2823d00b167dd3caf1a826e6f523912678fb563893444898ccfe22a302fb`다.
`pkg`와 Studio public의 JS/WASM이 일치한다. S60의 JS bytes와 선언 서명은 유지하며
선언 파일은 export 순서만 바뀌었다. 최종 비용·패키지 소비·최신 devel 보존은 T07–T09다.

증적 루트는 ignored `output/pr-review/issue8174/t04-completion-20261027/`다.
`production-freeze.json`, `source-review.json`, `evidence-manifest.json`과
`validated-commit.json`에 최종 source/검사/Native library·CLI/WASM·입력/기준 PDF·글꼴·전쪽
TSV·PNG·실제 명령/로그를 연결한다. 비공개 입력은 기존 로컬 경로에 두고 본문에 식별자를 싣지 않는다.

T04의 종료는 **자원 예약 연결과 명시한 요청 경로의 감사 완료**다.
`parserAllocationBoundsProven=false`와 host의 프로세스/Worker 감독 책임은 유지한다.
T05–T09와 최종 PR 제출 게이트·registry 게시를 완료로 표시하지 않는다.
