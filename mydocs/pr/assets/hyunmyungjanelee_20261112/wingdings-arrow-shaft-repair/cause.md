# U+F0FA의 표시 기호와 원본 전진 보존

U+F0FA 자체는 사설 사용 영역이며 일반 Unicode 문자의 고정된 의미가 없다. 이번 원본
`samples/issue8680/wingdings_pua_symbols.hwpx`에서는 Wingdings 코드 FA를 뜻한다.
원문은 그대로 보존하고 표시할 때만 대응하는 U+1F8AD로 투영한다.
[Unicode 정본](https://www.unicode.org/charts/PDF/U1F800.pdf)은 이를
`WHITE ARROW SHAFT WIDTH TWO THIRDS`로 명명한다. 일반 전각 빈 상자 U+25A1로 치환하지 않는다.

기준은 웹한글기안기 PDF가 아닌 독립 데스크톱 한컴 2020 11.0.0.9136 Print
`pdf/issue8680/wingdings_pua_symbols-hwpx-2020.pdf`다. 이 PDF의 Wingdings-Regular
CID9는 원 코드 FA와 대응하고 /W385를 가진다. 내장 글리프와 Mac 원본 Wingdings의
`bdash2`는 좌표·윤곽 끝점·플래그가 동일하다. 원본 글꼴 및 분석용 추출 파일은 Git에 넣지 않는다.

원인은 표시 매핑에 FA가 빠져 있고, 원 프로그램의 전진폭도 없었던 것이다. 실제 Studio
CanvasKit 후보들의 글리프 누락 교집합이 U+F0FA였으며 `textRun:glyphMapping`으로 거절했다.
사용자 요청으로 이 기호도 이번 메인터너 보정 범위에 포함했다.

기존 그림자 원형 기호의 공통 런 분리 경로를 확장해, 일대일 기호 투영에만 이미 배포 중인
Noto Sans Symbols2를 공급한다. 주변 본문·원문 주소·셀 소유와 원문 PUA를 유지한다.
이 공개 글꼴의 U+1F8AD 전진은 356/1000이므로 원문의 385/1000 전진을 별도로 보존하며,
기존 4HU 장치 양자화를 계속 적용한다. 이를 제거해 이상적인 부동소수점 폭으로 바꾸지 않는다.

Rust 정책과 Studio nominal replay는 단독 U+1F8AD를 허용하지만 변형 선택자·ZWJ·결합 문자는
계속 shaping 대상으로 분류한다. 글리프 결측 가드나 CanvasKit 준비 문턱을 우회하지 않는다.
공개 글꼴과 원 Wingdings는 같은 의미의 서로 다른 글리프 프로그램이며 윤곽 완전 일치를 주장하지 않는다.

집중 검사·실제 재출력 결과는 이 폴더의 검증 기록에 연결한다. 보정 전 증거와 새 소스 검증을 구분한다.
