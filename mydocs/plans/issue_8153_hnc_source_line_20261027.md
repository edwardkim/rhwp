# #8153: HNC 선의 원 프로그램과 표시·전진 보존

두 편람 59쪽의 원 HNC 문자 `U+F080F`를 실제 공급 SFNT의 글리프로 출력한다.
같은 모양의 Unicode `U+2500`으로 바꾸어 원 프로그램의 윤곽·폭을 잃는 문제다.
문서·쪽 번호나 글꼴 이름으로 보정하지 않고 기존의 공통 PUA 공급 계약을 확장한다.

## 원본·기존 해결과 독립 근거

기존 `d98df211d9`·`774d414cd9`는 실제 표시 공급자가 읽은 SFNT의 cmap·hmtx가
확인될 때만 원 PUA 표시와 전진폭을 공유한다. `native_pua_host_font_protocol`의
공급·해제·원본 IR 불변 검사를 유지한다. 현재 지원한 여덟 기호에는 `U+F080F`가
없어 같은 프로그램이 이미 공급돼 있어도 이 글자는 Unicode 선으로 투영된다.

59쪽 원본 HWPX 글자 모양 206은 12pt·99%·자간 -2%이며, 실제 원문은
`U+F080F, 공백`이다. 독립 PDF의 Haansoft Batang subset gid 26은 설치된 원본
HBATANG의 `U+F080F`와 윤곽·힌트·hmtx 512/1024가 모두 같다.
`U+2500`은 다른 윤곽이며 hmtx 1024/1024다. 기존 HNC BMP 표기 `U+A80F`는
같은 정규화 키를 사용한다. 새 폰트 파일·이름 변경·윤곽 복사는 필요하지 않다.

근거는 ignored `remaining-raw100-diagnostic/small-residuals/`의
`p059-original-hnc-line-glyph-proof.json`, `actual-print-text/p059-actual-print-text.json`에 있다.
원본·기준 PDF는 변경하지 않는다. `명조` HFT의 기존 독립 원본 폭 계약도 보존한다.

원 HNC 선과 일반 Unicode 선의 99·100·101%, 400pt 대조 여섯 쪽을 한컴 2024에서
실제 저장하고 인쇄했다. 보정 전 HNC 대조군은 raw 0%, Unicode 대조군은 약 57%로,
이 자료는 진단용이며 회귀 fixture로 등록하지 않는다. Native/fresh WASM에서 실제
출력을 먼저 개선하고 시각 선행 조건·확대 판독을 확인한 뒤에만 정식 회귀를 추가한다.
작업 ID·입력/PDF SHA와 전후 결과는 `hnc-line-width-ratio-controls/`에 보존한다.

## 공통 생산·소비와 반례

`원본 기호의 bounded 슬롯 조회 → 실제 공급 후보의 SFNT cmap/hmtx →
NativePuaMetrics → char_width_decision/compose → display_text → 각 paint`가
같은 원 프로그램을 소비하게 한다. `SUPPORTED_SYMBOLS`에 원 기호를 추가하며
프로그램 부재·해제 시에는 기존 Unicode 대체를 유지한다. 일반 `U+2500`을
원 HNC 기호로 재분류하거나 실제 glyph가 없는 폰트를 공급했다고 가정하지 않는다.

장평·정렬·글꼴의 기존 결정은 별도 계약으로 유지한다. 한컴 Print의 장치 단위 변환은
PDF 연산·공개 GDI 측정으로 조사 중이며, 관측값 일부만 맞는 평균 폭 반올림 식을
제품에 넣지 않는다. 원 기호 선택을 장평의 임시 보정으로 대신하지 않는다.

## 검증과 완료 조건

기존 여덟 기호의 공급·해제와 일반 Unicode 대조군을 유지한다. 먼저 두 편람의
59쪽·다른 남은 기호 쪽·이미 완료한 경계를 실제 Native로 비교한다. 독립 여섯 쪽도
전후 출력과 source glyph 선택을 확인한다. 후퇴가 있으면 분기 조건을 좁히는 대신
원인을 수정한다. 이후 표준 최적화 fresh WASM·전쪽 TSV·직접 확대 판독·정식 검사를
완료한다. 새 검사에서 원본 IR·저장본 불변, 소유·내용 순서도 함께 보호한다.

직전 장평 후보의 집중 Native 42조건·독립 세 쪽·정식 nextest 16건은 통과했으며
전쪽 검증은 별도로 계속된다. #8153의 전 1,532조건 raw 100%·미대응 0 목표와
최종 검증을 축소하지 않는다. upstream 동기화와 원격 변경 없이 PR 준비까지만 진행한다.

## 첫 Native 후보의 실제 결과

`f721556830`은 두 편람 집중 42조건에서 미대응 증가가 없었다. 59쪽은 HWP 8→4,
HWPX 19→15픽셀로 줄었다. 독립 원 HNC 선의 99·100·101% 세 쪽은 모두
raw 100%·미대응 0이다. 별도 Unicode 선 세 쪽의 기존 약 57% 결과는 같으며,
그 미달 대조군을 완료 또는 신규 fixture로 등록하지 않는다.

공통 공급 프로토콜의 output 제안 검사는 보정 전 7 PASS·2 FAIL, 보정 후 9 PASS다.
실제 fresh WASM 시각 검증과 정식 등록·최종 회귀는 남아 있으며 완료로 표시하지 않는다.
Native 전쪽 검증은 `hnc-source-line/native-whole-driver.log`에서 별도로 계속된다.
