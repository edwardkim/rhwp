# 작업 2464 단계 1 - G6 HWPX 왕복 통합 품질 게이트 보정

## 배경

kevin9327의 HWPX 왕복 보존 PR 7건을 최신 `upstream/devel` 위에 체리픽했다.
원 PR [#2464](https://github.com/edwardkim/rhwp/pull/2464)와
[#2484](https://github.com/edwardkim/rhwp/pull/2484)는 기능 테스트는 통과했지만 각각
`cargo fmt --check`, `clippy::needless_update` 때문에 CI lint가 실패했다.

## 이번 스테이지 범위

1. #2464의 `section.rs` 서식을 rustfmt 결과로 맞춘다.
2. #2484의 `LineShape`에서 모든 필드를 이미 지정한 뒤 남은 `..Default::default()`를 제거한다.
3. focused 회귀, fmt, clippy, 전체 회귀, WASM build를 통합 head에서 확인한다.

기능 동작이나 원 PR의 parser/serializer 의도는 변경하지 않는다.
