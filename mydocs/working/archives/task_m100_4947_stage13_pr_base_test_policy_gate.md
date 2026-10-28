# task_m100_4947 stage 13: PR base 테스트 정책 게이트

## 목표

회귀 테스트 구조 지침을 문서 권고가 아니라 CI 계약으로 강제한다. 기여자가
generated manifest 또는 source unit 기준선을 함께 갱신하더라도 PR base 대비
정책 위반이면 CI가 실패해야 한다.

## 변경

- Git 명령을 shell 없이 실행해 PR base JSON과 rename 관계를 읽는 공통 도구를 추가했다.
- base에 없던 integration source는 `tests/cases/**` 밖에서 생성할 수 없게 했다.
- source unit manifest의 총량·모듈별 최대값·support 허용 목록을 PR base와 비교한다.
- `--accept-baseline`으로 현재 manifest를 갱신해도 base 대비 증가는 실패한다.
- Git rename으로 확인되는 생산 코드·테스트의 순수 crate 이동은 기존 module 기준과
  대응시켜 허용한다.
- pull request CI가 base SHA를 fetch하고 두 검사기에 `--base-ref`로 전달한다.
- 개발자 가이드에 로컬 기준선 갱신과 CI 승인 계약의 차이를 명시했다.

## 검증 기준

- `node --test scripts/tests/rust-test-suite-manifest.test.mjs`
- `node --test scripts/tests/rust-unit-test-tiers.test.mjs`
- `node scripts/rust-test-suite-manifest.mjs --check --base-ref upstream/devel`
- `node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel`
- CI workflow YAML·정책 계약 테스트

## 보안·운영 계약

Git ref는 `execFileSync` 인자 배열로 전달하며 shell interpolation을 사용하지 않는다.
push·workflow_dispatch에서는 기존 현재-tree 검사를 유지하고, pull request에서만
GitHub가 제공한 base SHA를 추가 비교한다.
두 manifest를 처음 도입하는 이 PR처럼 base에 정책 파일 자체가 없으면 current-tree
검사로 부트스트랩하고, 병합 이후 후속 PR부터 base 비교를 자동 적용한다.
