---
kind: pr-review
status: active
pr: 4976
issue: 4947
author: jangster77
base: devel
head: task_m100_4947
last_verified: 2026-08-17
---

# PR #4976 자체검토 - Rust 회귀 테스트 링크 fan-out 자동 샤딩

## PR metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#4976](https://github.com/edwardkim/rhwp/pull/4976) |
| 작성자 | `jangster77` (collaborator self-review) |
| base / head | `devel` / `task_m100_4947` |
| head SHA | `a626eb493a5e59d8d9e98e43fc660a101e63e31a` |
| 규모 | 102 files, +9,900 / -172, 15 commits |
| draft | `false` |
| mergeable | `MERGEABLE` |
| merge state | `BLOCKED` (초기 GitHub Actions 진행 중) |

collaborator 자체 PR이므로 reviewer를 지정하지 않았다. 위 상태값은 문서 작성 시점의 참고값이며,
최종 판단 전 최신 head, GitHub Actions, mergeability를 다시 확인해야 한다.

## 관련 이슈와 변경 범위

[Issue #4947](https://github.com/edwardkim/rhwp/issues/4947)은 integration test 바이너리 557개 이상의
링크 fan-out을 줄여 로컬과 CI의 Rust 전수 회귀 비용을 낮추는 작업이다. PR 본문은 `Closes #4947`을
포함한다.

- integration source 564개를 32개 weighted generated suite와 8개 단독 예외 target으로 구성했다.
- 새 `tests/cases/**` 소스를 manifest에 자동 배정하고 generated harness와 Cargo target을 동기화한다.
- 기존 상위 `tests/*.rs`는 PR base에 존재하는 레거시 입력으로 인정하고 신규 파일만 새 배치 규칙을 적용한다.
- `src/**`와 내부 workspace crate의 단위 테스트를 ready/support/white-box 계층으로 전수 분류한다.
- PR base 대비 신규 상위 integration source와 source-side 단위 테스트 증가를 CI 정책 게이트가 거부한다.
- 암호, 기계 판독 계약, OOXML 차트 코드를 내부 workspace crate로 분리해 컴파일 경계를 마련했다.
- 개발자 가이드에 신규 회귀 테스트 작성·자동 배정·검증 명령을 기록했다.

## 대형 PR 검토 경계

1,000줄을 초과하는 대형 PR이므로 즉시 admin merge하지 않는다. +9,900줄의 대부분은 32개 generated
harness, 564개 소스 배정 manifest, 298개 단위 테스트 모듈 인벤토리와 단계별 근거 문서다. 수동 코드
검토의 핵심 경계는 다음과 같다.

- `scripts/rust-test-suite-manifest.mjs`와 PR-base 비교 모듈
- `scripts/rust-unit-test-tiers.mjs`와 두 정책 테스트
- `.github/workflows/ci.yml`의 base SHA 전달 및 test archive/shard 호출
- 루트와 세 내부 workspace crate 사이의 공개 API 재수출 경계
- `src/schema_registry.rs`의 제품 버전 소유권 파사드

## 검토 중 발견과 보정

첫 전체 nextest에서 `schema_registry_contract::capabilities_schema_registry_matches_constants` 1건이
실패했다. 계약 구현을 내부 `rhwp-contracts`로 옮기면서 `CARGO_PKG_VERSION`이 제품 `0.8.4`가 아니라
내부 크레이트 `0.1.0`을 반환한 것이 원인이었다.

내부 크레이트 버전을 제품 버전으로 위장하지 않고, 루트 `rhwp::schema_registry` 파사드가 자신의 제품
버전을 계약 조립 함수에 명시적으로 주입하도록 스테이지 15에서 보정했다. 해당 계약 단독 재실행과 이후
전체 nextest가 통과했다.

## 로컬 검증 결과

| 검증 | 결과 |
| --- | --- |
| 최신 `upstream/devel` 리베이스 | PR #4965 병합 커밋 `88b44de37` 기준, 충돌 없음 |
| upstream/current nextest 목록 대조 | 양쪽 모두 ignored 포함 6,556개 |
| release-test nextest 전수 실행 | 6,518 passed, 38 skipped, 실패 0 |
| workspace clippy | `--all-targets --all-features -D warnings` 통과 |
| release build | 통과 |
| workspace doc test | 8 passed, 2 ignored |
| WASM32 clippy/check | CI와 같은 `rhwp --lib` 범위로 모두 통과 |
| integration/unit manifest current/base 검사 | 통과: 564 sources, 2,476 integration attrs, 4,224 unit attrs |
| Node 정책 테스트 | 20 passed |
| Python workflow 테스트 | 427 passed, 1 skipped |
| rustfmt / diff check | 통과 |

## 렌더 영향과 시각 검증

renderer, typeset, pagination, paint 계약과 기준 PDF·golden·fixture는 변경하지 않았다. OOXML chart 코드는
내부 크레이트로 컴파일 경계만 이동했고 기존 테스트 165개를 그대로 보존했다. 출력 의미 변경을 주장하지
않으므로 visual sweep과 PDF 대조는 적용하지 않았다.

## 위험과 후속 조건

- 대형 구조 변경이므로 최신 head의 전체 GitHub Actions와 test archive/shard 결과를 독립적으로 확인해야 한다.
- PR #4970과 #4974가 `devel`에 병합되면 같은 head 브랜치를 최신 기준에 리베이스한다.
- 리베이스 뒤 integration 샤드, 단위 계층, nextest 전체 수를 다시 계산하고 로컬 정책 게이트를 재실행한다.
- 후속 push는 기존 GitHub Actions 결과를 무효화하므로 최종 head의 CI가 다시 완료돼야 한다.
- merge 직전 이 문서의 head SHA, 규모, CI 상태와 최종 권고를 최신 값으로 갱신한다.

## 현재 권고

**보류.** 현재 head의 로컬 검증은 모두 통과했지만 초기 GitHub Actions가 진행 중이고 PR #4970/#4974
병합 후 재리베이스·재계산이 예정돼 있다. 후속 보정과 최신 CI가 완료된 뒤 병합 권고로 전환한다.
