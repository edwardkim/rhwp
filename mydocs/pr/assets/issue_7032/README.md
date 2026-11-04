# #7032 HWPX 저장 줄의 독립 Print 확인

- 입력: `samples/hwpx/21761835_jeonjik_exemption_table.hwpx`, SHA-256
  `53b33bfca8d8a80534f27564ecee86403c6d1e3dc83cf405c7d6d158ae9625d2`.
- 입력의 `lastSavedWith`: 한컴오피스 2020, `11.0.0.9136`.
- 동일 HWPX의 한컴 2020 Print: `pdf/task2146/21761835_jeonjik_exemption_table-hwpx-2020.pdf`,
  SHA-256 `495fcddc4eed04918c776ae8b52931fde1923a59f8113fdfd9363b5311ad982f`.
  MCP 작업 `72b11ee2-5281-48ed-8fed-58fc0f72625f`, 실제 제품 `11.0.0.9136`,
  `pdf_print_method=0`, `hancom2020_pdf_driver_one_up`, 6쪽이다.
- 독립 PDF의 임베디드 폰트는 `Haansoft Batang`이며, 원본의 1쪽 제목 글자모양은
  `나눔명조`를 요청한다. 로컬에서 한컴 Print의 출력 face를 재현하기 위해
  `hwpx-print-font-environment.json`을 사용한다. 파일은 face 이름만 기록하며 폰트를 배포하지 않는다.
- 제품 보정 `5fdccefdd8`에 서식 변경만 더한 Native 실행 `7a982cead6b700a9516c70578405a7c95963dd6b2e654b33d5418cc60903bdcb`의
  96dpi PDFium 2px raw는 1쪽에서 기본 공급 96.42400%, 명시 공급 100.00000%다.
  이 결과는 1쪽 Native에 한정하며 다른 쪽이나 fresh WASM 성공을 뜻하지 않는다.

이전 `source8152-d2176`의 95~97% 비교는 HWPX 입력에 동일 원본 Print인지 입증되지 않은
기존 PDF를 사용했으므로 동일 HWPX Print와의 판정으로 사용하지 않는다. 기존 고정 좌표를
새 기준으로 재검토하려면 해당 1~6쪽의 Native와 fresh WASM 및 직접 판독이 모두 필요하다.

## 2026-10-29 대상 1~6쪽 재검증

동일 원본 Print와 96dpi PDFium, 색상 경계 보정 전 2px 실루엣을 사용했다. Native 제품
빌드 head는 `acd774b975`이며 제품 보정 `5fdccefdd8` 이후 서식만 변경됐다. fresh 최적화 WASM 빌드 head는 `a7686a0982`, source SHA-256
`fd08048a1ffafced513d7b6063c945c3fe5bec608e2579d69c9b6fab2de69050`, WASM SHA-256
`96ae8b9d7f809975525de5c611c78296501e15c066ef10dec7e4e66d7a807e46`이다. 두 원본 모두
각 6쪽이며 1~5쪽 raw는 Native·fresh WASM 모두 100.00000%다. 6쪽은 다음과 같다.

| 원본 | Native 6쪽 | fresh WASM 6쪽 | Native 전쪽 TSV | fresh WASM 전쪽 TSV |
| --- | ---: | ---: | --- | --- |
| HWP | 99.95290% | 99.95358% | [TSV](hwp-native.tsv) | [TSV](hwp-wasm.tsv) |
| HWPX | 99.44554% | 99.46831% | [TSV](hwpx-native.tsv) | [TSV](hwpx-wasm.tsv) |

TSV는 저장한 Native·WASM PNG와 동일 Print PNG를 `scripts/visual_sweep.py --png-pair`로
다시 비교해 표준 실루엣 형식·소수점 둘째 자리로 산출했다. 원본 값은 위와 같이 반올림 전 정밀도로 남긴다. HWPX 6쪽에는 표 본문과 후속
문장의 미세한 차이가 남으며 Native overlay를 직접 확인했다. 일치율 100%를 주장하지 않는다.

HWP는 동일 원본의 한컴 2020 Print `pdf/task2146/21761835_jeonjik_exemption_table-hwp-print-2020.pdf`
(SHA-256 `95487e5a7f193da6f96e8a16e0aa4ff0a39a15673a8f5dee10a78a0491fc1eb1`,
MCP 작업 `78e58b30-4c3c-4100-a2f8-c724fbb7b33b`, PrintMethod 0)을 사용한다.

두 독립 Print의 첫 가로선과 제목 행 하단 가로선의 PDF 경로 외곽 중심을 비교했다.
1쪽 간격은 39.317pt = 52.42267 CSS px, 2~6쪽 간격은 39.197pt = 52.26267 CSS px다.
PDFium 경로 bounds의 양 끝 중심을 사용했으며 글줄 간격이나 구현의 계산값과 구분한다.
각 페이지 Native/fresh WASM이 모두 98% 이상인 동일 글꼴 환경에서 이 독립 행 높이를
±2 CSS px로 검사하고, 기존 글줄 소유·순서·빈 문단 관계 검사도 유지한다.

같은 HWPX를 다시 Print한 PDF의 임베디드 글꼴 프로그램 SHA-256은
`6634690f6944dc6d687da89702f7411f7dcfd450987ce935343b3dbe647662be`로 최초 Print와 같다.
두 PDF의 1~6쪽은 96dpi PDFium 출력에서 모두 픽셀 단위로 동일하다. PDF 파일 전체 해시는
서로 다르며, 동일 입력 Print의 화면과 글꼴 비교만 뜻한다.

6쪽 직접 판독을 위한 [Native overlay](hwpx-p006-native-overlay.png)와
[fresh WASM overlay](hwpx-p006-wasm-overlay.png)를 보존한다.
