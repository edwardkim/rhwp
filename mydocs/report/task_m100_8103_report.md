# #8103 선택 오버레이 배치 완료 동기화 결과

Issue: [#8103](https://github.com/edwardkim/rhwp/issues/8103) (참조만 사용, 자동 종료하지 않음)

## 원인과 변경 동작

기준 `upstream/devel`은 `def947c2149df5bc812d801e6e79be19a403b4d1`이다.
별도 클라우드 환경의 `codex/issue-8103-selection-layout`에서 구현했다.
최초 제품 코드 커밋은 `6a120a58610d54b52db931006ab0cbbe8800baea`이며,
재검토 보정 code head는 `9801e5eb929d91a79611f6b0902cae7c39e450ab`이다.
아래 최초 검증과 재검토 검증을 구분한다. PR은 아직 생성하지 않았으며 제출 보류 사유도 기록한다.

`InputHandler.afterEdit`은 `document-changed` 뒤 곧바로 `updateCaret`을 실행한다.
그때 논리 선택과 WASM의 page-local 선택 rect는 복원돼 있지만,
`CanvasView.refreshPagesForMutation`은 renderer 선택을 await하고 있어 VirtualScroll은
삭제 직후 한 쪽 배치일 수 있다. `SelectionRenderer.render`는 새 쪽의 offset/width가 없는
상태에서 좌표를 투영한다. 일반 본문 스크롤은 이 음영을 다시 계산하지 않는다.

이제 `refreshPages`의 쪽 정보 수집·`recalcLayout` 완료 뒤 기존
`document-layout-refreshed` 이벤트에서 **현재 선택**을 다시 읽어 `updateSelection`한다.
비동기 시작 때 선택을 캡처하거나 anchor/focus를 재설정하지 않는다.
재검토에서 일반 완료 경계의 focus 캐럿도 기존 `updatePosition`으로 확정 쪽 원점에 재투영했다.
기존 `CaretLayoutReveal` 예약이 있는 Enter·쪽/단 나누기는 기존 캐럿 갱신과 자동 스크롤을
유지한다. 일반 선택 재투영은 캐럿 이동·자동 스크롤·문서 mutation을 일으키지 않는다.

auto mutation renderer 준비에는 epoch/current/dispose를 공통 guard로 적용했다.
metrics 적용과 await 뒤 renderer 적용을 함께 fence하며, 폐기된 뷰는 layout 완료 이벤트도
발행하지 않는다. 현재 선택 변경·해제는 완료 시점의 동기 조회로 반영한다.

선행 계약과 실제 좌표 생산·소비 경로는
[단계 기록](../working/task_m100_8103_stage1.md)에 연결했다.
#3416의 유효 선택 복원과 #7539의 Enter reveal을 유지한다.
#8102의 셀 경로·#594의 호스트 오류 복구·조판·저장 속성은 변경하지 않았다.

## 최초 수정 전후 재현과 검증

Linux 클라우드, Rust 1.93.1, wasm-pack 0.15.0, Node 24, 시스템 Chromium의 실제 Studio DOM을
사용했다. 입력은 E2E의 plain-text paste로 생성한 `P000`~`P399` 400문단이며
10쪽·400개 음영을 만들었다. 원문·독립 기대 계약은 테스트 코드에 포함된다.
실물 HWP/HWPX/PDF나 실제 사용자 데이터를 사용하지 않았다.

| 검사 | 수정 전 | 수정 후 |
| --- | --- | --- |
| 공통 73개 E2E assertion: 삭제·Undo·Redo, 100%/66%, 첫/중간/마지막 쪽, 새 선택 대조군, 스크롤·지연 중 새 선택/줌 | 41 PASS / 32 FAIL, exit 1 | 73 PASS / 0 FAIL, exit 0 |
| 확장 #8103 E2E: active selection 줌, 재선택 없는 복원 선택의 마지막 쪽 Backspace와 Undo, 지연 중 선택 해제 포함 | 위 공통 실패로 원인 검출 | 98 PASS / 0 FAIL, exit 0 |
| 실제 CanvasView 메서드의 epoch·후속 revision·current·dispose, 완료 이벤트 순서 | 0 PASS / 2 FAIL | 2 PASS / 0 FAIL |
| 기존 Enter·Undo·Redo E2E | 기존 계약 유지 검사 | 84 PASS / 0 FAIL |
| 기존 편집 Undo 계약 E2E | 기존 계약 유지 검사 | 99 PASS / 0 FAIL |
| 기존 전체 선택 E2E | 기존 계약 유지 검사 | 63 PASS / 0 FAIL |
| 전체 Studio·editor unit | 해당 변경 후 실행 | 2,365 PASS / 0 FAIL / 0 SKIP |
| TypeScript·production build | 해당 변경 후 실행 | exit 0 |
| E2E manifest | 기준 devel에 기존 파일 1개 미등재 | 166개 / 166행 일치 |

100%/66% Undo에서 잘못 투영된 음영은 수정 후 0개다. 쪽 번호 1·6·10의 음영과
전체 rect 목록을 같은 확정 레이아웃의 새 전체 선택 대조군에 비교했다.
기대 좌표는 고정 픽셀이 아니라 `확정된 쪽 원점 + page-local rect × zoom` 관계이며
0.02 CSS px의 직렬화 반올림 오차만 허용한다. 논리 anchor/focus, 문단 순서·내용,
새 선택·해제의 보존, 실제 Backspace/Redo가 지운 범위를 별도로 검사한다.

수정 전 중간 쪽에 본문은 있지만 음영이 없는 캡처와, 수정 후 첫/중간/마지막 쪽에서
음영이 본문 줄을 따라가는 100%/66% 캡처를 직접 열어 확인했다.
자동 DOM 검사를 독립 한컴 화면 일치나 사용자 개인 브라우저 확인으로 표시하지 않는다.

## 재검토의 발견과 보정

사용자 재검토 요청에 따라 초기 제출 후보 `cebd9040`의 구현과 검증을 다시 대조했다.
초기 98개 E2E는 기본 Canvas2D였으며, auto 경로·실제 quiet 타이머·문서 교체·연속 줌의
검증이 부족했다. [재검토 단계 기록](../working/task_m100_8103_stage2.md)에 사전 가설과
수정 전 증거를 남겼다. 다음 세 누락을 보정했다.

- 이전 편집의 300ms quiet 타이머가 새 mutation의 metrics await 중 실행되어 최신
  layout 완료를 폐기했다. 새 mutation 시작 시 이전 예약을 취소하고 최신 완료 후 다시 예약한다.
  실제 auto 브라우저의 450ms 지연 검사는 보정 전 타임아웃, 보정 후 통과다.
- quiet 재선택으로 backend/metrics가 변할 때도 배치 확정 후 현재 선택을 재투영한다.
  완료 직전 current/dispose guard로 오래된 결과의 알림을 막는다. 실제 제품 메서드의
  quiet 타이머 검사와 알림 검사는 이전 소스에서 실패하고 보정 후 통과했다.
- 복원 선택의 focus 캐럿은 초기 수정에서도 한 쪽 원점에 남았다. 마지막 쪽의 실제 DOM top
  772.9와 확정 배치 관계의 기대 top 10975.4가 달랐고, 보정 후 일치했다.
  이는 진단 관측값이며 테스트는 고정 px 대신 현재 쪽 원점과 page-local rect 관계를 검사한다.

### 재검토 code head의 실행 결과

`9801e5eb929d91a79611f6b0902cae7c39e450ab`에서 fresh dev web/Node WASM을 다시 만들고
동기화한 뒤 실행했다. Rust/WASM source 변경은 없으므로 산출물 해시는 최초 검증과 같다.
전체 unit과 병행한 브라우저 초기 응답은 느렸지만 타임아웃 없이 회복하여 끝까지 완료했다.

| 명령·검사 | 결과 |
| --- | --- |
| locked wrapper web·Node dev WASM, root pkg/public SHA-256 비교 | 모두 exit 0, JS·WASM 각각 동일 |
| `npx tsc --noEmit`, `npm --prefix rhwp-studio run build` | 각각 exit 0 |
| `npm --prefix rhwp-studio test` | 2,367 PASS / 0 FAIL / 0 SKIP, exit 0 |
| 확장 선택 E2E, 새 Canvas2D·auto 브라우저, 원본 글꼴 공급 | 280 PASS / 0 FAIL / pageerror 0건, exit 0 |
| 기존 Enter, Undo 계약, 전체 선택 E2E | 각각 84 / 99 / 63 PASS, FAIL 0, exit 0 |
| manifest·변경 문서 링크·`git diff --check` | 이상 없음 |
| CanvasKit 강제 모드의 추가 지연 경계 진단 | **타임아웃으로 실패(exit 1), 미해결** |

확장 E2E는 400문단의 100%/66% 복원 선택·첫/중간/마지막 쪽·새 선택 대조군·실제 전체 삭제와
Redo를 유지한다. focus 캐럿, 두 쪽 보기·viewport resize, 실제 smooth zoom·스크롤,
지연 중 역방향 새 선택·선택 해제·새 revision의 역순 완료·실제 문서 교체를 추가했다.
두 backend의 smooth zoom은 각각 브라우저 내 3회 관측에서 anchor/focus를 보존했고,
최대 투영 반올림 차이는 약 0.004982 CSS px였다. 기존 0.02 허용치는 바꾸지 않았다.
완료 count는 `source === 'mutation'`만 세어 quiet 알림이 대기를 거짓으로 만족하지 않게 했다.
해당 280개 실행의 caught console render error도 0건이다.

### CanvasKit 강제 모드의 실패와 검증 한계

클라우드 Chromium의 CDN 글꼴 요청은 직접 fetch 실패와 proxy 인증서 오류였다.
TLS 검증을 유지한 curl로 공개 원본 `HANBatang.woff`를 내려받아 정확한 URL에 원본 바이트만
공급했다. 글꼴 SHA-256은 `b3192018e3bd4da76b29b0530f71c79fb6f465a2b7cd41308bc38f9a4ae09a6c`다.
renderer·metrics·선택 rect는 대역하지 않았다. 테스트는 원본 바이트 소비를 검사하며,
강제 CanvasKit에서는 실제 backend도 검사하므로 fallback을 성공으로 세지 않는다.

이 실제 CanvasKit 실행은 앞선 전체 삭제·Undo·Redo·스크롤·줌 관계 검사 134개를 통과했지만,
metrics 대기 중 `CanvasKit font family가 준비되지 않았습니다: 함초롬바탕`이 발생하고
선택 해제 단계의 mutation 완료 대기가 타임아웃됐다. **전체 실행은 실패이며 134개를
완료 성공으로 합산하지 않는다.** Vite 메모리 내 transform으로 두 제품 파일만 `cebd9040`
소스로 공급한 대조군도 같은 글꼴 오류와 같은 단계의 타임아웃을 재현했다.
이는 이전 소스에도 존재한다는 근거이며 해결 완료의 근거가 아니다. 렌더 자원 수명과
호스트 복구 구현을 이슈 #8103의 수정에 섞지 않았으며 강제 CanvasKit의 해당 경계는 미해결이다.

재실행 명령(글꼴 파일은 커밋하지 않음):

```bash
curl --fail --location --output work/HANBatang.woff 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff'
sha256sum work/HANBatang.woff
(cd rhwp-studio && node e2e/run-with-vite.mjs -- node e2e/selection-layout-issue8103.test.mjs --mode=headless --font-cache=../work/HANBatang.woff)
# 실패를 확인한 추가 진단. 필수 선택 E2E의 성공과 구분한다.
(cd rhwp-studio && node e2e/run-with-vite.mjs -- node e2e/selection-layout-issue8103.test.mjs --mode=headless --renderer=canvaskit --font-cache=../work/HANBatang.woff)
```

## 실행 명령과 산출물

기본 작업공간 공용 target은 `/workspace/rhwp/target/pr-review`이며 다른 target을 만들지 않았다.
WASM은 저장소 루트에서 locked wrapper로 fresh **dev** web package와 Node package를 만들었다.
웹 package를 Studio `public/`에 자동 동기화했고 JS·WASM의 SHA-256 쌍이 각각 일치했다.
실제 E2E는 Vite가 이 fresh web package를 사용하는 새 Chromium 세션에서 실행했다.

```bash
npm_config_cache=/workspace/work/npm-cache npm --prefix rhwp-studio ci
CARGO_TARGET_DIR=/workspace/rhwp/target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev
CARGO_TARGET_DIR=/workspace/rhwp/target/pr-review scripts/wasm-pack-locked.sh --target nodejs --out-dir pkg-node --dev
sha256sum pkg/rhwp.js rhwp-studio/public/rhwp.js pkg/rhwp_bg.wasm rhwp-studio/public/rhwp_bg.wasm
(cd rhwp-studio && npx tsc --noEmit)
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
npm --prefix rhwp-studio run e2e:selection-layout
npm --prefix rhwp-studio run e2e:enter-caret
(cd rhwp-studio && node e2e/run-with-vite.mjs -- node e2e/undo-contracts.test.mjs --mode=headless)
npm --prefix rhwp-studio run e2e:select-all-cmd-a
python3 scripts/check_e2e_manifest.py
python3 scripts/check_markdown_links.py --changed-from upstream/devel
git diff --check upstream/devel...HEAD
git diff --check
```

web JS SHA-256: `bafa900f2814ed17312932aa6ca88f2d8dc57123998f313d637d1f7e5ecb7ec4`.
web WASM SHA-256: `adc91f301744636274a841f7bc028357ee6caf20e99d08874bef3afa81984f3c`.
생성 package·로그·HTML·브라우저 캡처는 Git에 포함하지 않는다.

### 실패·재실행의 구분

- 초기 npm cache 쓰기 권한 실패는 workspace cache로 보정했다.
- 첫 전체 unit은 Node WASM 부재로 2개가 skip됐다. fresh `pkg-node`를 빌드한 뒤
  전체를 재실행해 2,365 PASS / 0 SKIP을 확인했다. skip을 통과로 합산하지 않았다.
- 확장 E2E 한 실행은 병행 web WASM 재생성의 Vite reload 때문에 타임아웃됐다.
  이 실행은 유효한 제품 검증으로 세지 않았고, 빌드 완료 뒤 새 브라우저에서 98개를 재실행했다.
- manifest의 기존 미등재 파일은 #8120 통합 `80968497c2c178eeecb3b2e1d04d086ad27a0dc7`에
  들어온 `canvaskit-periodic-borders-issue7998.test.mjs`다. 기준 devel에도 목록 행이 없는 것을
  확인하고 행만 등록했다. 해당 테스트·fixture·다른 이슈의 구현은 변경하지 않았다.
- 기존 Undo E2E의 셀 캐럿 조회 warning은 assertion 실패와 구분한다.
  기존 #7539 보고서에도 기록된 경로이며 #8102 해결의 증거로 사용하지 않는다.

## 적용 게이트와 남은 확인

문서 paint·조판·페이지네이션·Rust 입력·baseline을 바꾸지 않는 Studio DOM UI 변경이다.
시각 거버넌스의 `studio/확장 UI (렌더 엔진 무관)` 행에 따라 기능 E2E와 화면 판독을 적용했다.
Native/fresh WASM 문서 Print PDF Visual Sweep·TSV는 비해당이다.
Rust 전체 lint/integration·release WASM 최적화·한컴 출력 일치를 검증했다고 주장하지 않는다.
성능 benchmark·Safari/Firefox·macOS의 실제 사용자 브라우저는 미검증이다.
정확한 PR head의 원격 CI·타인 리뷰는 로컬 검증과 별도 상태로 확인한다.

### PR 제출 보류 상태

재검토 시점의 기존 본인 PR [#8178](https://github.com/edwardkim/rhwp/pull/8178)은
새 head `e32f63d2aac839190d2b8085f0ec5346fa4e0479`에서 `CI Impact Policy` failure이며
CodeQL JS가 실행 중이다. [실패 run](https://github.com/edwardkim/rhwp/actions/runs/39933707917)의
`Publish exact-head policy status` step 실패를 API로 확인했다. raw 로그 다운로드는 Forbidden으로
하위 원인을 확인하지 못했다. [#8180](https://github.com/edwardkim/rhwp/pull/8180)도 새 열린
본인 PR이며 CI 실행 중이다. 두 PR의 개인/팀 reviewer 요청은 없고 확인된 리뷰는 보안 봇의
COMMENTED뿐이다. 다른 세션의 PR은 수정하지 않았다.

[새 PR 생성 전 절차](../manual/codex/docs_and_git_workflow.md#새-pr-생성-전에-확인할-항목)는
리뷰어 예외가 없는 기존 PR의 실패·진행 중 상태를 성공으로 간주해 새 PR을 만들지 않게 한다.
사용자 재검토 요청은 이 우선순위 예외 승인이 아니므로 신규 PR을 생성하지 않았다.
강제 CanvasKit 지연 경계도 미해결 실패로 남는다. 따라서 모든 검증·제출이 완료됐다고
보고하지 않으며 실제 #8103 PR URL은 아직 없다. merge·release·배포·이슈 종료는 실행하지 않았다.

사용자 직접 확인 절차:

1. 이 PR head에서 fresh WASM을 준비하고 Studio를 새로고침한다.
2. 빈 문서에 짧은 문단 400개를 붙여넣고 문서 처음에서 Cmd/Ctrl+A → Backspace →
   Cmd/Ctrl+Z를 실행한다. **재선택하지 않는다.**
3. 첫 쪽·중간 쪽·마지막 쪽으로 스크롤하면서 모든 본문 줄에 음영이 있는지 확인한다.
   100%와 66%로 배율을 바꾸고 쪽 경계를 지날 때도 음영이 본문을 따라가는지 확인한다.
4. 마지막 쪽을 보는 상태에서 Backspace를 눌러 음영 범위 전체가 삭제되고 빈 한 쪽만
   남는지 확인한다. Undo로 400문단·선택을 복원하고 Redo로 같은 범위가 삭제되는지 확인한다.
5. Undo 뒤 재선택 없는 화면과 Cmd/Ctrl+Home → Cmd/Ctrl+A의 새 선택 대조군을 비교한다.
   배치 완료만으로 스크롤이 선택 끝으로 이동하거나 anchor/focus가 변하지 않는지 확인한다.

자동 검증은 이 경로를 Chromium에서 확인했으며 위 절차의 사용자 환경 직접 판정은 남아 있다.
이번 요청은 PR 생성까지이며 merge·release·배포·이슈 종료는 실행하지 않는다.


## 로컬 인계 검증 (2026-10-24, PR 제출 보류)

클라우드 수치와 아래 macOS 실행을 구분한다. 인계 원격 head는
`a6753b509844c7c99c3bd8cafc6fce7af43d82c5` 그대로였으며, 기본 작업공간의 #8104
미커밋 변경은 보존하고 별도 worktree에서 진행했다. 최신 base는
`60972534680a0c5e51e54ec189b92e252e81a867`, 자동 리베이스 후보는
`f9e30f048574c15bb28bd5ef92b5c7d6d1e9fa2e`다. range-diff에서 제품 패치는 동일하고
manifest 문맥만 달라졌다. #8178·#8180은 병합됐으며 인증 계정의 열린 PR은 없었다.

### 실제 UI에서 추가 발견한 배치 경계

400문단을 실제 붙여넣고 전체 삭제→Undo 후 재선택 없이 중간 쪽을 확인했다.
100%에서는 본문과 음영이 일치했지만 확대/축소 대화상자로 66%를 적용하자 음영이
이전 쪽 원점을 사용했다. 마지막 음영이 새 문서 높이 밖으로 남아 스크롤 영역도 늘어났다.
입력은 공개 합성 문자열이며 사용자 문서를 사용하지 않았다.

경로는 `ZoomDialog.onConfirm → page-view-settings-changed → setPageViewSettings →
setZoom(zoom-changed) → InputHandler.updateViewportOverlayPositions → recalcLayout`이다.
transaction 중 CanvasView의 일반 zoom listener는 억제되므로 InputHandler가 이전 원점을
소비한다. 최종 `recalcLayout`·앵커 복원·visible 갱신 뒤 기존 완료 이벤트를 발행해
현재 선택과 캐럿을 재투영한다. 논리 선택·문서 조판·renderer 자원 복구는 바꾸지 않는다.
기존 #6987의 실제 merge `ffc88e54fd17be9432d7985cd1c7ffa9aae39d3b`와 리뷰 기록의
transaction/줌 보존 검증을 확인했다. 이 경계의 기존 검사는 복원된 전체 선택을 다루지 않았다.

실제 대화상자 E2E는 제품 보정 전 요청 배율 80% 적용을 확인한 상태에서 선택 전체·focus
캐럿·첫/중간/마지막 쪽 관계 5건이 실패했다. 처음 추가한 숫자 입력은 배율이 변하지 않아
재현 증거에서 제외했고, 요청 배율 assertion을 추가했다. 기대값은 확정 쪽 원점과 page-local
rect 관계이며 허용치를 변경하지 않았다. 보정 후 Canvas2D·auto·강제 CanvasKit에서 해당 5건과 요청 배율 검사가 모두 통과했다.

### CanvasKit 지연 장치의 경합

로컬 강제 CanvasKit도 선택 해제 단계에서 30초 timeout으로 실패했다. 기존
`beginDelayedMutation`은 새 mutation을 emit한 즉시 반환했으나 CanvasKit은 renderer resolve에서
실제 글꼴 준비를 await한다. 따라서 새 metrics gate 진입 전에 이전 release 함수를 호출하거나
준비 중 renderer를 사용하는 경합이 있었다. 매 회차 진입 표식을 초기화하고 이번
`prepareCanvasMetrics` 호출에 실제로 진입했는지 기다린다. renderer·metrics·선택 rect를
대역하지 않고 현재 메서드 실행을 지연시키는 기존 계약을 유지한다. 제품 글꼴 복구 코드는
변경하지 않는다. 보정 전 실패를 성공으로 합산하지 않는다.


### 로컬 실행 결과와 남은 차단 조건

검증 제품 head는 `19f943d9c171d23043dfbcfddd5457a68642fd70`이다. 이후 변경은
E2E 콘솔 오류의 발생 단계 기록과 이 보고서뿐이며 제품 소스는 동일하다.
기본 작업공간과 별개인 `/Users/melee/.codex/worktrees/issue-8103-selection-layout/rhwp`에서
fresh web·Node WASM을 locked wrapper로 순차 빌드했다. 공통 절대 target은
`/Users/melee/Documents/projects/forks/rhwp/target/pr-review`다. pkg/public의 JS SHA-256은
`4c8a5ff1ff30987c07cbce6b88d35d8487b067d274475c8b9946b5f9d5b21332`, WASM SHA-256은
`27a0fee1ad2875cfe8423ee43667b2411c9f040a3b4c8c43d92bbbca114fa5f1`로 각각 일치했다.
생성된 `public/rhwp.js`는 커밋하지 않는다.

| 검증 | 로컬 결과 |
|---|---|
| fresh dev web·Node WASM | 둘 다 성공 |
| `npx tsc --noEmit`, `npm test`, `npm run build` | 성공, unit 2427 PASS / 0 FAIL / 0 SKIP |
| selection-layout Canvas2D·auto | 합계 295 PASS / 1 FAIL, auto의 caught render error 검사 실패 |
| selection-layout 강제 CanvasKit | 148 PASS / 1 FAIL, caught render error 검사 실패 |
| Enter caret / Undo contracts / select-all | 84 / 99 / 63 PASS, 각각 FAIL 0 |
| zoom-dialog-transaction | 37 PASS / 0 FAIL |
| cell-target-issue8102 | exit 0, 오류 배열 비어 있음 |
| canvaskit-font-family-fallback-issue7445 | 1 PASS / 0 FAIL |
| manifest / Markdown 링크 / diff 공백 | 168개·168행 / 637문서·9변경 / 통과 |

브라우저 명령은 Studio 디렉터리에서
`CHROME_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
VITE_URL=http://127.0.0.1:7700 node e2e/selection-layout-issue8103.test.mjs --mode=headless`이며,
강제 경로는 `--renderer=canvaskit`을 추가했다. 원본 글꼴 cache 대역 옵션 없이 로컬 CDN을
사용했다. 실제 backend 확인 assertion을 유지한다. 다른 E2E도 같은 환경에서 해당 `.test.mjs`를
실행했다. 로그는 ignored `output/pr-review/8103/local/final-*.log`에 보존한다.

지연 장치의 gate 진입 확인으로 기존 30초 timeout은 해소됐지만, **전체 CanvasKit 통과는 아니다.**
최종 실행에서 좌표·선택 보존·역순 완료·문서 교체 검사와 pageerror 0건은 통과했다.
별도로 console.error를 수집하자 auto에서 `함초롬바탕` family 미준비 오류 4건이 검출됐다.
강제 CanvasKit에서는 66% 삭제/복원 단계의 이전 페이지 조회 오류 1건과 인위적 지연 단계의
family 미준비 오류 2건이 검출됐다. 발생 단계는 `final-canvaskit.log`에 보존했다.
콘솔 오류 assertion을 제거하거나 skip하지 않는다.

최신 base `60972534680a0c5e51e54ec189b92e252e81a867`의 두 제품 파일
`input-handler.ts`·`canvas-view.ts`를 `git show`로 읽어 별도 Vite pre-transform에서 공급하는
7701 대조 서버를 실행했다. 다른 제품 파일과 Rust/WASM 입력은 base와 동일함을 diff로 확인했다.
소스 파일을 바꾸거나 다른 checkout의 WASM을 복사하지 않았으며, 별도 Vite cache를 사용했다.
대조 서버 정의는 `output/pr-review/8103/local/base-server.mjs`다.
base auto에서도 초기화 중 같은 family 미준비 오류 2건이 발생했고, 원래 선택 투영 검사도
실패했다. base 실행은 후반 지연 검사에서 timeout으로 끝났으므로 전체 성공 대조군으로 쓰지 않는다.
근거는 `base-auto-diagnostic.log`다. 이는 클라우드의 검토 전 코드 대조와 별개로 확보한 최신
upstream/devel 재현 증거다. 강제 CanvasKit의 모든 오류가 base와 같다고 확대 해석하지 않는다.

리베이스로 들어온 #8180 셀 target과 #7445 font fallback, 기존 줌 transaction 검사는
base·후보 양쪽에서 각각 exit 0이었다. #8182 page text layout API는 선택 overlay의
getSelectionRects 경로와 다르고 해당 소스를 변경하지 않았다. packaging·PDF codec·기존 검증
문서 변경은 이번 DOM overlay 수정과 교차하지 않는다. 이 결과를 문서 렌더링 전반의 시각
동일성 증거로 확대하지 않는다. 본 수정은 문서 조판·paint를 변경하지 않는 선택/캐럿 DOM 배치다.

실제 로컬 UI에서도 400문단 붙여넣기 → 전체 삭제 → Undo → 재선택 없는 스크롤 → 확대/축소
대화상자 66% → 마지막 쪽을 확인했다. 보정 뒤 음영·마지막 짧은 문단·캐럿이 일치했고,
본문 재클릭 없이 Backspace로 빈 한 쪽, Undo로 전체 복원, Redo로 빈 한 쪽을 확인했다.
보정 전후 화면은 기본 작업공간의 ignored
`output/pr-review/8103/local/dialog-before.png`와 `dialog-after.png`에 보존했다.
이는 에이전트의 실제 UI 관측이며 사용자 본인의 판정은 아직 받지 않았다.

로컬 7700 서버는 유지한다. 사용자에게 Canvas2D·auto URL과 400문단 텍스트 파일, 상세 조작
순서를 전달했다. 사용자에게는 재선택하지 않는 스크롤·66% 배율·Backspace·Undo/Redo의 결과를
확인하도록 안내했다. **필수 E2E 오류가 남아 push·PR 생성은 보류한다.** 사용자 지시에서 제외한
다른 이슈의 renderer 자원 복구 구현으로 확장하지 않았다. merge·release·배포·이슈 종료도
실행하지 않았다. 선택 투영 개선과 별개인 렌더 오류의 해결 범위가 남아 있다.
