# #8174 v2-S48 — 공통 XML 인코더와 HWPX header의 요청 예산

Issue: #8174. 2026-10-26, 기준 `f3098b7085`.

HWPX header의 출력 버퍼와 숫자·속성·글자모양·채우기 scratch에 첫 저장의 같은 요청 예산을
연결했다. 공개 XML helper·`write_header`·`write_char_pr`의 기존 호출 계약은 유지한다.
**section/master-page 등 나머지 XML 호출부·Core 경계와 T04 전체는 아직 완료하지 않았다.**

## 변경 경계

- 공통 XML producer가 빌린 입력에서 XML 1.0 필터·escape 결과를 계산하고 같은 encoder로
  기록한다. Allocation은 크기를 계산해 예약한 뒤 실제 payload를 만든다. 속성별 필터 문자열과
  escape 사본을 제거했고, XML 이벤트는 계속 quick-xml Writer가 소비한다. 태그를 직접 출력해
  Writer의 상태를 우회하지 않는다. 일반 공개 helper는 기존처럼 Unbounded 경로를 사용한다.
- 속성의 탭/LF/CR은 `&#9;`/`&#10;`/`&#13;`, Text 이벤트의 CR은 `&#13;`으로 보존한다.
  기존 `xml_escape`의 탭/LF/CR은 그대로 둔다. 이 세 계약을 구분하며 다섯 markup 문자·잘못된
  XML 문자 제거·Unicode·빈 속성/텍스트·선언과 들여쓰기도 기존 bytes를 유지한다.
- `serialize_hwpx_with_storage → write_header_with_storage → BudgetCursor`로 같은 Storage를
  전달한다. header 증가와 원문 XML splice 전에 예약하며, 이 버퍼는 decoded 작업량을 소비한다.
  압축한 ZIP의 출력 상한을 압축 전 XML 상한으로 잘못 재사용하지 않는다.
- header의 숫자·색·글꼴 참조 문자열, charPr의 언어별 값과 장식도 guarded helper를 사용한다.
  고정 크기 속성 목록은 stack 배열로 바꿨다. 물리 문단 여백을 정규화할 때의 ParaShape 사본은
  `clone_record`로 예약한다. 같은 채우기 writer를 공유하는 borderFill도 Storage를 전달하고,
  gradient/image의 불필요한 전체 사본 대신 원본을 빌린다. 색·alpha·문자/자식 순서는 유지한다.
- 이 변경은 논리적 요청 작업량의 예약이다. 프로세스 전체 heap/RSS 상한이나 모든 parser 할당의
  증명이 아니며 `parserAllocationBoundsProven=false`와 host 책임을 유지한다.

## 실행 결과

ignored `output/pr-review/issue8174/v2-s48-20261026/`의 `production-freeze.json`,
`native-producer.json`, `checks.json`에 source·제품 hash와 실행 명령을 보존했다.
Native 제품은 root opt-level=1 / codegen-units=64의 기능 검증용이다.

| 항목 | 결과 |
| --- | --- |
| 공개 요청/저장/wire 계약 | 기존 19개 + header/속성 경계 2개 + XML wire 2개 = **23 PASS** |
| 기존 serializer/adapter·Allocation | **662 + 9 = 671 PASS**, 기존 기대값 변경 없음 |
| 실제 HWP/HWPX 11개 전후 대조 | header 11개·정상 ZIP 10개·기존 양식 거절 오류 1개의 **22개 파일 bytes 동일** |
| PDF 저장·재열기 | 17개·**34개 저장/98쪽 재열기**, 상세 진단 3개·선택 쪽 추가 4개 왕복 유지 |
| PDF 저장 파일 | 일반/상세 **37개 bytes가 S47과 동일** |
| CLI 실패·복구 | 출력/진단/잔여 예산 × HWP/HWPX **6 PASS**, 파일/stdout 보존과 정상 후속 요청 |
| Native 새 시각 | CFF **100.00000%**, Type3 11쪽 **99.97900%**, 새 review/overlay 6개의 bytes도 S47과 동일 |
| Rust·수집 검사 | Native/WASM Clippy `-D warnings`, feature-off·fmt·suite 배정 계약 **23 PASS** |
| 절편 source unit 정책 | 직전 HEAD 대비 **4,134 tests / 304 modules**, source-side 증가 없음 |
| 문서 검사 | 변경 문서 3개의 링크 정상. 장기 문서 metadata의 기존 20건은 대상 631개가 HEAD와 bytes 동일함을 확인했고 **신규 오류 0건** |

`header-wire-equivalence.json`, `byte-equivalence.json`, `visual-review-identity.json`이 파일별
대조 증거다. HWP/HWPX 11개 중 `hwpers_test4_complex_table.hwp`의 직접 HWPX 저장은 양쪽 모두
`FORM_ACTION_INVALID: invalid or excessive form action`으로 거절했다. 정상 저장 11건으로
세지 않았으며 header와 오류의 bytes 보존을 확인했다.

수명 계약과 XML wire는 정식 `tests/cases/` 원본으로 두고 현재 Native 라이브러리에 직접 연결해
실행했다. CI 설정이나 source-side private unit을 추가하지 않았다. 최종 suite 준비·전체 실행은
T08에서 수행하며 이 집중 실행을 전체 suite 통과로 표시하지 않는다.

## 실제 거절 경계와 대조군

S47 제품에서 정상 import 후 잔여 decoded 예산을 128 KiB로 제한했을 때, 256 KiB header 원문과
256 KiB의 `&` 글꼴 속성이 모두 저장됐다. 압축 ZIP이 작다는 이유로 압축 전 XML 생성 비용이
예산에 잡히지 않는 두 경로를 재현했다. 당시 test source를 `session-contracts-before.rs.txt`에
보존하고 `header-budget-before-producer.json`의 hash와 일치시켰다.

후보에서는 처음의 고정 128 KiB가 정상 저장도 거절했다. 새로 계상한 정상 저장 작업량을 실제로
측정했다: CFF의 시작 decoded는 1,624,457, 첫 HWPX 저장 뒤 1,790,306, 증가분은 165,849이며
ZIP은 6,984바이트다(`normal-budget.log`). 이것은 새 계상 범위의 작업량이지 live heap 측정이 아니다.

최종 후보 경계 검사는 정상 저장 실측값 + 64 KiB로 요청 한도를 정한다. 후보에서 이 한도의
정상 대조군이 저장됨을 확인하고, 512 KiB 원문 tail은 `head tail splice`, 256 KiB `&` 속성의
escape 확장은 `HWPX XML` 단계에서 거절되는 것을 검증했다. 실패 카운터는 한도 이내에 머물고
후속 호출은 실패를 유지한다. live IR·일반 저장 bytes를 보존하며 새 기본 요청에서 같은 bytes로
복구한다. **S47 재현과 최종 후보 검사는 위와 같이 한도/원문 크기가 다르다. 같은 매개변수의
FAIL/PASS 결과라고 표시하지 않는다.** 제품의 기본/host 한도는 높이지 않았다.

초기 기계적 변환의 함수 인수 위치·중복 import·괄호/Clippy 문제를 보정했다. 첫 XML 대조에서
발견한 quick-xml의 문맥별 whitespace escape 차이도 수정한 뒤 production을 고정하고 위 제품을
만들었다. 기존 XML 회귀 기대값이나 기준 출력은 변경하지 않았다.

새 CFF review에서 기호·ligature·간격을, Type3에서 26–30번 행의 본문·괘선·그림 9/10·캡션·
앞뒤 내용 관계를 직접 판독했다. 이전 미세 래스터 차이를 유지하며 새 누락·겹침은 관측하지
않았다. 이 두 쪽의 Native 선행 확인을 전체 49쪽의 최종 Native/fresh WASM 시각 검증으로
대신하지 않는다.

## 다음 작업

같은 guarded XML 경로를 section/master-page와 run/template·표/도형 등의 실제 scratch 소비까지
전달하고 table-grid/content-loss 및 Core pagination/중첩 composition·table-frame writeback을
계속 확인한다. T05 복사/재열기, T06 JPEG·의존성, T07–T09 최종 포장·전체 회귀·시각/성능·
base/PR 준비도 남는다. 현재 source의 최적화 fresh WASM/Studio 실행은 아직 없고 S37 제품을
현재 결과로 사용하지 않는다. 전체 source unit 이관 차단도 유지한다. 원격 작업은 수행하지 않았다.
