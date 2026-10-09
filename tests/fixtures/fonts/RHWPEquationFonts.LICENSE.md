# Native 수식 글꼴 소비 검증 자원

기존 `ttfs/opensource/NotoSansKR-Regular.ttf`와 `NotoSansKR-ExtraLight.ttf`에서 생성한
SIL OFL 1.1 subset이다. 원문 라이선스는 [NotoSansKR-OFL.txt](../../../ttfs/opensource/NotoSansKR-OFL.txt)다.
생성 명령은 `python3 scripts/generate-equation-font-fixtures.py`다.

- `RHWPEquationCJKLight.ttf`: 실제 200 weight 윤곽선을 보존하고 family를 `Noto Sans KR`로 지정했다.
  같은 family를 가진 사용자 자원의 교체가 수식 paint에 반영되는지 Regular 400 대조군과 비교한다.
- `RHWPEquationLatinOnly.ttf`: Regular의 ASCII만 보존하고 family를 `Batang`으로 지정했다.
  수식의 첫 후보에 한글 glyph가 없을 때 실제 CJK fallback의 윤곽선을 소비해야 한다.

두 자원은 선택 계약용이며 한컴 기본 글꼴을 재정의하는 제품 번들이 아니다. 원본 cmap·윤곽선과
weight에서 독립 기대값을 정하고 실제 `eq-01.hwp`의 수식 op를 통해 검사한다.
전체 페이지 일치와 실제 모양은 동일 원문의 한컴 PDF와 별도로 확인한다.
