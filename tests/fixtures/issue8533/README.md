# #8533 Print 출력 계약 fixture

`handbook-p140-print-frame.png`는 독립 한컴 Print PDF의 140쪽을 canonical PDFium
96 DPI로 변환한 뒤 `[166, 320, 592, 625]` 영역만 무보간 절단한 그림이다.
입력·PDF·원 페이지 PNG·영역 PNG의 SHA-256과 보정 전 일치율은
`handbook-print-frame.json`에 기록했다. 페이지나 PDF를 수정하지 않았다.

`RHWPPrintFallbackContext.ttf`는 저장소의 소유 합성 글꼴
`tests/fixtures/fonts/RHWPHostFixture-Regular.ttf`에 U+0628 cmap 항목만 추가했다.
기존 `가` glyph를 가리키며 새 글리프 윤곽·폭·이름·라이선스를 추출하거나 추가하지 않았다.
실제 아랍어 자형의 기준이 아니며, 한 글자여도 문맥 shaping 대상으로 분류되는 경우
기존 painter를 유지하는지를 검사한다. FontTools로 Unicode cmap마다
`cmap[0x0628] = cmap[0xAC00]`를 설정한 파일이다.
