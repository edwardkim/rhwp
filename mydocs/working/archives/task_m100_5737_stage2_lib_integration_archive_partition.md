# #5737 Stage 2 - lib/integration archive 분할 분석

## 목표

초기 archive 이름 짝홀수 분할을, 각 builder가 자신의 Cargo test target만
컴파일하면서 임계 경로도 균형 있게 유지하는 방식으로 대체한다.

## Stage 1 실측 근거

- 이름 짝홀수 분할 결과는 archive A 1,965건, archive B 5,860건이었다.
- `rhwp` library test binary 하나에만 3,893건이 있어, integration target 이름을
  짝홀수 bucket으로 나누는 방식으로는 균형을 맞출 수 없다.
- 두 archive 산출물 크기는 각각 216,427,704 bytes와 243,509,661 bytes였다.

## 확인된 구조

1. Archive A는 `cargo nextest ... --lib`만 선택한다.
2. Archive B는 파생 suite 준비 뒤 `cargo metadata --no-deps --format-version 1`으로
   찾은 root package의 모든 integration `--test <name>` target만 선택한다.
3. 각 archive는 `hash:1/2` worker 두 개를 두고, aggregate가 archive별 runnable
   total을 각각 검증한다.

## Stage 2 실측 결과

- lib archive: 3,893 runnable tests, 25,511,342 bytes
- integration archive: 3,920 runnable tests, 317,779,426 bytes
- 두 target group의 합계는 7,813 runnable tests이며, 분할 오차는 27건이다.

## 수용 기준

- 로컬 archive 목록이 실측 lib 3,893건, integration 3,920건 수준으로 균형을 이룬다.
- 재사용 builder는 어느 partition에서도 `--tests`를 쓰지 않아, 반대 partition의
  test target을 컴파일하지 않는다.
- workflow 계약 테스트가 target 탐색, archive label, 두 builder, 네 worker를 검증한다.
- PR CI에서 단일 archive 기준선과 비교해 builder 시간, artifact 크기, worker wall-clock,
  runner-minute 변화를 기록한 뒤 채택 여부를 결정한다.
