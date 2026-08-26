---
kind: pr-review
status: review-complete-pending-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5813 검토 - 저장 사다리의 spacing-before 회계

| 항목 | 내용 |
| --- | --- |
| PR / 작성자 | [#5813](https://github.com/edwardkim/rhwp/pull/5813) / `planet6897` |
| source head / 통합 적용 commit | `090ad9509617792f97e58f04708282d0bb22fe98` / `a58346f50` |
| 관련 issue | [#5801](https://github.com/edwardkim/rhwp/issues/5801) |
| 기준 | `upstream/devel@7df17a0ca9b8070192a230878fc9f56313ecae83` |
| GitHub 상태 | Open, non-draft, `CLEAN`; 최신 source CI 성공 |
| 통합 후보 | `review/green-ci-20260821-r2` |

저장된 line ladder가 문단 위 간격을 담지 않은 경우에는 spacing-before trim을 적용하지 않는다. 해당 gate는
`samples/2025 행정업무운영 편람(최종).hwpx` 272쪽의 `usedHeight`가 저장 좌표와 1px 이내로 맞고,
문서 전체 쪽수 383을 바꾸지 않는 계약으로 고정했다.

## 검증과 시각 증적

- focused `issue_5801_stored_ladder_spacing_before`: 2 passed.
- 통합 후보 전체 Rust nextest 8,068 passed, 0 failed; native-Skia, WASM build, native/WASM clippy와
  `cargo fmt --check`도 통과했다.
- 한컴 2020 MCP 비동기 job `fb592ace-b5e6-4ba8-919d-9437e66068dd`가 600초 후 성공했다. 동일 HWPX의
  editor/PDF 쪽수는 383/383이고 PDF SHA-256은
  `7929210788d61e1398e691139038e97afc4477b216f0fb8c7b455fd1b11c6711`이다.
- 272쪽 PDF/SVG sweep은 structural flag 0건이었다. 기준 PDF와 후보의 상세 비교 PNG는
  `mydocs/pr/assets/pr_5813_issue5801_p272_review.png`, 한컴 기준 PDF는
  `pdf/pr_5813/hancom2020/2025 행정업무운영 편람(최종)-2020.pdf`에 보관했다.
- PDF와 rhwp의 광범위한 272쪽 본문 fidelity 차이는 `upstream/devel@7df17a0`에서도 후보와 같은 SVG
  drawing으로 재현됐다. 두 SVG의 차이는 `@font-face` 선언 순서뿐이었다. 따라서 이번 회계 gate의
  회귀가 아닌 기존 fidelity 잔존이며, 별도 추적은 [#3820](https://github.com/edwardkim/rhwp/issues/3820)에서
  계속한다.

**통합 후보로 수용 권고.** 저장 사다리 회계의 구체 계약은 충족했고, 확인된 fidelity 잔존은 이 PR의
변경 전후에 동일하므로 현재 변경을 차단하지 않는다.
