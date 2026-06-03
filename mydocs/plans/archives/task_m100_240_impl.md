# 구현 계획서: Task #240 — BMP → PNG 재인코딩 (SVG 임베딩)

- 관련: `task_bug_240.md`
- 작성일: 2026-04-22

## Stage 1 — 의존성 추가 + BMP→PNG 변환 유틸 + 단위 테스트

- `Cargo.toml`에 의존성 추가
  - `image = { version = "0.25", default-features = false, features = ["bmp", "png"] }`
- 신규 함수: `src/renderer/svg.rs` 내부 `pub(crate) fn bmp_bytes_to_png_bytes(data: &[u8]) -> Option<Vec<u8>>`
  - `image::load_from_memory_with_format(data, ImageFormat::Bmp)` → `DynamicImage`
  - `img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)` → `Vec<u8>`
  - 모든 실패는 `None` 반환 (폴백 유지)
- 단위 테스트 (`src/renderer/svg.rs` 내 `#[cfg(test)] mod bmp_png_tests`)
  - 최소 유효 BMP(2×2 BI_RGB 32-bit)를 하드코딩 → 변환 → PNG 시그니처 확인
  - 잘못된 바이트 입력 시 `None` 반환 확인
- 산출물: `task_bug_240_stage1.md`

## Stage 2 — SVG 임베딩 경로에 변환 적용

- SVG 렌더러의 이미지 data URI 생성부 식별
  - `detect_image_mime_type` 호출 지점을 중심으로 BMP 판정 후 `bmp_bytes_to_png_bytes`를 호출하도록 수정
  - 성공 시: 데이터 = PNG 바이트, mime = `image/png`
  - 실패 시: 기존 BMP 그대로 유지 (회귀 방지 폴백)
- 변경 범위 최소화: SVG 경로만 수정, HWPX 직렬화/WASM API/web_canvas 미변경
- 회귀 테스트: `cargo test` 전체 통과 확인
- 산출물: `task_bug_240_stage2.md`

## Stage 3 — 샘플 재검증 + 최종 보고서 + 병합

- 검증 커맨드
  - `cargo run --bin rhwp -- export-svg samples/bitmap.hwp`
  - `cargo run --bin rhwp -- export-svg samples/한셀OLE.hwp`
- 자동 검증 지표
  - 생성된 SVG에 `data:image/bmp` 문자열이 **0건**
  - `data:image/png` 문자열 **1건 이상** 존재
  - 파일 크기 비교 (이전/이후)
- 산출물
  - `task_bug_240_stage3.md` (최종 단계 보고서)
  - `task_bug_240_report.md` (최종 결과 보고서)
  - `mydocs/orders/20260422.md` 상태 갱신 (없으면 생성)
- 브랜치 병합 (승인 후)
  - `local/task240` → `local/devel` (`--no-ff`)
  - 필요 시 `local/devel` → `devel` + `git push origin devel`
