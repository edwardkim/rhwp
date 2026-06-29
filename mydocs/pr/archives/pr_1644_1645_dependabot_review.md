# Dependabot PR #1644 / #1645 처리 보고서 — rhwp-studio devDep 업데이트

- PR: https://github.com/edwardkim/rhwp/pull/1644 (vite), https://github.com/edwardkim/rhwp/pull/1645 (puppeteer-core)
- 작성자: dependabot[bot]
- base: `devel`
- 처리일: 2026-06-29

## 1. 처리 결정

**둘 다 merge.** `rhwp-studio` devDependencies 의 MINOR/PATCH 업데이트. 런타임·배포 산출물
미포함(빌드도구·e2e 테스트 도구), CI 전부 pass, 로컬 install/build/e2e 검증 통과, breaking 없음.

## 2. 대상

| PR | 업데이트 | 종류 | 용도 | 비고 |
|---|---|---|---|---|
| #1644 | vite 8.0.16 → **8.1.0** (vite-stack 그룹) | MINOR | studio 빌드 | devDep |
| #1645 | puppeteer-core 25.1.0 → **25.2.1** | PATCH | e2e 테스트 | untrusted-session regression fix + Firefox 152 roll 포함(개선) |

둘 다 변경 파일은 `rhwp-studio/{package.json, package-lock.json}` 2개뿐(코드 0).
`vite`/`puppeteer-core` 모두 `devDependencies` 확인(prodDep 아님 → 배포 번들 무영향).

## 3. 로컬 검증

| 항목 | #1644 (vite) | #1645 (puppeteer) |
|---|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas) | 전부 pass | 전부 pass |
| `npm ci` (lockfile 정합) | exit 0 | added 380 pkgs OK |
| 설치 버전 확인 | vite/8.1.0 | puppeteer-core 25.2.1 |
| `npm run build` (studio) | ✓ built 646ms, PWA 정상 | — |
| e2e `node --check` (전 파일) | — | 전부 ✓ |

## 4. 메모

- dependabot.yml 은 cargo(그룹: resvg-stack) + npm 을 devel 타겟 주간 관리. 본 2건은 npm studio.
- MINOR/PATCH + devDep + CI green 이라 저위험. puppeteer 25.2.1 의 untrusted-session fix 는
  오히려 e2e 안정성 개선.
- 처리 방식: 내부 타스크의 stage 절차 불필요(외부 의존성 bump). merge + PR 코멘트.

## 5. 산출물

- 본 처리 보고서: `mydocs/pr/pr_1644_1645_dependabot_review.md`
