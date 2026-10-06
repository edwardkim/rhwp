# Task M100 #2217 Stage 1 - 로컬 글꼴 이름 해석 착수

## 목표

macOS Chrome이 반환한 영문 family와 HWP의 한글 full name/style 조합을 같은 설치 글꼴로 판정하도록
로컬 글꼴 메타데이터와 resolver 경계를 정리한다.

## 시작 근거

- `20200830.hwp`는 `08서울한강체 M`을 사용한다.
- 설치 TTF는 `08SeoulHangang`, `08서울한강체`, `08SeoulHangang M`, `08서울한강체 M`,
  `SeoulHangangM` 이름을 함께 가진다.
- 현재 `detectLocalFonts()`는 `FontData.family`만 snapshot에 저장한다.
- 문서 글꼴 상태와 CSS chain은 저장된 family의 정확 문자열 일치만 확인한다.

## 이번 Stage 범위

- 재현 파일을 `samples/issue2217/`에 보존한다.
- snapshot v2와 이름 resolver를 구현하고 v1 마이그레이션을 검증한다.
- 문서 상태와 CSS chain을 resolver 기반으로 전환한다.
- CanvasKit의 실제 SFNT 등록은 다음 Stage에서 별도로 처리한다.

## 완료 기준

- 08서울한강체의 한글/영문/full name/PostScript 별칭이 하나의 local record로 해석된다.
- HWP의 한글 글꼴명이 local available로 판정되고 canonical CSS family를 사용한다.
- 기존 snapshot v1과 Local Font Access API 미지원 fallback 회귀가 없다.

## 구현 및 검증 결과

- `samples/issue2217/20200830.hwp`를 원본 SHA-256
  `8e7a95cf591944bff56050879fa90251921ec57e28eac66d40c6fb8ad103016f` 그대로 보존했다.
- Local Font Access API의 `family`를 CSS canonical family로 유지하면서, SFNT `name` table의
  family/full name/PostScript name/style을 별칭 record로 저장하는 snapshot v2를 구현했다.
- 한글 full name `08서울한강체 M`은 `08SeoulHangang` CSS family로 해석한다. 글꼴 목록에는
  사용자가 식별하기 쉬운 한글 full name을 표시한다.
- 브라우저에서 로컬 글꼴 재감지를 실제로 수행했다. 기존 272개에서 670개를 저장했고, 목록에
  `08서울한강체 L`, `08서울한강체 M`가 표시되는 것을 확인했다.
- focused 검증: `node --test tests/local-fonts.test.ts tests/document-font-status.test.ts
  tests/font-substitution.test.ts` - 18 passed.
- Studio build: `npm run build` 성공. 기존 CanvasKit externalization/chunk-size 경고 외 실패는 없었다.

## 다음 Stage로 넘기는 항목

Canvas2D/CSS 경로는 canonical local family를 사용할 수 있게 되었지만, CanvasKit은 아직 기본 Noto
typeface만 사용한다. 다음 Stage에서 필요한 글꼴만 현재 렌더링 세션 메모리에 읽어 CanvasKit Typeface로
등록하고, 실패 시 기존 fallback을 유지한다.
