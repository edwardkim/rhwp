---
kind: pr-review
status: review-complete-pending-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5815 검토 - HWP5 한컴 기호의 평면-15 표시 키 정규화

| 항목 | 내용 |
| --- | --- |
| PR / 작성자 | [#5815](https://github.com/edwardkim/rhwp/pull/5815) / `kevin9327` |
| source head / 통합 적용 commit | `39ae6d633d26cce6812fcbf8675a39c0fcd060df` / `47251a1bc` |
| 관련 issue | [#5800](https://github.com/edwardkim/rhwp/issues/5800) |
| 기준 | `upstream/devel@7df17a0ca9b8070192a230878fc9f56313ecae83` |
| GitHub 상태 | Open, non-draft, `CLEAN`; 최신 source CI 성공 |
| 통합 후보 | `review/green-ci-20260821-r2` |

HWP5 BMP 원시 기호가 평면-15 한컴 표시표의 키로 정규화되지 않아 다른 Unicode glyph로 paint되던 문제를
고친다. 표시표에 실제 값이 있는 코드포인트만 변환하고, parser IR과 미등록 값은 원문을 보존한다.

## 검증과 시각 증적

- focused `issue_5800_hwp5_pua_plane15`: 3 passed. SVG는 `═`, `(인)`, `①②`, `━`를 paint하며 원시
  `U+A832` 등은 남지 않는다.
- 통합 후보 전체 Rust nextest 8,068 passed, 0 failed; native-Skia, WASM build, clippy 및 format도 통과했다.
- 한컴 2020 PDF 1/1쪽과 SVG 1/1쪽 sweep은 structural flag 0건이었다. PDF와 review PNG는 각각
  `pdf/pr_5815/hancom2020/issue5800-hancom-symbol-hancom2020.pdf`,
  `mydocs/pr/assets/pr_5815_issue5800_p001_review.png`에 보관했다.
- 한컴 PDF text layer는 private glyph를 추출하지 못해 text validation이 적용되지 않았다. 따라서 raster의
  글꼴 차이만으로 결함을 단정하지 않고, SVG의 display mapping 계약과 HWP5 원시 IR 보존을 함께 확인했다.

**통합 후보로 수용 권고.** 표시 경로만 정규화하고 IR·미등록 값은 보존하며, HWP5 fixture와 SVG 계약으로
회귀를 막는다.
