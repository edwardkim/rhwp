# 최종 보고 — #2396 custom scheme 최상위 legacy 요청 회귀

- 이슈: #2396
- assignee: `postmelee`
- 브랜치: `codex/issue-2396-custom-scheme-legacy-request`
- 최초 기준선: `upstream/devel@af5902b6`
- merge 직전 head: `371a31a3` (`upstream/devel@2d1aa588` 통합)
- merge commit: `7a64a7ce`
- 수행계획서: `mydocs/plans/task_m100_2396.md`
- 구현계획서: `mydocs/plans/task_m100_2396_impl.md`

## 결론

WKWebView custom URL scheme으로 `rhwp-studio`를 최상위 문서에 로드했을 때 legacy
`rhwp-request`가 origin 검사에서 폐기되어 timeout되는 회귀를 수정했다.

HTTP(S)가 아닌 origin을 일반적으로 허용하지 않고, `parentWindow === hostWindow`인 top-level
same-window 환경의 legacy `rhwp-request`만 window identity와 message type을 근거로 예외 처리했다.
`event.source === parentWindow` 검사는 모든 환경에서 유지하고, v1 `rhwp-connect`는 custom scheme
top-level에서도 기존 HTTP(S) origin 계약을 유지하므로 iframe parent와 forged sibling 요청도 계속 거부된다.

## 계획 대비 결과

| 단계 | 계획 | 결과 | 커밋 |
|---|---|---|---|
| 계획 | 수행·구현계획과 승인 게이트 | 완료 | `af5e7f19` |
| Stage 1 | positive 회귀 재현, negative 보안 기준선 | 13/14, 의도한 timeout 재현 | `41128eed` |
| Stage 2 | runtime gate 최소 수정, focused 검증 | 14/14 PASS | `11af6aa2` |
| Stage 3 | 전체 frontend test/build | 전체 PASS | `33c17cc1` |
| Stage 4 | 최종 보고와 PR 준비 | 완료 | 이 문서와 PR 초안 |

계획한 파일과 조건만 변경했으며 protocol version, public API, renderer와 WASM 구현에는 변경이 없다.

## 구현

`rhwp-studio/src/embed/runtime.ts`의 runtime 설치 시 top-level identity를 계산한다.

```ts
const isTopLevelSameWindow = options.parentWindow === options.hostWindow;
```

초기 gate는 다음 두 경우만 거부한다.

1. message source가 parent window와 다른 경우
2. origin이 usable HTTP(S)가 아니면서 top-level same-window legacy `rhwp-request`도 아닌 경우

기존 `isUsableParentOrigin()`과 connect/session/port binding, legacy handler는 변경하지 않았다.

## 회귀 계약

`rhwp-studio/tests/embed-protocol.test.ts`에 다음 세 테스트를 추가했다.

- custom scheme top-level same-window legacy `ready` 요청이 같은 custom origin으로 응답한다.
- custom scheme top-level same-window v1 connect는 port를 정리하고 binding을 설치하지 않으며, 이후 legacy
  `ready` 요청은 계속 응답한다.
- custom scheme iframe parent와 forged sibling은 handler에 도달하지 않고 transferred port를 정리한다.

Stage 1에서 첫 테스트의 timeout을 재현했고, PR review 보정 후 세 테스트를 포함한 focused 15건이 모두
통과했다.

## 최종 검증

| 게이트 | 결과 |
|---|---|
| Studio + editor Node tests | 365/365 PASS |
| focused embed protocol | 15/15 PASS |
| dev WASM package 생성 | PASS |
| Studio TypeScript + Vite production build | PASS |
| `git diff --check` | PASS |
| 추적 파일 범위 | 운영·계획·보고·runtime·test만 포함 |

`pkg/`, `target/`, `rhwp-studio/node_modules/`, `rhwp-studio/dist/`는 gitignored 생성물로 유지했다.

## 잔여 범위와 위험

- 실제 macOS WKWebView downstream representative suite는 이 저장소 환경에서 실행하지 않았다.
- custom scheme v1 MessageChannel connect, iframe custom scheme origin 허용, custom scheme allowlist,
  host 인증 정책은 이번 범위에 포함하지 않는다.
- Vite chunk-size/browser externalization과 wasm-pack prebuilt fallback 경고는 기존 비차단 build 경고다.
- #2396은 PR merge 뒤 closing keyword 자동 처리로 종료됐다.

## 완료 판단

수행계획서의 기능·보안·로컬 검증 완료 기준을 충족했다. PR review에서 발견한 v1 connect 권한 범위
확대도 `5e25b0b0`에서 보정했다. 최종 head `371a31a3`의 relevant checks 통과와 작업지시자 승인 뒤
PR #2398을 merge했고, #2396 자동 종료와 후속 검증 기록까지 확인했다. PR 본문 초안은
`mydocs/report/task_m100_2396_pr_draft.md`, 리뷰 기록은 `mydocs/pr/archives/pr_2398_review.md`에 둔다.

## PR 처리 결과

- Draft PR: [#2398](https://github.com/edwardkim/rhwp/pull/2398)
- base / head: `edwardkim/rhwp:devel` / `postmelee:codex/issue-2396-custom-scheme-legacy-request`
- 생성 직후 상태: open, draft, mergeable
- 생성 직후 CI: CI, Render Diff, CodeQL workflow가 runner 배정 전 일괄 실패
- 공통 annotation: `The job was not started because your account is locked due to a billing issue.`

초기 CI 실패는 job step이 하나도 실행되지 않고 `runner_id=0`인 GitHub 계정 billing blocker였다. 이후
billing lock 해소와 `devel` 통합 head `c8bc5644`에서 CI, Render Diff, CodeQL이 통과했다. PR review 보정
커밋 `5e25b0b0`에서도 CI, Render Diff, CodeQL이 모두 통과했다.

- 최종 head `371a31a3`: CI, Render Diff, CodeQL relevant checks 전체 통과
- merge: [PR #2398](https://github.com/edwardkim/rhwp/pull/2398), commit
  [`7a64a7ce`](https://github.com/edwardkim/rhwp/commit/7a64a7cef977f157893dd89cfd66d82c0d40e99a)
- 이슈: #2396 closing keyword 자동 종료 확인
- PR merge comment:
  [issuecomment-5014098546](https://github.com/edwardkim/rhwp/pull/2398#issuecomment-5014098546)
- 이슈 후속 comment:
  [issuecomment-5014097249](https://github.com/edwardkim/rhwp/issues/2396#issuecomment-5014097249)
