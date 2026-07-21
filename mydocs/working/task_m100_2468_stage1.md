# 작업 2468 단계 1 - G9 HWP3 계약 및 속성 보존 통합

## 배경

HWP3의 공통 편집 경로, 글자 속성 매핑, 문서 번호 시작값에서 각각 직접 버전 비교나
누락된 IR 매핑이 남아 있었다.

## 이번 스테이지 범위

1. [#2468](https://github.com/edwardkim/rhwp/pull/2468)의 native HWP3 판정을 `layout_profile()` 계약 질의로 통일한다.
2. [#2469](https://github.com/edwardkim/rhwp/pull/2469)의 HWP3 위첨자·아래첨자 IR 매핑을 통합한다.
3. [#2486](https://github.com/edwardkim/rhwp/pull/2486)의 HWP3 쪽·각주 시작 번호 IR 매핑을 통합한다.
4. #2468 원 PR의 rustfmt 불일치를 보정하고, focused 회귀, fmt, clippy, 전체 회귀, WASM build를 통합 head에서 확인한다.

렌더러를 직접 바꾸지 않고, HWP3 parser와 공통 편집 레이어가 이미 소비하는 계약 값을 정확히 제공한다.
