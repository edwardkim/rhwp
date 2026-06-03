# Task #283 단계 3 완료보고서 — 코드 변경 + 회귀 테스트

## 수행 내역

옵션 B (글리프 전환 + 임계치 분기) 를 3개 파일에 반영.

### 변경 파일

| 파일 | 위치 | 변경 |
|------|------|------|
| `src/renderer/equation/layout.rs` | L832 | `paren_w: fs * 0.3 → fs * 0.333` (Times advance 매치) |
| `src/renderer/equation/svg_render.rs` | `LayoutKind::Paren` arm | 높이 분기 (글리프 `<text>(</text>` / path `draw_stretch_bracket`) |
| `src/renderer/equation/canvas_render.rs` | `LayoutKind::Paren` arm | 동일 분기 (`ctx.fill_text` / path) |

### 분기 로직

```rust
let paren_w = fs * 0.333;
let use_glyph = lb.height <= fs * 1.2;
if use_glyph && (left == "(" || left == ")") {
    // <text>(</text> 또는 ctx.fill_text
} else {
    draw_stretch_bracket(...);  // 기존 path
}
```

- 임계치: `body.height ≤ fs * 1.2`
- 대상 글리프: `(` / `)` 만 (기타 괄호는 범위 밖)
- Matrix arm 은 항상 path (변경 없음)

### 테스트 갱신

`test_paren_svg` 기존 assertion (`svg.contains("<path")`) 이 새 동작과 배치 — 갱신:

```rust
#[test]
fn test_paren_svg() {
    // 텍스트 높이 파렌은 글리프로 렌더 (Task #283)
    let svg = render_eq("LEFT ( a RIGHT )");
    assert!(svg.contains("<text"));
    assert!(!svg.contains("<path"));
}

#[test]
fn test_paren_stretch_svg() {
    // 스트레치 파렌(분수 감쌈)은 path 유지 (Task #283)
    let svg = render_eq("LEFT ( a over b RIGHT )");
    assert!(svg.contains("<path"));
    assert!(svg.contains("<line"));
}
```

## 회귀 테스트 결과

| 명령 | 결과 |
|------|------|
| `cargo check` | ✅ 통과 |
| `cargo test --lib equation` | ✅ 49/49 (신규 `test_paren_stretch_svg` 포함) |
| `cargo test --test svg_snapshot` | ✅ 3/3 |
| `cargo clippy --lib --bins --tests` | ✅ 에러 없음 (경고만) |
| `cargo check --lib --target wasm32-unknown-unknown` | ✅ 통과 |
| `cargo test --lib` (전체) | 950 pass / 14 fail |

**전체 14개 실패는 Task #280 단계에서 기존 실패로 확인됨** (`serializer::cfb_writer::tests::*` + `wasm_api::tests::*`, Windows CFB writer 이슈). `git stash` 로 변경 전 상태와 비교해도 동일 실패 — 본 타스크와 무관.

## 실제 SVG 출력 확인

`samples/equation-lim.hwp` 를 새 코드로 export → `output/svg/equation-lim.svg`:

```xml
<text x="38.17" y="14.67" font-size="14.67" fill="#000000" font-family="...">(</text>
<text x="44.23" y="14.67" font-size="14.67" fill="#000000" font-family="...">2</text>
...
<text x="73.67" y="14.67" font-size="14.67" fill="#000000" font-family="...">)</text>
...
<text x="98.76" y="14.67" font-size="14.67" fill="#000000" font-family="...">(</text>
<text x="104.82" y="14.67" font-size="14.67" fill="#000000" font-family="...">2</text>
<text x="114.06" y="14.67" font-size="14.67" fill="#000000" font-family="...">)</text>
```

- **4개 파렌 모두 `<text>` 글리프** (line 13·17·20·22)
- **`<path>` 0건** (단계 1 스냅샷은 4건이었음)
- 본 문서는 모든 파렌이 body.height=14.67 = fs*1.0 → glyph 임계치 만족

## 완료 조건

- [x] `layout.rs` paren_w 변경
- [x] `svg_render.rs` Paren arm 분기 추가
- [x] `canvas_render.rs` Paren arm 분기 추가
- [x] 테스트 갱신 + 신규 스트레치 케이스 추가
- [x] 회귀 테스트 전 항목 통과 (기존 14 실패 제외)
- [x] 실제 샘플 SVG 출력 변화 확인

## 다음 단계

단계 4: before/after/pdf 3면 시각 비교 + exam_math.hwp 회귀 확인.
