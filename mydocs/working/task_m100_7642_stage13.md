# 이슈 #7642 — 곱셈 기호의 공통 잉크 영역

이전 단계: [명령 연산자 간격](task_m100_7642_stage12.md). 시작 head: `fd1856b78`.
원본 17쪽 Native는 96.95208%지만, 확대 판독에서 곱셈 기호가 한컴 Print보다 작다.

## 독립 근거와 원인

원본 Print가 사용하는 HYhwpEQ U+00D7과 설치된 같은 face의 전진폭은 831/1024em이다.
잉크 bbox는 (97,38,736,603)이며 가로 639/1024em, 세로 565/1024em이다.
현재 공개 Latin Modern Math의 곱셈 윤곽은 (148,9,630,491), UPM 1000으로
가로·세로 모두 482/1000em이다. 전진폭·여백을 맞춰도 실제 잉크 영역은 달랐다.
숫자 PUA glyph의 정규화 bbox는 공개 숫자와 가까워 ASCII Times 숫자 교체로 해결하지 않는다.
글꼴 경로·해시·bbox와 9/18pt Print 대조군은 기존 ignored 진단에 보존했다.

`Equation.font_name → source_font_symbol → 공통 glyph 원점·윤곽 →`
`Row 여백 → SVG/Canvas/Skia 및 paint JSON/Studio`를 연결한다.
문서 ID·화면 px로 위치를 보정하지 않는다. 원 face의 잉크 크기와 논리 전진폭을 구분하고,
공개 글꼴의 곱셈 윤곽을 이 잉크 영역에 배치한다. 공개 윤곽의 GUST 라이선스와 출처를
코드에 남기며 사설 글꼴 윤곽을 포함하지 않는다. 다른 face와 비해당 기호는 유지한다.

## 검증 계획

원본 17쪽, 독립 9/18pt 명령 연산자 대조군, 정상 eq-002 대조군을 먼저 재출력한다.
Native/fresh WASM 점수와 같은 배율 확대 판독을 별도로 확인한다. 이후 승인된 전체 입력과
관계형 회귀·필수 lint·전체 테스트·PR/CI/merge 후속 절차를 계속한다.
공통 윤곽은 `FontGlyph.outline`에 보존했다. Native SVG·Canvas·Skia는 같은 명령 벡터를
직접 재생하고, paint JSON은 이 벡터의 path를 Studio로 전달한다. Studio는 이미 있는
닫힌 윤곽 재생 경로로 표시하며 글꼴 파일이 필요한 prime은 기존 SVG 대체 경로를 유지한다.
row는 윤곽 glyph에도 원본 전진폭과 연산자 여백을 별도로 적용한다.

Native Clippy는 exit 0, 1분 55초이며 Studio `tsc --noEmit`도 exit 0이다.
Native 진단 빌드는 exit 0, 6분 1초이며 code head는 `960fe1fbf`다.
바이너리 SHA-256은 `8bb9528d319b5a295d56631506dcffad1606bb3a7d05f24ff105650683c5ac54`이고
파일별 소스 해시는 ignored `equation-multiplication-outline-native-provenance.json`에 있다.
원본 17쪽은 **96.95492%**, 독립 연산자 대조군은 **91.34684%**다.
같은 배율의 확대 비교와 fresh WASM의 최종 판정은 계속 진행한다.
Golden dumper는 추가 필드를 수용하는 패턴만 바꿨고 회귀 기대값은 갱신하지 않았다.

과거 비교에서 완료된 SVG 162개, 8.93GiB를 해시·경로 원장에 고정한 뒤 정리했다.
입력·Print PDF·PNG·TSV·JSON·로그·고정 바이너리·실행 중 산출물과 최저 쪽은 보존했다.
원장은 ignored `completed-svg-cleanup-stage13-20261008.json`이다.
