# Task #1151 Stage 2 (B) 완료 보고서 — TS bridge + handler floating 전달

수행계획서: [task_m100_1151.md](../plans/task_m100_1151.md) · 구현계획서: [task_m100_1151_impl.md](../plans/task_m100_1151_impl.md) · Stage 1: [task_m100_1151_stage1.md](task_m100_1151_stage1.md)

## 1. 변경 내용

### rhwp-studio/src/core/wasm-bridge.ts
`insertPicture` 시그니처에 `cellPathJson: string` 인자 추가. 반환 타입은 본문/셀 공통 `{ok, paraIdx, controlIdx}` 유지. JSDoc 으로 본문 inline vs 셀 floating 분기 설명.

### rhwp-studio/src/engine/input-handler-table.ts
`finishImagePlacement`: hit.cellPath 가 있으면 `parentParaIndex` (= 표가 들어있는 outer paragraph) 와 `JSON.stringify(hit.cellPath)` 를 전달. 본문 클릭은 기존 paragraphIndex / 빈 cellPathJson.

### rhwp-studio/src/engine/input-handler-keyboard.ts
Ctrl+V paste 이미지 핸들러도 동일 패턴 — `cursor.getPosition()` 의 cellPath 검사 후 parentParaIndex / cellPath JSON 전달.

## 2. 검증 결과

- WASM 빌드: `docker compose --env-file .env.docker run --rm wasm` → success
- TypeScript: `cd rhwp-studio && npx tsc --noEmit` → **무경고** (사전 무관 canvaskit-wasm 모듈 부재 에러 제외)
- Stage 1 의 Rust 단위 테스트 GREEN 상태 유지 확인

## 3. Stage 3 진입 조건

- TS bridge cellPath 관통 ✓
- WASM .d.ts 시그니처 동기 ✓
- 본문 호출 패턴 유지 (빈 cellPathJson) ✓

→ Stage 3 (브라우저 수동 검증 + 회귀 + 보고서 + PR) 진행 가능.
