---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-124/README.md
last_verified: 2026-07-17
---

# 일부 상용 제품 캔버스 렌더링 분석 및 DPR 스케일링 전략

작성일: 2026-02-21

## 1. 분석 배경

타스크 124(글리프 패스 렌더링)를 시도한 결과, WASM에서 글리프 아웃라인을 Canvas Path2D로 변환하는 방식으로는 한컴 웹기안기의 렌더링 품질을 따라잡을 수 없다는 결론에 도달했다. 이에 일부 상용 제품의 실제 Canvas 렌더링 방식을 분석하여 우리 제품에 적용할 전략을 수립한다.

## 2. 일부 상용 제품 렌더링 방식 비교

### 2.1 세 가지 접근법

| 접근법 | 제품 | 텍스트 렌더링 | DPR 처리 | WASM 크기 |
|--------|------|-------------|----------|-----------|
| WASM 비트맵 | 일부 상용 웹오피스 | WASM 내부 FreeType | WASM에 DPR 배율 전달 | 19MB |
| 서버 JSON + fillText | 한컴 웹기안기 | Canvas fillText | ctx.scale(DPR) | 서버 의존 |
| fillText + DPR | 구글 독스 | Canvas fillText | ctx.scale(DPR) | - |

### 2.2 WASM 비트맵 방식 (일부 상용 웹오피스)

**아키텍처**: C++ Emscripten WASM (19MB) → RGBA 비트맵 → putImageData

**핵심 코드 흐름**:
```javascript
// 1. DPR 감지
devicePixelRatio = window.devicePixelRatio;
dpi = Math.floor(96 * devicePixelRatio);

// 2. Canvas 크기 설정
canvas.width = logicalWidth * DPR;      // 물리 픽셀
canvas.height = logicalHeight * DPR;
canvas.style.width = logicalWidth + "px";   // CSS 논리 픽셀
canvas.style.height = logicalHeight + "px";

// 3. WASM 초기화 (DPR 배율 포함)
IR2.initScreen(width * DPR, height * DPR, dpi, locale);

// 4. 렌더링: WASM → RGBA 비트맵 → Canvas
ptr = IR2.getScreenBuffer();
rgbaData = new Uint8ClampedArray(IR2.HEAPU8.buffer, ptr, w * h * 4);
imageData = new ImageData(rgbaData, w, h);
context2d.putImageData(imageData, 0, 0);

// 5. 마우스 좌표: 화면 → WASM (DPR 곱셈)
IR2.hidAction(ACTION_DOWN, x * DPR, y * DPR, ...);
```

**특징**:
- JS 측에서는 `fillText` 등 Canvas 드로잉 API를 일절 사용하지 않음
- 모든 렌더링(텍스트, 도형, 이미지)을 WASM 내부에서 처리
- WASM에 FreeType/HarfBuzz가 내장되어 있어 19MB 크기
- 품질은 높으나 WASM 크기가 비실용적

**우리에게 적용 불가 사유**: WASM 크기 19MB vs 우리 1.4MB. FreeType/HarfBuzz를 내장하면 크기가 급격히 증가.

### 2.3 한컴 웹기안기 (서버 JSON + Canvas fillText)

**아키텍처**: HWP → 서버 "HWP 필터" → JSON 렌더링 커맨드 (LZ-String 압축) → 클라이언트 Canvas 2D

#### 2.3.1 DPR 처리

```javascript
// DPR 변수
this.Bs3 = window.devicePixelRatio || 1;

// 메인 캔버스 크기 설정
canvas.width = Math.floor(logicalWidth * this.Bs3);     // 물리 픽셀
canvas.height = Math.floor(logicalHeight * this.Bs3);
canvas.style.width = logicalWidth + "px";                 // CSS 논리 픽셀
canvas.style.height = logicalHeight + "px";

// 렌더링 전 ctx.scale 적용
ctx.save();
ctx.scale(this.Bs3, this.Bs3);     // 이후 모든 좌표는 논리 단위
ctx.clearRect(0, 0, logicalWidth, logicalHeight);
ctx.restore();

// 오버레이 캔버스도 동일
overlayCanvas.width = Math.floor(logicalWidth * this.Bs3);
overlayCanvas.height = Math.floor(logicalHeight * this.Bs3);
overlayCtx.scale(this.Bs3, this.Bs3);

// 오프스크린 타일 캔버스도 DPR 적용
tileCanvas = new Canvas(Math.floor(tileW * this.Bs3), Math.floor(tileH * this.Bs3));
tileCtx.scale(this.Bs3, this.Bs3);
```

#### 2.3.2 렌더링 커맨드 디스패치

서버가 보내는 JSON 배열의 첫 번째 원소가 커맨드 타입:

| 타입 코드 | 상수 | 용도 | Canvas 함수 |
|-----------|------|------|-------------|
| 0 | tft | 텍스트 | Q13 → J13 (fillText) |
| 1 | ift | 이미지 배경 | Zn3 (createPattern) |
| 2 | eft | 채우기 | Zn3 (fillRect/gradient) |
| 3 | nft | 테두리 | an3 (lineTo/stroke) |
| 11-16 | rft~aft | 선 | qNt, zNt (moveTo/lineTo) |
| 21-25 | fft~wft | 도형 | ns3+Zn3+lOt (path/fill/stroke) |
| 26 | dft | 이미지 | Wn3 (drawImage) |
| 27 | bft | 그룹 객체 | es3 (clip+재귀) |
| 31-32 | Cft/vft | 클리핑 | save/clip/restore |
| 33-34 | _ft/yft | 오프스크린 | 임시 캔버스→drawImage |
| 130 | Fft | 캐럿/커서 | Kr3 |

#### 2.3.3 텍스트 렌더링 (핵심 함수 J13)

**글자별 fillText 호출** — 한 글자씩 위치/폰트/장평을 개별 설정:

```javascript
J13: function(ctx, x, y, textCmd, zoom) {
  var fonts = textCmd.fonts;       // 글자 배열
  var lineHeight = textCmd.height * zoom;

  // 텍스트 색상
  ctx.fillStyle = textCmd.color;   // "#rrggbb" (CREF 변환)

  for (var i = 0; i < fonts.length; i++) {
    var char = fonts[i].char;
    if (char == " ") continue;

    var fontSize = fonts[i].fontSize * zoom;  // 글자 크기
    var hScale = fonts[i].hScale;             // 장평 (0.5~2.0)
    var dx = fonts[i].dx * zoom;              // 글자간격
    var dt = fonts[i].dt * zoom;              // 시작 오프셋
    var vPos = fonts[i].position * zoom;      // 수직 위치

    // CSS 폰트 문자열 조합
    var fontStr = fontStyle + " " + fontSize + "px " + fontFamily;
    // fontFamily = "'함초롬바탕','fallback1','fallback2'"

    ctx.save();
    ctx.scale(hScale, 1);                     // 장평 적용
    ctx.translate(-(charX - charX / hScale), 0);

    if (ctx.font != fontStr) ctx.font = fontStr;
    ctx.textBaseline = "alphabetic";

    ctx.fillText(char, charX, charY);         // 한 글자 렌더링
    ctx.restore();

    // 밑줄/취소선 등 텍스트 장식
    if (textCmd.decoration) {
      drawDecoration(ctx, charX, charY, ...);
    }
  }
}
```

**텍스트 렌더링 전 설정되는 Canvas 속성**:
- `ctx.font` = `"[bold] [italic] <size>px '<fontFamily>','fallback1','fallback2'"`
- `ctx.fillStyle` = `"#rrggbb"` (CREF 색상)
- `ctx.textBaseline` = `"alphabetic"` | `"bottom"` | `"hanging"` | `"middle"`
- `ctx.scale(hScale, 1)` — 장평(가로 비율) 적용
- `ctx.globalCompositeOperation` = `"destination-out"` (지우개 모드)

**텍스트 변형 지원**:
- 일반 텍스트: `fillText` 1회
- 윤곽선 텍스트: `fillText` + `strokeText` 조합
- 3D 엠보스: 3패스 (밝은색→어두운색→본색) 오프셋
- 3D 음각: 3패스 (어두운색→밝은색→본색) 오프셋
- 그림자: 별도 그림자 패스 후 본 텍스트

#### 2.3.4 폰트 처리

**폰트 패밀리 매핑** (XTt 함수):
```javascript
// 한글 폰트 별칭 테이블
qTt: {
  "HY헤드라인M": "'HYHeadLine M','HYHeadline medium','HYHeadline'",
  "HY궁서B": "'HYGungSo B','HYGungSo black','HYGungSo'",
  // ... 다수
}

// 폰트 패밀리 문자열 생성
XTt: function(fontName, isSpecial, isSymbol) {
  var alias = qTt[fontName.toUpperCase()];
  return alias
    ? "'" + fontName + "'," + alias + "," + fallbacks
    : "'" + fontName + "'," + fallbacks;
}
```

**웹폰트 동적 로딩** (FontFace API, Chrome만):
```javascript
fontRegistry = {
  "HY헤드라인M":  { url: baseUrl + "/hygtre.woff2" },
  "HY견고딕":     { url: baseUrl + "/hygtre.woff2" },
  "SpoqaHanSans": { url: baseUrl + "/SpoqaHanSans-Regular.woff2" }
};

// 지연 로딩
new FontFace(fontName, "url(" + fontUrl + ")")
  .load()
  .then(function() { triggerRerender(); });
```

**폰트 메트릭 (임베디드 .hft 파일)**:
- 글리프 폭 테이블을 JS 모듈로 임베디드
- 코드포인트 범위별 폭 배열 (emsize 1000 or 1024 기준)
- 레이아웃 계산에 사용 (서버 측)

**폰트 크기 측정**:
```javascript
// 숨김 span 방식
span.style.fontFamily = fontFamily;
span.style.fontSize = "400pt";
span.textContent = char;
width = span.getBoundingClientRect().width / 40;

// Canvas measureText 방식
ctx.font = "1000pt " + fontFamily;
width = ctx.measureText(char).width / 100;
```

#### 2.3.5 이미지 렌더링

```javascript
// drawImage로 이미지 렌더링 (크롭/회전/미러/그림자 지원)
Wn3: function(ctx, x, y, w, h, imageCmd, zoom) {
  var img = imageCmd.htmlImage;   // HTMLImageElement
  x *= zoom; y *= zoom; w *= zoom; h *= zoom;

  // 크롭 → 임시 캔버스
  // 미러 → ctx.transform(-1,0,0,1,...) 또는 ctx.transform(1,0,0,-1,...)
  // 회전 → ctx.translate(center) + ctx.rotate(angle)
  // 그림자 → 다중 패스 shadowBlur
  // 기울기 → ctx.transform(1, tan(a), tan(b), 1, ...)

  ctx.drawImage(processedImage, x, y);
}
```

**이미지 효과 파이프라인**:
1. 크롭 (Xkt, Jkt 속성)
2. 미러 (fkt=가로, pkt=세로)
3. 회전 (rotationAngle)
4. 그림자 (10패스 반복 shadowBlur)
5. 기울기 (transform 행렬)
6. 색상 필터 (getImageData/putImageData 픽셀 조작)

#### 2.3.6 선/테두리 렌더링

**HWP 선 유형 디스패치**:
```javascript
switch (lineType) {
  case SOLID: case DASH: case DOT:
    drawSimpleLine(ctx, ...);     // moveTo→lineTo→stroke
    break;
  case DOUBLE: case TRIPLE:
    drawMultiLine(ctx, ...);      // 2-3회 평행선
    break;
  case THICK_THIN:
    drawThickThin(ctx, ...);      // 두께 다른 이중선
    break;
  case WAVE:
    drawWaveLine(ctx, ...);       // 물결선
    break;
}
```

**대시 패턴** (선 너비 `e`에 비례):
```javascript
DASH:      [12*e, 2*e]
DOT:       [1.42*e, 2.01*e]
DASH_DOT:  [16*e, 4*e, 1.4*e, 4*e]
DASH_DOT_DOT: [16*e, 4*e, 1.4*e, 4*e, 1.4*e, 4*e]
LONG_DASH: [24*e, 8*e]
```

**표 셀 테두리** (4변 독립):
```javascript
// 좌/우/상/하 각각 다른 선 유형/두께/색상
drawBorder(ctx, LEFT,   x1, y1+topW/2, x1, y2-botW/2, leftStyle);
drawBorder(ctx, RIGHT,  x2, y1+topW/2, x2, y2-botW/2, rightStyle);
drawBorder(ctx, TOP,    x1-leftW/2, y1, x2+rightW/2, y1, topStyle);
drawBorder(ctx, BOTTOM, x1-leftW/2, y2, x2+rightW/2, y2, bottomStyle);
```

#### 2.3.7 배경/채우기 렌더링

```javascript
// 단색 채우기
ctx.fillStyle = color;
ctx.fillRect(x, y, w, h);

// 그라디언트
var gradient = ctx.createLinearGradient(x1, y1, x2, y2);
gradient.addColorStop(0, startColor);
gradient.addColorStop(1, endColor);
ctx.fillStyle = gradient;
ctx.fillRect(x, y, w, h);

// 패턴 이미지
var pattern = ctx.createPattern(image, "repeat");  // repeat, repeat-x, repeat-y
ctx.fillStyle = pattern;
ctx.fillRect(x, y, w, h);

// 투명도
ctx.globalAlpha = 1 - transparency / 100;
```

#### 2.3.8 클리핑 및 오프스크린 합성

```javascript
// 클리핑 (Cft → save/clip, vft → restore)
ctx.save();
ctx.beginPath();
ctx.rect(clipX, clipY, clipW, clipH);
ctx.clip();
// ... 렌더링 ...
ctx.restore();

// 오프스크린 합성 (_ft → 생성, yft → 합성)
offscreen = createCanvas(width, height);
offCtx = offscreen.getContext("2d");
// ... 오프스크린에 렌더링 ...
mainCtx.drawImage(offscreen, 0, 0);   // 메인에 합성
```

#### 2.3.9 모듈 구조

| 모듈 | 역할 |
|------|------|
| hc_k | 상수 정의 ($at 커맨드 열거형) |
| hc_mG | Canvas 래퍼 클래스 (Yr3=캔버스, Vr3=ctx, _Nt=줌) |
| hc_mH | 페이지 렌더러: DPR 설정, 디스패치 루프, 타일 관리 |
| hc_e$ | 텍스트 렌더링: J13, Q13, z13 (fillText 핵심) |
| hc_fe | 채우기/배경: Zn3, qn3, Qn3, $n3 |
| hc_ff | 도형 폴리곤 드로잉: lOt |
| hc_fd | 이미지 드로잉: Wn3, Un3 + 효과 파이프라인 |
| hc_fc | 이미지 픽셀 효과: 회색조, 색상 필터 |
| hc_fa~hc_fg | 채우기 세부: 단색, 그라디언트, 해칭, 이미지 패턴 |
| hc_fh | 테두리 드로잉 |
| hc_fj~hc_mr | 임베디드 폰트 메트릭 (.hft) |
| hc_mx~hc_mB | UI 위젯 렌더링 |

## 3. 핵심 발견

### 3.1 한컴도 Canvas fillText를 사용한다

한컴 웹기안기가 서버 기반이지만, 클라이언트에서 텍스트를 렌더링하는 방식은 **Canvas 2D `fillText`**이다. 이는 구글 독스와 동일한 접근법이다. 즉:

- WASM에 FreeType을 내장하지 않아도 됨
- 브라우저의 텍스트 렌더링 엔진(ClearType/CoreText/FreeType)을 활용
- DPR 스케일링만 올바르게 적용하면 선명한 텍스트 렌더링 가능

### 3.2 DPR 스케일링이 선명도의 핵심

세 제품 모두 동일한 DPR 패턴을 사용:
```
canvas.width  = logicalWidth  × DPR    (물리 픽셀)
canvas.style.width  = logicalWidth + "px"  (CSS 논리 픽셀)
ctx.scale(DPR, DPR)                       (좌표 자동 변환)
```

### 3.3 글자별 개별 렌더링

한컴은 텍스트를 **한 글자씩** `fillText`로 그린다. 이유:
- 글자별 장평(가로 비율) 적용: `ctx.scale(hScale, 1)`
- 글자별 위치/오프셋 개별 지정
- HWP의 글자간격(charSpacing)이 글자마다 다를 수 있음

### 3.4 줌은 좌표 곱셈으로 처리

줌 배율은 모든 좌표에 `× zoom`을 곱하여 처리. CSS transform이 아닌 **Canvas 좌표 스케일링**.

## 4. 우리 제품(rhwp)의 현재 상태

### 4.1 현재 렌더링 파이프라인

```
HWP → Rust 파서 → 렌더 트리 → WASM renderPageToCanvas() → Canvas 2D
```

- Rust `WebCanvasRenderer`가 Canvas 2D API를 직접 호출
- `fillText`로 텍스트 렌더링 (한컴과 동일)
- scale 파라미터 있음 (타스크 123에서 추가)
- **DPR 미적용** → 고해상도 디스플레이에서 흐릿함

### 4.2 현재 줌 처리

```typescript
// canvas-view.ts — CSS 스케일링 방식
if (zoom !== 1.0) {
  canvas.style.width = `${canvas.width * zoom}px`;
  canvas.style.height = `${canvas.height * zoom}px`;
} else {
  canvas.style.width = '';
  canvas.style.height = '';
}
```

- CSS `width/height`로 확대/축소 → 래스터 늘림 → 흐릿함
- 타스크 123에서 WASM에 scale 파라미터는 추가했으나 JS 측 미적용

## 5. rhwp 적용 전략

한컴/구글독스의 검증된 접근법을 우리 제품에 적용한다.

### 5.1 전략 개요 — "DPR은 WASM이, 줌은 즉시CSS+지연WASM"

```
┌────────────────────────────────────────────────┐
│  목표 렌더링 수식                                │
│                                                │
│  canvas.width  = pageWidth  × zoom × DPR       │
│  canvas.height = pageHeight × zoom × DPR       │
│  canvas.style.width  = pageWidth  × zoom + "px"│
│  canvas.style.height = pageHeight × zoom + "px"│
│  ctx.scale(zoom × DPR, zoom × DPR)             │
│  → 이후 모든 렌더링 좌표는 문서 단위(HWPUNIT→px) │
└────────────────────────────────────────────────┘
```

### 5.2 레이어별 변경 사항

#### Rust WASM 레이어

| 파일 | 변경 | 비고 |
|------|------|------|
| `src/wasm_api.rs` | scale 최대값 8.0 → 12.0 | zoom 3.0 × DPR 3.0 = 9.0 지원 |
| `src/renderer/web_canvas.rs` | 변경 없음 | `ctx.scale(scale)` 이미 올바르게 동작 |

WASM은 `scale = zoom × DPR`을 그대로 받으면 된다. 이미 `set_scale()` → `ctx.scale()` 파이프라인이 정상 동작 중.

#### TypeScript 레이어

**`page-renderer.ts`** — 핵심 변경:
```typescript
// [현재] scale 파라미터 미전달
this.wasm.renderPageToCanvas(pageIdx, canvas);

// [변경] zoom × DPR 전달
const dpr = window.devicePixelRatio || 1;
const scale = zoom * dpr;
this.wasm.renderPageToCanvas(pageIdx, canvas, scale);
```

**`canvas-view.ts` renderPage()** — CSS 크기 설정:
```typescript
// [현재] CSS로 줌 (래스터 늘림)
canvas.style.width = `${canvas.width * zoom}px`;

// [변경] WASM이 zoom×DPR로 렌더링한 캔버스를 CSS로 표시
// canvas.width = pageWidth × zoom × DPR (WASM이 설정)
// CSS 표시 크기 = pageWidth × zoom
const dpr = window.devicePixelRatio || 1;
canvas.style.width  = `${canvas.width / dpr}px`;
canvas.style.height = `${canvas.height / dpr}px`;
```

**`page-renderer.ts` drawMarginGuides()** — DPR 보정:
```typescript
// [현재] 좌표를 그대로 사용
ctx.moveTo(left, top - L);

// [변경] WASM이 이미 ctx.scale(zoom×DPR)을 적용했으므로
// drawMarginGuides는 WASM 렌더링 후에 별도로 그리므로
// ctx.setTransform으로 동일 스케일 적용 필요
const scale = zoom * dpr;
ctx.setTransform(scale, 0, 0, scale, 0, 0);
```

**`canvas-view.ts` onZoomChanged()** — 즉시CSS + 지연WASM:
```typescript
// 1단계: 즉시 CSS 스케일로 빠른 피드백 (150ms 미만)
const cssRatio = newZoom / oldZoom;
for (const canvas of activeCanvases) {
  canvas.style.width  = `${parseFloat(canvas.style.width) * cssRatio}px`;
  canvas.style.height = `${parseFloat(canvas.style.height) * cssRatio}px`;
}

// 2단계: 디바운스 후 WASM 벡터 재렌더링 (150~300ms)
this.scheduleVectorRerender(newZoom);
```

이 "즉시CSS + 지연WASM" 패턴은 한컴 웹기안기도 동일하게 사용한다.

### 5.3 오버레이 캔버스 동기화

선택/캐럿 오버레이 캔버스에도 동일한 DPR 패턴 적용:

```typescript
overlay.width  = logicalWidth  * zoom * dpr;
overlay.height = logicalHeight * zoom * dpr;
overlay.style.width  = logicalWidth  * zoom + "px";
overlay.style.height = logicalHeight * zoom + "px";
overlayCtx.scale(zoom * dpr, zoom * dpr);
```

### 5.4 마우스 좌표 변환

```
screen 좌표 → document 좌표:
  docX = (event.clientX - canvasRect.left) / zoom
  docY = (event.clientY - canvasRect.top)  / zoom

  * DPR은 CSS가 처리하므로 JS에서는 zoom만 나눈다
  * 한컴도 동일한 방식 (좌표 ÷ zoom, DPR은 Canvas/CSS가 흡수)
```

### 5.5 가상 스크롤 레이아웃

`virtual-scroll.ts`는 **변경 없음**. 이유:
- 가상 스크롤은 CSS 표시 크기(논리 크기 × zoom)로 레이아웃 계산
- DPR은 캔버스 내부 해상도에만 관여하고, CSS 레이아웃에는 영향 없음

### 5.6 변경 파일 요약

| 파일 | 변경 내용 | 난이도 |
|------|-----------|--------|
| `src/wasm_api.rs` | scale 최대값 조정 (8→12) | 낮음 |
| `rhwp-studio/src/view/page-renderer.ts` | zoom×DPR 전달, margin guides DPR 보정 | 중간 |
| `rhwp-studio/src/view/canvas-view.ts` | CSS 크기 계산 변경, 디바운스 줌 | 중간 |
| `rhwp-studio/src/view/viewport-manager.ts` | DPR 감지/저장/이벤트 | 낮음 |
| `rhwp-studio/src/ui/selection-overlay.ts` (해당 시) | 오버레이 DPR 동기화 | 중간 |

### 5.7 기대 효과

| 항목 | 현재 | 적용 후 |
|------|------|---------|
| 100% 줌 선명도 | DPR 1x 렌더링 (고해상도 디스플레이에서 흐릿) | DPR 배율 렌더링 (선명) |
| 200% 줌 선명도 | CSS 2배 늘림 (픽셀 보임) | 2×DPR 해상도 재렌더링 (선명) |
| 줌 반응 속도 | 전체 재렌더링 대기 | 즉시 CSS → 지연 벡터 (부드러운 UX) |
| WASM 크기 | 1.4MB (변화 없음) | 1.4MB (변화 없음) |

### 5.8 한컴과의 아키텍처 비교

```
[한컴 웹기안기]
  HWP → 서버(HWP필터) → JSON 커맨드 → JS Canvas 2D fillText
  DPR: ctx.scale(DPR) + 캔버스 크기 × DPR

[rhwp (적용 후)]
  HWP → Rust WASM 파서 → 렌더트리 → Rust Canvas 2D fillText
  DPR: ctx.scale(zoom×DPR) + 캔버스 크기 × zoom × DPR

  * 차이점: 한컴은 JS에서 fillText 호출, rhwp는 Rust/WASM에서 호출
  * 공통점: 둘 다 브라우저 Canvas 2D fillText 사용, DPR 스케일링 동일
  * 장점: 서버 불필요, WASM 렌더링으로 성능 우위
```
