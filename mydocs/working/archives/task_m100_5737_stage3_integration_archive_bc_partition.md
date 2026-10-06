# #5737 Stage 3 - integration archive B/C 교차 분할

## 배경

Stage 2의 lib/integration 두 archive는 test 건수를 거의 같게 나눴지만, 실제 PR run에서
lib archive build는 2분 54초, integration archive build는 11분 3초였다. lib archive는
test binary 하나와 약 27MB 산출물만 포함한 반면 integration archive는 41개 Cargo test target을
각각 링크하므로, runnable test 수만으로는 builder 비용이 균형을 이루지 못한다.

## 변경 설계

1. Archive A는 `--lib` 전용으로 유지하고 `hash:1/2`, `hash:2/2` worker 두 개가 소비한다.
2. metadata에서 정렬해 찾은 integration test target을 순번 교차 방식으로 B/C에 배정한다.
   현재 41개 target은 B 21개, C 20개가 되며 새 target도 자동으로 두 archive에 분산된다.
3. Archive B와 C는 각각 `hash:1/1` worker 하나가 전부 실행한다. 총 worker 수는 A1, A2,
   B1, C1의 네 개로 유지한다.
4. 집계는 `A1 + A2 = expected A`, `B1 = expected B`, `C1 = expected C`를 검증한다.

## 기대 효과와 검증 기준

- B/C builder는 별도 runner에서 병렬 compile/link되므로, Stage 2의 11분 3초 integration
  builder 임계 경로를 줄이는 것이 목표다. 공통 의존성은 두 runner에서 중복 컴파일하므로
  정확히 반으로 줄어드는 것을 보장하지는 않는다.
- 실제 PR CI에서 세 builder의 build 시간, archive 크기, 네 worker wall-clock과 전체 Build & Test
  완료 시간을 Stage 2 run과 비교한다.
- B/C의 runnable 합계와 A의 runnable 합계가 각 archive expected count와 정확히 일치해야 한다.
