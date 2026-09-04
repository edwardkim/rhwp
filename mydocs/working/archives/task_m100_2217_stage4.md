# Task M100 #2217 Stage 4 - 최종 로딩 상태 즉시 전환

## 배경

문서가 편집 가능한 상태가 된 뒤에도 상태 바가 `파일 로딩 100% - 완료`에 남아 보였다. 새 Chrome
세션 계측에서는 이 문구가 파일명으로 바뀌기까지 약 50.6ms였지만, 해당 단계 뒤에는 추가 작업이
없다. 따라서 최종 paint 대기는 사용자에게 진행 중이라는 인상만 남기고 실제 정보를 제공하지 않는다.

## 변경 방향

- 편집 핸들러를 활성화한 뒤 `100% - 완료` progress paint를 기다리지 않는다.
- 즉시 문서 파일명과 페이지 수를 상태 바에 표시한다.
- 로딩 순서 회귀 테스트가 최종 progress wait를 다시 넣지 않도록 고정한다.

## 완료 기준

- `20200830.hwp` 로드 시 96% 편집 상태 준비 후 상태 바가 곧바로 파일명으로 전환된다.
- 초기화 중 필요한 중간 progress paint와 로컬 글꼴 확인 대기는 유지된다.

## 구현 및 검증 결과

- `initializeDocument()`의 `updateLoadProgress(100, '완료')` 호출을 제거했다. 캐럿 연결이 끝나면
  상태 바에 파일명과 페이지 수를 즉시 쓴다.
- focused 검증: `node --test tests/document-initialization-order.test.ts tests/local-fonts.test.ts
  tests/document-font-status.test.ts tests/font-substitution.test.ts` - 21 passed.
- Studio production build: `npm run build` 성공. 기존 CanvasKit `fs`/`path` externalization 및
  chunk-size 경고 외 실패는 없었다.
- 실제 Chrome 별도 세션에서 저장된 local font snapshot과 `20200830.hwp`를 사용해 상태 변화를
  계측했다. `파일 로딩 96% - 편집 상태 초기화 중...` 다음 34.7ms에 파일명 상태가 나타났고,
  `파일 로딩 100% - 완료` 표시는 발생하지 않았다. `inputActive`와 textarea focus는 모두 true였고
  콘솔 오류도 없었다.
