# Task #3302 Stage 2 — 구현계획서

Stage 1 승인(2026-07-25) 기반. 수정 지점은 `src/renderer/skia/image_conv.rs` 1곳으로 확정
(236행 placeholder 는 디코드 성공 후 크기 가드라 무관).

## 변경 명세

### 1. 폴백 헬퍼 추가 (같은 파일)

```rust
/// [#3302] skia 가 디코드하지 못하는 인코딩(실측: HWP3 1bpp BMP 스캔)을 image crate 로
/// 디코드해 PNG 로 재인코딩한다. SVG 백엔드의 기존 해법(bmp_svg_render 트러블슈팅)과 대칭.
fn reencode_unsupported_image_to_png(bytes: &[u8]) -> Option<Vec<u8>>
```

- `image::load_from_memory(bytes)` → `write_to(Cursor, ImageFormat::Png)`.
- 실패 시 None (기존 placeholder 경로 유지).

### 2. 디코드 지점 재구성 (`Image::from_encoded` 실패 분기, 현 164행 인근)

```rust
let image = match Image::from_encoded(Data::new_copy(encoded_bytes)) {
    Some(image) => image,
    None => {
        // [#3302] skia 미지원 인코딩 → image crate 재인코딩 후 1회 재시도.
        match reencode_unsupported_image_to_png(encoded_bytes)
            .and_then(|png| Image::from_encoded(Data::new_copy(&png)))
        {
            Some(image) => image,
            None => {
                draw_missing_image_placeholder(x, y, width, height);
                return false;
            }
        }
    }
};
```

- 정상 경로 비용 불변(폴백은 skia 실패 시에만). 포맷·문서 특정 분기 없음.

### 3. 단위 테스트 (feature = native-skia)

- 픽스처: 테스트 내 합성 **1bpp BMP** 바이트(2×2, BITMAPFILEHEADER+INFOHEADER+팔레트 2색)
  — 신규 샘플 파일 불필요.
- 검증 1: 합성 바이트가 `Image::from_encoded` 로는 None(전제 재현)임을 확인.
  (skia 버전에 따라 성공한다면 전제가 사라진 것이므로 테스트가 이를 드러냄)
- 검증 2: `reencode_unsupported_image_to_png` → Some, 그 PNG 가 `Image::from_encoded`
  로 디코드 성공.

## 검증 사다리 (stage1 계획 그대로)

1. `cargo test --features native-skia` 신규 테스트 + Native Skia 3종.
2. 실측: `export-png SO-SUEOP.hwp -p 0` — 이미지 영역 placeholder(≈225 회색) 소멸,
   실 콘텐츠 렌더. SVG 경로 복원본(`p0_svg_image.png`)과 영역 상관 비교.
3. 전체 release-test + 42·43쪽 diff 0 유지 재확인.
4. 1쪽 before/after + SVG 대조 이미지 → 작업지시자 시각 판정.

## 커밋 구성

1. `fix(render/skia): skia 미지원 이미지 image-crate PNG 폴백 — 1bpp BMP placeholder 정정 (#3302)`
   — 헬퍼 + 재구성 + 단위 테스트.
2. 보고서 `task_m100_3302_report.md` (시각 판정 후).
