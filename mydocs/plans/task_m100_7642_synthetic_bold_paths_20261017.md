# #7642 합성 굵기 글리프의 원 윤곽선과 사용자 좌표 획

Issue: #7642.

## 독립 근거와 반례

법령 Print 50쪽은 실제 regular 휴먼명조·휴먼고딕 프로그램을 `2 Tr`로
채우고 윤곽선을 그린다. 글꼴 프로그램의 원 윤곽선·전진폭 일치는 기존
`stage2715-legal-p50-program-identity/`에서 확인했다. 획은 0.02em이며
획 맞춤 SA는 꺼져 있다. 조인과 miter는 PDF 기본값이다.

Native의 같은 프로그램 공급 대조군과 원 캡처는 픽셀이 완전히 같다.
글리프 윤곽선·hmtx를 보존한 채 힌트 프로그램만 제거한 로컬 진단은
raw 99.55288%에서 99.60977%로 달라졌고 굵은 표 제목·본문에도 변화가 있다.
즉 원 프로그램이 같더라도 장치 격자 보정의 소비가 같은 것은 아니다.
이는 원본 글꼴을 바꿀 근거가 아니며 진단 글꼴을 실제 공급 계약으로 쓰지 않는다.

Miter 10과 4로 각각 인쇄 PDF를 복제한 진단은 원 PDF와 픽셀이 완전히
같았다. 이 페이지의 하락을 miter 차이로 설명하지 않는다.
RHWP direct PDF의 PDFium 재출력도 하락이 남으므로 그것을 해결 증거로
사용하지 않는다. 모든 진단은 ignored output의 stage2791~2795에 보존한다.

## 실제 소비와 수정

현재 regular 프로그램 합성 굵기는 같은 TextRun 원점에서 Font의 일반
글자 그리기에 0.02em stroke를 얹는다. 독립 PDF의 stroked text는 글리프
윤곽선을 사용자 좌표에서 채우고 그리는 경로다. 새 공통 경로는 기존
합성 굵기 분기에서 실제 선택한 같은 font/glyph의 path를 사용한다.

- 원 프로그램을 다시 열어 시스템 가상 굵기를 제거하는 계약을 유지한다.
- 실제 bold 프로그램과 명시 outline pass에는 이 분기를 적용하지 않는다.
- 장평·원본 폭·기울임을 같은 Font 기하로 적용하고, 글리프 path를 원래
  TextRun 기준선 원점에 놓는다. 획의 사용자 좌표 폭은 유지한다.
- path를 얻을 수 없는 프로그램은 기존 그리기를 유지한다. 모든 path를
  확인한 뒤 그려 부분 출력 뒤 fallback으로 중복하지 않는다.
- 일반 글자·수식·GlyphReplay 등 다른 계약을 임의로 바꾸지 않는다.
  shaping span은 기존 별도 소비 경로이며 이번 cluster 경로의 검증으로
  그 경로까지 충족했다고 보고하지 않는다.

Skia의 `getPaths`는 path용 폰트를 만들 때 힌팅을 해제하며 원 크기로
되돌린 윤곽선을 반환한다. PDFium의 stroked text도 glyph path를 소비한다.
확인한 upstream main 코드는 구현 근거이며 설치된 PDFium binary와 같은
소스 revision임을 입증한 자료는 아니다.

- [Skia Font path 소비](https://skia.googlesource.com/skia/+/refs/heads/main/src/core/SkFont.cpp)
- [PDFium text path 소비](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/core/fxge/cfx_renderdevice.cpp)
- [PDFium glyph path의 힌팅 조건](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/core/fxge/cfx_face.cpp)

## 검증

줄 무효화 보정과 함께 한 source에서 새 Native/fresh WASM을 만들고 법령
전체 74쪽·굵은 글자의 확대 판독·같은 조건의 base 대조를 수행한다. 전체
102개 문서와 #7969·#7970 보존·필수 Rust/lint/UI 게이트를 유지한다.
이 코드의 개선 여부는 아직 미검증이다. 점수·기준 PDF·baseline을 바꾸지 않는다.
