# 안건표지의 표 배치와 〇 전진폭 대조

## 실제 차이와 독립 근거

PR #8563의 같은 입력 `samples/issue8560/21138757-agenda-cover-form.hwp`와 기존
`pdf/issue8560/21138757-agenda-cover-form-2020.pdf` 전1쪽을 제품 `048f44da99`로 대조했다.
Native raw 94.03%, fresh WASM raw 99.75%다. 원 주장의 네 표 배치·상하 순서·괘선은
대조 출력과 맞는다. 그러나 Native의 동그란 자리표시 문자가 가로로 좁게 그려지고,
그 뒤 조례 제목과 제출자 글자가 앞으로 당겨지는 차이를 직접 확인했다.

이 문자는 U+25CB ○가 아니라 **U+3007 〇 IDEOGRAPHIC NUMBER ZERO**다.
독립 Print는 실제 사용한 `/F3` Haansoft Batang 프로그램의 CID 0004를 U+3007에 연결하고,
그 전진폭은 1000/1000em이다. Unicode 17.0.0 EastAsianWidth도 U+3007을 W로 정의한다.
[Unicode 원문](https://www.unicode.org/Public/17.0.0/ucd/EastAsianWidth.txt)을 확인했다.
기준 PDF를 다시 출력하거나 글꼴 프로그램 바이트를 복사하지 않았다.

## 공통 생산·소비 경로

원본 기호 슬롯 한양신명조 HFT → 공통 char_width_decision의 메트릭 결측 →
CJK/전각 분류에서 U+3007 누락 → 반각 750HU(+장평·자간) → Native 글줄 전진과 glyph fit.
portable trace에서 실제 widthSource가 heuristicHalfwidth, 기준 전진이 750HU임을 확인했다.
WASM은 준비된 Canvas 폭으로 이를 보충하므로 두 backend 사이 차이가 나타난다.
문서별·Native 전용 숫자를 추가하지 않고 Unicode 문자 의미에 따라 공통 CJK 분류를 고친다.

실제 글꼴에서 얻은 메트릭 hit와 명시한 glyphFallback의 공급 계약은 기존 우선순위를 유지한다.
ASCII 0·반각형 문자·영문은 CJK 숫자 영과 구분한다. 기존 Native 글꼴 공급 계약의
`native_pua_host_font_protocol` 대조군도 실행한다. 구현 뒤 같은 전1쪽 Native/fresh WASM과
확대 판독을 완료한 뒤에만 해당 페이지의 새 조판 회귀를 등록한다.

## 현재 상태

R15에서 공통 U+3007 전각 분류를 보정했다. 현재 R11 전쪽 시각 배치는
보존된 producer로 계속한다. 필수 전체 검증·보호 PR #8389/#8397의 TSV 무하락 조건을 유지한다.

## 실제 보정 결과와 관계형 회귀

R15의 동일 원본·기존 독립 PDF 전1쪽 결과는 Native **99.74%**, fresh WASM **99.75%**다.
양쪽 review·overlay를 직접 판독해 네 표의 상하 순서·괘선, 조례 제목과 제출자 앞 숫자 영의
전진 및 뒤 본문 원점이 보존됨을 확인했다. WASM의 숫자 영 윤곽에는 PDF와 미세한 차이가
남아 있으므로 픽셀 동일성을 주장하지 않는다. 실제 px 좌표를 고정한 검사도 추가하지 않았다.

- 전쪽 TSV: [Native](r15/native.tsv) · [fresh WASM](r15/wasm.tsv).
- Native: [review](r15/native-review.png) · [overlay](r15/native-overlay.png).
- fresh WASM: [review](r15/wasm-review.png) · [overlay](r15/wasm-overlay.png).
- producer·소스 해시: [provenance](r15/provenance.json).

같은 원본의 공개 폭 추적 API에서 U+3007 11개를 확인한 전각 계약은 R11 FAIL(exit 1) /
R15 PASS(exit 0)다. 첫 잘못된 옵션 호출은 maxCharacters 상한 오류였으며 계약의 FAIL
증거로 세지 않았다. 기록은 ignored output의 `ideographic-zero-r11-contract.json`과
`ideographic-zero-r15-contract.json`이다.

전쪽 양쪽 90% 이상·직접 판독 뒤 새 관계형 검사 `pr_8563_ideographic_zero_fallback` 두 건을
추가했다. 미등록 가족과 내장 측정기를 사용해 세 글자 크기에서 전각 CJK와 같은 폭,
ASCII 숫자와의 구분, 반복 문자 뒤 본문의 경계·순서를 검사한다. 설치 글꼴·실물 px 핀에
의존하지 않는다. 최소 공개 글꼴 환경의 정식 suite_004 실행에서 두 건 모두 통과·exit 0이다.
전체 통합 회귀·lint·보호 문서 검증을 완료한 것으로 대신하지 않는다.
