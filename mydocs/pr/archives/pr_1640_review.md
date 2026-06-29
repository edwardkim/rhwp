# PR #1640 처리 보고서 — 브라우저 확장 hover card 타이머 취소 누락 수정 (#1521)

- PR: https://github.com/edwardkim/rhwp/pull/1640
- 제목: `Task #1521: hover card 타이머 취소 누락 수정`
- 작성자: postmelee (collaborator, 30건/20 merged)
- 연결: Refs #1521
- base ← head: `devel` ← `postmelee:task1521-hover-card-timer`
- 처리일: 2026-06-29

## 1. 처리 결정

**admin merge.** Chrome/Firefox/Safari content-script 의 hover 미리보기 카드가 링크를 벗어난
뒤에도 뒤늦게 표시·잔존하던 버그(#1521)를 수정. source content-script 만 변경(런타임 렌더
엔진·SW 무관), 충돌 0 + CI 전부 pass + SW 보안 회귀 통과. MERGEABLE/BEHIND(rebase만 필요).

## 2. BEHIND 착시 배제 (검토 주의점)

`git diff devel pr1640` 단순 비교는 258 files/-13091(테스트·fixture 대량 삭제)로 보이나
**PR base(`a94e2051`, 6/27)가 devel 보다 뒤처진 BEHIND 착시**다. merge-base 기준 PR 의
**실제 순변경은 정확히 10 files +1544/-60**:

| 분류 | 파일 |
|---|---|
| content-script (코어) | `rhwp-{chrome,firefox,safari/src}/content-script.js` (각 ~100줄) |
| 문서 | `mydocs/{orders,plans,report,working}/task_m100_1521*` 7건 |

**dist/ 산출물 변경 없음** — source 만 수정(배포 시 build 재생성, 아래 6).

## 3. 근본 원인 / 수정

원인: hover show 와 hide 를 **단일 `hoverTimeout`** 으로 관리 → 링크 mouseleave 시 pending
show timer 가 안정적으로 취소되지 않아, 이미 벗어난 뒤 `showHoverCard(anchor)` 실행. 일부
타이밍은 stale show 가 이후 hide timer 까지 지워 카드가 안 닫힘.

수정 (3브라우저 동일 패턴, 핵심 식별자 33개 일치):

- 타이머 분리: `showHoverTimeout` + `hideHoverTimeout` (+ `clearShow/HideHoverTimer`).
- `pendingAnchor`/`activeAnchor` 명시 상태화. anchor `mouseenter`→`pendingAnchor=anchor`,
  `mouseleave`→pending 취소 + `clearShowHoverTimer()` 즉시.
- `showHoverCard` 실행 시점 3중 가드: `pendingAnchor===anchor` + `isConnected` +
  `matches(':hover')` → stale show 차단.
- 비동기 썸네일 응답이 stale card 갱신 못 하도록 `activeCard===card && activeAnchor===anchor`.
- card `mouseenter`→hide timer 만 취소(링크→카드 이동 유지), `mouseleave`→hide 예약.

## 4. 검증 (로컬, Linux WSL2)

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas visual diff) | 전부 pass |
| 충돌 | 0건 |
| merge-base 기준 순변경 | 10 files (content-script 3 + 문서 7), dist 0 |
| `node --check` source 3개 | 전부 ✓ |
| `download-interceptor.test.mjs` (chrome/firefox) | 13/13 · 11/11 pass |
| `fetch-security.test.mjs` | passed (SW 보안 회귀 없음) |

## 5. 검토 메모

- postmelee 의 확장 SW 시리즈(#1498/#1515/#1517) 연속 — 확장 content-script 구조 숙지된 author.
- 순수 UX 버그(타이머 race) 수정. 보안 표면(다운로드 인터셉터·fetch policy) 무변경, 회귀 통과.
- 시각/렌더 판정 불필요(렌더 엔진 무관, 확장 UI 동작 한정).

## 6. 후속 — dist 재빌드 (배포 시점)

PR 은 source 만 수정하고 `dist/content-script.js` 는 미변경이라 현재 source≠dist 다. 정상이며
(dist 는 build.mjs 산출물, `project_extension_publicdir_false`), **확장 배포 시 dist 재빌드 +
source 동기 확인**이 필요하다. 본 PR 머지는 source 반영까지이고, 스토어 배포는 별도 빌드 단계.

## 7. 산출물

- 본 처리 보고서: `mydocs/pr/pr_1640_review.md`
