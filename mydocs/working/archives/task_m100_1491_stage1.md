# Task M100 #1491 Stage 1 - undo 후 로컬 resize 재현 회귀

- 이슈: #1491
- 브랜치: `local/task_m100_1491`
- 작성일: 2026-06-23
- 모드: 기여자 모드

## 증상

1. Shift+마우스로 특정 셀 좌우 크기를 조절한다.
2. Cmd+Z로 원복하면 화면상 셀 폭은 원래 형태로 돌아온다.
3. 이후 가장 우측 셀 경계를 일반 마우스로 조절하면, Cmd+Z로 원복했던 이전 Shift resize 형태가 다시 나타난다.
4. Shift+마우스로 셀 너비를 조절하려 할 때, 세로 컬럼 경계와 가로 행 경계가 만나는 지점에서는 높이 resize로 잘못 잡혀 너비 조절이 되지 않을 수 있다.

## 가설

- Undo는 문서 모델 상태를 되돌리지만, Studio 런타임의 `tableLocalResizeSegments` 같은 로컬 resize 히스토리 캐시가 함께 되돌아가지 않는다.
- 이후 일반 컬럼 resize가 `hasLocalResizeHistory()` 경로로 들어가면서, 문서 모델에는 없어야 할 이전 로컬 segment를 다시 `localResize/renderWidth` 힌트로 보강할 수 있다.
- 캐시 무효화가 resize 완료, undo/redo, 문서 변경 이벤트 사이에서 충분히 일어나지 않는지 확인한다.
- `TableResizeRenderer.hitTestBorder()`가 수평 경계선을 먼저 반환하면 교차점에서 세로 컬럼 resize 의도가 행 resize로 해석될 수 있다.

## 조사 파일

- `rhwp-studio/src/engine/input-handler-table.ts`
- `rhwp-studio/src/engine/input-handler-mouse.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/src/engine/command.ts`
- `rhwp-studio/src/core/document-dirty-state.ts`
- 기존/신규 Studio 회귀 테스트

## 목표

- Cmd+Z 이후 이전 Shift local resize 히스토리가 다음 일반 컬럼 resize에 재적용되지 않게 한다.
- 로컬 resize 히스토리는 현재 문서 모델 상태와 일치할 때만 사용한다.
- 경계선 교차점에서도 컬럼 경계에 가까운 마우스 위치는 컬럼 resize로 안정적으로 잡는다.
- 기존 Shift+단일 셀 resize, 일반 컬럼 resize, `셀 너비를 같게` 보정 동작은 유지한다.

## 검증

```bash
cd rhwp-studio && npm test
cd rhwp-studio && npx tsc --noEmit
cargo fmt --check
git diff --check
```

필요 시:

```bash
cargo test --profile release-test --test issue_493_cell_attrs -- --nocapture
```
