---
kind: pr-review
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5935 검토 - 한컴오피스 2010 마지막 저장 버전 식별

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5935](https://github.com/edwardkim/rhwp/pull/5935) / [@jangster77](https://github.com/jangster77) |
| 관련 issue | [#5934](https://github.com/edwardkim/rhwp/issues/5934) |
| base / code candidate | `devel` `61e4390436644d306bd2e0402bc24d841066b671` / `152bcbe8fcc611bba6d042a332ac79fbaa18f0cf` |
| code candidate 변경 규모 | 11 files, +26 / -4 |
| 작성 시점 상태 | non-draft; 새 code candidate의 GitHub required checks는 trailing push 뒤 재확인 필요 |
| reviewer | 작성자 본인 self-review, 별도 reviewer 미지정 |

GitHub mergeability와 CI 상태는 작성 시점 참고값이다. 이 trailing 기록 commit의 최신 head가
required check를 통과하고 작업지시자가 승인한 뒤에만 merge한다.

## 변경과 판단

- HWP5 `HwpSummaryInformation.revisionNumber`의 주 버전 8을 `hancom-office-2010`으로 분류한다.
- 기존 추적 fixture `samples/basic/Hyper(hwp2010).hwp`의 `8.0.0.460`을 `info --json` 계약에 추가했다.
- 기존 2018/2022/2024 분류와 알 수 없는 주 버전의 `product:null` 동작은 유지한다.
- CLI 매뉴얼과 에이전트 지식 지도의 `lastSavedWith` 설명을 2010 범위까지 갱신했다.

renderer, layout, 기준 PDF는 바뀌지 않았다. 아래의 저장 버전 증적 sample만 추가했으며,
JSON 메타데이터 계약 변경이므로 시각 검증은 요구하지 않았다.

## 원본 증적 fixture

한컴오피스별 새 저장본 네 개를 `samples/pr5935/`에 바이트 보존하고 계약 테스트에 직접 추가했다.

| 파일 | `revisionNumber` | SHA-256 |
| --- | --- | --- |
| `samples/pr5935/test-2010.hwp` | `8.0.0.466` | `a448aea8bb18299ab2e391ede290dd46ebe65174b543a5b8551ed7447db5a635` |
| `samples/pr5935/test-2018.hwp` | `10.0.0.5060` | `7abedd9239954e40bbf46e834e2d17caa81adf35acd176a49965eed87981d465` |
| `samples/pr5935/test-2022.hwp` | `12.0.0.4605` | `5df570e17e3ba0a716da2ad2f149c963a778cfa9e10b905f00218b9cf2b28a70` |
| `samples/pr5935/test-2024.hwp` | `13.0.0.3379` | `91a4f0cec582df4d21c2eced423482e6307aafd998639ac32c57f55ae56e740e` |

이 변경은 metadata parser와 CLI 계약만 검증하며 renderer 출력 비교를 주장하지 않는다. 새 fixture의
IR field sweep과 overflow-cell 원장 결과는 아래 로컬 검증에 기록한다.

overflow-cell 원장은 새 `samples/pr5935/` fixture에 0행을 기록했다. 반복 실행에서 동일하게 확인된
기존 감소도 원장에 반영했다: `issue3637/regulatory_impact_nested_table_escape.hwpx`의 8행은 해소됐고,
`task2097/75544_pii_bunseok.hwpx`는 19행에서 2행으로 감소했다. parser 변경이나 새 저장 버전 fixture가
renderer 동작을 바꾼 결과로 해석하지 않으며, 감소분만 래칫으로 조였다.

## 로컬 검증

- `node scripts/run-rust-test.mjs info_hancom_save_version_contract -- --locked --cargo-profile release-test --target-dir target/pr-review`를 exit code 0으로 통과했다.
  - 기존 fixture와 `samples/pr5935/`의 새 저장 HWP 2010/2018/2022/2024는 각각 대응하는 `lastSavedWith.product`와 버전을 반환했다.
  - HWP3와 HWPX는 `lastSavedWith:null`을 유지했다.
- `RHWP_IR_SWEEP_DUMP=/tmp/pr5935_ir_field_sweep_current.tsv cargo test --locked --profile release-test --target-dir target/pr-review --test regression_suite_014 ir_field_sweep_baseline::ir_field_sweep_does_not_regress -- --nocapture`를 통과했다.
  - 903건(3 skipped), 519개 발산 경로, 총 238,920건을 측정했고 기준 TSV와 diff가 없었다.
- `RHWP_OVERFLOW_CELL_DUMP=/tmp/pr5935_overflow_cell_current.tsv cargo test --locked --profile release-test --target-dir target/pr-review --test overflow_cell_baseline -- --nocapture`를 두 번 통과했다.
  - 두 dump는 동일했고, 785건(3 skipped), 14개 문서, 총 546줄이며 갱신한 기준 TSV와 diff가 없었다.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`를 fixture 추가 뒤 exit code 0으로 통과했다.
- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`,
  `cargo fmt --all -- --check`, `git diff --check`를 통과했다.
- `node scripts/rust-test-suite-manifest.mjs --prepare` 및 `--check`,
  `node scripts/rust-unit-test-tiers.mjs --check`를 통과했다.

모든 Cargo 검증은 `--locked`와 공유 검토 산출물 `target/pr-review`로 순차 실행했다. 파생 harness와
manifest는 검증에만 사용했고 PR diff에 포함하지 않는다.

## 최종 판정

**수용 권고, trailing CI 대기.** 확인된 2010 저장 메타데이터를 분류하되, 근거가 없는 주 버전에는
계속 제품 연도를 부여하지 않는다. 최신 trailing head의 GitHub Actions가 모두 성공하고 작업지시자가
승인한 뒤 merge한다.
