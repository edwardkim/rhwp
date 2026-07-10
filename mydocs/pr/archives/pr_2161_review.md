# PR #2161 검토 - planet6897 열린 PR 8건 체리픽 및 검토 기록

- PR: https://github.com/edwardkim/rhwp/pull/2161
- 작성자: `jangster77`
- base: `devel`
- head: `codex/planet6897-cherrypick-review-20260710`
- 작성일: 2026-07-10
- 문서 작성 시점 참고값: `mergeable=MERGEABLE`, `mergeStateStatus=BLOCKED`

## 결론

옵션 1 경로로 처리한다. 즉 원 PR 8건의 체리픽, maintainer 보정, PR별 review 문서,
MCP 기준 PDF 증적, #2087 회귀 테스트를 모두 PR #2161 안에 포함한다.

로컬 검증 기준 blocking finding 없음. merge 전에는 PR head 최신 커밋 기준 GitHub Actions 통과와
작업지시자 승인을 다시 확인한다.

## 변경 범위

- `planet6897` 열린 PR 8건 체리픽:
  - #2141, #2142, #2143, #2144, #2147, #2149, #2155, #2157
- #2149 테스트 주석 clippy 보정:
  - `tests/issue_2146_no_ls_label_cell_declared_height.rs`
- #2087 회귀 테스트 추가:
  - `tests/issue_2087_document_core_send.rs`
- 원 PR별 review/review_impl 문서:
  - `mydocs/pr/archives/pr_2141_review*.md`
  - `mydocs/pr/archives/pr_2142_review*.md`
  - `mydocs/pr/archives/pr_2143_review*.md`
  - `mydocs/pr/archives/pr_2144_review*.md`
  - `mydocs/pr/archives/pr_2147_review*.md`
  - `mydocs/pr/archives/pr_2149_review*.md`
  - `mydocs/pr/archives/pr_2155_review*.md`
  - `mydocs/pr/archives/pr_2157_review*.md`
- MCP 기준 PDF 증적:
  - `pdf/task2136/neartop_reset_sb2500-2020.pdf`
  - `pdf/task2137/156618554_petfood_press-2020.pdf`
  - `pdf/task2098/page_bottom_fixed_anchor_margin_split-2020.pdf`
  - `pdf/task2146/21761835_jeonjik_exemption_table-2020.pdf`
  - `pdf/hwp3/hwp3-sample14-2020.pdf`
  - `pdf/hwp3/hwp3-sample11-2020.pdf`

## README 반영

첨부 README가 있는 PR은 review 문서에서 해당 README의 전제와 기대값을 확인했다.

- #2141: synthetic fixture 전제와 실문서 재현원 분리
- #2142: 공개 보도자료 샘플과 1쪽 기대값 확인
- #2149: r0 행높이 축 해결과 전체 6p/7p 잔여 문제 분리

## 주의점

- #2141 synthetic fixture는 rhwp 테스트 기대 2쪽이지만 MCP PDF는 1쪽이다. 이 파일은
  한컴 PDF 직접 증적이 아니라 내부 회귀 조건 고정용으로 해석한다.
- #2143 synthetic fixture도 rhwp 테스트 기대 2쪽과 MCP PDF 1쪽이 갈린다. 권위 근거는
  #2144 10k warm PDF survey와 기존/신규 회귀 테스트 조합이다.
- #2149는 r0 행높이 팽창을 해결하지만 `21761835_jeonjik_exemption_table.hwp` 전체 페이지 수는
  rhwp 7쪽, MCP/HWP 2020 기준 PDF 6쪽으로 잔여가 남는다.

## 검증

- `git diff --check upstream/devel...HEAD` pass
- `cargo fmt --check` pass
- `python3 -m py_compile tools/verify_pi_page_vs_hangul.py tools/hangul_row_heights2.py tools/make_ls_ladder.py tools/probe_ls_ladder.py` pass
- focused tests pass:
  - `issue_2136_neartop_reset_sb2500`
  - `issue_2137_topbottom_float_anchor_saved_fit`
  - `issue_2098_margin_boundary_split`
  - `issue_2146_no_ls_label_cell_declared_height`
  - `issue_2151_hwp3_ghost_page`
  - `issue_2087_document_core_send`
- related regression tests pass:
  - `issue_1611_footer_page_bottom_pagination`
  - `issue_1733`
  - `issue_1750_split_guard_spacing_before`
  - `issue_1842`
  - `issue_1891`
  - `issue_2098_page_bottom_fixed_anchor_vpos0`
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` pass
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` pass

## 권고

CI 통과 후 admin merge 가능하다. merge 후 원 PR 8건에는 체리픽 PR로 반영됐음을 알리고,
필요하면 superseded/merged-by-maintainer 취지로 close 후속처리를 진행한다.
