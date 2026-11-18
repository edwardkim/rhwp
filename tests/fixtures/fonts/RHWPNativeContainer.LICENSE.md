# Native 컨테이너 선택 검사 자원

`RHWPNativeContainer.ttf`와 `RHWPNativeContainer.ttc`는 이 폴더의
`RHWPEquationLatinOnly.ttf`에서 `A`와 공백만 추출한 SIL OFL 1.1 수정본이다.
원본은 Adobe의 Noto Sans KR이며 한컴 글꼴의 윤곽선을 사용하지 않는다.
두 파일의 name table에 원본 저작권·OFL 고지를 보존한다. 라이선스 전문은
[RHWPEquationFonts.OFL.txt](RHWPEquationFonts.OFL.txt), 원본 출처와 고지는
[RHWPEquationFonts.LICENSE.md](RHWPEquationFonts.LICENSE.md)를 따른다.

family·full name·PostScript name·고유 식별자는 `RHWP Native Container Probe`
계열로 바꾸며 Reserved Font Name인 `Source`를 사용하지 않는다.
저장소의 MIT 라이선스로 재허가하지 않는다. 재배포 시 이 설명과 OFL 전문을 함께 보존한다.

TTF의 `A`는 원본 윤곽·advance를 유지한다. TTC의 face 0은 별도 Control family이며,
face 1만 같은 요청 family를 선언하고 `A`의 윤곽 폭과 advance를 절반으로 줄인다.
이는 두 프로그램의 실제 paint를 구별하기 위한 합성 대조군이다.
한컴 출력이나 실제 문서 조판의 정답으로 사용하지 않는다.

생성기는 `scripts/generate-native-container-font-fixtures.py`다.
