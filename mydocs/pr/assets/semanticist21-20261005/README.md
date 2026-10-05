# semanticist21 누적 검토 증거

24개 원 PR의 리뷰 기록은 `mydocs/pr/archives/pr_<번호>_review.md`에 각각 있다. #7508의 이번 후속 head는 `pr_7508_review_20261005.md`이다. 이 디렉터리는 시각·입력 출처 증거와 진단 재현 스크립트를 보존한다. nextest 등 검증 실행의 원문 출력은 ignored `output/pr-review/semanticist21-20261005/`에 저장한다.

- 최초 접수 base: `cdba77b609c399fdef26a6c9e637716aa32c2177`
- 최초 접수 code candidate: `1d809afe7b965c9ea6137012d59d139d63b038d0`
- 원 PR/적용 SHA/충돌 보정: `application.json`
- 집중 실행: `output/pr-review/semanticist21-20261005/run-records/focused-command.json`을 cargo 인자로 실행. `output/pr-review/semanticist21-20261005/run-records/focused-results.json`은 74건(73 PASS/1 FAIL)을 원 case별로 구분한다.
- Native base 진단: `output/pr-review/semanticist21-20261005/run-records/base-probes.txt`의 2개 테스트는 동일 원본의 표 좌표 및 긴 HTML 문단 길이를 관측한다. 테스트 기대값/golden 변경을 하지 않았다.

## Chromium 진단 재현

저장소 루트에서 fresh WASM과 `rhwp-studio/public`의 해시 일치를 먼저 확인한다. `npm --prefix rhwp-vscode ci --ignore-scripts`로 TypeScript를 설치하고, `npm install --prefix output/pr-review/semanticist21-20261005/browser --no-save --ignore-scripts playwright-core`로 Chromium 제어 의존성을 준비한다. Chromium 실행 경로는 이 서버의 `/snap/bin/chromium`이다.

```bash
python3 -m http.server 18765 --bind 127.0.0.1
# 별도 셸, 저장소 루트
node mydocs/pr/assets/semanticist21-20261005/browser-review.mjs
node mydocs/pr/assets/semanticist21-20261005/kerning-browser.mjs
node mydocs/pr/assets/semanticist21-20261005/outline-runtime.cjs
```

스크립트 출력 경로는 `output/pr-review/semanticist21-20261005/`다. 진단 출력은 공식 Visual Sweep gate나 한컴 PDF의 독립 기준을 대체하지 않는다. 안내문은 사용자 지시에 따라 screen 상태만 검사하고 PDF 비교에서 제외한다.

## 최종 폼 검증과 글꼴 경로 대조

Rust code `cf2336295540ea8ce3e94eb6517cb406fca8d28f`, JS 반영 head `7ca40721f`에서 fresh WASM을 빌드했다. `appearance-wasm-hashes.json`에 pkg/Studio public JS·WASM 해시 일치를 기록한다. 사용자 요청의 `rhwp-studio/public/rhwp.js`와 기준 PDF 두 개는 이미 커밋했다.

`font-path-verification.json`은 환경변수를 제거한 Native/WASM과 동일 PDF font face를 명시한 두 결과를 구분한다. 한컴 설치 자체는 이미 존재한다. 명시 경로는 재설치 요구나 실행 필수 설정이 아니라 독립 기준 PDF의 `Haansoft Batang / 한컴바탕`을 정확히 공급하기 위한 비교 조건이다. 등록 전 조사 당시 `fc-match '한컴바탕'`은 `/usr/local/share/fonts/hwp-convert-mcp-survey/8664669bd2d6-HANBatang.ttf`의 `HCR Batang / 함초롬바탕`을 반환한다. 이 face와 한컴 설치본 `All/HBATANG.TTF`의 face는 서로 다르다. RHWP의 파일 탐색 기본값도 `/usr/share/fonts`·`/usr/local/share/fonts`이고 한컴 app 내부 경로는 자동 추가하지 않는다.

환경변수 없이 원본/암호 Native와 fresh WASM의 2px 관용 실루엣 일치율은 각각99.34142%/98.81531%로 네 gate 모두 PASS다. 이 경우 Native SVG에는 font data URI가 없고 local 한컴바탕/함초롬바탕/HCR Batang alias가 있어 설치 글꼴 fallback을 사용한다. 동일 PDF face를 지정한 비교는99.88501%/99.90053%로 PASS다. 폰트 미설치로 단정하지 않으며 점수만으로 동일 glyph face라고 주장하지 않는다. 두 조건의 review·overlay·manifest는 `pdf/semanticist21-20261005/form-appearance/`에 각각 보존했다.

실제 WebCanvas 및 CanvasKit 진단은 `combobox-browser.mjs`, ignored `output/pr-review/semanticist21-20261005/historical-browser-raw/appearance-browser-results.json`이다. 루트 HTTP18765와 Studio Vite18766을 사용한다. 먼저 위 동일 face Native full embedding Sweep을 `output/pr-review/semanticist21-20261005/appearance-final-native`에 실행한다. 원 HBATANG 파일은 Chromium FontFace가 Invalid font data로 거부하므로, 진단 harness는 RHWP의 기존 full SVG 임베더가 bitmap table/cmap/checksum을 정리한 실제 `한컴바탕` font data URI를 재사용한다. 다른 glyph face로 대체하지 않는다. CanvasKit actual renderer의 render complete/error·fallback 수와 원본/암호 실제 화면을 확인했다. screen 안내문은 print PDF와 비교하지 않는다.

## 서버 한컴 글꼴 시스템 등록

사용자 요청으로 `/usr/local/share/fonts/hancom-office-2020`에서 기존 `/opt/hnc/hoffice11/Shared/TTF`를 연결하고 fc-cache를 갱신했다. Hwp/All/Install을 포함하며201개 face를 등록했다. 이전 `62-hwp-convert-mcp-hanbatang.conf`는 보존하고, 새 `99-rhwp-hancom-exact-family.conf`에서 `한컴바탕` 요청에 실제 Haansoft Batang을 prepend_first한다. 기존 강제 HCR alias를 그대로 두면 정확한 글꼴을 등록해도 선택이 HCR로 남는 것을 직접 확인했다. 최종 fc-match와 글꼴 SHA는 `hancom-system-font-registration.json`에 있다. 이 조치 이후 결과는 앞선 미등록 환경의 기본 경로 결과와 구분한다. 글꼴 바이너리를 저장소에 추가하거나 한컴 설치본을 수정하지 않았다.

## 실행 출력 보존 정책

`mydocs/manual/pr_review/local_validation.md`에 따라 nextest 및 로컬 검증의 원 출력·실행 JSON은 ignored `output/pr-review/semanticist21-20261005/logs/`와 `run-records/`에만 저장하며 Git에 포함하지 않는다. 확장자를 txt/json으로 바꾼 실행 출력도 동일하다. 개별 PR 문서에는 검토 source·명령·통과/실패 요약과 로컬 경로를 적는다. 입력 HWP/HWPX·독립 PDF와 Visual Sweep의 대표 이미지·입력/빌드 provenance는 시각 증거 정책에 따라 보존한다.

## 2026-10-06 검토 위치

현재 branch `review/semanticist21-20261005`는 최신 base `c167dc6abbebf69546575e2d16d06223791bab82` 위에 rebase했다. 원59개 적용 기록은 `application.json`에 유지하고 실제 rebase 위치를 개별 review에 기록했다. 위 접수/폼 결과는 당시 source의 이력이며 최신 code candidate 검증과 구분한다.

#7504 실제 등록 source, #7562 실제 XML 축, #7571 production adapter/바깥 상자, #7530 production 2쪽 저장의 입력과 Print PDF를 개별 디렉터리에 보존한다. 그 외 실제 API 입력도 PR 번호별 디렉터리에 두며 해당 PDF는 `pdf/semanticist21-20261005/pr<번호>/mcp`에 있다. 새 조판 회귀의 실제 FAIL/PASS·TSV·nextest 원 출력은 ignored output에서만 보존한다.


## 최종 후보 재캡처 — 2026-10-06

production source `2b1f21ef1ab35a13ebcae11f562a3ebf3a998e4d`와 Native 전용 관계 회귀 source `9af7586586587fa0aa617a9e57fd6acd0d4e3ba6`의 production 코드는 같다. fresh WASM SHA-256 `24565cae976b3c6929c858f13c52785a4651a26dc61c0fd566f5a8f801d631f7`, JS `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e`; root pkg/Studio 동기화를 확인했다.

각 backend 29개 검증 항목·33쪽 전쪽이90% 이상(최저93.40356%), 누락/측정 불가0이며 font exception은 쓰지 않았다. 검증 항목은 동일 입력의 별도 검사도 포함해 고유 입력 개수와 다르다. #7527은93.58%라도 다른 단 소속이 발견되어 보정했고 새3쪽98.89/98.79/99.99%의 실제 배치를 확인했다. #7508의 내부/한컴 조각 붙여넣기 저장본은 별도 MCP2020 Print와 Native/WASM100%다.

개별 PR 디렉터리의 review/overlay PNG는 이 재출력에서 복사했다. 원시 TSV·nextest·중간 JSON·브라우저 결과는 ignored `output/pr-review/semanticist21-20261005`에만 보존하며 이전 browser JSON도 `historical-browser-raw`로 이동했다. 원 저자·개별 체리픽 출처와 의미 있는 입력/PDF SHA·Print job provenance는 계속 보존한다. `query-geometry.mjs`, `combobox-browser.mjs`의 결과 경로도 ignored output이다.


페이지별 TSV는 현재 head에서 재출력한 각 `rhwp_png`/`pdf_png` 쌍에 canonical `--silhouette-only --png-pair`를 실행해 `output/pr-review/semanticist21-20261005/final-tsv/<native|wasm>/<key>/silhouette.tsv`에 산출했다. 58개 capture(backend별29항목)·66쪽 대응, 90% 미만0/누락0. 이는 기존 오래된 캡처의 재사용이 아니라 위 production source의 최종 재출력에서 PNG 해시를 고정한 TSV이며 전체 원문/독립 PDF의 쪽수도 별도로 대조했다. `not_evaluated` TSV를 승인으로 사용하지 않고 같은 capture의 full review gate와 직접 판독을 함께 적용했다.

실행 출력 정책을 중간 시각 검증에도 적용했다. #7491 manifest/metrics/run_manifest와 중간 gate JSON, 폼 PDF의 문자 추출 JSON, 글꼴 조건별 raw 실행은 ignored `historical-visual-raw`로 이동했다. 입력·기준 PDF·Print job·글꼴/source/산출물 SHA provenance와 대표 PNG는 보존한다. `font-path-verification.json`에는 source/실제 글꼴 경로만 남기고 원시 실행 경로를 연결했다.
