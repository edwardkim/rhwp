---
kind: investigation
status: historical
canonical: mydocs/tech/webhwp/README.md
last_verified: 2026-07-17
---

# webhwp 분석: 도형(Shape) 및 이미지 컨트롤

> 분석 대상: `webhwp/js/hwpApp.*.chunk.js` (5.17MB minified)
> 분석 일자: 2026-02-09

## 1. 도형 컨트롤 유형

| 컨트롤 ID | 유형 | 참조 수 | 설명 |
|-----------|------|---------|------|
| `wt.Z.Sfi` | GEN_SHAPE_OBJECT | 160 | 모든 그리기 객체의 메인 컨테이너 |
| `wt.Z.gfi` | Container | 120 | 그룹/컨테이너 (표도 여기 포함) |
| `wt.Z.Tfi` | EQEDIT | 62 | 수식 객체 |
| `wt.Z.Wli` | Image/Picture | 74 | 이미지/그림 렌더링 |
| `wt.Z.Yli` | TEXTART | 24 | 글맵시 (텍스트 아트) |
| `wt.Z.zli` | VIDEO | 35 | 동영상 컨트롤 |
| `wt.Z.Kli` | OLE/Embedded | 50 | OLE 임베디드 객체 |
| `wt.Z.Nfi` | Nested Container | 41 | 중첩 도형 계층 |

### 컨트롤 등록 패턴

```javascript
i[wt.Z.Sfi] = i[wt.Z.Sfi] || {
    efi: U.default.hri,           // 기본 핸들러
    vfi: "IDS_CTRL_NAME_GEN_SHAPE_OBJECT",
    rfi: wt.Z.Sfi,               // 컨트롤 ID
    hfi: Vt.Z.ffi.wfi,           // 핸들러 플래그
    ofi: null,                    // 옵션
    mfi: null,                    // 메타데이터
    dfi: ""                       // 정의
}
```

## 2. Canvas 그리기 연산

### 호출 횟수

| 메서드 | 횟수 | 용도 |
|--------|------|------|
| `lineTo` | 101 | 선분 그리기 |
| `arc` | 87 | 원형/호 도형 |
| `drawImage` | 61 | 이미지 렌더링 |
| `fillRect` | 11 | 사각형 채우기 |
| `save` | 186 | 상태 저장 |
| `restore` | 133 | 상태 복원 |
| `translate` | 40 | 위치 변환 |
| `scale` | 25 | 크기 변환 |
| `rotate` | 7 | 회전 변환 |

## 3. 이미지 렌더링

### getBase64Image() — 이미지 Base64 변환

```javascript
getBase64Image: function(imageElement) {
    var canvas = document.createElement("canvas");
    var ctx = canvas.getContext("2d");
    var w = Math.max(1, imageElement.naturalWidth);
    var h = Math.max(1, imageElement.naturalHeight);

    // data:URL이면 직접 사용
    if (imageElement.src.match(/^data:image\/(png|jpg);base64,/))
        return imageElement.src;

    // 최대 1024px로 축소
    var scaledW = w, scaledH = h;
    if (scaledW > 1024) {
        var ratio = 1024 / scaledW;
        scaledW *= ratio; scaledH *= ratio;
    }

    // 2MB 이하까지 반복 축소
    var quality = 1;
    do {
        ctx.drawImage(imageElement, 0, 0, w, h, 0, 0, canvas.width, canvas.height);
        var dataUrl = canvas.toDataURL();
        quality -= 0.2;
        canvas.width = scaledW * quality;
        canvas.height = scaledH * quality;
    } while (dataUrl.length > 2e6 && quality > 0);

    return dataUrl.replace(/^data:image\/(png|jpg);base64,/, "");
}
```

### drawImage 호출 패턴

```javascript
// 기본 drawImage (소스 → 대상)
context.drawImage(src, srcX, srcY, srcW, srcH, dstX, dstY, dstW, dstH);

// 투명도 적용
context.globalAlpha = opacity;
context.drawImage(imageElement, x, y, width, height);
context.globalAlpha = 1.0;
```

### 지원 이미지 형식

- PNG (81 참조)
- GIF (33 참조)
- JPEG (8 참조)
- BMP (5 참조)
- WMF/EMF (Canvas 변환 통해 지원)

## 4. 채우기 스타일

### 단색 채우기

```javascript
context.fillStyle = solidColor;
context.fillRect(x, y, w, h);
```

### 그라디언트 채우기

```javascript
var gradient = context.createLinearGradient(x0, y0, x1, y1);
gradient.addColorStop(0, "rgba(0,0,0,0)");   // 투명
gradient.addColorStop(0.5, "red");
gradient.addColorStop(1, "red");
context.fillStyle = gradient;
context.fillRect(0, 0, width, height);
```

### 패턴 채우기 (6회 사용)

```javascript
var pattern = context.createPattern(image, "repeat");
// 모드: "repeat", "repeat-x", "repeat-y", "no-repeat"
context.fillStyle = pattern;
context.fill();
```

## 5. 선 스타일

### 대시 패턴 (23회 setLineDash 호출)

| 대시 유형 | 패턴 |
|----------|------|
| 장단 파선 | `[15*width, 4.5*width]` |
| 사용자 정의 | `[10, 1.2, 2, 1.2]` |
| 점선 | `[0.1, 2.5*lineWidth]` |
| 혼합 패턴 | `[15*w, 4.5*w, 1.5*w, 4.5*w]` |

### 선 끝/연결 스타일

```javascript
context.lineCap = "round" | "butt" | "square";   // 13회
context.lineJoin = "round" | "bevel" | "miter";   // 5회
context.lineWidth = width;                         // 80회
```

## 6. 그림자 효과

### 기본 그림자

```javascript
context.shadowColor = "#000000";
context.shadowOffsetX = offsetX;
context.shadowOffsetY = offsetY;
context.shadowBlur = blurRadius;
```

### 고급 그림자 (Xbr — 점진적 렌더링)

```javascript
Xbr: function(context, imageBuffer, size, alpha, shadowColor, offset) {
    const shadowSteps = Math.round(8 + size/200 * 17);
    const blurPerStep = size / shadowSteps;

    for (var step = 0; step < shadowSteps; step++) {
        context.globalAlpha = alpha * (step / shadowSteps);
        context.shadowBlur = blurPerStep * (1.5 * step);
        context.drawImage(imageBuffer, offset, offset);
    }
    context.drawImage(imageBuffer, offset, offset);
}
```

### 블러 그림자 (ryr 함수)

```javascript
ryr: function(context, blurRadius) {
    var shadowCanvas = document.createElement("canvas");
    var shadowCtx = shadowCanvas.getContext("2d");
    shadowCtx.translate(-shadowCanvas.width, 0);
    shadowCtx.shadowColor = "#000000";
    shadowCtx.shadowOffsetX = shadowCanvas.width;
    shadowCtx.shadowBlur = 1.4 * blurRadius;
    shadowCtx.fillRect(blurRadius, blurRadius,
                       width - 2.4*blurRadius,
                       height - 2.4*blurRadius);
    // destination-in 합성으로 마스킹
    context.globalCompositeOperation = "destination-in";
    context.drawImage(shadowCanvas, 0, 0);
}
```

## 7. 도형 속성

### 변환

```javascript
rotate()     // 7회 — 도형 회전
scale()      // 25회 — 크기 조절
translate()  // 40회 — 위치 이동
transform()  // 8회 — 행렬 변환

// 기울기(italic) 변환
context.setTransform(1, 0, -0.4, 1, 0, 0);
context.translate(6*baseSize, 0);
```

### 속성 코드

| 코드 | 횟수 | 속성 유형 |
|------|------|----------|
| `Pqt` | 72 | 위치/좌표 |
| `Uqt` | 30 | UI 속성 |
| `Nqt` | 22 | 수치 치수 |
| `Mqt` | 14 | 측정 속성 |
| `Gqt` | 7 | 기하학 속성 |

### Z-순서 (레이어링)

```javascript
shape.zOrder = 0;  // 뒤에서 앞으로 정렬
// 합성 연산
context.globalCompositeOperation = "destination-in";   // 클리핑
context.globalCompositeOperation = "destination-out";  // 마스킹
context.globalCompositeOperation = "source-in";        // 색상 오버레이
```

### 앵커링 (텍스트 흐름)

```javascript
anchor: "character"    // 문자 수준 (인라인)
anchor: "paragraph"    // 문단 수준
anchor: "page"         // 페이지 수준
```

## 8. 그룹 도형 계층

```
GEN_SHAPE_OBJECT (wt.Z.Sfi)
├── 자식 도형 1 (wt.Z.Sfi)
├── 자식 도형 2 (wt.Z.Sfi)
└── 컨테이너 (wt.Z.gfi)
    ├── 그룹 도형 1
    ├── 그룹 도형 2
    └── 그룹 도형 N
```

렌더링 순서:
1. Z-순서 기반 뒤→앞 레이어링
2. 클립 연산으로 마스킹
3. 도형 간 합성 모드 전환
4. 메인 콘텐츠 전에 그림자 사전 렌더링

## 9. 동영상 컨트롤 (wt.Z.zli)

```javascript
i[wt.Z.zli] = {
    efi: U.default.hri,
    vfi: "IDS_CTRL_NAME_VIDEO",
    rfi: wt.Z.zli,
    hfi: Vt.Z.ffi.wfi | Vt.Z.ffi.afi
}
```

| 함수 | 역할 |
|------|------|
| `zgn(doc, data, index)` | 동영상 렌더링 메인 |
| `Wln()` (15회) | 동영상 속성 핸들러 |
| `Eyn()` (15회) | 동영상 이벤트 처리 |

## 10. 글맵시 (TextArt, wt.Z.Yli)

```javascript
i[wt.Z.Yli] = {
    efi: U.default.hri,
    vfi: "IDS_CTRL_NAME_TEXTART",
    rfi: wt.Z.Yli,
    hfi: Vt.Z.ffi.wfi | Vt.Z.ffi.afi
}
```

- 일반 텍스트와 동일한 Canvas 렌더링 파이프라인 사용
- 특수 도형/곡선 효과를 글리프에 적용
- 그라디언트/패턴 채우기를 개별 문자에 적용
- 패스 기반 렌더링으로 아트 효과 구현
- 통합 그림자/효과 파이프라인

## 11. 색상 관리

```javascript
// HWP 색상 참조 → RGB 변환
CREFtoRGB: function(colorRef, format) {
    // colorRef: HWP CREF 형식
    // format: "#" → "#RRGGBB" 반환
    return "#RRGGBB";
}

// 사용 예
context.fillStyle = CREFtoRGB(shape.fillColor, "#");
```

## 12. rhwp와의 비교

| 항목 | webhwp | rhwp |
|------|--------|------|
| 도형 렌더링 | Canvas 2D (arc, lineTo, fillRect) | Canvas 2D (제한적) |
| 이미지 렌더링 | drawImage + Base64 최적화 | drawImage 기본 |
| 채우기 | 단색/그라디언트/패턴 3종 | 단색만 |
| 그림자 | 점진적 블러 + destination-in | 미구현 |
| 그룹 도형 | Z-순서 + 합성 모드 | 미구현 |
| 동영상 | zgn 핸들러 지원 | 미구현 |
| 글맵시 | 패스 기반 글리프 효과 | 미구현 |
| 변환 | rotate/scale/translate/transform 전체 | scale 부분 지원 |
| OLE 객체 | 임베디드 렌더링 지원 | 미구현 |

---

*분석 일자: 2026-02-09*
