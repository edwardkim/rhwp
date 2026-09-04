# #3820 Stage 183 - upstream test helper rebase correction

## 목적

최신 `upstream/devel` 리베이스 뒤 전체 `release-test` 회귀를 시작하는 과정에서
`typeset.rs`의 #3820 first-fragment frame 단위 테스트가 helper 이전 이름을 import한
컴파일 오류를 보정한다.

## 재현

```text
CARGO_INCREMENTAL=0 cargo nextest run \
  --cargo-profile release-test \
  --target-dir target/pr-review \
  --tests --test-threads 10 --no-fail-fast

error[E0432]: unresolved import
  super::saved_rowbreak_first_fragment_overflow_allowance
```

구현 helper는 이미 `saved_rowbreak_first_fragment_flow_overflow_allowance`로 이름이
정리되어 있었지만, 같은 파일의 테스트 모듈 import와 다섯 호출이 이전 이름을 유지했다.

## 수정

- 테스트 모듈의 import와 다섯 호출을 현재 helper 이름으로 맞췄다.
- production RowBreak 계산은 변경하지 않았다.

## 검증 상태

동일한 `target/pr-review`에서 전체 `release-test` 회귀와 Native Skia 3종을 다시
실행한다. 결과는 PR 본문에 기록한다.
