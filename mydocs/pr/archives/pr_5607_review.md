---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-19
---

# PR #5607 검토 - lpaiu-cs #5556·#5570 통합

## 접수 메타데이터

| 항목 | 검토 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#5607](https://github.com/edwardkim/rhwp/pull/5607) / jangster77 |
| base / 통합 branch | `devel` / `integration/lpaiu-cs-20260819` |
| 최신 기준선 | `upstream/devel@9208c03b5` |
| 원 PR | [#5556](https://github.com/edwardkim/rhwp/pull/5556), [#5570](https://github.com/edwardkim/rhwp/pull/5570) / lpaiu-cs |
| 체리픽 입력 | `6607e42220194a293b209699c10d48f602c681aa`, `6c3ae7522c8d8ab20b3c158622d8678c3fc74051` |
| 메인터너 보정 | `#2225` Native Skia 함수 게이트를 독립 test target으로 분리 |

## 통합 판정

차단 결함은 발견하지 못했다. 두 원 PR은 최신 `upstream/devel` 위로 충돌 없이 누적했고, 검토 중
발견한 #2225 선택기 결함은 suite policy의 독립 target 예외와 회귀 계약으로 보정했다.

## 범위와 제외 사항

- #5556은 `PageRenderer`의 문서 범위 파생 상태를 문서 경계에서 회수하는 변경을 포함한다.
- #5570은 개체 회전·대칭을 스냅샷 저장 대신 역연산으로 기록해 스냅샷 예산을 줄이는 변경을 포함한다.
- #5570의 체리픽 입력은 검토 시작 시점 원본 `6c3ae752`로 고정했다. 현재 원 PR head
  `56ee07dfd4ce03a836477ee7ee81a05d91bffa41`의 후속 변경은 포함하지 않았다.
- 기존 메인터너 보정 `4c668b939632374f723c9a5ba2617f32caf17c66`도 작업지시대로 참고만 하고
  통합 branch에 넣지 않았다.

## #2225 메인터너 보정

`tests/issue_2225_missing_picture_placeholder.rs`는 독립 integration target인데, suite policy가
일반 generated suite로 배정해 `run-rust-test.mjs`가 `regression_suite_014` 내부 선택기로 실행했다.
그 결과 Native Skia 공식 명령은 `0 tests run`과 exit 4로 실패했다.

`tests/suites/suite-policy.json`에 독립 target 예외를 추가하고,
`scripts/tests/rust-test-suite-manifest.test.mjs`에 target과 인자 구성을 고정하는 회귀 계약을
추가했다. 보정 뒤 같은 명령은 2개 테스트를 선택해 모두 통과했다.

## 검증

| 범위 | 결과 |
| --- | --- |
| release build / release lib | 통과 |
| 전체 `nextest --no-fail-fast` | 7,313 passed, 38 skipped |
| Native Skia lib / #2225 / 직접 PDF | 58 passed / 2 passed / 4 passed |
| suite manifest 계약 | 17 passed |
| fmt, diff check, clippy, doctest | 통과 |
| Studio TypeScript / `npm test` | 통과 / 997 passed |
| WASM | Docker 미사용 환경에서 `wasm-pack build --target web --out-dir pkg` 통과 |

전체 `nextest`는 #2225 선택기 보정 전에 이미 `--no-fail-fast`로 통과했다. 보정은 test target
메타데이터만 바꾸므로 중복 전체 실행 대신 manifest 계약과 실패했던 #2225 명령만 재검증했다.

초기 code candidate CI는 Build & Test, Lint, Native Skia, Canvas visual diff, CodeQL, test archive,
slow·regular shard, Proptest, adapter inter-diff를 포함해 모두 통과했다. 영향 정책상 Frontend unit과
GitHub WASM job은 skipped였고, 로컬 WASM 빌드로 대체 검증했다.

## 계보와 후속

- 최초 code candidate CI 성공 뒤 `upstream/devel@9208c03b5`로 다시 충돌 없이 리베이스했다.
- 이 review archive와 오늘할일은 trailing commit으로 push한다.
- trailing head의 required CI가 성공하면 #5607을 merge하고 원 PR #5556·#5570의 통합 근거,
  branch 정리와 작업 기록 상태를 후속 절차에 따라 처리한다.
