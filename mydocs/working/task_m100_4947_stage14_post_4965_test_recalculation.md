# task_m100_4947 stage 14: PR #4965 병합 후 테스트 기준 재계산

## 목표

- PR #4965가 병합된 최신 `upstream/devel`을 기준으로 통합 테스트 샤드와 소스 단위 테스트 계층을 다시 계산한다.
- 구조 리팩터링 전후의 nextest 전체 테스트 수가 동일한지 확인한다.
- 이후 PR에서 테스트가 조용히 누락되지 않도록 현재 전체 테스트 수를 최소 기준으로 고정한다.

## 기준

- `upstream/devel`: `88b44de37b491a47494d2acac708b7b86a082951`
- 비교 명령: `cargo nextest list --cargo-profile release-test --target-dir <전용 target> --tests --message-format json`
- 현재 브랜치와 기준 브랜치는 서로 다른 워킹트리와 target 디렉터리에서 목록화했다.

## 재계산 결과

| 항목 | 결과 |
|---|---:|
| 통합 테스트 소스 | 564 |
| 통합 테스트 정적 `#[test]` 표식 | 2,476 |
| 생성 샤드 | 32 |
| 단독 예외 target | 8 |
| Cargo 통합 테스트 target | 40 / 48 |
| 소스 단위 테스트 정적 `#[test]` 표식 | 4,224 |
| 소스 단위 테스트 모듈 | 298 |
| white-box 단위 테스트 | 4,133 |
| support 단위 테스트 | 87 |
| ready 단위 테스트 | 0 |
| `#[cfg(test)]` 지원 항목 | 28 |
| nextest 전체 테스트 수 | 6,556 |
| nextest 기본 실행 대상 | 6,518 |
| ignored 테스트 | 38 |

최신 `upstream/devel`과 현재 브랜치의 nextest 전체 테스트 수는 모두 6,556개다. 내부 크레이트 분리와 통합 테스트 샤딩으로 테스트 바이너리 경로는 달라지지만 테스트 총량은 보존된다.

## 자동 배정

- `tests/issue_3557_package_preservation.rs` -> `regression_suite_011`
- `tests/issue_4397_ruby_hwp5_roundtrip.rs` -> `regression_suite_017`
- `tests/issue_4916_note_vpos_roundtrip.rs` -> `regression_suite_015`

기존 상위 `tests/*.rs`는 최신 기준 트리에 이미 존재하므로 레거시 입력으로 인정한다. 이 PR 병합 이후 새로 추가되는 통합 테스트 소스만 `tests/cases/**` 배치 정책의 적용을 받는다.

## 변경

- 생성 샤드 3개에 새 통합 테스트 소스를 연결했다.
- `tests/suites/manifest.json`에 새 소스 배정을 기록했다.
- 소스 변경으로 이동한 `#[cfg(test)]` 모듈 행 번호를 `tests/suites/unit-test-tiers.json`에 다시 기록했다.
- `minimumNextestCases`를 6,551에서 실제 기준값인 6,556으로 올렸다.

## 확인

- 최신 `upstream/devel` 리베이스: 충돌 없음
- 통합 테스트 샤드 `--sync`: 564 sources / 2,476 static test attrs
- 단위 테스트 계층 `--accept-baseline`: 4,224 tests / 298 modules
- 현재 브랜치 nextest 목록: 6,556개
- 최신 `upstream/devel` nextest 목록: 6,556개

전체 실행, lint 및 정책 게이트는 이 스테이지 커밋 이후 PR 준비 검증에서 수행한다.
