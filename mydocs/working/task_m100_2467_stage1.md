# 작업 2467 단계 1 - G8 HWPX-HWP 컨테이너 재귀 통합

## 배경

HWPX에서 HWP5로 변환할 때 일부 컨테이너 내부 문단이 adapter 또는
border-fill 참조 수집 경로에서 누락되어, 개체 보강과 채우기 참조가 저장 과정에서
사라질 수 있었다.

## 이번 스테이지 범위

1. [#2467](https://github.com/edwardkim/rhwp/pull/2467)의 각주·미주·숨은설명·표 캡션 내부 개체 adapter 재귀를 통합한다.
2. [#2483](https://github.com/edwardkim/rhwp/pull/2483)의 바탕쪽·각주·미주·숨은설명·머리말·꼬리말·표/도형 캡션 내부 border-fill 참조 수집을 통합한다.
3. #2483 원 PR의 rustfmt 불일치를 보정하고, focused 회귀, fmt, clippy, 전체 회귀, WASM build를 통합 head에서 확인한다.

두 변경은 HWPX to HWP adapter의 서로 다른 재귀 워크를 대칭으로 맞춘다. 렌더러와 레이아웃 규칙은 바꾸지 않는다.
