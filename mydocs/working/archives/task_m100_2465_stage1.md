# 작업 2465 단계 1 - G7 HWP5 serializer 보존 통합

## 배경

kevin9327의 HWP5 serializer 정보 보존 PR 세 건을 최신 `upstream/devel`과
검증 완료된 G6 HWPX 통합 head 위에 통합한다.

## 이번 스테이지 범위

1. [#2465](https://github.com/edwardkim/rhwp/pull/2465)의 Solid/Image 채우기 alpha 직렬화 보존을 통합한다.
2. [#2479](https://github.com/edwardkim/rhwp/pull/2479)의 그림 테두리 속성 워드 직렬화 보존을 통합한다.
3. [#2482](https://github.com/edwardkim/rhwp/pull/2482)의 각주와 미주 닫는 장식 문자 `0` 보존을 통합한다.
4. 통합 head에서 focused 회귀, fmt, clippy, 전체 회귀, WASM build를 확인한다.

각 변경은 HWP5 원본 또는 IR 재직렬화에서 사라지거나 변형되던 바이트를 원래 값으로 보존한다. 렌더링 레이아웃 경로는 변경하지 않는다.
