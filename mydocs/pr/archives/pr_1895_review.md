# PR #1895 Self Review — export-pdf 폰트 fallback 및 수식 폰트 옵션 추가

## 메타

| 항목 | 내용 |
|---|---|
| PR | #1895 |
| 제목 | `export-pdf 폰트 fallback 및 수식 폰트 옵션 추가` |
| 작성자 | `jangster77` |
| base | `devel` |
| head | `task/m100-1747-1884-pdf-font-options` |
| 관련 이슈 | #1747, #1884 |
| 문서 작성 시점 head | `8a16bae5c9fd201884ced891751eb11046f86fe2` |
| 처리 경로 | `mydocs/manual/pr_review_workflow.md` 옵션 1 |

## 변경 범위

- `export-pdf`에 PDF 전용 폰트 옵션을 추가했다.
  - `--font-path`
  - `--fallback-serif`
  - `--fallback-sans`
  - `--fallback-mono`
  - `--equation-font`
- OS별 기본 PDF fallback family를 적용한다.
- 사용자가 지정한 fallback/equation font가 fontdb에 없으면 PDF export 전에 warning을 출력한다.
- 기존 `svgs_to_pdf` / `svg_to_pdf` API는 default 옵션 경로로 유지하고, options 기반 API를 추가했다.
- 수식 SVG font-family 기본값을 상수로 노출해 PDF export 후처리에서 사용자 지정 우선 폰트를 적용한다.
- 공백 포함 경로/폰트명은 큰따옴표 사용을 권장하도록 CLI 도움말과 문서를 정리했다.
- macOS, Ubuntu, Windows SSH, Windows cmd, Windows PowerShell 경로의 smoke PDF를 검증 자료로 첨부했다.

## 렌더 영향 및 시각 검증 필요 여부

PDF export의 fontdb, generic fallback, 수식 font-family가 바뀌므로 렌더 영향 PR이다.

이번 PR은 한컴 기준 PDF와의 레이아웃 일치가 아니라, `export-pdf`가 OS/shell별 font option을 받아 정상 PDF를
생성하고 한글/수식 fallback 경로가 빈칸 없이 렌더되는지를 확인하는 범위다. 따라서 visual sweep diff가 아니라
생성 PDF의 대표 페이지 렌더 PNG와 `pdfinfo` 메타 검증을 시각 근거로 남긴다.

대표 시각 검증 asset:

- `mydocs/pr/assets/pr_1895_pdf_font_options_cross_platform_review.png`

검증 PDF:

- `mydocs/report/assets/task_m100_1747_1884/macos_export_pdf_p1.pdf`
- `mydocs/report/assets/task_m100_1747_1884/linux_ubuntu_export_pdf_p1.pdf`
- `mydocs/report/assets/task_m100_1747_1884/windows_ssh_default_export_pdf_p1.pdf`
- `mydocs/report/assets/task_m100_1747_1884/windows_cmd_export_pdf_p1.pdf`
- `mydocs/report/assets/task_m100_1747_1884/windows_powershell_export_pdf_p1.pdf`

`pdfinfo` 재확인 결과 다섯 파일 모두 `Producer: rhwp`, `Pages: 1`,
`Page size: 841.88 x 1190.52 pts (A3)`, `PDF version: 1.7`이다.

대표 PNG 육안 확인 결과 macOS, Ubuntu, Windows SSH, Windows cmd, Windows PowerShell 모두 p1 한글 본문과
선택지 영역이 빈칸 없이 렌더된다. 이 PR의 검증 범위에서 blocker는 보이지 않는다.

## 로컬 및 원격 환경 검증

PR 전 full CI 성격으로 다음 명령을 순차 실행했다.

| 항목 | 결과 |
|---|---|
| `env CARGO_INCREMENTAL=0 cargo build --release` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo test --release --lib` | 통과 (`2084 passed`, `6 ignored`) |
| `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | 통과 (`svg_snapshot` 포함) |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` | 통과 |
| `env CARGO_INCREMENTAL=0 cargo test --doc` | 통과 (`0 passed`, `1 ignored`) |
| `(cd rhwp-studio && npx tsc --noEmit)` | 통과 |
| `(cd rhwp-studio && npm test)` | 통과 (`153 passed`) |
| `wasm-pack build --target web --out-dir pkg` | 통과 |

추가 환경 검증:

- macOS: `export-pdf` 도움말, PDF 생성, `pdfinfo`, PNG 렌더 확인.
- Ubuntu 6.8.0: `ssh ubuntu-ted` 환경에서 `cargo test --lib renderer::pdf`, `cargo check --bin rhwp`,
  `export-pdf --help`, PDF 생성, `pdfinfo` 확인.
- Windows 10: `ssh win10-ted` 환경에서 별도 worktree를 만들어 검증.
  - 기본 SSH shell: 도움말과 PDF 생성 확인.
  - `cmd /C`: `.cmd` 스크립트에서 큰따옴표 포함 font family 인자 처리 확인.
  - `powershell -NoProfile`: `.ps1` 스크립트에서 큰따옴표 포함 font family 인자 처리 확인.

## 리뷰 결과

Blocking finding 없음.

`export-pdf` 사용자에게 필요한 PDF font fallback/equation font 설정 경로가 CLI 옵션, 문서, renderer API에
연결됐다. 기존 API는 default 경로로 유지되어 하위 호환성이 보존된다. 공백 포함 폰트명과 경로는 큰따옴표
권장 문구로 정리되어 macOS/Linux/Windows shell에서 설명이 일관된다.

최종 merge 조건:

- PR head 최신 커밋 기준 GitHub Actions 통과 또는 후속 문서/asset fast-pass 결과 확인.
- PR 본문 `Closes #1747`, `Closes #1884` 유지.
- merge 후 #1747/#1884 auto-close 여부를 확인하고, 자동 close 여부와 관계없이 후속 코멘트를 남긴다.
