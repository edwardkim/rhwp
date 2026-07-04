# Task M100 #1498 v2 1단계 완료보고서 — onCreated 과거 항목 신선도 가드

- 이슈: #1498 후속
- 브랜치: `local/task1498-v2`
- 작성일: 2026-06-24
- 단계: 1/1

## 1. 재현

스토어 0.2.7 배포본은 #1498의 `seen` 가드를 포함하고 있었지만, 다음 조건에서는 여전히
뷰어 탭을 열었다.

- `chrome.downloads.onCreated`로 들어온 항목
- `filename`/`url`/`mime`은 HWP로 판정 가능
- `state: "complete"`, 과거 `startTime`/`endTime`

기존 테스트는 `onChanged` 단독 과거 항목만 검증해서 이 케이스를 잡지 못했다.

## 2. 변경

### `rhwp-chrome/sw/download-interceptor.js`

- `workerStartedAt = Date.now()` 기록.
- `DOWNLOAD_FRESH_GRACE_MS = 5_000` 도입.
- `isFreshDownloadItem(item)` 추가:
  - `startTime`이 없거나 파싱 불가하면 호환성을 위해 fresh로 처리.
  - `startTime` 또는 `endTime`이 `workerStartedAt - 5초`보다 오래 전이면 과거/복원 항목으로 보고 무시.
- `onCreated`에서 fresh 항목만 `seen`에 등록하고 처리.
- `onChanged` 재조회 결과도 fresh 검사 후 처리.
- `processDownloadItem`에 최종 fresh 가드 추가.

### `rhwp-firefox/sw/download-interceptor.js`

- Chrome과 동일하게 `workerStartedAt`, `DOWNLOAD_FRESH_GRACE_MS`, `isFreshDownloadItem` 적용.
- `onCreated`, `onChanged` 재조회 경로 모두 fresh 검사.

### `rhwp-chrome/sw/download-interceptor.test.mjs`

신규 테스트 2건 추가:

- 과거 완료 항목이 `onCreated`로 들어와도 뷰어 미오픈.
- `onChanged` 재조회 결과가 과거 항목이면 `seen`에 있어도 뷰어 미오픈.

## 3. 검증

| 항목 | 결과 |
|---|---|
| 기존 코드 + 신규 테스트 | 실패 재현: 신규 2건 fail |
| `node --test rhwp-chrome/sw/download-interceptor.test.mjs` | 통과: 10 passed |
| `node --check rhwp-chrome/sw/download-interceptor.js` | 통과 |
| `node --check rhwp-firefox/sw/download-interceptor.js` | 통과 |
| `npm run build` (`rhwp-chrome`) | 통과 |
| `npm ci` (`rhwp-firefox`) | 통과, 기존 dependency audit high 1건 |
| `npm run build` (`rhwp-firefox`) | 통과 |
| dist 반영 확인 | chrome/firefox dist `sw/download-interceptor.js`에 fresh 가드 포함 |

## 4. 비고

Firefox 빌드 전 `rhwp-firefox/node_modules/vite`가 없어 최초 빌드가 실패했다. `npm ci` 후
잠금파일 기준 의존성을 설치하고 빌드를 재실행해 통과했다. `npm audit`의 high 1건은 기존
의존성 상태이며 이번 수정 범위 밖이다.
