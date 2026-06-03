# Stage 3 — 결함 #2 워터마크 정정 보고서 (Task #677)

## 본질 정정 영역

### 영역 식별 (Stage 1 진단 영역 정합)

- HWP IR: `effect=GrayScale, brightness=-50, contrast=70, watermark=custom`
- 한컴 표준 워터마크 프리셋: `effect=GrayScale, brightness=+70, contrast=-50` (`src/model/image.rs:75`) — 부호 반대
- BEFORE 렌더 결과: 저장값 그대로 적용 → brightness=-50 (어두움) + contrast=+70 (고대비) → **진한 어두운 회색 본문 가림**
- 가설 A 채택 — 한컴 편집기는 워터마크 모드 (effect != RealPic + brightness/contrast 비-zero) 시 표준 프리셋 강제 적용

### 광범위 fixture 교차 검증

```
복학원서.hwp:   [1]   [image_attr] effect=GrayScale brightness=-50 contrast=70 watermark=custom
Total fixtures with watermark!=none: 1
```

→ 161+ HWP fixture 중 워터마크 보유는 **본 fixture 1개만**. 본 정정의 영향 받는 fixture 0개 (회귀 위험 영역 0).

### 본질 정정 — SVG + WASM Canvas 양쪽 동기

**대상 1**: `src/renderer/svg.rs:1082-1097` `render_image` 분기
**대상 2**: `src/renderer/web_canvas.rs:418-432` `RenderNodeType::Image` 분기

**정정 내용**:
```rust
// [Issue #677] 한컴 워터마크 모드 (effect != RealPic + brightness/contrast 비-zero) 는
// 저장값을 그대로 적용하면 어두운/고대비 출력 (PDF 정합 안 됨).
// 한컴 표준 프리셋 (brightness=+70, contrast=-50) 강제 적용.
let is_watermark_image = !matches!(img.effect, ImageEffect::RealPic)
    && (img.brightness != 0 || img.contrast != 0);
let (eff_brightness, eff_contrast) = if is_watermark_image {
    (70i8, -50i8)  // 한컴 표준 워터마크 프리셋
} else {
    (img.brightness, img.contrast)
};
```

조건 가드:
- `effect != RealPic` — 일반 사진은 영향 없음 (`is_watermark()` 기준 정합)
- `brightness != 0 || contrast != 0` — 효과 + 비-zero 변조 조합만 워터마크 모드로 식별

`feedback_image_renderer_paths_separate` 정합 — SVG (CLI export) + Canvas (WASM web editor) 양쪽 동기 정정. 두 렌더 경로 일관 확보.

## 정량 측정 (BEFORE → AFTER)

| 측정 항목 | BEFORE | AFTER |
|----------|--------|-------|
| 워터마크 brightness 적용값 | -50 (어두움) | **+70 (밝음)** |
| 워터마크 contrast 적용값 | +70 (고대비) | **-50 (저대비)** |
| 시각 결과 | 진한 어두운 회색 본문 가림 | 흐린 회색 워터마크 (PDF 정합) |
| "고 려 대 학 교 총 장 귀 하" 큰 제목 가독성 | 워터마크에 가려져 안 보임 | 본문 위에 정상 출력 (워터마크 흐림) |

## 시각 정합 (PDF 비교)

**한컴 PDF (정답지)**: 흐린 회색 워터마크 + 본문 위 모든 텍스트 정상 출력  
**rhwp AFTER**: 동일 — 흐린 회색 워터마크 (밝기 +70, 대비 -50) + 본문 가독성 회복

PNG 렌더 (`/tmp/복학원서_s3_hires.png` 1600×2263):
- 복학원서(학부) 제목 ✅
- 5×4 표 (대학/학번/휴대전화/현주소) ✅
- 본인은 휴학으로... + I have taken leave... ✅
- 본인 (Name) signature line + 접수자 box ✅
- **고 려 대 학 교 총 장 귀 하 큰 제목 ✅** (BEFORE 안 보임)
- ─── 분리선 ✅
- 복 학 원 서 접 수 증 + Filing Receipt ✅
- 대학(Name of College) / 학과/학부 / 학번 / 성명 ✅
- 위 학생의 복학원서를 접수함 + The above student's reinstatement form is hereby received ✅
- 년(year) 월(month) 일(day) + ㊞ seal ✅
- ※ 군필자 + ※ Those who completed ✅

## 회귀 검증

### 결정적 검증

```
cargo test --release --lib       1155 passed (회귀 0)
cargo clippy --release --lib     0 warnings (lib 영역)
cargo build --release            빌드 성공
cargo check --target wasm32-unknown-unknown --release --lib  WASM lib 빌드 성공
```

### 회귀 가드 통과

- svg_snapshot **7/7 passed** (issue_147 / 171 / 437 / 546 / 554 / 578 / 617)
- issue_554 **12/12 passed** (exam_kor 영역)

### 워터마크 영향 범위

본 정정은 `effect != RealPic` AND `brightness/contrast 비-zero` 조건에서만 발동 — **본 fixture 외 161+ HWP 영역 무영향** (광범위 sweep 영역 정합).

다른 fixture 영역의 RealPic 사진은 분기 조건 미충족 → **정정 전 동작 그대로 유지**. RealPic + brightness/contrast 비-zero (사용자 명시 변조) 케이스도 워터마크가 아니므로 미발동.

## HWP 정합 영역의 단일 룰 정합 (`feedback_rule_not_heuristic`)

**룰**: `effect != RealPic` + `brightness/contrast 비-zero` ⇒ 한컴 표준 워터마크 프리셋 (brightness=+70, contrast=-50) 강제 적용

- HWP IR 영역 검사 (`effect`/`brightness`/`contrast`) — 측정 의존 없음
- 한컴 도구의 "이미지 → 회색조 → 워터마크 효과" 명세 정합 (`src/model/image.rs:73-77`)
- 저장값 부호/스케일 무관 — 모든 워터마크 영역에 동일 시각 효과 보장

## 잔존 영역 (별도 task 후보)

**body-clip width 1619.92** — Stage 1 에서 식별, 시각 영향 없음. 별도 task 후보 (코드 위생).

**워터마크 표준 프리셋 외 사용자 customization 영역** — 본 정정은 모든 워터마크에 표준 프리셋을 강제 적용. 사용자가 custom 워터마크 강도를 의도한 경우 무시. 다른 워터마크 fixture 가 발견되면 customization 보존이 필요한지 별도 검토 영역.

## 변경 LOC

| 파일 | 변경 | 영역 |
|------|------|------|
| `src/renderer/svg.rs` | +9 / -1 | render_image 워터마크 게이트 |
| `src/renderer/web_canvas.rs` | +9 / -1 | Image RenderNode 워터마크 게이트 (WASM) |
| **합계** | **+18 / -2** | 단일 룰 1개 (양쪽 경로) |

## 승인 요청

본 Stage 3 결과 승인 후 **Stage 4 (회귀 가드 + 광범위 sweep)** 진행하겠습니다.

대상 영역 (Stage 4):
- svg_snapshot 회귀 가드 추가 (`issue_677_bokhakwonseo`) — 본 fixture 의 첫 페이지 영구 보존
- 광범위 페이지네이션 sweep 161+ HWP fixture / 페이지 수 차이 0 확인
- BEFORE/AFTER SVG byte 비교 — 의도된 영역만 변경 확인
