---
kind: working
status: completed
last_verified: 2026-10-09
---

# #7688 1단계 — Canvas·SVG·PNG 정합성 조사

Issue: [#7688](https://github.com/edwardkim/rhwp/issues/7688). 이 문서의 완료는 **대표 입력의 1차 조사 완료**를 뜻한다. 제품 결함 수정·모든 출력 경로의 정합성 완료·이슈 종료를 뜻하지 않는다. 관련 #536의 미병합 P45–P47 구현과 구분한다.

## 결론과 처리 순서

| 순서 | 실제 관측과 원인 계층 | 판정·다음 처리 |
| --- | --- | --- |
| 1 | `eq-01.hwp`의 수식 안 한글은 Canvas2D·SVG에서 보이지만 Native Skia PNG에서 네모로 출력된다. 본문 한글은 출력된다. 실제 바탕 TTC·Noto Serif KR 경로를 지정해도 수식 네모가 남았다. | **실행 결함 / backend 글꼴 소비 경로**. 수식이 본문의 custom/bundled 후보 및 문자별 glyph coverage 선택을 소비하게 하는 수정 범위를 먼저 정한다. 좌표·쪽수를 맞추는 예외로 해결하지 않는다. |
| 2 | CanvasKit 탭 문단·수식 본문은 Canvas2D와 다른 face를 선택한다. 진단의 `바탕체 → Noto Sans KR`, `unregisteredDefault`가 실제 화면의 명조/고딕 차이와 연결된다. | **실행 출력 차이 / 글꼴 공급**. opt-in CanvasKit의 준비된 font bytes와 실제 fallback을 명시해야 한다. readiness 통과는 동일 face 사용의 증거가 아니다. |
| 3 | `pic-crop-01.hwp` 두 번째 그림의 아래 흰 밴드는 Canvas2D·SVG·Native PNG에 공통이고 한컴에서는 파란 영역이 아래 경계까지 보인다. Native/fresh WASM SVG 모두 86.54494%. | **실행 공통 출력 결함**. crop 좌표 기준의 누락·대체 해석을 먼저 확인한다. backend 간 일치로 한컴 일치를 주장하지 않는다. |
| 4 | `group-drawing-02.hwp`의 화살촉은 한컴보다 작다. 세 경로에 공통으로 관측된다. | **실행 공통 외형 차이**. 도형 소유·쪽수 차이는 관측하지 않았으며, 독립 기대 크기와 입력 속성 추적은 미검증이다. |
| 5 | Canvas 보충 메트릭 선택으로 표 첫 쪽과 emoji의 폭·중앙 정렬이 달라진다. 내보내기 후 12쪽의 트리가 정확히 복원된다. | **의도된 메트릭 계약 차이와 미검증의 분리**. 저장본에서 관측된 차이를 무조건 배치 버그로 분류하지 않는다. emoji의 이번 한컴 PDF 근거와 편집 후 재조판은 미검증이다. |
| 6 | 표의 font decision trace가 `generated metric ruleId mismatch for 한컴 윤고딕 230`로 실패한다. 그 문서의 페이지 출력은 계속 생성된다. | **실행 진단 경로 결함**. trace provenance validator를 수정할 별도 경계다. trace 오류를 paint 실패로 보고하지 않는다. |

제품 소스·기존 baseline·허용치를 변경하지 않았다. 새 렌더링 회귀 테스트나 golden을 추가하지 않았다. 이번 결과의 미달 범위를 유지하며 전체 시각 통과로 선언하지 않는다.

## 실행 신원

- source/base: `6f66932a73ee6fd05a74a83bdc438c9499a89a12`, 최신 fetch 당시 `upstream/devel`.
- worktree: `/tmp/rhwp-issue7688-review-20261009`, branch `work/issue7688-renderer-audit-20261009`.
- 공용 target: `/home/edward/mygithub/rhwp/target/pr-review`.
- Native: `release-test`, `native-skia`, binary SHA-256 `baf5979778da39292c0ac4d5521f411c500928c5862e76b67535eefee583bf9a`.
- WASM: Docker image `sha256:4824b312625eba7b8146aa1c7eaca95710618beb75c4cec15470cdb76bc2d99b`, optimized wrapper 빌드.
- WASM SHA-256: `d509f8a11a4a3151d7868000c7526e5b71e92b5ef4c0a4167a42211af2382fa5`. worktree `pkg/`와 `rhwp-studio/public/` 동일. 실제 7788 서버에서 fetch한 바이트도 확인했다.
- JS SHA-256: `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e`.
- Linux headless Chrome `154.0.8037.57`, CanvasKit software, zoom 1/DPR 1 및 96 DPI 기본 비교. screen/print 별도 실행.
- root `devel`의 기존 WASM·Studio 서버를 바꾸지 않고 별도 Studio `http://localhost:7788`을 사용했다.

## 생산 결과와 실제 소비 경로

일반 `PageLayerTree` 진입점은 portable guard 이후 Canvas 트리 생성 본체를 사용한다 (`src/document_core/queries/rendering.rs:1314–1345`). 그러나 `select_canvas_metrics`는 파생 layout context를 다시 구성한다 (`canvas_metrics.rs:92–110`). 같은 함수 본체가 다른 활성 메트릭을 소비하면 같은 좌표 결과를 보장하지 않는다.

`text_measurement.rs:1558–1585`의 보충 폭은 `heuristicHalfwidth` 경계에만 적용되며 공백·제어문자·보호된 폭 규칙을 대신하지 않는다. Studio는 `CanvasMetricSession`에서 브라우저 측정을 등록하고 export 중 portable context로 바꾼 뒤 finally에서 복원한다. #7084의 `2498b605ce` 및 `bdd63167cc`(2026-09-13, edwardkim)가 이 경계를 도입했다. emoji를 기존 halfwidth로 되돌려 화면 간 좌표만 같게 만드는 방식은 이 경계의 목적을 잃는다.

Canvas2D 실제 화면은 `CanvasView → PageRenderer → WebCanvasRenderer`와 DOM 그림·앞뒤 레이어를 합성한다 (`page-renderer.ts:210–294`). Native PNG는 `PaintOp::Equation → skia/renderer.rs:1475/1486 → equation_conv.rs:709–762`를 소비한다. 이 수식 함수는 `FontMgr`/system families만 받으며, `draw_text`는 시스템 후보의 첫 face 또는 legacy face를 선택한다. 본문 경로의 custom/bundled map 및 문자별 `unichar_to_glyph` 확인(`font_lookup.rs:52–104`)을 사용하지 않는다. producer의 수식 LayoutBox를 공유하더라도 최종 glyph 소비 계약은 공유하지 않는다는 구체적인 반례다.

그림 경로는 DOM의 `imageCropSourceRect`, SVG/WebCanvas/Skia의 `compute_image_crop_src`로 이어진다. 이번 이미지의 decoded 크기는 639×70, crop은 첫 그림 `(0,0,47940,5280)`, 두 번째 `(0,0,47940,4366)`이며 트리에 `originalSizeHu`가 없다. 적응 폴백은 양 축의 right/bottom을 전체 좌표 범위로 가정하여 두 그림 모두 원본 전체 창을 만든다 (`svg.rs:4153–4200`). 아래 자르기 경계를 전체 높이로 간주하는 가정이 의심된다. 원본 HWP record·독립 좌표 기준을 추가 확인하기 전 그 가정을 대체 규칙으로 확정하지 않는다.

## 비교 실행 결과

기존 manifest에서 문단·탭·표·crop·수식·그룹 도형 6개 입력을 사용했다. 표 3쪽과 emoji HWP/HWPX 2쪽씩까지 확장해 총 8개 문서·12쪽의 메트릭을 읽었다. 입력 해시는 `six-samples-manifest.json`과 `metric-probe-manifest.json`에 고정했다.

| 검사 | 실행 결과와 한계 |
| --- | --- |
| 기존 CanvasMetricSession·supplemental metric Node tests | 16 PASS / 0 FAIL / 0 skip. 한컴 출력 증거와 구분한다. |
| 6문서 × screen/print × Canvas2D·CanvasKit compat/default | 36 캡처. capture error 없음, hidden fallback 진단 위반 0. CanvasKit pixel parity는 기존 report-only 24 비교 중 16 pass/8 fail이며 변경하지 않았다. |
| Native 동일 바이너리 SVG·Skia PNG | legacy SVG 및 layer SVG screen/print, Skia PNG screen/print 출력. feature를 문서마다 재빌드하지 않았다. |
| Canvas2D·SVG raster·Skia PNG 고정 픽셀 비교 | 36 비교. report-only 0.5% pixel 예산 실패를 조판 결함으로 일괄 승격하지 않았다. |
| 같은 Chrome·Studio FontFaceSet·portable context의 Canvas2D/SVG 대조 | fresh WASM 7788, Light DOM에서 대표 6개 실행. Canvas/SVG 2px 실루엣은 일반 문단 98.16850%, 탭 99.52231%, 표 99.69169%, crop 92.12631%, 수식 99.48954%, 도형 99.69672%. 같은 FontFaceSet은 동일 실제 face byte 사용의 확정 증거가 아니며 일부 font response body hash 수집은 실패했다. |
| fresh WASM Canvas/portable/restored 트리 | 12/12 복원 정확히 일치. 표 첫 쪽 11 필드, emoji HWP/HWPX 둘째 쪽 각각 4 필드의 폭·위치 차이. 나머지 9쪽은 동일. 모든 쪽의 page count 유지. 트리 전체 diff에서 내용 추가·삭제·다른 소속은 관측하지 않았다. |
| 실제 CanvasView zoom·DPR·resize | 탭·DOM 그림·emoji에 initial → zoom 1.25 → viewport 980/DPR 1.5 → DPR 2 → initial 복원. 15/15 트리 동일. 원래 조건으로 돌아온 3/3 쪽 PNG는 완전 동일. 중간 확대 그림은 직접 판독용으로 정규화했으며 gate 이미지로 쓰지 않았다. |

페이지 캔버스의 `ElementHandle.screenshot()`으로 해당 화면 사각형을 캡처했다. 이는 `toDataURL()`이 아니므로 같은 사각형의 DOM 이미지와 레이어도 포함한다. 최초 parent 캡처는 여러 쪽·UI를 포함해 단일 쪽 검증으로 부적절했고, `studio-page-boundaries/`에서 캔버스 사각형으로 재캡처했다. 해당 이전 진단 파일은 삭제하지 않고 최종 쪽 비교 근거에서 제외했다.

보충 Light DOM 첫 실행은 기본 서버 7700으로 연결되어 현재 head 근거에서 제외했다. 이를 보존하고 `VITE_URL=http://localhost:7788`·실제 fetch WASM 해시 확인으로 다시 실행했다. 최종 근거는 `same-font-light-dom-results.json`·`light-dom-silhouette-results.json`이다. 이전 same-font/shadow 비교는 최종 승인 근거로 사용하지 않는다.

## 독립 한컴 Print PDF와의 대조

6개 입력의 저장 세대는 Hancom 2010이었다. canonical MCP client CLI로 Hancom 2020 Print PDF를 생성했다. 6/6 작업 성공, 출력 계약은 `pdf_print_method=0`, `hancom2020_pdf_driver_one_up`, 실제 버전 11.0.0.9136. 표 3쪽을 포함한 전체 8쪽을 Native/fresh WASM SVG에서 각각 비교했다.

| 문서 | 쪽 | Native SVG / fresh WASM SVG 실루엣 일치율 |
| --- | --- | --- |
| `lseg-01-basic.hwp` | 1 | 100.00000% / 100.00000% |
| `lseg-05-tab.hwp` | 1 | 98.13974% / 98.13974% |
| `hwp_table_test.hwp` | 1–3 | 98.23583%, 97.41234%, 97.87220% — 두 경로 동일 |
| `pic-crop-01.hwp` | 1 | **86.54494% / 86.54494%** |
| `eq-01.hwp` | 1 | 95.89322% / 95.89322% |
| `group-drawing-02.hwp` | 1 | 99.79162% / 99.79162% |

두 sweep 모두 crop으로 exit 1 / gate 미달. 점수가 높은 수식·그룹 도형도 앞 표의 실제 네모·화살촉 차이를 점수로 면제하지 않는다. 이 표는 **SVG → browser PNG의 한컴 비교**이며 Native Skia PNG나 Studio Canvas의 전체 한컴 gate 통과라고 읽으면 안 된다. 세 대표 문서의 Native compare/overlay/review PNG를 추가 출력해 직접 판독했다. fresh WASM 동일 점수는 전쪽 TSV 근거이며 별도 fresh WASM review PNG는 이번 단계에서 생성하지 않았다.

## 재현 명령과 증적

산출물 기본 경로: `/home/edward/mygithub/rhwp/output/pr-review/renderer-backend-audit-20261009/`.

- 전체 비교 뷰어: `/home/edward/mygithub/rhwp/output/pr-review/renderer-backend-audit-20261009/comparison.html`
- 수식 확대 비교: `/home/edward/mygithub/rhwp/output/pr-review/renderer-backend-audit-20261009/equation-inline-detail-panel.png`
- crop 확대 비교: `/home/edward/mygithub/rhwp/output/pr-review/renderer-backend-audit-20261009/image-crop-detail-panel.png`
- 그룹 도형 확대 비교: `/home/edward/mygithub/rhwp/output/pr-review/renderer-backend-audit-20261009/shape-group-detail-panel.png`
- Native crop review: `/home/edward/mygithub/rhwp/output/pr-review/renderer-backend-audit-20261009/pdf-native-review/image-crop/review/review_001.png`
- fresh 트리 차이·복원: `/home/edward/mygithub/rhwp/output/pr-review/renderer-backend-audit-20261009/canvas-portable-fresh-structure-results.json`
- Studio zoom/DPR/resize 결과: `/home/edward/mygithub/rhwp/output/pr-review/renderer-backend-audit-20261009/studio-page-boundary-results.json`
- 원래 표시 조건으로 복원한 PNG 결과: `/home/edward/mygithub/rhwp/output/pr-review/renderer-backend-audit-20261009/studio-page-restoration-pixel-results.json`

Docker wrapper는 worktree **루트**에서 실행했다:

```bash
docker run --rm --name rhwp-issue7688-wasm-20261009 \
  -v /tmp/rhwp-issue7688-review-20261009:/app \
  -v /home/edward/mygithub/rhwp/target/pr-review:/build-target \
  -v /home/edward/.cargo:/home/builder/.cargo \
  -w /app -e CARGO_TARGET_DIR=/build-target rhwp-wasm:latest \
  sh -ec 'scripts/wasm-pack-locked.sh --target web --out-dir pkg'
```

Native 및 actual Studio 진단:

```bash
# worktree root
cargo build --locked --profile release-test --features native-skia --bin rhwp \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review
/home/edward/mygithub/rhwp/target/pr-review/release-test/rhwp export-png \
  samples/eq-01.hwp --page 0 --profile print \
  --font-path /mnt/c/Windows/Fonts \
  --font-path /home/edward/vsworks/myproject/Noto_Serif_KR/static \
  --output /home/edward/mygithub/rhwp/output/pr-review/renderer-backend-audit-20261009/native-font-control

# root workspace, 이미 실행된 ignored 진단 스크립트 재현
VITE_URL=http://localhost:7788 \
CHROME_PATH=/home/edward/.cache/puppeteer/chrome/linux-154.0.8037.57/chrome-linux64/chrome \
node output/pr-review/renderer-backend-audit-20261009/studio-page-boundary-probe.mjs --mode=headless
```

완전한 Native/fresh WASM sweep argv·환경·cwd는 `pdf-native-command.json`·`pdf-wasm-command.json`, 대표 review 추가 출력은 `pdf-native-review-command.json`, 실행 로그·exit는 대응 파일에 있다. Build/input/manifest 해시는 `worktree-provenance.json`·`wasm-provenance.json`·`native-provenance.json`에 있다. MCP receipts와 PDF는 ignored 로컬 증적으로 보존했으며 인증 정보나 font 파일을 게시하지 않았다.

## 남은 미검증 범위

편집 후 재조판/부분 갱신, 실제 Windows/macOS renderer, CanvasKit GPU/WebGL, 도형 rotation/clip/z-order·이미지 효과·form·모든 glyph variant는 이번 대표 입력으로 완결하지 않았다. emoji 두 문서의 이번 한컴 PDF도 없다. source SHA가 바뀌면 필요한 경계와 fresh WASM 시각 증적을 다시 실행해야 한다.

따라서 #7688은 열린 상태로 유지한다. 1단계 근거를 바탕으로 수식 font 소비 경로부터 구현하고, 공통 crop의 입력 근거를 확정한 뒤 영향 페이지와 정상 대조군을 검증한다. 필수 90% 근거가 충족되기 전 새 rendering regression/golden을 추가하거나 PR을 완료로 보고하지 않는다.
