# #8174 v2-S49 — section/master-page XML 조립의 요청 예산

Issue: #8174. 2026-10-26, 기준 `4dff1b1668`.

section 템플릿 치환과 문단 누적, master-page 본문·참조·최종 조립에 첫 저장의 같은 요청 예산을
연결했다. 기존 공개 호출 형식과 XML bytes는 유지한다. **run/lineseg·중첩 control writer 내부의
생성 비용과 Core 경계는 남아 있으며 T04 전체 완료가 아니다.**

## 실제 전달 경로

`serialize_hwpx_with_storage`가 `write_section_with_storage`와
`render_master_page_xml_with_storage`에 같은 Storage를 전달한다. 일반 공개 함수는 기존
형식을 유지하며 Unbounded 경로를 사용한다.

- 빌린 문자열 구간을 쓰는 공통 `xml_replacen`이 기존 literal 치환 순서와 횟수를 유지하면서
  새 결과의 증가 전에 예약한다. 쪽 정의·테두리·visibility·secPr scalar·각주/미주 모양과
  첫 lineseg/문단/run 치환, 후속 secPr의 템플릿 사본도 같은 Storage를 소비한다.
- master ID 배열·문자열·참조, 첫 문단 여는 태그와 column 정의의 숫자·색·단별 폭 XML도
  생성 전에 예약한다. colPr/secd 원문 순서를 복원할 때의 insert도 증가 전에 예약한다.
- 추가 문단과 master-page 본문은 문자열을 누적하기 전에 예약한다. master-page 최종 format도
  같은 요청을 사용한다. paragraph parts가 돌아온 직후 context 실패를 확인해 이후 문단의
  생성을 중단한다. **paragraph parts 내부가 즉시 중단되는 구현은 아니다.**
- 기존 section 생성 함수 중 이번에 변경하지 않은 82개의 코드가 bytes 그대로임을 별도로
  확인했다(`unchanged-section-producers.json`). 기존 조판 규칙·속성 기대값은 바꾸지 않았다.

## 검증

ignored `output/pr-review/issue8174/v2-s49-20261026/`의 `production-freeze.json`,
`native-producer.json`, `checks.json`에 source·제품 hash·명령·결과를 보존했다. Native 제품은
root opt-level=1 / codegen-units=64의 기능 검증용이며 성능 측정 제품이 아니다.

| 항목 | 결과 |
| --- | --- |
| 새 공개 조립 경계 2개 | 같은 test source로 **S48 FAIL / 후보 PASS**, 실패 유지·live IR 보존·새 요청 복구 |
| 공개 요청/저장/wire 계약 | 기존 23개 + section/master-page 2개 = **25 PASS** |
| 기존 serializer/adapter·Allocation | **662 + 9 = 671 PASS**, 기존 기대값 변경 없음 |
| 실제 HWP/HWPX 11개 전후 대조 | header 11개·정상 ZIP 10개·기존 양식 거절 오류 1개의 **22개 파일 bytes 동일** |
| PDF 저장·재열기 | 17개·**34개 저장/98쪽 재열기**, 상세 진단 3개·선택 쪽 추가 4개 왕복 유지 |
| PDF 저장 파일 | 일반/상세 **37개 bytes가 S48과 동일** |
| CLI 실패·복구 | 출력/진단/잔여 예산 × HWP/HWPX **6 PASS**, 파일/stdout 보존과 정상 후속 요청 |
| Native 새 시각 | CFF **100.00000%**, Type3 11쪽 **99.97900%**. 새 review/overlay 6개의 bytes도 S48과 동일 |
| Rust·수집 검사 | Native/WASM Clippy `-D warnings`, feature-off·fmt 통과. suite 배정 계약 **23 PASS** |
| 절편 source unit 정책 | 직전 HEAD 대비 **4,134 tests / 304 modules**, source-side 증가 없음 |
| 문서 검사 | 변경 문서 3개의 링크 정상. metadata의 기존 20건은 대상 631개의 HEAD bytes 동일성을 확인했고 **신규 오류 0건** |

기존 양식 거절은 `hwpers_test4_complex_table.hwp`의 직접 HWPX 저장에서 양쪽 모두 반환한
`FORM_ACTION_INVALID: invalid or excessive form action`이다. 정상 저장 11건으로 세지 않았다.
파일별 대조는 `header-wire-equivalence.json`, `byte-equivalence.json`에 보존했다.

새 Native review를 열어 CFF의 글자·기호·ligature·간격과 Type3의 26–30번 행·괘선·그림
9/10·캡션·앞뒤 내용 관계를 직접 대조했다. 기존 미세 래스터 차이를 유지하며 새 누락·겹침은
관측하지 않았다. 이 두 쪽의 선행 확인을 전체 49쪽의 최종 Native/fresh WASM 검증으로
대신하지 않는다. 새 회귀는 `tests/cases/`에 두고 현재 Native 라이브러리에 직접 연결해
실행했으며 최종 suite 준비·전체 실행은 T08에서 수행한다.

## 재현과 계상 범위

S48의 라이브러리를 보존한 뒤 새 공개 회귀를 실행해 section과 master-page에서 모두 조립 예산
우회를 재현했다. 같은 공개 CFF 문서를 열고, 그 버전의 정상 첫 저장 실측값 + 64 KiB를 한도로
정한다. 기존 문단을 바탕으로 `<&`를 262,144회 반복한 텍스트를 추가 문단 또는 master-page에 넣는다.
압축 ZIP이 64 KiB 미만이어도 후보는 조립 증가를 거절하고, live IR·일반 저장 bytes를 보존한다.
실패한 요청은 실패 상태를 유지하며 새 기본 요청에서는 같은 bytes로 저장된다.

`part-budget-before-producer.json`의 test source hash가 후보에서 실행한 정식 source와 같다.
각 버전의 정상 작업량을 측정하는 동일한 한도 산식을 쓰므로 절대 cap 수치는 다를 수 있다.
이 입력은 caller가 수정한 IR의 할당 경계 검사이며 독립 한컴 문서의 시각 증거로 사용하지 않는다.

CFF 정상 저장의 시작 decoded는 1,624,457, 저장 뒤 1,980,556, 증가분은 356,099이며 ZIP은
6,984바이트다(`normal-budget.log`). 새로 예약한 템플릿 사본과 조립 작업을 포함한 논리적
작업량이다. heap/RSS 크기나 실행 시간 증가율로 해석하지 않는다. 제품의 기본/host 한도는
높이지 않았고 `parserAllocationBoundsProven=false`와 host 책임을 유지한다.

## 잔여

run/lineseg·중첩 control의 내부 문자열/벡터, table-grid/content-loss, Core pagination/중첩
composition·table-frame writeback을 계속 확인한다. T05 복사/재열기와 T06–T09의 JPEG·의존성·
포장·전체 회귀·전쪽 시각/성능·base/PR 준비, 전체 source unit 이관도 남는다. 현재 source의
최적화 fresh WASM/Studio 실행은 아직 없고 S37 제품을 현재 결과로 사용하지 않는다. 원격
push·PR 생성·registry 업로드는 수행하지 않았다.
