# Task M100 #2193 Stage 3 작업보고서 — Studio input-to-display 프로파일

## 0. 판정 요약

- **Stage 판정**: 완료
- **기준**: `upstream/devel@62bcae43`
- **production 변경**: 없음
- **신규 실행 진입점**: `npm run e2e:issue-2193`
- **측정 범위**: HWP/HWPX × stable 28번째 입력 / 첫 flow boundary 44번째 입력
- **기본 기준선**: case별 fresh document 10회
- **선택 재측정**: HWP stable, HWPX boundary 각 20회
- **정확성**: 115쪽, model/tree/layout/cursor/caret exact와 flush ordering 전부 통과
- **핵심 결과**: boundary input-to-2-rAF 약 1.0~1.1초 중 full pagination이 약 0.9~1.0초
- **다음 단계**: bounded/partial pagination 실행 범위와 안전 조건을 Stage 4에서 결정

## 1. 계측 방식

기존 #2214 E2E의 실제 앱 로드, target 이동, trace wrapper와 구조 assertion을 재사용했다.
`--profile-2193` 모드는 준비 입력을 마친 뒤 target 한 입력만 trace하므로 각 phase가 하나의
operation에 속한다.

1. fresh HWP/HWPX를 `open-document-bytes`로 로드
2. stable은 27자, boundary는 43자를 1글자씩 준비하고 2 rAF 대기
3. trace 설치 후 target 1글자 입력 시작 clock 기록
4. mutation, effect, full flush, exact cursor, invalidation/document-changed와 render 기록
5. 동기 handler 반환과 2 rAF 표시 완료 clock 기록
6. model, layer tree, page text layout, cursor와 DOM caret exact 검증
7. raw event와 nearest-rank p50/p95/max를 ignored JSON에 저장

stable은 page-local invalidation 1회, filtered page render 1회, full flush 0회를 요구한다.
boundary는 mutation → full flush → exact cursor 순서, document-changed 1회, full page render 2회와
flush 1회를 요구한다.

## 2. 실행 환경과 산출물

| 항목 | 값 |
|------|----|
| 기준 commit | `62bcae435370b58373248c284c126c9572098522` + #2193 계측 커밋 |
| Chrome | 150.0.7871.128, headless |
| viewport | 1280×900, DPR 1 |
| Vite | 8.1.4, `127.0.0.1:7714` |
| WASM | 해당 commit에서 `wasm-pack build --target web` 재빌드 |
| percentile | nearest-rank `ceil(count × quantile) - 1` |

- 기본 10회: `output/poc/task2193/stage3/studio-profile.json`
- HWP stable 20회:
  `output/poc/task2193/stage3/hwp-stable-rerun/studio-profile.json`
- HWPX boundary 20회:
  `output/poc/task2193/stage3/hwpx-boundary-rerun/studio-profile.json`

JSON은 환경, fixture SHA-256, raw clocks/events, phase summary와 정확성 snapshot을 포함하는
ignored local evidence다.

## 3. 결과

표의 값은 p50 / p95다. HWP stable과 HWPX boundary는 선택 20회, 나머지는 기본 10회 결과다.

| 형식 / case | operation | mutation | full flush | cursor | page render 합 | input→2 rAF |
|-------------|----------:|---------:|-----------:|-------:|---------------:|-------------:|
| HWP stable | 38.6 / 39.1ms | 0.2 / 0.2ms | 0 / 0ms | 38.0 / 38.5ms | 16.4 / 16.9ms | 73.4 / 75.0ms |
| HWP boundary | 969.6 / 976.9ms | 0.2 / 0.3ms | 915.4 / 923.0ms | 36.7 / 37.4ms | 53.1 / 54.6ms | 1,013.8 / 1,022.0ms |
| HWPX stable | 37.6 / 38.2ms | 0.2 / 0.4ms | 0 / 0ms | 37.1 / 37.7ms | 16.1 / 16.7ms | 66.0 / 67.0ms |
| HWPX boundary | 1,009.0 / 1,023.9ms | 0.2 / 0.3ms | 954.0 / 968.5ms | 37.9 / 38.3ms | 54.3 / 54.9ms | 1,054.9 / 1,071.1ms |

HWP stable 2 rAF는 약 66ms와 74~76ms 두 군으로 관찰됐다. operation/render 시간은 일정하므로
성능 outlier가 아니라 60Hz frame 경계에 따른 한 frame 차이다. HWPX boundary 20회에서는
flush와 input-to-display가 함께 움직였고 p95가 단일 최댓값에 지배되지 않았다.

## 4. 지배 구간 판정

### 4.1 stable 입력

mutation은 약 0.2ms지만 exact cursor query가 약 37~38ms, filtered page render가 약 16~17ms다.
두 작업은 서로 다른 시점에 실행되며 2 rAF frame 대기까지 합쳐 end-to-display는 약 66~75ms다.
full pagination은 0회이고 pending deferred 상태를 유지한다.

### 4.2 첫 flow boundary

mutation 비용은 stable과 동일하다. full pagination p50은 HWP 915.4ms, HWPX 954.0ms이며
input-to-display p50의 약 90%를 차지한다. flush 뒤 exact cursor는 약 37~38ms, full page render
2회의 합은 약 53~54ms다. 따라서 renderer나 mutation 최적화만으로는 약 1초의 boundary latency를
해결할 수 없다.

### 4.3 호출·표시 계약

- stable: deferred mutation → exact cursor → page-local invalidation → filtered render 1회
- boundary: deferred mutation → full pagination 1회 → exact cursor → document-changed → full render 2회
- 두 경로 모두 115쪽, target text length, 4/5 line, cursor offset, overflow false와 DOM caret exact
- 기존 `npm run e2e:issue-2214 -- --runs=1`도 HWP/HWPX focused와 raw 8건 GREEN

## 5. Stage 4 진입 결정

native와 browser가 모두 full pagination을 확정적인 지배 항으로 가리킨다. 다음 단계에서는
production 코드를 바로 수정하지 않고 아래 구현 게이트를 정리한다.

1. changed cell의 continuation cut이 영향을 주는 최초·최종 page 범위를 계산할 수 있는가
2. page 밖의 table continuation, footnote/header/footer와 다음 문단 vpos를 안전하게 보존하는가
3. bounded pagination 뒤 page count와 downstream page offsets를 증분 갱신할 수 있는가
4. 불확실한 경우 기존 full pagination으로 안전하게 fallback할 수 있는가
5. 별도 실행 이슈로 구현·전후 검증 범위를 분리해야 하는가

Stage 4는 위 질문에 코드 근거를 연결하고 실행 이슈 후보를 제안하는 문서 단계다. 사용자 승인
전 production paginator나 공개 WASM API는 변경하지 않는다.
