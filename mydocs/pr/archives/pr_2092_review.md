# PR #2092 리뷰 — #1921 RowBreak 블록컷 sliver 흡수

- 작성 시각: 2026-07-09 16:30 KST
- PR: https://github.com/edwardkim/rhwp/pull/2092
- 작성자: `planet6897`
- base / head: `devel` / `fix/1921-rowbreak-sliver-absorb`
- 문서 작성 시점 참고 head: `c529eb2f46ef5f78576c34c3b9f3cf92d2e65401`
- 문서 작성 시점 참고 merge state: `BEHIND`
- reviewer assign: `jangster77` 요청 완료
- 처리 경로: `codex/planet6897-prs-review-20260709` 에서 여러 PR 누적 체리픽 검토

## 변경 범위

- `src/renderer/layout/table_layout.rs`: `advance_row_block_cut_with_row_offsets` 경로에서 저장 hard-break 직전 tail sliver를 48px 이내 흡수.
- `mydocs/plans/task_m100_1921_float_table.md`, `mydocs/report/task_m100_1921_sliver_report.md` 추가.
- 새 재현 HWP/HWPX/PDF는 PR diff에 포함되지 않았지만, 보고서가 언급한 무회귀 핀 `86712` 샘플과 공식 PDF는 기존 저장소에 존재한다.

## 체리픽 검토

- 누적 체리픽 순서: 8/11.
- 적용 커밋: `bef9d36c8` (`Issue #1921: RowBreak 블록컷 sliver 흡수 ...`).
- 충돌: 없음.
- 선행 PR 의존: #2090 RCA 문서 이후 적용했으나 파일 충돌 없음.

## 검증

- GitHub Actions: 원 PR head 기준 `Build & Test`, `CodeQL`, `Canvas visual diff` 등 성공 확인.
- `git diff --check upstream/devel...HEAD`: 통합 브랜치 fixup 이후 통과.
- `cargo fmt --check`: 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과(exit 0).
- `cargo test --profile release-test --test issue_1891`: 3 passed.
- `target/debug/rhwp dump-pages samples/86712_regulatory_analysis.hwp`: 65 pages.
- `target/debug/rhwp dump-pages samples/issue1891/86712_regulatory_analysis.hwpx`: 65 pages.
- `pdfinfo samples/issue1891/86712_regulatory_analysis-2024.pdf`: 65 pages, Creator `Hwp 2024 13.0.0.3622`, Producer `Hancom PDF 1.3.0.550`.
- PDF 보존 규칙에 따라 기준 PDF를 `pdf/issue1921/86712_regulatory_analysis-2024.pdf`로 복사했다.
- contributor 보고서상 검증값: `59043` 48 -> 42, `86712` 65 유지, A/B 2,500 변화 0.

## 기준 샘플

| 항목 | 내용 |
|---|---|
| HWP | `samples/86712_regulatory_analysis.hwp` |
| HWP SHA-256 | `32e2ed30e5d744ad747f04f090c022eca8270f9dd2d55e0613e2ad61058099e9` |
| HWPX | `samples/issue1891/86712_regulatory_analysis.hwpx` |
| HWPX SHA-256 | `9695028f767efcb7ba4588ca76f9a33038a7a742fc66ea1afff30d0bf6286e25` |
| 기준 PDF | `pdf/issue1921/86712_regulatory_analysis-2024.pdf` |
| 기준 PDF SHA-256 | `b612638c849d967cf3005d0a7c82df18f47ac66f3b2777632ffe0912135b4a81` |

## 시각 검증

`86712` 공식 PDF 핀을 직접 sweep했다.

- 실행: `python3 scripts/task1274_visual_sweep.py --file-target pr2092_issue1921_86712 samples/86712_regulatory_analysis.hwp pdf/issue1921/86712_regulatory_analysis-2024.pdf --out output/planet6897_prs_visual_20260709_pr2092`
- rhwp SVG pages: 65
- render tree pages: 65
- PDF pages: 65
- flagged pages: 9/65
- flags: line band drift p2/p34/p50, column line band drift p2/p27/p31/p34/p50, render tree tail overflow p2/p4/p7/p8/p27/p31/p33/p34/p50
- overlay average pixel match: 91.35759%
- overlay average visual accuracy proxy: 8.39303%
- 대표 asset: `mydocs/pr/assets/planet6897_prs_20260709/pr2092_issue1921_86712_p002_review.png`, `mydocs/pr/assets/planet6897_prs_20260709/pr2092_issue1921_86712_p050_review.png`
- metrics: `mydocs/pr/assets/planet6897_prs_20260709/pr2092_issue1921_86712_overlay_metrics.json`, `mydocs/pr/assets/planet6897_prs_20260709/pr2092_issue1921_86712_visual_metrics.json`

사람 판정: `86712`는 공식 PDF 기준 페이지 수 65쪽 핀을 유지한다. visual sweep상 기존 글줄/컬럼 위치 drift 후보는 남아 있으나, #1921 sliver 흡수 PR의 회귀 핀으로 쓰인 페이지 수 조건은 로컬에서 직접 재현됐다.

## 판단

- 체리픽 가능 여부: 기계적 체리픽은 가능.
- `86712` 무회귀 핀: 저장소 샘플과 공식 PDF로 로컬 독립 검증 완료.
- `59043` 개선 주장: 현재 저장소에 `59043` 원 실문서와 기준 PDF가 없어 직접 재현하지 못했다. 이 축은 `mydocs/report/task_m100_1921_sliver_report.md`의 계측/검증값을 근거로 본다.
- 통합 PR 처리: 머지 차단 사유로 제출하지 않는다. 다만 merge 후 원 PR에는 `59043` 타깃 원본 HWP/HWPX 와 기준 PDF를 후속으로 보강하면 검증 추적성이 더 좋아진다는 의견을 남긴다.

## 후속 검토 코멘트 초안

@planet6897 작업 감사합니다. 체리픽 통합과 로컬 검증은 통과했습니다.

사후 확인 의견으로 하나 남깁니다. 보고서의 무회귀 핀인 `86712`는 저장소의 HWP/HWPX 및 공식 PDF로 65쪽 유지를 직접 확인했습니다. 다만 핵심 개선 타깃인 `59043` 원본 HWP/HWPX 와 기준 PDF는 PR diff와 저장소에서 찾을 수 없어, 해당 48 -> 42 개선 축은 report의 계측값을 근거로만 확인했습니다.

다음에 페이지 수나 시각 검증이 필요한 PR을 올려주실 때는, 타깃 원본 HWP/HWPX 파일과 한컴 2020/2024 등에서 저장한 기준 PDF를 함께 첨부해 주세요. 기준 PDF만 없으면 maintainer 측에서 HWP 2020 MCP로 산출할 수 있지만, 원본 HWP/HWPX가 없으면 검증과 회귀 추적이 어렵습니다.
