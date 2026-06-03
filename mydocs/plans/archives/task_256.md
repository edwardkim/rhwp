# Task 256 수행 계획서: 각주 편집 UI 연동

## 현재 상태

- Rust API (`footnote_ops.rs`): 텍스트 삽입/삭제/문단분할/병합 완료
- WASM 바인딩 + wasm-bridge.ts 메서드 완료
- **미구현**: 각주 영역 클릭 감지, 편집 모드 진입/탈출, 키 입력 라우팅

## 참조 패턴: 머리말/꼬리말 편집 모드

- `cursor.ts`: `_headerFooterMode`, `enterHeaderFooterMode()`, `exitHeaderFooterMode()`
- `input-handler-mouse.ts`: `hitTestHeaderFooter()` → 모드 진입/전환
- `input-handler-keyboard.ts`: Escape 탈출, Enter 문단분할, 화살표 이동

## 구현 계획

### 1단계: Rust 히트테스트 + 커서 렉트 API

Rust 측에 각주 영역 히트테스트/커서 위치 API 추가:

| API | 설명 |
|-----|------|
| `hitTestFootnote(pageNum, x, y)` | 각주 영역 클릭 여부 + (paraIdx, controlIdx) 반환 |
| `hitTestInFootnote(pageNum, paraIdx, controlIdx, x, y)` | 각주 내 정확한 (fnParaIdx, charOffset) 반환 |
| `getCursorRectInFootnote(sec, paraIdx, controlIdx, fnParaIdx, charOffset)` | 각주 내 커서 렉트 반환 |

### 2단계: cursor.ts 각주 모드 상태

- `_footnoteMode: boolean` 속성 추가
- `_fnParaIdx`, `_fnControlIdx`, `_fnInnerParaIdx`, `_fnCharOffset` 상태
- `enterFootnoteMode()`, `exitFootnoteMode()`, `setFnCursorPosition()` 메서드
- `updateRect()` 내 각주 모드 분기 추가

### 3단계: input-handler 각주 편집 라우팅

- **mouse**: 각주 영역 클릭 → 편집 모드 진입, 본문 클릭 → 탈출
- **keyboard**: Escape 탈출, Enter 문단분할, Backspace 문단병합, 화살표 이동, 텍스트 입력

### 참조 파일

- cursor.ts: `rhwp-studio/src/engine/cursor.ts`
- input-handler-mouse.ts: `rhwp-studio/src/engine/input-handler-mouse.ts`
- input-handler-keyboard.ts: `rhwp-studio/src/engine/input-handler-keyboard.ts`
- wasm-bridge.ts: `rhwp-studio/src/core/wasm-bridge.ts`
