# 구현 계획서 — Task #131

**이슈**: [#131](https://github.com/edwardkim/rhwp/issues/131)
**타이틀**: 한컴 단축키 호환성 전체 정비 — Ctrl+G 계열 + 재매핑 + / 커맨드 팔레트
**마일스톤**: M100
**작성일**: 2026-04-13
**브랜치**: `local/task131`

---

## 현재 구조 요약

| 구성요소 | 파일 | 현재 상태 |
|---------|------|----------|
| Chord 처리 | `input-handler-keyboard.ts` | `chordMapK`, `chordMapN`, `chordMapV` 구현됨. `chordMapG` 없음 |
| Ctrl+G Chord 1번째 키 | `handleCtrlKey()` | 미구현 — `Ctrl+G,C` shortcutLabel만 있고 실제 동작 안 함 |
| 서식 단축키 | `shortcut-map.ts` | `Ctrl+B/I/U` 구현됨. `Ctrl+]/[`, 정렬 없음 |
| 정렬 커맨드 | `format.ts` | `format:align-left/center/right/justify` 커맨드 존재 |
| `/` 커맨드 팔레트 | — | 미구현 |

---

## 단계별 구현 계획

### 1단계: `chordMapG` 추가 — Ctrl+G 계열 완성

**파일**: `rhwp-studio/src/engine/input-handler-keyboard.ts`

#### 1-1. `chordMapG` 테이블 추가

```typescript
/** 코드 단축키 → 커맨드 ID 매핑 (Ctrl+G,? 형태 — 보기/조판) */
const chordMapG: Record<string, string> = {
  c: 'view:ctrl-mark',       // 조판 부호
  ㅊ: 'view:ctrl-mark',      // 한글 IME
  t: 'view:para-mark',       // 문단 부호
  ㅅ: 'view:para-mark',      // 한글 IME
  p: 'view:zoom-fit-page',   // 쪽 맞춤
  ㅍ: 'view:zoom-fit-page',  // 한글 IME
  w: 'view:zoom-fit-width',  // 폭 맞춤
  ㅈ: 'view:zoom-fit-width', // 한글 IME
  q: 'view:zoom-100',        // 100%
  ㅂ: 'view:zoom-100',       // 한글 IME
};
```

#### 1-2. Chord 2번째 키 처리 블록에 `_pendingChordG` 추가

#### 1-3. `handleCtrlKey()`에 `Ctrl+G` 1번째 키 처리 추가

```typescript
if ((e.key === 'g' || e.key === 'G' || e.key === 'ㅎ') && !e.shiftKey && !e.altKey) {
  e.preventDefault();
  this._pendingChordG = true;
  return;
}
```

#### 1-4. `index.html` 보기 메뉴 `md-shortcut` 정비

```html
<!-- 현재 -->
<span class="md-shortcut">Ctrl+G+C</span>  <!-- 조판 부호 -->
<!-- 없음 -->                                <!-- 문단 부호 -->

<!-- 수정 후 -->
<span class="md-shortcut">Ctrl+G,C</span>  <!-- 조판 부호 (표기 통일) -->
<span class="md-shortcut">Ctrl+G,T</span>  <!-- 문단 부호 -->
<span class="md-shortcut">Alt+V,T</span>   <!-- 투명 선 -->
```

#### 1-5. `view.ts` shortcutLabel 정비

```typescript
shortcutLabel: 'Ctrl+G,C',  // 'Ctrl+G+C' → 'Ctrl+G,C' (한컴 표기 통일)
```

---

### 2단계: 서식 단축키 추가 + 브라우저 충돌 재매핑

**파일**: `rhwp-studio/src/command/shortcut-map.ts`

#### 2-1. 추가할 단축키

```typescript
// 글씨 크기 (한컴: Alt+Shift+E/R, 추가: Ctrl+]/[)
[{ key: ']', ctrl: true }, 'format:font-size-increase'],
[{ key: '[', ctrl: true }, 'format:font-size-decrease'],

// 정렬 — 브라우저 충돌 분석 결과 반영
// Ctrl+Shift+L: 주소창(브라우저 양보함) → 지원
// Ctrl+Shift+R: 강제 새로고침(브라우저 불양보) → 재매핑 불필요(Alt+Shift+R 유지)
// Ctrl+Shift+C: 요소 검사(브라우저 불양보) → 재매핑 불필요(Alt+Shift+C 유지)
// Ctrl+Shift+M: 양쪽 정렬 → 지원
[{ key: 'l', ctrl: true, shift: true }, 'format:align-left'],
[{ key: 'm', ctrl: true, shift: true }, 'format:align-justify'],
```

#### 2-2. `index.html` 서식 메뉴 `md-shortcut` 정비

진하게(`Ctrl+B`), 기울임(`Ctrl+I`), 밑줄(`Ctrl+U`) — 이미 구현됨, 메뉴 표시만 추가.

---

### 3단계: `/` 커맨드 팔레트

**신규 파일**: `rhwp-studio/src/ui/command-palette.ts`

#### 3-1. 데이터 구조

커맨드 시스템(`CommandDef`)의 `id`, `label`, `shortcutLabel`을 팔레트 검색 소스로 활용.

```typescript
interface PaletteItem {
  id: string;
  label: string;
  shortcut?: string;     // rhwp 실제 단축키
  hancomKey?: string;    // 한컴 원본 단축키 (재매핑된 경우 표시)
  conflictNote?: string; // 브라우저 충돌 안내
}
```

#### 3-2. UI 구조

```
┌─────────────────────────────────────────┐
│ /  [검색어 입력창]                        │
├─────────────────────────────────────────┤
│ 진하게               Ctrl+B             │
│ 기울임               Ctrl+I             │
│ 문단 부호            Ctrl+G,T           │
│ 첫 줄 내어쓰기       Ctrl+G,O  ⚠ 한컴Ctrl+F5→브라우저 충돌 │
│ ...                                     │
└─────────────────────────────────────────┘
```

#### 3-3. 동작

- `/` 키 입력 시 팔레트 표시 (본문 편집 모드에서만)
- 한글/영문 검색 지원
- 항목 클릭 또는 Enter → 커맨드 디스패치
- Escape → 닫기
- 브라우저 충돌로 재매핑된 키는 `⚠` 아이콘 + 원래 한컴 키 표시

---

### 4단계: 정책 문서 + 나머지 단축키

#### 4-1. `mydocs/tech/shortcut_policy.md` 작성

- 브라우저 충돌 분석 전체 표
- 재매핑 원칙 및 목록
- rhwp 독자 단축키 목록

#### 4-2. 나머지 단축키

```typescript
// Shift+Enter — 강제 줄 나누기
// (입력 처리 직접 구현 — shortcut-map 경유 불가)
```

---

## 작업 제약 조건

**표 관련 기존 단축키는 절대 변경하지 않는다.**

표 편집 모드 전용 단축키(`F5`, `Shift+F5`, `Tab`, `Shift+Tab`, `Alt+방향키`, `Ctrl+Enter`, `Ctrl+BackSpace`, `H/W/M/S/P/L/C/B/F` 등)는 이번 작업 범위에서 제외한다. 단축키 추가 시 표 모드와 충돌하지 않는지 반드시 확인한다.

---

## 파일 수정 범위

| 파일 | 변경 내용 |
|------|----------|
| `src/engine/input-handler-keyboard.ts` | `chordMapG` 추가, `handleCtrlKey` Ctrl+G 처리 |
| `src/command/shortcut-map.ts` | `Ctrl+]/[`, `Ctrl+Shift+L/M` 추가 |
| `src/command/commands/view.ts` | `shortcutLabel` 표기 정비 |
| `src/ui/command-palette.ts` | 신규 — `/` 커맨드 팔레트 |
| `index.html` | 보기/서식 메뉴 `md-shortcut` 정비 |
| `mydocs/tech/shortcut_policy.md` | 신규 — 단축키 정책 문서 |

---

## 승인 요청

위 구현 계획서를 검토 후 승인해주시면 1단계부터 구현을 시작하겠습니다.
