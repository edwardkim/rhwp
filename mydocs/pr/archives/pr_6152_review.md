---
kind: pr-review
status: approved
pr: 6152
issue: 6124
---

# PR #6152 검토 기록 - rowspan 인접 행 높이 복원

- 원 PR: [#6152](https://github.com/edwardkim/rhwp/pull/6152)
- 관련 이슈: [#6124](https://github.com/edwardkim/rhwp/issues/6124)
- 작성자: `@planet6897`
- 원 source head: `0d36692333af5e5c30e2d84471781da83d9b970d`
- 통합 후보: [#6154](https://github.com/edwardkim/rhwp/pull/6154), `review/planet6897-6148-6152-20260826`
- 체리픽: `f90d68922` (`-x`, 원 작성자와 source SHA 보존)
- 검토 기준: `upstream/devel@27773e5b1150c3e740780cf814e03041aeacc213`
- 라우팅: `collaborator_external_pr` + `intake_and_review` + `local_validation` + `visual_fixture_evidence` + `multi_pr_update_branch`

## 검토 범위

- 공통 행 높이로 비례 축소한 뒤에도 rowspan 셀의 내용·여백 높이가 부족하면 마지막 span 행에 부족분을 더한다.
- TAC table의 두 축소 경로 모두에서 복원 함수를 호출한다.
- 신규 integration case가 마지막 줄이 셀 안에 남는지 확인한다.

## 검증 근거

- 누적 체리픽 head에서 `cargo fmt --all -- --check`, Rust suite manifest prepare/check, source unit-tier 정책 검사를 통과했다.
- focused nextest `issue_6124_merged_cell_keeps_its_last_line_inside_the_cell`를 포함한 세 회귀는 `3/3` 통과했다.
- 누적 head 전체 nextest는 `8,397 passed, 43 skipped`로 완료했다.
- Native Skia lib 및 `issue_2225_missing_picture_placeholder`, `render_p37_direct_pdf_export`을 통과했다.
- Docker daemon이 실행 중이지 않아 Docker Compose WASM 경로는 실행하지 못했다. locked host WASM build는 통과했다.
- 통합 후보 `f90d68922ad0fd52ed132184eb260d61568dd664`의 Full CI, Build & Test, Rust CodeQL, Canvas visual diff가 모두 성공했다.

## 시각 증적

- 원본: `samples/issue6124/2737927_housing_evaluation_guideline.hwpx`
- 원본 SHA-256: `2421db10c9ac4a4fcb6a60d2dc7bab3ed28c0e81ba11ccddeaeccdf20af6752f`
- 저장 메타데이터: HWPX, `hancom-office-2020` `11.0.0.8362`, 8쪽
- 기준 PDF: `pdf/issue6124-2020.pdf`
- 기준 PDF SHA-256: `3b8a5deb8c10d828d0c6da5b66b8dd1e47ae7300a49877a184b548aa42e9e99c`
- Hancom 2020 MCP job: `a2c0837f-b4b5-4d4c-8f8f-4982c87cb950`, 성공, 8쪽
- 임시 sweep 경로: `output/visual-review-planet6897/issue6124/issue6124/`
- 장기 보관 경로: `mydocs/pr/assets/pr_6154/visual-sweep/issue6124/issue6124/`
- 직접 확인: p8 `review/review_008.png`, pixel match `91.11428%`, visual accuracy proxy `26.40204%`

p8에서 세로 병합 셀의 표 하단과 다음 행 경계가 유지되며, 마지막 내용 행의 잘림이나 표 밖 넘침이 없음을 기준 PDF와 직접 대조했다. 자동 일치율은 글꼴과 문자 폭 차이의 영향을 받는 보조값이며, 전체 fidelity 통과를 뜻하지 않는다.

## 결론

**승인.** #6154 code candidate CI가 성공했다. review-only 증적 기록 head의 required check를 다시 확인한 뒤 merge한다. #6124의 close 및 원 PR 후속 comment는 통합 PR merge 뒤 `post_merge.md` 절차에 따라 처리한다.
