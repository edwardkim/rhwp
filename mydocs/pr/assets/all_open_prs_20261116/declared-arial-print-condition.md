# Helvetica 문서의 독립 Print 재현 글꼴 조건

이 기록은 `control-styles-hwp.hwp` 보호 조건의 복원이며 제품의 전역 글꼴 대체 규칙 변경이 아니다.
원본은 Helvetica 계열 4개 요청에 Arial 대체를 선언하고, 공식 Print PDF에는
ArialMT·Arial-BoldMT·Arial-ItalicMT·Arial-BoldItalicMT가 포함되어 있다.

기존 Linux 보호 실행은 Arial prior 경로를 사용했다. Mac에서 Helvetica query를 그대로
사용한 첫 실행은 OS Helvetica를 선택해 base와 후보 모두 99.25722%였다.
Arial 프로그램을 명시하면서 query를 Helvetica로 남긴 첫 진단도 원 요청이 앞에 들어가므로
같은 99.26%였다. 이 결과를 이번 제품 회귀로 처리하거나 낮아진 보호 기준으로 수용하지 않는다.

원본이 선언한 Arial을 query와 family에 명시하고 네 스타일의 실제 프로그램 ID를 각각
연결한 결과 base Native·후보 Native·fresh WASM이 모두 100.00%다.
동일 쪽 review와 overlay에서 일반·굵게·기울임·굵은 기울임의 네 줄을 직접 판독했다.
재현 설정은 [환경 JSON](declared-arial-print-environment.json), 이미지는
[Native review](frame-boundary-validation/declared-arial/native/review_001.png)·
[overlay](frame-boundary-validation/declared-arial/native/overlay_001.png),
[WASM review](frame-boundary-validation/declared-arial/wasm/review_001.png)·
[overlay](frame-boundary-validation/declared-arial/wasm/overlay_001.png)에 보존했다.

공급한 Windows Arial 원본은 과거 보호 실행 파일과 SHA가 같다고 주장하지 않는다.
실제 사용한 각 스타일의 10문자, 총 40문자의 UPEM·분해된 윤곽·hmtx advance가
PDF 내장 TTF와 모두 같은지는 [글리프별 대조](arial-pdf-active-glyph-comparison.json)에서 확인했다.
글꼴 바이너리나 윤곽 배열은 Git에 포함하지 않는다. 프로그램 ID는 원본 파일 SHA와
collection index를 함께 해시한 값이므로 원본 파일 SHA와 혼동하지 않는다.
