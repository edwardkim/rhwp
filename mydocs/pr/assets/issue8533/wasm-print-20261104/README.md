# WASM Print 메트릭·글꼴 프로그램·JPEG 인쇄 검증

이 묶음은 #8533의 WASM 후속 보정 증적이다. 비교 base는
`1924d371d6757392d063b8dc2e9bbe9a298be817`이다. 편람 3·4쪽 클릭·Esc 편집 문제는
사용자 지시에 따라 [#8578](https://github.com/edwardkim/rhwp/issues/8578)로 분리했으며
이 PR의 해결 범위에 포함하지 않는다. 초기 묶음 우선 선택 구현도 철회했다.

## 입력과 측정 조건

- 입력: [2025 행정업무운영 편람(최종).hwpx](<../../../../../samples/2025 행정업무운영 편람(최종).hwpx>),
  SHA-256 `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6`.
- 독립 기준: [한컴 2024 Print PDF](<../../../../../pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf>),
  SHA-256 `299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e`.
  #8389에서 사용한 같은 PDF를 보존했다. 새로 출력하거나 PDF를 수정하지 않았다.
- 전체 383쪽, Print/compat 2024, 96 DPI, canonical PDFium RGB exact-DPI v2,
  내용 문턱 232, 2px 관용, `threshold_boundary_color_support_v1`.
  아래 판정에는 색상 경계 보정 전 `silhouette_raw_match_percent`를 사용한다.
- 원본·PDF의 실제 파일 바이트와 Git blob 일치, 페이지 1–383의 누락·중복 부재,
  root pkg/Studio public JS·WASM 일치를 확인했다. [생산 신원](visual-validation.json)에
  파일별 SHA-256, backend, 공개 공급 프로그램, 도구와 원 manifest 해시를 보존했다.
- 최소 글꼴 환경은 개인 글꼴 파일 접근과 CoreText의 `com.apple.fonts`·
  `com.apple.fonts.atsbridge` 조회를 함께 차단하고 공개 공급 9개 및 저장소 번들을 사용했다.
  파일만 차단하여 LastResort 물음표가 나온 초기 진단은 완료 증거가 아니다.

## 전쪽 결과

| 실제 출력·환경 | 쪽수 | 최저 raw % | 최저 쪽 | #8389 하한 하락 | 원본 TSV |
| --- | ---: | ---: | ---: | ---: | --- |
| Native Skia·최소 글꼴 | 383 | 99.96465 | 130 | 0 | [native.tsv](tsv/native.tsv) |
| fresh WASM Canvas2D·최소 글꼴 | 383 | 99.96403 | 130 | 0 | [wasm-public.tsv](tsv/wasm-public.tsv) |
| fresh WASM Canvas2D·일반 설치 | 383 | 99.96401 | 130 | 0 | [wasm-normal.tsv](tsv/wasm-normal.tsv) |
| Native Skia·최소 글꼴·옵션 단독 | 383 | 99.96465 | 130 | 0 | [native-option-only.tsv](tsv/native-option-only.tsv) |

옵션 단독 실행은 `--font-path`나 `--font-environment`를 직접 지정하지 않았다.
`--handbook-gate`가 누락된 공개 프로그램을 자동 공급했고 207.3초·exit 0으로 완료했다.
명시 공급 실행과 전383쪽 raw PNG의 SHA-256이 같으므로 아래 Native 대표 PNG를
그대로 연결한다. 설치 캐시가 읽을 수 없는 face를 설치됐다고 판단하던 결함도 보정했다.
새 반례는 실제 2 FAIL → PASS, 전체 Visual Sweep Python은 245 PASS다.

세 경로 모두 90% 미달·누락·측정 불가 0쪽이다. #8389의
[기준 TSV](../../issue_8153/final-50cdf-validation-20261030/handbook-hwpx-native.tsv)는
130쪽 99.96%, 나머지 382쪽 100.00%를 기록한다. 해당 **기록 정밀도 0.01%p를 그대로**
적용해 비교했으며 기준 파일·하한을 바꾸지 않았다. 100.00% 표시는 RGB 픽셀 완전 동일이나
원시값이 정확히 100%라는 뜻이 아니다. [정밀 raw·전쪽 PNG 해시](page-results.tsv)를 별도로 제공한다.

[직전 기록과의 페이지별 대조](recorded-source-comparison.tsv)도 보존한다. 기존 증적의
검증 호스트 `3ededa625581f4a4a052ef912dc30c0317fec0ed`와 현재 base `1924d371d6` 사이에는
`src/`, `crates/`, `scripts/`, Studio 제품 소스 및 Cargo 입력의 diff가 없다. 해당 기록을
새 base 실행으로 바꿔 표기하지 않는다. 같은 일반 설치 환경의 WASM은 39쪽 개선·하락 0쪽이다.
Native 열은 직전 일반 설치와 현재 최소 글꼴 실행의 환경 차이도 포함하므로 코드만의
변화로 해석하지 않는다. 관측 raw 최저 차이는 -0.00394%p이며 #8389의 기록 하한은 유지한다.

Native 실행 파일 SHA-256은 `8ae681680593809015ebb5c0a19ddaa5269ba11aa56d467882d6be81ab3d8728`,
최신 WASM은 `b366ca4bee534ef618ad82a38ab71801cf84585fb6b000438d0b6ce09d54d03b`이다.
WASM은 `f656a013b3` 및 이후 `dea51e0bec`에 커밋한 문맥 문자 경계 보완을 빌드한 결과다.
빌드 시 부모 SHA만 인용하지 않고 해당 소스 파일 해시도 현재 제출 소스와 대조했다.
Native 빌드 이후 보완한 WASM 전용 파일은 Native compiler 의존 목록에 포함되지 않는다.
PostScript 전용 face 보정과 클릭 코드 철회는 Rust/WASM 바이너리를 변경하지 않는다.

Mac fresh WASM은 locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
Docker 최적화 빌드로 주장하지 않는다. JS SHA-256은
`bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`이며
Studio 초기화 응답의 실제 WASM SHA도 위 최신 파일과 일치했다.

## 구현 주장과 검증

| 주장 | 실제 소비 경로·비적용 경계 | 검출과 결과 |
| --- | --- | --- |
| Screen 폭이 Print 조판에 남지 않음 | Rust Canvas Print 진입의 portable guard, Studio의 전환/오류 후 복원, Sweep의 쪽수 조회 전 전환 | #7084 실제 10 PASS/1 FAIL → 11 PASS. 원문·화면 측정 소속 유지 |
| 원본 TTF 프로그램을 family 체인 그대로 공급 | 실제 FontFace 프로그램 등록 → declared font/session의 같은 family 분리 규칙 → paint coverage | 기존 프로그램 공급 회귀와 실제 글꼴 체인 검사 통과. 198·220·272쪽 전쪽 결과에 포함 |
| 정적 TTF fallback의 Print 윤곽 일치 | 실제 공급된 선택 face의 cmap/weight → 제한된 program/contour cache → Print 2024 painter | 새 윤곽 검사 수정 전 FAIL → PASS. 원 face의 H, Screen, 2022 Print, HighQuality, 미공급 face, 단독 아랍어는 기존 painter 유지 |
| JPEG 프레임도 공통 Print 복호화 사용 | 일반 그림과 장치 프레임이 같은 `decoded_source`의 IJG RGB 소비 | 140쪽 독립 PDF 영역 raw 미일치 [7, 1] → [0, 0]. 기존 #8160 대조군 유지 |
| PostScript 이름만 있는 face도 공급 목록 생성 | 이름 병합 후 primary family 선택, cold/warm cache | 새 검사 2건 IndexError → 관련 Python 38 PASS. 이름이 전혀 없는 face의 명시 오류 유지 |

일·생활균형 기존 검사는 화면 메트릭이 활성화된 채 Print를 호출했다. 기존 원점·문자 소속
assertion을 지우거나 허용폭을 늘리지 않고 Screen과 portable Print 양쪽에 유지했다.
잘못된 Print 상태의 거부·복원도 검사한다. 본문·줄 구성·문자 전진을 시각 수치에 맞춰 수정하지 않았다.
분할·이어받기 계산과 원본/PDF·baseline·품질 문턱은 변경하지 않았다.

새 140쪽 고정 영역은 보정 전 Native 100%, fresh WASM 99.98238% 페이지에서
독립 PDFium PNG를 무보간 절단한 것이다. [fixture 정본](../../../../../tests/fixtures/issue8533/handbook-print-frame.json)에
출처를 기록했다. `RHWPPrintFallbackContext.ttf`는 단독 문맥 문자 경계용 합성 시험 파일이며
제품 글꼴·사용자 설치·웹폰트 배포에 포함하지 않는다.

## 대표 review·overlay와 직접 판독

각 행은 같은 물리 쪽의 실제 raw PNG, canonical review, standalone overlay다.
이미지 합성 과정에서 좌표·배율·입력 그림을 보정하지 않았다.

| 쪽·확인 영역 | Native | 최소 글꼴 fresh WASM | 독립 PDF |
| --- | --- | --- | --- |
| 30: 반각 닫는 괄호·본문 기준선 | [raw](png/native-p030-raw.png) · [review](png/native-p030-review.png) · [overlay](png/native-p030-overlay.png) | [raw](png/wasm-public-p030-raw.png) · [review](png/wasm-public-p030-review.png) · [overlay](png/wasm-public-p030-overlay.png) | [raw](png/pdf-p030-raw.png) |
| 130: 기존 최저 대조군·삽입 화면 | [raw](png/native-p130-raw.png) · [review](png/native-p130-review.png) · [overlay](png/native-p130-overlay.png) | [raw](png/wasm-public-p130-raw.png) · [review](png/wasm-public-p130-review.png) · [overlay](png/wasm-public-p130-overlay.png) | [raw](png/pdf-p130-raw.png) |
| 140: 로그인 JPEG 프레임 | [raw](png/native-p140-raw.png) · [review](png/native-p140-review.png) · [overlay](png/native-p140-overlay.png) | [raw](png/wasm-public-p140-raw.png) · [review](png/wasm-public-p140-review.png) · [overlay](png/wasm-public-p140-overlay.png) | [raw](png/pdf-p140-raw.png) |
| 198: 표 안 글머리표·문자 소속 | [raw](png/native-p198-raw.png) · [review](png/native-p198-review.png) · [overlay](png/native-p198-overlay.png) | [raw](png/wasm-public-p198-raw.png) · [review](png/wasm-public-p198-review.png) · [overlay](png/wasm-public-p198-overlay.png) | [raw](png/pdf-p198-raw.png) |
| 220: 표 안 글머리표 뒤 간격 | [raw](png/native-p220-raw.png) · [review](png/native-p220-review.png) · [overlay](png/native-p220-overlay.png) | [raw](png/wasm-public-p220-raw.png) · [review](png/wasm-public-p220-review.png) · [overlay](png/wasm-public-p220-overlay.png) | [raw](png/pdf-p220-raw.png) |
| 272: 원·괄호·번호·상하 위치 | [raw](png/native-p272-raw.png) · [review](png/native-p272-review.png) · [overlay](png/native-p272-overlay.png) | [raw](png/wasm-public-p272-raw.png) · [review](png/wasm-public-p272-review.png) · [overlay](png/wasm-public-p272-overlay.png) | [raw](png/pdf-p272-raw.png) |

직접 판독에서 30쪽 닫는 괄호의 끝, 140쪽 버튼·테두리·그림 내부, 198·220쪽 글머리표와
뒤 간격, 272쪽 원·번호의 모양과 기준선을 확인했다. 내용·줄바꿈·표 테두리·앞뒤 문장 누락은
보이지 않았다. 130쪽은 기존과 같은 삽입 화면의 미세 차이가 남으며 이를 100%로 표시하지 않는다.
색상 overlay의 주황색 자체는 gate 실패가 아니며 raw 실루엣 판정과 구분한다.

정식 제출 전 전체 회귀·필수 게이트의 최종 결과는 [검증 정본](validation.md)에 기록한다.
중간 로그·driver·진단은 `output/pr-review/issue8533-wasm-advance-20261104/`에 두며 커밋하지 않는다.
