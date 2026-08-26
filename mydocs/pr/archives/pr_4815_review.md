---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-15
---

# PR #4815 검토 - HWP3 손상 입력 강건성 감사와 줄간격 panic 차단

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4815](https://github.com/edwardkim/rhwp/pull/4815) · @kevin9327 |
| 원 head | `c27dab4b13efe00b31a8d6a4525dce83f3d41801` |
| 기준선 | `upstream/devel@4cf8a5898` |
| 누적 적용 | `c27dab4b1` → `bd7c7376d` |
| 메인터너 보정 | `c1747311a` |
| 원 CI | 작성 시점 참고값: CI 진행 중, CodeQL Rust 진행 중, mergeable `MERGEABLE` |
| 관련 이슈 | [#4814](https://github.com/edwardkim/rhwp/issues/4814), 무한루프 후속 [#4813](https://github.com/edwardkim/rhwp/issues/4813) |

## 변경과 보정 이유

결정적 절단·바이트 flip·header zero 변형으로 HWP corpus를 파싱해 panic과 timeout을 찾는
`gym/tools/robustness.py`를 추가하고, 실제로 발견된 HWP3 percent 줄간격 곱셈의 i32 overflow panic을
차단한다. parser 동작이 바뀌지만 손상 입력의 안정성 경계만 다루므로 기준 문서의 시각 검증은 대상이 아니다.

원 수정은 곱셈 중간값을 i64로 올렸지만 마지막 `as i32` 변환이 범위를 넘으면 wrap할 수 있었다. 보정은
공통 helper에서 i32 경계로 포화해 panic과 wrap을 함께 차단하고 극값 회귀를 추가했다. 강건성 감사도 빈
입력에서 위치 기반 mutant 생성 중 예외가 나지 않도록 별도 변형을 만들며, Windows NTSTATUS crash와 일반
CLI 오류 코드를 구분한다.

## 완료 검증

- `cargo fmt --all -- --check`: 통과.
- `cargo test --profile release-test --target-dir target/pr-review corrupt_hwp3_line_spacing_does_not_overflow_panic -- --nocapture`: 통과.
- `cargo test --profile release-test --target-dir target/pr-review percent_line_spacing_saturates_corrupt_extremes -- --nocapture`: 통과.
- `python3 -m unittest ... test_gym_robustness.py ...`: 46건 통과.
- `python3 gym/tools/robustness.py --bin target/pr-review/release-test/rhwp --limit 16 --timeout 5 --json`:
  16개 표본·128개 변형, panic 0·hang 0, 우아한 실패/부분복구 65건.
- `git diff --check`: 통과.

**메인터너 보정 후 수용 후보.** #4813의 loop cursor 전진 문제는 별도 범위로 유지한다. merge 직전에는
최신 원 PR head와 required check를 다시 확인한다.
