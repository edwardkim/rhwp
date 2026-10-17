# Native 공급 폰트 이름 계약 fixture

이 폰트는 레이아웃 정답지가 아닌 합성 이름 계약 입력이다.
`ttfs/opensource/NotoSansKR-Regular.ttf`에서 A만 subset하고 family와 PostScript 이름을
`Rhwp Family Contract` 계열로 바꿨다. 원본의 OFL은 `OFL.txt`에 보존했다.

Name table의 한국어 Family(1), Typographic Family(16), WWS Family(21)는 같은
프로그램의 가족 이름이다. 별도의 한국어 Full Face(4)는 가족 이름으로 수용하면 안 된다.
원본/산출 SHA와 각 이름은 `provenance.json`에 있다.

정식 검사는 `tests/cases/native_supplied_font_family_names.rs`다. 합성 A의
renderer-owned font decision을 관측하며 절대 px·저장 LineSeg·Hancom 문서 배치를 고정하지 않는다.
실물 문서의 모양·폭·페이지 검증은 이 계약 검사와 별도로 수행한다.
