---
kind: snapshot
status: historical
canonical: mydocs/tech/archive/README.md
last_verified: 2026-07-16
---

# 타스크: 문단 부호 표시 기능 구현

## Context

HWP 편집기의 **문단 부호 표시** 기능을 완성한다. 이 기능은 사용자가 Enter(문단 끝)나 Shift+Enter(강제 줄 바꿈)를 누른 위치를 파란색 기호로 화면에 표시하여 문서 구조를 시각적으로 확인할 수 있게 한다. 인쇄 시에는 나타나지 않는 편집 전용 표시이다.

### 현재 상태

| 항목 | 상태 | 위치 |
|------|------|------|
| `show_paragraph_marks` 플래그 | ✅ 구현됨 | `wasm_api.rs:101` |
| `setShowParagraphMarks()` WASM API | ✅ 구현됨 | `wasm_api.rs:132-134` |
| `is_para_end` 플래그 (TextRunNode) | ✅ 구현됨 | `render_tree.rs:217` |
| ¶ 기호 렌더링 (SVG/Canvas/HTML) | ✅ 구현됨 | `svg.rs:75-81`, `web_canvas.rs:107-114`, `html.rs:119-125` |
| WasmBridge TS 메서드 | ❌ 미구현 | `wasm-bridge.ts` |
| `view:para-mark` 커맨드 실행부 | ❌ 미구현 | `view.ts:80-85` (TODO) |
| 툴바 버튼 커맨드 연결 | ❌ 미구현 | `index.html:232` |
| 메뉴 항목 체크 상태 | ❌ 미구현 | `index.html:70` |
| 강제 줄 바꿈 기호 (↙) | ❌ 미구현 | 렌더러 전체 |

### 핵심 발견

1. **기호 불일치**: 현재 ¶ (U+00B6 PILCROW) 사용 중이나, HWP 표준은 ↵ (U+21B5) 사용
2. **강제 줄 바꿈 미지원**: `\n` (0x000A)으로 분할된 줄 끝에 기호가 없음
3. **ComposedLine 데이터**: 강제 줄 바꿈 시 줄 텍스트 끝에 `\n`이 포함됨 (composer.rs:150)

### WebGian 분석

| 항목 | WebGian 구현 |
|------|-------------|
| 커맨드 | `e_para_mark` → 액션 `ViewOptionParaMark` (ID 34576) |
| 플래그 | `o9` (문단 부호), `u9` (조판 부호) — 독립 토글 |
| 토글 로직 | 조판 부호 ON 상태에서 문단 부호 토글 시 조판+문단 모두 OFF |
| 활성 판정 | `o9 \| u9` 중 하나라도 ON이면 문단 부호 활성 표시 |
| 단축키 | Alt+G+T (데스크톱 Ctrl+G+T에 대응) |
| 아이콘 | 스프라이트 `.e_para_mark.btn_icon_inner` (-320px -280px) |

---

## 구현 계획 (3단계)

### 1단계: 렌더러 기호 수정 + 강제 줄 바꿈 지원 (백엔드)

**수정 파일**: `render_tree.rs`, `composer.rs`, `layout.rs`, `svg.rs`, `web_canvas.rs`, `html.rs`

#### 1-1. ComposedLine에 강제 줄 바꿈 플래그 추가

`composer.rs` — `ComposedLine` 구조체에 필드 추가:
```rust
pub struct ComposedLine {
    // ... 기존 필드
    /// 강제 줄 바꿈(\n)으로 끝나는 줄인지 여부
    pub has_line_break: bool,
}
```

`compose_lines()` 함수에서 줄 텍스트가 `\n`으로 끝나는지 검사하여 설정:
```rust
let has_line_break = line_text.ends_with('\n');
// \n 문자를 텍스트 런에서 제거 (렌더링 폭에 영향 방지)
let line_text = if has_line_break {
    line_text.trim_end_matches('\n').to_string()
} else {
    line_text
};
```

#### 1-2. TextRunNode에 강제 줄 바꿈 플래그 추가

`render_tree.rs` — `TextRunNode`에 필드 추가:
```rust
/// 강제 줄 바꿈(Shift+Enter) 뒤의 마지막 TextRun 여부
pub is_line_break_end: bool,
```

#### 1-3. layout.rs에서 플래그 전달

`layout_composed_paragraph()`에서 각 줄의 마지막 TextRun에 `is_line_break_end` 설정:
```rust
let is_line_break_end = comp_line.has_line_break
    && run_idx == comp_line.runs.len() - 1;
```

#### 1-4. 렌더러 기호 변경

세 렌더러 모두에서:
- 문단 끝(is_para_end): `¶` (U+00B6) → `↵` (U+21B5) 변경 — HWP 표준 부합
- 강제 줄 바꿈(is_line_break_end): `↵` (U+21B5) 동일 기호 사용 (HWP 도움말 참조)
- 색상: 기존 `#4A90D9` (파란색) 유지

**검증**: `docker compose --env-file /dev/null run --rm test`

---

### 2단계: 프론트엔드 토글 기능 구현

**수정 파일**: `wasm-bridge.ts`, `view.ts`, `index.html`

#### 2-1. WasmBridge에 메서드 추가

`wasm-bridge.ts`:
```typescript
setShowParagraphMarks(enabled: boolean): void {
    if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
    this.doc.setShowParagraphMarks(enabled);
}
```

#### 2-2. view:para-mark 커맨드 구현

`view.ts` — 토글 상태 관리:
```typescript
let showParaMarks = false;

{
    id: 'view:para-mark',
    label: '문단 부호',
    icon: 'icon-para-mark',
    canExecute: (ctx) => ctx.hasDocument,
    execute(services) {
        showParaMarks = !showParaMarks;
        services.wasm.setShowParagraphMarks(showParaMarks);
        services.eventBus.emit('document-changed');
    },
},
```

`document-changed` 이벤트가 `CanvasView.refreshPages()`를 트리거하여 모든 보이는 페이지를 재렌더링한다 (canvas-view.ts:40).

#### 2-3. 툴바 버튼 커맨드 연결

`index.html:232` — 문단 부호 버튼에 `data-cmd` 추가:
```html
<button class="tb-btn" data-cmd="view:para-mark" title="문단 부호">
```

#### 2-4. 메뉴 항목 체크 상태

`index.html:70` — 메뉴 항목에서 `disabled` 클래스 제거 (canExecute가 처리):
```html
<div class="md-item" data-cmd="view:para-mark">
```

활성 상태 시각 표시를 위해 `view:para-mark` 커맨드 실행 시 메뉴 항목과 툴바 버튼에 `active` 클래스 토글:
```typescript
execute(services) {
    showParaMarks = !showParaMarks;
    services.wasm.setShowParagraphMarks(showParaMarks);
    // 토글 시각 상태 갱신
    document.querySelectorAll('[data-cmd="view:para-mark"]').forEach(el => {
        el.classList.toggle('active', showParaMarks);
    });
    services.eventBus.emit('document-changed');
},
```

**검증**: 브라우저에서 문서 로드 후 메뉴/툴바 클릭으로 문단 부호 토글 확인

---

### 3단계: 빌드 검증 + SVG 내보내기 확인

1. `docker compose --env-file /dev/null run --rm test` — 전체 테스트 통과
2. `docker compose --env-file /dev/null run --rm wasm` — WASM 빌드
3. `cd rhwp-studio && npx vite build` — Vite 빌드
4. SVG 내보내기 검증: `show_paragraph_marks=true` 상태에서
   - `samples/sample.hwp` — 각 문단 끝에 ↵ 기호 확인
   - 강제 줄 바꿈이 있는 문서에서 ↙ 기호 확인

---

## 수정 파일 요약

| 파일 | 변경 | 규모 |
|------|------|------|
| `src/renderer/composer.rs` | ComposedLine에 `has_line_break` 추가, `\n` 제거 | ~10줄 |
| `src/renderer/render_tree.rs` | TextRunNode에 `is_line_break_end` 추가 | ~3줄 |
| `src/renderer/layout.rs` | `is_line_break_end` 플래그 전달 | ~10줄 |
| `src/renderer/svg.rs` | ¶→↵ 변경, 줄 바꿈 ↵ 추가 | ~10줄 |
| `src/renderer/web_canvas.rs` | ¶→↵ 변경, 줄 바꿈 ↵ 추가 | ~10줄 |
| `src/renderer/html.rs` | ¶→↵ 변경, 줄 바꿈 ↵ 추가 | ~10줄 |
| `rhwp-studio/src/core/wasm-bridge.ts` | `setShowParagraphMarks()` 메서드 추가 | ~4줄 |
| `rhwp-studio/src/command/commands/view.ts` | `view:para-mark` 커맨드 구현 | ~15줄 |
| `rhwp-studio/index.html` | 버튼 `data-cmd` + 메뉴 disabled 제거 | ~2줄 |
