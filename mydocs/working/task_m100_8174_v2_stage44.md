# #8174 v2-S44 — 구역·단·쪽 설정과 바탕쪽의 저장 예약

Issue: #8174. 2026-10-26, 기준 `be8979c1de`.

같은 요청 Storage를 SectionDef/ColumnDef, PAGE_DEF·각주/미주 모양·쪽 테두리와 바탕쪽
LIST_HEADER·내부 문단까지 전달했다. 일반 저장도 같은 borrowed producer를 Unbounded로
소비한다. **T04 전체·최종 WASM/Studio 검증과 PR 준비 완료는 아니다.**

## 실제 경로와 유지한 계약

`try_serialize_control`의 SectionDef/ColumnDef는 같은 CTRL_DATA 삽입 규칙을 소비한다.
SectionDef의 header와 자식 record를 분리해 counting 중 자식 문단이나 부수효과를 실행하지
않는다. 바탕쪽 raw header 복사·선택 쪽 슬롯 수정·일반 header/footer LIST_HEADER와 자식
문단도 같은 Storage로 연결한다. 루트 section의 extension master-page tail도 전달한다.

- 출력 버전·구역 flags의 page_num_type 비트, 단 너비 부족의 기존 폴백, 원본 extra와 직접
  CTRL_DATA의 중복 방지·중첩 원본 위치, 바탕쪽 optional page 번호의 기존 수정 위치를 유지한다.
- PAGE_DEF 방향 비트, 각주 구분선의 i32 길이와 위 여백, 추가 쪽 테두리와 원본 child record를
  그대로 소비한다. 명세 의미·좌표·기준값을 새로 바꾸지 않았다.
- 헤더 결합과 개별 payload·raw 사본·record 배열은 실제 생성/증가 전에 예약한다. 앞 절편의
  private TablePayload는 공통 ControlPayload로 정리했다. 다른 variant의 legacy 경로는
  여전히 실제 예약이 없는 상태로 명시하며 `bytes(0)`을 payload 예약으로 세지 않는다.

## PDF의 실제 생산 범위 대조

S43 생산 라이브러리로 17개 PDF의 Core와 첫 HWP snapshot을 각각 순회한 **34개 census**를
실행했다. S44가 바꾸지 않은 PDF lowering/adapter의 관측이다. source·라이브러리 hash와
문서별 집계는 ignored `output/pr-review/issue8174/v2-s44-20261026/control-census.json`에
보존한다. 개인 입력은 익명 label을 사용한다.

| 종류 | source 생산 지점과 관측 |
| --- | --- |
| Table | `structured_lowering`의 표/중첩 표 생성, 실제 Core/snapshot에서 관측 |
| Picture | `lowering::lower_picture`와 retained paint, 실제 관측 |
| Shape | `lowering`·`path_paint`·`structured_lowering/body`; Rectangle/Line/Polygon 관측 |
| SectionDef | snapshot adapter의 `insert_section_def_control`, 실제 snapshot에서 관측 |
| ColumnDef | census 이후 BodyText의 첫 문단 보충 분기에서 기본 단을 추가하는 source 경로 확인 |

constructor와 단순 분기 match를 구분해 대조했다. 관측한 17개만으로 모든 입력·향후 변경을
입증하지 않으며, 전체 다른 Control의 안전성을 이 표로 대신하지 않는다. 현재 PDF 생산
경로에 닿는 그림·도형 writer와 adapter/XML·Core pagination은 계속 필수 보정 대상으로 남긴다.

## 실제 검증

`production-freeze.json`과 `native-producer.json`에 source 파일 hash·실제 제품을 고정한다.

| 검증 | 결과 |
| --- | --- |
| Native 제품·Clippy | root opt-level=1 / codegen-units=64 빌드, `-D warnings` 통과 |
| 기존 serializer unit | **589 PASS**, 기대값 변경 없음 |
| 공개 요청/저장/wire 계약 | **16 PASS** |
| raw 재사용을 끈 실물 9개 | 다시 생성한 9개 BodyText·**34,600개 레코드 bytes 동일** |
| PDF 전체 저장·재열기 | 17개, **34개 저장·98쪽 재열기**, 상세 진단 3개. 사용량 이외 진단·render tree 동일 |
| 실제 저장 파일 | 일반/상세 **37개 bytes가 S43과 동일** |
| 선택 쪽 | 추가 HWP/HWPX **4개 왕복**, bytes·진단·render tree 동일 |
| CLI 실패·복구 | 출력/진단/잔여 예산 × HWP/HWPX **6 PASS**, 기존 파일·stdout 보존과 새 요청 정상 복구 |
| Native 새 시각 | CFF **100.00000%**, Type3 11쪽 **99.97900%**, 새 제품 review의 본문·표/그림·캡션 관계 유지 확인 |
| WASM target·feature-off | Clippy `-D warnings`와 `--no-default-features --lib` 통과 |
| source unit 정책 | 직전 HEAD 대비 **4,134 tests / 304 modules**, 증가 없음. 전체 PR의 기존 이관 차단은 유지 |

초기 변환 과정에서 두 순수 closure 반환(bool/u16)에 `?`가 잘못 붙어 컴파일 검사에서
검출됐다. 제거한 뒤 위의 제품·검사를 실행했으며 실패 로그도 보존했다. 생산 source는
그 보정 뒤 동결했다. 당시의 코드와 이번 실제 결과를 연결하며, 컴파일 실패를 제품 동작
실패나 검증 통과로 세지 않는다.

## 잔여 상태

그림/도형의 실제 저장 경로, 나머지 control·adapter/HWPX XML/context와 Core pagination/
중첩 composition, T05의 복사/재열기, T06의 의존성·JPEG 호환, T07–T09의 최종 포장·회귀·
시각/성능·base/PR 준비는 남는다. 현재 head의 최적화 fresh WASM/Studio 실행은 아직 없으며
S37 제품으로 대신하지 않는다. `parserAllocationBoundsProven=false`와 host 책임을 유지한다.
