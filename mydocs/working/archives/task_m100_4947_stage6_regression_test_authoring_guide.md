# task_m100_4947 stage 6: 회귀 테스트 작성·자동 배정 가이드

## 목적

weighted sharding은 Cargo와 CI 내부 구조만 바꾸는 것으로 끝나지 않는다. 신규 회귀 테스트 작성자가
generated harness를 직접 편집하거나 과거처럼 개별 Cargo target을 기대하면 target fan-out이 다시
늘거나 focused 실행이 실패할 수 있다.

## 정본 절차

1. 원본 test source를 `tests/` 최상위에 작성한다.
2. `node scripts/rust-test-suite-manifest.mjs --generate`를 실행한다.
3. `node scripts/run-rust-test.mjs <source 이름>`으로 해당 source만 실행한다.
4. 원본 source와 manifest·harness·Cargo generated 결과를 함께 커밋한다.

이름 변경·삭제는 `--sync`를 사용한다. 전체 `--rebalance`는 일반 작성 절차가 아니라 target 구조를
재조정하는 메인터너 작업으로 제한한다. `tests/generated/*.rs`는 직접 수정하지 않는다.

## 반영 문서

- 외부 기여자 정본: `CONTRIBUTING.md`
- PR 검증 정본: `mydocs/manual/pr_review/local_validation.md`
- 개발 환경 진입점: `mydocs/manual/dev_environment_guide.md`

Node 계약 테스트는 세 문서에 `--generate`, `--sync`, `tests/generated` 안내가 모두 있는지 확인한다.

## 전체 회귀 실측

2026-08-16 macOS에서 다음 명령으로 전체 Rust 회귀를 측정했다. 사용자 요청에 따라
`CARGO_INCREMENTAL=0`은 지정하지 않았다.

```bash
/usr/bin/time -p cargo nextest run \
  --cargo-profile release-test \
  --target-dir target/pr-review \
  --tests --test-threads 8 --no-fail-fast \
  --status-level fail --final-status-level fail
```

- 컴파일 완료: 3분 39초
- 전체 wall time: 407.56초
- 컴파일 이후 실행 구간 추정: 188.56초
- CPU time: user 1852.46초, sys 119.16초
- `nextest list`: 6,541개, ignored 38개, runnable 6,503개
- Rust suite: 44개(통합 40개와 lib/bin 4개)
- 전체 실행 종료 코드: 0, 실패: 0건

sharding source는 558개가 각각 한 번만 배정되어 중복되지 않았다. 동기화된 head에서 기존
6,484개보다 57개가 늘었으므로 manifest 최소 보존값도 6,541개로 올렸다.
