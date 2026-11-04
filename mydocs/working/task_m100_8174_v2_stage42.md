# #8174 v2-S42 — 첫 HWP 저장의 문단 payload·본문 임시 버퍼

Issue: #8174. 2026-10-26, 기준 `d9a1c55858`.

같은 요청 Storage를 첫 HWP 저장의 루트 문단 직렬화에 전달했다. 현재 단계의 실행 로그는
ignored `output/pr-review/issue8174/v2-s42-20261026/`에 남긴다. **T04 전체나 PR 준비 완료는
아니다.** 아래의 실행 완료 범위와 잔여 경계를 구분한다.

## 변경한 실제 경로

`PdfImportSession::export_hwp` → CFB/section writer →
`try_serialize_paragraph_with_msb` → `try_serialize_para_text`와 borrowed `ParaPayload`가
같은 Storage를 소비한다. 일반 저장도 같은 wire producer를 Unbounded로 호출한다.

- UTF-16 code-unit 배열·UTF-8 문자 인덱스·최종 LE byte backing, 필드 종료의 BTree/HashMap
  인덱스와 값 배열, 안내문 복원/서식 시프트 배열은 실제 증가 전에 검사한다. 8유닛 제어문자와
  안내문은 기존 emitter를 그대로 소비하며 정확한 길이를 먼저 예약한다. 필드의 닫힘·열림·빈
  필드 순서와 자리표시자 판정은 바꾸지 않는다.
- 문단 헤더·글자모양·저장 줄·range tag는 같은 borrowed producer를 counting writer와 예약된
  slice writer에 적용한다. 원본 extra header도 빌려 읽으며 counting 중 복사하지 않는다.
  형광펜 range tag는 한 번 생성해 헤더 수량과 실제 payload가 함께 소비한다.
- HWPX 축의 저장 줄 사본과 레코드 배열 증가, 5.0.3.2 헤더의 2바이트 추가, 보존 trailing
  record의 데이터 복사도 예약한다. 첫 문단에 secd/cold를 보충하기 위한 문단 복사·삽입도
  실패를 반환한다. secd의 전체 특수 runtime payload까지 입증한 것으로 확대하지 않는다.
- 메모 목록은 변경하지 않는 원본 문단을 빌려 선행 deep copy 한 번을 없앴다. 메모 출력에
  필요한 변경용 사본과 재귀 control writer의 내부 할당은 이번 완료 범위가 아니다.
- ordered-map의 기존 logical entry 계상은 공통 Storage로 옮겼다. std BTree node의 실제
  allocator 크기나 RSS 상한을 입증한 것으로 표현하지 않는다.

## 실행 증거

생산 파일 SHA-256은 `production-freeze.json`, Native 제품은 `native-producer.json`에 고정한다.
Native 빌드 뒤 기존 메모 검사 fixture 한 곳을 Vec 소유에서 같은 내용의 borrowed slice로
고쳤다. 기대값은 변경하지 않았다. 첫 검사 빌드의 타입 오류와 보정 전 파일을 보존하고,
`test-only-after-native-build.json`으로 생산 prefix가 동일함을 확인했다.

| 검증 | 결과 |
| --- | --- |
| Native 제품·Clippy | root opt-level=1 / codegen-units=64 실제 제품 빌드와 Native Clippy `-D warnings` 통과 |
| 공개 요청/저장/글꼴·wire 계약 | **15 PASS** |
| 원본 raw 재사용을 끈 HWP/HWPX 9개 | 다시 생성한 9개 BodyText·재파싱한 **34,600개 레코드 bytes 동일**. 필드·탭·표/그림·HWP3 대조 포함 |
| PDF 전체 저장·재열기 | 17개, **34개 저장·98쪽 재열기**, 상세 진단 3개. 사용량 이외 진단·재열기 render tree 동일 |
| 저장 파일 | 일반/상세 **37개 bytes가 S41과 동일** |
| 선택 쪽 | 추가 HWP/HWPX **4개 왕복**의 bytes·진단·render tree 동일 |
| CLI 음성·복구 | 출력·진단·잔여 예산 × HWP/HWPX **6 PASS**. stdout/기존 출력 보존, 새 요청의 byte 동일 복구 |
| Native 영향 페이지 | 실제 새 캡처 CFF **100.00000%**, Type3 11쪽 **99.97900%**. 대표 review 직접 판독으로 표/그림/캡션·본문의 순서와 간격 유지 확인 |

기존 본문 단위 검사 **34 PASS**를 확인했다. WASM target Clippy `-D warnings`와
feature-off `--no-default-features --lib`도 통과했다. source unit 정책은 직전 HEAD 대비
**4,134 tests / 304 modules**, 증가가 없다. 새 원격 CI 실행이나 최적화 fresh WASM 제품을
수행한 것으로 표시하지 않는다.

## 잔여 경계

재귀 control payload·메모/마스터 페이지의 변경용 사본, format adapter와 HWPX XML/context,
Core pagination·중첩 composition은 T04에 남는다. S41의 전체 source unit 이관 차단,
T05의 복사/동기 재열기, T06 이후 의존성·포장·최종 실행도 유지한다. `parserAllocationBoundsProven`
은 여전히 false다. S37의 WASM 제품은 이 변경분의 fresh 실행 증거가 아니며, 최종 조합의
T08에서 실제 제품·전체 시각·성능을 확정한다.
