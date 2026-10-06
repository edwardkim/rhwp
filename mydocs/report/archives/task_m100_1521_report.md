# Task M100 #1521 최종 보고서

## 요약

브라우저 확장 hover card의 지연 표시 타이머가 링크 `mouseleave` 이후에도 남아, 미리보기 카드가 뒤늦게 표시되고 화면에 남을 수 있는 문제를 정정했다.

원인은 Chrome/Firefox content-script가 show 예약과 hide 예약을 단일 `hoverTimeout`으로 관리한 데 있었다. 링크를 빠르게 벗어나면 pending show가 취소되지 않았고, 경우에 따라 늦게 실행된 `showHoverCard()`가 future hide timer까지 지워 카드가 닫히지 않을 수 있었다. Safari도 단일 타이머와 show 실행 직전 검증 부족을 같은 상태 모델로 정렬했다.

## 변경 내용

- `rhwp-chrome/content-script.js`
  - show/hide 타이머를 `showHoverTimeout`, `hideHoverTimeout`으로 분리
  - `activeAnchor`, `pendingAnchor` 상태 추가
  - 링크 `mouseleave`에서 pending show를 즉시 취소
  - `showHoverCard(anchor)` 실행 직전 `pendingAnchor`, DOM 연결, `:hover` 상태 검증
  - 카드 제거와 lifecycle 취소 책임을 분리해 stale show가 hide 예약을 지우지 못하게 정리
  - 썸네일 비동기 응답 guard를 `activeCard`와 `activeAnchor` 동시 검증으로 보강

- `rhwp-firefox/content-script.js`
  - Chrome과 같은 hover lifecycle 모델 적용
  - Promise 기반 썸네일 응답의 active card/anchor 검증 보강

- `rhwp-safari/src/content-script.js`
  - 기존 `activeAnchor`를 공통 상태 모델에 맞춰 정리
  - 단일 `hoverTimeout` 제거
  - 기존 show 250ms, hide 150ms 지연값 유지
  - 썸네일 비동기 응답이 닫힌 카드 DOM을 갱신하지 않도록 보강

## 해결되는 문제

- show delay 이전에 링크를 벗어나면 hover card가 생성되지 않는다.
- 링크를 충분히 hover하면 기존처럼 hover card가 표시된다.
- 링크에서 card로 이동하면 card가 유지된다.
- card를 벗어나면 hide delay 뒤 card가 닫힌다.
- 빠르게 여러 HWP/HWPX 링크를 지나가도 이전 링크의 stale card가 뒤늦게 남지 않는다.
- hover card click의 기존 `open-hwp` 요청 흐름은 유지된다.

## 검증

자동 검증 통과:

- `node --check rhwp-chrome/content-script.js`
- `node --check rhwp-firefox/content-script.js`
- `node --check rhwp-safari/src/content-script.js`
- `node --check rhwp-chrome/dist/content-script.js`
- `node --check rhwp-firefox/dist/content-script.js`
- `node --check rhwp-safari/dist/content-script.js`
- `node rhwp-chrome/sw/fetch-security.test.mjs`
- `node --test rhwp-chrome/sw/download-interceptor.test.mjs` — 13개 통과
- `node --test rhwp-firefox/sw/download-interceptor.test.mjs` — 11개 통과
- `git diff --check`

로드용 산출물 확인:

- `rhwp-chrome/dist/manifest.json` 파싱 성공: MV3, version `0.2.7`
- `rhwp-firefox/dist/manifest.json` 파싱 성공: MV3, version `0.2.7`
- `rhwp-safari/dist/manifest.json` 파싱 성공: MV3, version `0.2.1`
- 각 dist에 `manifest.json`, `viewer.html`, `background.js`, `content-script.js`, `wasm/rhwp_bg.wasm` 존재
- 각 dist의 `content-script.js`는 대응 source와 byte-for-byte 일치

수동 검증:

- 2026-06-29 작업지시자가 로드용 확장을 직접 로드해 hover card 수동 검증 통과를 확인했다.

## 빌드 참고

표준 `npm run build`는 현재 로컬 환경에서 Vite config 해석 문제로 실패했다.

확인된 원인은 이번 content-script 변경이 아니라 로컬 Node 의존성 설치 상태와 Vite config loader의 임시 경로 해석 문제다. 현재 환경에는 `rhwp-studio/node_modules`만 있고 `rhwp-chrome/node_modules`, `rhwp-firefox/node_modules`가 없었다. Vite가 `vite.config.ts`의 `import { defineConfig } from 'vite'`를 `/Users/melee/node_modules/.vite-temp/...` 기준으로 해석하면서 `vite` 패키지를 찾지 못했다.

이 문제는 이번 merge 변경에서 새로 도입되는 문제는 아니다. 다만 확장 빌드 경로가 의존성 설치 위치에 민감하므로, 별도 빌드 안정화 이슈로 분리하는 것을 권장한다.

## 변경 범위

추적 source 변경은 다음 3개 파일에 한정된다.

- `rhwp-chrome/content-script.js`
- `rhwp-firefox/content-script.js`
- `rhwp-safari/src/content-script.js`

작업 문서:

- `mydocs/orders/20260629.md`
- `mydocs/plans/task_m100_1521.md`
- `mydocs/plans/task_m100_1521_impl.md`
- `mydocs/working/task_m100_1521_stage1.md`
- `mydocs/working/task_m100_1521_stage3.md`
- `mydocs/working/task_m100_1521_stage4.md`
- `mydocs/report/task_m100_1521_report.md`

`rhwp-chrome/dist/`, `rhwp-firefox/dist/`, `rhwp-safari/dist/`는 로드용 산출물이며 git ignored 상태라 PR/커밋 대상에는 포함하지 않는다.

## 후속

- 최종 보고 승인 후 스테이징과 커밋을 진행한다.
- 이슈 #1521 close는 작업지시자 승인 후에만 수행한다.
- 확장 `npm run build` 안정화는 별도 이슈로 분리 검토한다.
