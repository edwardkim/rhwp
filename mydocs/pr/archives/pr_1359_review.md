# PR #1359 검토 — native PDF export API 노출 (P23, #536)

## 1. PR 개요

- PR: https://github.com/edwardkim/rhwp/pull/1359
- 작성자: `seo-rii` (Seohyun Lee) — 렌더링 백엔드(Canvas/Skia/PageLayerTree) 핵심 컨트리뷰터,
  P시리즈 20+ PR. 본 PR 은 그 연속선(P23).
- 상태: open / 라벨 `enhancement`
- base: `devel` ✓ (head: `render-p23`, 작업 커밋 3 + devel 동기화 merge 2)
- 연결 이슈: `Refs #536` (멀티 렌더러 지원 트래킹 이슈, OPEN — closes 아님, 단계 기여)
- 변경: 10파일 +821/-106 (Rust 코어 + CLI + contract test + CI workflow + 문서)

## 2. 변경 요약

PDF export 를 `DocumentCore` native API 표면으로 정리(P23). direct/vector backend 를 여는
PR 이 아니라, CLI 와 native caller 가 같은 PDF export surface 를 쓰도록 안정화하는 단계.

- `rendering.rs` (+34): `render_page_pdf_native(page)` / `render_pages_pdf_native(&[page])`
  (0-based 명시) / `render_document_pdf_native()` 3종. 전부 `#[cfg(not(wasm32))]`,
  기존 `render_page_svg_native` + `renderer::pdf::svgs_to_pdf` 재사용(호환 경로 유지).
  빈 페이지 선택 에러 처리.
- `main.rs` (+96/-96, 대부분 cfg 분기 인덴트): `export-pdf` 가 SVG 직접 수집 대신 native
  API 호출. 디렉터리 생성/변환/저장 실패 시 early return(실패 후 완료 메시지 안 찍도록 정리).
  WASM 빌드에선 "native 전용" 안내.
- `tests/render_p23_pdf_export_contract.rs` (+91): contract test 5건.
- CI `render-diff.yml`: poppler-utils 설치(`continue-on-error`) + report-only PDF visual diff
  (`RHWP_RENDER_DIFF_PDF=1`, 실패해도 CI gating 안 함). Canvas diff 는 hard gate 유지.
- `pdf-render-diff-report.mjs` (+555): export-pdf → pdftoppm rasterize → Canvas 출력 대비
  report. README/README_EN/cli_commands.md 문서 갱신.

## 3. 검증 (로컬, `pr1359-review` = local/devel + cherry-pick 3커밋)

- cherry-pick(5beb582e/0d8fe5e3/8ebd46d1): 충돌 없음 (author `seorii` 보존).
- `cargo fmt --check`: **FMT_OK**.
- `cargo test --test render_p23_pdf_export_contract`: **5/5 pass**
  (single/explicit/full + 빈 선택 거부 + 범위 밖(99) 에러 전파).
- `export-pdf samples/re-03-latin-only-hancom.hwp -p 0`: smoke.pdf 생성, `%PDF-1.7` 헤더.
- `cargo clippy --bin rhwp --lib`: **0 warnings/errors**.
- GitHub CI: Build & Test / Canvas visual diff / CodeQL / Analyze(rust·js·python) **전부 pass**.
  (WASM check skipping — native API 가 `#[cfg(not(wasm32))]` 라 wasm 경로 미변경.)

## 4. 평가

### 장점

- 설계 일관 — PDF 를 native/export surface 로 분리, WASM public API 미포함(non-goals 준수),
  PDF visual diff 는 report-only(canvas hard gate 유지). 호환 경로(SVG-derived) 유지로 회귀 위험 낮음.
- contract test 5건이 정상·에러 경로 모두 봉인. CLI 실패 경로 정리(early return)도 적절.
- 핵심 컨트리뷰터의 단계적 작업(P시리즈), 리뷰 피드백 반영 커밋 포함. CI 전부 녹색.
- 무관 변경 0, 명명·문서 규칙 정합.

### 검토 포인트 (블로커 아님)

- BEHIND(오래된 base 아님 — PR 에 devel 동기화 merge 포함, merge 깨끗). admin 또는 UI merge.
- `Refs #536` 트래킹 이슈 — close 대상 아님(단계 기여), merge 후 이슈 유지.
- PDF 출력은 한컴 시각 정답지 아님(PR 명시) — report-only diff 는 수치·artifact 수집 단계.

## 5. 판단

**merge 권고**. 코어 API 추가가 안전(cfg 격리·호환 경로·contract test), CLI 정리 적절,
CI report-only 격리 양호, 로컬·GitHub CI 전부 통과. 세부는 `pr_1359_report.md`.
