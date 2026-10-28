# #8174 T05 — 입력·Worker·편집 문서 교체 경계 완료

2026-10-28. **D01–D06 완료, 필수 잔여 0건.** [확정 범위](../plans/task_m100_8174_t05_completion.md)를
구현·실제 검증·가이드·보고까지 한 단위로 처리했다. 기준은 T04 완료
`f5098288552a7b8afba6b74dcf1df27ba92a68f1`, 계획 커밋은 `ceb0a27cd4`다.
완료 커밋은 이 문서와 함께 기록하며, 아래 제품·검증 파일을 그 커밋의 source와 대조한다.
T06–T09와 전체 PR 제출 준비는 남는다.

## 생산·소비와 소유권

| 항목 | 실제 연결과 완료 근거 |
| --- | --- |
| D01 옵션·입력 | `snapshotOptions`가 data property·허용 필드·유한한 선택 배열을 먼저 검사한다. 정책 조회 → `reserveInput` → 별도 Uint8Array 생성/복사 → transfer → WASM Vec 예약 순이다. `slice` override/Buffer 별칭과 await 동안 detach된 view도 검사한다. 원본 buffer를 전송하지 않으며, 실제 8 MiB 입력의 선행 예약 거절 시 WASM 선형 메모리는 증가하지 않았다. |
| D02 응답·출력 | `PdfImportSession`이 진단 UTF-8·SVG 성장·WASM→JS UTF-16/binary 사본을 예약한다. Worker는 문자열의 owner 응답 사본도 예약한다. Channel은 schema·UTF-8 길이·정확한 binary backing·절대 deadline을 검사하고 잘못된 응답과 전송 실패의 Worker를 폐기한다. 일반 SVG도 같은 renderer 결과를 소비하며 전체 String clone 하나를 제거했다. 실제 제품의 작은 출력 한도·실패 고착·새 Worker 복구를 통과했다. |
| D03 편집 재열기 | export 뒤 `remainingDecodedBytes` → `getEditorReopenLimits` → `PdfEditorReopen.open` → `PdfImportSession.reopen_hwpx_export` → 예약된 HWPX reader/Core 경로다. 기본 예산을 새로 부여하지 않는다. 입력/ZIP 사본·압축 해제 backing·Core를 예약하고 monotonic deadline을 협력적으로 검사한다. 작은 잔여 예산/만료 거절과 정상 재시도, 실제 그림의 lazy 읽기·편집·양쪽 저장을 통과했다. lazy reader의 초기 예약 lease는 반환 시 끝난다. |
| D04 최종 교체 | 읽기·변환·직렬화·재열기에 같은 deadline을 적용하며 대화상자를 교체까지 유지한다. `publishPreparedPdf`가 후보를 소유하고 준비 뒤 이벤트 루프에 양보해 취소·deadline·generation을 재검사한다. 실패는 후보만 해제한다. 공통 원자적 교체의 rollback 구간을 분리해 교체 후 observer 오류가 게시된 문서를 해제하지 않게 했다. 실제 편집한 기존 문서·비어 있지 않은 저장 draft·recents·핸들 보존을 확인했다. Canvas backend 전환의 실제 빈 화면도 아래 원인에 따라 보정했다. |
| D05 실제 실행 | 현재 Native 제품의 Linux 일반 사용자·사용자 네임스페이스 root 격리, CLI 거절/복구, 최적화 fresh WASM의 실제 Worker·Studio 실패/취소/종료/재생성·편집·양쪽 저장을 실행했다. Worker 17개·267 assertions와 실제 Studio 15쪽 복구를 통과했다. |
| D06 현행 계약 | 공통 Worker README·PDF 자원 정책·보안 제한·Studio 가이드·runtime capability를 실제 경계에 맞췄다. 필수 lint/test·최적화 제품·전쪽 TSV·직접 판독·비용·source/제품 대조를 완료했다. 증명하지 않은 전체 heap/RSS와 동기 선점은 보장으로 표시하지 않는다. |

주요 생산·소비 위치는 `rhwp-shared/pdf/import-session.js`, `import-worker.js`,
`src/wasm_api/pdf_request.rs`, `src/document_core/pdf_import_session.rs`,
`src/parser/hwpx/reader.rs`, `src/renderer/svg_buffer.rs`, `src/document_core/queries/rendering.rs`,
`rhwp-studio/src/core/pdf-open.ts`와 `wasm-bridge.ts`다. 조판의 줄 구성·좌표·높이 규칙과
기존 기대값·baseline·한도는 변경하지 않았다.

## 실행 중 발견한 실패와 보정

첫 실제 WASM 실행에서 새 transport/payload 예약의 오류가 canonical `PDF_RESOURCE_LIMIT`로
전달되지 않아 Worker 폐기·복구 계약 검사가 실패했다. 공통 오류 변환을 보정한 뒤 Native와
최적화 WASM을 다시 빌드하고 아래 최종 전체 검증을 실행했다. 첫 제품의 성공 결과로 대체하지 않았다.

실제 Studio 화면에서는 문서 쪽수는 15쪽인데 CanvasKit 자원 준비 실패 후 본문이 비었다.
Canvas 크기를 0으로 되돌려도 HTML Canvas의 context 종류는 바뀌지 않는데, pool이 기존
WebGL Canvas를 Canvas2D에 다시 빌려준 것이 원인이었다. 일반 생성자로 재열기와 T04 전체
frontend/제품에서도 같은 실패를 재현했다. 선행 이력은 CanvasKit 선택 PR #2394의
통합 `0a4caf020cdb8d1c2ece40d0182a36ed4bcf3aee`와 Canvas pool PR #6637/이슈 #6042의
통합 `ff1ce007b428547da74e0d6b7e9a196592c60ff6`까지 대조했다. 두 통합은 기준의 조상이다.

`CanvasView.applyRendererSelection`에서 backend가 바뀔 때만 사용 중·보관 중 Canvas를 함께
폐기한다. 같은 backend의 revision/zoom 갱신은 기존 pool을 재사용한다. 실제 호출 경로의
양방향 전환·같은 backend 재사용 검사는 수정 전 FAIL → 수정 후 PASS다. 실제 브라우저의
새 재열기·기존 생성자·처음부터 Canvas2D인 세 경로 모두 본문 표시와 console error 0건을
확인했다. 수정 전 흰 종이 픽셀 0개·오류 3건에서 수정 후 종이 719,431개·잉크 42,204개·오류
0건으로 복구했다. 이 수치는 화면 존재 검사이며 독립 기준과의 피델리티 판정은 다음 전쪽 검증이다.

## 최종 실행 결과

공용 `target/pr-review`의 Cargo 작업은 순차 실행했다. Rust 1.93.1, Node 24.15.0,
WSL2 Linux x86_64에서 실행했다. Native 기능 검사는 root opt-level 1/codegen-units 64,
Cargo jobs/test threads 4를 사용했다. 이는 T08의 최종 Native 성능 빌드를 대신하지 않는다.

| 검증 | 최종 결과 |
| --- | --- |
| Native/WASM32 Clippy, workspace build/all-target Clippy, feature-off check, fmt | 모두 PASS. workspace 밖 backend의 해당 feature Clippy·rustfmt도 PASS |
| 공개 요청 수명 검사 | 43 PASS. 추가 4건은 `tests/cases/`의 기존 자동 수집 경로로 실행 |
| 전체 release-test nextest | **11,721 PASS**, 46 skipped. 사용자가 지정한 `page_border_fill_reference_sources_keep_paper_and_page_basis` 제외만 유지 |
| Native Skia | 전체 library **3,891 PASS**, 기존 ignored 12·사용자 제외 1. 관련 workspace library 182 PASS, missing-picture 2 PASS, direct-PDF 14 PASS |
| 자동 suite·unit 범위 | manifest PASS. T05 source unit 증가 0. 기존 누적 34개 정책 이관은 T08에 명시된 기존 잔여로 유지 |
| Studio/공통 Node·TypeScript·production build | **2,473 PASS**, skipped 2·실패 0, TypeScript와 Vite/PWA build PASS. 공통 session 집중 검사 25 PASS |
| 실제 최적화 WASM Worker | **267 assertions / 17 Workers**. 실제 한도·입력 사본·진단·SVG·양쪽 export·잔여 예산 재열기·실패 고착/폐기·정상 재생성 PASS |
| 실제 Studio | 활성 편집 내용·13,824-byte IndexedDB draft/hash·recents·저장 핸들 보존. 입력/재열기 한도 거절, 실제 재열기 뒤 대기 중 취소, deadline·generation 경쟁, observer 오류 소유권 PASS. 정상 15쪽 표시·편집·HWP/HWPX 저장·각각 15쪽 재열기 PASS |
| Native 실제 격리 | 일반 사용자·`unshare -Ur` root에서 Landlock ABI 3/seccomp의 입력·스레드 허용과 상속 fd/무관 읽기·쓰기/network/exec/process/감독자 metadata 변경 차단 PASS |
| CLI 실패·복구 | HWP/HWPX 각각 출력/진단/잔여 예산 실패 6건에서 기존 출력 sentinel·빈 stdout 보존, 다음 정상 출력 bytes 동일 |

전체 nextest 후 manifest 검사에서 테스트 본문/포맷의 weight 변화에 따른 파생 suite 배치 drift를
검출했다. source나 검사 기대값을 바꾸지 않고 공식 prepare로 갱신했다. 1,739개 source path의
multiset과 모든 leaf source hash가 동일함을 확인하고, 실제 배치가 바뀐 4개 suite의 **1,031개를
재실행해 전부 통과**했다. 최종 workspace Clippy·manifest·fmt도 통과했다. 파생 suite는 커밋하지 않는다.

메모리 오류·engine trap 검증 중 일부는 실제 Worker 안에서 통제한 주입이다. 작은 WASM의
`unreachable`로 trap 경계를 실행했으며 제품 자체의 특정 panic 재현으로 부르지 않는다.
실제 제품의 예산 초과/출력 한도/만료 거절은 별도로 실행했다.

## 전쪽 출력과 직접 판독

같은 원본·기존 독립 Print PDF·올바른 기존 글꼴 공급으로 현재 Native와 최적화 fresh WASM을
재출력했다. TSV는 쪽수·누락·모든 페이지의 raw 일치율을 먼저 확인했으며 측정 불가·90% 미만은 0쪽이다.

| 범위 | Native | fresh WASM |
| --- | --- | --- |
| 기존 PDF 17개·49쪽 | 최저 **98.34664%** | 최저 **98.79246%** |
| 일반 HWPX inline 대조군 4개·38쪽 | 최저 **99.66991%** | 최저 **99.17555%** |
| 생성 HWPX handoff 3개·28쪽 추가 대조 | 최저 **98.34664%** | 최저 **98.79246%** |
| Native fallback 대조군 5쪽 | **100%** | 해당 없음 |

주 검증은 각 runtime **21개·87쪽**이다. 전쪽 PNG 174개가 T04와 bytes 동일하고 Native 저장
37개도 bytes 동일하다. handoff 28쪽은 새 `PdfEditorReopen` 결과의 모든 SVG가 일반 생성자 결과와
동일하며 before/candidate의 HWPX 3개도 bytes 동일하다. 이를 쪽수/텍스트만의 확인과 구분했다.

현재 source/product의 CFF, Type3 최저 페이지와 11쪽 수식·미세 간격, 중첩/여러 줄 셀,
inline 개체/빈 줄 경계·명시적 개행, distribution을 review/overlay와 확대 crop으로 직접
판독했다. 표 외곽·셀 포함·다음 내용·줄바꿈과 수식 기준선/첨자/기호 간격의 새 결함은 없었다.
기존 Type3의 weight/가는 획과 distribution 글리프 차이는 T04와 동일하게 남으며 새 글꼴 예외나
허용치 완화를 추가하지 않았다. 실제 Studio 최종 화면의 한글 본문·쪽 상태·종이도 직접 확인했다.

## 비용과 보장 범위

Cargo/시각 작업이 끝난 후 같은 브라우저에서 before/candidate 실행 순서를 번갈아 4회 측정했다.
첫 회는 cold로 구분하고 warm 3회의 **각 단계 중앙값을 합산**했다. 표의 시간은 단일 호스트의
변환·첫 저장·재열기이며 일반 페이지 편집이나 전체 제품 latency 보장이 아니다.

| 입력 | before import/export/reopen ms | 후보 import/export/reopen ms | 중앙값 합계 변화 |
| --- | --- | --- | --- |
| CFF 1쪽 | 3.0 / 1.2 / 1.0 | 2.8 / 1.2 / 1.0 | −0.2 ms |
| Type3 12쪽 | 315.2 / 98.5 / 74.9 | 316.4 / 99.4 / 86.3 | +13.5 ms, **+2.76%** |
| 비공개 대조군 15쪽 | 1,620.7 / 940.9 / 878.3 | 1,744.5 / 928.5 / 879.1 | +112.2 ms, **+3.26%** |

실제 남은 예산과 시간 검사를 추가한 비용을 기록하며 무비용 또는 속도 개선으로 보고하지 않는다.
최적화 WASM은 **25,496,895 → 25,598,495 bytes**, +101,600 bytes(약 0.40%)다.
이 측정에서 큰 문서 뒤 retained 선형 메모리의 최대값은 before 705,757,184, 후보 704,512,000 bytes다.
이는 module이 다음 입력에서도 보유한 선형 메모리 capacity이며 문서별 live heap·RSS가 아니다.

논리 decoded 작업량은 CFF import/export 1,976,316 + 재열기 129,008 bytes,
Type3 171,076,530 + 25,122,221 bytes, 15쪽 대조군 1,237,852,025 + 276,587,078 bytes다.
실제 소유 payload·사본·교체 backing의 누적량으로 현재 메모리 점유와 다르다.
JSON.parse/legacy XML 객체·ZIP 내부·renderer scratch·브라우저 전체 heap/RSS의 상한 증명은
주장하지 않고 `parserAllocationBoundsProven=false`를 유지한다.

Worker는 owner가 종료해 실행을 중단한다. main thread 동기 WASM은 같은 스레드 timer로
선점하지 못한다. 예약/읽기 경계와 전후의 협력 시간 검사 및 최종 교체 전 이벤트 루프 양보로
취소·deadline·generation을 확인한다. 성공 후 일반 편집·lazy 자원·저장은 초기 요청 credit에서 분리된다.

## 증적과 다음 작업

ignored 증적 루트는 `output/pr-review/issue8174/t05-completion-20261028/`다.

- `production-freeze.json`, `native-producer.json`, `candidate-wasm-producer.json`: source hash·빌드 제품.
  최적화 WASM SHA-256은 `e9e247a58f7d03edcacb998311035b0ca1098de62471079765609a925cf87015`이며
  pkg·Studio public·dist·실제 브라우저 HTTP 응답과 일치한다.
- `build-gates.json`, `backend-checks.json`, `runtime-checks.json`: 정확한 명령·exit status·시간.
  `suite-repack-proof.json`과 `test-leaf-freeze.json`: 전체 실행과 최종 파생 배치의 source 동일성.
- `browser-final/browser-result.json`, `studio-t05.json`, `ui-probe.json`, `ui-baseline.json`:
  실제 실행·응답 제품 hash·기존 상태/복구·화면 보정 전후. 이전 browser/attempt 결과와 구분한다.
- `visual/candidate/{native,wasm}/`, `visual/inline/candidate/`, `visual/handoff/`:
  전쪽 TSV·현재 PNG. `full-page-reviews/`, `inline-reviews/`, `direct-review.json`: 현재 비교·overlay·확대 판독.
- `cost.json`, `cost-summary.json`, `quiet-cost-result.json`: 마지막 단독 측정.
  동시 작업 중 측정은 `cost-concurrent.json`으로 보존하고 최종 비용 근거에서 제외했다.
- `completion-checks.json`, `final-source-files.json`, `evidence-manifest.json`, `validated-commit.json`:
  최종 게이트·source·증적 hash와 완료 커밋 연결. 실패한 이전 시도도 이력으로 보존한다.

비공개 문서는 로컬 검증에만 사용하고 공개 기록에는 집계와 비식별 결과만 남긴다.
다음 순서는 [TODO](task_m100_8174_pr_remaining_todo.md)의 **T06 호환/의존성 보정**이다.
T05 필수 항목을 T06–T09로 넘기지 않았다. 기존 T08의 누적 unit 정책 이관·최종 통합 성능과
T09의 최신 devel 기능 보존/PR 제출 게이트는 그대로 남는다. 원격 push·PR 생성·registry 게시·
토큰 읽기·태그는 수행하지 않았다.
