@edwardkim @jangster77 리뷰 감사합니다. 두 분 의견을 반영해 v2 방향을 다음처럼 정리했습니다.

게시 댓글: https://github.com/edwardkim/rhwp/issues/2023#issuecomment-4917066612
문서 전용 PR: https://github.com/edwardkim/rhwp/pull/2080

## 반영 방향

1. TS 복잡도 측정 도구는 `eslint` + `sonarjs/cognitive-complexity`를 우선 후보로 고정합니다.
   Phase 0에서는 fail gate가 아니라 snapshot/advisory로 시작하고, 실제 fail gate 승격은
   baseline 분포와 예외 정책을 본 뒤 결정하겠습니다.

2. v1 진단 수치(`CC>25`, 최대 CC, SOLID 54/100)는 공식 baseline이 아니라 heuristic baseline으로
   격하합니다. v2에서는 모집단과 제외군을 고정하고, Phase 0에서 공식 도구로 재측정하겠습니다.

3. `web/fonts` canonical은 `rhwp-studio/public/fonts` 실체화가 아니라 공통 `assets/fonts` 신설
   방향으로 잡겠습니다. Studio, Chrome, Firefox, Safari, VS Code, npm은 이 공통 루트에서 target별
   copy 계약으로 받는 구조로 정리하겠습니다.

4. 금지 목록은 후보가 아니라 확정 규칙으로 올립니다. `@rhwp/editor` iframe/postMessage/무의존
   계약, runtime framework 도입 금지(v1.0까지), CSP 완화 금지, inline script 금지,
   `web_accessible_resources` 확대 금지, sender/URL/file 검증 약화 금지, 리팩터링과 기능/보안
   변경 혼합 금지를 확정으로 두겠습니다.

5. Chrome/Firefox/Safari 중 하나의 extension 코드나 asset copy 계약을 건드리면 나머지 2곳과
   `rhwp-shared` 계약을 확인하는 3-browser sync 규칙을 추가하겠습니다.

6. Phase 0 산출물에 무변동 게이트 manifest를 포함하겠습니다. Studio build/test/e2e, extension
   build/dist asset smoke, VS Code compile/webview smoke, `@rhwp/editor` embed smoke, WASM JSON
   schema advisory snapshot을 표면별로 분리하겠습니다.

7. 실행 하위 이슈는 v2 승인 전에는 만들지 않고, 승인 후에도 우선 Phase 0 baseline freeze와
   Phase A `assets/fonts` canonical 이전만 분리하겠습니다. `InputHandler`, dialog, `WasmBridge`,
   `diff-engine` 해체는 Phase 0/A 완료 후 재평가하겠습니다.

## v2 산출물

위 내용을 `mydocs/plans/task_m100_2023_frontend_refactoring_plan_v2.md`에 반영했습니다.

GitHub에서 바로 확인 가능한 문서 전용 PR은 #2080입니다.

v2를 기준으로 다시 리뷰 부탁드립니다. 리뷰 승인 후에만 Phase 0/A 하위 이슈를 분리하겠습니다.
