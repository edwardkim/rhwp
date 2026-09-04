---
kind: report
status: completed
canonical: mydocs/plans/archives/task_m100_3514.md
last_verified: 2026-08-20
---

# Task M100 #3514 Stage 2 — packaged extension smoke 구현

- Issue: [#3514](https://github.com/edwardkim/rhwp/issues/3514)
- Stage 1 commit: `081a44af9`
- 기준 브랜치: `codex/issue-3514-extension-smoke`

## 구현 범위

`rhwp-chrome/dist`를 Puppeteer가 동반하는 Chrome for Testing에 실제 unpacked extension으로
설치하는 smoke 명령을 추가했다.

- `test:e2e:smoke`가 production extension build 뒤 harness를 실행한다.
- package lock으로 Puppeteer와 호환 Chrome 버전을 고정한다.
- 실행마다 격리된 profile·download 경로와 loopback fixture/proxy를 만든다.
- MV3 service worker URL에서 extension ID를 동적으로 얻고 Runtime·Log·Network 오류를 수집한다.
- viewer의 실제 HWP3 canvas·최종 파일명, 다크 SVG, options hydration, same-origin print surface,
  content-script HWP 배지 1개를 확인한다.
- 기존 `fetch-file` 메시지의 loopback 정책 거부 응답으로 background listener와 SSRF 방어 배선을
  확인한다.
- 외부 HTTP(S), 로컬 자산 4xx/5xx, page/worker 오류, 추가 탭과 정리 실패를 테스트 실패로 처리한다.

프로덕션 manifest 권한·CSP와 테스트 전용 런타임 hook은 변경하지 않았다.

## 절차 복구 경계

이 구현은 승인 게이트 누락 전에 이미 작업트리에 작성됐다는 사실을 보존한다. Stage 1 커밋 뒤
구현 세 파일만 다시 검토·검증했으며, 매뉴얼과 최종 보고서는 Stage 3까지 unstaged로 둔다.

## Focused 검증

| 게이트 | 결과 |
|---|---|
| `node --check rhwp-chrome/e2e/extension-smoke.test.mjs` | 통과 |
| Chrome·Firefox 확장 Node/dist 계약 | 85/85 통과 |
| `npm --prefix rhwp-chrome run test:e2e:smoke` | production build + 실제 package smoke 1/1 통과 |

실제 smoke는 새 임시 Chrome profile에서 viewer/options/print/service worker/content script를 모두
통과했고 자동 retry를 사용하지 않았다.

## Stage 2 승인 대상

- `rhwp-chrome/package.json`
- `rhwp-chrome/package-lock.json`
- `rhwp-chrome/e2e/extension-smoke.test.mjs`
- `mydocs/orders/20260820.md`
- `mydocs/plans/task_m100_3514.md`
- `mydocs/plans/task_m100_3514_impl.md`
- `mydocs/working/task_m100_3514_stage2.md`

Stage 2 승인 전에는 구현 커밋과 Stage 3 staging을 진행하지 않는다. 전체 Studio 회귀, Firefox
재빌드와 새 profile 10회 smoke는 Stage 2 커밋을 기준으로 Stage 3에서 다시 실행한다.

## 승인 결과

작업지시자는 2026-08-20 22:18 KST에 “진행해줘”로 Stage 2 구현 diff와 focused 검증 결과를
승인했다. 구현 커밋 뒤 Stage 3 전체 재검증과 최종 문서 경계로 이동한다.
