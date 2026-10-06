# task_m100_4947 stage 5: nextest 우선순위 module 계약

## 발견된 회귀

전체 회귀 실행을 시작하자 nextest가 컴파일 전에 종료 코드 96으로 중단됐다. 기존
`.config/nextest.toml`은 `binary(overflow_cell_baseline)`을 우선 실행 대상으로 지정했지만,
weighted sharding 이후 해당 개별 binary는 존재하지 않는다.

## 수정

우선순위의 의미는 특정 Cargo binary가 아니라 `overflow_cell_baseline` source의 테스트를 먼저
시작하는 것이다. filter를 generated suite 안에서도 유지되는 module test 이름
`test(/(^|::)overflow_cell_baseline::/)`으로 변경했다.

manifest의 `nextestPriorities`에도 case와 priority를 기록했다. 생성기 preflight는 priority case가
suite 또는 singleton에 실제 등록되어 있는지 확인하므로 파일 삭제·이름 변경으로 설정이 다시
무효화되는 것을 컴파일 전에 발견한다.

## 검증

수정 전 전체 실행은 0.8초에 설정 파싱 오류로 종료됐다. 수정 후 동일한 전체 release-test
nextest 명령을 다시 실행해 컴파일 시간, 현재 head의 전체 case 보존 여부, 실행 시간과 실패 수를 기록한다.
`CARGO_INCREMENTAL=0`은 사용하지 않는다.
