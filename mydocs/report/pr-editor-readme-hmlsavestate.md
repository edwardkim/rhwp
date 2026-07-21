# PR 처리 보고: @rhwp/editor README getHmlSaveState() 반환 형태 수정

## Issue
- #2752 — docs(editor): @rhwp/editor README의 getHmlSaveState() 반환 형태가 실제 DTO와 다름
- README에 `{ ok: true }` / `{ ok: false, blocker: '...' }`로 문서화되어 있었으나, 실제 반환 타입과 불일치

## 원인 분석
- `npm/editor/README.md` lines 217-219에 `getHmlSaveState()` 예시 코드가 `{ ok: true }` 또는 `{ ok: false, blocker: '...' }` 형태로 잘못 표기됨
- 실제 Rust backend 및 `index.d.ts`의 `HmlSaveState` 인터페이스는 다음과 같음:
  ```typescript
  export interface HmlSaveState {
    sourceFormat: string;
    hmlSavable: boolean;
    blockers: HmlSaveBlocker[];
  }
  ```
- `HmlSaveBlocker`는 `{ code, xmlPath, message, preserved }` 필드를 가짐
- `npm/editor/index.d.ts` (lines 35-46)는 이미 올바른 타입 정의를 가지고 있어 README만 불일치 상태였음

## 변경 내용
### `npm/editor/README.md`
- `getHmlSaveState()` 문서의 예시 코드를 실제 DTO에 맞게 수정
  - `{ ok: true }` → `{ sourceFormat: 'hwp', hmlSavable: true, blockers: [] }`
  - `{ ok: false, blocker: '...' }` → `{ sourceFormat: 'hwpx', hmlSavable: false, blockers: [ { code, xmlPath, message, preserved } ] }`
- 하단에 `blockers` 배열과 `hmlSavable` 관계 설명 및 각 blocker 필드에 대한 간략 설명 추가

## 검증
- `npm/editor/index.d.ts`의 `HmlSaveState`, `HmlSaveBlocker` 인터페이스와 일치
- `rhwp-studio/src/core/hml-save-capability.ts`의 `parseHmlSaveState()` 파싱 로직과 일치
- `rhwp-studio/src/core/wasm-bridge.ts`의 `getHmlSaveState()` 구현과 일치

## 작업 브랜치
- `pr/fix-issue-2752-editor-readme-hmlsavestate`
