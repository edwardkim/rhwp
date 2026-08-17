# #5164 stage 3: Native Skia preflight 직후 병렬 실행

## 목표

Native Skia 검증이 lint와 frontend gate의 완료를 기다리면서 CI 임계 경로가 길어지는 문제를 제거한다.

## 실측 근거

- PR #5170 CI run `32001574354`에서 Native Skia job은 8분 24초가 걸렸다.
- 해당 job은 preflight가 끝난 뒤 바로 실행되지 않고 lint·frontend 완료까지 약 5분 26초를 기다렸다.
- Rust cache는 약 627MB가 완전 적중했으므로 이번 병목은 cache miss가 아니다.
- 최종 `Build & Test` job은 lint, frontend, Native Skia 결과를 모두 입력으로 받아 각 lane의 success·skip 계약을 이미 검증한다.

## 변경

- `native-skia-tests.needs`를 `preflight` 하나로 제한한다.
- 실행 조건은 preflight 성공, fast pass 비활성, `native_skia_required=true`만 사용한다.
- lint·frontend·Native Skia의 조합 검증은 기존 최종 `Build & Test` 집계 job에 유지한다.
- 정책 테스트에서 Native Skia가 lint·frontend에 다시 직렬화되지 않도록 고정한다.

## 영향과 경계

- lint 또는 frontend가 실패하더라도 이미 시작된 Native Skia runner 비용은 발생할 수 있다.
- 대신 정상 PR의 전체 대기 시간은 Native Skia와 다른 gate가 겹치는 만큼 단축된다.
- 디스크 정리 조건화와 Native Skia Cargo 호출 통합은 별도 stage에서 다룬다.

## 완료 조건

- Native Skia job이 preflight 직후 lint·frontend와 병렬로 시작한다.
- Native Skia가 필요하지 않은 변경과 fast pass에서는 기존처럼 skip된다.
- 최종 `Build & Test`가 모든 lane 결과를 계속 판정한다.
