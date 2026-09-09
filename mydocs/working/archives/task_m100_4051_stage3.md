# #4051 Stage 3 - 출력 생성 순서 보정 계획

## 목표

동일 stem 충돌이나 dry-run에서 output root를 만들지 않도록 검증과 쓰기 순서를 분리한다.

## 변경

- 생성자의 선행 output root 생성을 제거한다.
- `convert_batch`에서 최종 파일 집합의 충돌을 먼저 확인한다.
- 충돌이 없고 dry-run이 아닐 때만 output root를 생성한다.

## 완료 조건

- 동일 stem 충돌 테스트에서 rhwp 호출과 output root 생성이 모두 없음을 확인
- 기존 18개 batch-convert 통합 테스트 통과

## 테스트 결과

`CARGO_TARGET_DIR=target/review-kevin9327-4052-20260806 CARGO_INCREMENTAL=0 cargo test -p batch-convert`를
실행해 18개 통합 테스트가 모두 통과했다.

- 동일 stem HWP/HWPX는 rhwp mock 호출 0회와 output root 미생성으로 사전 거부됐다.
- PDF 성공 + PNG 실패는 Failed 1건과 exit 1로 집계됐다.
- rhwp exit 2는 `max_retries=3`이어도 호출 1회로 끝났다.

## Stage 4 계획

- native-skia 없는 실제 rhwp와 all-formats 설정으로 PNG feature-gate 실패가 batch exit 1로
  반영되는지 확인한다.
- 동일 stem HWP/HWPX를 mock rhwp로 실행해 output root를 만들지 않고 거부하는지 다시 확인한다.
- fmt, clippy, diff check를 수행하고 결과를 기록한다.
