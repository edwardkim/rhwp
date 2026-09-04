# Task M100 #1520 Stage 4 완료 기록

## 범위

- 최종 자동 검증을 실행했다.
- 수동 재현 확인 가능 범위를 기록했다.
- 오늘 할일 상태를 `검증완료`로 갱신했다.
- 최종 보고서를 작성했다.

## 자동 검증 결과

통과:

- `node --test rhwp-shared/sw/document-url-resolver.test.js`
- `node rhwp-chrome/sw/fetch-security.test.mjs`
- `node --test rhwp-shared/sw/download-interceptor-common.test.js`
- `node --check rhwp-chrome/content-script.js`
- `node --check rhwp-firefox/content-script.js`
- `node --check rhwp-safari/src/content-script.js`
- `npm run build` (`rhwp-chrome`)
- `npm run build` (`rhwp-firefox`)

초기 작업은 오래된 `local/devel` 기준에서 시작되어 `rhwp-chrome/sw/fetch-security.test.mjs`가 없었다. 이후 최신 `upstream/devel` 기준의 `local/task1520-upstream` 브랜치로 포팅했고, 해당 테스트를 포함해 다시 검증했다.

## 수동 검증 기록

이번 Stage 4에서는 브라우저 확장을 실제 로드한 GitHub 화면 수동 확인은 수행하지 않았다.

수동 확인 시나리오:

- `https://github.com/edwardkim/rhwp/blob/main/samples/2010-01-06.hwp`: 배지/hover 표시 유지
- `https://github.com/edwardkim/rhwp/edit/main/samples/2010-01-06.hwp`: 배지/hover 미표시
- `https://github.com/edwardkim/rhwp/commits/main/samples/2010-01-06.hwp`: 배지/hover 미표시
- `raw.githubusercontent.com` 직접 `.hwp/.hwpx` URL: rhwp 열기 유지
- 일반 직접 `.hwp/.hwpx` URL: rhwp 열기 유지

## 변경 상태

- 스테이징 전 상태로 유지했다.
- Chrome/Firefox 빌드 산출물은 추적 변경으로 남지 않았다.

## 다음 단계

- 변경 범위 승인 후 `git add`로 스테이징한다.
