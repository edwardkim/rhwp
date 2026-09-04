# Task M100 #3888 4단계 - 두 builder runner의 Cargo test target 분할

- 이슈: [#3888](https://github.com/edwardkim/rhwp/issues/3888)
- 대상 PR: [#3892](https://github.com/edwardkim/rhwp/pull/3892)
- 기준 head: `0066ecdc3`

## 문제

3단계의 binary-affine archive 전송 분할은 worker의 artifact download 총량을 줄였지만,
`Build test archive` 단일 runner가 전체 test binary를 빌드한 뒤 여덟 archive를 순차 생성했다.
실측에서 전체 test binary build는 7분 27초였고, archive 생성과 upload는 각각 약 23초와 18초였다.
archive 생성만 다른 runner에 넘기면 빌드 산출물 전달 비용이 절감분보다 커진다.

Cargo metadata 기준 기본 member `rhwp`에는 test-enabled target 443개가 있으며, 이 중 440개는
독립 integration test target이다. `cargo nextest archive`는 `--test`, `--lib`, `--bin` target
선택을 지원하므로 target 자체를 두 builder에 나누어 컴파일할 수 있다.

## 구현 계획

1. metadata 전용 planner가 기본 package의 test-enabled target을 `slow`, `1`~`7` archive label로
   배정한다. `overflow_cell_baseline` target은 slow label에 단독 배정한다.
2. 4단계에서는 regular archive를 각각 63~64 target으로 배정하고 A가 slow와 네 개, B가 세 개 regular
   archive를 만들었다. 5단계에서 archive 4를 B로 옮기는 보정 계획은
   [`task_m100_3888_stage5.md`](task_m100_3888_stage5.md)에 기록한다.
3. `build-test-archive` job을 A/B matrix로 실행한다. 각 matrix job은 자기 label의 target selector만
   `cargo nextest archive`에 전달하고, 다른 builder의 target은 컴파일하지 않는다.
4. 각 archive 직후 같은 selector로 `cargo nextest list`를 실행해 실제 runnable test 수를 기록한다.
   집계 job은 eight archive의 expected count 합계와 eight worker Summary `run` 합계를 대조한다.
5. test worker는 archive 안에 든 모든 test를 실행한다. regular archive에서 slow test를 제외하는
   runtime filter가 더는 필요하지 않다. slow target 파일에는 실행 test가 하나뿐임을 정적 확인했다.

## 수용 기준

- GitHub Actions에서 `Build test archive (A)`와 `(B)`가 동시에 시작한다.
- 각 builder는 다른 builder가 선택한 Cargo test target을 빌드하지 않는다.
- slow + 일반 7개 worker, worker별 artifact 한 개 download, fail-fast 범위는 유지한다.
- expected archive count 파일과 worker run count 파일이 각각 8개이고 총합이 일치한다.
- workflow 정적 검사와 최신 PR CI에서 5,030개 수준의 전체 default-feature 테스트가 누락 없이 통과한다.

## 로컬 검증 결과

| 검증 | 결과 |
| --- | --- |
| 실제 `cargo metadata --no-deps` planner | 통과: target 443개, 일반 archive별 63~64 target, A/B source bytes 2,209,386 / 2,084,913 |
| synthetic Cargo metadata | 통과: slow 분리, target 중복·누락 없음, archive 8개, regular archive target 수 차이 1 이하 |
| `node --check` planner와 runnable count parser | 통과 |
| synthetic nextest JSON runnable count | 통과: ignored 1건을 제외한 2건 계산 |
| 실제 slow target archive/list | 통과: `--test overflow_cell_baseline`, archive 33,183,059 bytes, runnable 1건 (`overflow_cell_lines_do_not_grow`) |
| `actionlint .github/workflows/ci.yml` | 통과 |
| builder/worker/aggregate shell `bash -n` | 통과 |
| `git diff --check` | 통과 |

제품 Rust/TypeScript/fixture는 변경하지 않았다. 이 호스트에 `cargo-nextest 0.9.140`을 설치했고,
CI와 동일한 slow target selector의 실제 archive 생성·목록 검증을 통과했다. A/B runner의 compile 시간,
archive별 expected/run count 합계와 전체 CI 임계 경로는 최신 PR GitHub Actions에서 확인한다.
