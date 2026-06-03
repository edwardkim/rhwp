# 타스크 73 — 최종 결과 보고서

## 문단 부호 표시 기능 구현

### 개요

HWP 편집기에서 문단 끝(Enter)과 강제 줄 바꿈(Shift+Enter)의 위치를 ↵(U+21B5) 기호로 표시하는 기능을 구현하였다. 사용자가 메뉴 또는 툴바를 통해 토글할 수 있으며, 편집 전용 표시로 인쇄 시에는 나타나지 않는다.

### 구현 결과

| 단계 | 내용 | 결과 |
|------|------|------|
| 1단계 | 백엔드 렌더러 기호 수정 + 강제 줄 바꿈 지원 | 완료 |
| 2단계 | 프론트엔드 토글 기능 (WasmBridge, 커맨드, 메뉴/툴바) | 완료 |
| 3단계 | 빌드 검증 (test + WASM + Vite + SVG 내보내기) | 완료 |

### 수정 파일 목록

| 파일 | 변경 내용 |
|------|-----------|
| `src/renderer/composer.rs` | `ComposedLine.has_line_break` 필드 추가, `compose_lines()`에서 `\n` 감지 및 제거 |
| `src/renderer/render_tree.rs` | `TextRunNode.is_line_break_end` 필드 추가 |
| `src/renderer/layout.rs` | 10개 TextRunNode 생성 위치에 `is_line_break_end` 전달 |
| `src/renderer/svg.rs` | ¶(U+00B6) → ↵(U+21B5) 변경, `is_line_break_end` 조건 추가 |
| `src/renderer/web_canvas.rs` | ¶(U+00B6) → ↵(U+21B5) 변경, `is_line_break_end` 조건 추가 |
| `src/renderer/html.rs` | ¶(U+00B6) → ↵(U+21B5) 변경, `is_line_break_end` 조건 추가 |
| `rhwp-studio/src/core/wasm-bridge.ts` | `setShowParagraphMarks(enabled)` 메서드 추가 |
| `rhwp-studio/src/command/commands/view.ts` | `view:para-mark` 커맨드 구현 (IIFE 클로저, 토글 상태, active 클래스) |
| `rhwp-studio/index.html` | 메뉴 항목 `disabled` 제거, 툴바 버튼에 `data-cmd="view:para-mark"` 추가 |
| `rhwp-studio/src/main.ts` | `.tb-btn[data-cmd]` 클릭 → 커맨드 디스패치 핸들러 추가 |

### 주요 기술 사항

#### 백엔드 (Rust)

1. **강제 줄 바꿈 감지**: `composer.rs`에서 줄 텍스트가 `\n`으로 끝나는지 검사하여 `ComposedLine.has_line_break` 플래그 설정. `\n` 문자는 텍스트에서 제거하여 렌더링 폭에 영향을 주지 않도록 처리.

2. **렌더 트리 전달**: `layout.rs`에서 각 줄의 마지막 TextRun에 `is_line_break_end` 플래그를 설정하여 렌더러에 전달.

3. **기호 렌더링**: SVG/Canvas/HTML 세 렌더러 모두에서 `is_para_end` 또는 `is_line_break_end`인 TextRun 뒤에 ↵(U+21B5) 기호를 파란색(#4A90D9)으로 렌더링.

#### 프론트엔드 (TypeScript)

1. **토글 로직**: IIFE 클로저로 `showParaMarks` 상태 캡슐화. 토글 시 `wasm.setShowParagraphMarks()` 호출 후 `document-changed` 이벤트로 재렌더링 트리거.

2. **UI 연결**: 메뉴 항목과 툴바 버튼 모두 `data-cmd="view:para-mark"`로 통합. 활성 시 `active` CSS 클래스 토글.

### 검증 결과

| 항목 | 결과 |
|------|------|
| Rust 테스트 | 488개 통과 |
| WASM 빌드 | 성공 |
| Vite 빌드 | 성공 (36 modules, 783ms) |
| SVG 내보내기 | 정상 (기본 상태에서 ↵ 기호 미표시 확인) |

### 참고: WebGian 분석

한컴 웹기안기의 구현을 분석하여 참고하였다.

| 항목 | WebGian 구현 |
|------|-------------|
| 커맨드 | `e_para_mark` → `ViewOptionParaMark` (ID 34576) |
| 플래그 | `o9` (문단 부호), `u9` (조판 부호) — 독립 토글 |
| 토글 로직 | 조판 부호 ON 상태에서 문단 부호 토글 시 조판+문단 모두 OFF |
| 활성 판정 | `o9 \| u9` 중 하나라도 ON이면 활성 표시 |
