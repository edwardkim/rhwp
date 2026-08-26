---
kind: report
status: completed
canonical: mydocs/plans/task_m100_3514.md
last_verified: 2026-08-22
---

# Task M100 #3514 Stage 7 — fixture proxy client abort 안전 처리

## 1. 발생 조건

최신 `upstream/devel@65f71270f`에서 실행한 Chrome 10회 gate는 build와 page-budget 2건 뒤 첫 profile의
제품 surface assertion 전에 `read ECONNRESET`으로 종료됐다. loopback fixture 서버는 외부 HTTPS를
연결하지 않도록 CONNECT 요청에 502를 쓰지만, Chrome이 응답을 다 읽기 전에 socket을 닫을 수 있다.
기존 harness는 해당 socket에 `error` listener를 두지 않아 정상 client abort도 처리되지 않은 프로세스
오류가 됐다.

## 2. 변경

- CONNECT 차단 처리를 `rejectProxyConnect()`로 분리했다.
- 차단 응답을 쓰기 전에 socket `error` listener를 설치한다.
- `ECONNRESET`·`EPIPE`는 정상 client abort로 `proxyClientAborts` 진단에 남긴다.
- 그 밖의 socket 오류는 `[fixture-proxy]` 오류로 보존해 최종 smoke 판정을 실패시킨다.
- Node 계약으로 listener 순서, 정상 abort 흡수, 예상 밖 오류 보존을 고정했다.

제품 extension 코드·권한·CSP·package 의존성은 바꾸지 않았다.

## 3. 검증

| 영역 | 명령 | 결과 |
|---|---|---|
| 구문 | `node --check rhwp-chrome/e2e/extension-smoke.test.mjs` | 통과 |
| focused contract | `node --test rhwp-chrome/e2e/page-budget.test.mjs` | 4/4 통과 |
| extension Node 전체 | Chrome·Firefox shared/worker/options/page-budget 계약 | 134/134 통과 |
| actual Chrome | `RHWP_EXTENSION_SMOKE_REPEAT=10 npm --prefix rhwp-chrome run test:e2e:smoke` | build 1회, 새 profile 10개, retry 없이 10/10 통과 |
| dist | `node --test scripts/frontend-extension-dist.test.mjs` | 3/3 통과 |

Stage 6 실패 뒤 같은 code를 재시도한 것이 아니다. Stage 7 변경을 적용한 새 code head에서 10회 명령을
처음부터 한 번 실행했고, 명령 내부의 10개 격리 profile이 모두 첫 시도에 통과했다.

## 4. 판정

최신 upstream 반영으로 드러난 fixture proxy 수명주기 결함을 좁은 harness 변경과 결정적 계약으로
해소했다. #3514의 핵심 surface, 탭 예산, 격리 profile, 외부망 차단, 10회 무재시도 완료 조건을 다시
충족한다. Stage 7 변경과 이 보고를 별도 커밋한 뒤 승인된 remote push와 Open PR 게시를 진행한다.
