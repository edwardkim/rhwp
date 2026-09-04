# 작업 2506 단계 1 - Chrome manifest 버전 단일 소스

## 범위

- [#2506](https://github.com/edwardkim/rhwp/pull/2506)을 통합한다. content script에서 Chrome
  manifest의 extension version을 읽어 page-context DevTools helper에 DOM attribute로 전달한다.
- Firefox와 같은 순서를 보존한다. 즉 `dev-tools-inject.js`를 주입하기 전에 version attribute를
  설정한다.

## 검증 계획

1. Chrome source에 중복된 extension version literal이 더는 없는지 확인한다.
2. content script가 page-context helper 주입 전에 DOM attribute를 설정하는지 확인한다.
3. 확장을 설치하거나 publish하지 않고 JavaScript 문법 검사와 로컬 Chrome production build를
   실행한다.

## 보정

- 기여자 helper는 처음에 `data-rhwp-extension-version`을 읽었지만, Chrome content script,
  Firefox 구현, extension build guide는 `data-hwp-extension-version`을 사용한다.
- 기존 `data-hwp-extension-version` contract를 유지하고 producer/consumer 쌍과 주입 순서를
  검증하는 source-level 회귀 테스트를 추가한다.

## 결과

- Chrome은 이제 `chrome.runtime.getManifest().version`에서 version을 읽고 기존 DOM attribute를
  page-context DevTools helper에 전달한다.
- 회귀 테스트는 동적 source, producer/consumer attribute 일치, helper 주입 전 순서를 검증한다.
- focused Chrome 테스트 모음 15개가 통과했고, 확장을 publish하거나 설치하지 않고 로컬 production
  build를 완료했다.
