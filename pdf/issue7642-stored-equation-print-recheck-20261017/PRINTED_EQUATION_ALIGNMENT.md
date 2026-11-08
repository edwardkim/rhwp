# 본문 가운데 정렬 분수의 독립 Print 기준

Issue: #7642. 기존 #5583 합성 정렬 검사의 입력·기대값 보정 증거다.

## 원본과 재생성 대조군

기존 `tests/cases/issue_5583_body_equation_alignment.rs`의 합성 입력은
commit `d019cd23d3a335b1dc1cc38d348aaefdd1af2ad2`에서 확인할 수 있다.
용지 0×0, 수식 face/version 빈 문자열, 임의의 수식 크기 19094×4333 HU였다.
원본 HWPX와 정상 저장 HWP 모두 용지가 0×0여서 Print가 실패했다.
원본 진단 자료와 실패 기록은 ignored output의 stage2269·2273·2288에 보존한다.
정상화 대조군의 통과를 원본 입력의 시각 일치로 보고하지 않는다.

`calibrated-equation-cache-{0,6000}.hwpx`는 별도 재생성 대조군이다.
한컴 저장 HWP에서 확인한 수식 face/version·1376×2340 HU·기준선 67%와
LineSeg 메트릭을 명시하고, 테스트가 가정하던 A4 본문을 실제 용지에 설정했다.

- 용지: 59528×84188 HU, 좌우/상하 여백 5670 HU, 머리말/꼬리말 0 HU.
- 수식: `1 over 2`, HYhwpEQ, Equation Version 60, baseUnit 1000 HU,
  크기 1376×2340 HU, baseLine 67, treatAsChar=1.
- 줄: vertpos=0, vertsize/textheight=2340, baseline=1568, spacing=-1000,
  horzsize=48188, horzpos는 원래 두 검사처럼 0/6000, flags=0.
- 문단 가운데 정렬과 원본 수식 내용은 유지했다.

6000 대조군은 한컴이 정상 발행한 저장 줄이라고 주장하지 않는다. nonzero
horzpos와 전체 본문 너비 horzsize를 수동으로 함께 넣은 경계다. 독립 Print는
두 입력을 같은 가운데에 배치한다. 따라서 양수 horzpos만으로 비가운데 배치를
요구하던 기존 기대값이 독립 출력과 다르다. 이 결과를 정상 HWP5 저장 줄이나
#1256/#1308의 유효한 저장 좌표를 무시하는 근거로 일반화하지 않는다.

## 실제 Print 출처

두 HWPX를 각각 MCP `engine=2020`, 한컴 `11.0.0.9136`,
`pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`으로
PDF에 직접 Print했다. 파일 형식 변환이나 PDF SaveAs 출력이 아니다. 각각 1쪽이다.
비공개 endpoint와 인증 정보는 기록하지 않는다.

| 입력 | 입력 SHA-256 | PDF SHA-256 | Print job |
| --- | --- | --- | --- |
| `calibrated-equation-cache-0.hwpx` | `02da5c70a21ec255dd16ae8835a8aeb62356133356c41e60bee0b41af6c7a284` | `84bd4643f5dfaba9e217934b1f779ce1628a79b71c32dbd4adfdf290b552156f` | `58d84e90-217d-4b95-bef1-cda1bc7b1345` |
| `calibrated-equation-cache-6000.hwpx` | `9fdc20c3b987f85854a1e86dee326a782fee409f94323ce12dd0444f7151dd79` | `d6b2c334cb526f4be9d6cc66ea9124ee68769a948520352ea469951ad940c980` | `df133b35-043b-42d0-99c0-a0e0c36b6021` |

## 시각 검증과 독립 좌표

code source `cc4255a58d8112c07013e89eda1b1a065c824fea`, 96dpi/Cairo,
기본 인쇄 프로필/compat 2022, 시스템 폰트와 assets/fonts를 사용했다.
stage2291 Native·stage2292 fresh WASM에서 두 문서의 전체 1쪽은 모두 raw
100%이고 Print와 쪽수가 일치한다. WASM은 stage2258에서 새로 빌드한 고정
패키지를 사용했다. TSV·로그·중간 JSON은 ignored output에만 보존한다.

경로 접두어:
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`

- Native: `stage2291-calibrated-equation-native/calibrated-{0,6000}/silhouette.tsv`
- WASM: `stage2292-calibrated-equation-wasm/calibrated-{0,6000}/silhouette.tsv`
- 전체 및 같은 영역 review·standalone overlay: `stage2294-calibrated-equation-review/`.
- PDF 원점 추출: `stage2295-calibrated-equation-pdf-coordinates.json`.

직접 확대 판독에서 숫자 1/2의 모양·위아래 순서와 가운데 배치를 확인했다.
분수선 끝과 숫자 2의 래스터 경계에 작은 차이가 남는다. raw 100%는 2px 이웃
실루엣 통과이며 RGB 동일이라는 뜻이 아니다. 엄격 내용 픽셀 보조값은 Native
63.75%, fresh WASM 71.25%로 별도 기록한다.

독립 PDF의 CSS 96dpi 원점은 두 입력 모두 다음과 같다. 잉크 bbox가 아닌
PDF 글리프 baseline origin을 검사하며, layout 글리프 원점과 같은 양을 대조한다.

| 원본 내용 | HYhwpEQ 글리프 | x | baseline y |
| --- | --- | ---: | ---: |
| 분자 1 | U+E034 | 390.82817 | 86.94499 |
| 분모 2 | U+E035 | 390.82817 | 103.88646 |

## 정식 검사 계약

새 이름 `verified_printed_body_equation_alignment.rs`는 두 입력의 전체 쪽수와
수식 소유·내용 보존, 본문 내 포함, 가운데의 양쪽 여백 관계를 검사한다.
위 완료 페이지의 독립 숫자 원점을 ±2px로 추가 보호한다. 좌표나 고정 화면
영역으로 검사 대상을 찾지 않는다. 수식 script·원본 제어 소속으로 식별한다.
기존 무효 입력에서 실패하던 비가운데 기대를 현재 출력값으로 복사하지 않았다.
한컴 Print와 대조한 결과가 교정의 근거다.

이 입력은 실제 시험지 원본이 아니라 명시적으로 재생성한 합성 경계다.
실물 문서·다른 저장 줄 경로·전체 브랜치 검증 완료를 대신하지 않는다.

## 정식 Rust 실행 확인

source `60e35888c79c130b0d50b230a66e2cda67618255`에서 새 검사 2개가 모두 통과했다.
`stage2297-printed-alignment-nextest.log`의 2 tests run/2 passed와 exit 0을
확인했다. 빌드 21.31초, 실제 검사 0.023초다. 공용 target/pr-review에서
논리 기호 before/after가 끝난 뒤 순차 실행했고 build jobs 8/test threads 12를
사용했다. 이 결과는 전체 브랜치의 lint·회귀·시각 gate 통과를 대신하지 않는다.
