# #8174 T04 — 할당 경계 감사

## T05 연결 완료

2026-10-28의 [T05 완료 범위](../plans/task_m100_8174_t05_completion.md)는 입력 옵션·view 사본,
Worker 응답·출력 backing, 남은 예산으로 편집 HWPX 재열기와 Studio 최종 교체를 하나로 다룬다.
D01–D06 완료·필수 잔여 0건이며 생산/소비·실행 증거는 [T05 보고](task_m100_8174_t05_completion.md)에 연결했다.
T04 자체의 최종 판정과 아래 선행 이력은 보존한다.

## T04 최종 대조표

2026-10-28. [완료 범위](../plans/task_m100_8174_t04_completion.md)의 C01–C05를 통합했다.
최종 판정은 **T04 완료·필수 잔여 0건**이다. 아래 표는 현행 source의 항목별 판정이며, 뒤의 S10–S60은
당시의 발견·실패·부분 완료를 보존한 이력이다. 과거의 「남는다」를 현재 미완료 항목으로 합산하지 않는다.
실제 명령·source/제품 hash·전쪽 TSV·판독은 [완료 보고](task_m100_8174_t04_completion.md)에 연결한다.

| 대상·선행 이력 | 현행 생산 → 소비·종결 근거 | 현재 판정 |
| --- | --- | --- |
| B03 font-set/resource/CMap COW·S10/S11 | backend의 같은 request가 font/collection을 구분해 소유 복사 전에 예약한다. 실제 metric과 cache/인덱스를 혼동하지 않으며 font program/공유 map의 Arc 수명은 유지한다. | 연결 완료, 기존 공개 한도/실패 전파·복구 계약 유지 |
| 입력 사본·추출·인식/관측·clip/table graph·S11–S14/S20 | PDF policy → backend extraction → page observation/recognition의 clone·Vec/map·진단 문자열에 같은 요청을 전달한다. 원본 paint/clip 소유를 저장·재열기까지 유지한다. | 연결 완료, 실제 CLI/공개 PDF·비공개 원본 15쪽 검증 |
| glyph/reading-order·annotation·S18/S19/S21–S26 | backend text 후처리의 scratch·XY-cut·Unicode/decimal·appearance 생성에서 선행 예약, fatal 오류를 정상 fallback으로 삼키지 않는다. | 연결 완료, 현재 전체 회귀와 49쪽 출력 대조 |
| style projection·문자/본문/표 IR·S13–S17 | 실제 lowering/wrap의 source-row/스타일/문자 풀과 소유 clone을 같은 Storage로 소비한다. S15의 원점 보정과 정상 역편집/원상 복구도 유지한다. | 연결 완료, 편집/메타데이터 52 PASS·wire 보존 |
| 공통 tokenizer/frame/reflow·S27–S30/S51–S53 | bounded wrapper를 실제 wrap·측정·중첩 composition에서 호출한다. 저장 줄·리더·꼬리와 overflow memo도 같은 요청을 쓴다. | 연결 완료, frame/inline Print 22 PASS·4개 38쪽 |
| Core 초기화·snapshot·정규화·S31/S32/S41/S50/S54 | parse의 snapshot만 넘기지 않고 `PdfImportSession`의 request를 초기 Core·스타일/정규화·측정·조판까지 유지한다. 성공 후 일반 편집은 독립 수명이다. | 연결 완료, 공개 초기 거절/복구·편집 수명 계약 |
| HWP 컨테이너/레코드/adapter·S33–S35/S42–S46 | 최초 저장 snapshot → 공통 adapter → DocInfo/문단·표/셀/도형·CFB의 payload/record/인덱스를 예약한다. | 연결 완료, 실제 저장/재열기·header/ZIP 22개 bytes 보존 |
| HWPX ID/BinData/XML·S47–S50 | ID pool·BinData context → 공통 encoder → header/section/master-page/run·중첩 XML이 같은 요청을 쓴다. | 연결 완료, XML 공개 2 PASS·실제 저장 37개 bytes 보존 |
| 첫 저장 metadata/content-loss·S55 | 보존 메타데이터와 loss 수집·기록도 실제 export request를 소비한다. 실패 때 live 문서/보고서를 보존한다. | 연결 완료, 일반 저장과 요청 저장 bytes 대조 |
| Paginator/Typeset 구역·쪽/단·소유·S56–S58 | 실제 두 엔진에서 구역 준비·항목/단/쪽 이벤트·머리말·중첩 소유와 직접 이월을 예약한다. 초기 fallback도 검사한다. | 연결 완료, 전쪽 49개와 fallback 5개 bytes 보존 |
| 일반 문단/인라인과 State·S59/S60 + C01 | 공통 문단 서식/plan → fit/split → 항목·원점·흐름 끝/내용 하단 map → 단/쪽 확정까지 연결한다. 계산/수용/배치는 같은 줄·컷을 소비한다. | 연결 완료, 이번 공개 한도 2건 S60 FAIL → 후보 PASS |
| 표 순서/서식/frame·C02/C04 | `controls/paragraph_flow` → bounded stable control order/format → 행/셀/누적 높이·선언 frame 소유 복사 → decoration State. 기존 fit/높이/순서는 그대로다. | 연결 완료, 중첩·여러 줄 셀·측정 3,200px/용지 533.333px의 Paper guard와 정상 HWP/inline 대조 |
| 셀 유닛·행 색인 cache·C03 | lowerer의 루트 Paper/Paper·InFront·비 TAC + one-column → table-entry의 Paper guard → decoration의 Para-only continuation 반환. 첫 저장은 실제 PartialTable 컷이 있어야 cell-unit 호출에 도달한다. 높이와 무관하며 nested 측정은 위 bounded composition 경로다. | 초기 PDF/첫 저장의 cache producer 비도달로 감사 완료. S60/후보 18개·50쪽 관측과 일반 HWP 양성 4,378/4,210회는 코드 증명의 보조 자료 |
| 미주/각주·Square host·control 줄/일부 frame·C02/C04 | PDF 생성 control은 Table/Shape/Picture뿐이다. note/미주 큐는 비어 있고 저장 HWP 전용 분기는 적용되지 않는다. 루트 LineSeg 0/1 query는 배열 전에 반환하며 Square plan도 없다. 재귀 child producer는 S51–S60의 bounded 경로다. | 생성·소비 조건으로 비도달 감사 완료. 임의 HWP의 notes 전체에 같은 예산을 연결한 것으로 표시하지 않음 |
| exact 커닝/shaping·list/page field·S55/C04 | PDF lowerer는 resolved embedded exact registry·list/field를 생성하지 않는다. 초기 rebuild가 컨텍스트를 비우며 커닝/shaping은 컨텍스트 획득 후 projection을 만든다. 첫 export는 page-field 재조판으로 돌아가지 않는다. | 생성·소비 조건으로 비도달 감사 완료, 실제 17개·49쪽 registry/list/field 모두 0 |
| 계산 metadata·대기 원점 API·C01 | `record_computed_paragraph`·`queue_paragraph_origin`·`confirm_pending_paragraph_origins`는 정의 외 source 호출자가 없다. private State의 다른 쓰기도 없다. 원래 알고리즘을 유지한다. | 미사용 경로로 감사 완료. 실행 중인 bounded producer라고 주장하지 않음 |
| C05 통합·실행·직접 판독 | 현재 source freeze·Native library/제품·최적화 WASM과 전체 쪽/저장/명령/실패·복구를 연결한다. 각 항목의 「연결」과 생성·소비 조건에 따른 「비도달」을 구분한다. | 완료. 전체 nextest 11,717 PASS·Native/Worker/Studio·현재 직접 판독·제품/증적 대조 PASS. 테스트 배치/포맷만 정리한 최종 source도 동일성 확인과 실제 재검증 완료 |

범위는 PDF 정책/입력 → 실제 backend/frontend → 초기 Core 정규화/조판 → 첫 HWP/HWPX 저장이다.
`Storage` 예약이 프로세스 전체 heap/RSS·wall-time의 상한이라는 주장은 하지 않는다.
`parserAllocationBoundsProven=false`와 Native/Worker host 책임, 독립된 T05–T09를 유지한다.

## S10–S60의 당시 감사 이력

- 시작: `bfafd891f6`, 제품 code `1e7282ad64`. T03 증거·회귀·결정을 먼저 커밋했다.
- 승인: 남은 todo 순차 실행. T04 전체 완료나 비신뢰 입력 안전성 보증은 아직 아니다.
- 선행: #7911 B03/B04와 S06/S07의 실제 통합·계상·공유를 대조한다. 이미 검증한
  Type1/CFF·glyph name·Unicode memo·TrueType 공유·object payload copy를 다시 만들지 않는다.

## 실제 호출과 잔여 경계

| 경계 | 현재 생산·소비 | 이번 보정·검증 |
| --- | --- | --- |
| Font set·index | `PdfDocument::load_fonts_impl`의 3개 정렬 임시 벡터, extractor 이름 복사, set/fingerprint/name/identity cache | 요청 예산을 예약하고 fallible reserve. set 본체는 Arc로 공유해 cache hit마다 Vec·String을 deep copy하지 않음 |
| CMap 기증 COW | `TextExtractor::share_truetype_cmaps` → `Arc::make_mut(FontInfo)` | 요청별 소유 payload의 잔여 복사를 먼저 계상. 공유 CMap·program·memo는 다시 복사한 것으로 세지 않고 extractor별 격리 유지 |
| 기타 backend index·text | 실제 bounded 추출 호출의 object·color cache, glyph 생성·postprocess | 도달 경로별 선행 예약·실패 전파·일반 입력 보존을 이어서 감사 |
| Frontend·Core·저장 | `parse_pdf_with_policy` → extract → recognition/lowering → IR/진단 → Core → serializer | 현재 request budget을 실제 예약 전에 전달. 구조·내용·normal output과 작은 한도 반례 검증 |

범용 legacy API와 bounded 제품 호출을 구분하며, sticky failure 뒤 fallback 성공으로 숨기지
않는다. 카운터는 누적 요청 작업이며 allocator padding·프로세스 전체 heap/RSS의 증명이 아니다.
로그·중간 자료는 ignored `output/pr-review/issue8174/v2-s10-20261025/`에 보존한다.

## B03 보정 중의 실제 검사와 계상 분리

첫 보강분은 새 COW·font-set·fatal propagation 3개를 포함한 Native 14개가 통과했다.
그 뒤 `/Widths`·`/Differences`의 초기 map/문자열·덮어쓰기와 metadata·직접 descriptor 복사를
추가로 보강했다. 직접 resource 객체는 가능한 곳에서 빌려 읽고, `/W`·`/W2`의 실제 소유 복사는
기존 `object_copy::clone_dictionary`를 소비한다. 직접 glyph name lookup의 `_` 처리도 전체
분할 Vec를 만들지 않고 같은 첫 성분을 빌려 읽는다.

집중 범위를 167개로 늘린 첫 실행은 52 PASS·2 FAIL에서 fail-fast로 끝났다. 나머지 113개를
실행했다고 세지 않는다. 두 실패는 #7911의 정확한 3-entry metric 경계다. 새 코드가 정렬·
font set·cache index를 metric 한도에 함께 계상한 오류였으므로 기존 기대값을 유지하고
`CollectionAllocation`으로 인덱스를 collection 카운터에 분리했다. 실제 glyph/CID/encoding/
Unicode memo·소유 COW entry는 font 카운터에 남긴다. 수정 후 전체 집중 실행은 아직 진행 중이다.

`bfafd891f6`의 backend를 `git archive`로 별도 고정해 새 자원 계약을 같은 공개 API로 실행한다.
root source나 기대값을 되돌려 시험하지 않는다. 독립 소비 harness의 최초 실행은 간접 path
의존성 symlink 누락으로 빌드 전에 실패했고, 정확한 간접 의존성을 연결해 재실행 중이다.
이 환경 실패를 제품 실패나 통과로 세지 않는다.

표준 `FontInfo::clone`은 외부 host가 직접 호출할 수 있는 기존 Rust trait이다. 이를 결과 없는
trait에서 자원 오류로 바꾸거나 전역 request로 오염시키지 않는다. 이번 보정은 실제 bounded
제품 호출의 donation COW와 scoped font set/resource 저장 경로에 적용한다. program·lazy
CMap·TrueType map·Unicode memo의 Arc 공유와 다른 extractor/request의 격리는 유지한다.

## 이어서 추적할 B04 경계

현재 `parse_pdf_with_policy`의 최초 `bytes.to_vec`와 backend의 source 사본, extract의 세
벡터·clip index·barrier, table inference·recognition의 구조/진단 생성, source style projection·
wrap trial clone·character-pool key와 IR·metadata 생성에 예약을 연결해야 한다. 단순 parse의
마지막 snapshot을 Core·저장까지 적용한 증거로 바꾸지 않는다.

Core의 현재 실제 경로는 `from_pdf_bytes_with_policy` → `parse_pdf_with_metadata_and_policy`
→ `from_parsed_document`다. parser 반환 시 request Arc는 metadata의 snapshot만 남기고
소멸하며 Core 조판·공통 직렬화에는 전달되지 않는다. CLI worker는 해당 전체 수명을 별도
프로세스로 감독하지만 이것이 내부 예약 연결의 증거는 아니다. WASM/Studio의 초기 import
요청과 성공 후 일반 편집 세션을 구분해, 반복 편집/저장까지 같은 누적 import 카운터에 묶어
정상 세션을 소진시키지 않는 수명 계약으로 보강한다. 실제 구현·실행 완료 전에는 보류다.

## B03 Native 검증 결과 — 2026-10-25

위의 고정 전 source 실행은 **12 PASS·4 FAIL**로 끝났다. 새 COW·font-set 복사·긴 이름·단순
Widths/반복 Differences 계약에서 실제 누락을 재현했다. 다른 12개가 통과하는 것도 확인했다.
인덱스/메트릭 계상을 분리한 후보에서는 `--no-fail-fast`로 **167 PASS**를 완료했다. #7911의
정확한 3-glyph 경계·전역 cache 격리·실패 뒤 복구 기대값은 바꾸지 않았다. 새 5개 중 4개가
수정 전 실패하며, 나머지 1개는 기존 fatal propagation·request 분리를 함께 보호한다.

생성 suite의 실제 모듈 위치를 확인해 `regression_suite_026/001/011`의 해당 네 case를
`release-test` + `native-skia`로 실행했다. test helper의 font-set 슬롯 경계는 glyph 확장용
128-byte work 단위가 아니라 실제 `(String, Arc<FontInfo>)` 요청 payload + 두 이름 바이트를
기준으로 한다. 이 경계는 입력 resource-name 목록의 실제 복사 계약이며 조판 px 핀이 아니다.

실제 최적화 CLI를 새로 빌드하고 CFF 1쪽·Type3 12쪽·maintainer 문서 15쪽을 각각 HWP와
HWPX로 저장하고 전체 render tree로 재열기했다. **6개 저장·6개 재열기**가 모두 통과했다.
CLI의 `policy.backend.fontEntries=1`에서는 기존 CID 폭 대조군을 거절하고 기존 출력 sentinel을
보존했으며 다음 정상 CFF 가져오기도 성공했다. 음성 harness의 첫 설정은 CLI의 `policy`
봉투를 빠뜨려 `PDF_RESOURCE_POLICY`로 거절됐다. 이를 자원 한도 검증으로 세지 않고 올바른
봉투로 별도 재실행했다. 6개 정상 결과를 불필요하게 다시 실행하지 않았다.

제품 feature `jbig2,jpeg2000,icc-qcms`의 backend 단독 Clippy `-D warnings`도 통과했다.
CLI smoke 실행 중 Clippy가 겹쳤으므로 그 실행의 wall/RSS를 성능 비교로 해석하지 않는다.
B03의 Native 집중 보정·실행 결과이며 fresh WASM·최종 시각/성능·Core/직렬화와 B04의 나머지
경계는 계속 남는다. T04 전체 또는 PR 제출 준비 완료로 판정하지 않는다.

## B04 착수 — `f763a4508f` 이후, 진행 중

B03을 커밋한 뒤 frontend 전용 공통 예약 helper를 추가했다. 최종 완료 전의 구현 기록이다.

- `extract`의 span/order/font·clip 배열, clip index·paint barrier와 이미지 수집의 실제 backing
  buffer 증가에 같은 request를 연결한다. 작은 stencil을 합치는 경로는 handle 수만큼 이미지
  배열을 미리 만들지 않고 실제 발행 이미지 수에 따라 늘린다.
- character pool의 JSON key는 bounded writer가 실제 write 전에 output 길이·새 backing buffer를
  검사한다. `split_off` 대신 미리 fallible reserve한 버퍼로 shape를 이동하고 remap도 예약한다.
  JSON key의 바이트 의미·style dedup·원래 ID 복원은 유지한다.
- `ObservedText`가 원 요청을 보유하고 source 문자열·glyph·행/range/preview·ID 수집을 예약한다.
  backend의 `to_chars_with_budget`는 실제 glyph Vec와 반복 font-name 복사 전에 요청을 확인한다.
  일반 `to_chars` API의 형상 계산은 유지한다. glyph Vec collect의 replacement storage와 이름
  payload를 보수적으로 계상하며 실제 allocator header·RSS 상한으로 부르지 않는다.
- 각 span마다 만들던 baseline 후보 Vec는 가장 가까운 두 개만 stack에 유지하게 바꾼다.
  기존의 최근접 유일 후보·0.001 tie 판단과 stable tie 순서는 유지한다. 변경 출력의 검증은
  별도로 남으며 컴파일 성공을 시각 결과로 해석하지 않는다.

현재 char-pool/extract/observation 연결은 각각 `cargo check --locked`를 통과했다. 추가 보정 뒤
집중 계약·실제 Native/fresh WASM·시각/비용을 완료해야 한다. table inference/recognition의
JSON 값 생성·clone, glyph fitting·lowering와 Core/직렬화 예산은 아직 구현·검증이 남아 있다.

### B04 진단 값과 관측 ledger 보강 — 진행 중

`serde_json::Value`를 다 만든 뒤 크기를 재는 방식 대신, serde visitor에 요청을 전달해
문자열·숫자 map key·sequence 길이 힌트·중첩 object·Display 생성 전에 예약한다. wire JSON은
별도의 bounded writer를 사용한다. 이 내부 serializer의 소비자는 기존 typed record와
`Value`이며, native protocol의 opaque RawValue를 임의로 파싱하는 경로는 허용하지 않는다.
표준 라이브러리 tree node의 실제 allocator layout을 증명하는 수치로 발표하지 않는다.

`Ledger`는 같은 request를 유지하고 block/clip/index·candidate·axis evidence·shared edge·
composition·report의 실제 수집·복사에 전달한다. 이미 소유한 JSON 배열은 이동해 재복사를
줄인다. source-span 상세 JSON은 원래 상세 보고 옵션이 켜졌을 때만 생성한다. 요약의 count와
상세 schema·기존 hypothesis work 초과 시의 incomplete 의미는 유지하며, 요청 예산 초과는
fatal error로 전파한다. 별도 soft inference 한도를 올리거나 clipping을 완화하지 않는다.

사각 경로 판정은 최대 다섯 꼭짓점을 stack에 보관하고, axis snapping은 같은 정렬·고정 기준
차이 조건으로 입력 Vec 안에서 dedup한다. 글자의 유일한 행/열 소속 판단은 첫 두 iterator
결과만 사용한다. 이것은 자료구조 보강이며 새로운 조판 규칙으로 취급하지 않는다.
`cargo check --locked`는 통과했으며 집중 실행·실제 출력 비교와 후속 recognition/lowering·
Core/저장 연결은 아직 남는다. 이 문단을 T04 전체 완료 근거로 사용하지 않는다.

집중 계약 실행은 `release-test` + `native-skia`의 library 경로에서 **6 PASS**로 끝났다.
검증 범위는 Vec 증가 전 거절·JSON wire 길이·거대 sequence 힌트의 delegate 전 거절,
Display의 생성 중단, 숫자 key/enum/nonfinite 값의 기존 JSON 의미, fallible 중첩 literal과
외부 태그 enum의 두 단계 깊이 계상이다. 전체 4,058개 library 검사를 실행한 것은 아니다.
이 결과를 실제 문서의 Native/fresh WASM 시각 통과로 확대하지 않는다. 로그는 ignored
`output/pr-review/issue8174/v2-s11-20261025/frontend-contracts.log`에 보존했다.

library Clippy의 첫 실행은 char-pool 초기 삽입의 `contains_key`/`insert` 조합을 지적했다.
Vacant entry에 예약한 뒤 넣도록 보정해 같은 첫 ID와 dedup 의미를 유지한다. 집중 6건은
이 lint 보정 전 실행 결과이며, 관련 helper 본문은 바뀌지 않았다. 현재 보강분의 전체 문서
회귀와 최종 source 검증은 다음 잔여 보정과 함께 완료한다.

### B04 인식·입력 사본 보강 — 2026-10-26, 진행 중

관측/진단 기반은 `4ad0cc6535`에 커밋했다. 그 위에서 문단·표 전이 기록과 owner 목록,
body partition의 간격/분할 임시 배열, 공통 table hierarchy/topology의 실제 수집에 요청을
전달했다. 기존 stable tie 순서·분할 근거·추론 work 한도와 보류 조건은 유지한다. typed
전이 결과와 이미 소유한 JSON 배열은 가능한 곳에서 이동하며 같은 내용을 다시 직렬화하지
않는다. source style/IR 생성·나머지 table detection/clip tiling·Core/저장 수명은 여전히 남는다.

컨테이너를 열기 전의 최초 frontend 입력 사본과 backend 보관 사본도 선계상하고 fallible
reserve로 복사한다. backend memory reader의 기존 8 KiB 버퍼를 명시해 생성 전에 예약했다.
범용 unbounded `PdfDocument::from_bytes`의 계약은 바꾸지 않는다. 선택 쪽 목록과 페이지
메타데이터의 실제 backing buffer도 같은 요청으로 수집한다. 이 추가 계상이 큰 원본의
선택 쪽 가져오기를 기존 decoded cap에서 거절할 수 있으며, 한도를 임의로 올리지 않는다.

새 입력 사본 계약은 원본보다 1 byte 작은 decoded 예산에서 **수정 전 FAIL / 수정 후 PASS**다.
수정 전에는 컨테이너 생성이 성공했고, 수정 후에는 사본 예약에서 거절하며 사용량 0과 sticky
failure를 확인했다. 새 정상 요청은 성공했다. `bfafd891f6`의 고정 backend를 사용했으며 이
생성자 본문이 T04 직전 `4ad0cc6535`와 동일함을 대조했다. 독립 소비 harness는 같은 제품
codec feature를 사용한다. 이후 동일 backend 실행 파일의 기존 폰트 계약까지 **17 PASS**를
확인했다. source-copy-before-provenance.json·source-copy-before.log·source-copy-after.log·
backend-source-font-contracts.log는 ignored `v2-s11-20261025` 아래에 보존한다.

인식의 semantic 보류/오류는 기존 의미를 유지하지만, backend request의 sticky failure를
검출하면 `PDF_RESOURCE_LIMIT`와 실제 resource diagnostics를 먼저 전파한다. 자원 실패를
빈 성공 보고나 positioned fallback으로 바꾸지 않는다. 현재 이 연결의 library check는
통과했으며, 전이 기록 원자성과 빈 페이지 진단 한도의 집중 실행을 진행 중이다.

추가 집중 실행은 `release-test` + `native-skia` library의 **8 PASS**다. 앞서 6개와 새 인식
계약 2개를 같은 source에서 실행했다. 예산이 부족하면 부분 전이 기록을 발행하지 않고 빈
성공 페이지로 바꾸지 않으며, 새로운 요청에서 정상 trace·빈 페이지 보고가 성공하는 것을
확인했다. 전체 library 검사는 4,058개를 제외한 집중 실행이다. 실제 문서 회귀·전쪽 시각·
최적화 fresh WASM·Core/저장 수명의 완료 증거는 아니다. 로그는 recognition-contracts.log다.

같은 source의 root library Clippy `-D warnings`도 통과했다. 다음 단계는 이미 준비한 표 검출
경계와 clip tiling의 잔여 예약을 연결하는 것이며, 이 절편의 통과를 전체 T04 완료로 표시하지 않는다.

### B04 표 검출·clip tiling과 상세 진단의 실제 보정

표 검출의 edge/axis/coverage·집합/셀·하위 영역·owner 수집과 상세 JSON 생성에도 요청을
연결했다. stable sort의 임시 payload를 먼저 예약하고, 이미 소유한 JSON 배열은 이동한다.
source contributor 전체 복사와 유일 owner 판정의 불필요한 임시 Vec를 줄인다. 내용 clip·빈
영역으로 이루어진 기존 tiling의 판단 조건·의미는 유지하며 자원 실패는 fatal로 전파한다.
이후 source의 library check와 Clippy `-D warnings`는 통과했다.

첫 Native 제품은 별도 producer/hash로 고정했다. 17개·49쪽의 양쪽 저장/재열기 **34건**에서
기존 제품과 diagnostics(변경한 resourceUsage 제외) 및 전쪽 render tree가 동일했다. 그러나
상세 진단 비교의 첫 CFF에서 **FAIL**이 났다. 행 bbox의 Y 3개가 각각 1·0.8·0.7 pt 달랐다.
나머지 상세 2건은 이 실행에서 아직 수행하지 않았으며, 이를 전체 통과로 세지 않는다.

원인은 `4ad0cc6535`의 할당 보강에서 `ObservedSpan.bounds.y`를 입력 bbox 높이가 아닌
`font_size * 0.95`로 바꾼 데 있다. 할당과 무관한 가정을 제거해 `f763a4508f`의 원래 입력
사각형 변환을 복구했다. `ObservedText::lines`의 bbox/top/bottom은 이 값을 소비하고,
`ParagraphMachine::feed`의 intervening-object 경계가 line.top/previous.bottom을 읽으므로
단순 표시용 오차로 무시하지 않는다. 기존 `PdfSpanEvidence`의 다른 명목 envelope를 실제
line observation에 복제하지 않는다.

같은 실제 CFF fixture의 기존 회귀에 source observation bbox와 line bbox의 좌표 변환·높이
보존을 추가했다. 새 px/글꼴 높이 핀이나 변경한 기대값을 쓰지 않는다. 원래 PDF 좌표와
변환된 관측 사각형의 관계를 검사하며 실제 편집·양쪽 저장/재열기 검사는 유지한다.

실패 제품·producer·Native 로그와 34개 성공 결과는 ignored `diagnostic-drift-candidate/`,
`native-compare/diagnostic-drift-candidate/`에 보존했다. 최초 Native CFF 100%·Type3 p11
99.97900%와 직접 review/overlay 판독도 `preflight-diagnostic-drift/`에 남기며, 이 점수로 상세
진단 실패를 면제하지 않는다. bbox 복구 뒤 제품을 다시 빌드해 같은 정상·상세·시각 비교를
진행한다. 최종 source·fresh WASM 검증은 여전히 남는다.

bbox 복구 후 새 Native 제품의 비교는 정상·상세 모두 통과했다. 17개·49쪽의 HWP/HWPX
**34개 파일은 변경 전 제품과 바이트까지 동일**하다. 양쪽 저장본을 전쪽 재열기한 **98쪽
render tree**도 동일했고, 일반 diagnostics와 CFF·중첩 셀·텍스트 clip의 상세 diagnostics
3건도 변경한 resourceUsage를 제외하면 일치했다. 최초 상세 진단의 Y 차이는 없어졌다.
수정 후 Native preflight는 CFF 1쪽 **100%**, Type3 11쪽 **99.97900%**이며 새 review/overlay를
직접 열어 scalar·행 소속·셀 포함·그림/캡션·뒤 행을 확인했다. 나머지 쪽의 새 TSV나 fresh
WASM 실행을 이 비교로 대신했다고 기록하지 않는다. 같은 source의 관련 정식 회귀를 실행 중이다.

재현 자료는 `native-build-input.json`·`native-producer.json`, `native-compare-fixed.log`와
`native-compare/{before,candidate}/`, `native-preflight-fixed.log`와 `preflight/`다. producer는
`48727c054a` + 변경한 제품 Rust 6개 hash로 빌드 입력을 확인하며, 아직 미커밋 보강분의 제품이다.
정식 회귀 추가분은 producer의 testFiles로 별도 구분한다. source/style/IR/저장과 최종 제품
통합 검증 전의 부분 실행 결과다.

생성 manifest를 다시 준비해 현재 suite 위치를 확인하고 `001/005/010/011/020`에서
관련 8개 case 모듈을 `release-test` + `native-skia`, 4 test threads, `--no-fail-fast`로 실행했다.
**213 PASS**이며 기존 frontend/resource/detection·본문/retained border·consecutive MoveTo·폰트
계약과 강화한 CFF bbox 보존 검사를 포함한다. 해당 5개 binary의 범위 밖 1,147개 검사는
이번 집중 실행에서 제외했다. 로그는 inference-regressions.log다. 이 보강분을 커밋한 뒤
style 투영·text/IR builder·backend text postprocess·Core/serializer의 잔여 경계를 계속 처리한다.

### B04 스타일 투영·복사 작업량 — `4ba7c5116c` 이후

#3587의 원 사전 검사 `b1de53c326`, 자원 가져오기 보강 `037303d3cb`와 PR #7101의
[보고](../report/task_m100_3587_report.md)를 대조했다. 기존 방문자의 카운터·깊이/오류 계약을
모델의 `owned_work`로 이동하고 문단 블록은 같은 정책 adapter로 소비한다. `collect_str`는
전체 String을 만든 뒤 세지 않고 fmt write를 세며 생성 도중 멈춘다. serde에서 생략한 소유
payload와 visitor 호출 전에 임시 값을 만드는 custom serializer는 호출자가 별도로 감사한다.
이 helper를 임의의 전체 Document·바이너리에 바로 적용한 것으로 간주하지 않는다.

PDF CharShape의 소유 레코드 복사는 먼저 작업량을 검사하고, 그 뒤 실제 clone을 수행한다.
이 레코드는 derived serializer이며 raw_data도 순회한다. 글꼴 선언의 생략된 native_program은
소유 바이너리가 아닌 Copy enum이다. 다른 control/paragraph의 생략 데이터·custom serializer는
앞으로 해당 경계에서 따로 확인한다. Vec capacity·tree allocator header·전체 RSS의 증명은 아니다.

공통 character-style resolver와 기본 font-name decision의 벡터·문자열·family join은 같은
규칙을 유지하며 storage policy를 받는다. 일반 문서는 기존 infallible 경로를, PDF 투영은 실제
request의 선예약·fallible reserve를 소비한다. 언어 슬롯·치환/메트릭 선택·반올림·비율·자간
탐색 범위는 바꾸지 않았다. PDF의 페이지 투영은 원래 FontFace/CharShape를 빌려 읽어 전체
font 표와 compact DocInfo를 deep copy하지 않는다. 빈 NativePuaFonts의 Arc node와 실제
TextStyle·LatinSlotMetrics의 소유 문자열/Arc/Box payload를 생성 전에 계상한다.

폭 memo의 기존 16,384-entry reuse 한도는 유지하고 실제 index 삽입은 먼저 예약한다. 같은
요청 예산의 실패는 fit-spacing/whitespace 탐색에서 Result로 전파하며, cached width로 성공을
반환하거나 legacy line-frame fallback으로 소비하지 않는다. 같은 원본 스타일의 실제 공통
측정 값과 검색 순서·최종 결과는 유지한다. 전체 IR·wrap trial·Core/serializer는 아직 남는다.

Native Skia를 포함한 library 전체와 #3587의 budget/resources/repeat/validation/save, 기존 PDF
집중 범위를 함께 실행해 **4,323 PASS**를 확인했다. 작은 예산에서 clone 전에 거절하는 검사,
긴 font-name 예약 거절·새 요청 복구와 formatter 생성 중단도 포함한다. 선택한 binary의 다른
1,938개 integration 검사는 제외했다. 원본 문단 블록의 기존 기대값은 바꾸지 않았다.
library Clippy `-D warnings`도 통과했다. 로그는 ignored `v2-s12-20261026`의
style-and-model-regressions.log·style-clippy.log이며, 현재는 실제 최적화 CLI 결과를 준비 중이다.

같은 Native source의 실제 CLI 비교도 통과했다. 17개·49쪽의 HWP/HWPX 34개 저장본은
B03 이후 기준 제품과 바이트까지 같고, 재열기 98쪽·일반 진단·상세 진단 3건은
resourceUsage를 제외하면 동일하다. Native CFF 1쪽 100%, Type3 11쪽 99.97900%이며
새 review/overlay를 직접 열어 scalar·행 소속·셀 경계·그림/캡션·후속 행을 대조했다.
새 줄바꿈·누락·위치 차이를 발견하지 않았다. 전체 전쪽 fresh WASM·최종 성능 검증은 남는다.

제품 입력은 `v2-s12-20261026/native-producer.json`에 head `4ba7c5116c` + 제품 Rust 7개
hash로 고정했다. 실행 파일 SHA-256은
`8aef576fc619cbdd5eab5b09402c39a99b1f291ff8255e04c8aca92f1fcd2cf2`다.
`native-compare-candidate.log`, `native-compare/candidate/`, `native-preflight.log`,
`preflight/`는 ignored 증거다. 모델·style 부분을 커밋한 뒤 문자 재구성·IR 생성과 나머지
T04 경계를 계속 처리한다.

### B04 문자·위치 기반 IR 생성 — `4a6fc5ce32` 이후

`reconstruct_line`의 source glyph 수집 → 추론 공백을 포함한 result → UTF-8 text 생성과
그 결과를 소비하는 본문/셀/단일 span 경로에 실제 Allocation을 전달한다. source scalar·
원점·advance·UTF-16 출처·공백 추론 기준은 유지한다. 작은 예약 실패는 위치 기반 fallback
직전에 sticky 상태를 확인해 일반적인 “glyph metrics unavailable”로 숨기지 않는다.

문자열은 필요한 UTF-8 byte와 교체 버퍼를 생성·확장 전에 예약하며 실패 시 기존 String을
변경하지 않는다. 문자/탭/run·관측 근거·문단 stream-axis 변환과 metadata offset의 벡터·tree
work에도 예약을 연결한다. 문자 축은 기존 content UTF-16 → 확장 탭 8-unit stream 변환을
유지한다. 소유권과 물리 좌표를 바꾸지 않으며 source-space/tab·낱자 metric 탐색도 유지한다.

위치 기반 lowering의 font/CharShape/ParaShape 복사·인덱스·paint order 사본·text-box 문단·
그림 참조·shape/그림 Box payload와 source/diagnostic JSON도 요청을 소비한다. PNG 본체는
기존 codec 예약을 유지하고 복제하지 않고 옮긴다. 고정 scaffold·표 조립·wrap trial·backend
postprocess·Core/직렬화의 잔여 경계를 이 부분 변경만으로 완료했다고 표시하지 않는다.

`cargo check --locked --lib`는 통과했다. UTF-8 성장 거절 전 값 보존·새 요청 복구와 문자
재구성의 출처/순서 보존을 검사한다. 실행 자료는 ignored `v2-s13-20261026/`에 둔다.
Native Skia 집중 회귀 **225 PASS**와 library Clippy `-D warnings`를 확인했다. 이 실행은
선택한 suite의 범위 밖 검사 5,206개를 제외한다. 같은 source의 실제 CLI에서 17개·49쪽을
양쪽 형식으로 저장한 **34개 파일이 기준 제품과 바이트까지 같고**, 재열기 **98쪽**과
일반 진단·상세 진단 3건도 resourceUsage를 제외하면 같았다. Native CFF 1쪽 100%,
Type3 11쪽 99.97900%이며 새 review/overlay의 문자·줄바꿈·셀 경계·그림/캡션·다음 행을
직접 판독했다. 새 누락·중복·배치 차이를 발견하지 않았다.

`native-producer.json`은 `4a6fc5ce32` + 제품 Rust 5개 hash를 고정한다. 실행 파일 SHA-256은
`70e8a960fdaa033f905b4f91b7a401b631417a09dcf9c9ab91cb39b2f8062be4`다.
line-ir-regressions.log·line-ir-clippy.log·native-compare.log·native-preflight.log와 결과 폴더는
이 부분의 실제 실행 근거이며, 전체 전쪽 fresh WASM·최종 성능 결과를 대신하지 않는다.

### B04 표·본문·paint 조립 — `cd070a1fe6` 이후

표 조립·본문 commit의 source 참조·소유 집합·FlowItem·행/셀/형제 목록·소유 레코드
복사와 diagnostic JSON 생성은 실제 요청의 Allocation을 소비한다. 이미 소유한 Value
배열은 옮길 수 있는 곳에서 옮긴다. 표/본문 생성 실패의 기존 rollback은 유지하지만
요청 예약 실패는 먼저 확인해 “인식 불가, 원래 그림 유지”로 숨기지 않는다. source 계층
깊이·cell-slot·보존 경계·선 판정 tolerance·문단 배치 규칙은 바꾸지 않았다.

표 paint partition·native border·compound의 조각/집합/행렬/후보 수집과 source control
복사에도 예약을 연결한다. 일반/회전 text clip은 같은 좌표 변환으로 command를 만들며
생성·벡터 확장 전에 요청을 소비한다. 새 레코드 복사의 범위는 Core 이전에 PDF lowering이
만든 레코드다. 이 시점 Paragraph의 serde 생략 소유 캐시·vpos snapshot은 비어 있으며
seal 등 생략 값은 Copy다. 일반 편집 문단 전체의 clone 증명으로 확장하지 않는다.

`PdfStroke`의 홀수 dash 주기는 원래 반복 순서와 phase를 유지하며 원본 dash를 빌려
읽는다. 같은 공통 기하 함수가 rectangle publication callback을 받아 결과 버퍼 확장
전에 예약할 수 있다. 홀수 패턴의 독립적인 구간 대조와 생성 중단은 합성 계약 검사이며
한컴 출력의 시각 증거로 세지 않는다. 실제 전체 출력은 별도의 Native/fresh WASM 대조다.

생성 PNG의 경계선 witness를 읽는 compound 경로와 `border_paint::Resources`는 resident
PNG를 빌려 읽어 이미지 본체 사본을 없앤다. witness는 최대 4,096-byte typed JSON이며
숫자 배열의 기하급수적 교체 버퍼를 encoded byte당 32-byte work + 고정 record로 먼저
예약한다. 이 경로의 PNG는 같은 요청의 path producer가 생성한다. 외부 PNG decoder나
프로세스 전체 heap을 이 식으로 제한한다고 주장하지 않는다. 비거주 fallback은 기존
256 MiB bounded load의 상한을 예약한 뒤 읽는다. 실제 PDF lowering은 resident 자료를 쓴다.

공통 cell-paint prepend는 원래 slot/문자 축 producer를 호출한다. 호출 전 source scalar·
control·line·보조 slot 수에 따른 임시 char/offset/position/gap/shift의 누적 work를 예약하고
최종 control/CTRL_DATA backing을 실제 reserve한다. 기존 위치/소유 결정과 조판 수치를
호출 지점에 복제하지 않는다. 이 수치는 allocator bookkeeping·RSS의 측정값이 아니다.

library+tests Clippy `-D warnings`는 통과했다. 첫 실행의 새 helper lint와 테스트 생성자
누락은 수정했고, 더 넓은 test lint에서 `1611ec923e`의 기존 Widths 회귀에 발견된
`is_some`/`unwrap`은 같은 기대값의 `if let`으로 바꿨다. 테스트 의미나 한도는 바꾸지 않았다.
현재 Native library·집중 회귀를 실행 중이다. 자료는 ignored `v2-s14-20261026/`에 둔다.

Native library·첫 집중 실행은 **4,274 PASS**다. 이어 #7998의 다른 표/paint/탭/clip
25개 모듈을 기존 자동 배정 13개 suite에서 실행해 **140 PASS·1 FAIL**을 확인했다.
실패는 `inferred_gap_keeps_real_report_words_in_one_editable_paragraph`의 보고서7쪽을
HWP로 저장·재열기한 뒤 삽입→삭제하는 경로다. 문자는 같고 X=292.8133333333333도
같지만 첫 glyph Y가354.6533333333333→323.6933333333333으로 바뀌었다. 기존
0.25px 역편집 원점 검사와 내용·셀 포함·줄 순서 기대값은 유지했다. 실패 메시지에
route·문자열·좌표만 보강했으며 임시 상태 probe는 ignored 사본/결과를 보존하고 제거했다.

원 해결 PR #8120의 통합 `80968497c2`, stage34의 독립 PDF/Native/fresh WASM·사용자
판독과 후속 #8191의 before-space 공통 계약을 추적했다. 같은 공개 PDF의 SHA-256은
`d4c1fe7a8a3c025b2be3648ba6e2ad1123eff3dde008983edadcef7e8f7b6656`다.
표/paint 보강 전 `cd070a1fe6`을 별도 archive로 빌드한 검사도 **같은 좌표로 FAIL**이며,
기준 devel `9418ed0c7`의 실제 실행은 **PASS**다. 후자는 제품 source 2,884개 Git blob을
대조했다. suite 자동 재배정 뒤 처음의 잘못된003 호출은0건/exit4이므로 검사 결과가
아니며, 실제 cd070은004, devel은008에서 동일 case를 실행했다. Root WIP는 보존했다.

일반 기대값을 바꾸거나 known failure로 제외하지 않는다. 상태 관측에서 HWP의 원래
줄 cache는 비어 있고, 편집 후 생성한 네 줄은 implementation tag·첫 vpos0이다. 원래
PDF의2322 HWPUNIT 선행 띠는 paragraph before4644(raw)와 같은31px다. 공간을
점유하는 공통 before-space와 paint가 따로 판단하는 경계를 조사 중이다.
S14의 실제 CLI/시각 비교와 T04 전체 완료는 이 회귀의 보정·재검증까지 보류한다.

원격 fetch에서 devel은 `238de6de41a208648f4ade6b494fe433a81c1fce`까지 전진했다.
새 #8228/#8231의 소스·새 기능 보존은 동기화 후 별도 대조한다. 위 base 실행을 최신
원격 실행으로 바꾸어 기록하지 않는다.


### 편집 후 셀 앵커 회귀의 원점 추적 — S15

`v2-s15-20261026/before-origin-trace.log`에서 실제 문단 경로와 최종 첫 줄 원점을
대조했다. 빠른 부분 paint가 아니라 일반 `layout_horizontal_cell_paragraphs` →
`layout_composed_paragraph_in_frame` 경로다. 저장 전에는 실제 vpos 2322가 선행 띠를
소유한다. HWP 재열기 직후는 줄 cache가 없고 before 30.96px가 이를 소유한다.
편집 후에는 implementation-tag 줄의 첫 vpos 0을 `first_seg_vpos_is_anchor`가
절대 저장 앵커로 수용하면서 para_y를 311.18666666666667에서 280.2266666666667로
선차감했다. 뒤의 before 30.96px 재가산이 같은 높이를 복구하지 못하고 첫 glyph가
30.96px 위로 이동한다. 측정/paint의 before 계산 자체가 0인 문제는 아니었다.

352개 배경 paint의 `cell_frame_paint`·BehindText·Para 기준·비-TAC 소유 표지는
저장/재열기와 편집 전후 모두 유지된다. 기존 저장 경고는 한컴의 native 편집 소유 제한을
알리는 일반 고지이며 이 회귀의 표지 손실 증거가 아니다. `plain_text_before`에 paint를
추가하는 단독 시도는 3 PASS / 1 FAIL로 해결하지 못했다. 해당 시도와 임시 probe/trace는
제거했으며 출력 좌표의 clamp나 특정 문서 분기는 추가하지 않는다.

원 앵커 helper는 #3593/#3595 통합 `6d36bafaab`에서 도입했고 #4333의
`1faa8a33dc`에서도 같은 수용 조건을 유지했다. 현재 공통 NO_LS 계약은 implementation
줄을 원본 저장 줄로 인정하지 않는다. 같은 helper를 소비하는 배치와 셀 vpos 사다리
측정이 실제 원본 줄만 절대 앵커로 수용하도록 보정한다. 기존 #7998의 역편집·내용·탭·
셀 포함 검사를 그대로 실행하며 결과는 다음 절에 기록한다. 현재는 수정 후보의 실제
실행 중이며 Native/fresh WASM 시각·정상 대조를 완료한 상태가 아니다.


S15의 Native 후보는 기존 #7998 gap 회귀 **4 PASS**, library와 관련 PDF/셀 간격·저장
앵커 모듈을 합친 **4,142 PASS / 0 FAIL**이다. 5,354개 선택 밖 검사는 실행하지 않았다.
표준 Native 단독 capture는 동일한 기존 case의 본문과 단언을 사용하며 renderer library의
실제 rlib에 연결했다. 제품 source는 `ec6e997593` + `src/renderer/mod.rs` 한 파일이다.
`native-capture-producer.json`에 해당 파일·rlib·harness·원본 PDF의 SHA-256을 고정했다.

공개 원본 PDF의 7쪽·9쪽을 각각 한 쪽으로 가져온 세 경로에서 원본/역편집 상태 12개를
새 SVG·PNG로 출력했다. 기준은 원본 PDF의 해당 쪽을 canonical PDFium 96 DPI로 직접
래스터한 것이다. `write_silhouette_tsv`의 raw 최저값은 보고서 7쪽 **99.96087%**, 제목
9쪽 **99.85041%**이며 세 경로·편집 전후가 각각 같다. 각 결과의 전체 한 쪽 TSV를
`native-visual/*/silhouette.tsv`에 남겼다. review·overlay와 `native-p7-detail.png`를
직접 대조해 첫 줄의 선행 띠, 탭 뒤 시작점·줄바꿈·셀 안 포함과 다음 행, 제목 띠·그림/캡션
소속을 확인했다. 공급할 원본 프로그램이 없는 글꼴의 윤곽·획 차이는 남고, 이를 원본 글꼴
복원이나 100% 피델리티로 발표하지 않는다. 위치 이동·누락·중복은 발견하지 않았다.

기존 T03 제품 WASM에서도 별도 Chrome 여정으로 보고서 HWP 역편집의 약 31px 이동을
재현했다. HWPX는 첫 재열기부터 같은 선행 띠를 잃으며 역편집만 대조하면 드러나지 않는다.
`browser-gap-before.json`과 before SVG/PNG가 그 증거다. 현재 수정의 fresh WASM 결과는
아직 없으며, T04의 나머지 보정 후 최적화 제품 통합에서 이 여정과 양쪽 형식의 원본 대응을
다시 확인한다. 이번 Native 부분 통과를 fresh WASM 또는 T04 전체 완료로 바꾸지 않는다.

S15의 같은 Native source는 rustfmt와 `release-test` + `native-skia` library Clippy
`-D warnings`도 통과했다. Native 부분 결과와 fresh WASM 잔여를 구분해 로컬 커밋한다.


### B04 JSON 숫자와 의존성 feature 통합 — S16, 검증 중

`4ad0cc6535`의 value guard는 private `Number`와 `RawValue`를 함께 거절했다.
serde_json 1.0.151의 실제 `Number::serialize`와 `NumberValueEmitter`, 독립 소비 graph가
선택한 1.0.154의 같은 경로를 대조했다. arbitrary_precision의 숫자는 문자열 field를
정상 방문한 뒤 decimal scanner가 처리한다. arbitrary Raw JSON의 숨은 객체/배열 재파싱과
같은 계약이 아니다. 숫자 protocol에만 string 방문을 허용하고 일반 RawValue 차단은 유지한다.

`v2-s16-20261026/number-consumer`는 저장소와 독립된 Cargo workspace이며 rhwp는 실제 path 의존성,
serde_json은 arbitrary_precision·raw_value를 함께 활성화했다. dependency lock을 고정한
실제 PDF 가져오기 → HWP/HWPX 저장·재열기는 수정 전 **FAIL**이었다. 코드 `c62d26d76d`의
`PDF_RESOURCE_LIMIT: opaque frontend JSON value`를 consumer-before.log에 남겼다.
첫 보정은 같은 소비 여정 **PASS**였으며 consumer-after.log와 intermediate producer hash에
기록했다. 이후 scalar depth/정리와 추가 경계 검사를 포함한 최종 보정은 아직 실행 중이다.

decimal string의 delegate 전에 input copy·초기 16-byte scan String·기하급수적 교체 버퍼와
정수 정규화의 누적 payload work를 5×문자열 byte + 64로 먼저 예약한다. 근거는 위 두 실제
serde_json 버전의 decimal scanner이며 allocator overhead·RSS 상한으로 발표하지 않는다.
숫자의 내부 struct는 JSON scalar이므로 depth를 하나 더 소비하지 않는다. 128/129 경계,
큰 decimal의 거절·sticky failure·새 요청 복구와 RawValue delegate 전 차단을 집중 검사한다.
정상 feature graph의 자원 카운터나 parser 전체 안전성의 완료로 확대하지 않는다.


S16의 최종 숫자 경계 후보는 decimal protocol뿐 아니라 typed 숫자 primitive의 decimal
생성도 delegate 전에 64-byte scalar work로 예약한다. i128/u128/f64의 유한 표기 길이를
덮는 작은 예약이며 boolean 값은 inline으로 유지한다. 같은 숫자가 map key인 경로의 기존
64-byte 예약은 중복하지 않는다. 이후 Native Skia + root serde_json 1.0.151의
arbitrary_precision/raw_value 통합에서 **13 PASS**를 확인했다. 이 수는 다른 library
4,063건을 제외한 집중 실행이며 전체 회귀 통과로 표시하지 않는다. 128/129 JSON 깊이와
큰 decimal·숫자 primitive의 delegate 전 거절, opaque RawValue 차단, 요청 복구를 포함한다.

독립 consumer는 잠근 serde_json **1.0.154**에서 일반 본문 1쪽·CFF 1쪽·Type3 12쪽의
**3개 / 14쪽**을 실제 가져왔다. 양쪽 형식의 **6개 저장본 / 재열기 28쪽**이 모두 유지됐다.
consumer-final-*-retry.log와 consumer-final-producer.json에 실제 source·consumer lock·
manifest·harness hash를 연결했다. 매개변수화한 harness의 첫 빌드는 expected를 usize로
선언한 오류였으며 제품 실행 전 실패다. page_count의 u32 계약에 맞춰 수정했고 이 실패를
제품 회귀로 세지 않는다. 이 API/feature 호환성 실행을 실제 시각 일치의 증거로 세지 않는다.
최종 archive/0.9.0 소비·fresh WASM·통합 비용과 T04 나머지 경계는 계속 남는다.

S16의 같은 최종 source는 rustfmt와 Native Skia + 숫자 feature 통합 library Clippy
`-D warnings`도 통과했다. 이 부분 결과를 로컬 커밋하고 다음 B04 경계를 계속 처리한다.


### B04 wrap projection과 폭 탐색 — S17, 검증 중

폭 탐색은 원본 font table·CharShape·ParaShape·기본 Style·TabDef를 복사한 임시
Document 대신, 같은 공통 character resolver와 새 fallible paragraph resolver가 원본
DocInfo를 빌려 읽는다. 기존 infallible 문단 스타일 해소도 같은 내부 함수와 수식을 쓴다.
탭 stop의 backing만 storage를 통해 예약하며 HWPUNIT 환산·CHAR 단위의 기본 글자 크기·
여백·줄 간격·탭 순서의 계산은 유지한다. 이 private projection은 Core 이전 PDF source다.
기존 빈 Document projection에는 쪽번호 스타일·번호·테두리·HFT 계보가 없었다.

문단 복사·각 trial의 source 복사·행 끝 목록·최대 64회 탭 탐색 큐의 65-entry backing과
결과 JSON에도 같은 요청을 전달한다. 공통 reflow를 그대로 호출하며 이 부분에서 다른
줄 나누기 알고리즘을 작성하지 않는다. 예약 실패는 tab 재탐색이나 source-reflow-unverified
진단으로 감추지 않고 fatal 상태로 전파한다. 원본 paragraph의 비직렬화 보조 값은 이 위치에서
비어 있다. 재조판이 만든 space metrics는 결과와 함께 이동하며 다음 trial의 source로
사용하거나 S14의 원본 positioned control 복사에 되돌리지 않는다. 공통 reflow 내부의
임시 tokenizer·frame 버퍼와 나머지 T04 경계는 이 보강과 구분해 계속 남는다.

첫 check는 fallible JSON macro의 Failure를 String 반환형으로 바꾸지 않아 컴파일에서
거절됐다. 명시적인 `?` 전파로 보정한 check는 통과했다. Native 부분 실행은 package
rhwp에만 opt-level=1/codegen-units=64를 적용한 release-test + native-skia이며 성능
측정이나 최종 제출용 기본 profile 실행으로 표시하지 않는다. library와 기존 gap 회귀
**3,888건 중 3,886 PASS / 2 FAIL**이었다. 두 실패는 공개 보고서 7쪽의 PDF import에서
`document decoded bytes`로 거절된 실제 회귀다. 기대값·정책 상한은 바꾸지 않았다.

S16의 typed primitive 64-byte 예약은 숫자가 실제 String을 소유하지 않는 기본 graph에도
적용된 과다 계상이다. 지원하는 serde_json 1.0.151/1.0.154의 inline numeric enum과
arbitrary_precision String의 차이는 Number의 compile-time needs_drop 성질로 구분한다.
숫자가 소유 storage를 가진 경우만 primitive decimal을 예약하고, 같은 representation의
Number protocol은 실제 JSON scalar이므로 가상의 map/field-key 비용을 예약하지 않는다.
일반 struct·map과 실제 숫자 string scanner의 선행 예약은 유지한다. 기본 graph의 값과
자원 한도는 고정한 채 같은 source 두 건과 집중 경계를 다시 실행 중이다. 통과 전에는
이 보정만으로 실패 원인 해소나 S17 완료를 선언하지 않는다.

기본 numeric representation에 대한 과다 계상을 제거한 뒤 기존 실패 두 건을 포함한 집중
19건과 확대 Native 3,888건이 모두 통과했다. 실제 CLI로 17개·49쪽에서 양쪽 형식의
34개 저장본·98쪽 재열기를 수행했고, 이전 제품과 저장 byte·RenderTree·자원 사용량을
제외한 진단이 모두 같았다. 별도 SHA-256 비교는 `native-byte-equivalence.json`에 남겼다.
이 제품의 CFF 1쪽은 raw 100%, Type3 영향 11쪽은 99.97900%였다. 같은 배율의 review와
독립 PDF를 직접 대조해 표 행·그림/캡션·내용 순서와 네 줄의 경계를 확인했다.

해당 제품과 `native-producer.json`은 마지막 style resolver의 pdf-import cfg 표기 전
source를 고정한다. cfg 표기는 해당 feature가 없는 구성의 dead-code를 막으며,
이후 current source의 lib/tests Clippy는 통과했다. 같은 AST라는 이유로 과거 제품을
최종 source의 새 제품으로 표시하지 않는다. fresh WASM과 최종 source 재실행은 남는다.

독립 serde_json 1.0.154 소비에 같은 공개 보고서의 선택 7쪽·상세 table 진단을 추가하자
여전히 decoded bytes에서 거절됐다. 기본 graph의 통과로 이 graph를 대신하지 않는다.
실제 1.0.151/1.0.154 Number primitive는 stack formatter의 결과 길이만 to_owned하며,
nonfinite float는 inline null이다. primitive의 일괄 64-byte 예약을 할당 없는 wire 길이
계수로 바꾼다. decimal protocol도 input byte + 16-byte scan buffer와 실제 doubling
교체 payload + 최대 20-byte 정수의 정규화 copy를 예약한다. 이전 5n+64보다 작지만
실제 소유 payload와 교체 작업은 빠뜨리지 않으며, 요청 상한은 올리지 않는다.
이 정밀 계상 후보의 실제 소비·양쪽 graph 경계 검사는 진행 중이다.

정밀 계상 후보에서 외부 serde_json 1.0.154의 선택 보고서 7쪽·9쪽과 일반 본문·CFF·Type3
총 5개 / 16쪽의 가져오기·양쪽 저장·재열기가 통과했다. 선택 7쪽의 decoded work는
262,080,906 byte로 기본 268,435,456 byte 안에 들어온다. 이전 기본 graph의 같은 상세
진단 경로는 258,515,458 byte였다. 두 graph를 같은 사용량이라고 발표하지 않는다.
`consumer-precise-producer.json`에 당시 source·consumer lock·harness·binary를 고정했다.
이 소비 제품에는 그 뒤 발견한 serialize_bytes의 내부 Number 생성 보강이 들어 있지 않다.

Value의 serialize_bytes는 primitive 방문 없이 Number를 직접 만든다. 이 경로에도 소유
숫자인 graph에서 각 byte의 실제 십진 길이를 먼저 예약했다. 이 마지막 보강을 포함한
root serde_json 1.0.151 + arbitrary_precision/raw_value의 집중 검사는 **17 PASS**다.
작은 decoded 한도로 배열의 내부 숫자 생성을 거절하고 새 요청으로 복구하는 경계를
포함한다. 앞선 16 PASS와 마지막 17 PASS는 다른 source이며 로그를 각각 보존한다.

S17의 마지막 source는 숫자 feature 통합 Native library/tests Clippy와 PDF feature를
제외한 `--no-default-features --lib` Clippy 모두 `-D warnings`로 통과했다.
명령은 `final-check-commands.json`, 실제 로그는 `number-clippy-final.log`와
`pdf-feature-off-clippy.log`에 남겼다. 이번 부분 보강을 로컬 커밋한다. Native 3,888건·
34개 파일 동등·2쪽 시각 결과, 정밀 숫자 소비 제품, 마지막 bytes guard 집중 검사는
각각의 source hash와 범위를 유지하며 최종 head의 통합 증거로 합쳐 발표하지 않는다.
T04 공통 reflow 내부 scratch·backend 후처리·Core/직렬화와 T05 이후 작업은 남는다.

### B04 glyph-origin 후처리 scratch — S18, 검증 중

실제 `extract_ordered_painted_spans_with_fonts` → `postprocess_spans_for_import` →
`postprocess_spans_inner(false, true)` → `stamp_char_x_offsets`에서 baseline index,
정렬 scratch, span glyph/후보 index/참조/anchor/offset 배열을 예약하지 않고 만들었다.
char 추출의 resource 오류도 일반적인 위치 정보 부재와 같은 fallback으로 삼킨 뒤
후속 배열 작업을 계속할 수 있었다. import의 최종 경계에서 sticky failure를 확인하는
것만으로 이 사이 할당의 선행 검사를 대신하지 않는다.

같은 matching·60% anchor 문턱·baseline 후보와 안정 정렬 tie·interpolation 수식을
공통 helper로 옮기고, ordinary/bounded 문서가 모두 이를 소비한다. bounded 경로의
Vec backing과 stable-sort scratch는 같은 ImportBudget의 decoded work로 먼저 예약한다.
일반 metadata/char 정보 부재의 fallback과 180도/회전 glyph의 기존 수용 조건은 유지하며
resource 오류는 Result로 전파한다. painted char Vec은 임시 Arc를 만들지 않고 빌려 쓴다.

새 검사는 순수 좌표 입력에서 작은 한도·sticky 오류·새 요청 복구와 실제 PdfDocument의
후처리 호출 경계를 보호한다. 새 조판 fixture·golden이나 픽셀 기대값을 만드는 변경이
아니다. 실제 제품 비교·회귀·lint 결과는 실행 뒤 기록한다. 다른 reading-order·annotation·
Unicode 후처리와 common reflow/Core/직렬화는 이 helper 보강으로 완료되지 않는다.

최초 root `-p backend` 단독 test 지정은 dependency가 workspace member가 아니고
별도의 dev-dependencies가 필요하므로 Cargo가 거절했다. 다음 nextest의 offline metadata는
미취득 optional dependency 때문에 멈췄다. 둘 다 제품 실행 전 배선 실패다. backend의
기존 manifest/lock을 직접 사용하는 cargo test로 전환했다. Cargo는 순차 실행한다.

S07에서 임시 harness로만 보완했던 `4662c069e557e928ed7968d061121fde90b1f027`의
StreamData 타입 6곳·xref budget 인자와 누락된 DejaVu 테스트 참조를 실제 cfg(test)
연결에 반영했다. glyph-origin/연관 검사는 **83 PASS**였다. 존재하지 않는 import_limits
module filter의 0건 실행은 통과 수에 포함하지 않는다. 같은 소스의 backend 전체 검사도
실행했으며 **5,871 PASS / 7 FAIL / 2 ignored**였다. 실패는 외부 font/image fixture 여섯 건과
fork 패키지명을 반영하지 않은 NAME 검사 한 건이다. 새 정렬 수식의 실패는 아니지만 전체
통과로 표시하지 않으며, 독립 backend test 실행의 누락된 입력/계약을 이어서 보완한다.

기존 metadata/GID CFF subset 검사에는 외부 face 대신 원본 합성 네 glyph CFF를 제공한다.
독립 fontTools 생성기의 재생성과 OTTO·cmap·glyph 순서를 대조했으며 1,236 byte와
SHA-256 `9fdefe03f069ef9543863b751a466f04c5bc3072c201b9569f7b036ecb372867`를 고정했다.
기존 CFF magic·GID 보존 기대는 유지한다. 이 입력은 조판/피델리티 fixture가 아니며 외부
font 프로그램·라이선스를 복제하지 않았다. font/shaping·HTML smoke는 이미 배포하는
DejaVu를 참조한다. CMYK white 10×11 writer 검사는 기존 dev JPEG encoder로 같은
입력 계약을 생성하고 file API는 tempfile을 이용한다. 크기·변환·태그 기대는 유지한다.
NAME은 Cargo manifest의 현행 fork 이름을 검사한다. 전체 backend 재실행 결과는 대기 중이다.

누락된 test 입력과 fork 계약을 보완한 실제 backend 전체 재실행은 **5,878 PASS / 0 FAIL /
2 ignored**다. `backend-all-units-self-contained.log`의 직접 manifest 실행이며 사본에만
테스트 adapter를 넣은 과거 S07 harness 결과와 다르다. 기대값·허용치를 완화하거나 실패
검사를 제외하지 않았다. 제품 feature 조합의 backend Clippy와 새 Native CLI 빌드·정상
대조는 별도로 진행한다. 이 전체 unit 결과를 제품 시각 검증으로 대신하지 않는다.

제품 codec feature를 켠 lib/tests Clippy에서는 그동안 lib-only 검사에 드러나지 않았던
JP2 단위 검사의 누락 fixture와 fallible upsample의 Result 연결 누락 두 곳을 확인했다.
이는 검사 컴파일 실패이며 decoder 실행의 성공/실패로 세지 않는다. 기존 상대 geometry·
sample-count·비평탄·upsample 픽셀 기대는 유지하고, 누락 원본 scan을 복원했다고 주장하지
않는 별도 합성 DeviceGray 816×1056 입력과 생성기를 제공한다. 독립 Pillow/OpenJPEG
재생성에서 byte가 같고 전 픽셀도 원래 `(3x+5y)%256`과 같음을 확인했다. 축소해도 target
해상도를 덮는 기존 decoder 계약은 실제 unit 실행으로 별도 검증한다.

JP2 검사 연결까지 보완한 제품 codec feature(`jbig2,jpeg2000,icc-qcms`)의 lib/tests
Clippy `-D warnings`가 통과했다. 같은 feature의 backend 실제 전체 unit은 **5,882 PASS /
0 FAIL / 2 ignored**다. `backend-product-units.log`에 보존했다. 앞선 feature-off 5,878과
구분하며, test-only 변경의 각 기대값과 원본 합성 자료의 생성 조건도 유지한다.

새 Native CLI는 제품 source `38a6fb67cd` + `native-producer.json`의 backend source
hash에서 빌드했다. root에만 opt-level=1/codegen-units=64를 적용한 기능 사전 검증이며
최종 기본 profile 성능 근거가 아니다. 17개·49쪽의 34개 저장·98쪽 재열기와 진단 비교가
통과했다. 저장 bytes도 이전 제품과 모두 같고 상세 보고서 3건도 resourceUsage를 제외한
내용이 같다. source와 제품 SHA-256은 producer, 별도 byte 대조는
`native-byte-equivalence.json`에 고정했다. 제품 binary는
`3873dd4ffc0e005c1676cc620705b255399f87557d7322a39bdfe7ab7afaf6a9`다.

같은 새 제품의 CFF 1쪽 raw 100%, Type3 영향 11쪽 99.97900%를 새로 산출했다. 같은
배율의 review와 독립 PDF에서 글자·표 행·그림/캡션·여백을 직접 대조해 이전과 같은
배치와 source glyph 경계를 확인했다. 이 두 쪽을 다른 문서 전쪽 또는 fresh WASM의
완료 근거로 대신하지 않는다. 외부 숫자 feature graph의 선택 보고서·저장 왕복은 별도
재실행 중이다. T04 전체 완료와 최종 시각·성능은 계속 보류한다.

추가 API/자원 대조로 공개 보고서 전체 12쪽의 기본 CLI 가져오기도 실행했다. S17 제품과
S18 후보의 저장 byte가 같고, 후보 decoded work는 699,227,067 byte다. 이 실행은 전체
12쪽의 새 피델리티 통과 선언이 아니다. S17 제품의 전체 상세 table 진단은 이미 decoded
한도에서 거절됐다(`report-whole-diagnostics.stderr`). 기본 가져오기의 통과를 같은 상세
경로의 통과로 표시하지 않으며, S18의 새 실패라고도 분류하지 않는다. 상세 진단의 최종
자원 비용은 B04 JSON/직렬화와 T08의 최종 검증 범위에서 계속 대조한다.

backend는 root workspace에서 제외되어 있고 기존 suite/workflow가 새 unit을 자동 수집하지
않는 점도 확인했다. 원래 테스트를 제거하거나 중복 CI 호출을 추가하지 않는다. T08에서
최신 base workflow·실제 test 목록과 비용을 대조한 뒤 필요한 최소 연결을 확정한다.

S18의 외부 serde_json 1.0.154 arbitrary_precision/raw_value graph도 선택 보고서 7쪽·9쪽과
일반 본문·CFF·Type3의 **5개 / 16쪽**, 양쪽 형식 **10개 저장·재열기**를 통과했다.
선택 7쪽의 decoded work는 262,218,934 byte로 현재 정책 안에 남는다.
`consumer-producer.json`과 각 실제 실행 로그에 source·lock·제품 hash를 고정했다.
마지막 source의 rustfmt·diff 검사도 통과했다. 이 부분 결과를 로컬 커밋하고 같은 B04의
reading-order 정렬 scratch와 복사 경계를 이어서 보강한다. 최적화 fresh WASM·전체
postprocess/Core/직렬화 및 최종 CI 연결의 잔여를 완료로 바꾸지 않는다.

### B04 행 정렬 scratch와 소유 값 이동 — S19, 검증 중

`postprocess_spans_inner`의 일반·XY-cut fallback 정렬은 같은 row-key 생산자에 요청
storage를 전달한다. 기존 row seed·격자·baseline/top 거리·점유 겹침·동률 규칙은 유지한다.
font-size mode는 같은 반쪽 pt key의 정렬 그룹으로 계산해 빈도 최대·동률 최소 key를
고른다. opaque hash backing을 만들지 않으면서 원래 결정 규칙을 유지하는 공통 구현이다.
일반 utils 소비자도 같은 내부 함수를 사용한다.

row key·font-size key·정렬 순서·row 소속·후보 배열과 행/멤버 확장의 교체 backing을
실제 생성 전에 예약한다. 정렬된 span은 복제 Vec·clone_from_slice 대신 검증한 permutation으로
소유 값을 이동한다. index 크기·중복·범위를 검증하고 destination scratch의 예약이 끝난 뒤
이동하므로 실패 전에 원본 소유 문자열을 복제하거나 배열을 바꾸지 않는다. 이와 별개인
rowspan 승격·vertical/sidebar/XY-cut 자체·annotation/Unicode·common reflow/Core·직렬화는
계속 감사할 경계다.

순수 소유 값의 permutation·실패 전 원본 보존과 source row의 동행/다른 행 관계, 작은
scratch 한도·sticky failure·새 요청 복구를 추가했다. 새 조판 fixture나 px 기대값을 만들지
않았다. backend check와 제품 codec feature의 전체 unit **5,885 PASS / 0 FAIL / 2 ignored**를
확인했다. 실제 Native 제품·진단·저장 대조와 lib/tests Clippy는 진행 중이다.

S19의 제품 codec lib/tests Clippy `-D warnings`와 새 Native CLI 빌드가 통과했다. 같은
기능 사전 검증 profile에서 17개·49쪽, 34개 저장·98쪽 재열기와 진단(자원 사용량 제외)이
이전 제품과 같고 저장 bytes도 모두 같다. CFF 1쪽 100%, Type3 영향 11쪽 99.97900%를
새로 확인하고 직접 판독했다. 최종 기본 profile 성능 검증은 아직 아니다.

현재 Native library의 실제 Cargo artifact JSON에서 rlib와 root의 raw_value serde_json,
native link 경로를 확정해 기존 역편집 case와 같은 단언의 별도 capture를 연결했다. 첫
준비의 host/target serde_json 구분과 CARGO_MANIFEST_DIR 누락은 제품 실행 전 harness
배선 실패였고, 보정 후 실제 capture가 통과했다. 공개 보고서 원본 7쪽·9쪽의 세 경로와
원본/역편집 상태 12개를 새로 출력했다. 전 상태의 raw 최저는 각각 **99.96087% /
99.85041%**다. HWPX 첫 재열기와 역편집을 확대한 `native-p7-hwpx-detail.png`에서 선행
띠·탭 뒤 시작·네 줄·다음 셀 소속을 독립 source PDF와 대조했다. 제목의 그림/캡션·표
외곽과 기존 source font 대체 차이도 확인했다. 현재 Native에서 내용 누락·중복·원점
후퇴는 발견하지 않았다. 이 결과를 새 fresh WASM과 직접 대조하는 작업을 이어서 수행한다.

이 부분 source를 로컬 커밋한다. 그 뒤 같은 source의 최적화 WASM을 새로 만들어 실제
브라우저에서 이전 T03 제품의 HWP 역편집/HWPX 첫 열기 회귀 해소를 검증한다. S19만으로
다른 reading-order 분기·reflow/Core/직렬화나 T04 전체 완료를 선언하지 않는다.

### S19와 S15 역편집 보정의 fresh WASM 확인

source `136b4d4878`에서 locked wrapper·codegen-units=16으로 web 제품 WASM을 새로 빌드하고
wasm-opt까지 완료했다. 동결한 bundle과 Studio `public/`의 JS/WASM이 같다.
WASM은 **23,715,663 byte**, SHA-256은
`3cba38db56cd9fa185d13d0ff9bf72c7ccb51b43d52fa5c276a4b43e97b24bab`이다.
T03의 23,497,145 byte와 비교해 218,518 byte(약 0.93%) 늘었다. 이 비교는 S10–S19를
포함한 bundle 크기이며 S19만의 비용이나 실행 속도 결과로 발표하지 않는다.
처음 producer 준비에서 Studio의 실제 public 경로를 잘못 지정한 파일 부재를 보정했으며
제품 빌드 자체는 성공했다. wasm-producer.json에 정확한 경로·source hash를 기록했다.

기존 case와 같은 Print·`setHangul2024Compat(false)` 조건의 Headless Chrome
155.0.8059.79에서 동일 PDF 선택 7쪽·9쪽을 가져오고 HWP/HWPX 저장·
재열기·삽입→삭제를 실제 실행했다. 여섯 경로 모두 역편집 글자 원점의 dx/dy가 **0**이며,
HWP/HWPX 첫 열기의 첫 유효 글자 원점도 가져오기 직후와 **0** 차이다. 이전 T03 제품의
HWP 역편집 약 31px 이동과 HWPX 첫 열기의 선행 띠 소실은 이 새 제품에서 재현되지 않는다.
첫 열기와 역편집을 구분한 browser-gap-candidate.json에 근거를 남겼다.

새 WASM SVG 12개에 canonical font policy로 현재 Native의 같은 쪽 font-face 규칙만
공급했다. 원본 SVG와 font policy를 각각 해시 고정하며 기하를 옮기지 않았다. 새 PNG·
전 상태 TSV에서 7쪽 raw 최저 **99.96087%**, 9쪽 **99.85041%**를 확인했다. WASM의
HWPX 첫 열기/역편집 확대와 제목 쪽 review를 독립 PDF·현재 Native와 직접 대조했다.
선행 여백·탭·줄 소속·그림/캡션과 외곽선이 유지되며 기존 source font 대체의 획 차이는
남는다. Native/WASM PNG byte가 같은 것은 12개 중 7개이며, 전부 동일하다고 발표하지 않는다.

이 실행은 기존 역편집 회귀와 현재 B04 부분 source의 실제 fresh WASM 확인이다. 최종
head 전체 문서·Studio UI·한도/취소·속도·메모리와 남은 B04/B07의 통합 완료를 대신하지 않는다.

source `136b4d4878`의 외부 숫자 feature graph도 같은 5개·16쪽과 양쪽 형식 10개 저장·
재열기를 다시 통과했다. 선택 7쪽의 decoded work는 262,218,934 byte다. 이 페이지의
값이 S18과 같다는 사실을 다른 reading-order 분기의 예약 완료로 해석하지 않는다.
S19가 보강한 것은 일반 row sort와 XY-cut fallback이며 다른 분기의 호출 경계는 남는다.
이 Native/fresh WASM 부분 증거를 로컬 기록에 커밋하고 다음 후처리 경계를 계속 처리한다.

### B04 rowspan label 승격의 저장소 — S20, 검증 중

일반 row sort 다음과 XY-cut fallback의 `reorder_rowspan_labels`를 같은 fallible producer로
연결했다. 기존 X 근접 군집·dense column·공통 band·번호 목록 제외·sparse label 조건·
Y 군집/anchor·최종 안정 정렬의 규칙은 유지한다. 원래 단독 정보가 있는 각 column의
band를 `(band, column rank)`로 정렬·중복 제거해 support를 계산하며, 같은 column의 여러
run을 여러 column의 지지로 세지 않는다. 같은 dense 1/2/3-column 계약을 검사한다.

column/label/cluster·후보·정렬 scratch의 생성과 성장 전에 요청을 전달한다. dense band와
번호 제외 목록은 정렬된 index 목록으로, 승격 anchor는 source index의 Option 배열로
보관한다. 마지막 span은 S19의 검증한 permutation으로 이동한다. 일반 legacy 호출도
같은 계산을 사용하며 이 경로와 별개의 XY-cut 성공·vertical/sidebar·Unicode/Core 경계를
완료로 확대하지 않는다. 일반 iterator collect도 lower hint와 매 성장 전 backing을 예약하며
종료 시 fatal 상태를 다시 확인한다. 현재 source의 check는 통과했고 실제 전체 unit·lint·
Native 저장/출력 대조를 계속 실행한다.

S20의 제품 codec 전체 unit은 **5,887 PASS / 0 FAIL / 2 ignored**이며 같은 feature의
lib/tests Clippy `-D warnings`도 통과했다. 새 Native 제품으로 17개·49쪽의 34개 저장과
98쪽 재열기, 상세 진단 3건을 이전 제품과 대조했다. 저장 bytes·RenderTree와 자원 사용량을
제외한 진단이 모두 같다. CFF 1쪽 100%·Type3 영향 11쪽 99.97900%를 새로 산출하고
직접 판독해 글자·행·그림/캡션·내용 순서와 외곽선이 유지됨을 확인했다. 사용한 profile은
이전과 같은 root opt-level=1/codegen-units=64의 기능 사전 검증이다. 최종 기본 profile
속도 결과나 이 source의 새 WASM 결과로 표시하지 않는다. 해당 producer·byte 대조·로그는
`v2-s20-20261026`에 보존한다. 이 부분 변경을 로컬 커밋하고 다음 실제 후처리 경계를 이어간다.

### B04 publisher sidebar 분류와 문자열 — S21, 부분 완료

매 페이지에서 호출될 수 있는 sidebar 후보 판정의 line/member·histogram·band/body/sidebar·
line-count·정렬 배열을 같은 요청 storage로 연결했다. font/기하 판별·gutter·histogram 동률·
문자열 키워드 규칙과 최종 순서를 유지한다. 결과는 source index permutation으로 반환해
bounded postprocess가 소유 span을 그대로 이동한다. 기존 immutable legacy 소비자는 같은
순서를 받아 기존 Vec 계약으로 복사하며 별도 판정식을 만들지 않는다.

분류 문자열의 Unicode 소문자 변환은 실제 Rust 1.93.1의
[alloc::str](https://github.com/rust-lang/rust/blob/1.93.1/library/alloc/src/str.rs),
[String::push](https://github.com/rust-lang/rust/blob/1.93.1/library/alloc/src/string.rs)와
[RawVec](https://github.com/rust-lang/rust/blob/1.93.1/library/alloc/src/raw_vec/mod.rs)를 대조했다.
원문 길이의 최초 buffer와 2배/최소 8-byte 교체 buffer를 delegate 전에 예약한다. 유일한
언어 독립 contextual mapping인 sigma의 두 결과는 UTF-8 길이가 같으므로, scalar lowercase
순회로 할당 없이 출력 길이를 측정하되 실제 변환은 std에 맡긴다. 전체 sidebar 문자열도
그 측정 길이 합으로 먼저 예약한다. 이 계상은 실제 지원 compiler의 payload work이며 RSS
상한이나 미래 Rust 버전의 내부 할당 계약으로 확대하지 않는다.

publisher metadata와 body의 동일 baseline을 가진 합성 계약에서 각 source span의 보존·
body 뒤 metadata 순서를 확인하고, 작은 budget의 fatal 오류가 단순 not-matched로 변하지
않는지와 새 요청 복구를 검사했다. Unicode sigma/확장·ASCII의 선행 예약도 포함한 제품
codec 전체 unit은 **5,890 PASS / 0 FAIL / 2 ignored**, check도 통과했다. 실제 allocator
관측·lib/tests Clippy·Native 대조는 이어서 실행한다.

S21의 실제 allocator probe는 Unicode 단일 scalar 1,112,064개와 contextual/확장/growth
84건에서 선행 계상 미달 0건·길이 불일치 0건·계획 순회 중 heap 할당 0건이었다.
같은 source의 lib/tests Clippy도 통과했다. 새 Native에서 34개 저장 파일의 bytes,
98쪽 재열기 RenderTree와 자원 사용량 외 진단·상세 진단 3건이 이전 source와 같았다.
CFF 1쪽 100%·Type3 영향 11쪽 99.97900%를 새로 산출하고 글자·행·표·그림/캡션을
직접 대조했다. producer·원시 로그·TSV·compare/overlay/review는 ignored
`output/pr-review/issue8174/v2-s21-20261026/`에 보존한다. 기능 검증용 profile이며
최종 성능 또는 이 source의 fresh WASM 검증으로 확대하지 않는다. 남은 다단/XY-cut·
vertical/rotation·Unicode·공통 reflow/Core 경계의 감사는 계속 진행한다.

### B04 다단·글꼴 통계·세로쓰기·회전 정렬 — S22, 부분 완료

실제 positional import의 sidebar 다음 다단 판별에 요청 storage를 전달했다. clean gutter의
행과 persistent corridor의 행은 원래 BTreeMap의 key 순서·행 안 입력 순서를 보존하는 정렬된
배열로 표현한다. bimodal 시작점은 TextSpan 전체를 복사하지 않고 빌린 span을 정렬한다.
peak/gutter·격자/본문 임계값·font-size 동률과 corridor 수식은 바꾸지 않았다.

`PageFontStats::try_from_spans`는 같은 기본 API의 공통 생산 경로다. size/font의 문자 가중
mode를 정렬된 group으로 계산하고, 작은 size·사전순 작은 이름의 동률 선택과 원래 line-height/
char-width 계산을 유지한다. 문자열·배열·stable sort scratch를 먼저 예약한다. 기존 mode
HashMap의 평균 O(n) 대신 grouping O(n log n)이므로 최종 성능 비교 대상으로 명시한다.

세로쓰기는 같은 column clustering·안정 정렬 결과를 source permutation으로 적용해
`Option<T>`와 출력 배열을 따로 만들지 않는다. 회전 firewall도 angle group과 upright 정렬의
index만 만들고 horizontal 뒤에 원래 회전 group 순서를 적용한다. 이전 Vec 계약과 실제
bounded 호출이 같은 규칙을 소비하며 source-owned 문자열/좌표를 복제하지 않는다.

한도 실패를 single-column 판정·기본 통계로 바꾸지 않는지, 실패 전 입력 보존과 새 요청
복구, Clone이 없는 세로쓰기 payload 이동을 추가한 제품 codec unit은 **5,894 PASS /
0 FAIL / 2 ignored**다. Clippy는 테스트 모듈 배치 2건을 지적해 모듈을 뒤로 이동한 뒤
lib/tests `-D warnings`를 통과했다. Native 실제 저장·출력 비교를 이어서 수행한다.

S22 Native에서 34개 저장 bytes와 98쪽 재열기 RenderTree, 자원 사용량 외 진단·
상세 진단 3건이 S21과 같았다. 현재 source의 CFF 1쪽 100%·Type3 영향 11쪽
99.97900%를 산출하고 글자·행·표·그림/캡션과 내용 순서·포함을 직접 대조했다.
source별 digest·제품 hash·TSV·PNG·로그는 `v2-s22-20261026`에 보존한다.
기능 검증 profile이며 final 성능이나 S22 fresh WASM 검증으로 표시하지 않는다.

### B04 XY-cut 제목·재귀·결과 저장 — S23, 부분 검증

`postprocess_spans_inner`의 실제 XY-cut 성공 경로에 같은 요청 storage를 전달했다.
Err fallback 앞에서 resource error와 sticky 실패를 전파한다. fallback용 입력 사본은 기존
공통 `copy_import_spans` 예약을 소비하며 새 별도 사본 정책을 만들지 않는다.

제목 font-size/reading-order/run·synthetic/origin·합친 문자열과 최종 span slot/output을
생성 전에 예약한다. 제목의 두 번째 서로 다른 rounded baseline 유무는 전체 BTreeSet 대신
첫 baseline과 다른 값의 존재로 판정해 같은 admission을 보존한다. synthetic TextSpan의
문자열·glyph advance/origin 사본도 실제 소유 payload를 먼저 계상한다. 재귀의 source index
partition과 stable sort/row-snap, leaf wrapper·자식 group 결합/성장도 같은 fallible 생산자를
사용한다. 깊이 64·작은 region·기하/내용 판별과 branch 순서는 유지한다.

새 요청 복구·소유 payload 사전 거절과 제목/분할 결과 보존을 포함한 제품 codec unit은
**5,896 PASS / 0 FAIL / 2 ignored**다. Clippy에서 지적한 doc comment 뒤 빈 줄은
보정했다. 나머지 projection/valley/행 분류 내부 scratch는 다음 보강 범위이며, 이 부분
결과를 전체 읽기 순서의 예약 완료로 해석하지 않는다. Native/fresh WASM의 최종 출력·비용은
후속 경계를 포함한 source에서 실제 재검증한다. 원시 로그는 `v2-s23-20261026`에 보존한다.

S23 제품 codec lib/tests Clippy `-D warnings`도 통과했다. unit·lint 통과분을
로컬 커밋하고 projection·행 분류 보강을 이어간다. Native 제품 비교는 연속된 읽기 순서
보강을 포함한 다음 source에서 수행하며, 이번 절편에 실행하지 않은 결과를 기재하지 않는다.

### B04 XY-cut 투영·계곡·행 분류 — S24, 부분 완료

같은 Result 호출 그래프의 classifier·short-line corridor·두 prose detector·single-column
판별·horizontal/vertical split에 요청 storage를 전달했다. 실제 density·valley/interior/merged·
교차 여부·prefix·cluster·행별 정렬 사본과 TLS smoothing scratch의 capacity 성장을
생성 전에 예약한다. 판별 실패 `None/false`와 요청 실패 `Err`를 구분하며 recursive node가
자원 오류를 다시 row-aware 성공으로 바꾸지 않는다. 첫 split이 성공하면 다음 split을
실행하지 않는 기존 순서도 유지한다.

행별 BTreeMap은 stable key 정렬 뒤 같은 입력 순서의 값을 모으는 공통
`ImportStorage::grouped_values`로 바꿨다. 각 그룹의 min/max/문자 가중 합과 순회를 원래
ascending key·source insertion 순서로 유지하며 트리 node 크기를 추정해 예약하지 않는다.
public legacy `partition_region`/단독 도우미는 별도 계약으로 남고 실제 positional import의
성공 경로는 fallible graph를 소비한다. recursion 64·projection 100,000·gutter/표/제목의
판별식과 동률, source span의 실제 text/geometry는 바꾸지 않았다.

제품 codec 전체 **5,898 PASS / 0 FAIL / 2 ignored**, lib/tests Clippy
`-D warnings`를 통과했다. S23와 S24를 합친 새 Native에서 34개 저장 bytes·98쪽
재열기·상세 진단 3건이 S22와 같다. 공개 source 7·9쪽의 상세 표 진단을 켠 HWP/HWPX
추가 4개 저장·재열기도 이전 source와 bytes/RenderTree·자원 사용량 외 진단이 같다.
현재 CFF 1쪽 100%·Type3 영향 11쪽 99.97900%를 새로 산출하고 글자·행·표·그림/
캡션을 직접 판독했다. producer·비교·TSV·PNG는 `v2-s24-20261026`에 보존한다.
기능 검증 profile이며 최종 성능/fresh WASM 결과로 확대하지 않는다. 다음 Unicode/annotation·
공통 reflow/Core/직렬화 경계와 최종 통합 검증을 계속한다.

### B04 Unicode/combining/decimal 후처리 — S25, 부분 완료

positional import의 기존 decimal 수선·Unicode space 정규화·NFC combining 처리를
같은 fallible 생산자로 연결했다. 원래 decimal 인접 조건과 공백 생략, Unicode space 범위·
zero-width 제거·결합 mark/기하 조건을 보존한다. Char Vec·문자열·boundary split buffer와
정규화 scratch를 생성 전에 예약한다. erase-region Vec 복사는 같은 mutex guard 안의
불변 region slice를 소비하게 바꿔 제거했다. 기존 public/legacy 소비자는 같은 규칙을 사용한다.

실제 lock의 unicode-normalization 0.1.25·tinyvec 1.12.0과 Rust 1.93.1의 buffer
성장 경로를 대조했다. allocation 없는 canonical decomposition 순회로 mark-run과 한
scalar의 최대 atomic decomposition 길이를 측정한다. decomposition/recomposition의
TinyVec spill·교체와 disjoint pending-run sort를 먼저 예약한 뒤 실제 NFC는 기존 라이브러리에
맡긴다. 출력은 canonical scalar 수 × 4의 UTF-8 상한으로 미리 예약한다. 범용 heap/RSS
상한이 아니라 현재 검증 graph의 요청 payload work다.

첫 allocator probe는 Unicode scalar 1,112,064개·240개 조합을 통과했으나, 결합 문자 뒤
한글을 추가한 600개 조합에서 준비 중인 `mark_run + 2` 식의 미달 **16건**을 검출했다.
한 입력 scalar의 전체 Jamo 분해가 yield 전에 들어오는 실제 호출을 반영해
`mark_run + largest_atomic_expansion`으로 보정했다. 확장 전 실패 source·로그를
보존했으며 같은 1,112,064개·600개 재실행은 **미달 0·출력 상한 불일치 0·계획 중 heap
할당 0**이다. `A + 두 mark + 각`은 실제 scratch 64 bytes, 제품이 제어하는 출력 buffer
24 bytes를 관측했다. 80-byte 요청의 거절과 fresh request 복구를 정식 resource 검사에
추가했다. 정상 Unicode 변환을 바꾸거나 예산을 높여 맞추지 않았다.

보정 뒤 제품 codec unit은 **5,902 PASS / 0 FAIL / 2 ignored**다. lib/tests Clippy와
Native 저장/출력 대조는 이어서 수행한다. probe source/컴파일러/실제 dependency artifact
SHA와 실패·재실행 로그는 `v2-s25-20261026`에 보존한다.

S25 lib/tests Clippy `-D warnings`와 새 Native 제품 검증도 통과했다. 34개 저장
bytes·98쪽 재열기·상세 진단 3건, 공개 source 7·9쪽의 상세 진단 포함 HWP/HWPX
4개 추가 저장·재열기에서 이전 source와 의미·내용·RenderTree가 같다. CFF 1쪽 100%·
Type3 영향 11쪽 99.97900%를 새로 산출하고 현재 PNG와 확대 crop을 직접 대조했다.
원시 TSV·PNG·제품/source hash·before/candidate 결과는 같은 S25 경로에 보존한다.
기능 검증 profile이고 최종 성능/fresh WASM은 남는다.

### B04 annotation 문자열·appearance stream — S26, 부분 완료

실제 postprocess의 annotation 수집은 fallible Result를 반환하며, contents/appearance
추출 중 request 실패가 empty/None으로 바뀌지 않는다. 직접 Annots·annotation dict·Rect는
소유 원본에서 빌려 불필요한 clone을 제거했다. AP/Resources의 필요한 소유 사본은 기존
`object_copy` 예약을 소비한다. AP extractor는 원래 `set_document(self)`의 같은 budget을
소비하며 폰트·추출의 구조 오류 fallback 뒤에도 request 실패를 먼저 확인한다.

UTF-16 BE/LE와 PDFDocEncoding은 공통 fallible producer가 처리한다. 임시 u16 Vec
대신 std decode iterator를 사용하고 출력 buffer를 먼저 예약한다. 손상 UTF-16의 원래
전체 byte UTF-8 lossy fallback과 홀수 trailing byte 계약을 유지한다. lossy 변환도 std
UTF-8 chunk 분할을 소비해 정확한 출력 길이를 먼저 검사한다. trim·AP text join·annotation
결과 span/append buffer도 같은 요청을 소비한다. subtype의 ASCII 대소문자 비교로 분류용
문자열 사본을 제거했으며 보이는 FreeText/Stamp, hidden flags, Rect 조건은 바꾸지 않았다.

양쪽 endian에서 **1,112,064개 Unicode scalar 각각의 왕복**과 손상/홀수 byte 계약을
검사했다. Contents와 유효한 AP/Font 객체를 분리한 annotation-only PDF 대조군에서 실제
문자열 추출, budget 거절·fresh 요청 복구도 실행했다. 합성 메타데이터 계약이며 PDF
annotation의 새 조판/시각 지원 완료 주장으로 바꾸지 않는다. 제품 codec 전체
**5,906 PASS / 0 FAIL / 2 ignored**·lib/tests Clippy `-D warnings`가 통과했다.

새 Native에서 34개 저장 bytes·98쪽 재열기·상세 진단 3건과 source 7·9쪽의 HWP/HWPX
추가 4개 왕복이 직전 source와 같다. 현재 CFF 1쪽 100%·Type3 영향 11쪽 99.97900%
TSV와 새 PNG/확대 crop을 산출·직접 대조했다. 원시 증거와 producer는
`v2-s26-20261026`에 보존한다. 최종 성능/fresh WASM은 남으며 다음 공통 reflow/Core/
직렬화 경계를 계속 처리한다.

### B04 공통 frame scratch 선행 추적 — S27, 착수

`LayoutFrame::carve`의 가변 결과는 top/current_intervals/next_geometry_event이며
`commit_carved_row`는 row append·top 전진·interval clear다. 일반 fill의 각 row/최대
8회 높이 trial이 이 상태를 되돌리려고 모든 기존 rows·segments·exclusions까지 복제한다.
저장 row admission과 equation-prefix/stale-cache 입구의 restore 소비자도 확인했다.
각 경로는 carve/append만 하고, whitespace absorption은 실패가 없는 성공 뒤 새 row
범위에만 적용한다. 따라서 행 수와 바뀌는 geometry/interval을 보존하는 공통 append
checkpoint로 복제 범위를 줄일 수 있다. 실제 조판 규칙·cached row admission·kerning
fallback·높이/좌표 계산을 바꾸는 보정은 아니다. 적용/검증 전의 근거이며 결과는 아직 없다.

S27는 append checkpoint를 공통 frame에 구현했다. 이전 행 payload와 immutable
configuration은 원래 frame에 남기고 row count/top/interval/geometry event만 보존한다.
실제 다섯 snapshot·restore 소비자를 함께 전환했으며 row trial에서 snapshot을 매번
복제하지 않는다. interval buffer 생산·복구에는 기존 공통 Storage의 fallible reserve를
받는 진입점을 마련했다. 이 진입점이 현재 PDF reflow 전체에 연결됐다는 주장은 아니다.

128개 기존 row가 있어도 checkpoint payload가 같은 interval 수에만 의존하는지,
checkpoint/restore 예약 실패가 현재 frame을 바꾸지 않는지와 기존 row의 정확한 보존을
검사했다. Native frame 집중 **11 PASS**, Native library suite **4,070 PASS /
12 skipped**, library Clippy `-D warnings`를 통과했다. 공통 줄 구성·메트릭·admission·
kerning fallback 계산은 유지한다. source/hash·실행 로그는 `v2-s27-20261026`에 보존한다.

S26 AP 대조군의 appearance stream에 명시적 Form subtype/BBox와 내부 좌표를
보완했다. 문자의 기대값을 바꾸지 않았고, 보정한 메타데이터 입력으로 backend 제품 codec
전체 **5,906 PASS / 0 FAIL / 2 ignored**를 다시 통과했다. 이 fixture 보완과 Root의
생산 로직 변경을 구분한다. S27 Native CLI/fresh WASM·시각/비용은 연속된 공통 producer
보강을 포함한 source에서 검증하며, S26 제품 결과를 S27 결과로 바꾸어 표시하지 않는다.

### B04 공통 토큰화 producer — S28, 부분 완료

공통 Storage에 UTF-8 출력과 iterator 수집을 추가하고 실제 문자 수·출력 byte 수를
먼저 예약한다. 토큰·문자별 폭·supplemental mask는 같은 fallible producer를 사용한다.
단어를 다시 String과 문자 배열로 복사하던 두 경로는 원본 문자 slice를 빌린다.
폭만 필요한 소비자는 사용하지 않는 문자별 폭 배열을 만들지 않는다. 단일 scalar의 PUA
표시는 기존 공통 변환 규칙과 stack UTF-8 buffer를 사용한다. 언어별 폭 계산·공백 정책·
하이픈 분할 순서는 유지한다.

실제 PDF Allocation으로 작은 예산의 토큰 생성 거절과 새 요청 복구를 실행하고 일반
producer와 토큰 값의 동등성을 확인했다. 새 조판 기대값이나 시각 fixture는 추가하지
않았다. Native library **4,071 PASS / 12 skipped**, library Clippy `-D warnings`를
통과했다. source hash·초기 컴파일 오류와 보정 후 실행 로그는 `v2-s28-20261026`에
보존한다. 아직 PDF wrap trial에서 공통 reflow 전체로 Storage를 전달한 결과가 아니며,
frame·reflow·Core·직렬화 연결과 최종 제품/시각 검증은 계속 남는다.

### B04 공통 frame buffer — S29, 부분 완료

carve의 구간 배열·분할 결과, 저장 row admission의 구간/segment와 확정 row append,
공백 줄 흡수의 문자 배열 및 LineSeg/공백 metric 출력을 공통 Storage로 예약하는
진입점을 추가했다. 일반 API도 같은 producer를 Unbounded로 호출한다. carve의 비교용
전체 interval 복제는 제거했으며 판정은 원래 interval과 새 결과를 직접 비교한다.
suffix projection은 앞선 모든 row를 출력한 뒤 다시 복사하지 않고 요청한 row부터
한 번 출력한다. geometry·row 수용·메트릭·태그 계산은 유지한다.

request 오류는 cache miss와 구분한다. admission 실패는 이미 소유한 checkpoint를
소비해 추가 예약 없이 복구하므로 sticky budget 실패 뒤에도 원래 frame을 보존한다.
carve/commit/출력/공백 처리의 예약 실패 전 상태 보존, admission 중 실패 복구와
128개 row 중 마지막 row만 출력하는 예약량을 검사했다. 조판 baseline 추가가 아닌
자원/메타데이터 계약이다. Native library **4,073 PASS / 12 skipped**, library
Clippy `-D warnings`를 통과했다. source hash·실행 로그는 `v2-s29-20261026`에
보존한다. PDF reflow 전체 연결과 최종 Native/fresh WASM 제품 대조는 계속 진행한다.

### B04 PDF wrap → 공통 reflow 실제 연결 — S30, 부분 완료

실제 `wrap_width::Trial::run`이 clone 이후 `try_reflow_line_segs`에 같은 Allocation을
전달한다. 공통 tokenizer·문자별 자간·scalar fill 결과·frame checkpoint/carve/trial/
commit/출력까지 Result를 이어서 전달하며 request 실패가 일반 조판 miss로 바뀌지 않는다.
일반 문서 API는 같은 producer를 Unbounded로 호출한다. frame의 실패 복구는 이미
소유한 checkpoint를 소비하므로 요청이 실패한 뒤 새 예약을 요구하지 않는다.

일반 scalar reflow는 frame 진입 전에 만들고 버리던 문자/토큰/커닝 준비를 생략한다.
기존 frame 자격 조건 중 prefix가 비어 있다는 조건은 prefix 편집 요청 자체가 없다는
조건으로 이미 보장된다. frame이 수용하지 못하면 기존 scalar fallback을 실행한다.
줄 나눔·공백 정책·수용·메트릭의 계산 순서는 각 producer 안에서 유지한다.

문자 측정에서 복제하는 표시/metric face·원본 glyph metric Arc·영문 슬롯 Box도 공통
fallible 스타일 producer로 먼저 예약한다. 단일 문자 자간의 UTF-8은 stack buffer를
사용한다. PDF wrap의 source는 `structured_lowering::paragraph`가 만든 controls/field
없는 문단이며, projection은 kerning/shaping context가 없는 로컬 스타일이다. 개체 투영·
커닝 transaction 같은 일반 문서의 다른 경로 전체가 이번에 제한됐다는 주장으로 넓히지
않는다. Core 초기화·직렬화와 불투명한 font/renderer 작업의 감독은 여전히 남는다.

모델 복사까지만 허용하는 예산에서 실제 wrap trial이 reflow의 첫 backing 예약을 거절하고,
새 요청에서는 같은 LineSeg/공백 metric을 생성하는지 검사했다. 긴 글꼴 이름은 복사 전에
거절하며 새 요청에서 두 언어 슬롯의 전체 TextStyle이 일반 producer와 같다. 새 조판
baseline은 추가하지 않았다. Native library **4,075 PASS / 12 skipped**, library
Clippy `-D warnings`와 새 기능 검증용 Native 빌드를 통과했다. 스타일 guard 전
**4,074 PASS**와 첫 closure 컴파일 오류는 별도 로그로 보존하고 최종 결과와 구분한다.

현재 Native에서 **34개 저장 bytes·98쪽 재열기·상세 진단 3건**, 공개 source 7·9쪽의
HWP/HWPX 추가 4개 저장·재열기·상세 진단이 S26과 같다. 새 CFF 1쪽 **100%**, Type3
영향 11쪽 **99.97900%** TSV와 review/overlay를 산출하고 현재 PNG·동일 좌표 확대 crop을
직접 대조했다. Type3의 source-row 원점·셀/표 외곽·문자 순서·각도/간격과 그림/캡션의
기존 상태를 유지한다. 현재 Native page PNG도 S26과 byte 단위로 같다. font 획 차이를
새로 해결했다는 주장은 아니다.

source 7쪽 상세 진단의 논리적 decoded 계상량은 **260,090,471 → 260,787,963 bytes**다.
추가 작업을 기존 기본 한도 안에서 수용했고 9쪽은 171,470,972 bytes로 같다. 이 값은
누적 논리 작업량이며 RSS나 실행 시간으로 해석하지 않는다. 최종 성능과 현재 source의
fresh 최적화 WASM/Studio·전체 쪽 시각 검증은 T08에 남는다. 원시 결과·명령·전체
source/binary hash·TSV·현재 PNG는 `v2-s30-20261026`에 보존한다.

같은 source의 `--no-default-features --lib`와 `--target wasm32-unknown-unknown --lib`
컴파일도 통과했다. WASM 컴파일 검사는 새 최적화 bundle의 실제 브라우저 실행을 대신하지
않는다. 다음 절편은 parser 요청을 Core 초기화·첫 저장까지 이어 주는 수명과 실제 예약이다.

### B04 Core 초기화 요청과 스타일 목록 — S31, 부분 완료

`PdfImportSession`이 parser와 Core 초기화 동안 같은 request를 소유한다. 기존
`from_pdf_bytes_with_policy`와 암호 없는 PDF 자동 감지 경로가 이 세션을 사용하며
provenance 봉인은 기존 parser와 같다. 사용한 세션은 다시 open할 수 없고 실패 복구는
새 세션을 사용한다. Core에는 누적 import 카운터를 넣지 않으며, 초기화 후 metadata의
snapshot을 갱신한다. 첫 저장을 같은 요청으로 실행하는 연결은 아직 남는다.

Core 스타일 목록은 기존 single-style producer를 공통 fallible 목록 producer에서
호출한다. 문자/문단/테두리 배열·번호/글머리표 소유 복사·native PUA 기본 저장소·fallback
face·dirty section 배열을 먼저 예약한다. gradient의 positions/띠 출력과 Box도 같은
공통 producer로 예약하며 색상/위치 계산은 유지한다. 결측 영문 face의 DocInfo 복사는
공개 Serialize 계약을 추가하지 않고 기존 소유 필드와 별도 provenance digest 배열을
명시적으로 검사한다. 필드 패턴은 exhaustive라 새 소유 필드가 생기면 컴파일로 드러난다.
이 복사 helper의 세부 예산 반례는 이어지는 저장 snapshot 검사에 포함한다.

실제 CFF 입력에서 parser만 수용하는 예산이 Core 스타일 예약을 거절하고 새 세션이
복구하는지 검사했다. 성공 후 일반 재조판/호환 설정 변경은 보관 중인 import 카운터를
소진하지 않는다. 명시/자동 감지 Core의 페이지 수와 실제 초기화 계상량도 같다. 조판
기대값이나 새 fixture는 추가하지 않았다. Native library **4,077 PASS / 12 skipped**,
library Clippy `-D warnings`, feature-off·WASM 컴파일 검사를 통과했다. 초기 private
visibility/DocInfo Serialize 오류와 보정 후 source hash·로그는 `v2-s31-20261026`에
보존한다. 이 결과는 Core 내부 composition/pagination 전체 예약이나 첫 저장의 제한을
입증하지 않는다. 해당 경계와 실제 제품 대조는 다음 연결 절편에서 계속 처리한다.

### B04 저장 snapshot의 실제 소유 복사 — S32, 부분 완료

공통 Storage를 `src/storage.rs`로 옮기고 이전 내부 경로는 re-export로 유지한다.
renderer 전용 정책으로 model copy를 묶지 않으며 외부 API 경로는 바꾸지 않는다.
일반 snapshot은 같은 producer를 Unbounded로 호출하고 요청용 진입점은 같은 session의
Allocation을 전달한다. 실제 CLI/WASM 첫 저장에 이 진입점을 사용하는 연결은 아직 남는다.

snapshot 복사 전 DocInfo·DocProperties·header 원시 바이트·section/본문·미리보기·
BinData의 실제 소유 Vec/키·추가 stream/aux를 검사한다. lazy BinData resolver는
Arc만 유지하고 복사 검사 때문에 decode하지 않는다. DocInfo provenance의 각 digest
배열도 별도 검사하며 Document/Section/DocInfo 필드 패턴은 exhaustive로 유지한다.
serde가 생략하는 문단의 공백 metric·필드 표시값·원본 vpos·번호 문자열은 read-only
공통 tree walk로 중첩 표/개체/각주/머리말/바탕쪽까지 먼저 검사한다. 나머지 skipped
shape/수식/table seal은 inline 값이다. Form의 custom serializer가 visitor 전에 만드는
정렬 scratch와 HashMap clone의 capacity도 먼저 계상한다. allocator 전체/RSS의
계약으로 확대하지 않으며 일반 opaque resolver 실행까지 제한했다는 주장도 아니다.

중첩 셀의 skipped 배열/문자열과 DocInfo provenance·DocProperties 원시 바이트를
독립적으로 늘린 자원 반례에서 복사를 거절하고 새 요청에서 해당 payload를 보존하는지
검사했다. 실제 CFF의 같은 session에서 snapshot을 만들고 기존 저장 bytes와 대조했으며,
예산 거절 뒤 live IR의 일반 저장 bytes도 유지됐다. 조판 baseline 추가는 없다.
최종 library **4,079 PASS / 12 skipped**, Clippy `-D warnings`, feature-off·WASM
컴파일 검사를 통과했다. 초기 SpaceMetric Serialize/내부 type visibility 오류는 보존했고
모델의 공개 Serialize 계약을 추가해 우회하지 않았다.

S31·S32를 포함한 새 Native에서도 **34개 저장 bytes·98쪽 재열기·상세 진단 3건**과
공개 source 7·9쪽 HWP/HWPX 추가 4개 왕복이 S30과 같다. CFF **100%**·Type3 영향
11쪽 **99.97900%**를 산출하고 현재 PNG/확대 crop을 직접 대조했다. Type3 Native
page PNG도 S30과 byte 단위로 같다. 초기화까지 포함한 decoded 계상은 source 7쪽
**261,032,456 bytes**, 9쪽 **171,535,845 bytes**이며 기존 기본 한도 안에서 수용한다.

그 제품 검증 뒤 DocProperties 예약과 해당 자원 assertion을 추가했다. 이후 library
4,079 PASS·Clippy·두 compile 경계를 다시 실행했다. 제품/source hash와 그 이후 delta를
`v2-s32-20261026/after-native-product-source-delta.json`으로 분리한다. 보정한 경계는
현재 CLI가 아직 호출하지 않는 요청용 snapshot 경로이며 일반 저장은 Unbounded 복사를
사용한다. 새 예약의 실행 증거는 현재 source의 unit이고, 이전 제품 실행을 새 source의
전체 결과로 표시하지 않는다. first-save serializer 연결 뒤 제품·시각 검증을 다시 한다.

이 절편은 **복사 전 예약과 live 상태 보존**의 부분 결과다. snapshot 이후 format adapter·
serializer 버퍼와 실제 CLI/WASM 호출 연결, Core composition/pagination과 최종
fresh 최적화 WASM/Studio·성능 검증은 계속 남는다. 모든 원시 로그·source/binary hash·
TSV·PNG는 `v2-s32-20261026`에 보존한다.

### B04 실제 ZIP 출력과 공통 레코드 결합 — S33, 부분 완료

HWP 레코드 결합은 최종 길이를 먼저 예약한 뒤 공통 header/data append를 사용한다.
레코드마다 임시 출력 Vec를 만들고 다시 복사하던 단계를 제거했다. 기존 API와 요청용
API가 같은 producer를 쓰며 일반/확장 header·tag/level·byte 순서를 유지한다.
요청용 helper는 아직 HWP serializer 전체에 연결된 상태가 아니다.

HWPX ZIP writer는 기존 Cursor 기본형을 유지하면서 Write+Seek backing을 받는다.
요청용 backing은 실제 쓰기·빈 쓰기·seek·backpatch 때 동일 요청의 실패 상태와 output
상한을 확인하고 Vec capacity를 늘리기 전에 예약한다. 실패 뒤 writer는 닫히므로 ZIP의
정리 경로도 backing을 더 키울 수 없다. 전역/TLS 상태나 panic 제어 흐름은 사용하지 않는다.
공통 serializer body가 일반/요청용 ZIP을 모두 생산하며 결정론 옵션은 유지한다.

session 내부의 HWPX 저장 진입점을 연결했다. parser/Core가 남긴 예산을 사용하고 성공
metadata를 갱신하며, 저장 실패 뒤 같은 session을 재사용하지 않는다. 실제 CLI/WASM
진입점은 HWP CFB 경계까지 연결한 후 전환한다. XML/SerializeContext 생성과 기타 내부
scratch까지 이 ZIP backing으로 제한했다는 주장은 아니다.

4095-byte 경계의 독립 header bytes·기존 parser 재파싱, 작은 예산의 레코드 결합 거절과
새 요청 복구를 검사했다. 실제 CFF의 HWPX 저장은 일반 저장 bytes와 같고 재열기된다.
크기 1-byte/잔여 예산 0에서 저장이 실패하고 live IR의 일반 저장 bytes는 유지되며 새
session이 복구한다. buffer의 backpatch·gap·실패 후 쓰기/seek 거절도 실행했다.
Native library **4,083 PASS / 12 skipped**, library Clippy `-D warnings`, feature-off·
WASM 컴파일 검사를 통과했다. 새 조판 baseline은 추가하지 않았다. 원시 명령·로그·source
hash는 `v2-s33-20261026`에 보존한다. 현재 source의 Native CLI/fresh WASM 제품·시각·
성능과 HWP CFB/나머지 scratch·Core 경계는 다음 연결 절편에서 계속 검증한다.

### B04 실제 HWP 컨테이너·자산·CLI 첫 저장 — S34, 부분 완료

기존 HWP/HWPX serializer body에 공통 Storage를 전달했다. HWP DocInfo·본문의 최종
레코드 결합과 압축 backing, CFB 경로·이름·정렬·mini stream·mini FAT·FAT·최종 파일
버퍼를 늘리기 전에 같은 요청을 확인한다. CFB 최대 출력 길이는 최종 파일 버퍼와 FAT를
할당하기 전에 검사한다. UTF-16 디렉터리 이름의 임시 Vec는 stack 배열로 바꿨으며
기존 31-unit 절단과 디렉터리 순서는 유지한다. 일반 API는 같은 producer의 Unbounded
Storage를 사용한다. payload producer·format adapter·XML/context 내부 scratch 전체를
이 변경으로 제한한 것은 아니다.

BinData 복사는 resolver를 호출하기 전에 예약한다. 불변 공유 resolver는 할당 없는
`stable_size_bound`로 실제 길이의 상한을 제공한다. 기존 외부 resolver의 기본 구현은
힌트가 없으므로 명시적인 유한 최대 길이를 먼저 예약한다. 일반 API의 기존 resolver
호출은 바꾸지 않는다. 작은 예산에서는 materialization 호출 자체가 없으며 새 요청에서
복구하는 반례를 추가했다.

`PdfImportSession`은 실제 CLI의 parser → Core → HWP/HWPX 첫 저장 → 진단 JSON까지
같은 요청을 유지한다. 저장 실패는 session을 닫고 live IR을 바꾸지 않으며, 부모는 성공
frame 전까지 기존 출력 파일을 교체하지 않는다. 초기 요청은 반환한 편집 문서에 남기지
않는다. 진단 JSON은 deep-copy Value 없이 먼저 출력 길이를 세고 정확한 backing 한 번을
예약한다. 첫 실제 대조에서 f32의 문자열 표현이 달라진 것을 발견해, 이전 Value 경유의
f64 widening을 같은 formatter로 두 pass에 적용했다. 형식·좌표 기대값은 완화하지 않았다.

실제 공개 source 7쪽에서 새 첫 저장 예약을 연결하자 기존 선택 쪽 기반 decoded 축소가
요청을 거절했다. 가져오기 **261,032,456 bytes**, 스냅숏 복사 preflight의 다음 예약
**24,234,197 bytes**, 당시 유효 한도 **268,435,456 bytes**였다. 별도 allocator probe의
실제 문서 clone 요청량은 **11,472,214 bytes**로, 보수적인 logical copy work와 실제
allocator 요청량을 구분한다. HWPX도 저장 후 **263,552,858 bytes** 상태에서 진단 출력
**6,513,166 bytes**를 예약할 때 거절됐다. 실패 전후 로그·독립 probe source를 보존했다.

이 카운터는 이제 parser 이외의 Core·첫 저장·진단도 포함하므로, 선택 쪽수로 공유 자원과
출력 복사량을 예측하던 decoded 축소만 제거했다. 문서 전체 standard **2 GiB**와
constrained **256 MiB**, 더 낮은 사용자 cap, operator·단일 stream·출력·OS 상한은 유지한다.
카운터를 단계별로 초기화하거나 분리하지 않았다. 한 쪽의 300 MiB 논리 작업 수용과 낮은
host cap의 거절을 검사하며 [자원 정책](../manual/pdf_resource_policies.md)도 현행화했다.

현재 source의 기능용 Native에서 **34개 저장 bytes·98쪽 재열기와 상세 진단 3건**이
S32와 같다. 바뀐 decoded 정책 값 하나와 자원 사용량만 별도 계약으로 검증했고, 나머지
진단 숫자·IR·렌더 트리는 정확히 비교했다. 공개 source 7·9쪽의 HWP/HWPX 추가 **4개
왕복**도 저장 bytes·진단 의미·렌더 트리가 같다. 7쪽 HWP의 저장 후 계상은
**292,745,524 bytes**, HWPX는 **263,552,858 bytes**다. 이 snapshot은 진단 transport
인코딩 전 값이며, 진단 backing도 같은 요청에 추가로 예약된다.

최신 Native 영향 2쪽은 CFF **100%**, Type3 11쪽 **99.97900%**다. 새 compare·overlay·
review PNG를 직접 열어 문자·행 간격·표 외곽·그림/설명 소속과 누락 여부를 대조했다.
실제 CLI의 HWP/HWPX 출력/진단/잔여 decoded 실패 **6건**과 새 정상 요청 복구가 통과했다.
문제 7쪽에서도 명시적 **256 MiB cap**은 여전히 실패하고 stdout·기존 출력 sentinel을
보존하며, 다음 기본 요청은 이전과 동일한 파일을 생성한다. 추가 **2건** 모두 통과했다.
현재 source의 Native library **4,090 PASS / 12 skipped**와 Native CLI Clippy도 통과했다.
feature-off/WASM compile 검사도 통과했다. 이전 source의 4,088 PASS와 구분한다.
제품 검증 후 BinDataResolver의 두 메서드 사이 문서 주석 위치만 바로잡았다. 실행 코드
변경은 없으며 해당 source hash 차이를 별도로 기록했다.

기능용 빌드는 `release-test`에서 root opt-level 1·codegen-units 64를 사용했으므로 최종
성능 근거가 아니다. 현재 source의 최적화 fresh WASM·Studio·전체 시각/성능 검증과
Core composition/pagination·나머지 serializer scratch의 예약은 계속 남는다. S19의 WASM
결과를 S34의 실행 근거로 사용하지 않는다. 전체 원시 로그·source/binary hash·TSV·PNG는
ignored `output/pr-review/issue8174/v2-s34-20261026/`에 보존한다.

### B04 DocInfo 개별 payload·원시 인덱스 — S35, 부분 완료

DocInfo의 11개 생성 레코드가 같은 BinaryProducer/ByteWriter를 사용한다. 요청 경로는
소유 출력 없이 길이를 계산하고 남은 예산을 확인한 뒤, 정확한 backing 한 번을 예약해
고정 slice에 기록한다. 계산/기록 길이가 다르거나 쓰기가 실패하면 같은 요청을 닫는다.
일반 저장은 같은 field writer의 단일 Vec pass를 사용한다. 원본 레코드 재사용은 Cow로
빌려서 최종 스트림에 기록하며 중간 raw clone을 제거했다. Charset·번호·tab·fill·font·
style의 wire 필드 순서와 값은 유지한다. UTF-16 문자열의 전체 임시 Vec를 제거했고
65535-unit 길이 제한·surrogate 절단을 보존했다. Numbering의 기존 wire 계약도 유지했다.

원시 DocInfo의 레코드 위치 수집을 순차 scanner로 바꿨다. 구역/캐럿 갱신과 레코드 삽입은
필요한 위치만 읽고, 7개 언어의 글꼴 삽입은 고정 슬롯을 사용한다. 삭제는 동일 버퍼에서
압축하며 기존의 해석 불가/손상 꼬리 bytes를 보존한다. 현재 입력에서 전체 위치 Vec나
글꼴 위치 ref Vec를 새로 만들지 않는다.

제품 경로의 Native 컴파일·Clippy는 통과했다. 테스트 정리 전 library **4,094 PASS /
12 skipped**, 이전 `ea2576fb25`의 원시 보정 함수를 별도로 추출한 **96개 입력 × 5개 작업 =
480건**의 bytes/반환값 동등을 확인했다. 일반/확장 header·없는/빈 언어 구간·큰 선언 count·
손상 꼬리를 포함한다. 처음 추가한 테스트의 잘못된 Record API 이름은 실행 전에 보정했고
실패 로그를 보존했다.

새 source unit 증가를 금지하는 [테스트 정책](../manual/pr_review/local_validation.md#integration-test-source-추가와-자동-sharding)을
확인해 신규 검사는 `tests/cases/issue_8174_docinfo_request_storage.rs`로 정리했다. UTF-16
wire·확장 레코드/손상 꼬리 보존과 공개 PdfImportSession의 실제 첫 저장 quota 거절·live IR
보존·새 요청 복구 **3건**을 실행해 통과했다. 예산 경계는 공개 사용량으로 탐색하며 실제
`frontend binary payload bytes` 거절이 없으면 실패한다. private 구현을 위한 지원 API나
정책 기준선은 추가하지 않았다. 정리 뒤 S34 대비 unit-tier 검사도 통과했다. 이전 white-box
probe와 실행 로그는 진단 증거로 보존하며 최종 활성 CI 검사는 이 공개 API 회귀다.

테스트 위치를 정리하기 전에 만든 기능용 Native의 생산 코드는 이후 바뀌지 않았다.
제거한 cfg(test) 세 파일과 새 integration 원본의 hash 차이를
`test-relocation-source-delta.json`으로 기록한다. 이 제품에서 **34개 저장 bytes·98쪽 재열기·
상세 진단 3건**, 공개 source 7·9쪽의 양쪽 형식 추가 **4개 왕복**이 S34와 같다. 영향 Native
두 쪽은 **100% / 99.97900%**이며 새 review PNG를 직접 판독했다. 실제 CLI 한도 거절·
기존 출력 보존·새 요청 복구 **6건**도 통과했다. 테스트 정리 뒤 Native CLI Clippy·
feature-off/WASM 컴파일도 통과했다. 최신 최적화 WASM 제품의 실행 결과는 아니다.

브랜치 전체를 cached `upstream/devel`과 비교한 unit-tier 검사는 이전 절편의 신규 source
테스트 모듈/증가 때문에 실패한다. 이번 절편의 검사 통과를 PR 전체 통과로 해석하지 않는다.
T08에서 공개 API 회귀 이동과 실제 실행을 마치고 재검사한다. Core 조판과 나머지
BodyText/control payload·format adapter·XML/context 및 제품 격리/최종 검증은 남는다.
전체 원시 자료는 ignored `output/pr-review/issue8174/v2-s35-20261026/`에 보존한다.


## S41 — 초기 본문 composition 연결

[실행·성능 기록](task_m100_8174_v2_stage41.md). 같은 요청 Storage를 Core → section →
문단 → 저장 줄/글자모양·언어 분할 → 컨트롤 위치와 표시 문자열에 연결했다. 필드 슬롯
HashMap, 합성 마커 문단의 런타임 필드 복사도 증가 전에 예약한다. 측정·배치 규칙을 바꾸지
않고 7,258개 구성 결과와 37개 저장 bytes, 34개 저장/98쪽 재열기·선택 쪽 4개 왕복을
대조했다. 공개 API 검사 11 PASS와 Native 100%/99.97900% 직접 판독, WASM target Clippy를
통과했다. 최적화 fresh WASM 실행은 최종 조합의 T08로 남기며 이전 제품을 재명명하지 않는다.

수식 없는 텍스트의 후속 그림 예약 높이 보정을 생략하지 않았다. 수식 전용 소유 계산,
supplemental/exact-font 경로와 pagination의 중첩 composition, BodyText/control payload·
format adapter/XML/context scratch는 아직 전체 예약을 입증하지 않았다. logical work 계상을
프로세스/브라우저 heap 상한으로 해석하지 않으며 allocation bounds capability는 false다.


## S42 — 첫 HWP 저장의 루트 문단 payload

[실행 기록](task_m100_8174_v2_stage42.md). 같은 Storage를 루트 문단 UTF-16/text backing,
필드 종료·안내문/서식 시프트·range tag와 Header/CharShape/LineSeg/RangeTag의 실제 byte
producer에 전달했다. counting과 fixed slice가 같은 wire encoder를 사용하며 일반 저장도
Unbounded로 그 encoder를 소비한다. 메모의 변경 없는 선행 사본은 빌려 읽는다.

기존 본문 unit **34 PASS**, 공개 요청/저장/wire **15 PASS**. HWP/HWPX 실물 9개에서 raw
재사용을 끄고 생성한 **34,600개 레코드의 9개 stream이 bytes 동일**하다. Native 34개
저장/98쪽 재열기·상세 진단 3개·선택 쪽 4개 왕복과 37개 저장 bytes 동등, CLI 실패·출력
보존·새 요청 복구 6개, Native 새 시각 100%/99.97900% 직접 판독도 유지했다. Native/WASM
Clippy와 feature-off·절편 source unit 정책을 통과했다. 첫 메모 fixture의 차용 타입 보정은
생산 prefix와 기대값을 바꾸지 않았고 실패/보정 로그를 보존했다.

재귀 control writer·메모/마스터 페이지 사본, format adapter와 HWPX XML/context, Core
pagination/중첩 composition은 남는다. 전체 allocation proof나 fresh WASM 실행으로 표시하지
않으며 capability false와 T08 최종 통합 검증은 유지한다.


## S43 — 표·셀·캡션의 재귀 저장

[실행 기록](task_m100_8174_v2_stage43.md). 표 공통 속성·TABLE·셀/캡션 LIST_HEADER의 borrowed
byte producer와 CTRL_HEADER 결합·캡션 attr 사본을 예약한다. 같은 Storage가 내부 문단과
중첩 표까지 재귀한다. 셀 필드 이름은 파서의 검증된 UTF-16 view를 공유해 비교용 Vec/String을
제거하고 기존 lossy 값·미수정 raw를 보존한다. 원본 일치 digest는 기존 JSON→blake3 스트림을
확인했으며 별도 출력 Vec가 없어 수정하지 않았다.

기존 serializer **589 PASS**, 공개 계약 **16 PASS**, 실물 9개의 **34,600개 레코드 bytes
동일**을 확인했다. Native 전체/선택 저장·재열기와 37개 파일 bytes 동등, CLI 실패/복구 6개,
Native 새 시각 100%/99.97900% 직접 판독을 유지했다. Native/WASM Clippy·feature-off·절편
source unit 정책도 통과했다. 신규 셀 metadata wire 검사는 변경 전에도 통과하는 호환성
보호이며 수정 전 실패나 조판 px 증거로 표시하지 않는다.

`try_serialize_control`의 다른 variant는 아직 legacy writer다. `bytes(0)` 확인만 하는 분기를
할당 예약으로 세지 않는다. 다른 control·메모/마스터 페이지, format adapter/XML/context와
Core pagination/중첩 composition, 최종 fresh WASM·Studio 검증은 계속 남는다. 기존 deferred
pagination은 편집 descriptor와 단일 구역 cell-flow 대상이어서 초기 전체 재열기의 중단을
이미 제공한 것으로 사용할 수 없다. capability는 false를 유지한다.


## S44 — 구역·단·쪽 설정과 바탕쪽

[실행 기록](task_m100_8174_v2_stage44.md). SectionDef/ColumnDef와 PAGE_DEF·각주/미주 모양·
쪽 테두리를 같은 borrowed byte producer로 연결했다. 바탕쪽의 raw header 사본·선택 쪽 슬롯·
header/footer LIST_HEADER와 내부 문단, 루트 extension tail도 같은 요청으로 예약한다. 기존
직접 CTRL_DATA 중복 방지·중첩 소유·기록 순서·비트와 수치 계약은 유지했다.

Native 실제 제품·serializer **589 PASS**·공개 계약 **16 PASS**, 9개 BodyText/**34,600개
레코드 bytes 동일**을 확인했다. Native 전체/선택 저장·재열기, 37개 파일 bytes 동일과
CLI 실패·복구 6개, 새 Native 시각 100%/99.97900% 및 대표 판독도 유지했다. Native/WASM
Clippy·feature-off·절편 unit 정책을 통과했다. 17개 PDF의 Core/첫 snapshot **34개 census**와
실제 constructor를 대조했으며 BodyText가 이후 추가하는 ColumnDef도 확인했다. 일부 문서의
관측으로 모든 입력이나 다른 control의 전체 할당을 입증하지 않는다.

그림·도형, adapter/XML/context, Core pagination/중첩 composition과 최종 fresh WASM/Studio는
필수 잔여로 유지한다. source unit 전체 이관·T05 복사/재열기·T06 이후 작업도 완료로 바꾸지
않으며 capability false를 유지한다.


## S45 — 그림·primitive 도형과 글상자

[실행 기록](task_m100_8174_v2_stage45.md). 같은 요청을 Picture와 Rectangle/Line/Polygon의
header·component·종류별 payload·CTRL_DATA·record 배열, 캡션·글상자 LIST_HEADER와 내부
문단까지 전달했다. Ellipse/Arc/Curve/Chart도 같은 순수 producer를 사용한다. 투명도 원본
extra 패치와 변환 행렬의 중간 사본을 제거하며 기존 필드 순서·level·캡션/글상자 계약은 유지한다.

S44 실제 librhwp와 **1,264개 wire 조합 bytes 동일**, 기존 serializer **589 PASS**·공개
계약 **16 PASS**, 실물 9개/**34,600개 레코드 bytes 동일**을 확인했다. 실제 34개 저장/
98쪽 재열기·상세 3개·선택 쪽 4개 왕복과 37개 파일 bytes 동등, CLI 한도 실패·복구 6개,
Native 새 시각 100%/99.97900%와 직접 판독도 유지했다. 초기 사각형 자동 CTRL_DATA의
wrapper 인계 문제는 제품 빌드를 중단하고 보정했으며 최종 대조에는 그 분기를 포함했다.

Group/OLE의 legacy 내부 scratch는 전체 예약을 입증하지 않았다. 실제 PDF constructor 대조와
S44 census에서 생산되지 않은 종류이며 일반 API 지원을 제거하거나 `bytes(0)` 확인을 예약으로
세지 않는다. adapter/XML/context와 Core pagination/중첩 composition, 최종 fresh WASM/
Studio·전체 회귀/성능, T05–T09와 source unit 전체 이관은 남는다. capability false를 유지한다.


## S46 — HWP snapshot adapter의 같은 요청

[실행 기록](task_m100_8174_v2_stage46.md). 구역 정의 사본/Box·PBF 보충·문단과 글상자의
wire-only 필드·HashSet capacity·표 row-size와 payload, geometry 순회와 DocInfo cache의
실제 생성에 Storage를 전달했다. 문단·도형·표/셀·캡션은 재귀하고 일반 저장은 Unbounded로
같은 변환을 소비한다. SectionDef의 바탕쪽 문단 runtime 필드도 복사 전에 계상한다.
BinData는 metadata 선행 사본을 제거하고 binary content를 이동하며 첫 중복/invalid tail·
lazy 계약을 유지했다. href/표 layout ParameterSet은 중간 문자열/UTF-16 Vec 없이 발행한다.

기존 converter/serializer **662 PASS**, 공개 계약 **18 PASS**. 실물 11개의 IR 진단·보고·
BodyText **35개 파일 bytes 동일**, 기존 9개/**34,600개 레코드**, 1,264개 도형 wire 조합과
Native 저장 37개 bytes 동등을 확인했다. 34개 저장/98쪽 재열기·선택 쪽 4개·CLI 실패/복구
6개, Native 새 시각 100%/99.97900%와 직접 판독도 유지했다. 같은 낮은 cap에서는 추가
계상 뒤 진단 단계에서 거절되고 sentinel·새 요청 복구를 보존했다. adapter leaf의 직접 거절
증거와 구분한다. Native/WASM Clippy·feature-off·fmt·절편 unit 정책도 통과했다.

HWPX 전용 바탕쪽 slot 물질화와 BTree node의 실제 allocator layout은 이 기록의 전체 할당
증명이 아니다. PDF 분기는 해당 slot 호출/출처 marker를 거치지 않으며 BTree entry는 논리
작업량 계약이다. HWPX XML/context·Core pagination/중첩 composition·table-frame writeback,
T05–T09와 최종 fresh WASM/Studio·전체 회귀/성능 및 source unit 전체 이관은 남는다.
capability false를 유지한다.


## S47 — HWPX ID pool·BinData context

[실행 기록](task_m100_8174_v2_stage47.md). Session 소유의 같은 ImportBudget Arc에서 예약
객체를 만들어 context ID 등록과 런타임 참조 HashSet의 실제 증가 전에 전달한다. 첫 오류는
보존하고 해당 집합 증가를 중단하며, 생성한 section/master-page를 ZIP에 넣기 전에 전파한다.
이것을 XML 문자열 생성 내부의 중단으로 표시하지 않는다. BinData/순번 map·차트/정렬 metadata·
문자열·section/master 항목·manifest/ZIP 집합과 참조 검사 scratch도 같은 Storage를 소비한다.

새 공개 경계는 **S46 FAIL / 후보 PASS**이며 공개 계약 **19 PASS**, 기존 serializer/adapter
662개와 Allocation 9개를 합친 **671 PASS**를 확인했다. 실제 한도 sweep에서 초기 ID 등록,
metadata와 런타임 참조 HashSet·ZIP의 서로 다른 소비 경계에서 거절을 실행했다. 기존 34개
저장/98쪽 재열기·상세 3개·선택 쪽 4개, **37개 파일 bytes 동일**, CLI 실패/복구 6개와
Native 새 시각 100%/99.97900%·직접 판독도 유지했다. Native/WASM Clippy·feature-off·fmt·
절편 unit 정책과 suite 배정 계약을 통과했다.

XML part 내부 attribute/escape·template/run scratch와 table-grid/content-loss, Core
pagination/중첩 composition·table-frame writeback, T05–T09와 최종 fresh WASM/Studio·전쪽
시각/성능·source unit 전체 이관은 남는다. capability false와 host 책임을 유지한다.


## S48 — 공통 XML encoder·HWPX header

[실행 기록](task_m100_8174_v2_stage48.md). 빌린 XML payload를 같은 encoder로 계산·기록하고
header의 scalar/속성·charPr·채우기와 BudgetCursor 증가/원문 splice에 같은 Storage를 전달했다.
XML 이벤트와 문맥별 whitespace escape·기존 공개 API는 유지한다. section/master-page 등의
일반 helper는 아직 Unbounded이므로 공통 helper 추가를 모든 XML 경계의 예약으로 표시하지 않는다.

공개 **23 PASS**, 기존 **671 PASS**, 실제 header 11개·ZIP 10개·기존 양식 거절 1개의 bytes
보존과 PDF 34개 저장/98쪽 재열기·상세 3개·선택 4개를 확인했다. PDF 저장 파일 37개와 새
Native review/overlay 6개도 S47과 동일했다. 새 Native 시각 100%/99.97900%를 직접 판독했다.
Native/WASM Clippy·feature-off·fmt·절편 unit 정책과 suite 배정 계약도 통과했다.

S47의 XML 원문/속성 우회 재현과, 정상 저장 실측값을 바탕으로 한 후보의 tail/escape 거절·
sticky failure·live IR 보존·새 요청 복구를 구분해 기록했다. 전후 테스트 한도/원문 크기가
동일하지 않은 점을 공개했으며 제품 한도는 높이지 않았다. 숫자를 heap/RSS 크기로 해석하지 않는다.

나머지 XML part/template/run·table-grid/content-loss, Core pagination/중첩 composition·
table-frame writeback, T05–T09와 전체 source unit 이관·최종 fresh WASM/Studio는 남는다.


## S49 — section/master-page XML 조립

[실행 기록](task_m100_8174_v2_stage49.md). 같은 Storage를 section 템플릿 literal 치환·
scalar/각주·첫 문단/후속 secPr와 column 정의, master ID/참조·문단 누적·최종 조립에 전달했다.
context 실패는 paragraph parts 반환 직후 전파한다. 이미 생성한 run을 복사할 때의 예약을
run 내부 생성의 예약이나 중단으로 표시하지 않는다. 공개 호출과 XML bytes는 유지한다.

새 공개 조립 경계 두 건은 같은 test source의 **S48 FAIL / 후보 PASS**이며 공개 **25 PASS**,
기존 **671 PASS**를 확인했다. 실제 header 11개·ZIP 10개·기존 양식 거절 1개, PDF 저장 파일
37개의 bytes가 S48과 동일했다. 34개 저장/98쪽 재열기·상세 3개·선택 4개, CLI 실패/복구 6개와
새 Native 시각 100%/99.97900%·직접 판독도 유지했다. Native/WASM Clippy·feature-off·fmt·
절편 unit 정책과 suite 배정 계약을 통과했다.

run/lineseg·중첩 control의 내부 문자열/벡터·table-grid/content-loss와 Core의 나머지 경계,
T05–T09·최종 fresh WASM/Studio와 전체 source unit 이관은 남는다. 논리적 작업량과
프로세스 전체 heap/RSS 상한을 구분하고 capability false를 유지한다.


## S50 — run·중첩 XML과 Core 호출 경계의 통합 보강

[실행 기록](task_m100_8174_v2_stage50.md). 같은 요청을 run/lineseg의 내부 문자열·배열,
중첩 표·그림·도형의 XML event/scalar/행렬 backing과 row-grid sidecar의 projection/JSON에
전달했다. semantic 오류 분기에서도 예산 실패는 전파하며 문단·subList 상태를 복구한다.

Core의 초기 pagination orchestration과 측정 표·미주·머리말/꼬리말 결과 복사,
표 프레임 저장 계획·컷 집합·줄 소속 수집도 같은 요청을 사용한다. 파생 타입의 소유 backing은
fallible structural copy로 복사하고 Arc/Rc는 공유한다. 일반 Unbounded 복사는 기존 Clone을 쓴다.
컷·높이·원점 규칙과 기존 기대값은 바꾸지 않았다.

같은 source의 **S49 3 FAIL / 후보 3 PASS**, 공개 **29 PASS**, Core 집중 **624 PASS / 기존
8 ignored**, 기존 serializer/adapter·Allocation **671 PASS**를 확인했다. 실제 header/ZIP·
기존 거절의 22개와 PDF 저장 37개의 bytes가 S49와 같았다. 34개 저장/98쪽 재열기·선택 쪽
추가 4개·CLI 실패/복구 6개와 새 Native 시각 100%/99.97900%·직접 판독을 유지했다.
Native/WASM Clippy·feature-off·fmt·절편 unit 정책과 suite 배정 계약도 통과했다.

**예약한 결과의 소비와 그 결과를 처음 만드는 내부 producer는 구분한다.** 정규화·높이 측정·
중첩 composition/frame reflow·TypesetEngine/Paginator의 내부 state, cell-unit/index cache와
content-loss 내부 수집은 남는다. 표 프레임 helper의 결과 수집을 그 앞의 cell_units 생성까지
예산화한 것으로 표시하지 않는다. T04는 진행 중이며 T05–T09·최종 fresh WASM/Studio·
전쪽 시각/성능과 source unit 전체 이관도 남는다. capability false와 host 책임을 유지한다.


## S51 — 높이 측정·중첩 composition의 실제 요청 전파

[실행 기록](task_m100_8174_v2_stage51.md). Core의 일반/빈 구역·증분 측정이 같은 Storage로
문단/행/상세 셀 원장과 nested table·캡션의 초기 composition을 소비하게 했다. 기존 scoped
composer와 supplemental metric 조건을 유지하며 다른 shaping 규칙으로 교체하지 않았다.
재귀 callback의 실패를 Result로 전달하고, frame 문단/셀 projection·baseline/cache 복사도
생성 전에 예약한다. 공통 열 너비 소유자·control UTF-16 위치/저장 줄 소속·nested group·
stored block flow의 tree/vector/정렬 scratch도 같은 요청을 쓴다. 배치·컷·기존 기대값은 유지한다.

실제 PDF 두 개에서 같은 cap·source의 **S50 2 FAIL / 후보 2 PASS**, 공개 **31 PASS**,
Core **624 PASS / 기존 8 ignored**, composer/float/model **244 PASS**, 기존 **671 PASS**를
확인했다. 실제 header/ZIP·기존 거절의 22개 파일과 PDF 저장 37개 bytes가 S50과 같았다.
34개 저장/98쪽 재열기·선택 쪽 추가 4개·CLI 실패/복구 6개와 새 Native 시각
100%/99.97900%·직접 판독도 유지했다. Native/WASM Clippy·feature-off·fmt·절편 unit 정책·
suite 배정 계약을 통과했다. 계상 작업량 증가는 시간/RSS 증가율이나 물리 상한 증명이 아니다.

프레임 결과를 복사할 때의 예약과 frame reflow producer는 구분한다. stored-row resolver의
내부 frame/overflow/shaping cache, 일부 object-only/stored row/wrap helper, 정규화와
TypesetEngine/Paginator state·page-field 재분할·cell-unit/index cache·content-loss 내부는
남는다. 기존 fallible fill/frame helper의 실제 소비 경로를 이어 연결해야 한다. T04와
T05–T09·최종 fresh WASM/Studio·전쪽 시각/성능·전체 source unit 이관은 진행 중이다.


## S52 — 저장 줄·셀 frame reflow의 실제 요청 전파와 최신 WASM

[실행 기록](task_m100_8174_v2_stage52.md). 높이 측정·중첩 셀이 같은 Storage로 stored-row
admission·checkpoint·기존 fallible fill·LineSeg/space metric projection과 문맥 run 보존을
소비하게 했다. 유한 요청의 resolver 오류는 미리 예약한 owned checkpoint로 추가 할당 없이
복원한다. NO_LS/stale·저장 축·부분 수식·외부 어울림/물리 frame 소유권과 줄·컷·높이 조건은
유지한다. 일반 공개 함수도 같은 구현의 Unbounded wrapper를 쓴다.

같은 source/cap의 **S51 2 FAIL / 후보 2 PASS**, 공개 **33 PASS**, Core **624 PASS / 기존
8 ignored**, composer/float/model **244 PASS**, 기존 **671 PASS**를 확인했다. 실제 header/ZIP·
기존 거절 22개와 PDF 저장 37개 bytes가 S51과 같다. 34개 저장/98쪽 재열기·상세 3개·선택
4개·CLI 실패/복구 6개와 Native/WASM Clippy·feature-off·fmt·절편 unit 정책·suite 계약도
통과했다. 새 예산 cap은 Native64 실측이며 같은 수치의 WASM 경계 증거로 표시하지 않는다.

현재 source를 locked release/codegen-units=16·wasm-opt로 새 빌드했다. 실제 **249 assertions /
14 Workers**에서 12쪽 전쪽 렌더·양쪽 저장/재열기와 오류/취소/Memory.grow 실패 뒤 복구를
통과했다. Studio는 실패 3개·취소 뒤 기존 문서/draft/recents를 보존하고 실제 사용자 PDF
15쪽 가져오기·표시·양쪽 저장을 통과했다. 두 실행과 전쪽 sweep의 WASM hash가 새 제품과 같다.
S37 대비 WASM 크기 증가는 누적 S38–S52의 **3.71370%**이며 시간/RSS 증가율이 아니다.

Native/fresh WASM 각각 **17개·49쪽**, 최저 **98.34664% / 98.79246%**이며 누락·측정 불가·
쪽수 불일치·90% 미만은 없다. CFF·Type3 대표/최저 쪽·중첩 셀·여러 줄 셀의 새 review와
Type3 standalone overlay를 직접 판독했다. 그림 세선/문자 획의 래스터 차이는 남는다.
현재 source의 최신 WASM·Studio·전쪽 검증이며 최종 PR 게이트나 이후 source의 결과는 아니다.

일부 frame 내부 projection/control·stale/word-tail·leader·overflow/shaping helper, 정규화와
TypesetEngine/Paginator state·page-field 재분할·cell-unit/index cache·content-loss는 남는다.
T04와 T05–T09·최종 전체 회귀/성능·source unit 전체 이관은 진행 중이다. 최신 upstream/devel
전쪽 대조·2%p 후퇴 판정도 T09에 남는다. capability false와 host 책임을 유지한다.


## S53 — 내부 인라인 입력·저장 줄/리더·꼬리·overflow memo

[실행 기록](task_m100_8174_v2_stage53.md). 같은 요청으로 인라인 입력·개체 위치/표 실측·
필드 토큰과 폭 배열, run 메트릭의 style backing을 생성한다. TextStyle·LatinSlotMetrics의
소유 필드와 tab 배열/Box는 예약하고 immutable Arc는 공유한다. 표 선언 폭 우선·0일 때의
기존 열 fallback, raw UTF-16/scalar 슬롯·바깥 여백/캡션 점유 계약은 유지한다.

저장 줄 과밀/underfill·리더의 문자/후보 문자열·낱말 꼬리와 마커 frame trial/projection,
overflow memo의 map 증가도 같은 요청으로 연결했다. 원래 폭·정렬/개행·물리 쪽/단·
저장 축·cache key/clear 조건은 바꾸지 않는다. 오류를 false/None으로 버리지 않는다.

같은 검사 source·cap의 **S52 2 FAIL / 후보 2 PASS**, 공개 **35 PASS**, Core **624 PASS /
기존 8 ignored**, composer/float/model **244 PASS**, 기존 **671 PASS**, TAC/리더/번호·
condense 꼬리의 기존 integration **17 PASS**를 확인했다. 실제 header/ZIP·기존 거절 22개와
PDF 저장 37개 bytes가 S52와 같다. 34개 저장/98쪽 재열기·상세 3개·선택 4개·CLI 실패/복구
6개, Native/WASM Clippy·feature-off·fmt·절편 unit 정책·suite 계약도 통과했다.

현재 source의 최적화 fresh WASM에서 **249 assertions / 14 Workers**와 Studio의 기존 문서
보존·오류/취소/복구·15쪽 가져오기/표시·양쪽 저장을 통과했다. Native/fresh WASM **각각
17개·49쪽**, 최저 **98.34664% / 98.79246%**이며 **각각 49개 실제 PNG 모두 S52와 bytes
동일**하다. 새 review/standalone overlay도 직접 판독했다. WASM은 S52 대비 **0.54047%**
커졌으며 시간/RSS 증가율이나 최종 성능 결론으로 해석하지 않는다.

일부 KoPub/marker helper·kerning/shaping·frame metrics/하위 producer, 정규화와
TypesetEngine/Paginator state·page-field 재분할·cell-unit/index cache·content-loss는 남는다.
T04 전체·할당 상한 증명과 T05–T09/최종 PR 게이트를 완료로 표시하지 않으며 capability
false와 host 책임을 유지한다. 최신 upstream/devel 대조·2%p 후퇴 판정도 T09에 남는다.


## S54 — 공통 정규화와 연결한 측정 owner의 요청 예산

[실행 기록](task_m100_8174_v2_stage54.md). `paginate_pass_with_storage`가 정규화의 section
revision/결과·대상 인덱스·파생 문단/구성 줄과 overlay·path/map·저장 frame/unit backing까지
같은 Storage로 생성한다. 수식 줄/글상자·저장 꼬리의 기존 proof와 원점 산술·cache 재사용·
원본 보존은 유지한다. optional proof의 정상 None과 reservation 오류를 분리해 원 오류를
즉시 반환한다. 공통 reflow helper의 feature guard도 실제 feature-off 사용 경로에 맞췄다.

같은 test source·Native64 cap에서 **S53 2 FAIL / 후보 2 PASS**이며 기본 요청 복구와
저장 bytes를 보존했다. 공개 **37 PASS**, Core/정규화 **629 PASS / 기존 8 ignored**,
composer/float/model **244 PASS**, 텍스트 폭/글상자 **38 PASS**, serializer/Allocation
**662 + 9 PASS**, 기존 frame·정규화 integration **17 + 3 PASS**를 확인했다.
최종 Native/WASM Clippy·feature-off·fmt·절편 unit 정책·suite 계약을 포함한 12개도 통과했다.

최적화 fresh WASM은 **249 assertions / 14 Workers**와 실제 Studio의 기존 문서 보존·
오류/취소/복구·15쪽 가져오기/저장을 통과했다. Native/fresh WASM **각각 17개·49쪽**,
최저 raw **98.34664% / 98.79246%**이고, 각 runtime의 실제 PNG **49개 전부와 저장
37개 파일이 S53과 bytes 동일**하다. 각 runtime의 새 review 5개와 Type3 11쪽 standalone overlay로 기호·간격·표/셀/그림·캡션·앞뒤 내용을 직접 판독했다. 원시 증거는 ignored
`output/pr-review/issue8174/v2-s54-final-20261027/`에 보존했다.

일부 일반 helper·kerning/shaping/frame metric 하위 경로, TypesetEngine/Paginator state·
body page-field 재분할, 표 저장 cell-unit/index cache·content-loss 수집은 남는다.
이번에는 정규화 경계를 닫았으며 T04 전체/할당 상한 증명·T05–T09 완료로 표시하지 않는다.
기존 한도·`parserAllocationBoundsProven=false`·host 책임과 최종 source의 전체 검증을 유지한다.


## S55 — 첫 저장 메타데이터·content-loss의 실제 요청 전파

[실행 기록](task_m100_8174_v2_stage55.md). HWP/HWPX의 content-loss 경로 문자열과
임시/최종 목록, 텍스트 클립·링크 원래 서식·원 글꼴 프로그램·테두리 그림 소유 메타데이터의
기록·owner count/ID set·resource memo·출력 backing을 같은 Storage로 연결했다. 텍스트
경로/링크 서식은 원본을 빌려 직렬화하고 fingerprint는 고정 배열/직접 digest writer로
불필요한 임시 복사를 없앴다. 같은 소유 검증·순서·해시·clip 조건·flag와 형식을 유지한다.
metadata 판정의 정상 None과 요청 오류를 구분하며 실패 시 실제 저장을 중단한다.

같은 검사 source·Native64 정책의 **S54 2 FAIL / 후보 2 PASS**, 실패 유지·live 문서와
보고서 보존·새 요청 복구를 확인했다. 공개 **39 PASS**, 실제 편집/양쪽 저장/재열기를
보호하는 메타데이터 기존 **52개가 S54와 후보 모두 PASS**다. 저장 37개·header/ZIP 22개
파일 bytes와 손실 DTO를 보존했다. Core/정규화 **629 PASS / 기존 8 ignored**, composer/
model **244**·텍스트 측정 **38**·serializer/Allocation **662 + 9**·기존 frame **17 PASS**,
Native/WASM Clippy·feature-off·fmt·절편 unit 정책·suite 계약 등 **12개 실행 PASS**다.
Native **17개·49쪽**, 최저 **98.34664%**이고 실제 PNG 49개가 S54와 bytes 동일하다.
새 대표 5개 review와 Type3 11쪽 standalone overlay를 직접 판독했다. 최적화 fresh
WASM은 **249 assertions / 14 Workers**·Studio의 문서 보존/실패/취소/복구와 사용자 PDF
15쪽 표시/저장을 통과했다. fresh WASM도 **17개·49쪽**, 최저 **98.79246%**이고 실제
PNG 49개가 S54와 bytes 동일하다. 같은 새 대표 review/overlay를 직접 판독했다. 누락·
측정 불가·90% 미만·쪽수 불일치는 없다. WASM은 S54 대비 **0.50040%** 커졌고 JS ABI
바이트는 같다. 이 크기를 시간/RSS 증가율로 해석하지 않는다.

본문 page-field 재분할은 **현행 bounded 경로의 비도달로 감사 완료**했다. 초기 PDF lowerer가
Page/TotalPage와 layout 값을 생성하지 않아 수렴 진입점은 할당 없는 빈 반환이며, 첫 저장은
snapshot/serializer를 호출해 수렴이나 bounded pagination으로 재진입하지 않는다.
17개·49쪽의 실제 Core에서 해당 값이 0인 것도 확인했다. 일반 편집의 별도 수명은 기존 공개
검사로 확인하며, 일반 HWP 쪽번호 수렴 전체를 다시 작성하지 않는다. 커닝/shaping·목록
마커의 초기 가드도 대조했지만 임의 host 편집 뒤 하위 경로 전체 완료로 확대하지 않는다.

일부 일반 helper/커닝·shaping/frame metric과 TypesetEngine/Paginator state·cell-unit/
index cache는 남는다. 이번에 content-loss/보존 메타데이터 경계를 닫았으며 T04 전체와
T05–T09·최종 source 통합 게이트는 완료로 표시하지 않는다. 논리 계상·실제 heap/RSS와
host 책임·기존 한도·`parserAllocationBoundsProven=false`를 구분한다.


## S56 — Paginator 페이지·단 상태와 공통 소유 producer

[실행 기록](task_m100_8174_v2_stage56.md). Core의 실제 fallback/빈 구역 호출이 같은
Storage로 단 영역·페이지/단/항목·줄 경계/강제 분할·행 높이 fallback·layout/zone 복사와
감춤/방어 집합을 만든다. fit·행 전진/이월·원점/높이·캡션/각주 공간과 방어 상한은 유지한다.
페이지 번호의 본문 소유 색인·컨트롤 위치/이벤트·소비 집합과 활성 머리말/꼬리말, 중첩 표의
머리말·각주/미주 경로와 각주 참조의 경로 복사도 같은 요청으로 연결했다. 기하 우선과
원시 UTF-16 슬롯 fallback은 공통 helper를 유지하고 오류를 정상 None으로 바꾸지 않는다.
PDF에서 쓰지 않는 KoPub 이름 scratch도 기존 HWPX 조건 안에서만 만든다.

같은 검사 source·Native64 정책에서 **S55 2 FAIL / 후보 2 PASS**다. 자식 프로세스에서
실제 `RHWP_USE_PAGINATOR=1`을 실행해 부모/동시 테스트 환경을 보존한다. cap은 이전
fallback 측정 + 512 bytes로 추가 Core 단 영역 32 bytes보다 넉넉하며 Paginator에 도달한다.
공개 **41 PASS**, Core/정규화/두 엔진과 기존 page-layout/page-number **644 PASS / 기존
8 ignored**, composer/model **244**·텍스트 측정 **38**·serializer/Allocation **662 + 9**·
frame **17 PASS**, Native/WASM Clippy·feature-off·fmt 등 **12개 실행 PASS**다.

메타데이터 기존 **52개가 S55와 후보 모두 PASS**이며 저장 37개·header/ZIP 22개 bytes를
보존했다. Native 기본 **17개·49쪽**, 최저 **98.34664%**, 실제 PNG 49개가 S55와 bytes
동일하다. 별도 실제 fallback의 CFF/본문/다단/중첩 셀/여러 줄 셀 **5개·5쪽 모두 100%**,
PNG 5개도 bytes 동일하다. 현재 기본/fallback의 새 review와 standalone overlay를 직접
판독했다. 최적화 fresh WASM **249 assertions / 14 Workers**·Studio의 문서 보존/실패/취소/복구와 15쪽 표시/저장도 통과했다. fresh WASM도 **17개·49쪽**, 최저 **98.79246%**, PNG **49개 모두 S55와 bytes 동일**하며 새 대표 review/overlay를 직접 판독했다. WASM 크기는 **+0.09222%**, JS glue bytes와 선언 서명은 유지한다. source·검사·제품 hash/전쪽 증거를 대조했고 통과했다.

분할/확정의 실제 호출과 기존 실행 경계는 S56 기록에 연결했다. legacy fallback이 cell-unit
컷을 생성하지 않아 원래 ignored인 두 intra-row 검사를 PASS나 새 컷 구현의 증거로 세지
않는다. 기존 상태·기대값은 유지한다. 새 source unit/fixture나 기준값 완화는 없다.
TypesetEngine 내부 state·cell-unit/index cache와 일부 커닝/shaping/frame metric/helper는
남는다. T04 전체·할당 상한 증명·T05–T09 및 최종 PR 게이트로 완료를 확대하지 않으며
기존 한도·host 책임과 `parserAllocationBoundsProven=false`를 유지한다.


## S57 — TypesetEngine 구역 진입·쪽 확정과 공통 머리말 수집

[실행 기록](task_m100_8174_v2_stage57.md). Core의 실제 기본 pagination에 같은 Storage를
전달해 구역 단 영역·용지 float 증거, 머리말/꼬리말/중첩 소유 경로, 쪽 확정의 소유 색인·
컨트롤 위치/이벤트·번호 소비 집합과 활성 머리말 경로 복사를 선행 예약한다. 일반 API는
같은 구현의 Unbounded wrapper다. 기존 Paginator의 머리말 수집 producer를 공통 위치로
옮겨 두 엔진이 같은 순서/선택/경로를 소비한다. 원래 줄/표의 컷·요구/누적 높이·이월·배치
조건과 번호 발화 규칙은 유지한다. Core는 기존 normalization Arc를 생성자에 바로 전달한다.

같은 검사 source·Native64 한도에서 **S56 2 FAIL / 후보 2 PASS**다. 실패 유지·재사용
거절·새 요청의 같은 쪽수/저장 bytes 복구를 확인했다. 공개 **43 PASS**, Core와 두 엔진/
정규화 **644 PASS / 기존 8 ignored**, composer/model **244**, 텍스트 측정 **38**,
serializer/Allocation **662 + 9**, frame **17**, 메타데이터 기존 **52 PASS**다. 저장 37개·
header/ZIP 22개 bytes를 유지했다. Native **17개·49쪽**, 최저 **98.34664%**, PNG **49개
전부 S56과 bytes 동일**하다. 실제 fallback도 **5개·5쪽 모두 100%**·PNG **5개 동일**이다.
현재 기본/fallback의 대표 review/overlay를 직접 판독했다. 최적화 fresh WASM **249 assertions /
14 Workers**·Studio 문서 보존/실패/취소/복구·15쪽 표시/저장도 PASS다. fresh WASM **17개·
49쪽**, 최저 **98.79246%**, PNG **49개 전부 S56과 bytes 동일**이며 새 대표 review/
overlay를 직접 판독했다. WASM 크기는 **0.08779% 감소**했고 JS bytes·선언 서명은 유지한다.
현재 source·검사·제품 hash와 전쪽 TSV/저장/직접 판독 증거의 대조도 통과했다.

실제 library의 GDB symbol breakpoint로 초기 PDF/첫 HWP·HWPX 저장을 추적했다.
S56과 후보 양쪽에서 셀 유닛 producer/행 색인 scope 심볼의 breakpoint hit는 현재 **17개·49쪽 모두 0회**이고,
정상 HWP 대조군에서는 **4,210 / 4,378회**를 검출했다. 인라인 호출 포착을 보증하지 않는다. 일반 엔진의 극단 높이
fallback은 분할 준비로 이어질 수 있어, 이 관측을 모든 입력의 비도달/안전성 증명으로
바꾸지 않는다. 셀 유닛/색인 캐시와 주 엔진의 중간 배치 상태·일부 일반 helper는 남는다.
새 source unit/fixture/golden·기대값 변경이나 기존 ignored의 PASS 승격은 하지 않았다.
T04 전체·T05–T09와 최종 source 통합 게이트는 미완료다. 기존 한도·host 감독 책임과
`parserAllocationBoundsProven=false`를 유지한다.


## S58 — 구역 조정자의 페이지 생성·단 flush·직접 이월

[실행 기록](task_m100_8174_v2_stage58.md). 구역의 문단 진입과 종료·직접 이월이 같은
Storage로 page/column/items backing·page/zone layout 복사·사다리 band를 예약한다.
강제 쪽 전환과 하위 지연 도형/저장 프레임·Square 그림/앵커·overlay·각주 배출도 fallible
공통 경로를 호출한다. 기존 API는 같은 구현의 Unbounded wrapper다. 원래 컷·요구/누적
높이·소유·예약 max·원점과 전환/적용 순서를 바꾸지 않았다. 맵의 새 entry만 선행 예약하며
기존 key의 교체를 새 entry 증가로 세지 않는다.

같은 검사 source·Native64 정책에서 **S57 2 FAIL / 후보 2 PASS**다. 작은 한도 거절·
실패 유지·재사용 거절·새 요청의 같은 쪽수/저장 bytes 복구를 통과했다. 현재 실제 페이지
생성/종료의 요청 한도 증거와 일반 wrapper에서 수행한 기존 분할·이월 검사를 구분한다.
모든 분할 분기의 한도 실패를 새로 검증한 것으로 표시하지 않는다. 앞 절편의 cap도 그대로
유지했으며 새 producer가 연결돼 더 앞에서 거절될 수 있음을 주석에 정리했다.

공개 **45 PASS**, Core/두 엔진·정규화 **644 PASS / 기존 8 ignored**, composer/model
**244**, 텍스트 측정 **38**, serializer/Allocation **662 + 9**, frame **17**, 메타데이터 기존
**52 PASS**다. 저장 **37개**·header/ZIP **22개** 파일 bytes를 유지했다. Native **17개·
49쪽**, 최저 **98.34664%**, 실제 PNG **49개 모두 S57과 bytes 동일**하며 새 대표 review/
overlay를 직접 판독했다. Native/WASM Clippy·feature-off·fmt 등 **12개 실행 PASS**이고
최적화 fresh WASM **249 assertions / 14 Workers**·Studio 보존/실패/취소/복구·15쪽
표시/저장도 PASS다. fresh WASM **17개·49쪽**, 최저 **98.79246%**, 실제 PNG **49개
모두 S57과 bytes 동일**하며 현재 대표 review/overlay를 직접 판독했다. WASM 크기는
**+0.39203%**이고 JS bytes·선언 서명은 유지한다. 현재 source·검사·제품 hash와 전쪽/
저장/직접 판독 증거의 대조도 통과했다. source unit·fixture/golden·기대값 완화는 없다.

구역 조정자가 직접 호출하는 상태와 하위 producer를 연결한 범위다. 문단/표/미주 내부의
일반 상태 API 호출과 항목·메트릭 producer, 셀 유닛/색인 캐시·일부 일반 커닝/shaping/
frame helper는 남는다. S57의 심볼 hit를 현재 head의 비도달/안전성 증명으로 확대하지 않는다.
T04 전체·T05–T09와 최종 source 통합 게이트는 미완료다. 기존 한도·host 책임과
`parserAllocationBoundsProven=false`를 유지한다.


## S59 — 문단 서식·줄 메트릭의 요청 전달

[실행 기록](task_m100_8174_v2_stage59.md). 구역의 표 없는 문단이 같은 요청을 재구성·
TAC 줄 소유 투영·수식 배정/packing·그림 stack·줄 높이/간격과 fallback 배열에 전달한다.
파생 구성/host 줄 사본·마지막 줄 폭의 스타일 복사도 fallible 공통 경로를 쓴다. 일반 API는
같은 구현의 Unbounded wrapper다. 원래 재구성 선택·줄 소유·수식 배정·메트릭/fit 계산과
배치 조건·순서는 유지하며, 다음 저장 vpos의 임시 관측은 오류 전파 전에도 해제한다.

같은 source·Native64 입력 정책에서 **S58 1 FAIL / 후보 1 PASS**다. 작은 한도 거절·
실패 유지·재사용 거절·새 요청의 같은 쪽수/HWPX bytes 복구를 통과했다. 초기 진단의
512-byte 여유가 CFF의 새 계상 48 bytes를 검출하지 못했고 표 사례는 이 경로를 쓰지
않았음을 보존했다. S58 측정값 + 16 bytes의 새 CFF 정책으로 실제 차이를 검증했다.
오늘의 counter 값이나 거절 지점을 기대값으로 고정하지 않으며 기존 회귀 기대값·한도·
baseline은 바꾸지 않았다.

공개 **46 PASS**, 기존 메타데이터 **52 PASS**, 저장 **37개**·header/ZIP **22개**
bytes를 보존했다. Core/두 엔진·정규화 **644 PASS / 기존 8 ignored**, composer/model
**244**, 텍스트 측정 **38**, serializer/Allocation **662 + 9**, frame **17**,
Native/WASM Clippy·feature-off·fmt 등 **12개 실행 PASS**다. Native와 fresh WASM은
각각 **17개·49쪽**, 최저 raw **98.34664% / 98.79246%**, 각각 PNG **49개 모두 S58과
bytes 동일**하며 현재 대표 review/overlay를 직접 판독했다. 별도 Native fallback도
**5개·5쪽 100%**, 실제 PNG 5개는 S57 실행과 bytes 동일하고 새 review 5개를 판독했다.
최적화 fresh WASM·실제 Worker **249 assertions / 14 Workers**·Studio 보존/실패/
취소/복구·15쪽 표시/저장도 PASS다. WASM 크기는 **0.38755% 감소**했으며 JS bytes·
선언 서명은 유지한다. source·검사·제품 hash와 전쪽/저장/직접 판독의 대조도 통과했다.

다음은 표 문단의 인라인 plan/commit과 일반 State의 항목·원장 연결이다. 표/미주 서식과
셀 유닛/행 색인 캐시·일부 일반 frame/커닝/shaping은 남는다. 이번 일반 문단 서식을
전체 조판 할당의 완료로 확대하지 않는다. 새 source unit/fixture/golden·조판 기대값
변경은 없다. T04 전체·T05–T09·최종 통합/최신 devel 보존 게이트와 host 책임,
`parserAllocationBoundsProven=false`를 유지한다.

## S60 — 공통 인라인 계획과 수용 상태의 요청 전달

[실행 기록](task_m100_8174_v2_stage60.md). 구역의 표/일반 문단 → 공통 plan builder →
글자/개체·PUA·어절/구분 공백·배제 영역 → 기존 frame/행·상자 → 수용 계획의 State 확정에
같은 요청을 전달한다. 임시 다음 단의 전체 fit과 실제 전환 뒤 재계산, 분할 소유자·기준선·
높이 계산·배치 순서를 유지했다. 예산 실패는 정상 미지원 후보로 바꾸지 않는다. State의
모든 항목·ledger나 표/미주 내부 상태까지 완료한 것은 아니다.

같은 공개 검사·Native64 정책에서 실제 **S59 1 FAIL / 후보 1 PASS**다. 실패 유지·재사용
거절·새 요청의 같은 쪽수/HWPX bytes 복구를 확인했다. 기존 Allocation의 token storage
계약 하나에 두 bounded plan의 작은 한도 거절/복구를 추가했다. source unit 모듈·개수
증가는 없고 Allocation은 기존 **9 PASS**다. 초기 중첩 셀 cap 진단이 거절을 검출하지
못한 기록을 남겼으며 비도달/보편적인 거절 증명으로 해석하지 않는다. 최종 cold 관측에서
중첩 셀·여러 줄 셀·문단 원점은 각각 **+48 논리 bytes**, CFF는 동일하다.

공개 **47 PASS**, 기존 메타데이터 **52 PASS**, 저장 **37개**·header/ZIP **22개** bytes를
보존했다. Core/두 엔진·정규화/inline/frame **657 PASS / 기존 8 ignored**, composer/model
**244**, 텍스트 측정 **38**, serializer/Allocation **662 + 9**, frame/inline Print **22**,
Native/WASM Clippy·feature-off·fmt 등 **12개 실행 PASS**다. 최적화 fresh WASM과 실제
Worker **249 assertions / 14 Workers**·Studio 보존/실패/취소/복구·15쪽 표시/저장도 PASS다.

Native/fresh WASM은 기존 **17개·49쪽**에 inline Print **4개·38쪽**을 더해 각각 **21개·
87쪽**을 확인했고 PNG 각각 87개가 S59와 bytes 동일하다. 기존 최저 raw는
**98.34664% / 98.79246%**, 추가 범위는 **99.66991% / 99.17555%**다. 별도 Native
fallback 5쪽은 모두 100%이며 S59와 동일하다. 현재 대표 review/overlay와 추가 선택 panel
16개를 직접 판독했고 원본의 글자 굵기/획 차이는 기록했다. 초기 글꼴 부족의 미달은 설치된
실물 face를 한도 내 우선 공급한 뒤 양쪽을 재출력해 해소했다. 제품 catalog 한도·원본·PDF·
조판 기대값/fixture/golden은 바꾸지 않았고 초기 실패 증거를 보존했다.

최적화 WASM 크기는 **+0.18920%**, JS bytes·선언 서명은 유지한다. source·검사·제품·
글꼴·전쪽 TSV·현재 판독 hash를 대조했다. 다음은 일반 State 항목·문단 원점/계산 metadata·
원장과 표/미주 내부 상태, 셀 유닛/행 색인 캐시·일부 frame/커닝/shaping이다. T04 전체·
T05–T09·최종 통합/최신 devel 보존 게이트와 host 책임,
`parserAllocationBoundsProven=false`를 유지한다.
