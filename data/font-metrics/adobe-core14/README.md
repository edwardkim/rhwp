# Adobe Core 14 메트릭

Adobe의 [Core14_AFMs.zip](https://download.macromedia.com/pub/developer/opentype/tech-notes/Core14_AFMs.zip)에서
AFM 입력과 라이선스를 수정 없이 보존했다. 압축 파일의 SHA-256은
`8c892c3c49553cfd2d2a27c4495b4bb12e2875115be7fd127ed3876df19d8654`다.
저작권 표기와 `MustRead.html` 원문을 함께 보존하며 글꼴 outline은 포함하지 않는다.

`python3 scripts/generate_core14_metrics.py`는 AFM 문자 코드와 구분한 glyph 이름으로
공통 renderer의 ASCII 전진폭 표를 생성한다. 예를 들어 AFM 코드 39는 `quoteright`지만
Unicode U+0027은 `quotesingle`이다.
일곱 논리 항목은 Helvetica family의 스타일별 슬롯과 명시적 PostScript 이름을 다룬다.
기존 생성·실측 항목의 인덱스는 유지한다. 이 자료는 전진폭의 근거이며 paint backend에
해당 글꼴이 설치됐다는 증거는 아니다.
