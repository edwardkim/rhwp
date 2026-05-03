# Task #516 Stage 4 완료 보고서 — Web Canvas CSS filter + 회귀 테스트 + WASM

## 변경 요약

| 파일 | 영역 | 변경 |
|------|------|------|
| `src/renderer/web_canvas.rs` | 1, 2, 3, 4 | `compose_image_filter` 헬퍼 추가 + `RenderNodeType::Image` 분기에 `set_filter` / draw / `set_filter("none")` 패턴 적용 |
| `tests/issue_516.rs` (신규) | 검증 | 회귀 테스트 5 건 (헬퍼 단위 3 + JSON watermark 2) |

소스 변경: ~+50 / 0 (web_canvas.rs +50 + tests/issue_516.rs +90)

## 변경 상세

### 1. Web Canvas CSS filter 적용 (`src/renderer/web_canvas.rs`)

#### `compose_image_filter` 헬퍼 (신규)

```rust
#[cfg(target_arch = "wasm32")]
fn compose_image_filter(
    effect: ImageEffect,
    brightness: i8,
    contrast: i8,
) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    match effect {
        ImageEffect::GrayScale | ImageEffect::Pattern8x8 => {
            parts.push("grayscale(100%)".to_string());
        }
        ImageEffect::BlackWhite => {
            parts.push("grayscale(100%)".to_string());
            parts.push("contrast(1000%)".to_string());
        }
        ImageEffect::RealPic => {}
    }
    if brightness != 0 {
        let css_b = (100.0 + brightness as f64) / 100.0;
        parts.push(format!("brightness({:.4})", css_b));
    }
    if contrast != 0 {
        let css_c = (100.0 + contrast as f64) / 100.0;
        parts.push(format!("contrast({:.4})", css_c));
    }
    if parts.is_empty() { None } else { Some(parts.join(" ")) }
}
```

#### `RenderNodeType::Image` 분기 정정

```rust
RenderNodeType::Image(img) => {
    self.open_shape_transform(&img.transform, &node.bbox);
    if let Some(ref data) = img.data {
        // Task #516: 그림 효과 / 밝기 / 대비 / 워터마크를 CSS filter 로 적용
        let filter_str = compose_image_filter(img.effect, img.brightness, img.contrast);
        if let Some(ref f) = filter_str {
            self.ctx.set_filter(f);
        }
        self.draw_image_with_fill_mode(
            data, &node.bbox, img.fill_mode, img.original_size, img.crop,
            img.original_size_hu,
        );
        // 다음 그리기 작업에 영향 없도록 reset
        if filter_str.is_some() {
            self.ctx.set_filter("none");
        }
    }
}
```

복학원서.hwp 의 엠블렘 (effect=GrayScale, b=-50, c=70) → CSS filter:
```
grayscale(100%) brightness(0.5000) contrast(1.7000)
```

### 2. 회귀 테스트 (`tests/issue_516.rs`)

5 tests 추가:

1. `issue_516_image_attr_helper_hancom_preset` — 한컴 자동 프리셋 (b=70, c=-50, GrayScale) → `"hancom-watermark"`
2. `issue_516_image_attr_helper_custom_watermark` — 복학원서 패턴 (b=-50, c=70, GrayScale) → `"custom"`
3. `issue_516_image_attr_helper_no_watermark` — RealPic / effect-only / bc-only 모두 워터마크 아님
4. `issue_516_layer_tree_json_includes_watermark_for_emblem` — PageLayerTree JSON 에 `"watermark":{"preset":"custom"}` 포함
5. `issue_516_layer_tree_json_no_watermark_for_normal_image` — 워터마크 필드 출현 횟수 정확히 1 (엠블렘만, 학교 로고는 RealPic)

## 검증 결과

### 게이트 통과 현황

| 게이트 | 결과 |
|--------|------|
| `cargo build --lib` | ✅ Finished |
| `cargo build --release` | ✅ Finished |
| **`cargo test --test issue_516`** | ✅ **5 passed** (신규) |
| `cargo test --lib` | ✅ **1110 passed** (회귀 0) |
| `cargo test --test issue_418` (셀 padding) | ✅ 1 passed |
| `cargo test --test issue_501` (mel-001) | ✅ 1 passed |
| `cargo test --test issue_514` (PCX) | ✅ 3 passed |
| `cargo test --test svg_snapshot` | ✅ **6/6 passed** |
| `cargo clippy --lib -- -D warnings` | ✅ 0 건 |
| `cargo clippy --test issue_516 -- -D warnings` | ✅ 0 건 |

### WASM 빌드 + 동기화

| 산출물 | 이전 (Task #514 후) | Stage 4 (현재) | 변동 |
|--------|---------------------|---------------|------|
| `pkg/rhwp_bg.wasm` | 4,452,204 bytes | **4,453,541 bytes** | +1,337 bytes (web_canvas filter 분기 + paint json watermark) |
| `pkg/rhwp.js` | 230,417 bytes | 230,558 bytes | +141 bytes |
| `rhwp-studio/public/rhwp_bg.wasm` | stale | ✅ 동기화 (4,453,541 bytes) | — |
| `rhwp-studio/public/rhwp.js` | stale | ✅ 동기화 | — |

## 위험 점검

| 위험 | 결과 |
|------|------|
| `ctx.set_filter` 일부 브라우저 미지원 | 🟢 본 코드는 modern Canvas 표준 정합 (Chrome 52+, Firefox 35+, Safari 9.1+). 한국 사용자 환경 기준 안전 |
| Canvas filter 가 다음 그리기 작업에 영향 | ✅ `set_filter("none")` reset 으로 차단 |
| 헬퍼 추가로 다른 사용처 회귀 | ✅ 0 (`cargo test --lib` 1110 passed) |
| ImageNode ↔ ImageAttr 변환 비용 | ✅ Copy trait, no-alloc |

## 작업지시자 시각 판정 자료 (Stage 5 입력)

복학원서.hwp 가운데 엠블렘의 web Canvas 출력 비교:

**Before (Task #514 머지 시점):**
- 엠블렘이 원본 컬러로 강하게 출력 (워터마크 효과 미적용)

**After (Stage 4):**
- CSS filter `grayscale(100%) brightness(0.5000) contrast(1.7000)` 적용
- 흐릿한 회색조 워터마크로 표시 (한컴 정답지 정합 예상)

## CSS filter ↔ SVG feComponentTransfer 매핑 차이 (Stage 5 검증 항목)

Stage 1 §"CSS filter ↔ SVG feComponentTransfer 매핑" 에서 식별된 위험:

- CLI SVG: `출력 = 입력 × 1.7 - 0.85`
- CSS filter: `출력 = ((입력 × 0.5) - 0.5) × 1.7 + 0.5 = 입력 × 0.85 - 0.35`

**두 식의 결과 다름** — Stage 5 작업지시자 시각 판정으로 차이 허용 범위 확인.
- 차이가 시각적으로 무시 가능하면 본 매핑 유지
- 초과하면 ImageData 직접 변환 (방안 B) 또는 정확 매핑 알고리즘 (방안 C) 폴백

## 다음 단계

Stage 4 완료 보고서 승인 후 **Stage 5** 진행:
- 작업지시자 web Canvas 시각 판정 (rhwp-studio dev server 재시작 + hard reload)
- 한컴 정답지 (`samples/복학원서.pdf`) 와 web Canvas 출력 비교
- CSS filter 매핑 차이 허용 범위 검증
- 시각 판정 통과 후 → 최종 보고서 + merge + 이슈 close

## 산출물

- `src/renderer/web_canvas.rs` (compose_image_filter + Image 분기)
- `tests/issue_516.rs` (5 tests)
- `pkg/rhwp_bg.wasm` (4,453,541 bytes) → `rhwp-studio/public/` 동기화 완료
- 본 보고서 (`mydocs/working/task_m100_516_stage4.md`)
