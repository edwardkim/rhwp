# PR #1519 처리 보고서 — 다운로드 상태 머신 Firefox 적용 (cherry-pick 통합)

- PR: https://github.com/edwardkim/rhwp/pull/1519
- 제목: `Task #1516: Apply common download state machine to Firefox`
- 작성자: postmelee (collaborator)
- 연결: Depends on #1517(머지됨), Closes #1516
- base ← head: `devel` ← `postmelee:local/task1516-firefox-download-state-machine`
- 처리일: 2026-06-26

## 1. 처리 결정

**cherry-pick 통합 후 PR close.** #1517(shared 상태 머신) 머지 후 PR #1519가
`mydocs/orders/20260625.md` add/add 충돌로 CONFLICTING/DIRTY 상태가 되었다(소스 무충돌).
#1516 신규 firefox 3커밋만 devel 위로 cherry-pick 하여 작성자(postmelee)를 보존하고
shared 중복 없이 깔끔하게 통합한다.

## 2. 충돌 원인

#1519 브랜치는 #1515 shared 상태 머신 커밋을 함께 포함했는데, #1517 이 먼저 devel 에
머지되며 동일 영역이 겹쳤다. 실제 충돌은 오늘할일 문서(orders) add/add 1건뿐이고,
firefox 소스/테스트는 충돌 없이 적용되었다. PR 본문도 이 상황을 예고했다.

## 3. 통합 내용

devel 위 cherry-pick 3커밋 (작성자 보존):

- `Task #1516: Add Firefox state machine plan`
- `Task #1516: Apply download state machine to Firefox`
- `Task #1516: Record Firefox validation`

통합 범위 = firefox 3파일 + 문서 5개 (shared/chrome 은 #1517 로 이미 devel 반영, 중복 0):

- `rhwp-firefox/sw/download-interceptor.js` — 메모리 seen/handled/workerStartedAt 제거 →
  `rhwp-shared/sw/download-observer-state.js` 공통 상태 머신 + `browser.storage.session` TTL
  (없으면 메모리 fallback). chrome 과 동형.
- `rhwp-firefox/sw/download-observer-state.js` — firefox adapter.
- `rhwp-firefox/sw/download-interceptor.test.mjs` — event page 재시작 시뮬레이션 회귀 테스트.

orders 충돌 해소: #1510 / #1516 항목 양쪽 보존.

## 4. 검증 (로컬)

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze rust·js·python) | 전부 pass (충돌 전 기준) |
| `download-interceptor.test.mjs` (firefox, event page 재시작 포함) | 11 passed |
| `download-observer-state.test.js` (shared 회귀) | 14 passed |
| `download-interceptor.test.mjs` (chrome 회귀) | 13 passed |
| firefox 구문 / 빌드 / dist 내 observer-state 포함 | OK |
| firefox dist/sw 내 `onDeterminingFilename.addListener` | 없음 |

## 5. 의의

- chrome(#1515/#1517)과 firefox 의 다운로드 관찰자가 동일한 공통 상태 머신을 공유.
- event page unload/reload 시 상태 소실로 인한 자동 열기 누락·중복 탭·과거 항목 재오픈을
  구조적으로 방지.

## 6. 산출물

- 본 처리 보고서: `mydocs/pr/archives/pr_1519_review.md`
