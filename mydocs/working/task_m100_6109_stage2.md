# Task M100 #6109 Stage 2 완료 보고 — 원자 보기 설정 transaction

- **이슈**: [#6109](https://github.com/edwardkim/rhwp/issues/6109)
- **브랜치**: `codex/issue-6109-zoom-dialog-transaction`
- **stack base**: `codex/issue-6108-zoom-fit` `534cf40ed`
- **보고일**: 2026-08-28 KST
- **단계 상태**: 구현·focused test·결과 승인 완료

## 결과

확대/축소 대화상자에서 쪽 배치·쪽 이동·맞춤 모드·최종 배율을 각각 적용하던 경로를 하나의 보기 설정
transaction으로 통합했다.

- `UserSettings.setPageViewSettings()`가 쪽 배치와 이동을 함께 정규화하고 맞춤 모드까지 한 번에 저장한다.
  기존 개별 setter는 이 API를 경유하므로 호환성을 유지한다.
- 확인 명령은 최종 설정을 저장한 뒤 `page-view-settings-changed`를 한 번만 발행한다. payload에는 최종
  배율, 맞춤 모드와 중앙 zoom anchor가 함께 들어간다.
- `page-view-settings-changed`의 기존 배치-only payload도 계속 수용한다. 새 resolver는 잘못된 값은 기존
  정규화 규칙으로 수렴시키고, 유효하지 않은 zoom은 transaction에서 제외한다.
- `ViewportManager.setZoom()`이 발행하는 표준 `zoom-changed`는 그대로 유지한다. 눈금자·입력 커서·선택
  핸들 등 다른 소비자는 기존과 같이 최종 배율 알림을 한 번 받는다.
- CanvasView만 transaction 안에서 자신의 중첩 zoom handler를 억제하고, 최종 배치·이동·배율이 모두
  반영된 뒤 `recalcLayout()`과 anchor 복원, Canvas 해제, visible page 갱신을 한 경로로 수행한다.
- guard는 `try/finally`로 해제한다. 동기 이벤트 subscriber가 예외를 내더라도 최종 레이아웃을 마친 뒤
  예외를 다시 전달해 중간 상태와 영구 guard를 남기지 않는다.

## 변경 파일

- `rhwp-studio/src/view/page-view-settings-change.ts`
  - 기존 payload 호환과 선택적 zoom transaction 정규화 계약
- `rhwp-studio/src/core/user-settings.ts`
  - 쪽 배치·이동·맞춤 모드 단일 저장 API
- `rhwp-studio/src/command/commands/view.ts`
  - 최종 보기 설정 transaction 단일 발행
- `rhwp-studio/src/view/canvas-view.ts`
  - transaction 중 중첩 zoom layout 억제와 최종 상태 단일 layout 경로
- `rhwp-studio/tests/page-view-settings-change.test.ts`
  - 구 payload 호환, 결합 payload 정규화, invalid zoom 제외
- `rhwp-studio/tests/user-settings.test.ts`
  - 결합 설정의 정규화와 localStorage 단일 기록
- `rhwp-studio/tests/zoom-dialog-integration.test.ts`
  - command가 결합 setter와 transaction payload만 사용하는 구조 계약
- `rhwp-studio/tests/canvas-view-page-arrangement.test.ts`
  - guard·`try/finally`·단일 recalc·조건부 Canvas 해제 구조 계약

## 불변식

| 항목 | Stage 2 계약 |
| --- | --- |
| 설정 저장 | 쪽 배치·이동·맞춤 모드를 한 번에 기록 |
| 보기 이벤트 | 확인 한 번당 `page-view-settings-changed` 한 번 |
| zoom 이벤트 | 표준 `zoom-changed` 유지, CanvasView 자신의 중첩 처리만 억제 |
| 레이아웃 | 배치 또는 배율이 바뀌면 최종 상태 기준 `recalcLayout()` 한 번 |
| Canvas 해제 | 실제 배율 또는 행 토폴로지가 바뀔 때 한 번 |
| 예외 안전성 | transaction guard를 항상 해제하고 최종 layout 뒤 오류 재전달 |
| 하위 호환 | zoom 없는 기존 page-view payload 계속 지원 |

## 검증

작업 worktree의 `rhwp-studio`에서 다음 검사를 실행했다.

| 명령 | 결과 |
| --- | --- |
| `node --test tests/page-view-settings-change.test.ts tests/user-settings.test.ts tests/zoom-dialog.test.ts tests/zoom-dialog-integration.test.ts tests/canvas-view-page-arrangement.test.ts tests/viewport-manager-smooth-zoom.test.ts` | 49/49 통과 |
| `npx tsc --noEmit -p tsconfig.ci-unit.json` | 통과 |
| `git diff --check` | 통과 |

focused test는 순수 payload 정규화와 저장 횟수, command wiring, CanvasView의 단일 layout 구조를 검증한다.
실제 브라우저에서 이벤트 수·최종 layout 상태를 동적으로 계측하는 검증은 Stage 3의 명시된 범위다.

## 범위 확인

- Stage 2는 보기 설정 적용 순서와 transaction 경계만 바꿨다.
- Stage 1의 invalid 입력 UI·ARIA·Enter/Escape 계약은 변경하지 않았다.
- #6108이 제공한 쪽 배치별 맞춤 배율 계산식은 수정하지 않고 최종 결과만 transaction에 전달한다.
- slider·pinch preview, 적응형 render scale, 페이지 가상화 정책은 각각 #6040·#6041·#6042 범위로 남긴다.

## 승인 결과와 다음 단계

2026-08-28 작업지시자가 Stage 2 결과를 승인했다. 이 변경과 보고서를 checkpoint commit으로 고정한 뒤
Stage 3에서 실제 Chrome으로 invalid/valid/Enter/Escape/cancel과 배치+배율 transaction의 이벤트·layout
횟수를 검증하고 전체 Studio test·build 및 저장소 필수 format gate를 수행한다.
