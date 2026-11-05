# PDF family 보정의 Native·fresh WASM 전쪽 측정

이 기록은 새 base `7fdd5441ff` 반영 뒤 제품 코드 `4b149b6cdf`의 Native와
빌드 head `7bb9e66fd0`의 최적화 fresh WASM 전46쪽 재측정이다. 원본 PDF와 공개 HWPX를
바꾸지 않고 두 경로 모두 누락·쪽수 불일치·90% 미달 0쪽을 확인했다. 최종 통합 공통 검증은 남아 있다.

96 DPI, Print 2024, 전체 시스템 글꼴을 공급했다. Native 실행 파일 SHA-256은
`71a6e065770d741971ad618e614a2d0bc1759cf2eefd4ce9b72ad395032f8d8f`다.

| 판본 | 입력 SHA-256 | 기준 PDF SHA-256 | 전체 쪽수 · 최저 raw | 전쪽 Native TSV |
| --- | --- | --- | --- | --- |
| 2010 | `54df47aa980966679a67f579916972d26d6147a08d327368398abc2552d84412` | `609d56421009ccab5a50cb2a8f4e78af4d0961de2e7ae85d811905e8434ac251` | 16쪽 · 97.80% | [TSV](postscript-2010-native.tsv) |
| 2020 | `f1929a65e3901b90087fe9ccb4b0e446ceed32926b1479d4307e22474d23f49e` | `a1a8b0ffd6ae9b23627f7b1bf58bbdfdc0714a369713cc057aa836c75b143f1e` | 15쪽 · 99.19% | [TSV](postscript-2020-native.tsv) |
| plain | `5a13ddd9b7fe2a51c95009e34c8e0a27337d35f145c637a3193025e9c455b1c6` | `6bc54661a73af788d437463e000f76652b1325b44607e58443e7391d23e2db83` | 15쪽 · 98.54% | [TSV](postscript-plain-native.tsv) |

2010·2020판 HWPX는 이전 제출본과 ZIP 전체 바이트가 같다. 무표기판만 header·section의
family 및 그 메트릭으로 계산한 자간·비율이 달라졌다. 독립 PDF는 재산출하거나 수정하지 않았다.
무표기판 이전 12쪽 미달은 새 실제 API 저장본의 전15쪽에서 해소했다.

| 원 주장 영역 | 현재 Native raw | 실제 같은 쪽의 review · overlay |
| --- | --- | --- |
| 공백 줄 뒤 본문 · 5쪽 | 99.67471% | [review](postscript-plain-p005-native-review.png) · [overlay](postscript-plain-p005-native-overlay.png) |
| 공백 줄 뒤 본문 · 13쪽 | 99.67735% | [review](postscript-plain-p013-native-review.png) · [overlay](postscript-plain-p013-native-overlay.png) |

5쪽과 13쪽의 본문 줄 시작·줄바꿈·상자 및 앞뒤 간격을 같은 독립 PDF와 직접 대조했다.
원 주장인 공백 줄 뒤 본문과 문항·보기의 소속, 누락·중복 없이 이어지는 내용을 확인했다.
13쪽 쪽번호의 글리프 외형 차이는 남아 있으므로 페이지 전체의 글리프가 같다고 주장하지 않는다.
이 보고서는 변경한 본문 배치와 측정값을 구분하며, 쪽번호에 px 좌표 회귀를 추가하지 않는다.
현재 공개 HWPX는 이 보정 저장본이다. 수정 전 `dc87f4fa2e` 무표기판 증적은 이전 `7a54ea97…` HWPX에 해당하며,
새 저장본의 fresh WASM 증거로 재사용하지 않는다. 현재 Native 명령·manifest·JSON은 ignored output의
`visual/4b149b6cdf/native/`와 `logs/rebase-4b149b6cdf-native-direct.log`에 보관했다.

## 최적화 fresh WASM 전46쪽

같은 제품 source의 실제 빌드 head `7bb9e66fd006109ca9f7ee4c4d3f752c82e60fae` package를 사용했다.
WASM SHA-256은 `0fcc2a5d219ade9de64ce9657de95ce8769ce21d3bed4062835dc15e205ae3e1`다.
같은 원본 PDF·공개 HWPX·96 DPI·Print 2024와 전체 시스템 글꼴 공급에서 측정했다.
실제 WASM 가져오기·저장 결과도 위 HWPX 세 파일과 각각 바이트 단위로 같았다.

| 판본 | 전체 쪽수 | fresh WASM 최저 raw | 전쪽 TSV |
| --- | --- | --- | --- |
| 2010 | 16쪽 | 98.08% | [TSV](postscript-2010-wasm.tsv) |
| 2020 | 15쪽 | 99.18% | [TSV](postscript-2020-wasm.tsv) |
| plain | 15쪽 | 98.44% | [TSV](postscript-plain-wasm.tsv) |

전46쪽에 90% 미만·누락이 없고 독립 PDF 쪽수와 같다. 새 저장본의 fresh WASM 결과이며,
이전 저장본의 WASM 결과와 구분한다.

| 원 주장 영역 | fresh WASM raw | 같은 쪽의 review · overlay |
| --- | --- | --- |
| 공백 줄 뒤 본문 · 5쪽 | 99.68359% | [review](postscript-plain-p005-wasm-review.png) · [overlay](postscript-plain-p005-wasm-overlay.png) |
| 공백 줄 뒤 본문 · 13쪽 | 99.69047% | [review](postscript-plain-p013-wasm-review.png) · [overlay](postscript-plain-p013-wasm-overlay.png) |

두 쪽을 독립 PDF와 같은 배율로 직접 대조했다. 공백 줄 뒤 본문의 시작·줄바꿈·문항·보기의
위치와 상자 경계·내용 보존을 확인했다. 위에 기록한 13쪽 쪽번호 외형 차이와 본문 배치의
검증 범위는 계속 구분한다. 원시 실행과 실제 산출물은 ignored output의
`visual/4b149b6cdf/wasm/`와 `logs/rebase-7bb9e66fd0-wasm-manual-review-priority-case-74.log`부터 `case-76.log`에 있다.
