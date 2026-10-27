# #8153 Native Print의 글리프 윤곽 재생 정밀도

## 독립 근거와 원인 계층

두 편람 79쪽의 U+3000은 HWP5 표 7의 NBSP·고정폭 공백 제어와 다른 실제 문자다.
원본 KoPub Dotum Light의 cmap·glyf·hmtx와 독립 Print PDF의 부분 집합 glyph를
직접 대조했으며, 같은 작은 사각형 윤곽과 전진 폭을 갖는다. 원본 문자·글줄 원점·
크기·장평을 유지한 진단에서 Native `draw_str`의 raster cache는 RGB 232,
동일하게 선택한 `Font::get_path`의 벡터 재생은 RGB 211이었다. 독립 Print의
해당 잉크는 RGB 230·219이고 고정된 실루엣 검사에서 전자는 미대응 2개,
후자는 해당 작은 영역에서 0개다. 이는 전쪽 통과 근거가 아닌 원인 분리 결과다.

같은 원본 프로그램과 좌표의 Chromium Canvas `fillText`·원본 path 재생은 모두
RGB 211이었다. Native의 hinting, linear metrics, embedded bitmap, edging,
subpixel 설정을 바꾼 대조에서는 이 차이가 해소되지 않았다. 따라서 공통 줄 구성이나
장평 값을 보정하지 않고 Native Print의 최종 glyph cache 소비 경계를 다룬다.
글꼴 파일의 복사·수정, 임계값·반경·원본·기준 PDF 변경은 하지 않는다.

## 구현 계약과 범위

`TextStyle → 실제 cmap fallback으로 선택한 Font → 기존 cluster 원점·장평·fit scale
→ Print glyph paint`를 따라간다. Native의 기존 geometric Print 경로에서 독립적인
단일 문자 glyph는 실제 선택한 Font의 path로 재생한다. 문서·쪽·글꼴 이름·문자 값이나
작은 크기 임계값에 따른 조건을 두지 않는다. 그림자·윤곽선은 기존 paint와 원점을
그대로 소비하며 장식선·전진 폭·소유·내용 순서를 변경하지 않는다.

Native의 glyph cache와 Chromium의 텍스트 재생은 실제 저수준 출력 계약이 다르다.
Canvas의 검증된 재생을 별도 glyph 공급 registry로 대체하지 않고, Native가 이미
선택한 프로그램의 윤곽을 소비하도록 한다. HFT의 기존 CFF 계약과 합성 굵기의 path
재생은 유지한다. 여러 scalar를 가진 cluster의 shaping, path가 없는 bitmap·색상 glyph,
Screen 경로는 기존 재생을 유지하며 일반 glyph의 성공을 이 경로들의 증거로 쓰지 않는다.

## 검증

우선 Native 집중 46조건과 독립 장평 99/100/101% 대조를 실행한다. 원본 두 문서의
79쪽은 독립 Print와 전체 페이지를 대조하고 실제 PNG를 확대 판독한다. 다른 문자·
크기·장평·앞뒤 내용에 후퇴가 있으면 원인부터 재검토한다. 설치 글꼴에 의존하지 않는
MIT 합성 글꼴의 가는 실제 glyph로 수정 전 실패·수정 후 통과를 확인한다.

최종 후보에서는 두 문서 전쪽 Native·표준 최적화 fresh WASM TSV와 내용·소유 보존을
다시 확인한다. 목표는 1,532조건 전쪽 raw 100%·정수 미대응 0이며 집중 진단으로
완료를 대신하지 않는다. 필수 정식 회귀·lint 등도 완료한 뒤 PR 준비까지만 진행한다.
upstream 동기화와 원격 쓰기는 하지 않는다.
