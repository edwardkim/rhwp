# task_m100_5164 stage 1: 단일 nextest archive와 테스트 단위 분할

## 배경

PR #4976은 integration test 링크 fan-out을 줄였지만 최종 CI는 약 16분 49초가 걸렸다.
세 archive builder가 공통 `rhwp` 코드를 중복 컴파일하고 archive가 `Lint` 완료를 기다리는
직렬 임계 경로가 남았기 때문이다.

관련 이슈: https://github.com/edwardkim/rhwp/issues/5164

## 계약

- 기본 Rust archive는 full lane마다 한 번만 compile/link한다.
- archive builder와 `Lint`는 preflight 직후 병렬로 시작한다.
- slow worker는 `overflow_cell_baseline` binary만 실행한다.
- regular worker 세 개는 slow binary를 제외한 test case를 `hash:1/3`부터 `hash:3/3`으로 나눈다.
- 네 worker는 동일 archive를 사용한다.
- aggregate는 archive runnable 수와 네 worker 실행 합계가 같은지 확인하고 slow 실행 수 1건을 고정한다.
- `tests/generated` 32개 suite와 manifest 자동 배정 계약은 변경하지 않는다.
- 내부 workspace crate 테스트는 required `Lint` job의 별도 실행 계약을 유지한다.

## 변경

- 세 archive builder를 단일 `build-test-archive` job으로 통합했다.
- target별 archive 계획기를 제거하고 `--tests` 전체 archive 하나를 생성한다.
- worker reusable workflow에 filterset과 선택적 partition 입력을 추가했다.
- CI aggregate와 정책 테스트를 단일 archive 구조에 맞게 갱신했다.

## 검증 결과

- Python workflow 계약 테스트 41건 통과
- generated suite manifest 검사 통과: 565 source, 2,478 static test attr, 32 suite와 9 exception
- generated suite manifest 단위 계약 9건 통과
- source unit-test tier 계약 11건 및 4,225개 source test 기준선 검사 통과
- 변경한 workflow 3개의 `actionlint`와 `git diff --check` 통과
- 격리된 cold target에서 단일 `release-test --tests` archive 생성 통과
  - compile/link + archive: 248.43초
  - test binary: 45개, archive: 242,709,884바이트
  - archive runnable: 6,339개
  - slow filter runnable: 1개, regular filter runnable: 6,338개

저장소 전체 `actionlint`는 이번 변경 밖의 `release-binary.yml`에 이미 존재하는 정보성 ShellCheck
경고로 종료 코드 1을 반환한다. 변경한 세 workflow만 대상으로 한 검사는 통과했다.

`nextest list --partition`은 JSON 목록을 partition별로 축소하지 않으므로 목록 출력의 합으로
hash partition을 증명하지 않는다. 실제 worker의 `Summary` 실행 수를 수집해 aggregate에서 archive
runnable 수와 대조하는 PR CI 결과를 판정 근거로 사용한다.

PR CI에서 확인할 잔여 항목:

- archive builder가 실제로 1개만 실행되는지 확인
- worker 4개의 runnable 합계가 단일 archive 예상 수와 일치하는지 확인
- generated suite 32개가 hash partition 전체에서 누락·중복 없이 실행되는지 확인
- PR #4976 최종 CI 16분 49초와 동일 조건 총 수행시간 비교
