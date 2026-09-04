# Task M100 #2217 Stage 2 - CanvasKit 로컬 Typeface 등록

## 목표

Stage 1에서 해석한 local record를 CanvasKit의 실제 SFNT Typeface와 연결한다. 브라우저 CSS가 아닌
CanvasKit backend에서도 `08서울한강체 M` 문서 text run이 설치된 `08SeoulHangang` face를 사용해야 한다.

## 범위

- 저장 snapshot에는 바이트를 넣지 않고, 현재 문서에서 쓰는 local record만 PostScript name으로 다시
  조회한다.
- 조회한 SFNT 바이트는 renderer 수명 동안 메모리에만 유지하고 같은 face의 중복 조회를 막는다.
- 일반 `Font` 생성과 Paragraph shaping 경로가 모두 local Typeface/FontMgr를 우선 사용한다.
- 접근 거부, 지원하지 않는 폰트 컨테이너, CanvasKit 등록 실패는 기존 Noto fallback을 유지한다.
- 첫 CanvasKit 렌더를 막지 않고, 저장된 감지 결과 또는 최초 감지 성공 뒤에 비동기로 준비한 다음
  등록된 face가 있을 때만 페이지를 다시 그린다.

## 비범위

- 임의의 시스템 글꼴 전체를 CanvasKit에 일괄 등록하지 않는다.
- 로컬 글꼴 원본 바이트를 localStorage, 문서 파일, 저장소에 보존하지 않는다.
- fallback glyph 단위 조합, 글꼴 라이선스 배포 정책, PDF fidelity는 별도 축이다.

## 완료 기준

- 한글 HWP font name으로 local record를 찾으면 CanvasKit이 해당 record의 bytes를 한 번만 등록한다.
- 실패 시 CanvasKit 전체 렌더가 중단되지 않고 기본 Noto fallback을 사용한다.
- resolver/byte registry와 renderer 선택 경로에 focused 회귀가 있다.

## 구현 및 검증 결과

- `loadLocalFontBytesFor()`가 문서 후보의 PostScript name들을 한 번에 조회하고, face별
  `ArrayBuffer` Promise를 현재 페이지 수명에만 cache한다. localStorage와 snapshot에는 바이트를
  기록하지 않는다.
- `CanvasKitLayerRenderer.prepareLocalFonts()`는 local Typeface/FontMgr를 PostScript/full name 기반
  face key별로 보관한다. 같은 CSS family의 `L`/`M` face가 서로 덮어쓰지 않으며, 일반 `Font` 및
  Paragraph shaping 경로가 모두 이를 우선한다. 등록 실패는 기본 Noto fallback으로 이어진다.
- `DocumentInfo.fontsUsed`는 실제 text run만이 아니라 HWP DocInfo의 선언 face table 전체를
  포함한다. 따라서 초기 문서 표시 전에 face를 하나씩 등록하지 않고, 첫 CanvasKit 렌더 후 비동기로
  일괄 준비하고 등록된 경우에만 다시 그리도록 조정했다.
- 실제 설치 파일 `08SeoulHangangM_0.ttf`(8,749,500 bytes)를 Node CanvasKit으로 확인했다.
  Typeface 생성 3.894ms, FontMgr 생성 3.209ms, family는 `08SeoulHangang`이었다.
- 브라우저에서 `?renderer=canvaskit`과 `20200830.hwp` URL 로드를 수행했다. 4페이지가
  `20200830.hwp — 4페이지 (126.5ms)`로 열렸고, local face 등록 뒤 CanvasView가 다시 렌더된 것을
  로그와 화면으로 확인했다.
- 이 검증은 local Typeface 등록 경로를 대상으로 한다. CanvasKit P16의 기존 조판 fidelity는 이 변경의
  완료 기준이 아니며 별도 렌더 backend 축으로 남는다.
- focused 검증: `node --test tests/local-fonts.test.ts tests/document-font-status.test.ts
  tests/font-substitution.test.ts` - 19 passed.
- Studio build: `npm run build` 성공. 기존 CanvasKit `fs`/`path` externalization 및 chunk-size
  경고 외 실패는 없었다.
