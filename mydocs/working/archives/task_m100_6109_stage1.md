# Task M100 #6109 Stage 1 완료 보고 — 사용자 배율 검증·접근성

- **이슈**: [#6109](https://github.com/edwardkim/rhwp/issues/6109)
- **브랜치**: `codex/issue-6109-zoom-dialog-transaction`
- **stack base**: `codex/issue-6108-zoom-fit` `711365b35`
- **보고일**: 2026-08-28 KST
- **단계 상태**: 구현·focused test·실제 Chrome smoke·결과 승인 완료

## 결과

사용자 정의 배율을 제출할 때 더 이상 잘못된 값을 10~500%로 조용히 보정하지 않는다.

- 빈 값, 숫자가 아닌 값, 정수가 아닌 값, 10% 미만, 500% 초과를 순수 validator에서 구분한다.
- 오류가 있으면 `ZoomDialog.onConfirm()`이 `false`를 반환해 대화상자와 입력값을 유지하고 callback을
  실행하지 않는다.
- 입력과 안정된 오류 ID를 `aria-describedby`로 연결하고, 실패 시 `aria-invalid="true"`와
  `role="alert"`를 제공한다.
- 실패 입력에 focus를 되돌리고 전체 값을 선택해 바로 교정할 수 있게 한다.
- 유효한 값으로 교정하면 오류 alert와 invalid 상태를 해제한다.
- 사용자 정의 입력 안의 Enter는 공용 확인 버튼 경로를 사용한다. Escape는 공용 모달 취소 계약을
  유지하고, 대화상자를 닫을 때 전용 capture listener를 제거한다.
- 정수 10~500만 제출 가능한 값으로 정의해 입력값과 적용 백분율이 정확히 일치한다. 기존 내부 방어용
  clamp와 현재 배율 복원은 변경하지 않았다.

## 변경 파일

- `rhwp-studio/src/view/zoom-dialog-state.ts`
  - `validateCustomZoomPercent()`와 discriminated result 추가
- `rhwp-studio/src/ui/zoom-dialog.ts`
  - 제출 차단, 오류 표시·해제, ARIA, focus/select, Enter 처리와 listener 정리
- `rhwp-studio/src/styles/zoom-dialog.css`
  - 저장소 danger/text/size 토큰을 사용한 invalid input·오류 텍스트
- `rhwp-studio/tests/zoom-dialog.test.ts`
  - empty/non-number/fraction/9/10/137/500/501 validator 경계
- `rhwp-studio/tests/zoom-dialog-integration.test.ts`
  - 공용 `false` 유지 계약, 오류 ARIA·focus, Enter·listener 정리 구조 회귀

## 검증

### focused test·TypeScript

| 명령 | 결과 |
| --- | --- |
| `node --test tests/zoom-dialog.test.ts tests/zoom-dialog-integration.test.ts` | 18/18 통과 |
| `node --test tests/zoom-dialog.test.ts tests/zoom-dialog-integration.test.ts tests/dialog-apply-standard.test.ts tests/issue-3414-modal-focus.test.ts` | 24/24 통과 |
| `npx tsc --noEmit -p tsconfig.ci-unit.json` | 통과 |
| `npx tsc --noEmit` | worktree 내부 WASM `pkg/` 준비 후 통과 |
| `git diff --check` | 통과 |

첫 `npx tsc --noEmit`은 worktree에 파생 WASM package가 없어 기존 `@wasm/rhwp.js` TS2307 5건으로
중단됐다. 원 저장소의 이미 빌드된 `pkg/`를 worktree에 임시 복사해 전체 검사를 통과했고, 검증 뒤 복사본은
worktree 밖 `/private/tmp/rhwp-issue-6109-stage1-pkg-20260828-1528`로 이동했다. source 변경은 없다.

### 실제 Chrome smoke

worktree Vite 서버 `http://127.0.0.1:7729/`와 설치된 macOS Chrome headless에서
`2010-01-06.hwp`를 열어 다음 21개 assertion을 확인했다.

| 시나리오 | 결과 |
| --- | --- |
| 빈 값·9·501 제출 | 각 dialog 유지, `aria-invalid=true`, 연결 alert 표시, 입력 focus, zoom 무변경 |
| invalid 9 → valid 137 교정 | alert hidden, `aria-invalid=false` |
| 사용자 입력에서 Enter | dialog 종료, 최종 zoom 정확히 1.37 |
| 200 입력 뒤 Escape | dialog 종료, 기존 zoom 1.37 보존 |

숫자가 아닌 문자열은 `input[type=number]`의 브라우저 값 sanitization으로 빈 문자열이 되므로 실제 UI에서는
빈 값 오류 경로로 수렴한다. 순수 validator 단위 테스트는 API에 직접 들어오는 비숫자 문자열도 별도로
검증한다.

## 범위 확인

- Stage 1은 입력 검증과 대화상자 접근성·키보드 계약만 변경했다.
- command의 쪽 배치·배율 순차 적용과 CanvasView 렌더 경로는 아직 변경하지 않았다.
- Stage 2에서 사용자 설정 저장과 `page-view-settings-changed` transaction을 원자화한다.
- 임시 smoke script `/private/tmp/issue-6109-stage1-smoke.mjs`와 E2E 산출물은 커밋 대상이 아니다.

## 승인 결과와 다음 단계

2026-08-28 작업지시자가 Stage 1 결과를 승인했다. 이 변경과 보고서를 checkpoint로 commit한 뒤
Stage 2 원자 보기 설정 transaction을 구현한다.
