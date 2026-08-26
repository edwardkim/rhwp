---
kind: pr-review
status: approved
pr: 6150
issue: 6123
---

# PR #6150 검토 기록 - RowBreak 경계 행 절단

- 원 PR: [#6150](https://github.com/edwardkim/rhwp/pull/6150)
- 관련 이슈: [#6123](https://github.com/edwardkim/rhwp/issues/6123)
- 작성자: `@planet6897`
- 원 source head: `e14594318ba81983a0bc2f1e4cdfcc5cc58d10f9`
- 통합 후보: [#6154](https://github.com/edwardkim/rhwp/pull/6154), `review/planet6897-6148-6152-20260826`
- 체리픽: `270224b78`, `c48b43c26` (`-x`, 원 작성자와 source SHA 보존)
- 검토 기준: `upstream/devel@27773e5b1150c3e740780cf814e03041aeacc213`
- 라우팅: `collaborator_external_pr` + `intake_and_review` + `local_validation` + `visual_fixture_evidence` + `multi_pr_update_branch`

## 검토 범위

- 저장 프레임이 행 경계보다 짧아도, 현재 측정치와 저장 행 높이의 누적 오차 범위 안이면 해당 행의 줄 조각을 첫 프레임에 남긴다.
- 저장 행 높이가 없을 때는 기존 보수적 동작을 유지한다.
- 새 사례는 source-side test 총량을 늘리지 않고 기존 `typeset` 계약 모듈에 합쳤다.

## 검증 근거

- 누적 체리픽 head에서 `cargo fmt --all -- --check`, Rust suite manifest prepare/check, source unit-tier 정책 검사를 통과했다.
- focused nextest `issue_6123_boundary_row_is_cut_by_lines_not_carried_whole`를 포함한 세 회귀는 `3/3` 통과했다.
- 누적 head 전체 nextest는 `8,397 passed, 43 skipped`로 완료했다.
- Native Skia lib 및 `issue_2225_missing_picture_placeholder`, `render_p37_direct_pdf_export`을 통과했다.
- Docker daemon이 실행 중이지 않아 Docker Compose WASM 경로는 실행하지 못했다. locked host WASM build는 통과했다.
- 통합 후보 `f90d68922ad0fd52ed132184eb260d61568dd664`의 Full CI, Build & Test, Rust CodeQL, Canvas visual diff가 모두 성공했다.

## 시각 증적

- 원본: `samples/issue6123/3112461_railway_emc_criteria.hwpx`
- 원본 SHA-256: `23d545bb87dd23ea755de79341a7b2fab57088493d84fbc691172dbcba1927de`
- 저장 메타데이터: HWPX, `hancom-office-2020` `11.0.0.8362`, 14쪽
- 기준 PDF: `pdf/issue6123-2020.pdf`
- 기준 PDF SHA-256: `da3acec729899697fd3ffa5558acbc46ead0ad0a247253888c055d1fc8b03c22`
- Hancom 2020 MCP job: `da948292-391f-42c6-ac3f-763d974382f4`, 성공, 14쪽
- 임시 sweep 경로: `output/visual-review-planet6897/issue6123/issue6123/`
- 장기 보관 경로: `mydocs/pr/assets/pr_6154/visual-sweep/issue6123/issue6123/`
- 직접 확인: p7 `94.16312%` / `7.74717%`, p8 `96.90439%` / `5.99288%` (pixel match / visual accuracy proxy)

p7의 표가 본문 하단을 넘지 않고, p8에서 이어지는 표 조각이 시작되는 흐름을 기준 PDF와 직접 대조했다. 자동 일치율은 글꼴과 문자 폭 차이의 영향을 받는 보조값이며, 전체 fidelity 통과를 뜻하지 않는다.

## 결론

**승인.** #6154 code candidate CI가 성공했다. review-only 증적 기록 head의 required check를 다시 확인한 뒤 merge한다. #6123의 close 및 원 PR 후속 comment는 통합 PR merge 뒤 `post_merge.md` 절차에 따라 처리한다.
