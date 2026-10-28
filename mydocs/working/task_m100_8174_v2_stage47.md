# #8174 v2-S47 — HWPX ID pool·BinData context의 요청 예산

Issue: #8174. 2026-10-26, 기준 `7127bd5f6c`.

HWPX의 초기 ID 등록과 XML 쓰기 중 참조 집합, BinData metadata·manifest 대조에도 같은
요청 예산을 연결했다. 공개 Context/IdPool의 타입 인수·기존 메서드 계약은 유지한다.
**XML part writer 내부 scratch·T04 전체와 PR 준비 완료는 아니다.**

## 실제 경계와 오류 전파

`PdfImportSession::export_hwpx`가 자신의 ImportBudget Arc에서 요청용 예약 객체를 만든다.
같은 Arc를 참조하는 Allocation과 함께 HWPX writer에 전달하고, Context의 여섯 ID pool이
예약 객체를 공유한다. Allocation 자체는 여전히 빌린 ImportBudget이며 Arc를 소유한 것으로
취급하지 않는다. 전역/TLS 상태나 공개 API의 새 lifetime 인수는 추가하지 않았다.

- ID HashSet은 capacity가 늘기 전에 공통 hash payload 작업량을 예약하고 fallible reserve한다.
  초기 등록은 오류를 즉시 전파한다. 기존 반환형이 없는 `reference()`에서는 첫 오류를 보존하고
  이후 해당 집합의 증가를 중단한다. section/master-page XML 생성 직후, 그 part의 ZIP 기록 전에
  `check_storage()`가 오류를 전파한다. **part 문자열 생성 내부를 중단하는 구현은 아니다.**
- BinData/순번 map·차트 엔트리 배열과 manifest ID/href/MIME 문자열, 정렬한 metadata 사본,
  안정 정렬 scratch·미등록 ID 대조/오류 문자열을 생성 전에 예약한다. map 덮어쓰기에서는 새
  슬롯으로 계산하지 않는다. 기존 MIME·외부 링크/스트림 부재·ID 사상과 재귀 깊이는 유지한다.
- section/master-page 항목 배열·경로, manifest metadata 사본과 ZIP href 집합도 같은 Storage를
  사용한다. 3-way 대조는 href를 빌려 읽어 검사용 문자열 사본을 제거한다. 기존 집합 대조와
  실패 의미는 유지하며 실제 내용 손실을 빈 성공 ZIP으로 바꾸지 않는다.
- 예약 객체는 첫 저장 동안만 살아 있다. 반환한 DocumentCore의 일반 편집·저장을 과거 import
  카운터에 연결하지 않는다. hash/정렬 작업량은 보수적인 요청 작업이며 전체 heap/RSS 보증이 아니다.

## 실제 검증

ignored `output/pr-review/issue8174/v2-s47-20261026/`의 `production-freeze.json`,
`native-producer.json`, `checks.json`에 source·제품 hash와 명령·결과를 연결한다.
Native 제품은 root opt-level=1 / codegen-units=64의 기능 검증용이며 성능 측정 빌드가 아니다.

| 항목 | 결과 |
| --- | --- |
| 새 공개 HWPX 첫 저장 경계 | **S46 FAIL / 후보 PASS**. context의 잔여 예산 거절·sticky failure·live IR 보존·새 요청 bytes 동등 복구 |
| 공개 요청/저장/wire 계약 | 기존 18개 + 새 수명 회귀 1개 = **19 PASS** |
| 기존 adapter/serializer + Allocation | **662 + 9 = 671 PASS**, 기존 fixture/기대값 변경 없음 |
| PDF 저장·재열기 | 17개·**34개 저장/98쪽 재열기**, 상세 진단 3개·선택 쪽 추가 4개 왕복 유지 |
| 실제 저장 파일 | 일반/상세 **37개 bytes가 S46과 동일** |
| CLI 한도 실패·복구 | 출력/진단/잔여 예산 × HWP/HWPX **6 PASS**, 기존 파일·stdout 보존과 새 요청 복구 |
| Native 새 시각 | CFF **100.00000%**, Type3 11쪽 **99.97900%**. 새 review/overlay 6개 bytes가 S46과 동일 |
| Rust 검사 | Native/WASM Clippy `-D warnings`, feature-off·fmt 통과 |
| 테스트 정책·수집 | 직전 HEAD 대비 **4,134 tests / 304 modules**, source-side 증가 없음. 기존 suite 배정 계약 **23 PASS** |

같은 공개 CFF 입력으로 실제 한도를 64바이트씩 올린 실행도 보존했다. 시작 decoded 값은
1,624,457이며 아래 값은 여기에 더한 한도다. 정상 import 뒤 실제 첫 HWPX 저장을 호출했다.

| 추가 한도 | 관측한 경계 |
| --- | --- |
| 64 | 예약 객체 24바이트를 계상한 뒤 초기 등록의 320바이트 hash 작업을 거절 |
| 1,536 | context 초기화를 지난 뒤 ZIP의 60바이트 증가를 거절 |
| 2,752 | section 경로 등 metadata의 32바이트 증가를 거절 |
| 3,008 / 3,328 | XML 쓰기 중 런타임 참조 집합의 288바이트 hash 예약을 거절 |
| 3,712 | 참조 집합 경계를 지난 뒤 ZIP의 2,500바이트 증가를 거절 |
| 21,184 | 전체 첫 저장 성공 |

`context-sweep.tsv`, `context-boundaries.log`와 producer hash가 실행 증거다. 단순히 마지막
카운터만 바뀐 것으로 판단하지 않고, 코드의 등록 → metadata → section 참조 → ZIP 소비와
실제 failure의 used/requested/limit를 대조했다. 이 수치를 allocator의 live heap 크기로
해석하지 않는다. 프로필·명시적 host cap은 올리지 않았다.

새 review를 직접 열어 CFF의 글자·기호·간격, Type3 26–30번 행·그림 9/10·캡션·괘선·앞뒤
본문 관계를 대조했다. 기존 미세 래스터 차이를 유지하며 새 누락·겹침은 관측하지 않았다.
전체 49쪽의 최종 Native/fresh WASM 확인을 두 쪽의 선행 확인으로 대신하지 않는다.

초기 검사에서 빌린 Allocation으로 Arc를 복제하려던 타입 오류와 Clippy의 조건 순회 지적을
보정했다. 예약 객체를 실제 Arc 소유자인 Session에서 만드는 현재 경로로 동결한 뒤 위 검사를
실행했다. 공개 회귀의 최초 독립 harness는 serde_json 연결 누락을 보정한 뒤 S46의 실제 실패를
확인했다. 별도 ID 실험의 실패는 파서가 manifest 순번으로 ID를 재부여하는 계약과 다른
동일성 가정이었다. 그 단정을 정식 회귀로 넣거나 parser/ID 의미를 변경하지 않았다.

## 잔여

XML part writer의 attribute/escape·template/run 임시 문자열/배열, table-grid/content-loss
등 나머지 HWPX scratch, Core pagination/중첩 composition·table-frame writeback,
T05 복사/재열기와 T06–T09 의존성·JPEG·최종 포장/회귀/시각/성능·base/PR 준비는 남는다.
현재 source의 최적화 fresh WASM/Studio 실행은 아직 없으며 이전 S37 제품으로 대신하지 않는다.
전체 PR의 source unit 이관 차단과 `parserAllocationBoundsProven=false`·host 책임을 유지한다.
