# Task #516 Stage 1 완료 보고서 — 본질 진단 + 워터마크 임계값 결정

## 진단 결과 요약

본 task 의 본질 6 영역에 대해 1차 진단 완료:

| 영역 | 진단 결과 |
|------|----------|
| 1. Web Canvas effect 적용 | ❌ `web_canvas.rs` 에 `ctx.filter` 사용 0 건 — 완전히 누락 |
| 2. Web Canvas brightness/contrast 적용 | ❌ 동일 (1과 같은 함수에 합성 추가 필요) |
| 3. effect + bc 합성 painter ordering | CSS filter 단일 문자열로 합성 가능 — 함수 순서 결정 필요 |
| 4. PCX 배경 투명도 + 워터마크 결합 | Task #514 의 RGBA PNG (alpha 채널) 가 CSS filter 통과 시 보존됨 (Canvas 표준) |
| 5. `rhwp dump` image_attr 메타정보 | 4 개 dump 사이트 (`src/main.rs:1430/1644/1687/1763/1800`) 모두 누락 |
| 6. AI 메타정보 (PageLayerTree JSON `watermark` 필드) | `paint/json.rs::PaintOp::Image` 에 신규 필드 추가 필요 (additive) |

## samples/ image_attr sweep 결과

samples/ 173 HWP 전체 sweep (Picture controls 153 개) 결과:

| 분류 | 카운트 | 사례 |
|------|--------|------|
| 비기본 image_attr | 3 | — |
| **워터마크 후보** (effect + bc 모두 존재) | **1** | `복학원서.hwp s0:pi=2` (effect=GrayScale, b=-50, c=70) |
| effect-only (b=c=0) | 2 | `pr-149.hwp s0:pi=4` (GrayScale), `s0:pi=6` (BlackWhite) |
| bc-only (effect=RealPic) | 0 | — |

**samples/ 전체에서 워터마크 패턴은 복학원서.hwp 1 건뿐.** 단일 사례지만 본 task 의 정합 fixture 로 충분.

## 워터마크 식별 임계값 결정

### 한컴 도구 동작 (작업지시자 정합)

> **한컴 도구**: 이미지 → 회색조 → 워터마크 효과
>
> **워터마크 효과 체크 시 자동 적용 프리셋: 밝기 70%, 대비 -50%**

이는 rhwp-studio 의 `picture-props-dialog.ts:1604` 의 워터마크 체크박스 핸들러와 정합:
```typescript
this.picBrightnessInput.value = '70';
this.picContrastInput.value = '-50';
```

자동 인식 조건도 동일:
```typescript
this.picWatermarkCheck.checked = (pp.brightness === 70 && pp.contrast === -50);
```

### 복학원서.hwp 의 IR 정합 (작업지시자 확인 — 가능성 A 확정)

samples sweep 결과 복학원서.hwp 의 IR 은 **`effect=GrayScale, brightness=-50, contrast=+70`**.

한컴 자동 워터마크 프리셋 (`brightness=+70, contrast=-50`) 과 부호와 위치가 swap.

**작업지시자 정합:**

> 이 문서의 편집자가 의도적으로 이미지를 회색조 설정 한 후, 다시 워터마크 효과 적용한 겁니다.

**편집 이력 재구성:**

1. 편집자가 그림 (고려대학교 엠블렘 JPEG, bin_id=2, 131×131mm) 삽입
2. **회색조 설정** 적용 → `effect = GrayScale`
3. **워터마크 효과 체크** → 자동 프리셋 → `brightness = 70, contrast = -50`
4. 편집자가 워터마크 적용 후 brightness/contrast 슬라이더로 추가 미세 조정 → 최종 `brightness = -50, contrast = +70`

또는 회색조 + 워터마크 적용 후 사용자가 직접 값 변경. 어느 흐름이든 **편집자 의도** 적 사용자 정의 워터마크 (`custom` preset).

**시사점:**

- HWP IR 부호 규약과 한컴 GUI 부호 표시 매핑은 **swap 없음** — i8 값 그대로 저장
- 한컴 GUI "밝기 70%" = IR `brightness=+70` (직접 매핑)
- 한컴 GUI "대비 -50%" = IR `contrast=-50` (직접 매핑)
- 복학원서.hwp 의 `b=-50, c=+70` 은 **custom 워터마크** — 한컴 자동 프리셋과 다른 사용자 정의 값

이로써 임계값 결정 + preset 분류 로직이 명확해진다.

### 워터마크 식별 임계값 (Stage 1 확정 — 보수적 OR 정의 유지)

위 가능성 A/B 결정과 무관하게 본 task 의 식별 로직은 보수적으로:

```
watermark = (effect != RealPic) AND (brightness != 0 OR contrast != 0)
```

**근거:**
1. samples sweep 결과 워터마크 후보가 1 건뿐이라 통계적 임계값 도출 불가 → 보수적 OR 정의
2. 한컴 자동 프리셋 정합: `brightness == 70 && contrast == -50` (또는 가능성 B 의 swap 형태) 는 본 정의의 부분 집합
3. effect-only (pr-149.hwp 의 GrayScale / BlackWhite) 는 단순 흑백 변환이지 워터마크 아님 → 정의로 구분 가능
4. 한컴 도구 정확 매핑 (가능성 A/B 결정) 은 dump 메타정보로 노출하고 (영역 5), 더 정확한 식별은 향후 task 에서 정교화

### 영역 6 — preset 분류 (AI 메타정보)

dump 와 PageLayerTree JSON 의 `watermark` 메타에 두 가지 정보 노출:

| preset | 조건 |
|--------|------|
| `"hancom-watermark"` | `effect=GrayScale && brightness=70 && contrast=-50` (한컴 자동 프리셋 정확 정합) |
| `"custom"` | 그 외 워터마크 정의 만족하는 모든 케이스 (사용자 정의 — 복학원서 등) |
| `null` (또는 필드 부재) | 워터마크 아님 |

이로써 AI 활용 시:
- **워터마크 적용 여부** 식별 (보수적 정의)
- **한컴 표준 프리셋 정합 여부** 식별 (preset 분류)

복학원서.hwp 의 가운데 엠블렘은 `preset = "custom"` 으로 분류. 향후 다른 fixture 추가 시 더 많은 preset 식별 가능 (예: 인쇄 워터마크, 글자 워터마크 등).

## Web Canvas 정정 방안 검토

### 방안 A — CSS filter 문자열 (권장)

```rust
// web_canvas.rs::draw_image
let mut filter_parts = Vec::new();
match img_effect {
    ImageEffect::GrayScale | ImageEffect::Pattern8x8 => filter_parts.push("grayscale(100%)".to_string()),
    ImageEffect::BlackWhite => {
        filter_parts.push("grayscale(100%)".to_string());
        filter_parts.push("contrast(1000%)".to_string());  // 흑백 임계화 모방
    }
    ImageEffect::RealPic => {}
}
if brightness != 0 {
    let css_b = (100.0 + brightness as f64) / 100.0;
    filter_parts.push(format!("brightness({:.4})", css_b));
}
if contrast != 0 {
    let css_c = (100.0 + contrast as f64) / 100.0;
    filter_parts.push(format!("contrast({:.4})", css_c));
}
if !filter_parts.is_empty() {
    self.ctx.set_filter(&filter_parts.join(" "));
} else {
    self.ctx.set_filter("none");
}
```

**장점**: 간단, 빠름 (GPU 가속 가능), 알파 채널 보존
**단점**: SVG feComponentTransfer 와 시각 미세 차이 가능

### 방안 B — ImageData 직접 조작

각 픽셀에 brightness/contrast 변환 적용 후 `putImageData`. 정확하지만 느림 + 알파 채널 처리 복잡.

### 방안 C — 오프스크린 SVG → drawImage

OffscreenCanvas 또는 image element 에 SVG 임베드. 복잡, 비동기 처리 필요.

**선택**: **방안 A (CSS filter)**. 시각 미세 차이는 Stage 5 작업지시자 시각 판정 게이트로 점검. 차이 발견 시 Stage 5+ 에서 방안 B 폴백.

## CSS filter ↔ SVG feComponentTransfer 매핑

CLI SVG 의 brightness/contrast 적용 (svg.rs:1217-1248):
```
slope = (100 + contrast) / 100
intercept = (0.5 - 0.5 * slope) + brightness/100
```

CSS filter 의 brightness/contrast (W3C):
- `brightness(N)`: 출력 = 입력 × N (multiplicative, N=1.0 가 원본)
- `contrast(N)`: 출력 = (입력 - 0.5) × N + 0.5 (linear about 0.5)

**HWP brightness=-50, contrast=70 매핑:**
- CLI SVG: slope=1.7, intercept=-0.85 → 출력 = 입력 × 1.7 - 0.85
- CSS filter: brightness(0.5) contrast(1.7) → 출력 = ((입력 × 0.5) - 0.5) × 1.7 + 0.5 = 입력 × 0.85 - 0.35

**두 식의 결과가 다름.** CSS filter 의 함수 적용 순서와 정의가 SVG feComponentTransfer 와 다름.

**결정**: 시각 차이의 정도를 Stage 5 시각 판정으로 점검. 차이가 작업지시자 허용 범위 내면 방안 A 유지, 초과하면 ImageData 직접 변환 (방안 B 매핑) 또는 SVG feComponentTransfer 와 등가 매핑 알고리즘 도입.

## 변경 영역 정합 (Stage 2 구현 계획서 입력)

| 파일 | 영역 | 예상 변경 |
|------|------|----------|
| `src/renderer/web_canvas.rs` | 1, 2, 3 | `draw_image` 의 image emit 분기 앞에 CSS filter 적용 + 적용 후 reset |
| `src/main.rs` | 5 | dump 의 그림 출력 4 사이트에 `[image_attr] effect=... b=... c=... watermark=...` 추가 |
| `src/paint/json.rs` | 6 | `PaintOp::Image` 직렬화에 `"watermark": true/false` 추가 + 판정 헬퍼 |
| `src/model/image.rs` (선택) | 6 | `ImageAttr::is_watermark() -> bool` 메서드 추가 (재사용 가능) |
| `tests/issue_516.rs` (신규) | — | 회귀 테스트 (CSS filter 적용, dump 메타, JSON `watermark`) |

## 다른 영역 의존성 점검

### Task #514 의 PCX RGBA 정합

CSS filter 가 RGBA 의 알파 채널을 보존하는지 확인:
- **W3C CSS Filter Spec**: filter 는 RGBA 채널 모두 처리. `grayscale(100%)` 는 RGB 만 영향, alpha 보존
- **Canvas 2D 표준**: `ctx.filter` 적용 후 `drawImage` → 결과는 알파 보존
- ✅ Task #514 의 흰색→투명 PCX → PNG 가 web Canvas 에서 워터마크 효과와 결합 가능

### PR #510 의 PageLayerTree JSON

PR #510 으로 `brightness`/`contrast` 가 JSON 에 직렬화됨. 본 task 는 추가로:
- `effect` 는 이미 직렬화 (`paint/json.rs:727`)
- `watermark: bool` 신규 추가 (additive, schemaVersion 유지)

## 위험 영역 (Stage 2 입력)

| 위험 | 가능성 | 회피책 |
|------|--------|--------|
| CSS filter 의 brightness/contrast 매핑 차이 | 🟧 높음 | Stage 5 작업지시자 시각 판정. 초과 시 ImageData 폴백 |
| `ctx.set_filter` 일부 브라우저 미지원 | 🟢 작음 (Chrome 52+, Firefox 35+, Safari 9.1+) | feature detection 후 폴백 |
| Canvas filter 가 다음 그리기 작업에 영향 | 🟨 중간 | drawImage 직후 `set_filter("none")` 으로 reset |
| 워터마크 임계값 한컴 정합 미입증 | 🟨 중간 | Stage 5 시각 판정 + 향후 fixture 추가 시 정교화 |

## 다음 단계

Stage 1 완료 보고서 승인 후 **Stage 2 구현 계획서** 작성:
- Web Canvas CSS filter 적용 정확한 코드 위치 + reset 정책
- dump 출력 형식 최종 확정 (텍스트 파싱 회귀 가드 포함)
- JSON `watermark` 필드 schemaVersion 영향 (PR #510 정합)
- 회귀 테스트 설계

## 산출물

- 본 보고서 (`mydocs/working/task_m100_516_stage1.md`)
- 임시 진단 테스트 (`tests/diag_image_attr.rs`) 는 Stage 1 종료와 함께 제거 (samples sweep 결과만 보존)
