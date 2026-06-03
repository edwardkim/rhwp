# Task #514 최종 보고서 — 복학원서.hwp PCX 학교 로고 → PNG 변환

## 결함 요약

| 항목 | 내용 |
|------|------|
| **문서** | `samples/복학원서.hwp` (고려대학교 복학신청서 양식) |
| **결함** | 양식 좌상단의 고려대학교 학교 로고가 SVG / Canvas 출력에 안 보임 |
| **본질** | 학교 로고가 **PCX 포맷** (`BIN0001.PCX`, ZSoft Paintbrush v3.0+, 1980년대 비표준 포맷) 으로 임베드됨. rhwp 의 `detect_image_mime_type` 가 PCX 시그니처 (`0x0A 0x05`) 를 인식하지 못해 `application/octet-stream` 으로 emit → 브라우저가 이미지로 렌더링 불가 |
| **시각 영향** | 양식 발급 기관 (고려대학교) 식별 정보 누락 → 양식의 시각적 신뢰성 / 정합성 손상 |
| **이슈** | [#514](https://github.com/edwardkim/rhwp/issues/514) (M100 v1.0.0, bug, edwardkim assign) |
| **commit** | (Stage 6 에서 결정) |

## 정정 본질

PCX 포맷을 인식하고 PNG 로 변환하여 emit. **흰색 픽셀을 투명 알파로 매핑한 RGBA PNG** 로 변환 (작업지시자 요구 — HWP 의 BehindText 배경/로고 호환).

CLI SVG 와 Web Canvas 가 각각 별도의 `detect_image_mime_type` 함수를 사용하므로 **두 경로 모두 정정**.

## 변경 영역

| 파일 | 변경 | 비고 |
|------|------|------|
| `Cargo.toml` | `pcx = "0.2"` dependency 추가 | lightweight PCX 디코더 |
| `src/renderer/svg.rs` | +75 / -2 | `detect_image_mime_type` PCX 분기 + `pcx_bytes_to_png_bytes` 변환 함수 + 두 emit 사이트 (paragraph_layout / draw_image) PCX → PNG 변환 |
| `src/renderer/web_canvas.rs` | +10 / 0 | Web Canvas 의 별도 `detect_image_mime_type` 와 `draw_image` 에 PCX 분기 추가 (svg::pcx_bytes_to_png_bytes 재사용) |
| `tests/issue_514.rs` (신규) | +73 | 회귀 테스트 3 건 |

총 변경: 4 파일, ~+160 / -2.

## 정정 코드 요지

### `detect_image_mime_type` 보강 (svg.rs + web_canvas.rs 양쪽)

```rust
// PCX: 0A 05 (ZSoft Paintbrush v3.0+, 1980년대 비표준 포맷)
if data.starts_with(&[0x0A, 0x05]) {
    return "image/x-pcx";
}
```

### `pcx_bytes_to_png_bytes` 변환 함수 (svg.rs)

```rust
pub(crate) fn pcx_bytes_to_png_bytes(data: &[u8]) -> Option<Vec<u8>> {
    let mut reader = pcx::Reader::new(Cursor::new(data)).ok()?;
    let (width, height) = (reader.width() as u32, reader.height() as u32);
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    if reader.is_paletted() {
        // paletted (8bpp): 인덱스 + 팔레트 → RGBA, 흰색 → 투명
    } else {
        // RGB (24bpp): row 별 RGB → RGBA, 흰색 → 투명
    }

    let img = RgbaImage::from_raw(width, height, rgba)?;
    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png).ok()?;
    Some(out)
}
```

### Image emit 사이트 PCX 분기 (svg.rs 2 곳 + web_canvas.rs 1 곳)

```rust
let (render_data, render_mime) = if mime_type == "image/x-pcx" {
    match pcx_bytes_to_png_bytes(data) {
        Some(png) => (Cow::Owned(png), "image/png"),
        None => (Cow::Borrowed(data), mime_type),  // 폴백
    }
} else { /* 기존 */ };
```

## 검증 결과

### 결정적 검증 (cargo test)

| 게이트 | 결과 |
|--------|------|
| `cargo build --lib` | ✅ Finished |
| `cargo build --release` | ✅ Finished |
| `cargo test --lib` | ✅ **1110 passed** (회귀 0) |
| `cargo test --test issue_514` (신규) | ✅ **3 passed** |
| `cargo test --test issue_418` (셀 padding) | ✅ 1 passed |
| `cargo test --test issue_501` (mel-001) | ✅ 1 passed |
| `cargo test --test svg_snapshot` | ✅ **6/6 passed** |
| `paint::json::tests` (PR #510 호환) | ✅ 4/4 passed |
| `cargo clippy --lib -- -D warnings` | ✅ 0 건 |
| `cargo clippy --test issue_514 -- -D warnings` | ✅ 0 건 |

### 시각 판정 (작업지시자)

| 경로 | 결과 |
|------|------|
| **CLI SVG** | ✅ 학교 로고 image href: `data:application/octet-stream` → **`data:image/png;base64,iVBORw0KGgo...`** (PNG 878×1001 px, RGBA color type 6, 흰색 → 투명 처리) |
| **Web Canvas** | ✅ 작업지시자 검증: "이제 PCX 이미지가 웹 캔바스에도 출력됩니다." |
| 워터마크 직인 (JPEG) | ✅ 회귀 0 (변경 없이 유지) |

### WASM 빌드

| 산출물 | 이전 | 현재 | 변동 |
|--------|------|------|------|
| `pkg/rhwp_bg.wasm` | 4,376,286 | **4,452,204** | +75,918 bytes (pcx crate + image RGBA + web_canvas 분기) |
| `pkg/rhwp.js` | 230,417 | 230,417 | 변동 없음 |
| `rhwp-studio/public/rhwp_bg.wasm` | stale | ✅ 동기화 | — |

## samples sweep 결과 (구현 계획서 §2.1)

비표준 이미지 포맷 분포 (samples/ 173 HWP 파일):

| 포맷 | 파일 수 | 비고 |
|------|---------|------|
| **PCX** | 1 | 본 task 정정 — 복학원서.hwp |
| WMF | 7 | pic-in-table-01, hwpspec — 본 task 범위 외 |
| EMF | 2 | exam_social — 본 task 범위 외 |
| OLE | 3 | 한셀OLE, bitmap — 본 task 범위 외 |

본 task 는 PCX 한정. WMF/EMF/OLE 는 별도 task 후보.

## 본 task 에서 발견한 구조적 사실 (메모리 후보)

**rhwp 의 image MIME 감지 / 변환 코드는 renderer 마다 별도 사본을 사용.**

- CLI SVG: `src/renderer/svg.rs::detect_image_mime_type` (pub(crate))
- Web Canvas: `src/renderer/web_canvas.rs::detect_image_mime_type` (`#[cfg(target_arch = "wasm32")]` 가드된 별도 fn)
- PageLayerTree JSON: `src/paint/json.rs::image_effect_str` (effect 문자열만)
- Web SVG / HTML 등: 별도 분기 가능성

**시사점**: 시각 결함 정정 시 한 곳만 수정하면 다른 경로에서 누락. Stage 3 의 svg.rs 정정 후 Web Canvas 가 그대로였던 본 task 가 명확한 사례. 이슈 #516 (web 그림 회색조 누락) 도 동일 패턴.

→ 메모리 `feedback_image_renderer_paths_separate` 후보 (별도 작업으로 등록 권장).

## 위험 점검 결과

| 위험 (구현 계획서 §5) | 결과 |
|----------------------|------|
| pcx crate 디코딩 결함 | 🟢 본 fixture 정상 변환, 폴백 분기로 panic 회피 |
| WASM binary 크기 영향 | 🟢 +75,918 bytes 합리적 (pcx + RGBA encoder) |
| PNG 변환 결과 시각 한컴 정합 | ✅ 작업지시자 시각 판정 통과 |
| PCX 분기가 다른 이미지에 부수 효과 | 🟢 mime == "image/x-pcx" 가드로 차단, JPEG/PNG 회귀 0 |
| svg_snapshot 회귀 | 🟢 6/6 통과 (PCX 미사용 fixture) |

## 메모리 정합

- `feedback_process_must_follow` — 이슈 → 브랜치 → 할일 → 계획서 → 단계별 → 보고 절차 준수
- `feedback_assign_issue_before_work` — 이슈 #514 메인테이너 assignee 지정
- `feedback_search_troubleshootings_first` — Stage 1 사전 검색 완료
- `feedback_hancom_compat_specific_over_general` — `mime == "image/x-pcx"` case-specific 가드
- `reference_authoritative_hancom` / `feedback_pdf_not_authoritative` — 한컴 정답지 직접 시각 판정
- `feedback_visual_regression_grows` — Stage 5 의 Canvas 시각 검증이 결정적이었음 (CLI SVG 단독 검증으로는 web_canvas 누락 검출 불가)
- `feedback_commit_reports_in_branch` — 본 보고서 task 브랜치에서 commit
- `feedback_self_verification_not_hancom` — HWP raw BinData 변경 없음 (라운드트립 보존)
- `feedback_close_issue_verify_merged` — close 전 commit 의 devel 머지 검증

## 후속 task 후보

1. **이슈 #515** — s0:pi=16 표 x 좌표 + 그림 조판 한컴 불일치 (이미 등록)
2. **이슈 #516** — rhwp-studio web 그림 회색조/밝기/대비 적용 누락 (이미 등록)
3. **새 이슈 후보** — WMF / EMF / OLE 비표준 포맷 처리 (samples/ 12 fixture 영향)
4. **새 메모리 후보** — `feedback_image_renderer_paths_separate` (renderer 별 별도 함수 사실)
5. **새 task 후보** — `cargo clippy --all-targets` 의 사전 결함 44 건 (`unused Result` in src/wasm_api/tests.rs) 정리

## Stage 진행 요약

| Stage | 내용 | 상태 |
|-------|------|------|
| 1 | 본질 진단 (PCX 포맷 인식 누락 확정) | ✅ 완료 |
| 2 | 구현 계획서 (PCX 디코더 통합 정책) | ✅ 완료 |
| 3 | PCX 디코더 통합 + svg.rs 변환 함수 | ✅ 완료 |
| 4 | 회귀 테스트 + 통합 회귀 0 | ✅ 완료 |
| 5 | 시각 판정 + WASM 빌드 + **web_canvas.rs 추가 정정** | ✅ 완료 |
| 6 (현재) | 최종 보고 + merge + push + 이슈 close | ✅ 본 보고서 |
