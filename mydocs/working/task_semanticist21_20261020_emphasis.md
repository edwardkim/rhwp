# 강조점의 독립 문자·글꼴·배치 계약

## 구현 전 근거와 범위

#8038의 첫 줄 장식 clip 변경을 한컴 Print와 대조했다. 기존 painter는 강조점을0.3em 크기의
대체 기호로 그린다. 원본의 Print는1~4에 U+0307/U+030A/U+030C/U+0303,5~6에
U+302E/U+302F를 각 글자의 크기로 그린다. 혼합 크기 대조군은 Native039에서71.17595%다.
위 밑줄의 줄 최대 크기와 구분하며 문서명·쪽·특정 텍스트로 분기하지 않는다.

유효한 공개 API 저장본 `emphasis-font-controls-149`의 Arial·Times New Roman·맑은 고딕·
함초롬돋움 본문에서도 강조점의 실제 프로그램은 HCRBatang이다. 장평·굵기 대조군의 Print는
해당 속성을 유지한다. 새 `emphasis-style-controls-168`의 기울임·자소 조합은 같은 방식으로
PrintMethod0으로 출력한다. PDF 텍스트 행렬의 기울임은0.333, 위 네 기호의 기준선 이동은
약0.37em이다. 원점은 글자 전진의 절반에 약0.225em을 더한 위치이며 장평을 함께 적용한다.
방점은 각 글자 왼쪽에 원래 글리프를 놓는다. 공백·개체 자리표시자에는 표시하지 않는다.

브라우저의 U+302E/U+302F 단독 셰이핑은 U+25CC 보조 원을 삽입한다. 실제 HCRBatang
프로그램의 Canvas probe로 확인했다. 잉크가 없는 초성·중성 채움 문자를 문맥으로 주면 원을
삽입하지 않는다. 채움 문자의0.5em 전진만 상쇄하며 원문·저장 문자열은 바꾸지 않는다.
원래 glyph를 직접 그리는 Native/CanvasKit 경로에는 이 브라우저용 문맥을 넣지 않는다.

## 생산과 소비 계획

공통 geometry는 원본 표시 자소·저장 전진·글줄의 표시 방향에서 강조점 소속·원점·기준선·
가로 배율·기울임과 실제 글꼴 역할을 정한다. Native·SVG·Canvas2D·Paint JSON/CanvasKit은
같은 결과를 소비한다. clip은 같은 강조점의 보수적 em 경계를 포함하되 글줄·캐럿 크기를
변경하지 않는다. host 글꼴 요청과 SVG 부분 임베딩에도 실제 강조점 face를 포함한다.

Native039와 fresh WASM014는 수정 전 제품이다. 기존6종·4크기·혼합 줄·4본문 글꼴과 새
속성 반례를 독립 Print로 다시 대조한다. 전쪽 양쪽90%와 직접 판독 전에는 정식 회귀나 새
fixture를 승격하지 않는다. 이번 작업과 #8064의 정상 출력을 함께 유지한다.

## Native040 단계 검증

새 공통 geometry를 Native·SVG·Canvas2D·Paint JSON/CanvasKit과 clip/paint bounds에 연결했다.
CanvasKit host 요청과 SVG 부분 임베딩에도 강조점의 실제 face를 포함했다. 기울임은 실제
Print 행렬처럼 glyph에 적용하며 원점이나 글줄 전진을 이동하지 않는다. 방점의 브라우저용
채움 문자는 일반 원문이나 Native의 직접 glyph 그리기에 삽입하지 않는다.

Native040의 첫 빌드는 없는 BoundingBox 메서드를 호출한 컴파일 오류3건으로 실패했다.
필드 기반 경계 계산으로 수정한 두 번째 빌드는 exit0,252초다. TypeScript와 Studio 전체
검사도2,354 PASS /0 FAIL /2 SKIP이다. 로그는 ignored output에만 보존한다.

`emphasis-175-emphasis-fixed-native`의64조건 중63조건이 전쪽90% 이상이다. 수정 전 강조점
30조건은3통과·27미달이었고 같은 조건의 수정 후는29통과·1미달이다.6종 기호,4크기,
혼합 크기,4본문 글꼴은 모두100%다. 굵기100%, 기울임99.02946%, 굵기+기울임99.24555%,
새 장평50/150%의6종 입력은96.76856/91.88265%다. 점수는2px raw이고 RGB 동일 주장이 아니다.
기존 위 밑줄·위상31조건과 #8064의 국어 두 형식20+20쪽·HWPML122쪽도 수치 통과했다.

실패한 `emphasis-style-controls-168/graphemes.hwpx`는87.99371%다. `emphasis-native040-panels-178`
의 review/overlay를 직접 확인했다. 강조점과 공백 소속은 개선됐지만 본문의 옛한글이 다르다.
원본 U+F53A는 표시용 자모 U+1112/U+119E/U+11AB로 확장됐고, Native가 그 문자를 따로
그리는 경로를 추가 추적한다. 실제 공급 HCRBatang Regular는 Print와 같은 프로그램
SHA256 `8664669bd2d6a4fe8b7538706d2bb2a1004c074aa02dca59051e515e731294f5`다.
파일 부재·잘못된 fontFamily setter 문제로 오인하지 않는다. 새 대조군을 삭제하거나
기준 PDF를 바꿔 통과로 처리하지 않는다.

혼합 크기·Arial 본문·기울임의 새 review와 standalone overlay도 직접 판독했다. 본문 기울임과
장평의 남은 가장자리 차이는 강조점의 개선과 구분하며 추가 판독한다. 실제 Print10입력·180개
강조점 span·글꼴과 텍스트 행렬은 `emphasis-independent-print-174.json`에 보존했다.

이 단계는 Native 공통 강조점 보정의 진행 상태를 고정하는 commit이며 전체 자격의 완료가
아니다. 같은 소스의 fresh WASM015를 빌드·검증하고 옛한글 출력의 공통 원인을 보정한다.
미달 입력의 정식 회귀·fixture는 양쪽90%·직접 판독 전까지 추가하지 않는다.
