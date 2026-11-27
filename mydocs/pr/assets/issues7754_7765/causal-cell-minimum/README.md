# 중첩 표의 선언 최소 공간: 독립 Print 인과 실험

원본은 `samples/issue1891/86712_regulatory_analysis.hwpx`다. 원본3280HU와
선언 최소 높이0·2000·2400·4280HU만 바꾼 네 사본을 한컴2024 Hancom PDF Print로
인쇄했다. ZIP 모든 member를 비교했고 section XML의 `cellSz` 속성 한 곳만 달랐다.
`provenance.json`에 원본·사본·Print SHA-256, 단일 변경과 실제 변환 job·종료 시각을 기록했다.

26쪽 마지막 중첩 행의 PDF 하단은 각각1023.36003·1028.95467·1034.22803·1041.26131px다.
원본3280HU도1041.26131px다. 최소 상자가 작으면 실제 내용이 끝을 정하고, 커지면 빈 공간을
추가하며, 쪽 경계에서는 남은 공간까지만 같은 내용 컷이 점유한다. 실제 재측정 기하도 보존했다.

이 실험의 PDF는 `pdf/issue7765/causal-cell-minimum/`에 있다. 사본은 같은 바이트의
`.hwpx.bin`으로 보존하여 활성 fixture/golden·자동 corpus에 등록하지 않는다. 사본의
전쪽 피델리티를 통과로 주장하지 않으며, 원본의 전쪽 Native/fresh WASM과 실제 배치 회귀는
별도 source32 근거를 따른다. 글꼴 프로그램이나 인증 정보는 포함하지 않았다.
