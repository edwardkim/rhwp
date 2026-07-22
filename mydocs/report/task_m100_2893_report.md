# task m100-2893: cli_commands.md `--backend direct` 서술 오류 수정

## 이슈

edwardkim/rhwp#2893

## 문제

`mydocs/manual/cli_commands.md` 122번째 줄이 `export-pdf`의 direct/vector
`PageLayerTree → PDF` backend(`--backend direct`)를 "아직 후속 작업"이라고 서술하고
있었으나, `src/main.rs`와 기존 통합 테스트를 확인한 결과 이 backend는 이미 CLI
옵션으로 완전히 배선되어 있고 `native-skia` feature 빌드에서 동작이 검증되어 있었다.
또한 `--backend`, `--raster-dpi` 옵션 자체가 `export-pdf` 옵션 목록에 전혀 나열되어
있지 않은 문제도 함께 있었다.

## 근거

- `src/main.rs:183,187` — `--help` 출력에 `--backend <svg|direct>`, `--raster-dpi <DPI>`
  가 이미 존재.
- `src/main.rs:1347-1368` — `--backend`/`--backend=` 파싱 로직 존재.
- `src/main.rs:1585-1610` — `PdfBackend::DirectLayer` 분기가
  `render_pages_pdf_direct_native_with_profile_and_options`로 실제 렌더링 연결(`native-skia`
  feature 조건부).
- `src/main.rs:1630-1632` — direct backend 성공 시 `"PDF backend: direct"` 완료 로그.
- `tests/render_p37_direct_pdf_export.rs` — `#[cfg(... feature = "native-skia")]` 하에서
  `render_page_pdf_direct_native` 등 3개 API 전체를 호출해 완결된 PDF(`%PDF-` ~
  `%%EOF`)를 검증.
- `tests/render_p37_pdf_backend_cli.rs` — CLI 레벨에서 `--backend svg` 기본 동작 동일성과,
  `native-skia` 없는 빌드에서만 `--backend direct`가 종료코드 1 + 명시적 오류 메시지를
  반환함을 검증(`direct_backend_reports_missing_native_skia_feature`).

## 변경 사항

`mydocs/manual/cli_commands.md`
- `export-pdf` 옵션 목록에 `--backend <svg|direct>`, `--raster-dpi <DPI>` 항목을 추가하고,
  `direct` backend가 `native-skia` feature 빌드를 요구하며 없을 경우 명시적 오류를
  반환한다는 점을 명시.
- 122번째 줄의 "아직 후속 작업이다" 서술을 "`--backend direct`로 이미 사용 가능하다
  (`native-skia` feature 빌드 필요)"로 정정.

## 검증

이번 변경은 마크다운 문서 수정만 포함하는 docs-only 변경이므로 코드 검증은
코드/테스트 인용을 통한 cross-check로 대체했다(별도 cargo build/test 불필요).
- `grep -n "backend" src/main.rs` 및 `tests/render_p37_*` 파일 내용을 직접 읽어 문서
  서술과 실제 구현/테스트가 일치하는지 확인함.
- `grep -n "backend" mydocs/manual/cli_commands.md`로 수정 후 문서 내 유일한 backend
  언급이 정정된 내용과 일치함을 확인함.
