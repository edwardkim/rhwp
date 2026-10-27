# 첫 줄 결합 강조점의 본문 clip 검사

#8038 원 head `20a7d2787ae531036ceceb88818504e87093cb7c` → 누적 `5370948158ce`를 확인했다.
현재 엔진은 독립 Print와 같은 결합점 U+0307을 SVG transform으로 배치한다.
예전 검사 helper는 검은 원 U+25CF만 찾아 무한대를 반환했다. 출력 자체의 회귀와 구분한다.

기존 `fixture`를 그대로 사용한 `raised-dot.hwpx`와 한컴 2020 Print method0·1-up PDF를
보존했다. 입력 전처리는 none이며 한 쪽이다. Native209/fresh056은 양쪽 raw 100%다.
실제 명령·생산 소스·입력/PDF·전체 TSV SHA는 [evidence.json](evidence.json)을 따른다.
원래 실행한 output 경로와 보존본은 같은 바이트다. 현재 표 여백 보정 후의 재출력 결과와
이 동결 생산물의 결과를 구분한다.

| 경로 | 전체 TSV | review | standalone overlay |
| --- | --- | --- | --- |
| Native | [TSV](native/silhouette.tsv) | [review](native/review_001.png) | [overlay](native/overlay_001.png) |
| fresh WASM | [TSV](wasm/silhouette.tsv) | [review](wasm/review_001.png) | [overlay](wasm/overlay_001.png) |

양쪽 대표 PNG에서 강조점의 위치·기호·본문과의 간격, 첫 줄의 글자와 공백을 직접 확인했다.
raw 100%와 엄격 RGB 일치를 구분하며 raster 농도 차이는 남는다.
검사는 결합점의 실제 transform을 적용한 em 상자 상단을 읽는다. 이는 실제 글꼴 잉크보다
보수적인 경계다. 본문 clip의 포함 관계, 위 밑줄 0·6·10의 획 포함과 clip 하단 보존,
강조점 적용 전후 캐럿 불변을 유지한다. 절대 px 위치나 허용치 완화로 대체하지 않았다.

같은 동결 Native209 library에서 옛 helper는 강조점 검출 실패(3324), 정정 후 두 장식 검사는
PASS(3325)였다. 같은 실행의 계산 빈 호스트 대조군 다섯 검사는 모두 PASS이고,
표 바깥여백 검사는 그대로 FAIL이다. 이 실패를 장식 검사 정정으로 숨기지 않았다.
전체 Rust·시각 범위 또는 PR 준비 완료를 뜻하지 않는다.
