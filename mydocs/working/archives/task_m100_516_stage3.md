# Task #516 Stage 3 완료 보고서 — 모델 헬퍼 + dump + JSON 구현

## 변경 요약

| 파일 | 영역 | 변경 |
|------|------|------|
| `src/model/image.rs` | 헬퍼 | `ImageAttr::is_watermark()` / `is_hancom_watermark_preset()` / `watermark_preset()` 3 메서드 추가 |
| `src/main.rs` | 5 (dump) | 1-line 요약 + 일반 dump + 셀 내부 그림 3 사이트에 `[image_attr]` 줄 추가 |
| `src/paint/json.rs` | 6 (AI JSON) | `PaintOp::Image` 직렬화에 `watermark.preset` 필드 조건부 추가 |

**소스 변경: ~+50 / -8** (3 파일)

## 변경 상세

### 1. ImageAttr 헬퍼 (`src/model/image.rs`)

```rust
impl ImageAttr {
    pub fn is_watermark(&self) -> bool {
        !matches!(self.effect, ImageEffect::RealPic)
            && (self.brightness != 0 || self.contrast != 0)
    }

    pub fn is_hancom_watermark_preset(&self) -> bool {
        matches!(self.effect, ImageEffect::GrayScale)
            && self.brightness == 70
            && self.contrast == -50
    }

    pub fn watermark_preset(&self) -> Option<&'static str> {
        if self.is_hancom_watermark_preset() { Some("hancom-watermark") }
        else if self.is_watermark() { Some("custom") }
        else { None }
    }
}
```

### 2. dump 메타정보 (`src/main.rs`)

3 사이트 정정 (1430 1-line 요약 / 1644 셀 내부 / 1685 일반 dump):

**일반 dump (1685):**
```
그림: bin_id=2, common=37128×37180 (131.0×131.2mm), ...
  [image_attr] effect=GrayScale brightness=-50 contrast=70 watermark=custom
  border_x=...
```

**셀 내부 (1644):**
```
ctrl[1] 그림: bin_id=2, ...
  [image_attr] effect=GrayScale brightness=-50 contrast=70 watermark=custom
```

**1-line 요약 (1430):**
```
그림(bin_id=2, w=37128, h=37180, tac=false, watermark=custom)
```

**미적용 사이트** (1770/1807 머리말/꼬리말 1-line 요약): 30자 truncate 가 발생하는 위치라 생략. watermark 정보가 더 가치 있는 사이트 (전체 dump) 에서 노출되므로 충분.

### 3. PageLayerTree JSON `watermark` 필드 (`src/paint/json.rs`)

`PaintOp::Image` 직렬화에 조건부 필드 추가:

```rust
let attr = ImageAttr {
    brightness: image.brightness,
    contrast: image.contrast,
    effect: image.effect,
    bin_data_id: image.bin_data_id,
};
if let Some(preset) = attr.watermark_preset() {
    let _ = write!(buf, ",\"watermark\":{{\"preset\":\"{}\"}}", preset);
}
```

JSON 출력 예시 (워터마크 적용):
```json
{
  "type": "image",
  "effect": "grayScale",
  "brightness": -50,
  "contrast": 70,
  "watermark": { "preset": "custom" },
  "transform": ...
}
```

JSON 출력 예시 (워터마크 미적용):
```json
{
  "type": "image",
  "effect": "realPic",
  "brightness": 0,
  "contrast": 0,
  "transform": ...
}
```

**schemaVersion 정책**: additive change, `PAGE_LAYER_TREE_SCHEMA_VERSION` 유지 (PR #510 정합).

## 검증 결과

### 게이트 통과 현황

| 게이트 | 결과 |
|--------|------|
| `cargo build --lib` | ✅ Finished |
| `cargo build --bin rhwp` | ✅ Finished |
| **`cargo test --lib`** | ✅ **1110 passed** (회귀 0) |
| `cargo test --lib paint::json` | ✅ 4 passed |

### dump 출력 검증 (`samples/복학원서.hwp`)

```
[2] 그림: bin_id=1, common=5776×6592 (20.4×23.3mm), ...   ← 학교 로고 (PCX)
[2]   [image_attr] effect=RealPic brightness=0 contrast=0 watermark=none

[1] 그림: bin_id=2, common=37128×37180 (131.0×131.2mm), ...  ← 엠블렘 (JPEG)
[1]   [image_attr] effect=GrayScale brightness=-50 contrast=70 watermark=custom ★
```

학교 로고 = `none` (워터마크 미적용)
엠블렘 = `custom` (사용자 정의 워터마크) — Stage 1 의 분류 정합 ✅

### PaintOp::Image JSON 검증

기존 paint::json 단위 테스트 4 건 통과:
- `serializes_backend_replay_payload_fields` 의 ImageNode (BlackWhite + b=-50, c=70) 가 watermark="custom" 으로 분류됨 → contains assertion 들은 모두 통과 (영향 없음)
- 별도 watermark assertion 은 Stage 4 의 `tests/issue_516.rs` 에서 추가

## 위험 점검

| 위험 | 결과 |
|------|------|
| 헬퍼 메서드 추가로 다른 사용처 회귀 | ✅ 0 (`cargo test --lib` 1110 passed) |
| dump 출력 형식 변경 외부 파싱 회귀 | 🟢 작음 — 추가 줄 분리 (`[image_attr] ...`) 로 기존 줄 형식 유지 |
| JSON additive 변경 downstream 영향 | 🟢 매우 작음 — schemaVersion 유지, 기존 필드 의미 변경 없음 |
| ImageNode ↔ ImageAttr 임시 변환 비용 | 🟢 매우 작음 — Copy trait, no-alloc |

## 다음 단계

Stage 3 완료 보고서 승인 후 **Stage 4** 진행:
- `src/renderer/web_canvas.rs::compose_image_filter` 헬퍼 + `set_filter` / draw / `set_filter("none")` 패턴
- `tests/issue_516.rs` 신규 (4 tests — dump / JSON / 헬퍼 단위 / 워터마크 미적용)
- 통합 회귀 (lib + svg_snapshot + issue_418/501/514 + paint::json)
- WASM 빌드 + studio 동기화

## 산출물

- `src/model/image.rs` (헬퍼 3 메서드)
- `src/main.rs` (dump 3 사이트)
- `src/paint/json.rs` (watermark 필드)
- 본 보고서 (`mydocs/working/task_m100_516_stage3.md`)
