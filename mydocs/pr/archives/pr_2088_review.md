# PR #2088 리뷰 — #2083 hide_fill 페이지 export-png 검은 페이지 수정

- 작성 시각: 2026-07-09 16:30 KST
- PR: https://github.com/edwardkim/rhwp/pull/2088
- 작성자: `planet6897`
- base / head: `devel` / `fix/2083-hide-fill-black-page`
- 문서 작성 시점 참고 head: `a5b343c483521a928fff7b2a588abb63fa7625f4`
- 문서 작성 시점 참고 merge state: `BEHIND`
- reviewer assign: `jangster77` 요청 완료
- 처리 경로: `codex/planet6897-prs-review-20260709` 에서 여러 PR 누적 체리픽 검토

## 변경 범위

- `samples/issue2083_hide_fill_page.hwpx` 추가.
- `src/renderer/layout.rs`: `hide_fill=true` 페이지에서도 raster flatten 시 검은 페이지가 되지 않도록 흰 page background 방출.
- `tests/issue_2083_hide_fill_page_background.rs`: native-skia PNG 렌더가 투명/검정이 아닌 흰 종이 바탕인지 검증.

## 체리픽 검토

- 누적 체리픽 순서: 6/11.
- 적용 커밋: `e77039018` (`Issue #2083: hide_fill 페이지 흰 종이 바탕 유지 ...`).
- 충돌: 없음.
- 선행 PR 의존: 없음.

## MCP 기준 PDF

PR에 기준 PDF가 없어 HWP 2020 MCP로 생성했다. 서버 URL/IP와 토큰은 공개 문서에 기록하지 않는다. MCP 접근 정보가 필요한 collaborator는 `@jangster77`에게 요청한다.

| 항목 | 내용 |
|---|---|
| 입력 HWPX | `samples/issue2083_hide_fill_page.hwpx` |
| 입력 SHA-256 | `7758c15c57b1ef14fda6e6d29409ae3425f344931f2901641af84a40ef413d2e` |
| 출력 PDF | `pdf/issue2083_hide_fill_page-2020.pdf` |
| 출력 SHA-256 | `9f8670591c23fd4d0b852026da03b49088f2bc1adce7519a69e37af19343ea5f` |
| MCP job id | `1e48e9d9-e030-4f35-91c8-2ddcf50e3690` |
| run_status / validation | `0` / `ok` |
| pdfinfo | 4 pages, PDF 1.7, Producer `cairo 1.18.0` |

## 로컬 검증

- GitHub Actions: 원 PR head 기준 `Build & Test`, `CodeQL`, `Canvas visual diff` 등 성공 확인.
- `git diff --check upstream/devel...HEAD`: 통합 브랜치 fixup 이후 통과.
- `cargo fmt --check`: 통과.
- cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 산출물 삭제 후 진행.
- `CARGO_INCREMENTAL=0 cargo test --features native-skia --test issue_1842 --test issue_2004_cell_image_stack_pagination --test issue_2083_hide_fill_page_background --test issue_2093_saved_single_line_spacing_after --test issue_2097_none_table_declared_fits`: 통과.
  - #2088 관련 `tests/issue_2083_hide_fill_page_background.rs`: 1 passed.

## 시각 검증

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --file-target pr2088_issue2083 samples/issue2083_hide_fill_page.hwpx pdf/issue2083_hide_fill_page-2020.pdf \
  --out output/planet6897_prs_visual_20260709
```

요약:

- rhwp SVG pages: 4
- MCP PDF pages: 4
- flagged pages: 0/4
- overlay average pixel match: 87.56323%
- overlay average visual accuracy proxy: 28.73233%
- 대표 asset: `mydocs/pr/assets/planet6897_prs_20260709/pr2088_issue2083_p004_review.png`
- metrics: `mydocs/pr/assets/planet6897_prs_20260709/pr2088_issue2083_overlay_metrics.json`

사람 판정: p4는 기준 PDF와 폰트/스케일 차이가 있으나, PR의 핵심인 검은 페이지/투명 페이지 회귀는 보이지 않는다.

## 판단

- 체리픽 가능 여부: 가능.
- blocking finding: 없음.
- PR 목적은 raster 검은 페이지 수정이며, native-skia test와 MCP PDF 기준 페이지 수가 모두 이를 뒷받침한다.
