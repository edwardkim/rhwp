# #8103 선택 오버레이 배치 완료 동기화 결과

Issue: [#8103](https://github.com/edwardkim/rhwp/issues/8103) (참조만 사용, 자동 종료하지 않음)

## 원인과 변경 동작

기준 `upstream/devel`은 `def947c2149df5bc812d801e6e79be19a403b4d1`이다.
별도 클라우드 환경의 `codex/issue-8103-selection-layout`에서 구현했다.
제품 코드 커밋은 `6a120a58610d54b52db931006ab0cbbe8800baea`이며,
후속 변경은 E2E의 실제 Backspace 검증·기존 목록 누락 행·이 보고서다.
최종 제출 head와 그 head의 재실행 결과는 PR 본문에 기록한다.

`InputHandler.afterEdit`은 `document-changed` 뒤 곧바로 `updateCaret`을 실행한다.
그때 논리 선택과 WASM의 page-local 선택 rect는 복원돼 있지만,
`CanvasView.refreshPagesForMutation`은 renderer 선택을 await하고 있어 VirtualScroll은
삭제 직후 한 쪽 배치일 수 있다. `SelectionRenderer.render`는 새 쪽의 offset/width가 없는
상태에서 좌표를 투영한다. 일반 본문 스크롤은 이 음영을 다시 계산하지 않는다.

이제 `refreshPages`의 쪽 정보 수집·`recalcLayout` 완료 뒤 기존
`document-layout-refreshed` 이벤트에서 **현재 선택**을 다시 읽어 `updateSelection`한다.
비동기 시작 때 선택을 캡처하거나 anchor/focus를 재설정하지 않는다.
기존 `CaretLayoutReveal` 예약이 있는 Enter·쪽/단 나누기는 기존 캐럿 갱신과 자동 스크롤을
유지한다. 일반 선택 재투영은 캐럿 이동·자동 스크롤·문서 mutation을 일으키지 않는다.

auto mutation renderer 준비에는 epoch/current/dispose를 공통 guard로 적용했다.
metrics 적용과 await 뒤 renderer 적용을 함께 fence하며, 폐기된 뷰는 layout 완료 이벤트도
발행하지 않는다. 현재 선택 변경·해제는 완료 시점의 동기 조회로 반영한다.

선행 계약과 실제 좌표 생산·소비 경로는
[단계 기록](../working/task_m100_8103_stage1.md)에 연결했다.
#3416의 유효 선택 복원과 #7539의 Enter reveal을 유지한다.
#8102의 셀 경로·#594의 호스트 오류 복구·조판·저장 속성은 변경하지 않았다.

## 수정 전후 재현과 검증

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
