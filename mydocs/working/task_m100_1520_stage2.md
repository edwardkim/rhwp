# Task M100 #1520 Stage 2 완료 기록

## 범위

- Chrome content script의 링크 후보 판정 로직을 `anchor.href` 전체 정규식에서 URL pathname 기반 분류로 변경했다.
- Firefox content script에도 동일한 후보 판정 로직을 적용했다.
- GitHub `blob` 문서 URL은 후보로 유지하고, `edit`, `commits`, `blame`, `tree` 라우팅 URL은 후보에서 제외한다.
- 명시적 `data-hwp="true"` 링크는 기존처럼 배지 대상으로 유지한다.

## 검증

- `node --check rhwp-chrome/content-script.js`
- `node --check rhwp-firefox/content-script.js`
- `node --test rhwp-shared/sw/document-url-resolver.test.js`

결과: 모두 통과.

## 다음 단계

- Stage 3: Safari content script에 동일한 후보 판정 로직을 적용하고 브라우저 확장 빌드/검사를 실행한다.
