# #8174 v2-S46 — HWP 저장 adapter의 요청 예산 연결

Issue: #8174. 2026-10-26, 기준 `10f2096458`.

첫 HWP snapshot의 변환 adapter에도 같은 Storage를 전달했다. 구역 정의 사본, 문단·글상자·
표의 wire-only 필드와 참조 집합을 실제 생성·증가 전에 예약한다. 일반 저장도 같은 변환을
Unbounded로 소비하며 공개 converter의 기존 반환형은 유지한다. **T04 전체·최종 제품 검증과
PR 준비 완료는 아니다.**

## 실제 생산·소비와 보존 계약

`PdfImportSession → try_prepare_hwp_export_snapshot → try_convert_if_hwpx_source →
convert_to_hwp_ir`와 문단·도형·중첩 표·캡션의 재귀 경로가 같은 Storage를 사용한다.
변환 뒤 DocInfo raw 캐시를 재생성해야 할 때도 기존 fallible DocInfo writer에 전달한다.

- 구역/컨트롤의 세 PAGE_BORDER_FILL 보충과 SectionDef 사본, control 배열·Box를 예약한다.
  SectionDef의 바탕쪽 문단은 serde에서 빠지는 runtime 소유 필드도 예약한 뒤 복사한다.
  BodyText의 첫 구역 정의 사본도 같은 계약을 소비한다. 선행 컨트롤 자리 확보 함수는 기존
  offset·글자모양·range tag·LineSeg를 제자리에서 이동하며 추가 배열을 생성하지 않음을 확인했다.
- 문단의 CTRL_DATA 슬롯, 자동 번호 range tag, 문단/구역/셀/글상자 tail, 표 row-size 배열,
  공통 개체 속성 payload의 backing을 생성 전에 예약한다. 기존 flags·폭·여백·record schema와
  변환 순서는 바꾸지 않는다. 그림 기하 순회의 raw extra·수식 글꼴 문자열도 같은 Storage로 연결했다.
- 문단/개체 border-fill 참조 HashSet의 실제 capacity 증가에는 fallible reserve를 적용한다.
  BinData 순서 수집은 기존 순회 순서와 BTreeSet 중복 제거를 유지한다. BTree entry는 공통
  논리 작업량 예약이며 표준 라이브러리 node layout이나 전체 heap 상한을 입증한 수치가 아니다.
- BinData metadata 전체의 선행 사본을 제거하고 필요한 항목만 예약 후 복사한다. binary content는
  예약한 Option 슬롯과 결과 배열로 이동한다. 중복 ID의 첫 content 선택, 유효 ID의 remap,
  범위 밖/0 ID 꼬리의 원래 순서와 lazy resolver의 비실행을 보존한다.
- 그림 href·표 layout ParameterSet은 borrowed binary producer가 counting/fixed-slice와 일반
  저장에 같은 필드를 발행한다. 정규화 문자열·UTF-16 중간 Vec를 제거하며 기존 escape 규칙과
  65,535-unit 제한을 보존한다. href의 마지막 high surrogate가 한도에서 잘리는 기존 wire 계약을
  description용 HWP string의 다른 절단 계약과 혼동하지 않는다.

HWPX 전용 바탕쪽 slot 물질화는 기존 함수를 유지했다. `FileFormat::Pdf` 분기는 그 호출과
HWPX/HWP3 출처 marker 보강에 들어가지 않는다. 이 비PDF 경로와 일반 Group/OLE 저장까지
전체 allocation proof를 확보한 것으로 표시하지 않는다. Core의 table-frame writeback과
header/footer marker 처리는 별도 소비 지점이며 adapter 연결만으로 전체 완료로 세지 않는다.

## 실제 검증

생산 source를 동결한 뒤 Native 제품과 검사를 실행했다. ignored
`output/pr-review/issue8174/v2-s46-20261026/`의 `production-freeze.json`,
`native-producer.json`, `checks.json`에 source·binary hash, 실제 명령과 결과를 연결한다.
Native 기능 제품은 root opt-level=1 / codegen-units=64이며 최종 성능 측정 빌드가 아니다.

| 항목 | 결과 |
| --- | --- |
| 기존 adapter·serializer unit | **662 PASS**. 이 중 기존 serializer 589개·converter 73개. fixture/기대값은 유지하고 private 함수의 Storage 인수와 Result 소비만 맞춤 |
| 공개 요청/저장/wire 계약 | 기존 16개 + 새 adapter 2개 = **18 PASS** |
| adapter 전후 대조 | 실물 HWP/HWPX 11개, IR 진단·보고·재생성 BodyText **35개 파일 bytes 동일** |
| 그림·도형 wire 대조 | **1,264개 조합 bytes 동일**, S45의 검증된 제품 wire와 대조 |
| 실물 BodyText 재생성 | raw stream 재사용을 끈 9개·**34,600개 레코드 bytes 동일** |
| PDF 저장·재열기 | 17개·**34개 저장/98쪽 재열기**, 상세 진단 3개·선택 쪽 추가 4개 왕복 유지 |
| 실제 저장 파일 | 일반/상세 **37개 bytes가 S45와 동일** |
| CLI 실패·복구 | 출력/진단/잔여 한도 × HWP/HWPX **6 PASS**, 파일·stdout 보존과 새 요청 복구 |
| 같은 낮은 한도 대조 | **1,675,885**에서 S45 성공·후보 거절, 기존 출력 보존·새 요청 bytes 동등 복구 |
| Native 새 시각 | CFF **100.00000%**, Type3 11쪽 **99.97900%**, 새 review/overlay 6개 bytes가 S45와 동일 |
| Rust 검사 | Native/WASM Clippy `-D warnings`, feature-off·fmt 통과 |
| source unit 정책 | 직전 HEAD 대비 **4,134 tests / 304 modules**, cfg support 항목도 증가 없음 |
| 수집 계약 | 새 `tests/cases/` source의 기존 자동 배정 계약 검사 통과. 파생 manifest나 CI 개별 호출은 추가하지 않음 |

새 href 검사는 한글·보조 평면 문자·여러 scheme delimiter·기존 escape와 UTF-16 한도 경계를
독립적인 ParameterSet 필드/표준 UTF-16 변환으로 확인한다. 변경 전 S45에서도 통과하는
호환성 보호이며 수정 전 실패 증거로 표시하지 않는다. 새 BinData 검사는 첫 중복 항목·ID remap·
잘못된 ID 꼬리·lazy 미해석과 내용 보존을 실제 공개 converter에서 확인한다. 두 검사 모두
렌더링 좌표·fixture/golden을 고정하는 검사가 아니다.

같은 낮은 한도 대조에서 CFF의 decoded 논리 작업량은 1,671,886 → 1,675,015다. 추가
3,129바이트의 계상 뒤 최종 진단 인코딩의 3,999바이트 요청이 기존 한도를 넘어서 거절됐다.
**adapter leaf에서 직접 거절된 사례로 주장하지 않는다.** 실제 추가 계상이 후속 단계까지
유지되는 대조이며, 사용량을 heap/RSS나 실행 시간으로 해석하지 않는다. 프로필과 host cap은
올리지 않았다. 실행은 `adapter-budget-contrast.json`에 보존한다.

새 review를 직접 열어 CFF의 글자·기호·간격과 Type3 26–30번 행·그림 9/10·캡션·앞뒤 본문·
괘선 관계를 대조했다. 기존 미세 래스터 차이를 유지하며 배치 누락이나 겹침은 관측하지 않았다.
전체 49쪽의 최종 Native/fresh WASM 검증을 이 두 쪽의 확인으로 대신하지 않는다.

IR 대조 harness의 최초 컴파일은 전체 FileHeader/DocInfo/Section에 Serialize가 없어서 실패했다.
제품 타입의 derive를 늘리지 않고 Debug 진단으로 바꿨다. `adapter-equivalence.json`에 해당
진단의 형식을 명시하며 BodyText bytes와 보고도 별도로 대조한다. 기계적 변환 중 지역 walker
동명 함수의 적용 범위, row-size 타입 추론과 u32 필드는 초기 fmt/compile에서 검출·보정했다.
위 결과는 보정 후 동결한 source의 실행이며 초기 검사 실패를 제품 실패나 통과로 세지 않는다.

## 남은 범위

T04의 HWPX XML/context와 Core pagination/중첩 composition·table-frame writeback 등
나머지 실제 할당 경계, T05의 복사/재열기, T06의 의존성·JPEG와 T07–T09 최종 포장·전체 회귀·
시각/성능·base/PR 준비는 남는다. 현재 source의 최적화 fresh WASM/Studio 실행은 아직 없으며
S37의 이전 제품으로 대체하지 않는다. 전체 PR의 source unit 이관 차단과
`parserAllocationBoundsProven=false`·host 책임을 유지한다.
