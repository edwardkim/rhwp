# Task #514 구현 계획서 — PCX 포맷 → PNG 변환 + MIME 인식 추가

## 1. 결함 요약 (Stage 1 진단 결과)

`samples/복학원서.hwp` 의 s0:pi=0 학교 로고가 **PCX 포맷** (`BIN0001.PCX`) 으로 임베드되어 있고 rhwp 의 `detect_image_mime_type` 가 PCX 를 인식하지 못해 `application/octet-stream` 으로 emit → 브라우저 렌더링 불가. 본질 정정은 **PCX 를 PNG 로 변환 후 emit**.

## 2. 사전 조사 결과

### 2.1 samples/ 비표준 포맷 sweep (173 HWP 파일)

| 포맷 | 파일 수 | 대표 |
|------|---------|------|
| **PCX** | 1 | 복학원서.hwp (본 task) |
| WMF | 7 | pic-in-table-01, pic-in-head-01, hwpspec |
| EMF | 2 | exam_social |
| OLE | 3 | 한셀OLE, bitmap |

**PCX 는 본 fixture 한정**. WMF/EMF/OLE 는 별도 task (본 task 범위 외).

### 2.2 PCX 디코더 옵션

| 옵션 | 평가 |
|------|------|
| `image` crate v0.25 PCX feature | ❌ **지원 안 함** (image v0.25 features 에 `pcx` 없음) |
| `pcx` crate v0.2.5 | ✅ Lightweight PCX 디코더, 단순 API |
| 자체 구현 | ❌ 디코더 직접 작성은 본 task 범위 초과 |

**선택**: `pcx` crate v0.2.5 추가.

### 2.3 기존 dependency 정합

`Cargo.toml` 에 이미 `image = { version = "0.25", default-features = false, features = ["bmp", "png"] }` 존재 → **PNG 인코딩 경로는 이미 사용 가능**. PCX 디코드 → image::RgbaImage → PNG 인코드 흐름 가능.

## 3. 변경 영역

### 3.1 변경 파일

| 파일 | 변경 |
|------|------|
| `Cargo.toml` | `pcx = "0.2"` dependency 추가 |
| `src/renderer/svg.rs` | `detect_image_mime_type` 에 PCX 시그니처 추가 + PCX→PNG 변환 함수 추가 + image emit 경로에서 PCX detect 시 변환 호출 |
| `tests/issue_514.rs` (신규) | 회귀 테스트 — `samples/복학원서.hwp` SVG 출력의 학교 로고 image href 가 `data:image/png` 로 시작 |

소스 변경 규모 예상: +50 / -2 (svg.rs 만), 테스트 +30~50.

### 3.2 변경 정책

**`detect_image_mime_type` 보강:**

```rust
// PCX: 0A 05 (ZSoft Paintbrush v3.0+)
if data.starts_with(&[0x0A, 0x05]) {
    return "image/x-pcx";
}
```

**PCX → PNG 변환 함수 추가:**

```rust
fn convert_pcx_to_png(pcx_data: &[u8]) -> Option<Vec<u8>> {
    use pcx::Reader;
    use image::{RgbaImage, ImageEncoder, codecs::png::PngEncoder};

    let mut reader = Reader::new(pcx_data).ok()?;
    let (width, height) = (reader.width() as u32, reader.height() as u32);
    let mut buffer = vec![0u8; (width * height * 4) as usize];
    // pcx 디코딩 → RGBA 변환
    // ...
    let img = RgbaImage::from_raw(width, height, buffer)?;
    let mut png = Vec::new();
    let encoder = PngEncoder::new(&mut png);
    encoder.write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgba8).ok()?;
    Some(png)
}
```

**Image emit 경로 정정:** SVG 의 `<image href="data:...">` emit 직전 분기:

```rust
let mime = detect_image_mime_type(&image_data);
let (final_mime, final_data) = if mime == "image/x-pcx" {
    if let Some(png) = convert_pcx_to_png(&image_data) {
        ("image/png", png)
    } else {
        // 변환 실패 시 원본 유지 (octet-stream 폴백 회피)
        (mime, image_data.to_vec())
    }
} else {
    (mime, image_data.to_vec())
};
// emit href="data:{final_mime};base64,{base64(final_data)}"
```

### 3.3 schemaVersion / 라운드트립 영향

- HWP 파일 raw BinData 는 변경하지 않음 (parse 시점에 PCX 보존)
- 변환은 **render 시점만** 적용 (HWP→HWP 라운드트립 보존, 메모리 `feedback_self_verification_not_hancom` 정합)
- SVG / PageLayerTree JSON / Canvas 출력 경로 모두 동일 변환 함수 호출 (가능하면 공통화)

## 4. 단계 분리 (3 stages)

### Stage 3 — PCX 디코더 통합 + 변환 함수 구현

**작업:**
1. `Cargo.toml` 에 `pcx = "0.2"` 추가
2. `cargo build --lib` 으로 dependency resolve 확인
3. `src/renderer/svg.rs` 의 `detect_image_mime_type` 에 PCX 분기 추가
4. `convert_pcx_to_png` 함수 작성 + 단위 테스트
5. SVG image emit 경로 (예: `paragraph_layout.rs` 또는 `svg.rs` 의 `<image>` emit 위치) 에서 PCX 변환 호출

**완료 기준:**
- `cargo build --lib` 성공
- `convert_pcx_to_png` 단위 테스트 통과 (`samples/복학원서.hwp` 의 BIN0001.PCX 데이터로 변환 → PNG 출력 검증)

**산출물**: `mydocs/working/task_m100_514_stage3.md`

### Stage 4 — 회귀 테스트 + 정합 점검

**작업:**
1. `tests/issue_514.rs` 신규: 복학원서.hwp SVG 출력의 학교 로고 image href 가 `data:image/png;base64,iVBORw0K` (PNG magic 의 base64 prefix) 로 시작함을 검증
2. 기존 통합 테스트 회귀 0 점검:
   - `cargo test --lib` (1110 passed 유지)
   - `cargo test --test issue_418` (셀 padding)
   - `cargo test --test issue_501` (mel-001)
   - `cargo test --test svg_snapshot` (6/6)
3. `cargo clippy --lib -- -D warnings` 통과

**완료 기준:**
- `tests/issue_514.rs` 1 passed
- 기존 회귀 0
- clippy 0건

**산출물**: `mydocs/working/task_m100_514_stage4.md`

### Stage 5 — 시각 판정 + WASM 빌드 + 최종 보고

**작업:**
1. `cargo build --release` + `rhwp export-svg samples/복학원서.hwp -o output/svg/task514_after/`
2. before/after SVG 비교 — 학교 로고 영역의 image href MIME 타입 변화 확인
3. **작업지시자 시각 판정** (한컴 2010 + 2022 + 복학원서.pdf 의 학교 로고 위치/크기 와 비교)
4. WASM 빌드 (Docker) → `rhwp-studio/public/rhwp.js` 갱신 (필요 시)
5. 최종 보고서 작성: `mydocs/report/task_m100_514_report.md`
6. `mydocs/orders/20260502.md` 갱신 (완료 상태)

**완료 기준:**
- 학교 로고가 한컴 정답지와 동일 위치/크기로 보임
- 작업지시자 시각 판정 승인
- WASM 갱신 완료 (필요 시)
- 최종 보고서 + orders 갱신 완료

**산출물**: `mydocs/report/task_m100_514_report.md` + orders 갱신

## 5. 위험 영역

| 위험 | 가능성 | 회피책 |
|------|--------|--------|
| `pcx` crate v0.2.5 의 디코딩 결함 (특정 PCX variant 미지원) | 🟨 중간 | 변환 실패 시 원본 데이터 emit (octet-stream 폴백 유지) — 결함 회피 후 별도 task |
| WASM 빌드 시 `pcx` crate 의 binary 크기 영향 | 🟢 작음 | pcx crate 는 lightweight (수 KB). WASM 빌드 결과 점검 |
| PNG 변환 결과의 시각이 한컴과 다름 (컬러팔레트 / 안티앨리어싱) | 🟨 중간 | Stage 5 작업지시자 시각 판정 게이트 |
| PCX 변환 함수가 다른 이미지 (PNG/JPEG) 에 부수 효과 | 🟢 매우 작음 | `mime == "image/x-pcx"` 가드로 PCX 한정 분기 |
| svg_snapshot 회귀 (PCX 변환이 다른 fixture 에 영향) | 🟢 매우 작음 | sweep 결과 PCX 는 복학원서.hwp 한정. svg_snapshot fixture 6개 (table_text/issue_157/issue_267_ktx/form_002/issue_147_aift/render_is_deterministic) 모두 PCX 미사용 |

## 6. 검증 게이트 (요약)

| 게이트 | Stage | 통과 기준 |
|--------|-------|----------|
| `cargo build --lib` | 3 | exit 0 |
| `convert_pcx_to_png` 단위 테스트 | 3 | BIN0001.PCX → 유효 PNG 변환 |
| `tests/issue_514.rs` | 4 | 1 passed (학교 로고 PNG MIME) |
| `cargo test --lib` | 4 | 1110 passed (회귀 0) |
| `cargo test --test svg_snapshot` | 4 | 6/6 (회귀 0) |
| `cargo clippy --lib -- -D warnings` | 4 | 0 건 |
| 작업지시자 시각 판정 | 5 | 한컴 정답지와 학교 로고 위치/크기 정합 ★ |
| WASM 빌드 | 5 | exit 0, binary 크기 회귀 합리적 |

## 7. 본 task 범위 외 (별도 task 후보)

- WMF / EMF 비표준 포맷의 SVG 출력 — 7 + 2 = 9 fixture 영향 (별도 issue 등록 권장)
- OLE 임베드 객체 (한셀, bitmap) — 3 fixture
- PCX 디코더 결함 회복 (변환 실패 시 placeholder image 등)

## 8. 메모리 정합

- `feedback_process_must_follow` — Stage 1 → Stage 2 (현재) → Stage 3~5 절차 준수
- `feedback_search_troubleshootings_first` — Stage 1 사전 검색 완료
- `feedback_hancom_compat_specific_over_general` — `mime == "image/x-pcx"` 가드로 case-specific 정정
- `reference_authoritative_hancom` / `feedback_pdf_not_authoritative` — Stage 5 한컴 2010 + 2022 직접 시각 판정
- `feedback_visual_regression_grows` — Stage 5 시각 판정 필수 게이트
- `feedback_commit_reports_in_branch` — task 브랜치 (`local/task514`) 에서 보고서 + 소스 함께 commit
- `feedback_self_verification_not_hancom` — HWP 파일 raw BinData 는 변경하지 않음 (라운드트립 보존)
- `feedback_close_issue_verify_merged` — close 전 정정 commit 의 devel 머지 검증

## 9. 승인 게이트

1. 본 구현 계획서 승인 → Stage 3 구현 시작
2. Stage 3 완료 보고서 승인 → Stage 4 회귀 테스트
3. Stage 4 완료 보고서 승인 → Stage 5 시각 판정 + WASM
4. Stage 5 시각 판정 승인 → 최종 보고서 + 머지 + 이슈 close

## 10. 다음 단계

작업지시자 본 구현 계획서 승인 후 Stage 3 (PCX 디코더 통합) 진행.
