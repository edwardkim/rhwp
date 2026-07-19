# PR #2398 리뷰 — custom scheme 최상위 legacy 요청 복구

## 메타데이터

| 항목 | 값 |
|---|---|
| PR | [#2398](https://github.com/edwardkim/rhwp/pull/2398) |
| 관련 이슈 | [#2396](https://github.com/edwardkim/rhwp/issues/2396) |
| 작성자 | `postmelee` |
| base / head | `devel` / `codex/issue-2396-custom-scheme-legacy-request` |
| 리뷰 경로 | collaborator self-merge 후보 |
| merge 직전 통합 | `upstream/devel@2d1aa588`, head `371a31a3` |
| 최종 결과 | merge commit `7a64a7ce`, #2396 자동 종료 |

## 변경 범위

- custom URL scheme으로 로드된 top-level same-window 문서에서 legacy `rhwp-request`만 허용한다.
- custom scheme top-level v1 `rhwp-connect`, iframe parent, forged sibling은 계속 거부한다.
- 거부된 MessageChannel port 정리와 이후 legacy 요청 복구를 회귀 테스트로 고정한다.
- protocol version, public API, renderer, WASM 구현은 변경하지 않는다.

## 리뷰 발견과 보정

최초 구현은 top-level same-window라는 조건만으로 unusable origin 예외를 적용해 custom scheme v1
`rhwp-connect`까지 허용했다. 이는 #2396의 의도인 legacy 호환 복구보다 넓은 권한이었다.

보정 커밋 `5e25b0b0`에서 예외 조건을 `event.data?.type === 'rhwp-request'`까지 제한했다. 추가 테스트는
custom scheme v1 connect가 binding을 만들지 않고 port를 닫는지, 그 거부 뒤에도 legacy `ready`가 정상
응답하는지를 함께 검증한다.

## 검증

| 게이트 | 결과 |
|---|---|
| focused embed protocol | 15/15 PASS |
| Studio + editor Node tests | 365/365 PASS |
| dev WASM package 생성 | PASS |
| Studio TypeScript + Vite production build | PASS |
| `git diff --check` | PASS |
| GitHub Actions (`5e25b0b0`) | CI, CodeQL, Render Diff PASS |
| GitHub Actions (`371a31a3`) | CI, CodeQL, Render Diff relevant checks 전체 PASS |
| 최신 `devel` 통합 | 충돌 없이 완료 |

## 시각 검증 판단

renderer, DOM/CSS, 페이지 출력과 UI를 변경하지 않는 message gate 수정이므로 별도 visual sweep 대상이 아니다.

## 잔여 위험

- 실제 macOS WKWebView downstream representative suite는 이 저장소 환경에서 별도로 실행하지 않았다.
- custom scheme v1 MessageChannel connect와 iframe custom scheme origin 허용은 이번 변경 범위가 아니다.

## 최종 의견

코드 리뷰에서 발견한 권한 범위 확대를 보정했고, 회귀 테스트·로컬 검증·최종 PR head relevant checks가
통과했다. 작업지시자 승인 뒤 PR #2398을 merge했으며 #2396 자동 종료와 후속 comment를 확인했다.

## Merge 후 확인

- merge commit: [`7a64a7ce`](https://github.com/edwardkim/rhwp/commit/7a64a7cef977f157893dd89cfd66d82c0d40e99a)
- PR merge comment:
  [issuecomment-5014098546](https://github.com/edwardkim/rhwp/pull/2398#issuecomment-5014098546)
- 이슈 후속 comment:
  [issuecomment-5014097249](https://github.com/edwardkim/rhwp/issues/2396#issuecomment-5014097249)
- 별도 코드·테스트·시각 검증 후속 작업은 없다.
