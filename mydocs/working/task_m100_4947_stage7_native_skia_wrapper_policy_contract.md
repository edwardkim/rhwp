# task_m100_4947 stage 7: Native Skia wrapper 정책 계약

## 발견한 회귀

PR 준비 과정에서 CI workflow mirror 테스트 52개 중 3개가 실패했다. Native Skia job은 회귀
test source를 generated suite로 자동 해석하기 위해 `run-rust-test.mjs --cargo-test`를 사용하지만,
Python 정책 테스트는 과거의 직접 `cargo test --test` 문자열만 추출했다. 그 결과 실제로 배선된
6개 target을 모두 누락으로 잘못 판정했다.

## 보정

정책 테스트가 `--test <name>`과 `--cargo-test <name>`을 모두 target 선언으로 인식하도록 수정했다.
release-test와 release의 집합 비교는 각 command 행의 profile flag와 target option을 함께 읽는다.
따라서 wrapper 도입 전·후 형식을 모두 감시하면서 두 profile의 target 대칭성과 classifier 배선을
계속 강제한다.

## 검증

```bash
python3 -m unittest scripts.tests.test_ci_impact_workflow
python3 -m unittest scripts.tests.test_ci_impact_workflow \
  scripts.tests.test_codeql_workflow \
  scripts.tests.test_review_only_fast_pass_workflows
```

두 명령이 모두 통과한 뒤 Stage 7을 커밋하고 전체 PR 검증을 재개한다.
