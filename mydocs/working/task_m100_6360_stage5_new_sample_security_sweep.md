---
kind: working
status: done
issue: 6360
---

# #6360 Stage 5: 신규 샘플 보안 sweep 범위 재보정

## 기준

- 이전 checkpoint commit: `8266040da`
- branch: `fix/pdf-reference-fast-pass-20260829`

Stage 4 이후 신규 sample 문서 처리 방향을 다시 확인했다. `security_corpus_regression.rs`와
`injection_scan_contract.rs`는 "기본 대표 샘플 + PR 신규 샘플"이 아니라 **PR에서 새로 추가된
샘플만** 검사해야 한다.

## 보정 원칙

- `RHWP_SECURITY_SWEEP_SAMPLES_JSON`이 없거나 `[]`이면 신규 sample 문서가 없는 PR이므로 clean sweep은
  명시적으로 skip한다.
- env가 있으면 `samples/**/*.hwp`, `samples/**/*.hwpx`, `samples/**/*.hml` 중 신규 추가 파일만 검사한다.
- 대표 샘플이나 기존 `samples/` 전체를 fallback으로 돌리지 않는다. 기존 샘플은 해당 샘플을 들여온 PR
  시점의 검증 책임으로 본다.
- sample PDF/PNG는 시각 증적 또는 기준 자료이므로 review-only fast-pass 범위에 남긴다.

## 수정 내용

- `injection_scan_contract`의 신규 sample clean sweep을 `#[test]`로 복원하고 HML 신규 샘플도 검사 대상에
  포함했다.
- 두 보안 sweep 테스트 이름에서 대표 샘플 fallback 표현을 제거했다.
- `review_only_fast_pass.md`에 신규 문서 샘플은 대표 fallback 없이 신규 추가분만 검사한다고 명시했다.

## 검증 결과

| 항목 | 결과 |
| --- | --- |
| `cargo fmt --all && cargo fmt --all -- --check && git diff --check` | 통과 |
| `node --test scripts/tests/ci-impact-classifier.test.cjs scripts/tests/ci-impact-policy.test.cjs` | 76 통과 |
| `python3 -m unittest scripts/tests/test_ci_impact_workflow.py scripts/tests/test_nextest_archive_workflow.py scripts/tests/test_review_only_fast_pass_workflows.py scripts/tests/test_proptest_roundtrip_workflow.py scripts/tests/test_adapter_diff_workflow.py` | 90 통과 |
| `cargo test --profile release-test --target-dir target/pr-review --test regression_suite_015 --test regression_suite_025 --no-run` | 통과, 33.57초 |
| `RHWP_SECURITY_SWEEP_SAMPLES_JSON='[]' ... new_sample_documents_are_clean_across_all_three_detectors` | 통과, 신규 샘플 없음 skip |
| `RHWP_SECURITY_SWEEP_SAMPLES_JSON='[]' ... new_normal_sample_documents_are_clean` | 통과, 신규 샘플 없음 skip |
| `RHWP_SECURITY_SWEEP_SAMPLES_JSON='["samples/hwp3-sample.hwp","samples/hml/formatting_table.hml"]' ... new_sample_documents_are_clean_across_all_three_detectors` | 통과, 0.06초 |
| `RHWP_SECURITY_SWEEP_SAMPLES_JSON='["samples/hwp3-sample.hwp","samples/hml/formatting_table.hml"]' ... new_normal_sample_documents_are_clean` | 통과, 0.06초 |

## 결론

보안 clean sweep은 더 이상 대표 샘플이나 기존 `samples/` 전체를 fallback으로 실행하지 않는다.
신규 sample 문서가 없는 PR에서는 두 테스트가 명시적으로 skip되고, 신규 HWP/HWPX/HML 문서가 있으면
CI preflight가 넘긴 해당 경로만 검사한다.
