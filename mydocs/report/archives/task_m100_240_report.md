# 최종 보고서 — Task #240

- 이슈: edwardkim/rhwp#240 — BMP 임베딩 HWP 문서가 SVG에서 렌더링되지 않음
- 유형: 버그 (마일스톤 없음)
- 브랜치: `local/task240`
- 기간: 2026-04-22 (당일)

## 요약

`bitmap.hwp`, `한셀OLE.hwp`와 같이 BMP 이미지(또는 OLE 객체의 BMP 미리보기)를 포함한 HWP 문서를 `export-svg`로 변환하면, SVG 내부에 `data:image/bmp` URI로 원본 BMP가 그대로 임베딩되어 브라우저에서 렌더되지 않는 문제가 있었다.

SVG 내보내기 경로에서 BMP로 감지된 이미지 데이터를 PNG로 재인코딩하여 `data:image/png`로 임베딩하도록 수정하였다. 실패 시 원본 BMP를 유지(폴백)하여 기능 회귀 가능성을 차단하였다.

## 변경 파일

- `Cargo.toml` — `image = "0.25"` 의존성 추가 (features: `bmp`, `png` 최소)
- `src/renderer/svg.rs`
  - `bmp_bytes_to_png_bytes` 신규 함수 추가
  - 배경 이미지 임베딩(L161~) BMP→PNG 적용
  - `render_image_node`(L1035~) BMP→PNG 적용
  - tests.rs에 단위 테스트 3건 추가
- `src/renderer/layout/shape_layout.rs` — OLE `native_image` 폴백 경로에 BMP→PNG 적용
- `samples/bitmap.hwp`, `samples/한셀OLE.hwp` — 저장소 루트 → `samples/` 이동
- 문서
  - `mydocs/plans/task_bug_240.md` (수행 계획서)
  - `mydocs/plans/task_bug_240_impl.md` (구현 계획서)
  - `mydocs/working/task_bug_240_stage{1,2,3}.md` (단계별)
  - `mydocs/working/task_bug_240_report.md` (본 문서)
  - `mydocs/orders/20260422_issue_bmp_svg_render.md` (이슈 원안)

## 검증

| 항목 | 결과 |
|------|------|
| 단위 테스트 추가 | 3건 (bmp→png 성공/실패/무효 입력) |
| 전체 회귀 테스트 | 941 pass / 1 ignored / 0 fail |
| `samples/bitmap.hwp` | `data:image/bmp` 0, `data:image/png` 1 |
| `samples/한셀OLE.hwp` | `data:image/bmp` 0, `data:image/png` 1 |
| 파일 크기 | bitmap.svg 7.6MB → 45KB, 한셀OLE.svg 174KB → 3.7KB |

## 영향 범위

- SVG 내보내기 경로에 한함 (HWPX 직렬화, WASM 캔버스 등 타 경로 무영향)
- WMF/TIFF 등 다른 포맷은 본 작업 범위 외 (기존 동작 유지)
- EMF 변환기가 내부적으로 생성하는 DIB→BMP data URL은 별도 컨텍스트(외부 `<img>` 가능성)라 미변경

## 후속 과제 (제안)
- TIFF가 포함된 샘플이 있다면 동일 접근으로 PNG 변환 검토
- `image` crate 추가로 바이너리/WASM 사이즈 영향 측정 (features 최소화 상태)
