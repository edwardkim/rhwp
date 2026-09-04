# 작업 2489 단계 1 - 문서·기여자 메타데이터 정합

## 범위

- [#2489](https://github.com/edwardkim/rhwp/pull/2489)의 README, 영문 README, 기여 가이드에
  기록된 Rust 테스트 수를 현재 release-test 목록과 대조한다.
- [#2494](https://github.com/edwardkim/rhwp/pull/2494)의 영문 변경 이력에 `Unreleased` 섹션을
  추가해 한국어 변경 이력과 구조를 맞춘다.
- [#2508](https://github.com/edwardkim/rhwp/pull/2508)의 `.mailmap` 기여자 별칭 통합을 적용해
  shortlog 통계가 같은 기여자를 분리하지 않도록 한다.

## 검증 계획

1. `cargo test --profile release-test --tests -- --list` 결과로 문서의 `5,500+` 주장을 확인한다.
2. Markdown 변경의 위치와 한국어·영문 변경 이력 구조를 대조한다.
3. `.mailmap` 적용 전후의 `git shortlog --mailmap` 결과에서 cskwork와 lpaiu-cs 별칭이 각각
   하나의 정체성으로 통합되는지 확인한다.

## 경계

- `#2489`의 수치가 현재 목록과 다르면 문서 숫자를 임의로 유지하지 않고 실제 결과에 맞춰
  메인터너 보정한다.
- 패키지 배포, GitHub 릴리스, 원격 push는 수행하지 않는다.

## 결과

- `cargo test --profile release-test --tests -- --list`는 `3,405`개 테스트를 열거했다. 따라서
  `#2489`의 `5,500+` 표기는 적용하지 않고, README·영문 README·기여 가이드 모두에서 검증 가능한
  `3,400+`로 보정했다.
- 한국어와 영문 변경 이력 모두 최상단에 `Unreleased` 섹션이 있다.
- `.mailmap` 적용 뒤 `git check-mailmap`은 cskwork와 lpaiu-cs의 과거 이름·이메일을 각각 하나의
  canonical GitHub noreply 정체성으로 반환한다.
