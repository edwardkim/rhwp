# #6205 Stage 1 - B/C integration archive 시간 기반 target 배정

## 배경

현재 `integration-b`와 `integration-c`는 root integration target을 이름순 짝수·홀수
위치로 나눈다. 각 archive와 실행 shard는 하나씩만 유지하므로 worker 수는 작지만, 실행 시간이
긴 target이 한쪽에 몰리면 해당 쪽이 지속적으로 CI 임계 경로가 된다.

완료된 `devel` CI 로그에는 target별 실행 시간이 보관되어 있지 않다. 따라서 과거 로그만으로
신뢰할 수 있는 시간 프로필을 만들거나, 임의의 추정값으로 새 배정을 적용해서는 안 된다.

## 결정

1. `integration-b`, `integration-c` archive와 실행 shard는 각각 하나만 유지한다.
2. target은 알려진 실행 시간이 긴 순서로 정렬하고, 동률이면 이름순으로 정렬한다. 각 target은
   현재 추정 시간이 더 짧은 archive에 배정한다.
3. 실행 시간 정책은 저장소의
   `tests/suites/nextest-target-duration-policy.json`에 버전 관리한다. 기능 PR은 이 파일을 읽기만
   하며, 실행 결과로 자동 수정하지 않는다.
4. 새 target 또는 아직 측정되지 않은 target에는 `fallback_seconds: 1`을 적용한다. 빈 정책에서는
   기존 이름순 교차 배정과 같은 결과가 나와 첫 도입에서 임의의 실행 시간 변화가 없다.
5. B/C 실행은 nextest JUnit 보고서에서 target별 시간을 추출한다. 성공한 `devel` 실행의 B 보고서
   하나와 C 보고서 하나만 별도 artifact로 보관한다.
6. 유지보수자는 두 artifact의 run, ref, commit SHA를 검토한 뒤
   `refresh-nextest-target-duration-policy.mjs`로 정책 후보를 만들고, 별도의 정책 갱신 PR에서
   반영한다.

## 이번 단계 범위

- 결정론적인 시간 기반 target 선택 스크립트를 추가한다.
- nextest JUnit에서 target별 실행 시간을 수집하는 스크립트를 추가한다.
- B/C 측정 보고서를 정책 파일로 합치는 명시적 유지보수 스크립트를 추가한다.
- 빈 정책 호환성, 결정성, JUnit 파싱, B/C 보고서 조합을 검증하는 테스트를 추가한다.
- B/C archive build가 정책 선택기를 사용하도록 연결한다.
- 성공한 `devel`의 B/C 실행에서만 측정 artifact를 30일간 보관한다.

## 이번 단계에서 하지 않는 일

- B/C 실행 shard를 추가하지 않는다.
- source-side unit test를 integration test로 일괄 이동하지 않는다.
- PR 실행 결과로 정책 파일을 자동 생성하거나 자동 커밋하지 않는다.
- 검토·병합된 `devel` 측정값이 없는 상태에서 CI 시간이 줄었다고 주장하지 않는다.

## 수용 기준

1. 빈 정책은 기존 이름순 교차 B/C 분할과 동일한 target 구성을 만든다.
2. 검토된 시간 프로필이 있으면 B/C의 추정 시간을 결정론적으로 균형화한다.
3. B/C 측정 artifact는 test binary target 이름과 실행 시간만 포함한다.
4. 정책 갱신은 성공한 동일 `devel` 실행의 B 보고서와 C 보고서를 각각 하나씩 요구한다.
5. 갱신된 정책에는 각 측정 보고서의 run, ref, commit SHA 출처가 남는다.
