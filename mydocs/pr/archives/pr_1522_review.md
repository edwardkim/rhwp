# PR #1522 처리 보고서 — 확장 GitHub 비파일 HWP URL 후보 차단 (cherry-pick 통합)

- PR: https://github.com/edwardkim/rhwp/pull/1522
- 제목: `Task #1520: 브라우저 확장: GitHub 비파일 HWP URL 후보 차단`
- 작성자: postmelee (collaborator)
- 연결: Closes #1520
- base ← head: `devel` ← `postmelee:local/task1520-upstream`
- 처리일: 2026-06-26

## 1. 처리 결정

**cherry-pick 통합 후 PR close.** GitHub의 비파일 HWP URL(`edit`/`commits`/`blame`/`tree`)에
배지·hover 가 생성되던 오탐을 정정한다. #1517/#1519 머지 후 본 PR 이 orders 문서 add/add
충돌로 CONFLICTING(소스 무충돌)이 되어, 단일 신규 커밋만 cherry-pick 통합한다.

## 2. 충돌 원인

#1520 신규 커밋(`c0929572`)은 단일 커밋이며 stacked 중복이 없다. 충돌은 오늘할일
(orders) add/add 1건뿐이고, 소스(content script·document-url-resolver)는 충돌 없이 적용된다.

## 3. 통합 내용

devel 위 cherry-pick 1커밋 (작성자 보존):
- `Task #1520: GitHub 비파일 HWP URL 후보 차단`

핵심:
- `rhwp-shared/sw/document-url-resolver.js` 정적 URL classifier:
  - GitHub `blob` HWP/HWPX, `raw.githubusercontent.com` 직접 URL → `openable`
  - GitHub `edit`/`commits`/`blame`/`tree` → `not-document` (명시 차단)
  - 일반 직접 `.hwp/.hwpx` pathname → 허용, query 문자열만 매치 → 제외
- chrome/firefox/safari content script 의 배지/hover/prefetch 후보 판정을 pathname 기반으로
  좁힘. `data-hwp="true"` 명시 링크 정책 유지.

orders 충돌 해소: #1516 / #1520 항목 양쪽 보존.

## 4. 검증 (로컬)

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze rust·js·python) | 전부 pass |
| `document-url-resolver.test.js` (GitHub classifier) | 23 passed |
| `download-interceptor-common.test.js` (회귀) | 26 passed |
| `fetch-security.test.mjs` | passed |
| content script 구문 (chrome/firefox/safari) | OK |
| chrome/firefox 빌드 | 통과 |

## 5. 의의

- GitHub provider 페이지(편집/이력/blame/디렉터리)에서 배지·hover 오탐 제거.
- provider 구조를 아는 경우에만 명시 차단(보수적). 일반 직접 HWP URL 동작은 유지.
- chrome/firefox/safari 3종 content script 일관 적용.

## 6. 후속

- #1521 (hover lifecycle 버그)는 PR 본문에서 별도 후속으로 분리 명시.

## 7. 산출물

- 본 처리 보고서: `mydocs/pr/archives/pr_1522_review.md`
