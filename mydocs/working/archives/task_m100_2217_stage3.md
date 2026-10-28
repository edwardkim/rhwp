# Task M100 #2217 Stage 3 - 로컬 글꼴 확인 뒤 편집 활성화

## 배경

`20200830.hwp`를 Studio에서 연 직후 `initDoc` 로그는 완료되었지만, 로컬 글꼴 확인 모달 또는
CanvasKit의 비동기 local face 등록이 뒤이어 화면을 다시 그릴 수 있었다. 기존 순서는 이 비동기
단계보다 먼저 `InputHandler`를 활성화해, 사용자에게는 로드가 끝난 것처럼 보이지만 실제 입력 포커스와
캐럿 레이어가 뒤늦은 화면 재설정과 경합할 수 있었다.

별도 Chrome 세션에서 저장된 로컬 글꼴 결과가 있는 동일 파일은 표 셀 클릭, `EDITCHECK` 입력 및
Enter를 정상 처리했다. 따라서 문서 편집 엔진이 아니라 초기화 순서와 비동기 뷰 갱신을 보완 대상으로
한정한다.

## 변경 방향

- HWPX 검증과 로컬 글꼴 확인이 모두 끝난 뒤에만 `activateWithCaretPosition()`을 호출한다.
- local face 등록 완료 후에는 `CanvasView.loadDocument()`로 문서 뷰를 초기화하지 않고
  `document-view-changed` 이벤트로 현재 페이지를 다시 그린다.
- 초기화 순서와 재그리기 경로를 Studio source-contract 테스트로 고정한다.

## 완료 기준

- 로컬 글꼴 확인 모달이 열려 있는 동안 입력 핸들러는 비활성 상태다.
- 확인 완료 뒤 캐럿과 hidden textarea가 다시 활성화된다.
- CanvasKit local face 등록이 문서 스크롤·캐럿 상태를 초기화하지 않는다.

## 구현 및 검증 결과

- `initializeDocument()`는 HWPX 검증과 로컬 글꼴 확인을 마친 뒤에
  `activateWithCaretPosition()`을 호출하도록 순서를 바꿨다. 확인 모달이 열린 동안 상태 표시는
  `파일 로딩 94% - 문서 검증 및 글꼴 확인 중...`으로 남아 문서가 이미 편집 가능한 것처럼 보이지 않는다.
- CanvasKit local face 등록 완료 후에는 `document-view-changed` 이벤트를 보내 현재 페이지를 다시
  그린다. `loadDocument()`로 가상 스크롤과 canvas를 초기화하지 않으므로 현재 캐럿 위치를 보존한다.
- focused 검증: `node --test tests/document-initialization-order.test.ts tests/local-fonts.test.ts
  tests/document-font-status.test.ts tests/font-substitution.test.ts` - 21 passed.
- Studio production build: `npm run build` 성공. 기존 CanvasKit `fs`/`path` externalization 및
  chunk-size 경고 외 실패는 없었다.
- 실제 Chrome 별도 세션에서 저장된 로컬 글꼴 결과가 없는 `20200830.hwp`를 열었다. 모달 중
  `inputActive: false`를 확인했고, `대체 글꼴로 보기` 선택 뒤 textarea focus가 복원됐다. 표 셀에
  `EDITCHECK` 입력 후 Enter를 눌러 다음 셀 문단으로 이동했고 콘솔 오류는 없었다.
