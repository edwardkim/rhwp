# #8174 v2-S52 — 저장 줄·셀 frame reflow의 요청 예산 연결

Issue: #8174. 2026-10-27, 기준 `e92caaf39a`.

저장 줄의 수용 검증과 셀 frame reflow가 S51의 측정·중첩 composition과 같은 요청 Storage를
소비하게 했다. **공통 조판 코드를 변경했지만 줄 나눔·좌표·높이·캐시 수용 규칙은 유지한다.**
기존 공개 함수는 같은 구현의 Unbounded wrapper로 유지한다. 기존 통합 진행 승인 범위다.

## 생산·소비와 실패 처리

| 경계 | 연결한 실제 경로 |
| --- | --- |
| 호출 | 높이 측정의 본문 frame과 중첩 표/가로 셀 frame이 같은 Storage를 전달한다. [본문](../../src/renderer/height_measurer.rs#L2204), [중첩 셀](../../src/renderer/composer/nested_table_frames.rs#L101) |
| 저장 줄 검증 | [resolver](../../src/renderer/composer/line_breaking.rs#L4692)가 checkpoint·stored-row admission과 기존 fallible fill을 소비한다. NO_LS·부분 수식 재구성·stale/cache 분기, HWP3 원점·물리 frame/외부 어울림 소유권은 유지한다. |
| 실패 복구 | 유한 Storage의 resolver는 진입 checkpoint를 먼저 예약한다. 오류면 소유한 checkpoint를 이동해 추가 할당 없이 복원한다. 일반 cache/prefix 사양도 owned restore를 쓰며, 예산 실패를 None/false로 버리지 않는다. |
| 재구성·projection | [공통 frame owner](../../src/renderer/composer/stored_cell_reflow.inc.rs#L50)가 문단/결과 복사·LineSeg/space metric projection·prefix 보존·실측 인라인 높이를 예약한다. 기존 scoped composer를 사용하며 저장 축 보정과 기존 조판 규칙은 유지한다. |
| 문맥 payload | [context run 보존](../../src/renderer/composer/stored_cell_reflow.inc.rs#L487)의 span/절단점·run 조각·문자열은 같은 Storage를 쓴다. display/각주/글자 겹침 소속의 기존 조건과 순서는 유지한다. |
| 재시도·진단 | stored-tail trial frame의 복사와 projection, 선택적인 stored-row mismatch 진단의 임시 문자열도 같은 요청을 소비한다. 원본 문단·기존 composed 결과를 바꾸지 않고 파생 결과를 만든다. |

한 값의 연결은 `현재 문단/셀 상자 → 같은 frame의 stored-row admission 또는 기존 fill →
LineSeg·space metric projection → 기존 scoped composition → HeightMeasurer의 문단/행 높이 →
기존 TypesetEngine/Paginator의 fit·배치`다. 새 앵커·행 높이·분할 컷·배치 조건을 만들지 않았다.
새 렌더링 fixture/golden이나 좌표 핀·쪽수 기대값은 추가·완화하지 않았다.

## Native 실행과 전후 대조

ignored `output/pr-review/issue8174/v2-s52-20261027/`에 명령·제품/source hash·원시 결과를
보존했다. `production-freeze.json`의 src 아래 Rust 파일 1,510개를 빌드·검증 동안 고정했다.
Native는 root opt-level=1 / codegen-units=64의 기능용 제품이며 최종 성능 제품이 아니다.

| 항목 | 결과 |
| --- | --- |
| 새 실제 PDF 예산 경계 | 같은 test source·cap으로 **S51 2 FAIL / 후보 2 PASS**, 실패 유지·새 요청 복구 |
| 공개 요청·저장·wire 계약 | **33 PASS**, 기존 자동 배정 원본을 Native 최종 library에 직접 연결 |
| Core·높이 측정·typeset·pagination | **624 PASS / 기존 8 ignored**, sample16을 읽는 검사는 사용자 제외 유지 |
| composer·float placement·table/paragraph model | **244 PASS** |
| serializer/adapter·Allocation | **662 + 9 = 671 PASS**. Core 필터와 일부 겹치므로 고유 개수로 합산하지 않음 |
| 실제 HWP/HWPX 11개 | header 11개·정상 ZIP 10개·기존 양식 거절 1개의 **22개 파일 bytes 동일** |
| PDF 저장·재열기 | 17개·49쪽, **34개 저장/98쪽 재열기·상세 3개·선택 쪽 추가 4개** |
| 저장 bytes | 일반/상세 파일 **37개가 S51과 동일** |
| CLI 실패·복구 | 출력/진단/잔여 예산 × HWP/HWPX **6 PASS**, 기존 파일/stdout 보존·정상 후속 요청 |
| 정적·수집 검사 | Native/WASM Clippy `-D warnings`, feature-off·fmt 통과, suite 배정 계약 **23 PASS** |
| 절편 source unit 정책 | HEAD 대비 **4,134 tests / 304 modules**, source-side 증가 없음 |

기존 양식 거절은 `hwpers_test4_complex_table.hwp`의
`FORM_ACTION_INVALID: invalid or excessive form action`이다. 정상 ZIP 11개로 세지 않았다.

S51의 실제 Native 초기 decoded 실측값 + 64 bytes를 cap으로 지정했다. S51은 그 cap으로
Core를 반환해 새 검사에서 실패하고, 후보는 같은 요청의 pagination 단계에서 거절한다.
요청 실패 후 재사용 거절·카운터 한도 준수·새 기본 요청의 동일 페이지 수/HWPX bytes도 확인했다.
이 경계는 64-bit Native 실측 cap을 쓰므로 WASM의 같은 수치 경계 검사라고 표시하지 않는다.

| 입력 | S51 초기 decoded | 후보 초기 decoded | 새 cap |
| --- | ---: | ---: | ---: |
| CFF 대조군 | 1,626,516 | 1,626,516 | 해당 없음 |
| cell-nested | 1,820,188 | 1,823,204 | 1,820,252 |
| multiline-cell | 7,606,256 | 7,606,720 | 7,606,320 |
| paragraph-origins 대조군 | 7,640,385 | 7,640,617 | 해당 없음 |

`core-budget-before.tsv`·`core-budget-after.tsv`·`core-budget-producer.json`,
`run-budget-before-producer.json`과 공개 계약 로그에 library/source hash·명령을 연결했다.
이 수치는 논리적 payload 작업량이다. heap/RSS·실행 시간의 증가율이나 절대 상한으로 해석하지
않으며 제품 한도·`parserAllocationBoundsProven=false`는 유지한다.

## 현재 최적화 WASM·실제 브라우저

`scripts/wasm-pack-locked.sh --target web --out-dir pkg`를 Rust 1.93.1·locked·release
codegen-units=16으로 실행해 wasm-opt까지 완료했다. 이전 pkg/Studio 제품은 ignored 증적에
보존하고 새 root pkg의 JS/WASM과 Studio public의 실행 bytes를 일치시켰다. immutable
`candidate-wasm/`을 별도로 보존했으며 **현재 제품의 검증이다. S37 제품의 재사용이 아니다.**

- 최적화 WASM: **25,073,284 bytes**, SHA-256
  `bf16a590e5c51020816532de04dd36f38306d131c0f4f76110c866b72373946d`.
- 실제 Chrome Worker 검사: **249 assertions**, 일반 11개·메모리 실패 3개의 **14 Workers**.
  실제 12쪽 PDF 전쪽 렌더·양쪽 형식 저장/재열기, 출력 한도·codec 작업 한도·잘못된 입력·
  취소·timeout·실제 Memory.grow 실패 뒤 활성 문서 보존과 새 Worker/정상 요청 복구를 통과했다.
- Studio UI: 기존 HWP 문서가 열린 상태에서 operator 실패 3개·취소 1개 뒤 generation·
  파일명·쪽수·draft·저장 draft·handle·recents를 보존했다. 이후 실제 사용자 PDF 15쪽의
  가져오기·표시·HWP/HWPX 저장을 통과했고 pageerror는 0개였다.
- Worker에서 제공한 제품과 Studio에서 실제 로드한 WASM의 해시가 위 제품과 같다.
  `browser/browser-result.json`, `t02-studio.json`, `candidate-wasm-producer.json`,
  `wasm-checks.json`에 결과·명령을 연결했다.

S37 대비 크기는 **897,804 bytes / 3.71370% 증가**했다. S38–S52의 누적 차이이며 S52만의
비용이나 실행 시간/RSS 증가율이 아니다. JS glue는 S37과 bytes 동일하고 d.ts는 InitOutput
member 순서만 다르며 나머지 선언 bytes·member/서명은 같다. 최종 성능 확정은 T08에 남는다.
Worker 검사의 `visualJudgment` 고정 문구는 자동 API 검사 자체가 시각 판정이 아님을 나타내며,
아래 실제 전쪽 Visual Sweep과 직접 판독 결과를 별도로 사용한다.

## 현재 제품의 전쪽 시각 확인

원본 PDF 자체를 독립 기준으로 Native/fresh WASM 각각 **17개·49쪽**을 전쪽 비교했다.
2px raw 실루엣 TSV 원본은 `visual/candidate/{native,wasm}/<문서>/silhouette.tsv`에 있다.
`candidate-visual-summary.json`·`visual-evidence-audit.json`에 각 TSV의 hash·쪽수·최저값과
제품 출처를 연결했다. **90% 미만·측정 불가·누락 쪽·쪽수 불일치는 모두 0개**다.

| 범위 | Native 최저 raw | fresh WASM 최저 raw |
| --- | ---: | ---: |
| 전체 17개·49쪽 | **98.34664%** | **98.79246%** |
| CFF·11개 본문/셀 PDF·이미지 clip/batch | 100.00000% | 100.00000% |
| 텍스트 clip 3쪽 | 99.88074% | 100.00000% |
| Type3 12쪽 | 98.34664% | 98.79246% |
| Type3 대표 11쪽 | 99.97900% | 99.98717% |
| 실제 사용자 PDF 15쪽 | 99.13508% | 99.11167% |

각 runtime의 CFF 1쪽·Type3 11쪽/최저값인 3쪽·cell-nested/multiline-cell 1쪽에 새
compare·standalone overlay·review PNG를 만들었다. 열 쪽의 review와 양쪽 Type3 11쪽의
standalone overlay를 직접 열어 판독했다. CFF 기호·ligature·간격, Type3 26–30번 행의
괘선·그림 9/10·캡션·앞뒤 본문과 3쪽의 그림/표 경계, 중첩 셀의 앞뒤 내용·A/B/C/D 순서,
여러 줄 셀의 줄 소속·포함을 대조했다. 새 누락·겹침을 관측하지 않았으며 그림 세선과 문자
획의 래스터 차이는 남는다. Native 선행 두 쪽의 새 PNG 6개는 S51과 bytes도 동일하다.
`full-page-reviews/`와 `direct-review.json`에 경로·직접 판독 내용·hash를 남겼다.

이것은 현재 source의 실행·시각 확인이다. silhouette-only의 자동 상태는 `not_evaluated`이며
PR 승인으로 바꾸지 않았다. **최신 upstream/devel과 후보의 전쪽 Native/fresh WASM 대조 및
2%p 후퇴 판정은 T09에 남는다.** 현재 source의 통과를 이후 코드 변경 뒤의 결과로 사용하지 않는다.

## 남은 통합 범위

- T04: frame fill이 소비하는 일부 내부 producer(인라인 projection/control·stored-row stale/word-tail
  probe·leader·overflow/shaping cache), 일부 object-only/wrap 측정 helper. 이번에는 기존
  fallible frame owner를 실제로 연결했으며 그 내부의 모든 할당을 입증한 것은 아니다.
- T04: 정규화·TypesetEngine/Paginator 내부 state·page-field 재분할과 table-frame 저장의
  cell-unit/index cache·content-loss 내부 수집.
- T05의 WASM/Studio 잔여 복사·전송·동기 재열기, T06의 JPEG/의존성, T07의 최종 포장/소비,
  T08의 전체 회귀·성능·이전 source unit 이관과 T09의 최신 base/기능 보존·PR 준비.

현재 최적화 WASM·Studio 실행과 전쪽 시각 확인은 이번에 완료했다. T04 전체 완료·전체
할당 증명·최종 PR 제출 게이트 완료로 표시하지 않는다. 문서 메타데이터 검사는 기존 오류
20개가 같고 새 오류는 없으며, 기존 권위 문서는 변경하지 않았다. 원격 push·PR 생성·registry
업로드나 토큰 열람은 수행하지 않았다.
