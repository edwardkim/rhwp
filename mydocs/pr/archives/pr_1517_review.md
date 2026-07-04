# PR #1517 처리 보고서 — 다운로드 관찰자 상태 머신 도입 (Chrome)

- PR: https://github.com/edwardkim/rhwp/pull/1517
- 제목: `Task #1515: Add shared download observer state machine`
- 작성자: postmelee (collaborator)
- 연결: Closes #1515, Refs #1471/#1480/#1498/#1513/#1516
- base ← head: `devel` ← `postmelee:local/task1515-download-state-machine`
- 처리일: 2026-06-26

## 1. 처리 결정

**admin merge.** #1498/#1513(다운로드 관찰자 과거 항목 미오픈)의 구조 개선 후속.
관찰자 흐름을 명시적 상태 머신으로 정식화하고 MV3 service worker 재시작 한계를 해결한다.
CI 전부 pass + 로컬 검증 통과 + 충돌 0건.

## 2. 변경 범위

| 파일 | 내용 |
|---|---|
| `rhwp-shared/sw/download-observer-state.js` | 브라우저 API 비의존 상태 머신 (evaluateDownloadCreated/Changed, markHandled/Terminal, TTL/fresh 판정) |
| `rhwp-chrome/sw/download-observer-state.js` | chrome adapter |
| `rhwp-chrome/sw/download-interceptor.js` | 메모리 seen/handled/workerStartedAt 제거 → `chrome.storage.session` TTL 저장 기반 흐름 |
| `rhwp-chrome/sw/download-interceptor.test.mjs` | session mock + SW 재시작 시뮬레이션 |
| `rhwp-shared/sw/download-observer-state.test.js` | 상태 머신 단위 테스트 |
| 문서 6건 | #1515 계획/보고서 |

## 3. 검토 결과

- #1498(seen) + v2(#1513 startTime fresh 가드)를 상태 머신으로 통합. `DEFAULT_FRESH_GRACE_MS`(5s),
  `DEFAULT_STATE_TTL_MS`(10m) 로 과거 항목 무시·재오픈 방지·TTL 정리 일원화.
- **MV3 한계 해결**: 메모리 Set → `chrome.storage.session` 영속(+ 메모리 fallback) → SW 재시작
  후에도 handled 상태 유지 → 중복 open 방지.
- **#1471 원인 미재발**: `onDeterminingFilename.addListener` 재도입 없음 (dist/sw 확인).
- 범위 격리: 확장 SW 4파일 + 테스트. WASM·공통 판정(shouldInterceptDownload) 무변경.
- autoOpen=false / file:// cancel·erase / 대용량 경고 / openViewer 동작 보존.

## 4. 검증 (로컬)

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze rust·js·python) | 전부 pass |
| 충돌 시뮬레이션 (`merge-tree`) | 0건 |
| `download-observer-state.test.js` (shared) | 14 passed |
| `download-interceptor.test.mjs` (chrome, SW 재시작 포함) | 13 passed |
| `download-interceptor-common.test.js` (공통 회귀) | 26 passed |
| 구문 체크 / chrome 빌드 / dist 내 observer-state 포함 | OK |
| dist/sw 내 `onDeterminingFilename.addListener` | 없음 |

## 5. 시리즈 순서

- #1519(Task #1516, "Depends on #1517") 와 #1522(Task #1520) 가 후속.
- 본 PR(#1517)을 먼저 머지해야 #1519/#1522 처리가 성립한다. 순서 정합.

## 6. 산출물

- 본 처리 보고서: `mydocs/pr/archives/pr_1517_review.md`
