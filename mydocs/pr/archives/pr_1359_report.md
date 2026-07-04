# PR #1359 최종 보고서 — native PDF export API (P23, seo-rii)

## 1. 결정

**merge 수용** — PR 작업 커밋 3개를 `local/devel` 에 cherry-pick → `devel` push 방식.

## 2. 변경 본질

PDF export 를 `DocumentCore` native API(`render_page_pdf_native`/`render_pages_pdf_native`/
`render_document_pdf_native`)로 정리(P23). CLI `export-pdf` 가 같은 native surface 사용 +
실패 경로 정리. report-only PDF visual diff CI 추가(canvas hard gate 유지). WASM public API
미포함, direct/vector backend 는 후속(non-goals).

10파일 +821/-106 (rendering.rs +34, main.rs cfg 분기, contract test +91, CI workflow, 문서).

## 3. 검증

- cherry-pick `5beb582e`/`0d8fe5e3`/`8ebd46d1` → local/devel: 충돌 없음 (author seorii 보존).
- `cargo fmt --check`: OK.
- `cargo test --test render_p23_pdf_export_contract`: 5/5 pass.
- `export-pdf -p 0`: `%PDF-1.7` 생성.
- `cargo clippy --bin rhwp --lib`: 0.
- GitHub CI: Build & Test / Canvas visual diff / CodeQL / Analyze(rust·js·python) 전부 pass.

## 4. merge 방식 — cherry-pick 선택

base=devel 이고 CI 전부 pass 였으나, 검증된 작업 커밋 3개(merge 커밋 제외)만 `local/devel`
에 cherry-pick(무관 변경 0, author 보존)하여 push. PR 은 "devel 에 포함됨" 으로 close.

## 5. 후속

- `Refs #536`(멀티 렌더러 트래킹 이슈)은 close 대상 아님 — 유지.
- direct/vector `PageLayerTree -> PDF` replay backend 는 별도 후속 PR.
- PDF visual diff 는 report-only 단계 — 향후 수치·artifact 축적 후 gate 승격 검토.
- merge 후 리뷰/보고서 archives 이동, 오늘할일 반영.
