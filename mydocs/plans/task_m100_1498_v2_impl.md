# Task M100 #1498 v2 구현계획서 — startTime 기반 다운로드 신선도 가드

- 이슈: #1498 후속
- 브랜치: `local/task1498-v2`
- 작성일: 2026-06-24
- 수행계획서: `mydocs/plans/task_m100_1498_v2.md`

## 구현 개요

0.2.7의 `seen` 집합은 유지하되, `onCreated`와 `onChanged` 재조회 결과 모두에 `DownloadItem`
신선도 검사를 추가한다. 핵심은 `onCreated`로 들어온 항목도 과거 기록일 수 있다는 전제를 코드에
반영하는 것이다.

핵심 불변식:

- `onChanged` 단독으로는 열지 않는다.
- `onCreated`로 들어와도 service worker 기동 이전의 완료/종료 항목은 열지 않는다.
- service worker 기동 이후 시작된 새 다운로드는 기존처럼 열린다.

## 1단계 — chrome 신선도 함수와 테스트

대상:

- `rhwp-chrome/sw/download-interceptor.js`
- `rhwp-chrome/sw/download-interceptor.test.mjs`

구현:

1. 모듈 로드 시 `const workerStartedAt = Date.now();` 추가.
2. `isFreshDownloadItem(item)` 함수 추가.
   - `item.startTime`이 유효하고 `Date.parse(item.startTime) < workerStartedAt - graceMs`이면 false.
   - `item.endTime`이 유효하고 `Date.parse(item.endTime) < workerStartedAt - graceMs`이면 false.
   - `startTime`이 없거나 파싱 불가한 경우는 새 다운로드 호환성을 위해 true로 둔다.
3. `onCreated`에서 fresh가 아니면 `seen.add`와 `processDownloadItem` 모두 수행하지 않는다.
4. `onChanged`의 `downloads.search` 결과도 fresh 검사 후 `processDownloadItem` 호출.

테스트 추가:

- `past completed download delivered through onCreated does not open the viewer`
- `past download returned from onChanged search does not open the viewer`
- 기존 새 다운로드 케이스에 `startTime: new Date().toISOString()`을 명시하거나, 없는 경우 호환 경로로 통과 확인.

완료 기준:

- `node --test rhwp-chrome/sw/download-interceptor.test.mjs` 통과.

## 2단계 — firefox 동일 적용

대상:

- `rhwp-firefox/sw/download-interceptor.js`

구현:

- chrome과 동형의 `workerStartedAt`, `isFreshDownloadItem` 가드 추가.
- `onCreated`, `onChanged` 재조회 결과 모두 fresh 검사.

검증:

- `node --check rhwp-firefox/sw/download-interceptor.js`

## 3단계 — 빌드와 보고서

검증:

- `node --check rhwp-chrome/sw/download-interceptor.js`
- `node --test rhwp-chrome/sw/download-interceptor.test.mjs`
- `npm run build` in `rhwp-chrome`
- `npm run build` in `rhwp-firefox`

문서:

- `mydocs/working/task_m100_1498_v2_stage1.md`
- `mydocs/report/task_m100_1498_v2_report.md`
- `mydocs/orders/20260624.md` 상태 갱신

## 위험 / 주의

- `startTime`이 없는 브라우저/환경에서 정상 다운로드를 막지 않도록 파싱 실패는 허용한다.
- 너무 짧은 시간 기준은 service worker 기동 직후 시작된 정상 다운로드를 오탐할 수 있으므로,
  `workerStartedAt` 기준에 5초 grace window를 둔다.
- `shouldInterceptDownload`는 문서 판정 전용으로 유지하고, 신선도 판단을 관찰자 레이어에 둔다.
