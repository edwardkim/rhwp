# Task #7974 — 첫 마스크 구현과 집중 검증

- 날짜: 2026-10-17
- 이슈: [#7974](https://github.com/edwardkim/rhwp/issues/7974)
- 승인: 사용자 “계획·이슈 등록·첫 구현 승인”. [승인 계획](../plans/task_m100_7949_backlog_followup.md).
- 기준: `upstream/devel` `515bd72dcde0eecdeece333666b675354b7fa91d`.
- 브랜치/작업공간: `task_m100_7974`, `/home/edward/mygithub/rhwp-backlog7949`.
- 빌드 시 HEAD는 준비 문서 commit `b4b388e2e368689800b7e050950e54fa9758d5c1`이며 제품 변경은 미커밋 상태였다.
  실제 입력 파일 SHA는 ignored `output/pr-review/backlog-7949/source-files.json`과
  `implementation.patch`에 연결한다. 뒤의 테스트·보고 추가는 제품 코드를 바꾸지 않았다.
- 현재 상태: 제한된 첫 구현·Native/Docker WASM/Studio 집중 검증과 **사용자 직접 시각 판정 통과**.
  2026-10-17 사용자가 비교 화면의 마스크 가려짐·확대 경계에 “시각 판정 통과”라고 회신했다.
  전체 회귀·lint·Worker/패키지 게이트와 PR 준비는 다음 단계이며 실행 승인을 기다린다.

## 1. 적용한 계약과 실제 소비 경로

PDF의 색상 키 `/Mask`는 Decode·색 변환 **전** 원래 성분을 모두 비교한다. Gray/RGB/CMYK,
Indexed, 1/2/4/8/16-bit packed sample을 처리한다. Indexed의 같은 RGB인 다른 index와
16-bit 인접 샘플도 구분한다. `/SMask` 우선순위를 보존하며 잘못된 배열·범위·길이를 거절한다.

독립 `/ImageMask`는 현재 nonstroking 색·alpha를 받는다. 페이지/Form의 `q/Q`, Form 재사용,
resource 색상, inline image, 반전 Decode와 행 패딩을 확인했다. Pattern 등 지원하지 않는
색상과 추가 codec을 임의의 RGB나 원시 비트로 해석하지 않는다. 크기·stride·픽셀 수는 checked
산술로 계산하고 실제 source/alpha/RGBA 예약과 루프는 기존 요청 예산·취소 검사에 연결한다.

선택한 값 `Interpolate`의 실제 경로는 다음과 같다.

`PdfImageHandle::decode` → `Mask::apply` 결과의 sampling → bounded `png_limited::encode`
→ PNG/BinData → `ImageNode.data` → Native `skia/image_conv::draw_image_bytes`,
production `SvgLayerRenderer`가 호출하는 `SvgRenderer::render_image_node`,
`WebCanvasRenderer::render_image`, Paint JSON/좁은 flow query → Studio CanvasKit/DOM 그림이다.

- PNG IHDR 직후 private ancillary unsafe-to-copy `rhWP`에 버전 1과 보간 값을 기록한다.
  소비 helper는 고정 47-byte prefix와 CRC32만 읽는다. HWP/HWPX 저장은 원본 PNG를 보존하고,
  인식하는 자체 picture tone 변환은 선언을 함께 보존한다. 일반 그림의 profile 선택은 유지한다.
- Paint schema는 additive minor 26이다. `imageInterpolation`이 없는 기존 자료의 기본 동작을 유지한다.
  bitmap/binary effect의 nearest 우선순위도 Native·CanvasKit에서 유지한다.
- base와 mask의 보간이 같거나 다른 plane이 상수이면 단일 texture 선언을 전달한다.
  두 plane 모두 변화하고 보간이 다르면 기존 compositor/profile을 유지한다. 초안의 새 거절이
  기존 type002를 열지 못하는 회귀를 만들어 제거했다. **두 plane의 독립적인 sampling을 정확히
  표현하는 기능은 후속 백로그**이며 이 구현의 해결 범위로 주장하지 않는다.
- 원본 크기·CTM·clip·개체 소유를 보정값으로 바꾸지 않았다. 선언 대신 가짜 해상도·문서 조건을
  추가하지 않았다. `parserAllocationBoundsProven=false`도 유지한다.

근거는 [PDF Reference 1.7 §4.8.5](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf)와
[PNG chunk 규약](https://www.w3.org/TR/png-3/#5Chunk-naming-conventions)이다.

## 2. 수정 전후와 시각 증거

증거 루트는 ignored `output/pr-review/backlog-7949/`다. 공개 source PDF는 `pdf/issue7974/`,
개인 type002와 실제 글꼴/로그/PNG는 Git에 추가하지 않았다.

| 범위 | 수정 전 | 현재 실제 실행 |
|---|---|---|
| 동일 작은 입력 21개 | 기존 binary의 공통 import 경로에서 13개 미지원 오류 | 모두 1쪽 열기. 20개 Poppler, 16-bit 한 건 MuPDF와 전쪽 대조 |
| 저해상도 명시적 Mask | Native 경계가 흐려짐, 94.36919% | Native/fresh WASM 100%, 동일 크기·위치에서 경계 보존 |
| 확대 scaled 대조군 | Native 87.98561% | Native/fresh WASM 100% |
| 전체 작은 입력 | 기존 8개만 열림 | Native 최저 92.92381%, WASM Canvas/SVG 최저 92.64496%. 나머지 20개는 양쪽 100% |
| 기존 type002 | 기준 devel은 19쪽 열기 | 19쪽 일치. Native 최저 98.48631%, fresh WASM 98.40687% |
| HWP/HWPX 저장 재열기 | 새 유형을 가져올 수 없어 새 저장 불가 | 작은 21개와 type002의 쪽수 보존. 각 문서 첫 쪽 Native PNG는 저장 전과 동일 |
| 실제 Studio | 이전 WASM을 현재 것으로 추정하지 않음 | Canvas2D·CanvasKit software에서 3개 재현본 화면 확인. 실제 초기화 WASM hash 일치, browser error 없음 |
| DOM flow image | 마스크 PDF의 positional 그림 경로와 구분 | 저장 PNG를 새 문서에 insertPicture하여 실제 DOM 그림 생성·decode·pixelated CSS와 화면 캡처 확인 |

`linear-control`은 보간을 켠 정상 대조군이며 Native의 수정 전후 실루엣은 모두 92.92381%다.
Poppler/Skia/브라우저의 2×2 texture 보간 모양·양자화가 다르다. 최저 점수를 숨기거나
정답 위치를 수정하지 않았다. PDF 규정의 보간 선택과 원래 사각형·색상/크기는 유지하며
점수와 직접 판독을 구분한다.

`gray16-key`는 raw 32767/32768의 Mask 판정이다. Poppler 출력은 두 값을 같은 8-bit 값으로
축소해 Mask를 구분하지 못한다. 사양과 MuPDF 출력으로 대조한 Native/fresh WASM은 100%다.
Poppler 출력도 `edge-inputs/gray16-key.png`에 보존하며 기대값을 축소 결과에 맞추지 않았다.

직접 판독은 원색/가려진 칸, Indexed duplicate 색, 16-bit alpha, Form별 색, 반전 행,
현재 색·중간 alpha, CMYK, 확대 경계, inline을 포함한다. 실제 Studio는 요청한 두 backend를
사용했으며 기본 7700을 수정하지 않았다. `studio/evidence.json`은 실제 WebAssembly 초기화에
넘긴 응답 bytes를 hash해 빌드와 대조한다. 별도 사후 fetch만을 증거로 삼지 않았다.

**한계:** 저장본의 PNG 동일 검사는 첫 쪽 한정이다. 저장본 전체 쪽을 비교했다고 주장하지 않는다.
현재 type002 전쪽 값은 독립 PDF와의 비교이며 exact-base fresh WASM의 전쪽 하락 비교를 대신하지 않는다.
`--silhouette-only`의 `not_evaluated`를 최종 PR 통과로 바꾸지 않는다.

## 3. 실행과 정식 검사

- Native: 공용 `/home/edward/mygithub/rhwp/target/pr-review`에서
  `CARGO_BUILD_JOBS=8 cargo build --locked -p rhwp --lib --bin rhwp --profile release-test --features native-skia --target-dir …` 성공.
- WASM: 저장소 루트에서 기존 `-p rhwp` named volumes를 재사용한
  `docker compose --env-file .env.docker -p rhwp run --rm --no-deps -e CARGO_BUILD_JOBS=8 -e CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 wasm` 성공.
  `pkg`와 Studio/public의 glue/WASM hash가 일치한다. WASM SHA-256:
  `4932c4c08302648a3758e9acb3fafef8e899e27fd38ff5cf95dd3a0d52beccd5`.
- `run-visual.py`, `run-visual.py --wasm`: canonical Visual Sweep의 실제 CLI Skia/Canvas PNG 전쪽 실행.
  `run-svg.py`는 production SVG를 실제 Chrome에서 rasterize했다. 16-bit은 같은 공식
  silhouette 함수의 PNG-pair 경로로 MuPDF와 비교했다. 명령·출력은 각 `*.log`와 manifest에 보존한다.
- 새 `tests/cases/issue_7974_pdf_source_masks.rs`: **정식 nextest 8건 통과**.
  raw sample/alpha, 동일 RGB의 index 구분, 16-bit, Form별 색/소유, 반전/행 패딩/alpha,
  손상·과도 입력 뒤 복구, 저장·production SVG sampling, SMask 우선순위/성분 비교를 검사한다.
  source PDF와 spec이 기대값이며 실제 조판 px/golden/PNG hash는 고정하지 않는다.
- 수정 전 증거는 동일 입력을 같은 public import→Native paint로 실행한
  `before-same-inputs/results.tsv`와 PNG다. 새 formal test binary 자체를 base에서 실행한 결과와는
  구분한다. 기존 unsupported 오류·실제 흐림을 해당 경로의 실패 근거로 남긴다.
- Studio 집중 Node **25건 통과**, TypeScript `tsc --noEmit -p rhwp-studio` 통과.
- 기존 SVG layer/paint 집중 회귀 **29건 통과** (`existing-focused-tests.log`).
- manifest의 기준 SHA 비교, unit-tier, format, diff 검사 통과. 전체 integration/CI를 실행한 결과는 아니다.

준비/빌드 실패는 제품 결함 재현과 구분했다. 초기 SVG의 fmt::Write import 누락을 고쳤고,
pkg 이전 TypeScript module 오류는 실제 빌드 후 해소했다. 구소스 WASM 최적화는 코드 변경 후
중단하고 최종 소스를 빌드했다. CDP 응답 body cache 부족은 실제 instantiate 응답 hash로 확인하는
진단을 사용해 해소했다. 첫 정식 실행은 파생 harness에 새 case가 빠져 0건이었으므로 통과로 세지
않고 canonical prepare/check 후 **실제 8건**을 실행했다.

## 4. 다음 경계와 사용자 확인 경로

- 사용자 확인: `E:\tmp\issue7974-mask-review-20261017\index.html`.
  원본·저장 HWP/HWPX·전쪽 TSV·PNG, 기존 type002 19쪽 비교가 모두 상대 경로로 열리며 파일 존재를 확인했다.
- Studio: `http://localhost:7795/?review=issue7974-mask&lang=ko`. 기존 7700/메인 작업공간 WIP는 보존했다.
- 서로 다른 두 sampling plane, MOF 원본 확보·글꼴/간격, 부분 투명 텍스트·graphics-state soft mask·
  blend/text clip·표 의미 승격·추가 codec은 백로그에 남긴다. MOF 원본 경로는 아직 확인되지 않았다.
- 사용자 직접 시각 판정은 통과했다. 다음 단계 승인 뒤 전체 회귀·lint·공개 API/Worker/패키지 소비와
  최신 base 하락 비교를 정리한다. 원격 push·PR·이슈 종료는 아직 수행하지 않았다.
- Docker가 동기화한 `rhwp-studio/public/rhwp.js`는 로컬 실행 산출물로 보존하고 이번 source commit에
  포함하지 않는다. `pkg/`·WASM·중간 증거·`.env`류도 stage하지 않는다. 같은 제품 소스를 반복 빌드하지 않는다.

## 5. 사용자 판정 반영과 재사용 확인

2026-10-17 사용자는 `E:\tmp\issue7974-mask-review-20261017\index.html`과 Studio 7795의
색상 키·stencil·저해상도 확대에 대한 확인 요청에 **“시각 판정 통과”**라고 회신했다.
판정 대상 제품 commit은 `6b072955e9b83d8f7729133eda5e93536ee453f5`다.
새 fetch의 devel도 `515bd72dcde0eecdeece333666b675354b7fa91d`로 같아 추가 통합은 필요 없다.
기록된 19개 runtime 입력 SHA가 현재 파일과 동일하며 Native/fresh WASM TSV의 raw 값도
90% 이상임을 기존 자료에서 확인했다. 동일 빌드·집중 검사·시각 캡처를 다시 실행하지 않았다.

다음 검증은 `docs_and_git_workflow.md`의 Internal Task PR Approval에 따른 전체 CI 성격의
검사 승인 후 수행한다. 이 시각 판정을 원격 push·PR 생성·전체 CI 승인으로 확장하지 않는다.
