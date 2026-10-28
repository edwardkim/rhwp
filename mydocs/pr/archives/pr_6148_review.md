---
kind: pr-review
status: approved
pr: 6148
issue: 6122
---

# PR #6148 검토 기록 - 표 셀 안 TAC 그림 줄바꿈

- 원 PR: [#6148](https://github.com/edwardkim/rhwp/pull/6148)
- 관련 이슈: [#6122](https://github.com/edwardkim/rhwp/issues/6122)
- 작성자: `@planet6897`
- 원 source head: `617996125f153ce1dbfe585f37a20a06f509a998`
- 통합 후보: [#6154](https://github.com/edwardkim/rhwp/pull/6154), `review/planet6897-6148-6152-20260826`
- 체리픽: `5fe9d862e` (`-x`, 원 작성자와 source SHA 보존)
- 검토 기준: `upstream/devel@27773e5b1150c3e740780cf814e03041aeacc213`
- 라우팅: `collaborator_external_pr` + `intake_and_review` + `local_validation` + `visual_fixture_evidence` + `multi_pr_update_branch`

## 검토 범위

- 표 셀 안의 TAC inline 그림이 셀 본문 폭을 넘을 때 저장된 다음 줄로 재배치한다.
- 부분 표 레이아웃에서도 같은 줄바꿈과 그림 아래 캡션의 흐름 높이를 유지한다.
- `tests/fixtures/ir_field_sweep_baseline.tsv` 충돌은 최신 `devel`의 기존 항목과 #6122의 두 항목을 함께 보존했다.

## 검증 근거

- 누적 체리픽 head에서 `cargo fmt --all -- --check`, Rust suite manifest prepare/check, source unit-tier 정책 검사를 통과했다.
- focused nextest `issue_6122_cell_tac_pictures_stack_inside_the_cell`를 포함한 세 회귀는 `3/3` 통과했다.
- 누적 head 전체 nextest는 `8,397 passed, 43 skipped`로 완료했다.
- Native Skia lib 및 `issue_2225_missing_picture_placeholder`, `render_p37_direct_pdf_export`을 통과했다.
- Docker daemon이 실행 중이지 않아 Docker Compose WASM 경로는 실행하지 못했다. locked host WASM build는 통과했다.
- 통합 후보 `f90d68922ad0fd52ed132184eb260d61568dd664`의 Full CI, Build & Test, Rust CodeQL, Canvas visual diff가 모두 성공했다.

## 시각 증적

- 원본: `samples/issue6122/2181727_press_guard_test_method.hwp`
- 원본 SHA-256: `feb7fd1860b5b1b74e1bd75ec28a6d9ed506db41cfb2c789f06955521d183dc6`
- 저장 메타데이터: HWP5, `hancom-office-2020` `11.0.0.3524`, 12쪽
- 기준 PDF: `pdf/issue6122-2020.pdf`
- 기준 PDF SHA-256: `d0c9f90023207192ac57772b572a5ee3f1c7902d0e14e34798a66127ce53a17e`
- Hancom 2020 MCP job: `f1b5db0c-fc03-4be0-80f5-5c4fa46837ac`, 성공, 12쪽
- 임시 sweep 경로: `output/visual-review-planet6897/issue6122/issue6122/`
- 장기 보관 경로: `mydocs/pr/assets/pr_6154/visual-sweep/issue6122/issue6122/`
- 직접 확인: p6 `review/review_006.png`, pixel match `92.84330%`, visual accuracy proxy `13.85877%`

p6에서 그림이 셀 밖으로 넘치지 않고 다음 줄에 배치되며, 캡션이 그림 아래에 유지됨을 기준 PDF와 직접 대조했다. 자동 일치율은 글꼴과 문자 폭 차이의 영향을 받는 보조값이며, 전체 fidelity 통과를 뜻하지 않는다.

## 결론

**승인.** #6154 code candidate CI가 성공했다. review-only 증적 기록 head의 required check를 다시 확인한 뒤 merge한다. #6122의 close 및 원 PR 후속 comment는 통합 PR merge 뒤 `post_merge.md` 절차에 따라 처리한다.
