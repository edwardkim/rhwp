# Task #516 구현 계획서 — Web Canvas 워터마크 효과 + dump 메타정보 + AI JSON

## 1. 결함 요약 (Stage 1 진단 결과)

**본질**: 본 task 는 6 영역의 통합 정정.

| 영역 | 위치 | 정정 |
|------|------|------|
| 1, 2, 3 | `src/renderer/web_canvas.rs::draw_image_with_fill_mode` 등 | CSS filter (`grayscale + brightness + contrast`) 적용 + reset |
| 4 | (자동) | CSS filter 가 RGBA 알파 보존 (W3C 정합) — Task #514 정합 |
| 5 | `src/main.rs` (dump 사이트 4 곳) | `[image_attr] effect=... b=... c=... watermark=... preset=...` 추가 |
| 6 | `src/paint/json.rs` + `src/model/image.rs` | `watermark` + `preset` 필드 추가 (additive) |

**한컴 워터마크 프리셋** (Stage 1 작업지시자 정합):
- 자동 프리셋: `effect=GrayScale, brightness=70, contrast=-50`
- 복학원서.hwp: `effect=GrayScale, brightness=-50, contrast=70` — 편집자 의도적 사용자 정의 (`preset = "custom"`)

## 2. 변경 영역 정리

### 2.1 신규 모델 헬퍼 — `src/model/image.rs`

`ImageAttr` 에 워터마크 식별 헬퍼 메서드 추가:

```rust
impl ImageAttr {
    /// 워터마크 효과가 적용되어 있는지 (effect != RealPic && bc 변경)
    pub fn is_watermark(&self) -> bool {
        !matches!(self.effect, ImageEffect::RealPic)
            && (self.brightness != 0 || self.contrast != 0)
    }

    /// 한컴 자동 워터마크 프리셋 정합 여부
    /// (effect=GrayScale && brightness=70 && contrast=-50)
    pub fn is_hancom_watermark_preset(&self) -> bool {
        matches!(self.effect, ImageEffect::GrayScale)
            && self.brightness == 70
            && self.contrast == -50
    }

    /// 워터마크 preset 분류
    /// - Some("hancom-watermark"): 한컴 자동 프리셋 정합
    /// - Some("custom"): 사용자 정의 워터마크
    /// - None: 워터마크 아님
    pub fn watermark_preset(&self) -> Option<&'static str> {
        if self.is_hancom_watermark_preset() {
            Some("hancom-watermark")
        } else if self.is_watermark() {
            Some("custom")
        } else {
            None
        }
    }
}
```

이 헬퍼는 이후 dump / JSON / 기타 경로에서 재사용.

### 2.2 영역 1, 2, 3 — Web Canvas CSS filter 적용

#### 변경 위치

`src/renderer/web_canvas.rs::WebCanvasRenderer::render_node` 의 `RenderNodeType::Image(img)` 분기 (line 324):

```rust
RenderNodeType::Image(img) => {
    self.open_shape_transform(&img.transform, &node.bbox);
    if let Some(ref data) = img.data {
        // [신규] image effect / brightness / contrast 를 CSS filter 로 적용
        let filter_str = compose_image_filter(img.effect, img.brightness, img.contrast);
        if let Some(ref f) = filter_str {
            self.ctx.set_filter(f);
        }

        self.draw_image_with_fill_mode(
            data, &node.bbox, img.fill_mode, img.original_size, img.crop,
            img.original_size_hu,
        );

        // [신규] filter reset (다음 그리기 작업에 영향 없도록)
        if filter_str.is_some() {
            self.ctx.set_filter("none");
        }
    }
}
```

#### CSS filter 합성 헬퍼 (web_canvas.rs 또는 별도 module)

```rust
#[cfg(target_arch = "wasm32")]
fn compose_image_filter(effect: ImageEffect, brightness: i8, contrast: i8) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();

    match effect {
        ImageEffect::GrayScale | ImageEffect::Pattern8x8 => {
            parts.push("grayscale(100%)".to_string());
        }
        ImageEffect::BlackWhite => {
            // 회색조 → 고대비로 흑백 모방 (CLI SVG 의 feComponentTransfer discrete 와 등가는 아니나
            // 시각적 근접. 차이는 Stage 5 시각 판정으로 점검)
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

#### CSS filter 매핑 정합성 점검

Stage 1 §"CSS filter ↔ SVG feComponentTransfer 매핑" 에서 확인된 대로 두 식의 결과는 다름:

- CLI SVG: `출력 = 입력 × (1 + c/100) + (b/100 + 0.5×(1 - (1+c/100)))`
- CSS filter: `brightness((100+b)%)` + `contrast((100+c)%)` — 함수 합성 순서 의존

**Stage 5 시각 판정 게이트**:
- 작업지시자 한컴 정답지 vs Web Canvas 출력 비교
- 차이가 작업지시자 허용 범위 내면 본 매핑 유지
- 초과 시 ImageData 직접 변환 (방안 B) 폴백 또는 SVG feComponentTransfer 와 정확 매핑 알고리즘 도입 (방안 C)

#### Trait 변경 회피

`Renderer::draw_image` trait 시그니처를 바꾸지 않고, `render_node` 의 `Image` 분기에서 `set_filter` / 호출 / `set_filter("none")` 패턴으로 처리. 트레이트 호환성 유지.

### 2.3 영역 5 — `rhwp dump` 메타정보 추가

#### 변경 위치 (4 사이트 — `src/main.rs`)

| 라인 | 컨텍스트 |
|------|----------|
| ~1430 | 컨트롤 요약 (1-line) — `Control::Picture(p) => format!("그림(bin_id=...)")` |
| ~1644 | section/paragraph dump — `ctrl[{}] 그림: bin_id=..., w=..., h=..., tac=..., wrap=..., orig=..., crop=...` |
| ~1687 | 일반 dump (`prefix그림: bin_id=..., common=..., orig=..., cur=..., tac=...`) |
| ~1763 / 1800 | 셀 내부 그림 dump (동일 형식) |

#### 출력 형식 정책 (텍스트 파싱 회귀 가드)

**기존 줄 형식 유지** + **별도 줄로 image_attr 추가**:

```
[2] 그림: bin_id=2, common=37128×37180 (131.0×131.2mm), orig=..., cur=..., tac=false
[2]   [image_attr] effect=GrayScale brightness=-50 contrast=70 watermark=custom
```

### 2.4 영역 6 — PageLayerTree JSON `watermark` + `preset` 필드

#### 변경 위치 — `src/paint/json.rs::PaintOp::Image` serialization (line 261~)

기존 (PR #510 보강분 포함):
```rust
",\"effect\":{},\"brightness\":{},\"contrast\":{}"
```

확장 후:
```rust
",\"effect\":{},\"brightness\":{},\"contrast\":{}{}"
```

여기서 마지막 `{}` 는 워터마크 메타 (없으면 빈 문자열):
```rust
let watermark_str = if let Some(preset) = image.image_attr.watermark_preset() {
    format!(",\"watermark\":{{\"preset\":\"{}\"}}", preset)
} else {
    String::new()
};
```

JSON 예시 (워터마크 적용 시):
```json
{
  "type": "image",
  "effect": "grayScale",
  "brightness": -50,
  "contrast": 70,
  "watermark": { "preset": "custom" }
}
```

워터마크 미적용 시:
```json
{
  "type": "image",
  "effect": "realPic",
  "brightness": 0,
  "contrast": 0
}
```

#### schemaVersion 정책

- additive change → **`PAGE_LAYER_TREE_SCHEMA_VERSION` 유지**
- 기존 필드 의미 변경 없음, 기존 필드 제거 없음
- PR #510 의 정책과 동일 정합

### 2.5 PaintOp::Image 의 image_attr 접근 점검

`src/paint/json.rs:261` 의 `PaintOp::Image { image }` 의 `image` 가 `ImageNode` (renderer 의 IR) 임. ImageNode 가 effect/brightness/contrast 만 가지고 있고 ImageAttr 전체를 가지고 있지 않을 수 있음 → 점검 필요. 만약 그렇다면 `is_watermark()` / `watermark_preset()` 를 ImageNode 에도 helper 로 추가.

### 2.6 회귀 테스트 (`tests/issue_516.rs` 신규)

```rust
#[test]
fn issue_516_dump_includes_image_attr_watermark() {
    // rhwp dump samples/복학원서.hwp 의 출력에 [image_attr] 메타가
    // effect=GrayScale brightness=-50 contrast=70 watermark=custom 포함
}

#[test]
fn issue_516_layer_tree_json_includes_watermark_preset() {
    // PageLayerTree JSON 의 PaintOp::Image 에 "watermark":{"preset":"custom"} 포함
    // 복학원서.hwp 의 가운데 엠블렘 (bin_id=2)
}

#[test]
fn issue_516_layer_tree_json_no_watermark_for_normal_image() {
    // 워터마크 없는 그림 (예: 워터마크 미사용 fixture) 의 JSON 에는
    // "watermark" 필드 부재
}

#[test]
fn issue_516_image_attr_helper_methods() {
    // ImageAttr::is_watermark() / is_hancom_watermark_preset() / watermark_preset() 단위 검증
    // 한컴 프리셋 (b=70, c=-50, GrayScale) → "hancom-watermark"
    // 복학원서 패턴 (b=-50, c=70, GrayScale) → "custom"
    // 워터마크 미적용 → None
}
```

Web Canvas 의 CSS filter 적용은 native cargo test 로는 검증 불가 (`#[cfg(target_arch = "wasm32")]`) → Stage 5 작업지시자 시각 판정 게이트로 검증.

## 3. 단계 분리 (3 stages — Stage 3/4/5)

### Stage 3 — 모델 헬퍼 + dump + JSON 구현

**작업:**
1. `src/model/image.rs` 의 `ImageAttr` 에 헬퍼 3 메서드 추가 (`is_watermark`, `is_hancom_watermark_preset`, `watermark_preset`)
2. `src/main.rs` 의 4 dump 사이트에 `[image_attr] ...` 출력 추가
3. `src/paint/json.rs::PaintOp::Image` serialization 에 watermark 필드 추가
4. ImageNode 측에 effect/brightness/contrast 가 있는지 점검 + 헬퍼 호출 가능 위치 확인

**완료 기준:**
- `cargo build --lib` 성공
- `rhwp dump samples/복학원서.hwp` 출력에 `[image_attr] effect=GrayScale brightness=-50 contrast=70 watermark=custom` 포함
- PaintOp::Image JSON 출력에 `"watermark":{"preset":"custom"}` 포함

**산출물**: `mydocs/working/task_m100_516_stage3.md`

### Stage 4 — Web Canvas CSS filter + 회귀 테스트 + WASM

**작업:**
1. `src/renderer/web_canvas.rs::compose_image_filter` 헬퍼 함수 추가
2. `render_node` 의 `Image` 분기에 `set_filter` + 호출 + `set_filter("none")` 패턴 적용
3. `tests/issue_516.rs` 신규: dump / JSON / 헬퍼 단위 테스트 4 건
4. 통합 회귀 점검:
   - `cargo test --lib` (1110 passed 유지)
   - `cargo test --test issue_514` (3 passed)
   - `cargo test --test issue_418/501/svg_snapshot/paint::json` (회귀 0)
5. `cargo clippy --lib -- -D warnings` 통과
6. WASM 빌드 (Docker) → `rhwp-studio/public/rhwp_bg.wasm` 동기화

**완료 기준:**
- 모든 cargo test 통과 + clippy 0
- WASM 빌드 성공

**산출물**: `mydocs/working/task_m100_516_stage4.md`

### Stage 5 — 시각 판정 + 최종 보고

**작업:**
1. `cargo build --release` + `rhwp export-svg samples/복학원서.hwp -o output/svg/task516_after/` (CLI SVG 비교 베이스라인)
2. **작업지시자 시각 판정**:
   - rhwp-studio dev server 재시작 + hard reload
   - 복학원서.hwp 가운데 엠블렘이 한컴 정답지 (`samples/복학원서.pdf`) 와 동일한 흐릿한 워터마크로 표시되는지 확인
   - CLI SVG ↔ Web Canvas 의 시각 정합성 점검 (CSS filter 매핑 차이 허용 범위 검증)
3. CSS filter 시각 차이가 허용 범위 초과 시 → ImageData 폴백 정책 검토 (Stage 5+)
4. 최종 보고서 + orders 갱신
5. local/task516 → local/devel merge → devel push → 이슈 #514 close

**완료 기준:**
- 작업지시자 시각 판정 승인
- WASM 갱신 + studio 동기화

**산출물**: `mydocs/report/task_m100_516_report.md` + orders 갱신

## 4. 위험 영역 (재정리)

| 위험 | 가능성 | 회피책 |
|------|--------|--------|
| CSS filter brightness/contrast 매핑 차이 | 🟧 높음 | Stage 5 작업지시자 시각 판정. 초과 시 ImageData 폴백 |
| `ctx.set_filter` 일부 브라우저 미지원 | 🟢 작음 (Chrome 52+, Firefox 35+, Safari 9.1+) | feature detection 후 폴백 (Stage 5+) |
| Canvas filter 가 다음 그리기 작업에 영향 | 🟨 중간 | drawImage 직후 `set_filter("none")` reset (코드 정책) |
| ImageNode 가 image_attr 를 모두 가지고 있지 않을 가능성 | 🟨 중간 | Stage 3 점검 후 필요 시 ImageNode 에 헬퍼 추가 |
| dump 출력 형식 변경 외부 텍스트 파싱 회귀 | 🟢 작음 | 추가 줄 분리 (`[image_attr] ...`) — 기존 줄 형식 유지 |
| AI 메타정보 schemaVersion 불일치 | 🟢 작음 | additive change, schemaVersion 유지 |
| Task #514 PCX 알파 + CSS filter 결합 | 🟢 작음 (W3C 정합) | Stage 5 시각 판정 (학교 로고 + 엠블렘 동시 검증) |

## 5. 검증 게이트 (요약)

| 게이트 | Stage | 기준 |
|--------|-------|------|
| `cargo build --lib` | 3 | exit 0 |
| `cargo test --lib` (헬퍼 단위 + 기존) | 3 | 1110+ passed 유지 |
| `rhwp dump samples/복학원서.hwp` | 3 | `[image_attr] ... watermark=custom` 출력 |
| PaintOp::Image JSON | 3 | `"watermark":{"preset":"custom"}` 출력 |
| `tests/issue_516.rs` | 4 | 4 tests passed |
| `cargo test --lib` (전체 회귀) | 4 | 회귀 0 |
| svg_snapshot / issue_418 / 501 / 514 / paint::json | 4 | 회귀 0 |
| `cargo clippy --lib -- -D warnings` | 4 | 0 건 |
| WASM 빌드 | 4 | exit 0, binary 크기 회귀 합리적 |
| 작업지시자 Web Canvas 시각 판정 | 5 | 한컴 정답지 + CLI SVG 정합 ★ |

## 6. 본 task 범위 외 (별도 task 후보)

- WMF / EMF / OLE 비표준 포맷의 effect/brightness/contrast 적용
- 한컴 인쇄 워터마크 (글자 워터마크) 처리 — `print_watermark.htm` 영역
- 한컴 GUI 의 picture-props-dialog 워터마크 체크박스 자동 인식 로직 정밀화 (`brightness=70 && contrast=-50` 외 다른 프리셋 지원)
- CSS filter ↔ SVG feComponentTransfer 정확 매핑 알고리즘 (Stage 5 시각 차이 발견 시 별도 task)
- ImageEffect::Pattern8x8 의 정확한 그래픽 표현 (현재는 GrayScale 폴백)

## 7. 메모리 정합

- `feedback_process_must_follow` — Stage 1 → Stage 2 (현재) → Stage 3-5 절차
- `feedback_search_troubleshootings_first` — Stage 1 사전 검색 완료
- `feedback_hancom_compat_specific_over_general` — preset 분류는 case-specific (`hancom-watermark` 정확 정합 + `custom` 폴백)
- `reference_authoritative_hancom` / `feedback_pdf_not_authoritative` — Stage 5 한컴 직접 시각 판정
- `feedback_visual_regression_grows` — Stage 5 시각 판정 필수
- `feedback_commit_reports_in_branch` — task 브랜치 commit
- `feedback_close_issue_verify_merged` — close 전 commit devel 머지 검증
- `feedback_self_verification_not_hancom` — IR 변경 없음 (라운드트립 보존)

## 8. 승인 게이트

1. 본 구현 계획서 승인 → Stage 3 구현 시작
2. Stage 3 완료 보고서 승인 → Stage 4 진행
3. Stage 4 완료 보고서 승인 → Stage 5 시각 판정
4. Stage 5 시각 판정 승인 → 최종 보고 + 머지 + 이슈 close

## 9. 다음 단계

작업지시자 본 구현 계획서 승인 후 Stage 3 (모델 헬퍼 + dump + JSON 구현) 진행.

---

# 보강 — Stage 5+ 다층 레이어 도입 (2026-05-02 작업지시자 확정)

## 10. 배경 및 결정 (Discussion #529)

Stage 4 까지의 정정 (CSS filter) 후 작업지시자 시각 판정에서 3 결함 식별 (배경 투명/hit-test/multiply blend). 이는 **단일 Canvas 평면의 구조적 한계** 로, 본질 정정에 다층 레이어 인프라 필요.

기술 조사 보고서 [`mydocs/tech/multi_layer_rendering_strategy.md`](../tech/multi_layer_rendering_strategy.md) + Discussion [#529](https://github.com/edwardkim/rhwp/discussions/529) 의 후보 3 안 비교 결과:

- **본 task #516 의 Stage 5+: 후보 C (HTML Hybrid) 도입 ✅ 확정**
- **M200 v2.0.0: 후보 B (WebGPU) 단계적 도입 ✅ 확정** (별도 사이클, DTP 정체성 본격화)

본 task 가 **다층 레이어 인프라 도입의 첫 사이클** 이 됨.

## 11. Stage 5+ 의 본질

### 11.1 후보 C 의 구조

```
<div class="page-container">
  <canvas class="layer-flow"></canvas>             <!-- 본문 텍스트 + 표 + 어울림 그림 (Canvas 2D, 기존 코드 재사용) -->
  <div class="overlay-behind">                      <!-- BehindText 그림 -->
    <img src="data:..." style="
      position:absolute; left:..px; top:..px;
      width:..px; height:..px;
      filter:grayscale(1) brightness(.7) contrast(.5);
      mix-blend-mode:multiply;
      pointer-events:none;" />
  </div>
  <div class="overlay-front">                       <!-- InFrontOfText 그림 -->
    <img src="data:..." style="position:absolute; ..." />
  </div>
</div>
```

- 본문 (어울림/위아래 그림 포함) → Canvas 2D 한 layer
- BehindText / InFrontOfText 그림 → HTML `<img>` overlay
- 결함 1, 2, 3 모두 자연스럽게 해결

### 11.2 결함 ↔ 정정 매핑

| 결함 | 정정 (후보 C) |
|------|--------------|
| 1. 엠블럼 흰색 배경 투명 | `<img>` 에 `mix-blend-mode: multiply` — 흰색 = 투명 효과 (워터마크 본질) |
| 2. BehindText 위 텍스트 클릭 안 됨 | `<img>` 에 `pointer-events: none` — 클릭이 본문 Canvas 까지 통과 |
| 3. 워터마크 multiply blend | mix-blend-mode 가 한컴 워터마크 동작 정합 |

## 12. Stage 5+ 의 sub-stages 분리

본 작업은 4 sub-stages 로 분리:

### Stage 5.1 — PageLayerTree 의 wrap 모드별 분류 (Rust)

**목표**: PaintOp 또는 LayerNode 트리에 wrap 모드 정보를 명시적으로 노출하여 web 측이 분리 렌더링 가능하도록.

**작업:**
1. `src/paint/json.rs::PaintOp::Image` 에 `wrap` 필드 추가 (BehindText / InFrontOfText / Square / TopAndBottom / None / Through 등)
2. `src/renderer/render_tree.rs::ImageNode` 에 `text_wrap: TextWrap` 필드 추가 (기존에 없으면)
3. layout 단계에서 Picture 의 `common.text_wrap` 을 ImageNode 로 전파
4. PageLayerTree JSON 직렬화 검증

**산출물**:
- `mydocs/working/task_m100_516_stage5_1.md`
- `tests/issue_516.rs` 에 wrap 필드 검증 테스트 추가

### Stage 5.2 — Layer 분리 정책 (rhwp-studio TypeScript)

**목표**: PageLayerTree JSON 을 TypeScript 가 받아 wrap 모드별로 분리 렌더링.

**작업:**
1. `rhwp-studio/src/core/wasm-bridge.ts` 또는 `view/page-renderer.ts` 에 layer 분리 함수 추가:
   - `splitByWrap(layerTree)` → `{ flowOps, behindImages, frontImages }`
2. 본문 (flowOps) 은 기존 Canvas 경로로 렌더링
3. behindImages / frontImages 는 별도 `<img>` 생성 → DOM 에 overlay 로 추가
4. base64 data URL 캐싱 (이미지 중복 방지)

**산출물**:
- `mydocs/working/task_m100_516_stage5_2.md`
- e2e 테스트 (rhwp-studio/e2e/) — 복학원서.hwp 의 다층 레이어 렌더링 확인

### Stage 5.3 — CSS filter / blend mode 적용 정책

**목표**: 워터마크 효과 / 그림 효과 의 정확한 시각 적용.

**작업:**
1. `<img>` 에 inline style 로 적용:
   - `filter: grayscale(...) brightness(...) contrast(...)` (Stage 4 의 매핑 재사용)
   - `mix-blend-mode: multiply` (워터마크일 때만)
2. 워터마크 식별 로직: PageLayerTree JSON 의 `watermark.preset` 필드 (Stage 3 의 정정) 활용
3. CSS filter 매핑 차이 점검 (Stage 1 의 위험 영역 — Canvas 와 DOM 의 filter 적용 정합성)

**산출물**:
- `mydocs/working/task_m100_516_stage5_3.md`

### Stage 5.4 — 시각 판정 + 회귀 검증 + 최종 보고

**목표**: 작업지시자 시각 판정 통과 + 다층 레이어 도입의 다른 fixture 회귀 0 확인.

**작업:**
1. 복학원서.hwp 시각 판정:
   - 결함 1 (배경 투명) ✅ 확인
   - 결함 2 (텍스트 클릭) ✅ 확인
   - 결함 3 (워터마크 multiply blend) ✅ 확인
2. 다른 BehindText / InFrontOfText 그림 fixture 회귀 점검 (sweep)
3. Canvas visual diff (PR #498) 의 layer 분리 정책 점검 + 필요 시 갱신
4. cargo test --lib (1110+) + svg_snapshot 6/6 + issue_418/501/514/516 회귀 0
5. WASM 빌드 + studio 동기화
6. 최종 보고서 작성: `mydocs/report/task_m100_516_report.md`
7. orders 갱신
8. local/task516 → local/devel merge → devel push → 이슈 #516 close

## 13. 위험 영역 (Stage 5+)

| 위험 | 가능성 | 회피책 |
|------|--------|--------|
| BehindText/InFrontOfText 그림이 있는 다른 fixture 회귀 | 🟧 중간 | Stage 5.4 fixture sweep + 작업지시자 시각 판정 |
| layer 간 픽셀 정확 정렬 (Canvas px ↔ DOM px) | 🟨 중간 | 동일 좌표계 + DPR scale 일치 정책 |
| Canvas visual diff (PR #498) 의 합성 비교 정책 영향 | 🟨 중간 | 본문 Canvas 만 비교 → BehindText/InFrontOfText 별도 비교 정책 |
| 인쇄 (window.print) 시 다층 layer 정상 출력 | 🟢 작음 | 브라우저 native 처리 (DOM + Canvas 모두 인쇄) |
| 줌 / DPR 변경 시 overlay 크기 동기화 | 🟨 중간 | overlay 컨테이너에 `transform: scale(...)` 일괄 적용 또는 각 img 의 inline width/height 재계산 |
| 메모리 (img element 다수 생성) | 🟢 작음 | 페이지 단위 cleanup + base64 캐시 |

## 14. 통합 검증 게이트 (Stage 5+)

| 게이트 | 기준 |
|--------|------|
| `cargo test --lib` | 1110+ passed (회귀 0) |
| `cargo test --test issue_516` | 5+ passed (wrap 필드 검증 추가) |
| svg_snapshot / issue_418 / issue_501 / issue_514 | 회귀 0 |
| `cargo clippy --lib` | 0 건 |
| WASM 빌드 | exit 0 |
| **작업지시자 web Canvas 시각 판정** | 결함 1/2/3 모두 통과 ★ |
| 다른 fixture (BehindText/InFrontOfText 사용) 회귀 점검 | 작업지시자 시각 판정 |

## 15. 메모리 정합 (Stage 5+ 추가)

- `feedback_visual_regression_grows` — 다층 레이어 도입은 시각 회귀가 가장 중요한 게이트. 작업지시자 시각 판정 필수
- `feedback_v076_regression_origin` — BehindText/InFrontOfText 가 있는 다른 fixture 회귀 점검 의무
- (신규 패턴) DTP 정체성 (Discussion #529 Appendix) — 본 task 가 후보 C 도입의 첫 사이클임을 보고서에 명시

## 16. 다음 단계

작업지시자 본 보강 승인 후 Stage 5.1 (PageLayerTree wrap 분류) 진행.
