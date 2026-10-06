# Task M100 #1520 Stage 3 완료 기록

## 범위

- Safari content script의 링크 후보 판정을 Chrome/Firefox와 동일하게 URL pathname 기반 분류로 변경했다.
- GitHub `blob` 문서 URL과 `raw.githubusercontent.com` 직접 문서 URL은 후보로 유지했다.
- GitHub `edit`, `commits`, `blame`, `tree` 라우팅 URL은 후보에서 제외했다.
- 명시적 `data-hwp="true"` 링크는 기존처럼 배지/미리보기 대상으로 유지했다.

## 검증

- `node --check rhwp-safari/src/content-script.js`
- `node --test rhwp-shared/sw/document-url-resolver.test.js`
- `npm run build` (`rhwp-chrome`)
- `npm run build` (`rhwp-firefox`)

결과: 모두 통과.

## 비고

- Safari 전체 `build.sh`/Xcode 빌드는 이번 구현계획서 범위 밖으로 두었다.
- Chrome/Firefox 빌드 산출물은 추적 변경으로 남지 않았다.

## 다음 단계

- Stage 4: 최종 diff와 테스트 결과를 정리하고 스테이징 전 변경 범위를 점검한다.
