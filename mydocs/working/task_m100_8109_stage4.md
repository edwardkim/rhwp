# #8109 4단계 — 성능 보정과 PR 제출 검증

- 날짜: 2026-10-21. 현재 미커밋 파일을 포함한 PR 진행 지시에 따라 수행했다.
- 기준 devel: `80968497c2c178eeecb3b2e1d04d086ad27a0dc7`, 보정 전: `f1ae43d70`.
- `pdf_table_inference_feedback_20261017.md`를 원문 그대로 이번 PR에 포함했다.
  그 문서의 표 추론 구현은 이번 범위에 추가하지 않았다.

## Self-review 보완

사용자 self-review 지시 뒤 저장·편집 결과26개의 devel 비교 누락을 확인해 Native26·WASM26·
SVG6의 기준 TSV58개를 추가했다. 고정 devel의 producer·입력·Print·font·DPI 조건을 유지했고
[전쪽80행 비교](../pr/assets/issue_8109/stage4/devel-comparison.tsv)의 최대 하락은0.00000%p다.
입력45개의 HEAD blob·SHA도 일치했다. Rust 집중6개·Studio24개·실제 Chrome155 E2E를
통과했다. 코드와 후보 출력은 같으며 전체 회귀는 정확한 녹색 code head CI를 재사용했다.
검토 head·CI·초기 로딩 실패와 정상 재실행·최종 판정은 [self-review](../pr/archives/pr_8132_review.md)에 기록했다.

## 구현과 수명

`main.ts`의 preflight transform은 `complete=false`이면 원래 report를 즉시 반환한다.
기존 autoPreflightIncomplete 판정·Canvas2D fallback이 유지되고 불필요한 Tree 출력이 사라진다.
정상 준비는 `host-font-requests.ts`에서 unique fontFamily/bold/italic 선언만 수집한다.
같은 report의 transform과 준비는 WeakMap을 통해 이 작은 집합을 공유한다.

`RendererSession.readPreflight`가 문서/편집/resource 결정마다 만드는 새 report에 수명을 맞췄다.
transform이 report를 감싸 반환하는 경우에도 같은 선언 집합을 연결한다. 새 report는 다시 수집한다.
선택된 LocalFontRecord나 전체 Tree를 저장하지 않으며, `loadStoredLocalFonts()` 뒤에 현재 공급
가능한 face를 다시 해소한다. 이 때문에 공급자 해제·재등록과 늦은 local 갱신은 예전 선택을 재사용하지 않는다.
원본 face·AFM·advance·좌표·크기·대체 선언·저장 결과는 변경하지 않았다.

## 실제 수정 전후와 정상 대조군

수정한 공식 `canvaskit-font-preparation-issue7688.test.mjs`를 실제 WASM에서 실행했다.
보정 전 TS4개를 `f1ae43d70`에서 공급한 headless Chrome152에서는 아래2개 검사가 의도대로
FAIL(exit1)했다. 보정 후 실제 Windows Chrome155에서는 기존 검사와 함께 모두 PASS(exit0)했다.

| 제품 콜백에 대한 검사 | 보정 전 | 보정 후 |
| --- | --- | --- |
| incomplete report의 추가 Tree 조회 | 1회·FAIL | 0회·PASS |
| 정상 transform+준비의 합계 조회(1쪽) | 2회·FAIL | 1회·PASS |
| 새 report의 fresh 수집 | 1회·PASS | 1회·PASS |
| 준비 목록 보존·실제 공급 face/style·늦은 local 갱신 | PASS | PASS |
| 실제 리소스 실패의 Canvas2D fallback·문서 보존 | PASS | PASS |

before 로그의 sharedCalls3에는 앞서 검사한 incomplete 조회1회가 포함된다.
초기 동시 CDP 실행에서 모듈 fetch/target 종료가 발생한 시도는 환경 실패로 보존했고 결함 재현으로
세지 않았다. 순차 실행한 최종 after 및 독립 headless before를 위 근거로 사용했다.

같은 Chrome155와 최적화 WASM에서 `samples/kps-ai.hwp`77쪽을 매번 다시 로드했다.
워밍업1쌍을 제외한9쌍을 순서 교차했다. raw 값은
[performance.tsv](../pr/assets/issue_8109/stage4/performance.tsv)에 보존했다.

| 경로 | 보정 전 중앙값 ms | 보정 후 중앙값 ms |
| --- | ---: | ---: |
| auto 파싱·조판 + view 준비 | 1,284.9 | **626.2** |
| 위 view 준비만 | 1,038.4 | 380.7 |
| 위 transform 콜백 | 648.7 | 0.0 |
| 위 추가 Tree 조회 | 77회 | 0회 |
| 직접 Canvas2D 파싱·조판 + view 준비 | 487.7 | 487.5 |

총 시간 차이의 paired bootstrap10,000회·seed8109·95% 구간은 -678.6 ~ -653.1ms다.
첫 화면 paint/FPS·폰트 다운로드를 재는 값은 아니다. 같은 report를 워밍업한 직접 콜백의
77쪽 비용도222.4 → 0ms였다. 원본 성능 검토의 Native/Core·산출물 크기 수치는 그대로 유지하며
성능이 모든 경로에서 정확히0이라고 주장하지 않는다.

## 필수 검증과 증거 재사용 경계

- 새 source에서 TypeScript·Studio 전체 테스트 **2,350 PASS·0 FAIL·2 skip**·배포 build PASS.
  관련 Node 집중20개 PASS, 실제 font 준비 E2E PASS, host provider 양 backend E2E PASS.
- 새 Studio 목록·네 스타일 × CanvasKit/Canvas2D의 실제4쪽을 다시 캡처했다. 전쪽 raw는
  100 / 99.94072 / 99.40245 / 99.70580%, 모두1=1쪽이며4개 PNG는3단계와 바이트가 같다.
  독립 Print와4배 직접 판독해 목록·계속 줄·뒤 본문·네 스타일·간격의 보존을 확인했다.
- `git diff f1ae43d70 -- src vendor tests scripts Cargo.toml Cargo.lock .github`는 비어 있다.
  Rust source와 모든 Rust test·fixture·manifest·workflow가 동일해3단계의 Rust lint·release·
  전체 nextest11,215 PASS·Native Skia3종·doc 검증을 재사용한다. 추가 변경은 Studio5개 파일과 아래 CI 계약 검사1개 및 문서/증적이다.
- Native CLI `cfb7cb5f91201212f786a061a0a50e880635dda0ce454cfb39d66bfce29a551e`,
  최적화 WASM `b8c20fcfe5952ed2a1d3f4188842438275f4a964bceaa55f1f170fc6d640ffe1`,
  JS `1c40e6aa15e5b87643148b717fba0683ae622910e96e369659465d2c0abfab9a`가 이전과 일치한다.
  root pkg/Studio public도 같은 WASM이다. 따라서 그 producer에서 산출한3단계 Native37·
  fresh WASM37·SVG6 전쪽 TSV와 고정 devel 대비0.1%p 하락0쪽 비교를 재사용했다.
  실제 Studio의 TS 소비 경로는 위 새 캡처·실제 face E2E로 별도 재검증했다.
- 새 회귀 입력·렌더링 baseline·허용치·golden을 추가하거나 완화하지 않았다. 원본·독립 Print는
  기존 커밋 경로와 동일한 파일이다. 로그·중간 JSON·실험 스크립트는 ignored stage4 output에 보존한다.

재현 명령과 로그는 `output/pr-review/issue8109/stage4/`의 `open.mjs`, `frontend.mjs`,
`before-vite.mjs`, `open.json`, `frontend.json`, `e2e-before-clean.log`, `e2e-after.log`,
`host-{canvas2d,canvaskit}.log`, `studio-tests.log`, `studio-build.log`, `studio-scores.log`에 있다.
최종 source·producer·캡처 hash 및 대표 PNG는 [4단계 증적](../pr/assets/issue_8109/stage4/README.md)에 연결했다.

필수 기능·성능 보정과 로컬 검증을 완료했다. PR 생성과 exact-head GitHub CI의 실제 결과는
생성 이후 별도 기록한다. merge·이슈 close는 수행하지 않는다.

## PR #8132 첫 CI의 source guard 보정

최초 code head `89cf12827`의 Render Diff run39592776418은 픽셀 비교 전
`renderer-contract.test.mjs:556`에서 실패했다. `prepareLocalFonts(fontRequirements.requiredFontFamilies)`라는
이전 직접 인자 표기를 고정한 검사였다. 원래 로컬 face 준비는 PR #4517 / 실제 merge `04520749b51b`에 포함된 `e55366cd47d`에서 도입됐고,
현재 검사식은 #7688의 실제 통합 PR #7697 / `21ed64071f81`에서 당시 변수명에 맞춰 보정됐다.
원 계약은 첫 replay 전에 저장된 local face를 준비하고 bundled fallback을 이어 준비하는 순서다.
이번에는 stored snapshot 뒤의 `documentFontRequirements(fontRequirements)` 해소와 그 결과를
`prepareLocalFonts(requiredFontFamilies)`에 전달하는 경로까지 검사하도록 갱신했다.
제품 코드는 바꾸지 않았고 실제 물리 face·style·준비 순서의 E2E 증거는 그대로 유지했다.

놓쳤던 Render Diff 사전 계약 검사 경로를 로컬에서 추가 실행했다. 고정 Chromium1660786과
현재 동일 Native/WASM으로 renderer 계약·폰트 coverage·native diff self-test가 PASS했고,
Canvas 비교3/3 PASS, 직접 PDF compatibility gate3/3 PASS, auto CanvasKit readiness8/8 PASS였다.
PDF report-only 비교는4 warning·0 error였으며 Canvas와72DPI PDF의 출력 크기 차이도 보고한다.
이 report-only 값을 독립 Print의 시각 통과 증거로 사용하지 않는다. golden·threshold·workflow는
변경하지 않았다. 로그는 `renderer-contract.log`, `font-coverage.log`, `render-diff-local.log`,
`canvaskit-readiness.log`와 해당 ignored 하위 출력에 보존했다. 보정 commit의 GitHub CI를 다시 확인한다.
