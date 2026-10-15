# Native 수식 글꼴 소비 검증 자원

테스트 fixture도 공개 GitHub 저장소에 포함하면 재배포다. 테스트 목적이나 제품 번들 제외는
저작권·라이선스 조건의 예외가 아니며, 아래 출처와 OFL 고지를 함께 보존해야 한다.

기존 `ttfs/opensource/NotoSansKR-Regular.ttf`와 `NotoSansKR-ExtraLight.ttf`에서 글자를 일부
추출하고 이름을 바꾼 SIL OFL 1.1 수정본이다. 한컴 폰트에서 윤곽선을 복사한 파일이 아니다.
저작권자는 Adobe이며 원본의 저작권 고지를 각 파일의 name table에 보존한다.
라이선스 전문은 같은 폴더의 [RHWPEquationFonts.OFL.txt](RHWPEquationFonts.OFL.txt)에
동봉한다. 원본 고지는 [NotoSansKR-OFL.txt](../../../ttfs/opensource/NotoSansKR-OFL.txt)다.
이 폰트 수정본에도 OFL이 적용되며 저장소의 MIT 라이선스로 재허가하지 않는다.

| 파일 | 원본과 보존 범위 | 검사 목적 |
| --- | --- | --- |
| `RHWPEquationCJKRegular.ttf` | Noto Sans KR Regular의 한글·라틴·수식 문자 일부 | 공급한 Regular 윤곽선을 사용하는 양성 대조군 |
| `RHWPEquationCJKLight.ttf` | Noto Sans KR ExtraLight의 같은 문자 범위와 가는 윤곽선 | 같은 family의 공급 파일을 바꾸면 수식 paint가 달라지는지 확인 |
| `RHWPEquationLatinOnly.ttf` | Noto Sans KR Regular의 ASCII | 첫 후보에 한글 glyph가 없으면 CJK fallback을 사용하는지 확인 |

세 파일의 family/typographic family는 모두 `RHWP Equation Contract`다. full name·PostScript
name·고유 식별자도 `RHWP Equation Contract` 계열로 지정하며 원본 저작권·OFL 고지는 유지한다.
문서가 요청하는 `Haansoft Batang`은 테스트에서 `with_font_family_alias`로 공급한
`RHWP Equation Contract`에 명시적으로 연결한다. 실제 폰트의 내부 이름을 상용 글꼴 이름으로
바꾸지 않으며 기본 폴백에는 이 테스트 family를 추가하지 않는다. 한컴 글꼴의 윤곽선·배포 허가나
동일한 출력이라는 증거로 사용하지 않는다.

ExtraLight 원본은 가는 윤곽선을 독립 family로 공급하기 위해 weight class가 400으로 설정되어
있다. 이 검사는 weight class를 200으로 가정하지 않고 Regular와 실제 윤곽·먹물량을 비교한다.
원본 cmap·윤곽선에서 독립 기대값을 정하고 실제 `eq-01.hwp`의 수식 op를 통해 검사한다.
전체 페이지 일치와 실제 모양은 동일 원문의 한컴 PDF와 별도로 확인한다.

생성 명령은 `python3 scripts/generate-equation-font-fixtures.py`다. 재배포할 때는 세 폰트에
이 설명과 OFL 전문·저작권 고지를 함께 보존하고, 원본의 Reserved Font Name 조건을 따른다.
