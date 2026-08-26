---
kind: pr-review
status: review-complete-pending-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5786 검토 - 버그헌트 r4 조판, 셀 그림, 직렬화, Studio 배경

## 접수와 범위

| 항목 | 내용 |
| --- | --- |
| PR / 작성자 | [#5786](https://github.com/edwardkim/rhwp/pull/5786) / `planet6897` |
| source head / 적용 commit | `e4e8aa3a1dc7ed350e6e657ae8ff6655d8463ef9` / `19c381dee` |
| 관련 issue | [#5755](https://github.com/edwardkim/rhwp/issues/5755), [#5731](https://github.com/edwardkim/rhwp/issues/5731), [#5734](https://github.com/edwardkim/rhwp/issues/5734), [#5142](https://github.com/edwardkim/rhwp/issues/5142), [#5757](https://github.com/edwardkim/rhwp/issues/5757), [#5780](https://github.com/edwardkim/rhwp/issues/5780) |
| 기준 | `upstream/devel@fb434269eea237cc12053914560a2dbaf16270bf` |
| GitHub 상태 | Open, non-draft, `MERGEABLE`; source CI 성공 |
| 라우팅 | `maintainer_general` + `intake_and_review` + `local_validation` + `visual_fixture_evidence` + `multi_pr_update_branch` |

저장 `vpos` 되감김의 페이지 넘김, 셀 그림 T&B 흐름 배치, 다중 `secPr` 구역 분할, 선언 모순 축소,
Studio flow-image 페이지 배경을 한 배치로 고친다. #5781의 좁은 #5780 변경은 이 PR이 포괄하므로
중복 적용하지 않는다.

## 로컬 검증

- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`:
  **8,059 passed, slow 5, skipped 39, 301.295초**.
- `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`,
  standard `wasm-pack` web build가 통과했다.
- native-Skia library test와 `issue_2225_missing_picture_placeholder` 2건,
  `render_p37_direct_pdf_export` 4건이 통과했다.
- Studio `npm test`는 **1,058 passed, 1 skipped**, production build와 Page key/Home-End E2E가 통과했다.
  manifest check의 세 누락 항목은 `devel`에도 있는 기존 상태이며 이번 통합에 포함하지 않았다.

## 시각 증적

HWP 2020 MCP로 아래 입력을 PDF로 변환하고 Visual Sweep으로 p1을 확인했다. endpoint와 인증 정보는
기록하지 않는다.

| 항목 | 값 |
| --- | --- |
| 입력 | `samples/issue5780/flow_image_page_background.hwpx` |
| 입력 SHA-256 | `e8986e2eb4a07c5f3b8c66716bc1bd02e471323649e19f266b2ff1f6627c017e` |
| 기준 PDF | `pdf/pr_5786/hancom2020/issue5780_flow_image_page_background_hancom2020.pdf` |
| PDF SHA-256 / page | `507ccee52dfdf0e32a99a881d78caf495f256c86c62c3bd203fa60dccb6c00be` / 1 of 1 |
| 변환 판정 | MCP `success`, `run_status=0`, `validation=ok`, `PrintToPDFEx`, `PrintMethod=0` |
| Visual Sweep | p1 candidate 0, pixel match 98.5481%, visual accuracy proxy 98.50499% |
| 안정 asset | `mydocs/pr/assets/pr_5786_issue5780_flow_image_page_background_review.png` |

사람 검토에서도 rhwp와 기준 PDF 모두 남색 표지, 흰 제목/날짜, placeholder 위치가 유지됐다. 이는 해당
변경의 출력 근거이며 전체 문서 fidelity 100% 보장을 뜻하지 않는다.

## 최종 권고

**수용 권고.** 통합 code candidate `05fd35449`의 GitHub Build & Test, Lint, Native Skia, archive shard,
Render Diff, Proptest, Adapter inter-diff 및 CodeQL Rust/JavaScript/Python도 모두 통과했다. merge 뒤 issue와
원 PR comment에는 devel에 존재하는 위 asset의 raw URL을 사용한다.
