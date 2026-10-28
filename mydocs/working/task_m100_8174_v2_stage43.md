# #8174 v2-S43 — 표·셀·캡션 저장의 재귀 요청 전달

Issue: #8174. 2026-10-26, 기준 `dfafdc6a44`.

첫 HWP 저장의 같은 Storage를 표의 공통 속성·TABLE·셀/캡션 LIST_HEADER와 그 내부 문단까지
연결했다. 일반 저장도 같은 구현을 Unbounded로 호출한다. **나머지 컨트롤·Core pagination과
T04 전체, PR 제출 준비는 완료가 아니다.**

## 실제 적용

`try_serialize_paragraph_with_msb` → `try_serialize_control(Table)` →
`try_serialize_table` → `try_serialize_cell/try_serialize_caption` →
`try_serialize_paragraph_list` → 같은 문단 writer로 재귀한다. 표 CTRL_DATA의 삽입 위치와
캡션 → TABLE → 셀의 원래 기록 순서는 유지한다.

- 공통 개체 속성·TABLE·셀 추가 필드·캡션은 borrowed byte producer를 counting/fixed slice
  writer에 적용한다. CTRL_HEADER의 id 결합과 캡션 attr용 사본, 레코드 배열 증가도 먼저
  예약한다. 표 raw seal 판단·캡션 bit 29·셀의 폭 참조·수직 정렬·줄바꿈 규칙은 바꾸지 않는다.
- 셀 필드 이름의 기존 길이 검사와 UTF-16 view를 파서/저장기가 공유한다. UTF-16 Vec와
  비교용 String을 만들지 않고 기존 lossy 해석과 비교한다. 현재 이름과 같으면 손상 surrogate를
  포함한 원본 extra를 보존하며, 이름을 수정하면 기존 schema로 다시 기록한다. 출력용 UTF-16도
  Vec를 만들지 않고 스트리밍한다.
- 다른 Control variant는 아직 legacy writer를 소비한다. 해당 분기의 `bytes(0)` 전후 확인을
  실제 payload 예약으로 세지 않는다. 메모/마스터 페이지·그림/도형·나머지 control scratch는
  계속 감사·보정한다. Form의 전역 order처럼 writer 부수효과가 있는 경로를 counting으로
  두 번 실행한 것으로 확대하지 않는다.

## 실제 검증

ignored `output/pr-review/issue8174/v2-s43-20261026/`의 `production-freeze.json`과
`native-producer.json`에 source 파일 hash·실제 제품을 고정했다.

| 검증 | 결과 |
| --- | --- |
| Native Clippy·실제 제품 | `-D warnings`, release-test root opt-level=1 / codegen-units=64 빌드 통과 |
| 기존 serializer unit | **589 PASS**. 기대값 변경 없음 |
| 공개 요청/저장/wire 계약 | **16 PASS** |
| 신규 공개 셀 필드 계약 | 8종 UTF-16 × 원본/수정의 **16번 실제 기록·재파싱**. 독립 `String::from_utf16_lossy`와 논리 값 대조, 미수정 raw bytes 보존·수정 이름 반영. 변경 전 S42에서도 통과하는 호환성 대조이며 수정 전 FAIL 증거로 표시하지 않음 |
| raw 재사용을 끈 HWP/HWPX 9개 | 다시 생성한 9개 BodyText와 **34,600개 레코드 bytes 동일** |
| PDF 전체 저장·재열기 | 17개, **34개 저장·98쪽 재열기**, 상세 진단 3개. 사용량 이외 진단·render tree 동일 |
| 실제 저장 파일 | 일반/상세 **37개 bytes가 S42와 동일** |
| 선택 쪽 | 추가 HWP/HWPX **4개 왕복**, bytes·진단·render tree 동일 |
| CLI 음성·복구 | 출력/진단/잔여 예산 × HWP/HWPX **6 PASS**, 기존 파일·stdout 보존과 새 요청 정상 복구 |
| Native 새 시각 | CFF **100.00000%**, Type3 11쪽 **99.97900%**. 새 제품 캡처·대표 review 직접 판독으로 표/그림·본문·캡션 관계 유지 확인 |
| WASM target·feature-off | Clippy `-D warnings`와 `--no-default-features --lib` 통과 |
| source unit 정책 | 직전 HEAD 대비 **4,134 tests / 304 modules**, 증가 없음. 전체 브랜치의 기존 이관 차단은 유지 |

초기 Clippy에서 schema 주석 뒤 빈 줄 하나가 검출되어 제거했다. 타입·동작이나 기대값을
바꾸지 않았다. 실패 로그도 보존한다. 신규 검사는 렌더 좌표/golden이 아닌 셀 metadata wire
계약이며, 기존 자동 cases 수집 경로를 사용한다. 최종 suite 수집·실행은 T08에서 확정한다.

## 함께 확인한 기존 경계와 잔여 작업

`raw_provenance::record_digest`와 section fingerprint는 이미 JSON을 blake3 writer에
스트리밍한다. 별도 JSON 출력 Vec가 없으므로 이를 새로 수정하지 않았다. PDF 생성 section의
raw/tail 부재와 Table/Shape/Picture 소유 순회를 실제 source와 대조했다. 이 관측을 Form 등
모든 custom serializer의 scratch 검증으로 확대하지 않는다.

기존 deferred pagination은 편집 descriptor와 단일 구역의 cell-flow 대상에 한정되어 초기
문서 전체의 비동기 재열기를 바로 대체하지 못한다. 현재 동기 재열기를 중단 가능하다고
주장하지 않는다. Core pagination/중첩 composition, 다른 control과 format adapter/HWPX
XML/context, T05의 복사/재열기, T06 이후 작업과 최종 fresh WASM/Studio·시각/성능이 남는다.
S37 제품을 현재 source의 fresh 실행 증거로 재사용하지 않으며 capability false도 유지한다.
