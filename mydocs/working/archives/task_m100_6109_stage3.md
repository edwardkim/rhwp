---
kind: working
status: complete
issue: 6109
stage: 3
last_verified: 2026-08-28
---

# Task M100 #6109 Stage 3 완료 보고 — 실제 Chrome transaction과 통합 회귀

- **이슈**: [#6109](https://github.com/edwardkim/rhwp/issues/6109)
- **PR**: [#6290](https://github.com/edwardkim/rhwp/pull/6290)
- **브랜치**: `codex/issue-6109-zoom-dialog-transaction`
- **stack base**: PR #6289 `codex/issue-6108-zoom-fit@711365b35`
- **선행 Stage**: Stage 1 사용자 입력 검증, Stage 2 원자 보기 설정 transaction

## 실제 Chrome 회귀 계약

`zoom-dialog-transaction.test.mjs`를 공용 `run-with-vite.mjs`와 macOS Chrome headless로 실행했다.

- 빈 값·9%·501%는 대화상자를 유지하고 alert·ARIA·focus를 복원한다.
- invalid 제출은 배율·보기 이벤트·레이아웃을 모두 0회 유지한다.
- 밝은·어두운 테마에서 입력 테두리와 오류 문구가 `--ui-danger-strong` 계산값을 공유한다.
- 501%를 137%로 교정하면 오류와 `aria-invalid`를 해제한다.
- 두 쪽·세로 이동·휠 좌우·137%를 한 번에 적용하면 page-view 1회, zoom 1회, recalc 1회다.
- 유일한 recalc 시점에는 배치·이동·배율이 모두 최종 상태다.
- Escape와 취소 버튼은 상태를 바꾸지 않고, 10%·500% 경계는 정확히 적용한다.

총 37개 assertion이 모두 통과했다. 로컬 HTML 보고서는
`output/e2e/zoom-dialog-transaction-report.html`, 오류 화면은
`rhwp-studio/e2e/screenshots/issue-6109-invalid-custom-zoom.png`에 생성했다. 둘 다 재현 가능한 gitignore
산출물이며 source PR에는 포함하지 않는다.

## stack 재기반화 보정 확인

하위 PR #6289 review에서 여러 쪽이 실제 배율만 계산하고 맞춤 규칙을 `none`으로 저장하는 결함을
확인했다. 하위 head의 `resolveZoomDialogFitMode()`를 상위 transaction에도 연결해 여러 쪽은 최종
`fitPage`를 같은 snapshot과 event payload에 담는다.

상위 head에서 하위 Chrome E2E도 다시 실행해 2×2 `fitPage` 저장, 새 세션 배치·규칙 복원과 다른 쪽
크기의 `0.446` 재계산을 포함한 32/32 assertion을 통과했다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| Studio 전체 `npm test` | 1,235 tests, 1,234 pass / 1 skip / 0 fail |
| `npm run build` | TypeScript·Vite production build 통과, 239 modules |
| Chrome `e2e:zoom-dialog-transaction` | 37/37 assertion 통과 |
| Chrome `e2e:zoom-fit-mode` | 32/32 assertion 통과 |
| `npm run e2e:manifest-check` | tracked 120 / manifest 120, 이상 없음 |
| `git diff --check` | 통과 |

E2E 원장은 신규 #6109 여정을 추가할 때 stack base에 이미 tracked됐지만 누락돼 있던
`issue-4969-shaping-replay.test.mjs`, `issue-6117-cell-underline-canvas2d.test.mjs`도 함께 등재해야 양방향
완전성 검사를 통과했다. 제품 범위를 넓힌 것이 아니라 기존 원장 표류를 함께 해소한 것임을 PR 본문에
명시한다.

## 범위와 인계

- 이 Stage는 입력 오류 UX와 원자 보기 transaction의 실제 브라우저 검증만 다룬다.
- slider·pinch preview, 적응형 render scale, 페이지 가상화는 #6040·#6041·#6042에 유지한다.
- 최종 stack 제출 전 파생 Rust suite 준비와 `cargo fmt --all -- --check`를 실행한다.
- GitHub Actions 완료 확인과 merge는 작업지시자에게 인계한다.
