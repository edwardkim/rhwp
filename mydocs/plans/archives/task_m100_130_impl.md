# 구현 계획서 — Task #130

**이슈**: [#130](https://github.com/edwardkim/rhwp/issues/130)
**타이틀**: 투명 선 토글 단축키 추가 및 초기값 버그 수정 — Alt+V, T (Chord)
**마일스톤**: M100
**작성일**: 2026-04-13
**브랜치**: `local/task130`

---

## 구현 목표

1. `Alt+V → T` Chord 단축키로 `view:border-transparent` 토글
2. `showBorders` 초기값 불일치 버그 수정 — 첫 번째 토글부터 정상 동작

---

## 현재 구조 요약

| 구성요소 | 파일 | 현재 동작 |
|---------|------|----------|
| Chord 1번째 키 처리 | `input-handler-keyboard.ts:821-831` | `Ctrl+K`, `Ctrl+N` → `_pendingChordK/N = true` |
| Chord 2번째 키 처리 | `input-handler-keyboard.ts:82-102` | `_pendingChordK/N` 플래그 확인 후 커맨드 디스패치 |
| Alt 조합 처리 | `input-handler-keyboard.ts:639-646` | `matchShortcut()` → `shortcut-map.ts` 경유 |
| 투명 선 커맨드 | `view.ts:89-103` | `let showBorders = false` 초기값 불일치 |

**`Alt+V` Chord 구조**:
- 기존 `Ctrl+K/N` Chord는 `handleCtrlKey()`에서 처리
- `Alt+V`는 Alt 조합 처리 블록(L639)에서 별도로 `_pendingChordV = true` 세팅 필요
- 2번째 키(`T`)는 Chord 2번째 키 처리 블록(L82) 상단에 추가

---

## 단계별 구현 계획

### 1단계: 초기값 버그 수정 — `view.ts`

**파일**: `rhwp-studio/src/command/commands/view.ts`

**현재 코드**:
```typescript
(() => {
  let showBorders = false;
  return {
    id: 'view:border-transparent',
    ...
    execute(services) {
      showBorders = !showBorders;
      services.wasm.setShowTransparentBorders(showBorders);
```

**문제**: WASM 초기 상태는 `false`(투명선 숨김)인데, 메뉴에서 '투명 선'을 클릭하면 이미 표시 상태로 보여지는 경우가 있어 불일치 발생.

**수정**: `showBorders` 초기값을 WASM 실제 상태와 동기화.
`getShowTransparentBorders()` WASM API가 없으면 초기값을 `true`로 변경 (메뉴 클릭 시 즉시 숨김 동작).

검증 후 올바른 초기값으로 고정.

---

### 2단계: `chordMapV` 추가 및 Alt+V Chord 처리

**파일**: `rhwp-studio/src/engine/input-handler-keyboard.ts`

#### 2-1. `chordMapV` 테이블 추가 (L55 부근)

```typescript
/** 코드 단축키 → 커맨드 ID 매핑 (Alt+V,? 형태) */
const chordMapV: Record<string, string> = {
  t: 'view:border-transparent',
  ㅅ: 'view:border-transparent', // 한글 IME
};
```

#### 2-2. Chord 2번째 키 처리 블록에 추가 (L82 부근)

```typescript
if (this._pendingChordV) {
  this._pendingChordV = false;
  const key = e.key.toLowerCase();
  const cmdId = chordMapV[key];
  if (cmdId && this.dispatcher) {
    e.preventDefault();
    this.dispatcher.dispatch(cmdId);
    return;
  }
}
```

#### 2-3. Alt 조합 처리 블록에 `Alt+V` Chord 1번째 키 처리 추가 (L639 부근)

```typescript
if (e.altKey && this.dispatcher) {
  // Alt+V → Chord 대기 (보기 메뉴 단축키)
  if ((e.key === 'v' || e.key === 'V' || e.key === 'ㅍ') && !e.shiftKey) {
    e.preventDefault();
    this._pendingChordV = true;
    return;
  }
  const cmdId = matchShortcut(e, defaultShortcuts);
  if (cmdId) {
    e.preventDefault();
    this.dispatcher.dispatch(cmdId);
    return;
  }
}
```

---

### 3단계: 검증

1. 투명 선이 있는 문서 열기
2. `Alt+V → T` 입력 → 투명 선 빨간 점선 표시 확인
3. 다시 `Alt+V → T` → 숨김 확인
4. **첫 번째 토글부터** 즉시 적용 확인 (버그 수정 검증)
5. 한글 IME 상태에서 `Alt+V → ㅅ` 동작 확인
6. 기존 단축키 (`Alt+T`, `Alt+L` 등) 회귀 없음 확인
7. `cargo test` / TypeScript 빌드 통과 확인

---

## 파일 수정 범위

| 파일 | 변경 내용 |
|------|----------|
| `rhwp-studio/src/command/commands/view.ts` | `showBorders` 초기값 수정 |
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | `chordMapV` 추가, Chord 2번째/1번째 키 처리 추가 |

---

## 승인 요청

위 구현 계획서를 검토 후 승인해주시면 1단계부터 구현을 시작하겠습니다.
