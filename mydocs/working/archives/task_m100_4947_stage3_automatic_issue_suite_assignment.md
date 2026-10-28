# task_m100_4947 stage 3: 신규 issue test 자동 suite 배정

## 배경

Stage 2는 suite manifest와 target 예산을 도입했지만, 새 `tests/issue_*.rs`가 추가될 때마다
개발자가 suite를 선택하고 파일을 이동한 뒤 manifest를 직접 편집해야 했다. 이 수작업은 issue
회귀 테스트가 계속 늘어나는 저장소에서 누락과 다시 늘어나는 test binary를 만들 수 있다.

## 목표

- 새 top-level issue test를 Git 상태에서 자동으로 발견한다.
- suite 선택, 파일 이동, manifest 등록, harness 생성을 한 명령으로 처리한다.
- case 수가 지정 상한에 도달할 때만 다음 test binary를 만든다.
- 기존 standalone issue test는 별도 이관 단계 전까지 그대로 둔다.

## 정책

`tests/suites/manifest.json`의 `automaticIssueAssignment`가 배정 정책의 정본이다.
현재는 `issue_regression_pilot`부터 채우며 suite당 최대 32개 case를 허용한다. pilot에는
20개가 있으므로 다음 12개 신규 issue는 새로운 integration target을 만들지 않는다. 이후에는
`issue_regression_002`, `issue_regression_003` 순서로 32개씩 배정한다.

전체 target 예산은 고정된 비자동 target 수와 실제 자동 suite 수의 합으로 계산한다. 따라서
새 issue 하나마다 예산이 증가하지 않고, 가득 찬 suite 뒤에 필요한 shard 하나만 허용한다.

## 사용법

새 top-level issue 파일을 만든 뒤 일반 생성 명령을 실행하면 Git 추가 파일을 자동 수집한다.

```bash
node scripts/rust-test-suite-manifest.mjs --generate
```

명시적으로 새 파일만 수집하거나 특정 파일을 옮길 수도 있다.

```bash
node scripts/rust-test-suite-manifest.mjs --adopt-new
node scripts/rust-test-suite-manifest.mjs --adopt tests/issue_5000_example.rs
```

CI의 `--check`는 top-level issue target 증가를 발견하면 `--adopt-new` 실행 방법을 출력한다.
개별 case 실행은 기존 `scripts/run-rust-test.mjs <case>` 명령을 그대로 사용한다.

## 검증 계획

이번 단계는 자동 배정 정책과 Node 도구만 변경한다. 장시간 Rust 전수 회귀 테스트는 실행하지
않고, PR 준비 단계에서 Node 계약 테스트와 manifest 검사를 수행한다. `CARGO_INCREMENTAL=0`은
사용하지 않는다.
