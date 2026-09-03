# Task M100 #6672 Stage 2

## 목적

“더 공격적으로 정리할 수 있다”는 이슈 피드백을 반영해 paragraph composer에 한정한
범위를 renderer 전체 callable로 확대한다.

## 판정

native, wasm32, native-skia product lib의 `-W dead-code` 결과를 교집합으로 계산했다.
target 한 곳에서만 죽은 항목은 다른 renderer가 쓰는 구현일 수 있으므로 제외했다.
함수·메서드만 대상으로 삼고 dormant field/type/constant는 이번 이슈에서 제외했다.

## 구현

- unit/integration test가 직접 검증하는 56개 callable을 `#[cfg(test)]`로 제품
  graph에서 제외했다.
- 저장소 전체에서 호출자도 없는 43개 callable은 삭제했다.
- module별 commit으로 equation, flow/pagination, layout, table metric, kerning,
  shaping accessor 경계를 분리했다.
- test-only 전환 뒤 native, wasm32, native-skia를 다시 컴파일했고 모두 통과했다.

## 결과

세 제품 구성의 renderer `dead_code` 교집합에는 field/type/constant 34건만 남고,
함수·메서드 진단은 0건이다. 총 99개 callable과 1,265 source lines가 제품 graph에서
정리됐다. 최종 `origin/devel` 기준 release-test 전체 suite와 native-skia,
WASM, Studio renderer gate를 모두 통과했다.
