# #8153 결측 화살표의 실제 공급자 전진폭

대상은 기존 두 편람의 101쪽이다. HWP5 사양 표 33·34의 언어별 글꼴 참조,
장평·자간과 원본 `charPr 206`을 유지한다. 임의 이동량을 더하지 않는다.

## 독립 근거와 기존 경로

원본은 KoPub바탕체 Light, 크기 1200HU, 장평 99%, 자간 -2%로 `⇨`를 쓴다.
독립 한컴 2024 Print는 이 글자만 Haansoft Batang으로 출력한다. 같은 줄의
첫 두 화살표는 다음 공백까지 15.52px를 전진한다. 현재 Native의 전진은
14.45333px이며, 첫 화살표의 시작점은 Print보다 약 1.61px 오른쪽이다.

설치된 열두 KoPub/KoPubWorld 프로그램의 Arrows U+2190..U+21FF 전체를 읽었다.
모두 cmap coverage가 `0x1400000000000003ff`이고 U+21E8은 없다. HBATANG의 같은
112개 문자는 모두 존재하며 hmtx 전진은 각각 1024/1024em이다. 원 프로그램 SHA는
기존 metadata와 같다. 글꼴 파일을 복사·변조·커밋하지 않는다.

기존 `090fa51cc2`와 `e40f3a2a08`은 확인한 cmap 결측에 한해 선언 paint 체인의
실제 공급자로 측정하는 공통 경로를 만들었다. 현재 coverage는 CJK 구두점과
U+2024만 포함하여 화살표의 실제 결측이 이 경로에 도달하지 않는다. 한글용
KoPub 전진 추정이 남고, 공급자 DB에도 Arrows 구간이 없어 실제 hmtx를 못 읽는다.

## 생산·측정·출력 연결과 보정 범위

- 생산: 실제 SFNT cmap·hmtx에서 확인한 coverage와 전진만 기존 metadata에 보충한다.
- 측정: `verified_named_program_has_glyph → char_width_decision`의 기존 선언 공급자
  탐색이 `FontMetric::get_width`의 원 프로그램 값을 소비한다. 원본 장평·자간과
  장치 단위 반올림, 양쪽 정렬은 기존 공통 계산을 그대로 쓴다.
- 출력: Native `font_lookup::select_typeface_for_character → text_replay::fonts::for_text`와
  Canvas의 선언 family 체인은 이미 같은 공급자를 사용한다. paint 선택 규칙과
  font-rule registry를 바꾸지 않으며, Native/WASM에 별도 위치 보정을 추가하지 않는다.
- 정상 반례: 실제 KoPub에 있는 `→`는 기존 원 글리프와 900/1000em 계약을 유지한다.
  미확인 face·기울임·HFT·다른 측정 face를 실제 결측으로 추정하지 않는다.

먼저 현재 producer의 계약 실패를 확인한다. 별도 소형 입력에서 결측 `⇨`와 실제 hit
`→`, 장평 99/100/101%의 한컴 정규화·Print를 확보한다. 그 뒤 기존 공통 경로의
metadata 보충을 적용하고 영향 쪽·정상 대조군 및 전쪽 Native/fresh WASM을 대조한다.
정식 회귀 등록은 양쪽 시각 선행 조건을 충족한 뒤 `tests/cases/issue_8153_*`에서 한다.

현재 첨자 producer `262c660b6e`의 전쪽 검증은 보존한다. 전쪽 100%·미대응 0과
최종 전체 검증은 아직 완료하지 않았다. upstream 동기화와 원격 GitHub 쓰기를 하지
않으며, 전체 목표 달성 후 PR 준비까지만 완료한다.
