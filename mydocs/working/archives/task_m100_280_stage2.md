# Task #280 단계 2 완료보고서 — `canvas_render.rs` 영향도 조사

## 목적

Phase 1 의 실제 변경 범위 결정:
- `svg_render.rs` 만 고치면 되는지
- `canvas_render.rs` 도 함께 손봐야 하는지
- 추가로 영향받는 코드(svg 렌더러, 폰트 임베딩 등) 가 있는지

## 조사 결과

### 1. `canvas_render.rs` 의 폰트 지정 위치

`src/renderer/equation/canvas_render.rs:219-226` 의 `set_font` 헬퍼 1곳에서만 font-family 를 지정:

```rust
fn set_font(ctx: &CanvasRenderingContext2d, size: f64, italic: bool, bold: bool) {
    let style = if italic { "italic " } else { "" };
    let weight = if bold { "bold " } else { "" };
    ctx.set_font(&format!(
        "{}{}{:.1}px 'Latin Modern Math', 'STIX Two Math', 'Cambria Math', 'Pretendard', serif",
        style, weight, size,
    ));
}
```

모든 `fill_text` 호출이 이 헬퍼를 거치므로 **1 줄 수정**으로 캔버스 전체 수식 렌더링에 반영됨.

### 2. svg_render.rs 와의 관계

두 파일이 폰트 스택 문자열을 **각자 중복 보유**함:

| 파일 | 위치 | 형식 |
|------|------|------|
| `svg_render.rs:11` | `EQ_FONT_FAMILY` 상수 | ` font-family="..."` (SVG 속성 래퍼 포함) |
| `canvas_render.rs:223` | `set_font` 함수 내 literal | `"{size}px ..."` (Canvas font shorthand) |

문자열 포맷이 달라 단순 상수 공유는 어려움. 다만 **목록(`'Latin Modern Math', ..., serif`) 부분은 동일**.

→ Phase 1 에서는 **양쪽을 동시에 수정** (중복 수정). 상수 공유 리팩터는 Phase 2 후보.

### 3. 추가 영향 지점: `src/renderer/svg.rs` 폰트 임베딩

`src/renderer/svg.rs:330-342` — `--embed-fonts` 옵션 사용 시 수식에서 사용된 문자들을 수집하는 로직:

```rust
if self.font_embed_mode != FontEmbedMode::None {
    let codepoints = self.font_codepoints
        .entry("Latin Modern Math".to_string())   // ← 고정 폰트명
        .or_default();
    // SVG <text> 요소 내부의 텍스트에서 문자 추출
    for segment in eq.svg_content.split("</text>") { ... }
}
```

그리고 `src/renderer/svg.rs:2290` 에 `Latin Modern Math → latinmodern-math.otf` 매핑이 있음.

**판단**: 이 로직은 "수식에 사용된 글자들을 `Latin Modern Math` 라는 키로 수집 → 해당 폰트 파일(`latinmodern-math.otf`) 을 서브셋해서 SVG 에 임베딩" 하는 흐름. `--embed-fonts` 옵션 사용자를 위한 기능.

폰트 스택 재정렬 후에도:
- 새 스택이 `'Latin Modern Math'` 를 **여전히 첫 번째로 유지**하면 → 이 로직 그대로 유효. Latin Modern Math 가 설치된 시스템에서는 그게 사용되고, 없으면 다음 폴백으로. 임베딩 파이프라인도 자연스럽게 Latin Modern Math 서브셋을 만듦.
- 만약 `Latin Modern Math` 를 제거하면 이 부분도 업데이트 필요 → 보수적으로 **유지**.

### 4. WASM 빌드 경로

`canvas_render.rs` 는 `#[cfg(target_arch = "wasm32")]` — WASM 에서만 컴파일됨. 네이티브 `cargo test --lib` 는 이 모듈을 빌드/테스트하지 않음. 회귀 확인은 WASM 빌드로 별도 진행하거나, `cargo check --target wasm32-unknown-unknown` 으로 컴파일만 확인.

## 제안 수정안 (수행계획서 업데이트)

원래 제안:
```rust
'STIX Two Text', 'Latin Modern Roman', 'Times New Roman', 'Times', 'Cambria', serif
```

수정 제안 (폰트 임베딩 파이프라인 호환 유지):
```rust
'Latin Modern Math', 'STIX Two Text', 'STIX Two Math', 'Times New Roman', 'Times', serif
```

**근거**:
- `Latin Modern Math` 를 첫 번째로 유지 → `svg.rs:330` 의 임베딩 키와 일치. LaTeX 설치된 머신에서는 그대로 사용되며 기존 동작 보존.
- `STIX Two Text` 추가 → Math 보다 획이 얇음 (본문용). 설치된 환경에서 이상적.
- `STIX Two Math` 유지 → 기존 우선순위 약간 뒤로.
- `Times New Roman` / `Times` 추가 → 거의 모든 시스템에서 매칭. Windows 에서 여기서 stop.
- **`Cambria Math` 제거** → Windows "볼드 인상" 의 원인.
- **`Pretendard` 제거** → Pretendard 는 산세리프(한글+라틴). 수식 렌더링에 부적합.

### 예상 폴백 결과별 시나리오

| 환경 | 매칭 폰트 | 결과 |
|------|-----------|------|
| LaTeX 설치 (Linux/Mac/Windows) | Latin Modern Math | 고전적 얇은 세리프 (PDF 와 가장 유사) |
| Mac (STIX 내장) | STIX Two Text | 얇고 깔끔한 세리프 |
| Windows 기본 | Times New Roman | 얇은 클래식 세리프 (Cambria Math 제거 효과) |
| 리눅스 최소 환경 | serif (기본) | 시스템 기본 세리프 (liberation-serif 등) |

## 변경 범위 (Phase 1)

- `src/renderer/equation/svg_render.rs:11` — `EQ_FONT_FAMILY` 상수 (1줄)
- `src/renderer/equation/canvas_render.rs:223` — `set_font` 함수 내 format 문자열 (1줄)
- `src/renderer/svg.rs:332` — **변경 없음** (Latin Modern Math 키 유지, 임베딩 파이프라인 호환)

## 범위 밖 (Phase 2 후보)

- 두 파일의 폰트 스택 문자열을 공용 상수로 리팩터링
- 괄호 path 폭 (`fs * 0.3`) 조정
- 폰트 임베딩 키를 동적으로 결정 (여러 폰트를 한꺼번에 서브셋)

## 완료 조건

- [x] `canvas_render.rs` 의 폰트 지정 위치 파악 (`set_font:219`)
- [x] `svg_render.rs` 와의 공유/중복 관계 확인 (별도 literal, 동시 수정 필요)
- [x] 추가 영향 지점 탐색 (`svg.rs:332` font embedding — 변경 불필요)
- [x] 수정 범위 및 새 폰트 스택 최종안 도출

## 다음 단계

단계 3: 코드 수정 (`svg_render.rs` + `canvas_render.rs`) + 회귀 테스트.
