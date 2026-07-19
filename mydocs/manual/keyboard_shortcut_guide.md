---
kind: guide
status: active
canonical: mydocs/manual/keyboard_shortcut_guide.md
last_verified: 2026-07-17
---

# 키보드 단축키 추가 가이드

## 단축키 추가 방법

### 1. 커맨드 시스템 경유 단축키 (권장)

대부분의 단축키는 이 방식으로 추가한다.

**파일**: `rhwp-studio/src/command/shortcut-map.ts`

```typescript
export const defaultShortcuts: [ShortcutDef, string][] = [
  // 기존 항목...
  [{ key: 'enter', ctrl: true }, 'page:break'],  // ← 여기에 추가
];
```

**필드 설명**:
- `key`: `e.key.toLowerCase()` 값. 예: `'enter'`, `'z'`, `'f7'`, `'home'`
- `ctrl`: `true`이면 Ctrl(Windows) 또는 Meta(Mac) 필요
- `shift`: `true`이면 Shift 필요
- `alt`: `true`이면 Alt 필요

**커맨드 구현**: `rhwp-studio/src/command/commands/` 하위 파일에 커맨드 정의

```typescript
{
  id: 'page:break',
  label: '쪽 나누기',
  shortcutLabel: 'Ctrl+Enter',  // 메뉴 표시용
  canExecute: (ctx) => ctx.hasDocument,
  execute(services) { /* 구현 */ },
}
```

### 2. 코드 단축키 (Ctrl+K → ?)

두 키 연속 입력 방식. `chordMapK` 테이블에 추가.

**파일**: `rhwp-studio/src/engine/input-handler-keyboard.ts`

```typescript
const chordMapK: Record<string, string> = {
  b: 'insert:bookmark',
  ㅠ: 'insert:bookmark',  // 한글 IME 상태
};
```

### 3. 모드별 키 처리 (직접 핸들링)

편집 모드에 따라 동일 키가 다르게 동작하는 경우. `onKeyDown` 내 모드별 분기에서 직접 처리.

**해당 모드**:
- 머리말/꼬리말 편집 모드
- 각주 편집 모드
- F5 셀 선택 모드
- 그림/표 객체 선택 모드
- 연결선/다각형 그리기 모드

**파일**: `rhwp-studio/src/engine/input-handler-keyboard.ts`

## onKeyDown 처리 순서

```
1.  코드 단축키 2번째 키 (Ctrl+K → ?)
2.  특수 모드 탈출 (연결선/다각형/이미지/글상자 배치 모드 → Escape)
3.  IME 조합 중 네비게이션 키 보류
4.  편집 모드별 키 처리 (머리말꼬리말 / 각주)
5.  F5 셀 선택 모드
6.  셀 선택 모드 키 처리
7.  그림/표 객체 선택 모드 키 처리
8.  Ctrl/Meta 조합 → handleCtrlKey() → shortcut-map.ts 경유
9.  Alt 조합 → shortcut-map.ts 경유
10. 본문 키 처리 (Esc, Backspace, Enter, Arrow 등)
```

**주의**: 상위 단계에서 `return`하면 하위 단계에 도달하지 않는다.

## 관련 파일

| 파일 | 역할 |
|------|------|
| `src/command/shortcut-map.ts` | 단축키 → 커맨드 ID 매핑 테이블 |
| `src/command/commands/*.ts` | 커맨드 정의 (execute 함수) |
| `src/command/dispatcher.ts` | 커맨드 디스패처 |
| `src/engine/input-handler-keyboard.ts` | 키보드 이벤트 핸들러 (onKeyDown) |
| `src/engine/input-handler.ts` | InputHandler 메인 클래스 (handleCtrlKey 위임) |

## 한글 IME 대응

한글 IME 활성 상태에서 알파벳 키가 한글로 변환된다. 단축키 등록 시 한글 키도 함께 등록한다.

```typescript
[{ key: 'l', alt: true }, 'format:char-shape'],
[{ key: 'ㄹ', alt: true }, 'format:char-shape'],  // 한글 IME
```

## WASM API 연동이 필요한 경우

1. Rust에 `*_native` 함수 구현 (`src/document_core/commands/`)
2. WASM 바인딩 추가 (`src/wasm_api.rs`)
3. TypeScript 브릿지 추가 (`src/core/wasm-bridge.ts`)
4. 커맨드 execute에서 브릿지 호출
