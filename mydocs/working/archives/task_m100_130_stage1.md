# 단계별 완료 보고서 — Task #130 Stage 1+2+3

**이슈**: [#130](https://github.com/edwardkim/rhwp/issues/130)
**타이틀**: 투명 선 토글 단축키 추가 및 초기값 버그 수정 — Alt+V, T (Chord)
**작성일**: 2026-04-13

---

## 완료 내용

### 1단계: 초기값 버그 수정

**파일**: `rhwp-studio/src/command/commands/view.ts`

- 클로저 변수 `let showBorders = false` 제거
- `services.wasm.getShowTransparentBorders()`로 WASM 실제 상태를 읽어 토글
- 셀 진입 자동 ON 등으로 인한 상태 불일치 근본 해결

**파일**: `src/wasm_api.rs`

- `get_show_transparent_borders()` getter 추가 (`js_name = getShowTransparentBorders`)

**파일**: `rhwp-studio/src/core/wasm-bridge.ts`

- `getShowTransparentBorders()` 브릿지 메서드 추가

**파일**: `pkg/rhwp.d.ts`

- `getShowTransparentBorders(): boolean` 타입 정의 추가

### 2단계: chordMapV + Alt+V Chord 처리

**파일**: `rhwp-studio/src/engine/input-handler-keyboard.ts`

- `chordMapV` 테이블 추가: `t/ㅅ → view:border-transparent`
- Chord 2번째 키 처리 블록에 `_pendingChordV` 처리 추가
- Alt 조합 블록에 `Alt+V(ㅍ)` → `_pendingChordV = true` 처리 추가

---

## 검증

- `cargo check` 통과
- `npx tsc --noEmit` 통과

---

## 수정 파일 목록

| 파일 | 변경 내용 |
|------|----------|
| `src/wasm_api.rs` | `getShowTransparentBorders` getter 추가 |
| `pkg/rhwp.d.ts` | 타입 정의 추가 |
| `rhwp-studio/src/core/wasm-bridge.ts` | `getShowTransparentBorders()` 브릿지 추가 |
| `rhwp-studio/src/command/commands/view.ts` | 초기값 버그 수정 — WASM 실제 상태 기반 토글 |
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | `chordMapV` + Chord 처리 추가 |
