# 작업 2471 단계 1 - 각주 hit-test fast-reject 통합

## 배경

대형 문서에서 본문 클릭마다 각주 영역 hit-test가 render tree를 다시 구성해 불필요한 비용이 발생한다.
[#2471](https://github.com/edwardkim/rhwp/pull/2471)은 페이지네이션 결과의 각주 참조를 먼저 확인해,
각주가 없는 페이지에서는 해당 native 호출을 생략한다.

## 이번 스테이지 범위

1. #2471의 페이지 단위 `footnotes` fast-reject와 WASM/Studio 호출 경로를 통합한다.
2. 실제 `footnote-01.hwp` fixture로 각주 보유 페이지의 true, 각주가 없는 페이지와 범위 밖 페이지의 false를 고정한다.
3. Rust focused 회귀, formatter, clippy, 전체 회귀, WASM build 및 Studio package gate를 통합 head에서 확인한다.

기존 각주 내부 편집 hit-test의 결과·동작은 변경하지 않는다. fast-reject는 각주가 없는 페이지에서
`hitTestFootnote` render tree 빌드를 생략하는 데만 사용한다.
