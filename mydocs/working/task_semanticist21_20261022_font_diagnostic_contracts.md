# 글꼴 진단 집계의 측정 출처와 문자별 프로그램 소유

## 범위와 선행 근거

현재 45개 PR 통합 범위와 행정업무편람 추가 개선 제외를 유지한다. 새 PR은
체리픽하지 않는다. 현재 실패 46개 중 글꼴 coverage 두 개와 세션 등록 분류
하나를 함께 보정하며 다른 실패의 완료를 뜻하지 않는다.

원 #4962는 PR #5857, 실제 merge `4bd9c5d60efe85d80e935ff76ed5441c17d56699`에
연결된다. 도입 `9d7a0ef765`와 Stage M100의 분류·분모 계약을 확인했다.
metric entry가 있고 characterMatch가 hit이며 상위 alias/surrogate 관계가
없으면 exact-hit이다. 미등록 출처나 모순된 상태는 계속 실패해야 한다.

현재 producer는 canonicalHangulMetric와 declaredGlyphFallbackMetric를 발행한다.
앞 출처는 Rust 집계에 있지만 공개 분류 계약에는 없고, 뒤 출처는 Rust 집계에서도
빠져 실제 공개 API가 unclassified widthSource 오류를 낸다. frozen Native117의
1815 진단에서 `Ж😀Ж`의 가운데 문자만 Segoe UI Emoji entry494를 소비하며
양쪽 문자는 HCR Batang entry0의 char-miss임을 확인했다. 원래의 글꼴 선택·폭을
테스트 통과용으로 되돌리지 않는다.

## 생산 결과와 소비 위치

text_measurement의 CharWidthDecision → coverage의 classify_decision → category와
decisionUsage를 추적했다. 분류뿐 아니라 해당 구간의 첫 metric 이름을 모든 문자에
재사용하는 별도 경로도 있다. 문자별 entry가 달라지면 요청/해소 face 이름도 해당
결정의 이름이어야 한다. 같은 이름 쌍의 Arc만 재사용하며 측정 결정을 재계산하지 않는다.

집계·분류는 읽기 전용 Native 분석 표면이다. renderer의 lookup·실제 폭·배치·paint를
바꾸지 않는다. W2 trace와 W3 aggregate를 문자별 실제 entry/name/source로 대조하고
IR·SVG·HWP/HWPX 저장 바이트 불변 검사를 유지한다. 새 렌더링 fixture나 좌표 기대값을
추가하는 작업으로 분류하지 않으며 기존 시각 증적을 새 제품의 재출력으로 표시하지 않는다.

세션 등록은 register_text_font_glyph_coverage_native의 pua_coverage만 갱신한다.
문서 IR·원본 stream·serializer 입력을 바꾸지 않으므로 passthrough 면제의
SessionState에 정확한 공개 API 보존 검사를 근거로 등록한다. 무효화 호출을 넣어
통과시키거나 미분류 함수를 모두 허용하지 않는다.

## 보정과 검증 계획

- 분류 계약 JSON과 Rust가 두 측정 출처를 같은 metric-hit 조건으로 읽게 한다.
  characterMatch hit·실제 metric entry 확인과 미등록 출처 실패는 유지한다.
- 문자별 실제 요청/해소 프로그램 이름을 집계 key에 연결한다.
- 읽기 전용 report·classification을 작은 모듈로 나누어 기존 1247줄 파일도
  1000줄 이하로 유지한다. 기존 정렬·분모·자원 한도는 바꾸지 않는다.
- 실제 char-miss인 `Ж`와 실제 fallback-hit인 `😀`를 각각 검사하며, canonical
  자모 출처를 과거 heuristic으로 바꾸어 기대값을 맞추지 않는다.
- 기존 coverage·session 등록·passthrough 검사와 Node 분류 계약을 함께 실행한다.
  새로운 혼합 프로그램 검사는 W2/W3의 이름·entry·수량을 대조한다.

실행 로그와 중간 JSON은 ignored output의 semanticist21-20261019 아래에 둔다.
최종 전체 Rust·lint·통합 검증과 남은 실패 해결 전에는 PR을 제출하지 않는다.


## 실제 보정 결과

Native118은 시작 head `59bf51198d`, 고정 base `f7abae4a8e`의 보정 작업 트리에서
`CARGO_BUILD_JOBS=8`과 공유 `target/pr-review`로 빌드했다. 249.05초, exit 0이며
빌드 전후 소스 해시가 일치했다. `char-overlap-native-build-118-result.json`에
명령·전체 SHA·산출물 해시와 로그를 보존했다. 이번 단계의 renderer·paint 변경은 없다.

| 검증 | 실제 결과 | output 증거 |
| --- | --- | --- |
| coverage 공개 API·보존·자원 제한·혼합 프로그램 | 8 PASS | `current-previous-failures-1822-result.json` |
| passthrough 분류·실재·위임·무효화 원장 | 5 PASS | 같은 영수증의 #2724 검사 |
| 선택적 글꼴 세션 등록과 저장 원본 보존 | 3 PASS | 같은 영수증의 text_font_coverage_registration 검사 |
| 혼합 프로그램 신규 검사의 수정 전후 | 실제 Native117의 unclassified widthSource 오류로 FAIL → Native118 PASS | `font-diagnostic-before-after-1826.json` |
| Node 분류 계약 | 10 PASS, private POC 1 SKIP | `logs/font-diagnostic-contract-node-1816.log` |
| 공개 분류 계약과 역사 W1 확인 | PASS, 역사 후보 1352개 | `logs/font-diagnostic-contract-check-1816.log` |
| workspace fmt | exit 0 | `logs/font-diagnostic-fmt-1827.log` |
| suite prepare/check | 1745 sources, 7584 static test attrs, 48/48 targets | `logs/font-diagnostic-suite-check-1827.log` |
| unit tier | 4101 tests / 296 modules | `logs/font-diagnostic-unit-policy-1827.log` |

신규 검사는 `Ж😀Ж`의 같은 언어 구간에서 실제로 다른 두 metric 프로그램의
요청 face·해소 face·entry·폭 출처·문자 수를 W2와 W3 사이에서 대조한다.
집계 전후 IR·SVG·HWP/HWPX 바이트도 같다. 올바른 metric-hit 출처를 정식 계약에
추가했으며 미등록 출처와 모순된 hit/miss 거절을 유지했다. 쿼리의 분류·집계 보정으로
기존 시각 증적을 새 출력으로 재표시하지 않으며 최종 head의 필수 검증은 별도로 남는다.

기존 46개 실패 중 이번 단계의 3개 실패는 위 검사에서 통과했다. 이 수치는 전체
Rust 재실행의 최종 실패 수가 아니다. 나머지 실패와 최종 통합 검증은 완료 전이다.

## 다음 분석과 구분

기존 #4961 source trace 실패의 공개 API 생성 HWP/HWPX를 각각 독립 MCP job으로
한컴 2020 11.0.0.9136의 PrintMethod 0·1-up PDF로 출력했다. 두 입력은 모두
한 쪽이나 Native117 raw 최저값은 각각 47.68752%와 50.20350%다.
중첩 셀의 텍스트 잘림과 표 높이 차이를 review·overlay에서 확인했다.
따라서 기대값이나 회귀 검사를 갱신하지 않았고 시각 완료로 분류하지 않았다.
`two-tac-source-and-print-observations-1825.json`에 별도 진단을 보존했다.

HWPX 본문의 주소는 `layout_inline_flow_plan`의 원문 `range.start` 발행 경로를
우선 추적한다. legacy inline-table painter의 `+table_idx`를 호출 증거 없이
직접 원인으로 확정하지 않는다. 이 보정은 다음 구현 단계에서 독립 Print와 함께
검증하며, 이번 읽기 전용 집계 보정 커밋에 섞지 않는다. 행정업무편람은 추가
개선 대상에서 계속 제외한다.
