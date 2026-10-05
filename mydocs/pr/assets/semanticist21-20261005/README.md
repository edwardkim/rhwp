# semanticist21 누적 검토 증거

19개 원 PR의 리뷰 기록은 `mydocs/pr/archives/pr_<번호>_review.md`에 각각 있다. #7508의 이번 후속 head는 `pr_7508_review_20261005.md`이다. 이 디렉터리는 실행 증거와 진단 재현 스크립트이며 통합 리뷰 문서가 아니다.

- base: `cdba77b609c399fdef26a6c9e637716aa32c2177`
- code candidate: `1d809afe7b965c9ea6137012d59d139d63b038d0`
- 원 PR/적용 SHA/충돌 보정: `application.json`
- 집중 실행: `focused-command.json`을 cargo 인자로 실행. `focused-results.json`은 74건(73 PASS/1 FAIL)을 원 case별로 구분한다.
- Native base 진단: `base-probes.txt`의 2개 테스트는 동일 원본의 표 좌표 및 긴 HTML 문단 길이를 관측한다. 테스트 기대값/golden 변경을 하지 않았다.

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

`font-path-verification.json`은 환경변수를 제거한 Native/WASM과 동일 PDF font face를 명시한 두 결과를 구분한다. 한컴 설치 자체는 이미 존재한다. 명시 경로는 재설치 요구나 실행 필수 설정이 아니라 독립 기준 PDF의 `Haansoft Batang / 한컴바탕`을 정확히 공급하기 위한 비교 조건이다. 현재 `fc-match '한컴바탕'`은 `/usr/local/share/fonts/hwp-convert-mcp-survey/8664669bd2d6-HANBatang.ttf`의 `HCR Batang / 함초롬바탕`을 반환한다. 이 face와 한컴 설치본 `All/HBATANG.TTF`의 face는 서로 다르다. RHWP의 파일 탐색 기본값도 `/usr/share/fonts`·`/usr/local/share/fonts`이고 한컴 app 내부 경로는 자동 추가하지 않는다.

환경변수 없이 원본/암호 Native와 fresh WASM의 2px 관용 실루엣 일치율은 각각99.34142%/98.81531%로 네 gate 모두 PASS다. 이 경우 Native SVG에는 font data URI가 없고 local 한컴바탕/함초롬바탕/HCR Batang alias가 있어 설치 글꼴 fallback을 사용한다. 동일 PDF face를 지정한 비교는99.88501%/99.90053%로 PASS다. 폰트 미설치로 단정하지 않으며 점수만으로 동일 glyph face라고 주장하지 않는다. 두 조건의 review·overlay·manifest는 `pdf/semanticist21-20261005/form-appearance/`에 각각 보존했다.

실제 WebCanvas 및 CanvasKit 진단은 `combobox-browser.mjs`, `appearance-browser-results.json`이다. 루트 HTTP18765와 Studio Vite18766을 사용한다. 먼저 위 동일 face Native full embedding Sweep을 `output/pr-review/semanticist21-20261005/appearance-final-native`에 실행한다. 원 HBATANG 파일은 Chromium FontFace가 Invalid font data로 거부하므로, 진단 harness는 RHWP의 기존 full SVG 임베더가 bitmap table/cmap/checksum을 정리한 실제 `한컴바탕` font data URI를 재사용한다. 다른 glyph face로 대체하지 않는다. CanvasKit actual renderer의 render complete/error·fallback 수와 원본/암호 실제 화면을 확인했다. screen 안내문은 print PDF와 비교하지 않는다.

## 서버 한컴 글꼴 시스템 등록

사용자 요청으로 `/usr/local/share/fonts/hancom-office-2020`에서 기존 `/opt/hnc/hoffice11/Shared/TTF`를 연결하고 fc-cache를 갱신했다. Hwp/All/Install을 포함하며201개 face를 등록했다. 이전 `62-hwp-convert-mcp-hanbatang.conf`는 보존하고, 새 `99-rhwp-hancom-exact-family.conf`에서 `한컴바탕` 요청에 실제 Haansoft Batang을 prepend_first한다. 기존 강제 HCR alias를 그대로 두면 정확한 글꼴을 등록해도 선택이 HCR로 남는 것을 직접 확인했다. 최종 fc-match와 글꼴 SHA는 `hancom-system-font-registration.json`에 있다. 이 조치 이후 결과는 앞선 미등록 환경의 기본 경로 결과와 구분한다. 글꼴 바이너리를 저장소에 추가하거나 한컴 설치본을 수정하지 않았다.
