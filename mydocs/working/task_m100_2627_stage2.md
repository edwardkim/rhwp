---
kind: working
status: completed
issue_or_pr: 2627
stage: 2
last_verified: 2026-07-21
---

# PR #2627 · #2655 · #2561 메인터너 통합 Stage 2

## 목적

Stage 1의 수동 통합이 renderer, CLI, 기존 회귀 테스트 전체에 미치는 영향을 확인한다.

## 검증 범위

1. `cargo test --profile release-test --tests`
2. `cargo clippy --all-targets -- -D warnings`
3. 변경 후 최신 `upstream/devel`과의 차이 및 충돌 여부 재확인

## 판정 기준

- #2559는 94쪽 핀과 기준 PDF 92쪽의 잔여 +2쪽을 구분해 기록한다.
- #1733의 241쪽은 한컴 기준 242쪽과 다른 알려진 트레이드로만 허용하며, 테스트가 한컴 정합이라고 오인하게 두지 않는다.
- #2655는 기존 #2552와 중복되는 동작을 되살리지 않고 CLI 파싱 오류의 가시성만 보완한다.
- #2561은 원시 10k 증적이 보존되지 않았으므로 역사적 요약으로만 포함한다.

## 결과

- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 성공
  - #2559의 94쪽 회귀 핀과 #1733의 문서화된 241쪽 핀을 포함한 전체 통합 테스트가 통과했다.
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 성공
  - 경고를 오류로 처리한 상태에서도 경고가 없다.
- 최종 리베이스 기준 `upstream/devel`은 `51acf8538`이다.
  - 기준 head `8ed630944` 이후 #2675와 [PR #2561](https://github.com/edwardkim/rhwp/pull/2561)이 통합됐다.
  - #2675의 `tools/task2430/` 및 기존 오늘할일/PR review 문서는 이번 renderer·CLI·회귀 테스트와 파일 충돌이 없다.
  - #2561은 같은 r17 보고서를 새로 추가해 add/add 충돌이 났다. 원 보고의 수치와 문단은 보존하고, 원시 증적 부재를 밝히는 재현성 보정만 유지했다.

## 다음 단계

검증 기록을 커밋한 뒤 `upstream/devel`의 `51acf8538` 위로 리베이스했다. #2561 보고서의 add/add 충돌만 위 원칙으로 해소했고, 코드·테스트 충돌은 없었다.

- 리베이스 후 대상 검증:
  `CARGO_INCREMENTAL=0 cargo test --profile release-test --test dump_pages_cli --test issue_2559_footnote_footer_band --test issue_1733`
  성공
  - `dump_pages_cli` 3개, #1733 2개, #2559 1개가 모두 통과했다.
