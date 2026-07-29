# Task #2456 Stage 1 - HWPX DISTRIBUTE_SPACE 모델 보존과 렌더 정합

## 문제

- OWPML 스키마에서 `DISTRIBUTE_SPACE`는 나눔 정렬(공백에만 배분)이다.
- HWPX parser는 이를 `Alignment::Justify`로 정규화해 `Split` 모델 값이 재로드 때 유실된다.
- 단순히 `Split`으로 파싱하면 현재 renderer가 `Split`을 모든 글자 분배 경로로 처리하고, 머리말 단일 줄도
  마지막 줄이라 분배하지 않아 `SO-SUEOP.hwpx` p5 기준 계약을 깨뜨린다.

## 보정 방침

1. `DISTRIBUTE_SPACE`를 `Alignment::Split`으로 파싱한다.
2. `Split`은 스키마 의미에 맞게 `Justify`와 동일한 공백 전용 분배 경로를 사용한다.
3. 머리말/꼬리말 단일 줄은 기존 `Justify`와 같이 폭 전체에 공백을 분배한다.
4. 기존 `SO-SUEOP.hwpx` 회귀는 모델 값 `Split`과 페이지 5 머리말 폭을 함께 확인한다.

## 검증 계획

- parser 정렬 unit test
- `issue_1692_so_sueop_header_footer_page5_matches_reference_contract`
- full release-test integration, fmt, clippy, WASM build
