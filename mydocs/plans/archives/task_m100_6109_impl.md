# 구현 계획 — Task M100 #6109

- **이슈**: [#6109](https://github.com/edwardkim/rhwp/issues/6109)
- **브랜치**: `codex/issue-6109-zoom-dialog-transaction`
- **stack base**: `codex/issue-6108-zoom-fit` `711365b35`
- **문서 성격**: 구현 전 파일 단위 설계

## Stage 1 — 사용자 배율 검증·접근성 계약

### `rhwp-studio/src/view/zoom-dialog-state.ts`

- 사용자 정의 입력 문자열을 받는 순수 validator를 추가한다.
- 빈 문자열, 유한 숫자가 아닌 값, 정수가 아닌 값, 10 미만, 500 초과를 구분하고 한국어 오류 사유를
  반환한다.
- 성공값은 clamp에 의존하지 않는 정수 10~500으로 반환해 제출값과 실제 배율이 정확히 일치하게 한다.
- 기존 `clampCustomZoomPercent()`는 현재 배율 복원과 신뢰된 내부 수치 방어에 유지한다.

### `rhwp-studio/src/ui/zoom-dialog.ts`

- 사용자 정의 배율 입력 옆/아래에 안정된 ID의 hidden alert 요소를 만든다.
- 입력에 `aria-describedby`를 연결하고 실패 시 `aria-invalid="true"`, alert 표시, focus/select 후
  `onConfirm()`에서 `false`를 반환한다.
- invalid에서는 callback을 호출하지 않는다. preset·fit·여러 쪽처럼 사용자 입력이 적용되지 않는 선택은
  기존 경로를 유지한다.
- 입력이 유효하게 교정되면 alert와 invalid 상태를 해제한다.
- 공용 모달의 capture 경계 뒤에서 사용자 입력 Enter를 확인 버튼 click으로 연결하고 `hide()`에서 listener를
  제거한다. Escape는 공용 모달 계약을 그대로 사용한다.

### `rhwp-studio/src/styles/zoom-dialog.css`

- 기존 `--color-*`, `--font-size-*`, spacing 토큰으로 오류 텍스트와 invalid input 상태를 표현한다.
- 오류가 hidden일 때 레이아웃 공간을 차지하지 않으며 좁은 화면 grid도 깨뜨리지 않는다.

### Stage 1 테스트

- `zoom-dialog.test.ts`: validator의 empty/non-number/fraction/9/10/500/501 경계
- `zoom-dialog-integration.test.ts`: false 유지, alert/ARIA 연결, Enter listener 정리 계약
- 실제 Chrome smoke: invalid 확인 뒤 dialog 유지·callback 무실행·focus, 유효 교정 뒤 오류 해제

## Stage 2 — 원자 보기 설정 transaction

### `rhwp-studio/src/core/user-settings.ts`

- `setPageViewSettings(arrangement, movement)`를 추가해 `resolvePageViewSettings()`를 한 번 실행하고 정규화된
  두 값을 한 번에 대입·저장한다.
- 기존 단일 setter는 다른 호출부 호환을 위해 유지하되 확대/축소 대화상자는 통합 setter만 사용한다.

### `rhwp-studio/src/command/commands/view.ts`

- 확인 시 #6108 공통 resolver로 최종 zoom을 계산한 뒤 사용자 배치를 통합 setter로 한 번 저장한다.
- `page-view-settings-changed` payload 하나에 정규화된 arrangement·pageMovement와 최종
  zoom·zoomFitMode·`CENTER_ZOOM_ANCHOR`를 함께 넣는다.
- command에서 `vm.setZoom()`을 별도로 호출하지 않아 순차 두 commit을 제거한다.
- transaction 완료 뒤 `command-state-changed`는 기존처럼 한 번 발행한다.

### `rhwp-studio/src/view/canvas-view.ts`

- `page-view-settings-changed` payload의 optional zoom transaction을 해석한다. zoom이 없는 기존 발행자는
  현재 배치-only 경로와 호환된다.
- 배치·이동·배율의 변경 여부와 이전 중심 쪽 box·layout topology를 commit 전에 한 번 캡처한다.
- transaction 중 `ViewportManager.setZoom()`이 발행하는 표준 `zoom-changed`는 그대로 다른 소비자에게
  전달한다. CanvasView의 `zoom-changed` subscriber만 `applyingPageViewTransaction` guard에서 반환한다.
- guard는 `try/finally`로 해제한다.
- 최종 arrangement·movement·zoom이 모두 반영된 뒤 `recalcLayout()`을 한 번 호출하고 중심 앵커를 한 번
  복원한다.
- zoom 수치 또는 topology가 바뀐 경우에만 Canvas·pending render/prefetch를 한 번 해제하고 최종 visible
  pages를 한 번 갱신한다.
- 중첩 zoom handler가 건너뛴 `zoom-level-display`는 transaction method가 최종 zoom으로 한 번 발행한다.
- 문서가 비어 있는 경우에도 설정 snapshot과 ViewportManager 상태는 같은 순서로 commit하되 render는 하지
  않는다.

### Stage 2 테스트

- `user-settings.test.ts`: 배치+이동 정규화가 한 snapshot으로 저장되는지 확인
- `zoom-dialog-integration.test.ts`: command가 한 payload만 emit하고 별도 `vm.setZoom()`을 호출하지 않는지
  확인
- `canvas-view-page-arrangement.test.ts`: transaction guard, 한 recalc, 최종 zoom의 display, 조건부 한 Canvas
  해제 계약
- `viewport-manager-smooth-zoom.test.ts`: 표준 zoom/fit-mode 이벤트가 한 번 유지되는지 focused 회귀

## Stage 3 — 통합·실제 브라우저 회귀

### `rhwp-studio/e2e/zoom-dialog-transaction.test.mjs`

- #6108 E2E helper 흐름을 재사용하되 #6109 수용 기준만 독립 테스트로 둔다.
- 사용자 정의 선택 뒤 빈 값·비숫자·9·501을 제출해 dialog 유지, 오류, ARIA, focus, 설정·배율 무변경을
  확인한다.
- 10·137·500을 마우스와 Enter로 제출해 입력값과 최종 zoom이 일치하고 dialog가 한 번 닫히는지 확인한다.
- invalid 뒤 유효값을 입력해 오류 상태가 해제되는지 확인한다.
- 배치·배율을 바꾼 뒤 Escape와 취소 버튼 각각에서 원래 arrangement·movement·zoom·fit mode를 보존한다.
- 배치와 수치 배율을 동시에 바꿀 때 event count를 계측하고, 개발 빌드의 CanvasView method를 일시 감싼
  probe로 `recalcLayout()`이 정확히 한 번이며 그 시점의 zoom·arrangement가 모두 최종값인지 확인한다.
- probe는 테스트 종료 전에 원 메서드를 복원하고 제품 진단 API를 새로 노출하지 않는다.

### `rhwp-studio/package.json`

- 독립 재현 명령 `e2e:zoom-dialog-transaction`을 추가한다.

### 최종 검증·보고

- focused test와 TypeScript 검사 뒤 Studio 전체 test·production build를 실행한다.
- 실제 Chrome E2E 보고서와 대표 오류·성공 화면을 로컬 산출물로 확인한다.
- `mydocs/report/task_m100_6109_report.md`에 event/recalc 횟수와 검증 명령 결과를 기록한다.
- Rust source는 바꾸지 않지만 PR/push 직전 저장소 필수 `cargo fmt --all` 및 `-- --check`를 실행한다.

## 불변식

1. invalid 입력은 callback·settings save·view event를 0회 실행한다.
2. valid 입력은 callback과 view transaction을 1회 실행한다.
3. transaction은 표준 `zoom-changed`를 다른 소비자에게 1회 전달한다.
4. CanvasView는 transaction 안에서 이전 배율 레이아웃을 만들지 않고 최종 상태로만 1회 recalc한다.
5. 취소는 user settings와 ViewportManager 상태를 바꾸지 않는다.
6. 기존 배치-only `page-view-settings-changed` 발행자는 동작을 유지한다.
7. #6108의 쪽 배치별 맞춤 계산값과 저장 복원 계약은 바뀌지 않는다.

## 커밋 경계 후보

1. `docs: #6109 사용자 배율·보기 트랜잭션 계획`
2. `fix(studio): 사용자 배율 입력을 제출 전에 검증한다`
3. `perf(studio): 보기 설정과 배율을 원자적으로 적용한다`
4. `test(studio): #6109 대화상자 트랜잭션 회귀를 검증한다`
5. `docs(test): #6109 통합 검증 결과`

각 경계는 해당 Stage 결과 승인 뒤에만 커밋한다. #6108 bottom branch에는 #6109 변경을 역으로 포함하지
않는다.

## stacked PR 게시 계획

1. #6109 최종 결과를 승인받은 뒤 #6108·#6109의 exact local branch 관계를 확인한다.
2. 두 branch에 대해 필수 format·전체 Studio test·build·각 E2E를 통과한다.
3. 한국어 PR 제목·본문 초안에 stack 순서와 bottom-first merge 조건을 명시한다.
4. 별도 게시 승인 뒤 native `gh stack`으로 bottom #6108은 `devel`, top #6109는 #6108을 base로 push·PR
   생성한다.
5. 게시 뒤 base/head, issue link, 한글 본문, exact head를 API로 다시 확인한다.
