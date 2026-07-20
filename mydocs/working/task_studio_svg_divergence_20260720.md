# studio(CanvasKit) ↔ CLI export-svg 렌더 상이 문서 조사·작업목록

작성일: 2026-07-20
상태: 조사 완료(1차), 이슈화 대기(작업지시자 승인 후)

## 목적

rhwp-studio 화면 출력(CanvasKit/Skia 래스터)과 CLI `export-svg` 출력(SVG 백엔드)이
**시각적으로 상이한** 샘플 문서를 찾아내고, 상이 유형을 분류하여 수정 작업목록으로 정리한다.
두 경로는 같은 레이아웃 엔진을 공유하고 **렌더 백엔드만 다르므로**, 여기서 잡히는 차이는
백엔드 렌더 정합(개체/글리프/이미지 효과) 문제로 좁혀진다.

> **기준(baseline) = CLI `export-svg`(SVG 출력).** 따라서 "상이"는 곧 **studio(CanvasKit)가 SVG 기준에서 벗어난 지점**이고,
> 원칙적 **수정 대상은 studio** 다. 예외: 기준(SVG) 자체가 결함으로 보이는 경우(§B)는 studio 를 SVG 에 맞추는 대신
> **기준 결함부터 판정**한다.

## 방법

- **대상(큐레이션 서브셋)**: `scripts/renderer_baseline_manifest.json` 의 120 (파일,페이지) 쌍
  (문단/폰트/표/이미지/수식/머리말/각주/미주/필드/컨트롤/시험지/도형/폼/hwpx/차트/혼합 등 카테고리 망라).
- **studio 측**: headless Chrome(Chrome for Testing 151) + rhwp-studio dev 서버에서
  `pageRenderer.renderPage(page, canvas, 1,1,1)` 로 지정 페이지를 native 해상도 캔버스에 렌더 → PNG.
  (renderer-baseline e2e 캡처 방식 재사용, 로컬 폰트/타이프페이스 정착 대기 포함)
- **CLI 측**: `rhwp export-svg <file> -p <page>` → SVG → `rsvg-convert -w W -h H`(studio 캔버스와 동일 px)로 래스터 → PNG.
- **비교 지표**: 잉크 마스크(평균 채도 <240) 를 정렬 오차 흡수용으로 **반경 2px 팽창(dilation)** 한 뒤,
  한쪽에만 남는 잉크 비율(structScore = max(studioOnly/inkA, cliOnly/inkB)) 을 산출. 원시 픽셀차(rawDiffRatio)도 병기.
  - structScore 는 **개체 누락/큰 위치이동/색 상이** 같은 구조적 차이에 민감하고, 서브픽셀 안티앨리어싱에는 둔감.
- 빌드 리비전: core/pkg 모두 `devel` HEAD(v0.7.19)로 재빌드 후 실행.

### 재현(하니스)

하니스 스크립트와 산출물은 세션 스크래치패드에 있다(리포 미커밋). 영구 게이트가 필요하면
`rhwp-studio/e2e/` 로 승격 검토.

```
# 사전: pkg 빌드(wasm-pack build --target web --out-dir pkg), rhwp release 빌드, vite dev(:7700),
#       chrome 경로 CHROME_PATH 지정
CHROME_PATH=.../chrome VITE_URL=http://127.0.0.1:7700 \
  node <scratch>/studio_vs_cli.mjs --mode=headless
# 개별: SVC_ONLY=chart-line-markers-hwp,pr-149-regression ...
```

## 결과 요약

- 대상 120쌍 중 studio 캡처 성공 119, 실패 1(exam-kor: 대용량 로드 타임아웃).
- structScore ≥ 5% = 유의 상이 후보. 이 중 **모서리 재단표시/서브픽셀 잡음**을 시각 확인으로 걸러낸 뒤
  실제 내용 상이는 아래 A~D.

기준=SVG, 수정대상=studio 원칙. "studio vs SVG 기준" 열은 studio 가 기준에서 어떻게 벗어났는지를 뜻한다.

| # | 유형 | 심각도 | 대상 문서 수 | studio vs SVG 기준 | 수정 대상 |
|---|------|--------|--------------|---------------------|-----------|
| A | 차트 개체 studio 미렌더 | **높음** | 4 | studio 백지(기준엔 차트 있음) | **studio** |
| B | 이미지 회색조/흑백 효과 | **높음** | 1(+유형) | studio 렌더, **기준(SVG)이 누락** | **기준(SVG)** ← 예외 |
| C | native bitmap 글리프 렌더 상이 | 중간 | 1(+유형) | studio 깨짐(기준은 정상 벡터) | **studio** |
| D | 텍스트 글리프 위치/메트릭 미세 상이 | 중간(광범위)/낮음(개별) | 다수 | studio 글리프가 기준과 미세 오프셋 | **studio** |
| E | studio 대용량 문서 로드 실패 | 트리아지 | 1 | studio 렌더 불가(기준은 정상) | **studio**(툴/성능) |
| F | 모서리 재단표시 studio-only | 낮음 | 다수 | studio-only(기준 미표시) | 의도 확인(수정 아닐 가능성) |
| G | 그림 미지정 placeholder 프로필 차이 | 정상(기지) | 1 | 프로필별 계약차 | 없음(#2225) |

증적 PNG(studio/cli/diff)는 `<scratch>/compare/<id>.{studio,cli,diff}.png`.

---

## A. 차트 개체가 studio(CanvasKit)에서 렌더되지 않음 — 최우선

- **증상**: 차트 포함 페이지가 studio 에서 **완전 백지**(잉크 86px, 페이지 모서리 마크만). CLI SVG 는 라인/축/범례/데이터 정상 렌더.
- **대상 문서**
  - `samples/chart/라인/표식이있는꺽은선형.hwp` (p0) — struct 100%, inkA 86 / inkB 5460
  - `samples/chart/라인/표식이있는꺽은선형.hwpx` (p0) — 동일
  - `samples/chart/기타/고가저가종가.hwp` (p0) — struct 100%, inkA 86 / inkB 3691
  - `samples/chart/기타/고가저가종가.hwpx` (p0) — 동일
- **방향**: 기준(SVG)에는 차트가 있고 studio 두 백엔드 모두 차트를 미렌더. 수정 대상 = studio.

### 근본 원인(국소화 완료, 2026-07-20)

차트는 벡터 **`RawSvg` op**(`class="hwp-ooxml-chart"`, viewBox 4715B SVG 조각)로 emit된다. `getPageLayerTree(0)`
에 `rawSvgCount=1` 로 존재하고, 그 조각을 `wrap_svg_fragment` 로 감싸 `HtmlImageElement` 로 로드하면
`naturalWidth=430 · naturalHeight=250` 로 **정상 디코드**되고 캔버스에 그리면 잉크 3730px 로 렌더된다
(즉 SVG 데이터·디코드는 무결).

- **SVG 백엔드**(`src/renderer/svg.rs:516`): RawSvg 를 인라인 → `export-svg` 정상(기준).
- **studio canvaskit 백엔드**(`rhwp-studio/src/view/canvaskit-renderer.ts:894`):
  `case 'rawSvg': this.unsupportedOps.add('rawSvg:unsupportedDirectReplay')` — **rawSvg 미지원**, 아예 안 그림.
- **studio canvas2d 백엔드(기본)**: `src/renderer/web_canvas.rs:1072 render_raw_svg` → `draw_image(svg_bytes)`.
  `draw_image`(2659) 는 SVG 를 **동기 래스터화 못함**(`decode_image_to_canvas`(88) 는 `image` 크레이트라
  SVG 디코더 없음 → None) → 비동기 `HtmlImageElement` 경로로만 그릴 수 있어 **첫 페인트에는 차트가 없다**.
  이후 로드 완료 시 재렌더가 다시 그려야 하는데, 그 트리거가 벡터 rawSvg 를 커버하지 못한다:
  - `page-renderer.ts:946 prefetchLayerImages` 는 **raster data URL 만**(정규식 `"type":"image"…"base64"`,
    `data:image/…;base64`) 프리페치 → 벡터 차트엔 내부 raster 가 없어 `tasks.length===0 → false` →
    `scheduleReRender`(834) 의 early `finish()`(로드 완료 즉시 재그리기)가 **호출되지 않음**.
  - 남는 트리거는 **단발 1500ms fallback 타이머**(`IMAGE_RE_RENDER_FALLBACK_DELAY_MS`) 뿐인데,
    실측상 이 단발 재렌더가 overlay 에 차트를 칠하지 못한다(로드 타이밍 미스/미착지).

- **실측 증거**(headless canvas2d 기본, `probe_final.mjs`):
  앱의 `flow-static` overlay 캔버스는 로드 3.5s 후에도 `ink=0`(흰 배경만 opaque=889746). 그러나 이미지
  로드 후 그 **동일 overlay 에 `renderPageToCanvasFiltered(0, overlay, 'flow-static')` 를 1회 수동 호출하면
  즉시 `ink 0 → 4689`**(차트 렌더). 반복 호출(`probe_flowstatic.mjs`)도 4689 안정. → WASM 캔버스 경로는
  정상이며, 결함은 **로드 완료 후 overlay 재페인트가 발생하지 않는 studio 재렌더 트리거**에 있다.

### 수정 방향(studio)

1. `prefetchLayerImages`/`scheduleReRender` 가 **벡터 rawSvg 도** 대기·재렌더 대상으로 포함:
   rawSvg 조각을 `wrap_svg_fragment` 규약대로 감싼 SVG data URL 을 프리페치(load 완료 신호 확보)하여
   로드 즉시 `finish()`(overlay 재렌더)가 발동하도록 한다. 단발 1500ms fallback 의존 제거.
2. (대안/병행) `web_canvas.rs`/이미지 캐시에서 rawSvg 를 `resvg` 등으로 **동기 래스터화**해 첫 페인트에 포함.
3. **canvaskit 백엔드에 rawSvg 지원 추가**(현재 `unsupportedDirectReplay`) — 위 rasterize 자원을 공유.

### 수정 적용(1안, 2026-07-20) — 기본 canvas2d 백엔드 해결

`rhwp-studio/src/view/page-renderer.ts` (TS-only, wasm 재빌드 불필요):
- 모듈 헬퍼 `utf8ToBase64`, `rawSvgFragmentToDataUrl`(= `wrap_svg_fragment` 와 바이트 동일 래핑),
  `collectVectorRawSvgDataUrls`(레이어 트리에서 내부 raster 없는 벡터 rawSvg 수집) 추가.
- `prefetchLayerImages` 에 벡터 rawSvg data URL 프리페치 추가(`json.includes('"type":"rawSvg"')` 가드).
  → 벡터 rawSvg 로드 완료 시 `scheduleReRender` 의 early `finish()` 가 발동 → flow-static overlay 재렌더 →
  WASM 캐시의 SVG 이미지가 로드 완료 상태로 그려진다.

**회귀 검증(headless canvas2d 기본)**: 차트 4문서 모두 overlay 잉크 `0 → 렌더`
(line-markers hwp/hwpx 4689, stock hwp/hwpx 3056), 자연 스크린샷 시각 확인(차트/축/범례 정상,
`compare/chart-{line-markers-postfix,stock}-VERIFY-natural.png`). 비-rawSvg 문서(biz_plan) 무회귀.
`npx tsc --noEmit` 0 errors.

**남은 후속**: canvaskit 백엔드(비기본)는 여전히 rawSvg 미지원(위 3항) — 별도 작업.

### 증적 — 인과 A/B(수정 ON/OFF), 2026-07-20

동일 하니스·동일 문서(`chart/라인/표식이있는꺽은선형.hwp`, headless Chrome-for-Testing 151, 기본 canvas2d)에서
`page-renderer.ts` 의 수정만 토글해 앱 실제 `#scroll-container` 의 `flow-static` overlay 잉크를 측정.
파일 되돌림→측정→재적용→측정을 각 3회 반복(플레이키 아님을 배제).

| 상태 | 파일 마커 | flow-static overlay 잉크(3회) | 판정 |
|------|-----------|-------------------------------|------|
| 수정 OFF(pristine) | 0 | **0 · 0 · 0** | 차트 미렌더(백지) |
| 수정 ON(적용) | 4 | **4689 · 4689 · 4689** | 차트 렌더 |

- 수정만 토글했을 때 `0 ↔ 4689` 로 재현성 있게 뒤집힘 → **실제·결정적 버그**이며 수정이 **인과적**(우연/타이밍 아님).
- 증적 이미지(같은 페이지, 같은 뷰):
  - 수정 OFF(백지): `assets/studio_chart_A_prefix_blank.png`
  - 수정 ON(렌더): `assets/studio_chart_A_postfix_rendered.png`
  - 기준(CLI export-svg): `assets/cli_chart_A_baseline.png`
- 검증 한계(정직 고지): headless Chromium(= 헤드리스여도 실제 Chromium 렌더 엔진) 기본 canvas2d 경로에서
  측정. 나머지 3개 차트 문서도 overlay 잉크 `0→렌더` 동일 확인(line/stock, hwp·hwpx). 최종 정본
  판정(한컴 편집기/기준 PDF 대조)은 작업지시자 권위. canvaskit 백엔드(비기본)는 이 수정 범위 밖.

- **재현**: `SVC_ONLY=chart-line-markers-hwp node .../studio_vs_cli.mjs --mode=headless`,
  `rhwp export-svg "samples/chart/라인/표식이있는꺽은선형.hwp" -p 0 -o out/`,
  국소화 프로브 `.../scratchpad/probe_{rawsvg,flowstatic,appcanvas,final}.mjs`.
- **증적**: `compare/chart-line-markers-hwp.{studio,cli}.png`, `chart-line-markers-VERIFY-natural.png`.

## B. CLI export-svg 가 이미지 회색조/흑백 효과 개체를 누락 — 높음

- **증상**: `pr-149.hwp` 는 동일 이미지의 **원본/회색조/흑백** 3종을 세로로 배치. studio 는 3종 모두 렌더,
  **CLI SVG 는 회색조·흑백 이미지를 누락**(원본만 렌더, 회색조는 깨진 조각). inkA 251869 vs inkB 93948, raw 18.4%.
- **대상 문서**: `samples/pr-149.hwp` (p0). 같은 이미지 효과(gray/mono/duotone)를 쓰는 문서 전반에 잠재.
- **방향(예외)**: 본 조사 기준은 SVG 지만, 이 항목은 **기준(SVG) 쪽이 결함**으로 보인다 — studio 는 3종을 정상 렌더,
  SVG 가 회색조·흑백 개체를 누락. 따라서 studio 를 SVG 에 맞추면 안 되고, **SVG 백엔드(export-svg)를 수정**해야 한다.
- **조사 포인트**: SVG 백엔드의 그림 효과(회색조/흑백/듀오톤) 이미지 파생본 생성·삽입 경로.
  studio(CanvasKit)는 효과 이미지를 그리므로 원본 데이터/IR 은 존재 → SVG 직렬화 단계 누락 의심.
- **증적**: `compare/pr-149-regression.{studio,cli}.png`.

## C. native bitmap 글리프 렌더 상이 — 중간

- **증상**: `render-p35-font-native-bitmap.hwpx` 의 단일 글자("건")가 studio 에서 **깨진 축소 비트맵**,
  CLI 에서 정상 벡터 글리프. inkA 135 / inkB 50, struct 63%.
- **대상 문서**: `samples/hwpx/render-p35-font-native-bitmap.hwpx` (p0).
- **방향**: 양쪽 상이. CLI 가 시각적으로 정상. studio(CanvasKit) 의 native bitmap 글리프 처리 결함 의심.
- **조사 포인트**: CanvasKit 타이프페이스/네이티브 비트맵 글리프 경로(글리프 아웃라인 payload 상태 관련).
- **증적**: `compare/font-native-bitmap.{studio,cli}.png`.

## D. 텍스트 글리프 위치/메트릭 미세 상이(백엔드 파리티) — 중간(광범위)

- **증상**: 본문 텍스트가 양쪽에 모두 있으나, 글리프 가장자리에서 studio-only(빨강)/CLI-only(파랑)가
  전반적으로 남음 = **미세 수평 오프셋/자간/셰이핑 차이**. 개별 페이지는 4~10%대.
- **대상(대표)**
  - `samples/hwpx/hwpx-02.hwpx` (p0) — struct 5.7%, **raw 6.6%**, inkA 40869 / inkB 52025 (본문 전반, 가장 두드러짐)
  - `samples/tac-case-001..005.hwp` (p0) — 4~7%, 특정 줄/런에서 상이
  - `samples/form-01.hwp` (p0) — 4.8%, 상단 몇 줄
  - `samples/re-mixed-malgun-timesnew-hancom.hwp` (p0) — 9.9%(저잉크, 혼합폰트)
  - `samples/hwpx/ref/ref_text.hwpx` (p0) — 18.9%(저잉크, 대부분 모서리마크 기여)
- **방향**: 양쪽 상이. 한컴/기준 PDF 대조로 어느 쪽이 정본인지 판정 필요(작업지시자 권위).
- **조사 포인트**: CanvasKit 텍스트 셰이핑/자간·폰트 메트릭 vs SVG 백엔드 텍스트 배치. 한 개의 공통
  글리프-어드밴스/폰트 대체 원인일 가능성 → **개별 이슈보다 파리티 계열로 묶어** 우선 hwpx-02 로 재현·국소화 권장.
- **증적**: `compare/hwpx-basic-02.diff.png` 등.

## E. exam-kor: studio 대용량 문서 로드/렌더 실패 — 트리아지

- **증상**: `samples/exam_kor.hwp`(10MB, 20p) 로드 시 studio 캔버스 대기 타임아웃(`#scroll-container canvas`).
  CLI 는 정상(export-svg 20p 성공). 렌더 상이라기보다 **로드 성능/타임아웃** 가능성.
- **후속**: headless 타임아웃 상향 후 재현 여부 확인 → 진짜 실패면 대용량 문서 로드 경로 조사.

## F. 페이지 모서리 재단표시 studio-only — 낮음(계측/설계 확인)

- 저잉크 문서 다수에서 structScore 를 부풀린 원인. studio 는 페이지 모서리 재단표시(L자 마크)를 그리고,
  CLI export-svg(legacy)는 그리지 않음. **본문 내용은 일치**. 편집기 chrome 대 인쇄 등가의 의도적 차이로 추정
  → 의도 확인만 필요(수정 대상 아닐 가능성 높음).

## G. 그림 미지정 placeholder 프로필 차이 — 정상(기지)

- `missing-picture-profile`(hwpx opengov)에서 screen 프로필 placeholder 차이. #2225/#2297 계약(편집기 표시 ↔ 인쇄 억제)에 따른 정상 동작.

---

## 계측 한계

- structScore 는 정렬오차 2px 를 흡수하지만, **모서리 재단표시(F)** 와 **혼합폰트 저잉크 페이지**에서
  과대 계상될 수 있음 → 상위 후보는 반드시 diff/studio/cli PNG 시각 확인으로 검증함(본 문서 A~D 는 확인 완료).
- 최종 정본 판정(어느 렌더가 맞는가)은 한컴 편집기/기준 PDF 대조가 필요(작업지시자 권위).
- 본 스윕은 각 문서 **대표 1페이지**만 대상. 다페이지 개체(표지·차트·이미지)는 별도 페이지 확대 필요.

## 이슈화 후보(승인 대기)

작업지시자 승인 시 아래를 각각 GitHub 이슈로 등록(로컬 task 번호 ≠ upstream 번호, 등록 전 실제 번호 확인).

1. **[높음] studio 차트 개체 미렌더** — A, 대상 4문서. 수정=studio(CanvasKit). 증적 첨부.
2. **[높음] export-svg 이미지 회색조/흑백 효과 누락** — B, `pr-149.hwp`. **수정=SVG 백엔드**(기준 결함, 예외).
3. **[중간] studio native bitmap 글리프 렌더 결함** — C, `render-p35-font-native-bitmap.hwpx`. 수정=studio.
4. **[중간] studio 텍스트 글리프 메트릭을 SVG 기준에 맞춤** — D, 대표 `hwpx-02.hwpx` 로 국소화. 수정=studio.
5. **[트리아지] exam_kor 대용량 문서 studio 로드 실패** — E. 수정=studio(툴/성능).
6. (확인) F(모서리 재단표시)/G(placeholder) 는 의도/기지 여부 확인 후 종결.
