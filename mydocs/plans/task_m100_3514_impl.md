# Task M100 #3514 구현계획서 — packaged extension smoke harness

- 이슈: #3514
- 수행계획서: `mydocs/plans/task_m100_3514.md`
- 브랜치: `codex/issue-3514-extension-smoke`
- 작성일: 2026-08-20
- 현재 게이트: Stage 5 승인 완료, remote push·PR 생성 승인 대기

## 1. Hyper-Waterfall 단계

### Stage 1 — 조사·범위·계획

- `mydocs/orders/20260820.md`
- `mydocs/plans/task_m100_3514.md`
- `mydocs/plans/task_m100_3514_impl.md`
- `mydocs/feedback/task_m100_3514_hyper_waterfall_recovery.md`
- `mydocs/working/task_m100_3514_stage1.md`

이 다섯 경로만 먼저 stage한다. 구현 후보는 작업트리에만 두고 Stage 1 승인·커밋 전에는
index에 넣지 않는다.

### Stage 2 — packaged smoke 구현

- `rhwp-chrome/package.json`
- `rhwp-chrome/package-lock.json`
- `rhwp-chrome/e2e/extension-smoke.test.mjs`
- `mydocs/orders/20260820.md` 상태 갱신
- `mydocs/plans/task_m100_3514.md` 상태 갱신
- `mydocs/plans/task_m100_3514_impl.md` 상태 갱신
- `mydocs/working/task_m100_3514_stage2.md`

Stage 1 커밋 뒤 위 구현만 stage하고 focused 검증과 코드 diff를 Stage 2 승인 게이트에 제시한다.

### Stage 3 — 최종 검증·운영 문서

- `mydocs/manual/chrome_edge_extension_build_deploy.md`
- `mydocs/feedback/task_m100_3514_hyper_waterfall_recovery.md` 복구 완료 갱신
- `mydocs/plans/task_m100_3514.md` 상태 갱신
- `mydocs/plans/task_m100_3514_impl.md` 상태 갱신
- `mydocs/working/task_m100_3514_stage3.md`
- `mydocs/report/task_m100_3514_report.md`
- `mydocs/orders/20260820.md` 상태 갱신

Stage 2 커밋을 기준으로 전체 회귀와 새 profile 10회 smoke를 다시 실행한다. 최종 승인과 커밋 뒤에도
remote push와 draft PR 생성은 GitHub 권한 경계에 따라 각각 명시적 승인을 확인한다.

### Stage 4 — 탭 예산 즉시 중단 보강

PR 전 자체 검토에서 `targetcreated`가 예상 밖 page를 기록만 하고 실제 실패는 다음
`assertPageBudget` checkpoint까지 늦춘다는 완료 조건 불일치를 확인했다. 작업지시자의 Stage 4 진행
승인에 따라 다음을 보강한다.

- 예상 밖 page target 최초 생성 시 공유 실패 promise를 reject한다.
- worker/viewer/options/print/content-script의 모든 비동기 surface 작업을 이 실패 promise와 race한다.
- 정상 owned page와 service worker target은 허용한다.
- controller listener는 `finally` 정리 전에 detach한다.
- 끝나지 않는 surface도 page 이벤트만으로 실패하는 Node 계약 테스트를 smoke 명령에 포함한다.
- 새 profile 10회 실제 packaged smoke로 정상 경로 오탐이 없음을 확인한다.

Stage 4 변경과 검증 보고는 작업지시자 승인 전 커밋하지 않으며, 승인 뒤에도 push와 PR 생성은 별도
GitHub 경계로 남긴다.

### Stage 5 — 최신 `devel` 정합화·재검증

PR 생성 전 장기간 경과한 작업 브랜치를 최신 `upstream/devel@f26c2e7ca`에 리베이스하고 다음을
확인한다.

- 리베이스 전·후 네 Stage 커밋의 대응 관계를 `git range-diff`로 검토한다.
- 충돌은 당일 작업 목록의 최신 항목을 보존하면서 #3514 행을 합치는 문서 충돌로 한정한다.
- 최신 Rust 기준으로 네이티브 `wasm-pack --no-opt` package를 새로 만들고 frontend 전체 회귀를
  다시 실행한다.
- 실제 packaged Chrome smoke를 retry 없이 새 profile 10개로 다시 실행한다.
- Docker daemon, review-only 파생 파일처럼 로컬에서 충족되지 않은 표준 gate는 성공으로 오인하지
  않고 Stage 5 보고서에 검증 경계로 기록한다.

Stage 5에서는 기능 코드를 추가하지 않는다. 다음 문서만 갱신해 작업지시자 승인 뒤 별도 커밋하며,
remote push와 draft PR 생성은 계속 별도 GitHub 승인 경계로 남긴다.

- `mydocs/orders/20260820.md`
- `mydocs/plans/task_m100_3514.md`
- `mydocs/plans/task_m100_3514_impl.md`
- `mydocs/working/task_m100_3514_stage5.md`
- `mydocs/report/task_m100_3514_report.md`

## 2. 파일별 변경

### `rhwp-chrome/package.json`·`package-lock.json`

- `puppeteer`를 개발 의존성으로 추가해 호환되는 Chrome for Testing을 함께 고정한다.
- `test:e2e:smoke`가 extension build 뒤 smoke runner를 실행하게 한다.

### `rhwp-chrome/e2e/extension-smoke.test.mjs`

- 임시 profile/download 디렉터리와 loopback fixture/proxy 서버를 만든다.
- `enableExtensions: [distPath]`로 unpacked dist를 headless Chrome에 설치한다.
- service worker URL에서 extension ID를 동적으로 얻고 worker Runtime/Log 오류를 수집한다.
- page 생성 직후부터 console, pageerror, request/response 실패를 수집한다.
- `samples/hwp3-pagedef-1915.hwp`를 CORS가 허용된 loopback fixture로 제공한다.
- viewer에서 fixture 문서 로드와 `#scroll-container canvas`를 기다린다.
- 다크 media를 강제한 viewer에서 `data-theme-effective=dark`와 아이콘 자산 응답을 검증한다.
- options에서 네 설정 입력의 표시·활성화·version text를 확인한다.
- print page의 same-origin, title, loading status DOM을 확인한다.
- fixture page에서 extension-ready marker와 `.rhwp-badge` 정확히 1개를 확인한다.
- 예상 page budget과 외부 HTTP(S) request allowlist를 매 단계 검사한다.
- 예상 밖 page target은 공유 실패 gate로 진행 중 surface를 즉시 중단한다.
- 실패 시 surface별 오류·열린 page URL·worker URL을 한 번에 출력한다.

### `mydocs/manual/chrome_edge_extension_build_deploy.md`

- 새 smoke 명령, 전제조건, 검증 범위와 비범위를 추가한다.

### 작업 기록

- `mydocs/orders/20260820.md`, 계획서, 단계 보고서와 최종 보고서를 갱신한다.

## 3. 테스트 설계

### 오류 판정

- page `error` console과 `pageerror`
- extension/loopback resource의 HTTP 4xx/5xx와 request failure
- CSP violation console
- worker `Runtime.exceptionThrown`, error-level console, `Log.entryAdded`
- allowlist 밖의 page HTTP(S) request
- page budget 초과와 단계 timeout
- page-budget 계약 테스트는 끝나지 않는 surface를 두고 예상 밖 page 이벤트만으로 즉시 reject되는지
  검증한다.

### 허용 네트워크

- `chrome-extension://<runtime-id>/...`
- `http://127.0.0.1:<fixture-port>/...`
- `data:`, `blob:`, `about:` 같은 로컬 브라우저 scheme

외부 HTTP(S)는 loopback proxy에서 연결하지 않고 차단한다.

### 준비 상태

- viewer: 실제 HWP3 fixture의 document canvas가 존재하고 status text가 최종 파일명을 포함
- options: 네 checkbox가 모두 enabled이고 i18n title/version이 채워짐
- print: extension origin과 title/status DOM 확인
- content script: root marker와 정확히 한 개의 badge

## 4. 검증 명령

```bash
node --check rhwp-chrome/e2e/extension-smoke.test.mjs
npm --prefix rhwp-chrome test
npm --prefix rhwp-chrome run build
node --test scripts/frontend-extension-dist.test.mjs
npm --prefix rhwp-chrome run test:e2e:smoke
```

`rhwp-chrome`에 일반 `test` script가 없다면 기존 Node test 파일을 명시해 실행한다. 최종 smoke는
환경 재사용 없이 10회 연속 실행해 flake와 정리 누수를 확인한다.

## 5. 승인 근거

작업지시자는 2026-08-20 조사 결과와 제한 확장안을 확인한 뒤 “진행해줘”라고 승인했다.
이는 범위·착수 승인이었으며 formal Stage별 승인으로 기록하지 않는다. 이후 구현·검증을 단일 Stage로
묶은 절차 이탈을 작업지시자가 지적했고 “그렇게 진행해줘”로 위 3-Stage 복구 절차를 승인했다.
