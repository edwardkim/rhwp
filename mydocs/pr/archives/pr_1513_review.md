# PR #1513 검토 — Task #1498 v2 다운로드 기록 게이트웨이 후속 회귀

- PR: <https://github.com/edwardkim/rhwp/pull/1513>
- 제목: `Task #1498: Fix stale download observer items after #1498`
- 작성자: `postmelee` / Taegyu Lee
- base: `devel`
- head: `local/task1498-v2`
- 문서 작성일: 2026-06-24
- 처리 결과: merge 완료 (`f54756c405d3f43f2c0bcbbc487da0f1e68fe52a`)

## 1. 배경

Chrome 확장 0.2.7 핫픽스에는 #1498의 `seen` 게이트가 포함되어 있었지만, 사용자는 여전히 브라우저
재시작 또는 확장 service worker 재기동 뒤 과거 HWP/HWPX 다운로드 기록이 탭으로 다시 열리는 현상을
보고했다.

기존 0.2.7 게이트는 `onChanged` 단독으로 들어오는 과거 항목을 막았지만, Chrome이 과거 완료 항목을
`onCreated`로 전달하는 경우까지는 차단하지 못했다. 이 PR은 `DownloadItem.startTime`/`endTime`을
사용해 service worker 기동 이전의 오래된 항목을 신선하지 않은 항목으로 거르는 후속 방어선이다.

## 2. PR 메타

| 항목 | 값 |
|---|---|
| 작성 시점 참고 head SHA | `c22f680181f51c8e80994891d1ed3349cedd404c` |
| 커밋 수 | 1 |
| 변경 파일 | 8 |
| 규모 | +384 / -4 |
| GitHub mergeable 참고값 | `MERGEABLE` / `CLEAN` |
| CI 참고값 | Build & Test 성공, CodeQL 성공, WASM Build skipped |

최종 merge 전에는 PR head 최신 커밋 기준으로 GitHub Actions와 mergeable 상태를 다시 확인해야 한다.

## 3. 변경 범위

| 파일 | 검토 내용 |
|---|---|
| `rhwp-chrome/sw/download-interceptor.js` | `workerStartedAt`, 5초 grace window, `isFreshDownloadItem()` 추가. `onCreated`, `onChanged` 재조회, 최종 처리 경로에 fresh 가드 적용 |
| `rhwp-firefox/sw/download-interceptor.js` | Chrome과 동일한 startTime/endTime 기반 fresh 가드 적용 |
| `rhwp-chrome/sw/download-interceptor.test.mjs` | `onCreated` 과거 완료 항목, `onChanged` 재조회 과거 항목 회귀 테스트 추가 |
| `mydocs/plans/task_m100_1498_v2*.md` | 후속 회귀 수행/구현 계획서 |
| `mydocs/working/task_m100_1498_v2_stage1.md` | 1단계 완료보고서 |
| `mydocs/report/task_m100_1498_v2_report.md` | 최종 보고서 |
| `mydocs/orders/20260624.md` | #1498 v2 작업 기록 |

## 4. 로컬 검토 결과

구조:

- `devel`은 `local/pr1513`의 조상이다. 즉 PR head는 현재 `devel` 위에 1커밋으로 올라간 상태다.
- GitHub 앱/CLI 기준 작성 시점 merge state는 `CLEAN`이다.
- PR 코멘트는 작성 시점 기준 0건이다.
- 관련 이슈 #1498은 이미 closed 상태다. 이 PR은 #1498 핫픽스의 후속 회귀 보완으로 보는 것이 맞다.

코드 검토:

- 기존 `seen` 게이트는 유지하면서, `onCreated` 자체가 과거 항목일 수 있다는 전제를 반영했다.
- `startTime`이 없거나 파싱 불가한 항목은 호환성을 위해 fresh로 처리한다. 이 선택은 오래된 브라우저/모의 환경 호환성 측면에서 보수적이다.
- `startTime` 또는 `endTime`이 `workerStartedAt - 5초`보다 오래 전이면 무시한다.
- Chrome 쪽은 `processDownloadItem()`에도 최종 fresh 가드를 넣어 `onCreated`와 재조회 경로 양쪽에서 방어한다.
- Firefox 쪽도 동일한 정책을 적용했다.

로컬 명령:

| 명령 | 결과 |
|---|---|
| `node --test rhwp-chrome/sw/download-interceptor.test.mjs` | 통과 |
| `node --check rhwp-chrome/sw/download-interceptor.js` | 통과 |
| `node --check rhwp-firefox/sw/download-interceptor.js` | 통과 |
| `git diff --check devel..local/pr1513` | 통과 |
| `npm run build` (`rhwp-chrome`) | 로컬 의존성 문제로 실패: `canvaskit-wasm` resolve 실패 |
| `npm run build` (`rhwp-firefox`) | 로컬 의존성 문제로 실패: `vite` 미설치 |

빌드 실패는 PR 코드 변경 자체의 실패라기보다 현재 검토 환경의 node_modules 의존성 미설치/불완전 상태로 본다.
GitHub Actions의 Build & Test는 PR head 기준 성공했다.

## 5. 리스크

- `startTime`이 없는 과거 항목은 fresh로 간주된다. 다만 Chrome/Firefox `DownloadItem`에는 통상 `startTime`이 있으므로, 이 선택은 호환성을 위한 fallback으로 판단한다.
- service worker 기동 직전 시작된 정상 다운로드는 5초 grace window 안에서는 허용된다.
- 오래 진행 중이던 다운로드가 service worker 재기동 뒤 완료되는 경우는 신선하지 않은 항목으로 무시된다. 이 PR의 목표가 "기동 이후 새 다운로드만 처리"인 만큼 의도된 제한으로 볼 수 있다.

## 6. 권고

현재 검토 기준으로는 **merge 후보**로 판단했고, 작업지시자 승인 후 merge 완료했다.

merge 전 최종 조건:

- PR head 최신 커밋 기준 GitHub Actions 성공 재확인: 완료
- `mergeable` / `mergeStateStatus` 최신값 재확인: `MERGEABLE` / `CLEAN`
- 작업지시자 승인: 완료
- 필요 시 작업지시자 환경에서 Chrome 확장 0.2.7 재현 케이스가 탭을 열지 않는지 최종 시각/행동 검증

## 7. 처리 결과

- PR 상태: `MERGED`
- merge 시각: 2026-06-24 23:06 KST
- merge commit: `f54756c405d3f43f2c0bcbbc487da0f1e68fe52a`
- merged by: `edwardkim`
- 관련 이슈 #1498: 기존 closed 상태 유지
- `local/devel` 동기화: `origin/devel` fast-forward 완료
