# 작업 2505 단계 1 - Safari 빌드 위생 정리

## 범위

- [#2505](https://github.com/edwardkim/rhwp/pull/2505)를 통합해 Safari local build directory를 ignore한다.
- [#2507](https://github.com/edwardkim/rhwp/pull/2507)을 통합해 Safari content script의 참조되지
  않는 `escapeHtml` helper를 제거한다.
- [#2502](https://github.com/edwardkim/rhwp/pull/2502)은 cherry-pick하지 않는다. 유일한
  `MAX_FILE_SIZE` 삭제는 commit `7304b385a`의 Safari HML gate 보정에 이미 포함됐다.

## 검증 계획

1. 새 ignore rule을 Chrome, Firefox extension directory와 비교한다.
2. `escapeHtml`의 남은 Safari call site가 없는지 확인하고 JavaScript 문법을 검사한다.
3. 확장을 설치하거나 publish하지 않고 영향받는 extension build를 실행한다.

## 결과

- `rhwp-safari/.gitignore`는 `node_modules/`, `dist/`로 Chrome, Firefox extension rule과
  일치한다.
- `escapeHtml`에는 Safari call site가 없었고 link detection이나 HML 처리를 바꾸지 않고 제거됐다.
- 문법 검사와 공유 document signature 회귀 테스트가 통과했다. Safari `dist`는 성공적으로 다시
  생성됐고 signed Xcode build만 로컬 Mac development certificate 때문에 막혀 있다.
