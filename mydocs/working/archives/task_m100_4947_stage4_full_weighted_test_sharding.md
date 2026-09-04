# task_m100_4947 stage 4: 전체 테스트 weighted sharding

## 전수 조사 결과

현재 Cargo testable target은 integration 539개와 lib/bin 4개를 합쳐 543개다. 소스 정적
집계는 `src` 4,154개, integration 2,463개 test attribute이며 동기화된 현재 head의 nextest
기준 실행 case는 6,541개다. 병목은 case 실행보다 539개 integration crate의 반복 컴파일·링크
fan-out이다.

integration source 558개를 전수 조사한 결과 550개는 module harness로 통합할 수 있다.
나머지는 `mod ...;`, `#[path]`, crate-root 전용 선언 등 경로·링크 의미를 보존하기 위해
singleton target으로 유지한다.

## 구조 변경

- Cargo package의 `autotests`를 끈다.
- 원본 test source는 이동하지 않는다. 상대 fixture와 `include_*` 경로를 그대로 보존한다.
- 550개 source를 32개 generated harness에 weighted LPT 방식으로 배정한다.
- source byte 수와 정적 test attribute 수를 함께 가중치로 사용한다.
- module 통합 blocker는 자동 탐지해 기존 파일명 singleton target으로 등록한다.
- Cargo의 `[[test]]` 블록과 harness는 manifest에서 함께 생성한다.

예상 integration target은 `32 suites + 8 exceptions = 40개`다. 기존 539개 대비 92.6%를
줄이면서 현재 head의 전체 실행 case는 유지한다. lib/bin unit test는 이미 4개 target에 모여 있으므로
추가 분해하지 않는다.

일반 신규 test source는 기존 32개 suite에 편입되므로 target 수가 증가하지 않는다. 자동
singleton이 필요한 신규 blocker를 위해 전체 integration target 상한은 48개로 두며, CI가
상한 초과를 거부한다. nextest case 수는 향후 증가를 허용하되 전수 실측 6,541개 아래로 감소하지
않는 최소 보존 계약으로 관리한다.

## 신규 테스트 처리

`--generate`는 source roots를 전수 대조하고 새 파일을 현재 weight가 가장 낮은 suite에
배정한다. 일반 신규 파일은 integration target 수를 늘리지 않는다. blocker가 있는 파일만
안전한 singleton으로 등록한다. 기존 배정은 유지하며 전체 재분산은 명시적인 `--rebalance`에서만
수행한다.

```bash
node scripts/rust-test-suite-manifest.mjs --generate
node scripts/rust-test-suite-manifest.mjs --rebalance
node scripts/rust-test-suite-manifest.mjs --check
node scripts/run-rust-test.mjs issue_1035_alignment
```

## 호환 계약

개별 source 이름은 `scripts/run-rust-test.mjs`가 suite target과 module filter로 변환한다.
native-skia CI도 동일한 래퍼의 cargo-test 모드를 사용하므로 기존 source 단위 검증 범위를
유지한다. singleton 예외는 filter 없이 기존 target 전체를 실행한다.

## 검증 계획

### 적용 직후 구조 검증

- 전체 source: 558개
- 정적 test attribute: 2,463개
- generated suite: 32개
- 자동 singleton 예외: 8개
- Cargo integration target: 40개
- suite weight 범위: 444,290~555,740
- Node manifest 계약 테스트: 4건 통과

이 단계에서는 생성기 자체 계약까지만 적용한다. PR 준비 단계에서는 다음을 확인한다.

1. Cargo metadata integration target이 40개인지 확인한다.
2. nextest list의 실행 case가 최소 6,541개로 유지되는지 확인한다.
3. 전체 release-test nextest를 실행해 실패 0건을 확인한다.
4. native-skia 개별 source 게이트가 새 suite filter로 동일하게 실행되는지 확인한다.
5. cold build와 단일 source 수정 후 증분 build 시간을 기존 기준과 비교한다.

검증에서는 `CARGO_INCREMENTAL=0`을 사용하지 않는다.
