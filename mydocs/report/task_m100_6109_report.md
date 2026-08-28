# Task M100 #6109 최종 보고 — 사용자 배율 검증과 원자 보기 설정

- **이슈**: [#6109](https://github.com/edwardkim/rhwp/issues/6109)
- **브랜치**: `codex/issue-6109-zoom-dialog-transaction`
- **stack base**: `codex/issue-6108-zoom-fit` `711365b35`
- **보고일**: 2026-08-28 KST
- **결과 상태**: Stage 1~3 구현·검증 및 최종 결과 승인 완료

## 완료 결과

확대/축소 대화상자의 사용자 정의 배율을 제출 전에 검증하고, 쪽 배치·쪽 이동·맞춤 모드·최종 배율을
하나의 보기 설정 transaction으로 적용한다.

- 빈 값, 숫자가 아닌 값, 정수가 아닌 값, 10% 미만, 500% 초과는 callback이나 보기 변경 없이
  대화상자 안에서 설명한다.
- 오류 입력은 안정된 `role="alert"`와 연결하고 `aria-invalid`를 설정하며 focus/select를 복원한다.
- 오류 입력 테두리와 문구는 고정 빨강 fallback이 아니라 밝은·어두운 테마가 제공하는
  `--ui-danger-strong` 의미 토큰을 공유한다.
- 유효한 값으로 교정하면 오류 상태를 해제한다. 사용자 정의 입력의 Enter는 설정, Escape와 취소 버튼은
  무변경 종료로 동작한다.
- 쪽 배치·쪽 이동·맞춤 모드는 localStorage에 한 번의 정규화된 snapshot으로 저장된다.
- 대화상자는 최종 배치·이동·배율·anchor를 담은 `page-view-settings-changed`를 한 번 발행한다.
- 표준 `zoom-changed`는 한 번 유지해 눈금자·커서 등 기존 소비자를 갱신한다.
- CanvasView 자신의 중첩 zoom handler만 억제하고, 유일한 `recalcLayout()`은 이미 최종 배치·이동·배율이
  반영된 상태에서 실행한다.
- 기존 zoom 없는 page-view payload와 개별 사용자 설정 setter는 호환 경로를 유지한다.

## 단계별 checkpoint

| 단계 | 결과 | checkpoint |
| --- | --- | --- |
| 계획 | 사용자 배율·보기 transaction의 범위와 게이트 확정 | `bcbdcc75c` |
| Stage 1 | validator, 오류 ARIA·focus, Enter/Escape 계약 | `0d9c26972` |
| Stage 2 | 설정 저장·event payload·CanvasView 최종 layout 원자화 | `03c412bf5` |
| Stage 3 | 실제 Chrome 회귀, E2E 원장, 전체 Studio·format gate | 본 checkpoint commit |

## Stage 3 실제 Chrome 검증

새 `zoom-dialog-transaction.test.mjs`를 공용 `run-with-vite.mjs`로 실행했다. 설치된 macOS Chrome을
headless로 기동하고 빈 문서 한 쪽에서 실제 대화상자 DOM과 CanvasView 인스턴스를 계측했다.

| 시나리오 | 확인 결과 |
| --- | --- |
| 빈 값·9%·501% | dialog 유지, alert·ARIA·focus, zoom 무변경, page-view/zoom event·recalc 각 0회 |
| 밝은·어두운 테마 오류 색 | 입력 테두리·오류 문구가 각 테마의 `--ui-danger-strong` 계산값과 일치 |
| invalid 501% → 137% 교정 | alert와 `aria-invalid` 해제 |
| 두 쪽+세로 이동+휠 좌우+137%, Enter | page-view 1회, zoom 1회, recalc 1회 |
| recalc 순간 snapshot | 두 쪽·휠 좌우·137%가 모두 이미 반영된 최종 상태 |
| Escape·취소 | zoom·event·recalc 무변경 |
| 10%·500% 경계 | 정확한 배율, 각 page-view 1회·zoom 1회·recalc 1회 |

총 37개 assertion이 모두 통과했다. `input[type=number]`의 브라우저 sanitization상 비숫자 문자열은 실제
UI에서 빈 문자열로 수렴하며, 순수 validator 테스트는 비숫자 입력을 별도 경로로 검증한다.

실행 중 생성한 HTML 보고서와 오류 UI 스크린샷은 각각
`output/e2e/zoom-dialog-transaction-report.html`,
`rhwp-studio/e2e/screenshots/issue-6109-invalid-custom-zoom.png`에 있으며 둘 다 로컬 검증 증적이라
커밋하지 않는다.

## 전체 검증

| 명령 | 결과 |
| --- | --- |
| `npm run e2e:zoom-dialog-transaction` | 37/37 assertion 통과 |
| `npm test` | 1,235 tests: 1,234 pass, 1 skip, 0 fail |
| `npm run build` | TypeScript·Vite production build, 239 modules 통과 |
| `npm run e2e:zoom-fit-mode` | 하위 맞춤 저장·복원 32/32 assertion 통과 |
| `node --check e2e/zoom-dialog-transaction.test.mjs` | 통과 |
| `python3 scripts/check_e2e_manifest.py` | tracked 120 / manifest 120, 이상 없음 |
| `node scripts/rust-test-suite-manifest.mjs --prepare` | 995 sources·48/48 integration targets 준비 완료 |
| `cargo fmt --all` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |

E2E 원장 검사에서 stack base에 이미 tracked된 `issue-4969-shaping-replay.test.mjs`와
`issue-6117-cell-underline-canvas2d.test.mjs`의 누락 행도 발견했다. 이번 신규 E2E를 등재하면 원장 검사가
양방향 완전성을 요구하므로, 두 파일의 실제 용도·fixture·배선을 확인해 원장 행만 함께 보완했다.

Rust format 검사용 파생 suite와 manifest는 검증 뒤 정확한 경로를 작업트리 밖
`/private/tmp/rhwp-6109-stage3-token-generated`,
`/private/tmp/rhwp-6109-stage3-token-manifest.json`으로 이동했다.
source PR에는 포함하지 않는다.

게시 승인 뒤 공식 `gh-stack`에 기존 #6108·#6109 브랜치를 등록했다. PR review 보정 뒤 최신
`upstream/devel@94ff48d2b8` 및 #6108 self-review head `711365b35` 위로 cascading rebase했다. 위 전체
검증표와 Chrome 37/37·하위 회귀 32/32 결과는 해당 최상단 stack diff에서 다시 실행한 값이다.

## 변경 범위

### 제품 코드

- `rhwp-studio/src/view/zoom-dialog-state.ts`
- `rhwp-studio/src/ui/zoom-dialog.ts`
- `rhwp-studio/src/styles/zoom-dialog.css`
- `rhwp-studio/src/core/user-settings.ts`
- `rhwp-studio/src/command/commands/view.ts`
- `rhwp-studio/src/view/page-view-settings-change.ts`
- `rhwp-studio/src/view/canvas-view.ts`

### 검증·문서

- 사용자 배율·통합 command·설정 저장·CanvasView transaction 단위 테스트
- `rhwp-studio/e2e/zoom-dialog-transaction.test.mjs`
- `rhwp-studio/e2e/MANIFEST.md`, `rhwp-studio/package.json`
- 수행·구현 계획, Stage 1·2 보고서, 본 최종 보고서와 오늘할일 기록

## 비범위

- slider·pinch preview/settle 성능: #6040
- 적응형 Canvas render scale과 surface 예산: #6041
- 페이지 LRU·행 가상화·prefetch scheduler: #6042
- 쪽 배치별 맞춤 계산: stack base #6108
- 반응형 눈금자 표시 정책과 resize 깜빡임: #6187

## 승인 요청과 게시 경계

작업지시자가 최종 결과와 로컬 UI를 승인했다. Stage 3 변경·본 보고서를 checkpoint commit으로 고정했고,
#6108 위의 #6109 stack diff를 다시 확인했다. remote push와 native stacked PR 생성은 별도 게시 승인을
받은 뒤에만 수행한다.
