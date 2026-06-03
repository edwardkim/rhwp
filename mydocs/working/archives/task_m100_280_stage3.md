# Task #280 단계 3 완료보고서 — 폰트 스택 변경 + 회귀 테스트

## 수행 내역

### 1. 코드 변경

#### `src/renderer/equation/svg_render.rs:10-13`

변경 전:
```rust
/// 수식 전용 font-family (Latin Modern Math → STIX Two Math → Cambria Math → Pretendard → serif)
const EQ_FONT_FAMILY: &str = " font-family=\"'Latin Modern Math', 'STIX Two Math', 'Cambria Math', 'Pretendard', serif\"";
```

변경 후:
```rust
/// 수식 전용 font-family
/// 순서: Latin Modern Math (LaTeX 설치 시) → STIX Two Text (Mac/STIX 설치 시) → STIX Two Math → Times New Roman (Windows 기본) → serif
/// Cambria Math 는 Windows 에서 "볼드 인상" 을 유발해 제외. Pretendard 는 산세리프라 수식 부적합으로 제외. (Task #280)
const EQ_FONT_FAMILY: &str = " font-family=\"'Latin Modern Math', 'STIX Two Text', 'STIX Two Math', 'Times New Roman', 'Times', serif\"";
```

#### `src/renderer/equation/canvas_render.rs:219-227`

변경 전:
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

변경 후:
```rust
fn set_font(ctx: &CanvasRenderingContext2d, size: f64, italic: bool, bold: bool) {
    let style = if italic { "italic " } else { "" };
    let weight = if bold { "bold " } else { "" };
    // svg_render.rs 의 EQ_FONT_FAMILY 와 동일 스택 유지. (Task #280)
    ctx.set_font(&format!(
        "{}{}{:.1}px 'Latin Modern Math', 'STIX Two Text', 'STIX Two Math', 'Times New Roman', 'Times', serif",
        style, weight, size,
    ));
}
```

### 2. 회귀 테스트

| 명령 | 결과 | 비고 |
|------|------|------|
| `cargo build --release` | ✅ 성공 | 22.26s |
| `cargo test --lib equation` | ✅ **48 passed** / 0 failed | 수식 관련 전 테스트 통과 (svg_render 내 `test_simple_text_svg`, `test_fraction_svg`, `test_paren_svg`, `test_eq01_svg` 포함) |
| `cargo test --lib` (전체) | ⚠️ 949 passed / **14 failed** (pre-existing) | 실패는 모두 `serializer::cfb_writer::tests::*`, `wasm_api::tests::test_*` — CFB 파서 Windows path `\` 문제. 수식과 무관 |
| `cargo test --test svg_snapshot` | ✅ 3 passed / 0 failed | SVG 스냅샷 회귀 없음 |
| `cargo clippy --lib -- -D warnings` | ✅ 경고 없음 | |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ 성공 | WASM 빌드도 클린 |

#### 기존 실패 건 검증

수정 전 상태(`git stash` 후) 에서도 동일한 14건이 실패. 이번 변경과 무관한 기존 환경 문제 — 별도 이슈로 추적 중이거나 Windows 개발 환경 한정 문제로 추정.

### 3. SVG 출력 검증

재빌드 후 `samples/equation-lim.hwp` 에서 SVG 재생성:

```
<text x="0.00" y="21.56" font-size="14.67" fill="#000000" font-family="'Latin Modern Math', 'STIX Two Text', 'STIX Two Math', 'Times New Roman', 'Times', serif">lim</text>
```

새 폰트 스택이 정확히 반영됨.

## 스냅샷 영향

- `cargo test --test svg_snapshot` 의 스냅샷 파일은 수식이 포함된 케이스가 아니어서 영향 없음 (`table_text_page_0`, `form_002_page_0`, `render_is_deterministic_within_process`)
- `tests/snapshots/` 등 별도 스냅샷 디렉토리도 확인 필요 없음 — 수식 전용 스냅샷 파일 부재

## 완료 조건

- [x] `svg_render.rs` 폰트 스택 변경
- [x] `canvas_render.rs` 폰트 스택 변경 (svg 와 동기)
- [x] `cargo test --lib equation` 통과
- [x] `cargo test --test svg_snapshot` 통과
- [x] `cargo clippy --lib -- -D warnings` 클린
- [x] WASM 빌드 클린
- [x] SVG 출력에서 새 폰트 스택 반영 확인

## 다음 단계

단계 4: 변경 후 PNG 스냅샷 생성 → before/after/pdf 3종 시각 비교 → `samples/exam_math.hwp` 회귀.
