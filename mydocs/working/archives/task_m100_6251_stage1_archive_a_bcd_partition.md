# #6251 Stage 1 - Archive A 통합과 B/C/D 시간 기반 분할

## 배경

[CI run 33075195320](https://github.com/edwardkim/rhwp/actions/runs/33075195320)에서
Archive A의 두 worker는 실제 nextest 실행이 각각 20.809초와 15.927초였지만, Archive B와
C는 각각 10분 19초와 9분 12초가 걸렸다. 전체 CI의 임계 경로는 A가 아니라 B/C
integration 실행이다.

## 결정

1. `--lib` 전용 Archive A의 `hash:1/2`, `hash:2/2` worker를 하나의 `hash:1/1`
   worker로 통합한다.
2. root integration target 전체를 기존 B/C 두 archive 대신 B/C/D 세 archive에
   시간 기반 LPT 방식으로 결정론적으로 배정한다.
3. B/C/D는 각각 build job 하나와 test worker 하나를 사용한다. A/B/C/D의 전체
   worker 수는 네 개로 유지한다.
4. B/C/D의 JUnit target duration artifact는 같은 PR 또는 `devel` run의 동일
   provenance를 가진 세 보고서가 모두 있을 때만 metrics data branch에 반영한다.
5. 집계 job은 A/B/C/D 각각의 archive runnable count와 worker count가 정확히 같은지
   검증해 누락과 중복을 막는다.

## 기대 효과와 한계

- A의 두 runner가 만드는 setup/download 비용을 없앤다.
- B/C/D의 integration 실행은 시간 기반으로 약 3분할되어 현재 B의 10분 19초보다 짧은
  임계 경로를 목표로 한다.
- D archive도 공통 Rust 의존성을 별도 compile/link하므로 총 runner 사용량은 증가할 수 있다.
  이 단계의 성공 기준은 총 runner-minutes가 아니라 실제 PR CI의 wall time 단축과 전체 회귀
  커버리지 유지다.

## 검증 계획

1. selector 단위 테스트로 B/C/D 결정성, 예상 시간, 누락·중복 없는 전체 배정을 확인한다.
2. metrics refresh 단위 테스트로 동일 provenance의 B/C/D 세 측정값만 허용함을 확인한다.
3. workflow 계약 테스트로 D builder/worker, A 단일 worker, aggregate count 검증을 확인한다.
4. PR CI에서 B/C/D 실측 시간과 전체 Build & Test 완료 시간을 run `33075195320`과 비교한다.
