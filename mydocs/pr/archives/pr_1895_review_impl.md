# PR #1895 Review Impl — export-pdf 폰트 옵션

## Stage 1. 변경 내용 확인

완료.

- `src/renderer/pdf.rs`에 `PdfExportOptions`와 options 기반 PDF 변환 API가 추가됐다.
- `src/document_core/queries/rendering.rs`는 기존 PDF export API를 유지하면서 options 기반 API를 제공한다.
- `src/main.rs`의 `export-pdf` CLI가 fallback/equation/font-path 옵션을 파싱한다.
- `src/renderer/equation/svg_render.rs`는 수식 SVG font-family 기본값을 상수로 노출한다.
- `mydocs/manual/cli_commands.md`, `mydocs/manual/rhwp_cli_skill_guide.md`에 큰따옴표 권장 사용법이 반영됐다.

## Stage 2. 검증

완료.

- macOS/Linux/Windows 3경로 PDF smoke export를 확인했다.
- PR 전 full CI를 순차 실행했고 모두 통과했다.
- `cargo test --profile release-test --tests` 안에서 `tests/svg_snapshot.rs`도 함께 통과했다.
- WASM 빌드 후 tracked diff가 생기지 않는 것을 확인했다.

## Stage 3. 옵션 1 운영 기록 보강

완료.

- review 문서:
  - `mydocs/pr/archives/pr_1895_review.md`
  - `mydocs/pr/archives/pr_1895_review_impl.md`
- 대표 시각 검증 asset:
  - `mydocs/pr/assets/pr_1895_pdf_font_options_cross_platform_review.png`
- 오늘할일:
  - `mydocs/orders/20260704.md`

## Stage 4. merge 전 확인 항목

- GitHub Actions 최신 head 통과 또는 후속 문서/asset fast-pass 결과 확인.
- mergeable 상태 재확인.
- merge 후 #1747/#1884 close 상태 확인 및 후속 코멘트 작성.
- merge 후 `devel` sync 및 `task/m100-1747-1884-pdf-font-options` 로컬/원격 브랜치 정리.
