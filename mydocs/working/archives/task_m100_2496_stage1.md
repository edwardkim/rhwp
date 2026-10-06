# 작업 2496 단계 1 - npm editor package metadata 통합

## 범위

- [#2496](https://github.com/edwardkim/rhwp/pull/2496)을 통합한다. `node:test` 기반 package test
  suite에 Node.js 18 이상을 선언하고 package file 목록에 `README.md`를 넣는다.
- [#2504](https://github.com/edwardkim/rhwp/pull/2504)를 통합한다. package funding link를 추가하고
  명시적인 README package-file 선언을 유지한다.
- [#2503](https://github.com/edwardkim/rhwp/pull/2503)은 별도 cherry-pick하지 않는다. funding만
  바꾸는 변경이 #2504에 완전히 포함돼 있다.

## 검증 계획

1. `package.json`을 검증하고 editor package 테스트를 실행한다.
2. publish 없이 `npm pack --dry-run`으로 배포 package manifest와 README 내용을 확인한다.

## 결과

- 통합된 manifest는 `engines.node: >=18.0.0`을 선언하고 `README.md`를 명시적으로 열거하며
  프로젝트 funding page를 연결한다.
- `npm test`는 editor package 테스트 18개를 모두 통과했다.
- `npm pack --dry-run`은 tarball을 publish하지 않고 README와 예상한 package file 다섯 개를 포함한다.
