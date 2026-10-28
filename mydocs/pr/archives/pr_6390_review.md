---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6390
issue: 6360
author: jangster77
---

# PR #6390 review - 장기 baseline 병목 분할

## 라우팅과 metadata

- PR: [#6390](https://github.com/edwardkim/rhwp/pull/6390), 관련 이슈: [#6360](https://github.com/edwardkim/rhwp/issues/6360).
- base route: `collaborator_self_merge.md`.
- modifiers: `intake_and_review.md`, `local_validation.md`.
- loaded documents: `AGENTS.md`, `mydocs/manual/pr_review_workflow.md`,
  `mydocs/manual/pr_review/README.md`, `mydocs/manual/pr_review/collaborator_self_merge.md`,
  `mydocs/manual/pr_review/intake_and_review.md`, `mydocs/manual/pr_review/local_validation.md`.
- 작성자·self-review: `jangster77`; collaborator 본인 PR이므로 reviewer request는 등록하지 않았다.
- 작성 시점 참고값: base `devel`, code candidate
  `93c0bf16a76a761adec48ad43a3a3eeebf190528`, `MERGEABLE/CLEAN`, Open non-draft PR,
  4 files, `+363/-8`.

## 변경 범위와 판단

- `overflow_cell_baseline`, `off_canvas_baseline`, `oracle_page_count_baseline`을 8개에서 16개
  partition으로 늘리고, 30초 이상 샘플 시간을 stderr에 남긴다.
- `issue2063_huge_cellbreak_table.hwp`의 중복 전수 스캔은 세 baseline에서 제외한다. 해당 문서의
  완주 성능과 page-count pin은 `tests/issue_2063.rs::huge_cellbreak_table_paginates_without_quadratic_blowup`
  전용 sentinel이 계속 담당한다.
- overflow-cell 원장은 SVG 문자열 생성까지 수행하는 `render_page_svg_native()` 대신
  `build_page_render_tree()`까지만 수행한다. 기존 TSV와 비교해 값이 동일함을 확인했다.
- 변경은 test baseline과 stage 문서뿐이다. renderer/layout 구현, HWP/HWPX sample, 기준 PDF, golden,
  workflow는 변경하지 않아 visual sweep은 적용 대상이 아니다.

## 로컬 검증

- `cargo fmt --all -- --check`: 통과.
- `node scripts/rust-test-suite-manifest.mjs --prepare && node scripts/rust-test-suite-manifest.mjs --check`:
  통과. 48 integration target과 최소 6,559 nextest case 계약을 확인했다.
- 다음 focused nextest를 실행했다.

  ```bash
  cargo nextest run --cargo-profile release-test --target-dir target/pr-review \
    --test overflow_cell_baseline \
    --test regression_suite_024 \
    --test regression_suite_029 \
    -E 'test(overflow_cell_lines_do_not_grow_partition) | \
        test(off_canvas_baseline::off_canvas_does_not_grow_partition) | \
        test(oracle_page_count_baseline::page_counts_do_not_drift_from_hancom_oracle_partition)' \
    --no-fail-fast --test-threads 12
  ```

  48 passed, 274 skipped, nextest wall 52.255초, shell wall 58.80초였다.
- `RHWP_OVERFLOW_CELL_DUMP`의 16개 partition 결과와 기존
  `tests/fixtures/overflow_cell_baseline.tsv`를 전용 `issue2063` fixture 제외 후 정렬 비교했다.
  diff가 없어서 render-tree 경로 전환이 overflow-cell 원장을 바꾸지 않음을 확인했다.

## GitHub CI 실측

- code candidate의 [Full CI run 33263967924](https://github.com/edwardkim/rhwp/actions/runs/33263967924)는
  lint, archive A/B/C/D build, A/B/C/D worker, Build & Test aggregate를 모두 통과했다.
  Proptest roundtrip과 Adapter inter-diff도 각각 통과했다.
- [CodeQL run 33263967885](https://github.com/edwardkim/rhwp/actions/runs/33263967885)에서
  JavaScript/TypeScript, Python, Rust 분석이 모두 성공했다. 최상위 CodeQL check의 `neutral`은
  분석 job 성공 뒤의 집계 상태이며 실패가 아니다.
- `Refresh nextest target duration data`의 skipped는 PR event에서 expected다. merge 후 `devel` push가
  trusted PR artifact를 사용해 duration data를 갱신한다.

`nextest-target-durations-33263967924-b/c/d` artifact의 testcase 실측값은 다음과 같다.

| baseline testcase | CI 실측 |
| --- | ---: |
| `overflow_cell_baseline` partition 9 | 31.332초 |
| `off_canvas_baseline` partition 9 | 32.484초 |
| `oracle_page_count_baseline` partition 11 | 5.253초 |

이는 이슈의 직전 policy 값인 overflow 385.481초, oracle 327.829초, off-canvas 191.485초의
critical path가 사라졌다는 직접 근거다. artifact의 B/C/D testcase 누적 시간은 각각 499.024초,
698.604초, 682.769초지만 병렬 실행 누적값이므로 worker wall time과 동일하지 않다. 실제 worker는
B 3분 49초, C 4분 28초, D 4분 32초로 완료했다.

남은 장기 항목은 전용 coverage를 유지하는
`issue_2063::huge_cellbreak_table_paginates_without_quadratic_blowup` 168.994초와 범위 밖인
`ir_field_sweep_baseline::ir_field_sweep_does_not_regress` 111.175초다. 따라서 #6360은 아직 열어 두고,
후속 stage에서 두 축을 별도로 다룬다.

## 최종 권고와 후속 조건

**수용.** 중복 초대형 fixture 스캔을 제거하면서 전용 sentinel을 보존했고, focused 검증과 exact code
candidate의 Full CI 실측이 모두 이를 뒷받침한다.

- 이 review 및 오늘할일만 포함한 trailing commit을 같은 branch에 추가한다.
- 최종 merge 전에는 trailing head의 required CI, `MERGEABLE/CLEAN`, head SHA를 다시 확인하고
  작업지시자 승인을 받는다.
- merge 뒤에는 `post_merge.md`에 따라 duration refresh가 PR artifact를 정상 재사용했는지와
  `ci-metrics/nextest-target-durations` 갱신을 확인한다.
- #6360에는 [종료 보류와 잔여 병목 근거](https://github.com/edwardkim/rhwp/issues/6360#issuecomment-5463728710)를
  기록했다. 이 확인이 끝나기 전에는 이슈를 close하지 않는다.
