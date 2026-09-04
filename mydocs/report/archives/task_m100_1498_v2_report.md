# Task M100 #1498 v2 최종 보고서 — 확장 0.2.7 과거 다운로드 onCreated 재오픈 방지

- 이슈: #1498 후속
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1498-v2`
- 작성일: 2026-06-24

## 1. 결론

0.2.7 배포본에는 #1498의 `seen` 가드가 포함되어 있었지만, `onCreated`로 전달되는 과거 완료
다운로드 항목을 막지 못했다. `seen`은 `onChanged` 단독 경로만 막았고, `onCreated` 자체를
"새 다운로드"로 신뢰한 것이 원인이다.

이번 수정으로 `DownloadItem.startTime`이 service worker/event page 기동 시각보다 충분히 오래 전인
항목은 `onCreated`와 `onChanged` 재조회 양쪽에서 무시한다. 기존 새 다운로드 동작은 유지한다.

## 2. 변경 파일

| 파일 | 변경 |
|---|---|
| `rhwp-chrome/sw/download-interceptor.js` | `workerStartedAt`/`isFreshDownloadItem` 추가, onCreated/onChanged/process 최종 fresh 가드 |
| `rhwp-firefox/sw/download-interceptor.js` | Chrome과 동일한 startTime 기반 fresh 가드 |
| `rhwp-chrome/sw/download-interceptor.test.mjs` | onCreated 과거 완료 항목, onChanged 재조회 과거 항목 회귀 테스트 추가 |
| `mydocs/orders/20260624.md` | #1498 v2 후속 작업 기록 |
| `mydocs/plans/task_m100_1498_v2*.md` | 수행/구현 계획서 |
| `mydocs/working/task_m100_1498_v2_stage1.md` | 단계 완료보고서 |

## 3. 검증

| 명령 | 결과 |
|---|---|
| `node --test rhwp-chrome/sw/download-interceptor.test.mjs` | 10 passed |
| `node --check rhwp-chrome/sw/download-interceptor.js` | 통과 |
| `node --check rhwp-firefox/sw/download-interceptor.js` | 통과 |
| `npm run build` (`rhwp-chrome`) | 통과 |
| `npm ci` (`rhwp-firefox`) | 통과, 기존 dependency audit high 1건 |
| `npm run build` (`rhwp-firefox`) | 통과 |

## 4. 영향

- Chrome/Edge/Firefox 확장에서 service worker 재기동 또는 브라우저 재시작 후 과거 다운로드 완료
  항목이 `onCreated`로 들어와도 뷰어를 열지 않는다.
- service worker 기동 직후의 정상 다운로드 지연을 고려해 5초 grace window를 둔다.
- `startTime`이 없는 브라우저/환경은 기존 호환성을 위해 fresh로 처리한다.
- `shouldInterceptDownload`와 WASM/viewer 로직은 변경하지 않았다.

## 5. 후속

- 확장 재배포 시 0.2.8 또는 0.2.7 재패키징 여부를 릴리스 단계에서 결정한다.
- 실제 Chrome에서 닫은 과거 문서가 재시작 후 다시 열리지 않는지 작업지시자 환경에서 최종 확인한다.
