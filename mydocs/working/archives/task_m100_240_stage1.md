# Stage 1 완료 보고서 — Task #240

- 작업: BMP→PNG 변환 유틸 추가 + 단위 테스트
- 일자: 2026-04-22

## 변경 내역

### 의존성
- `Cargo.toml`: `image = { version = "0.25", default-features = false, features = ["bmp", "png"] }` 추가

### 신규 함수
- `src/renderer/svg.rs`: `pub(crate) fn bmp_bytes_to_png_bytes(data: &[u8]) -> Option<Vec<u8>>`
  - `image::load_from_memory_with_format(data, ImageFormat::Bmp)` 로 디코드
  - `DynamicImage::write_to` + `ImageFormat::Png` 로 인코드
  - 디코드/인코드 실패 시 `None` (폴백)

### 단위 테스트
- `src/renderer/svg/tests.rs`: `make_minimal_bmp_2x2`, `test_bmp_to_png_success`, `test_bmp_to_png_invalid_returns_none`
- 2×2 BI_RGB 32-bit BMP → PNG 시그니처(`89 50 4E 47`) 검증
- 잘못된 입력 시 `None` 반환 확인

## 테스트 결과

```
cargo test --lib bmp
running 3 tests ... ok. 3 passed; 0 failed
```

## 다음 단계
- Stage 2: SVG 렌더러 임베딩 경로(3개 지점)에 변환 적용
