# PR #8196 self-review — Undo 선택 배치와 CanvasKit 전환

## 최종 판정

**승인 — 2026-10-29 리베이스·보정 head를 재검토했으며 추가 결함을 발견하지 않았다.**

검토 code SHA는 `9d4a974db0d31ed302df4f2da0f513fd6c9b9577`, 통합 base는
`6a3cb5a9c194366169a9bb11ee75b6addfa2e8c4`다. 사용자의 재리뷰 요청에 따른 collaborator
본인 PR의 self-review이며 GitHub self-approve·reviewer 지정·병합은 수행하지 않았다.
문서-only trailing head의 실제 CI 확인과 사용자 병합 승인은 별도 조건이다.

### 재검토 경로와 범위

기본 경로는 `collaborator_self_merge`, 보조 경로는 `intake_and_review`, `local_validation`,
`visual_fixture_evidence`, `review_template`, `rework_and_exceptions`, `review_only_fast_pass`다.
PR 작성자는 postmelee, 대상은 devel, #8103은 참조이며 붙여넣기 성능은 #8197에서 추적한다.
19개 파일·1,760줄 추가·48줄 삭제를 제품 코드, 회귀 검사, 검증 기록으로 나눠 검토했다.

최신 upstream/devel `784ef465d8`까지 merge simulation은 충돌 없이 통과했다.
새 base의 #8372는 Native host 패키징이며 이번 Studio 변경과 별개다. 이를 source에 다시
리베이스하지 않았으며 해당 Native 패키지 기능을 이 후보에서 실행했다고 주장하지 않는다.

### 코드와 실행 증거 대조

| 검토 항목 | 근거와 판정 |
| --- | --- |
| 이전 보류 해소 | 현재 준비 실패는 공통 prepareRendererRevision에서 portable metrics·Canvas2D로 복구한다. stale·dispose 실패는 최신 pending을 변경하지 않는다. 충족 |
| Undo 선택과 줌 | 현재 anchor/focus를 확정 page origin에 투영하고 reveal 예약만 별도로 소비한다. 실제 Backspace·Undo·Redo, 역선택·선택 해제·문서 교체를 확인했다. 충족 |
| 충돌 해소 | #8360의 backendChanged → releaseAllRenderedPages → discardAll 순서를 유지했다. 같은 backend는 context를 폐기하지 않는다. 충족 |
| 로컬 필수 검증 | 같은 code tree의 unit 2,675 PASS, 실제 WASM 타입 production build, package 계약 3 PASS, fresh WASM 최적화 완료. 충족 |
| 재리뷰 추가 실행 | 실제 CanvasView 경계·surface cache·zoom scheduler 64 PASS, Chrome auto WebGL 선택 E2E 160 PASS. pageerror·caught render error 0. 충족 |
| 리베이스 기능 보존 | 아래 구현 보고서의 base/후보 Undo 99·전체 선택 63·Enter 84·줌 37·모달 4·provider 34·PDF Studio 14 PASS 및 Worker 271개 단언을 대조했다. 충족 |
| 일반성과 소비 경로 | 문서별 분기·clamp·golden 완화 없이 공통 준비 수명과 DOM 선택 투영을 수정했다. Canvas2D·CanvasKit·auto의 실제 실행 증거를 분리했다. 충족 |
| 줄 구성·분할·이어받기·Print 기하 | Rust/model/serializer/paint 기하를 변경하지 않는다. 이 PR의 DOM overlay·context 수명 계약에는 비해당 |
| 실물 fixture·기준값·전쪽 Print TSV | 실물 조판 회귀나 인쇄 기대값을 추가·변경하지 않았다. 합성 편집 동작과 화면 선택 계약이며 Native/한컴 fidelity 판정은 비해당 |
| 주장과 증거 | 실패 주입을 자연 발생 자원 오류로 표시하지 않는다. 실제 package 타입 검증 성공과 base의 테스트 전용 CI stub API 누락 실패를 구분했다. 충족 |

[충돌 해소·기능 보존 보고서](../../report/task_m100_8103_report.md)의 2026-10-29 절에
입력·실행 명령·이전 base/head·반영 PR·WASM 해시·제약을 기록했다. 그 code tree는 현재 SHA와 같으며
전체 2,675개 검사·633개 선택 E2E를 불필요하게 반복하지 않았다. 이번 추가 실행은
`node --test tests/selection-layout-refresh.test.ts tests/canvas-view-surface-cache.test.ts tests/canvas-view-zoom-scheduler.test.ts`와
`node e2e/selection-layout-issue8103.test.mjs --mode=headless --renderer=auto --surface=webgl`이다.
Chrome·7700·SwiftShader 환경은 구현 보고서와 같고 로그는 ignored
`output/pr-review/8196/rebase-20261029/review-focused.log`, `review-webgl.log`에 남겼다.

### 정확한 code head CI

- [CI 40870269748](https://github.com/edwardkim/rhwp/actions/runs/40870269748): success
- [Render Diff·CanvasKit 40870269581](https://github.com/edwardkim/rhwp/actions/runs/40870269581): success
- [CodeQL 40870269648](https://github.com/edwardkim/rhwp/actions/runs/40870269648): success
- [Adapter inter-diff 40870269749](https://github.com/edwardkim/rhwp/actions/runs/40870269749): success
- [Proptest 40870269729](https://github.com/edwardkim/rhwp/actions/runs/40870269729): success
- CI Impact Policy: success. 영향 정책의 skipped job·GHAS neutral을 실제 실행 통과로 세지 않는다.

### 시각 판독과 남은 범위

[66% 마지막 쪽 PNG](../assets/issue_8103/auto-66-last-page.png)를 다시 열어 P369~P399와 선택 음영의
일치를 확인했다. 이 파일은 직전 보정에서 만든 동일 code tree 증적이며 이번 재리뷰의 새 캡처로
표시하지 않는다. 새 브라우저 실행에서도 400문단·10쪽·400개 음영, 재선택 없는 실제 삭제 범위와
스크롤·줌 중 투영을 확인했다. 한컴 Print 일치율이나 수식·실물 조판 개선을 주장하지 않는다.

### Merge 후 contributor PR comment 계획

사용자가 병합을 승인하고 실제 merge가 끝난 뒤에만 merge SHA·CI·보정 결과·#8197 분리를
한국어 존댓말로 게시한다. [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을
연결하되 본 검토는 DOM 선택 판독임을 명시하며 존재하지 않는 TSV 수치를 쓰지 않는다.
이미지는 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_8103/auto-66-last-page.png`를
사용한다. asset의 devel 반영 확인 → body-file 게시 → API 재조회 순서를 따른다.

## 이전 판정 이력 — 2026-10-26

**승인 — #8103 범위의 P2 보정과 로컬 재검증, 정확한 code head CI를 완료했다.**

2026-10-26 collaborator 본인 PR 검토다. 검증 code head는 `1a5cea08307132d3deba18a1a01b85869717b726`다.
원 head `5ab102cc6b55af8c3a6776e217d3840fe74dffb4`는 준비 실패 회귀로 보류했고,
사용자의 보정 승인 뒤 공통 실패 복구를 구현했다. GitHub self-approve·reviewer 지정·merge는 하지 않았다.
문서 trailing head의 실제 CI/fast-pass 확인과 사용자 병합 승인은 별도 조건이다.

## 접수와 검토 범위

- [PR #8196](https://github.com/edwardkim/rhwp/pull/8196), 작성자 `postmelee`, base `devel`.
- 원 base `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`, 검토 중 최신 base
  `49bc935a149c20b942d3045c2f5c733aeccf57d0`. 문서 때문에 rebase/merge하지 않았다.
- [#8103](https://github.com/edwardkim/rhwp/issues/8103)은 참조다. 400문단 붙여넣기 성능은
  [#8197](https://github.com/edwardkim/rhwp/issues/8197)로 측정·재현 절차를 분리했다.
- collaborator_self_merge를 기본으로 intake_and_review, local_validation, review_template,
  visual_fixture_evidence, rework_and_exceptions, review_only_fast_pass를 적용했다.
  외부 contributor 중개 절차는 적용하지 않았다. 1,000줄 초과 범위는 별도 검토·실행·판정 cycle로 확인했다.
- 원 해결 PR·통합 보정 SHA와 음성 대조는 [구현 보고서](../../report/task_m100_8103_report.md),
  최종 코드·증거 연결은 [구현 검토](pr_8196_review_impl.md)에 있다.

## 보정 후 검증

| 검사 | 결과 |
| --- | --- |
| Studio/editor unit | 2,447 PASS / 0 FAIL / 0 SKIP |
| production build / CI unit TypeScript | exit 0 |
| 선택 E2E Canvas2D + auto software | 314 PASS |
| 선택 E2E 강제 CanvasKit | 159 PASS |
| 선택 E2E auto WebGL SwiftShader | 160 PASS |
| 기존 Undo / 전체 선택 / Enter / 줌 대화상자 | 99 / 63 / 84 / 37 PASS |
| renderer-contract / command-error-contract | exit 0 |
| E2E manifest | 169개 / 169행, 이상 없음 |
| 현재 실패 / stale pending / stale 완료 / 문서 교체 / dispose | 현재 실패만 복구, 최신 준비·선택 유지 |

선택 E2E 세 실행 모두 준비 실패를 주입한 뒤 본문·논리 선택·음영 보존과 실제 Canvas2D repaint를
확인했다. pageerror와 caught render error는 0이다. 예상 준비 실패는 진단 warning으로 기록된다.
정상 zoom/scroll/Undo/Redo·역순 선택·quiet 재선택·문서 교체도 통과했다.
검증한 제품·테스트 트리를 위 code SHA로 커밋했으며 이후 코드 변경은 없다.

명령은 `npm --prefix rhwp-studio test`, `npm --prefix rhwp-studio run build`, Studio에서
`npx tsc --noEmit -p tsconfig.ci-unit.json`, `node e2e/selection-layout-issue8103.test.mjs --mode=headless`다.
브라우저 환경은 `CHROME_PATH=/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`,
`VITE_URL=http://127.0.0.1:7700`이며 강제 CanvasKit은 `--renderer=canvaskit`, WebGL은
`CHROME_EXTRA_ARGS=--use-angle=swiftshader --enable-unsafe-swiftshader`와 `--renderer=auto --surface=webgl`이다.
ignored `output/pr-review/8196/fix-*.log`, `final-*.log`에 결과를 보존했다.

로컬 WASM은 같은 Rust 입력으로 10월 24일 fresh 생성한 산출물을 재사용했다. 오늘 새 로컬 빌드로
표시하지 않는다. public/pkg SHA-256은 모두
`27a0fee1ad2875cfe8423ee43667b2411c9f040a3b4c8c43d92bbbca114fa5f1`이다.
새 code head CI에서는 fresh WASM 빌드와 프런트엔드 gate, Render Diff를 별도로 완료했다.

## Code head GitHub CI

다음 여섯 workflow 모두 `1a5cea08307132d3deba18a1a01b85869717b726`의 완료·성공을 확인한 뒤 이 문서를 작성했다.

- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/40318570668): completed / success
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/40318570620): completed / success
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/40318570641): completed / success
- [CI](https://github.com/edwardkim/rhwp/actions/runs/40318570567): completed / success
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/40318570442): completed / success
- [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/40318565714): completed / success

영향 범위상 skipped job이나 GHAS neutral을 실제 검사 실행으로 세지 않는다.
code head의 성공을 아직 실행 전인 trailing head의 성공으로 간주하지 않는다.

## 조판 원칙·시각 증거

현재 선택·확정 page origin·zoom의 관계를 사용하며 문서 ID 분기, 임의 clamp, golden 완화는 없다.
공통 renderer 준비 수명과 DOM overlay 투영을 보정하며 Rust 줄 구성·페이지 분할·paint 기하는 변경하지 않는다.
HWP/HWPX/PDF fixture나 인쇄 기대값을 추가하지 않았다. 합성 400문단의 본문·소유·순서·삭제 범위를 검사한다.
Native/한컴 Print TSV의 fidelity 판정은 이 DOM 선택/context 수명 계약에 비해당이다.
CI Render Diff 성공을 독립 한컴 전체 출력 일치로 승격하지 않는다.

새 code tree의 WebGL 66% 마지막 쪽 캡처를 직접 열어 P369~P399 본문과 선택 음영의 일치를 확인했다.
[제출 PNG](../assets/issue_8103/auto-66-last-page.png)를 갱신하고 PR 본문은 code SHA 고정 이미지로 표시했다.
수식·실물 px/조판 개선을 주장하지 않는다. 최신 base와의 merge simulation은 충돌 검증이며
새 base 전체 렌더 기능을 source에 통합해 재검증했다는 의미가 아니다.

## 원 head의 P2 재현과 해소 이력

아래는 보정 전 head의 결함 기록이다. 보정 후에는 공통 `prepareRendererRevision`이 현재 epoch 실패만
portable metrics·Canvas2D로 복구한다. 아래 해제 조건을 단위·브라우저 검사로 충족했다.

### [P2] 준비 Promise 실패 뒤 현재 revision의 렌더 대기를 복구해야 한다

`rhwp-studio/src/view/canvas-view.ts:502-509`에서 `rendererSelectionPending = true`를 설정한 뒤
renderer resolve·metrics 준비를 await한다. 성공한 `applyRendererSelection`만 false로 돌린다.
실패 경로에는 상태 복구나 bounded retry가 없다. 같은 문제는 auto mutation 경로의 457-467행에도 있다.

`prepareCanvasMetrics()`가 reject되면 이후 `updateVisiblePages`·`renderPage`·`renderCanvas`는
`deferRenderUntilRendererReady()`에서 계속 반환한다. 단순 scroll/zoom으로는 revision 선택을
다시 시작하지 않으므로, 새 편집·문서 로드 등 다음 revision이 준비될 때까지 페이지 다시 그리기가 막힌다.
기존 예외 전파 문제에 이번 PR이 지속적인 repaint 차단 상태를 추가한다.

준비 API 전체가 no-throw 계약인 것은 아니다. `WasmBridge.prepareCanvasMetrics`는
metafile 준비·host font catalog·원본 PUA 요청 등 CanvasMetricSession의 portable fallback catch
바깥 호출도 await한다. 다만 이번 검토에서는 실제 특정 사용자 파일의 자원 오류를 자연 재현하지 않았고,
**실패 주입으로 상태 머신의 회귀를 재현**했다. 자원 오류의 발생 빈도나 모든 글꼴 실패가 reject된다고 주장하지 않는다.

검증 방법:

1. 실제 CanvasView를 Vite SSR로 로드하고 `prepareCanvasMetrics`만 reject시키는 진단을 실행했다.
2. base 소스는 Vite pre-transform으로 공급했다. 실패 후 scroll 경로는 base에서 viewport 처리에
   도달했으나 head에서는 pending/deferred만 true가 되고 viewport 처리에 도달하지 못했다.
3. 실제 Chrome·WASM 새 문서에서도 같은 준비 실패를 한 번 주입하고 원래 메서드를 복원했다.
   scroll 재호출 뒤 pending/deferred는 true였다. 새 revision을 명시적으로 실행한 뒤에만 pending이 false가 됐다.

```jsonl
{"version":"base","pending":false,"deferred":false,"reachesViewport":1}
{"version":"head","pending":true,"deferred":true,"reachesViewport":0}
{"error":"review: preparation failure","afterFailure":{"pending":true,"deferred":true},"afterNext":{"pending":false,"selected":true}}
```

해제 조건: 현재 epoch의 준비 실패를 명시적으로 처리하고 안전한 fallback 또는 제한된 재시도로
repaint를 복구한다. 무조건 finally에서 pending을 false로 바꾸면 stale 실패가 더 최신 준비를
해제할 수 있으므로 epoch/dispose 경계를 유지해야 한다. 현재 실패·stale 실패·문서 교체 후 실패,
성공 뒤 같은 backend repaint를 정식 회귀 검사에 포함하고 실제 브라우저로 재검증한다.


## 남은 원격 조건과 후속 처리

문서-only trailing commit의 merge-tree 공백·링크·오늘할일 보존을 검사하고 push한다.
그 뒤 최신 head의 actual fast-pass·required check와 mergeability를 확인한다. 병합은 사용자 승인 단계다.
merge·issue close·release·배포는 수행하지 않았다.

병합이 승인되면 확정 merge SHA·CI·보정 결과·#8197 분리를 한국어 존댓말 comment로 남기고,
같은 asset의 merge SHA 고정 raw 이미지를 사용한다. 게시 시 body-file과 API 재조회로 본문을 확인한다.
현재는 해당 comment를 게시하지 않았다.
