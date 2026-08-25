---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6074 review - TAC 셀 마지막 줄 음수 trailing 줄간격 클램프 (#6030)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6074](https://github.com/edwardkim/rhwp/pull/6074) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| 원 head | `c7d3c66398acdb3c093fdea6fef573e14f771a6c` |
| 통합 적용 commit | `a340f70ee` |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, CI 24 success/1 skip |
| 판정 | **수용 권고** |

## 검토 요약

TAC 표 셀의 마지막 줄에 음수 trailing 줄간격이 포함되면서 행 높이가 과소 측정되고, 실제 페인트는
마지막 글리프 박스를 줄이지 않아 descender가 행 괘선에 깎이는 문제를 수정한다. 변경은
`height_measurer`의 TAC 셀 마지막 줄 측정과 #6030 재현 샘플·회귀 테스트·IR sweep 원장에 제한되어
있다.

원 PR은 한글 2022 COM PDF 실측값과 before/after 수치를 함께 제공했고, 통합 후보에서도 신규
회귀 테스트와 IR sweep baseline을 재검증했다.

## 검증 근거

- 원 PR CI: 전체 GitHub checks 성공, 실패 0
- `node scripts/run-rust-test.mjs issue_6030_tac_last_line_negative_trailing -- --cargo-profile release-test --target-dir target/pr-review`:
  1 pass, 115 skipped
- `RHWP_IR_SWEEP_DUMP=/tmp/ir_field_sweep_current_6074.tsv cargo test --locked --profile release-test --target-dir target/pr-review --test regression_suite_028 ir_field_sweep_baseline::ir_field_sweep_does_not_regress -- --nocapture`:
  1 pass, 130 filtered out, 82.89s
- `diff -u <(LC_ALL=C sort tests/fixtures/ir_field_sweep_baseline.tsv) <(LC_ALL=C sort /tmp/ir_field_sweep_current_6074.tsv)`:
  차이 없음
- 최종 통합 head 전체 nextest:
  `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`
  run ID `4a3bdd9c-04c9-47f7-ab52-408c6022e116`, 8,364 pass, 43 skip, 208.190s

## 권고

원 PR의 증적과 통합 후보 로컬 검증이 같은 방향을 가리킨다. #6030의 행 높이 과소 측정 문제는
수용 가능하며, 별도 메인터너 보정은 필요하지 않다.
