---
kind: investigation
status: historical
canonical: mydocs/tech/webhwp/README.md
last_verified: 2026-07-17
---

# webhwp 분석: 표(Table) 컨트롤

> 분석 대상: `webhwp/js/hwpApp.*.chunk.js` (5.17MB minified)
> 분석 일자: 2026-02-09

## 1. 표 컨트롤 식별

| 항목 | 값 |
|------|-----|
| 컨트롤 ID | `wt.Z.gfi` |
| 리소스 이름 | `IDS_CTRL_NAME_TABLE` |
| 참조 횟수 | 120회 |
| 플래그 | `hfi: 0` |

## 2. 셀 데이터 모델 (qUn)

```javascript
// 셀 데이터 구조
{
    JUn: rowIndex,        // 행 주소
    ZUn: colIndex,        // 열 주소
    $Un: cellWidth,       // 셀 폭 (-1 = 자동)
    tWn: cellHeight,      // 셀 높이 (-1 = 자동)
    Bun: [colAddr, rowAddr],  // 주소 배열
    Fun: [colSpan, rowSpan],  // 병합 범위 배열
    iWn: [],              // 셀 내용 (텍스트/HTML)
    nWn: [],              // 플래그 배열
    eWn: [],              // 스타일 속성 배열
    rWn: [],              // 스타일 값 배열
    MUn: measuredWidth    // 레이아웃 후 측정된 폭
}
```

### 셀 접근 패턴

```javascript
r.Chn(h, u)   // 행 h, 셀 u 접근
r.Phn(h)      // 행 h의 셀 개수
v.Bun[0]      // 열 주소
v.Bun[1]      // 행 주소
```

## 3. 행/열 메타데이터

### 열 메타데이터 (LUn)

```javascript
{
    columnWidth: -1,    // 열 폭 (-1 = 자동/미설정)
    MUn: measuredWidth, // 측정된 폭
    BUn: extraWidth,    // 테두리에 의한 추가 폭
    FUn: rowSpanAccum   // 행 병합 누적값
}
```

### 행 메타데이터 (UUn)

```javascript
{
    WUn: -1,            // 행 높이 (-1 = 자동/미설정)
    GUn: widthAdj,      // 폭 조정값
    KUn: extraHeight    // 추가 높이
}
```

### 표 전체 구조 (YUn)

```javascript
{
    jUn: totalHeight,   // 표 전체 높이
    gUn: totalWidth,    // 표 전체 폭
    VUn: colCount,      // 열 개수
    zUn: rowCount       // 행 개수
}
```

## 4. 셀 병합 처리

### 병합 감지

```javascript
rowSpan = v.Fun[1];
colSpan = v.Fun[0];
// 단일 셀: 1==s.colSpan && s.$Un > 0
// 병합 셀: 1==s.colSpan && s.$Un > i.QUn[s.ZUn].columnWidth
```

### 병합 셀 크기 계산

```javascript
// 병합된 열의 폭 합산
for (let n = 0; n < e.colSpan; n++) {
    if (e.ZUn + n < i.VUn)
        t += i.QUn[e.ZUn + n].columnWidth;
}
e.$Un = t;
```

- `M1n()` 함수로 병합 셀 차원 계산
- 이미 렌더링된 병합 셀은 건너뜀

## 5. 테두리(Border) 렌더링

### 테두리 유형 (mt 상수)

| 상수 | 유형 | 대시 패턴 |
|------|------|----------|
| `mt.NEt` | 실선 (Solid) | 없음 |
| `mt.kEt` | 파선 (Dashed) | `[2, 1.2]` |
| `mt.HEt` | 이중 파선 | `[10, 1.2, 2, 1.2]` |
| `mt.PEt` | 삼중 파선 | `[10, 1.2, 2, 1.2, 2, 1.2]` |
| `mt.xEt` | 특수선 | — |
| `mt.OEt` | 기본선 | — |

### 테두리 속성

```javascript
{
    uqt: borderType,     // 선 유형 (NEt, kEt 등)
    rqt: linePattern,    // 선 스타일 패턴
    strokeStyle: color,  // RGB 색상
    lineWidth: width,    // 두께 (px)
    ec: capStyle,        // "round" / "square"
    lw: lineWidthValue,  // 선 폭 값
    lc: lineColor,       // CREF 형식 색상
    lt: lineType         // 선 종류
}
```

### 렌더링 함수

| 함수 | 역할 |
|------|------|
| `bbr()` | 기본 선 그리기 (`moveTo/lineTo/stroke` + `setLineDash`) |
| `Cbr()` | 모서리 테두리 (클리핑 영역 사용) |
| `Tbr()` | 범용 테두리 디스패처 (좌표 변환 + 줌 적용 후 bbr/Cbr 호출) |
| `gbr()` | 이중선 모서리 처리 |
| `xbr()` | 삼중선 처리 |

### 모서리 방향 상수

| 상수 | 방향 |
|------|------|
| `mt.OCt` | 위쪽 (Top) |
| `mt.kCt` | 아래쪽 (Bottom) |
| `mt.ACt` | 왼쪽 (Left) |
| `mt.RCt` | 오른쪽 (Right) |

## 6. 셀 배경 채우기

### 단색 채우기

```javascript
t.fillStyle = Lo.CREFtoRGB(h.NJt, "#");  // HWP 색상 → RGB 변환
t.fillRect(o, a, f, l);                   // 배경 칠하기
```

### 패턴 채우기

```javascript
// 8가지 패턴 유형 (e[mt.*] 캔버스)
e[mt.eyt]  // 격자 패턴 (5,5 교차선)
// 패턴 적용
t.fillStyle = t.createPattern(patternCanvas, "repeat");
```

### 패턴 색상 변환 (Ubr)

```javascript
Ubr: function(patternType, colorRef) {
    var canvas = e[patternType];
    var ctx = canvas.getContext("2d");
    ctx.globalCompositeOperation = "source-in";
    ctx.fillStyle = CREFtoRGB(colorRef, "#");
    ctx.fillRect(-1, -1, canvas.width+1, canvas.height+1);
    return canvas;
}
```

## 7. 셀 정렬

```javascript
// 수평 정렬
align: "" | "left" | "center" | "right"
textAlign: r.TIr  // "center", "left"

// 수직 정렬
vAlign: "top" | "middle" | "bottom"
textBaseline: r.vAlign
```

## 8. 표 레이아웃

### 폭 모드

```javascript
// 고정 폭 (gUn > 0)
if (i.gUn > 0) {
    // 비례 배분: (i.gUn - h) / (i.VUn - u)
}

// 자동 폭 (gUn <= 0)
// 셀 내용으로부터 측정된 columnWidth 사용
```

### 높이 모드

```javascript
// 고정 높이 (jUn > 0)
// 배분: (i.jUn - a) / (i.zUn - t)

// 자동 높이 (jUn <= 0)
// 셀 내용 + 패딩에서 계산
```

### 열 폭 배분 알고리즘 (M1n)

```javascript
// 1. 각 열의 columnWidth 합산
for (let t = 0; t < i.VUn; t++) {
    if (i.QUn[t].columnWidth > 0) { h += ...; u++; }
}

// 2. 총 폭과 차이 조정
if (i.gUn != h) {
    for (let t = 0; t < i.VUn; t++) {
        i.QUn[t].columnWidth = nzn(columnWidth, gUn, h);
    }
}

// 3. 병합 셀 폭 재계산
for (let n = 0; n < e.colSpan; n++) {
    t += i.QUn[e.ZUn + n].columnWidth;
}
e.$Un = t;
```

### 관련 함수

| 함수 | 역할 |
|------|------|
| `M1n()` | 최소 폭/높이 계산 |
| `B1n()` | 공간 배분 (다중 스팬 셀) |
| `K1n()` | 희소 테이블 빈 셀 채우기 |

## 9. 표 편집

### 셀 분할 (SplitCell)

```javascript
SplitCell(preview, params) {
    // 1. 편집 가능 여부 확인 (ZRe())
    // 2. 셀 참조 (nse())
    // 3. 분할 파라미터 읽기
    // 4. 셀 분할 마킹
    // 5. 레이아웃 재계산
}
```

### 셀 병합 (MergeCell)

```javascript
MergeCell() {
    // 1. 선택된 셀 범위 확인
    // 2. 직사각형 선택 검증
    // 3. rowSpan/colSpan 결합
    // 4. QUn 매트릭스 갱신
}
// 활성화 체크: t.vti.yJn().MergeCell(true, null)
```

### 행/열 삽입/삭제

```javascript
// 행 삽입
type: Jw (ROW_INSERT)
value: { rowIndex: insertPosition }

// 열 삽입
type: Kw (COL_INSERT)
value: { colIndex: insertPosition }

// 행 삭제: type: $w (ROW_DELETE)
// 열 삭제: type: Lw (COL_DELETE)

// 균등 배분
equalTableRow   // 높이 균등 배분
equalTableCol   // 폭 균등 배분
```

### 메뉴 항목

```javascript
table: [
    "insertRow", "insertColumn",
    "tRowInsert", "tCellInsert",
    "tRowDelete", "tCellDelete",
    "insertCell", "deleteCell",
    "mergeCell", "splitCell",
    "deleteTable", "selectTable",
    "alignCell", "borderCell",
    "tableLine", "selectCell"
]
```

## 10. rhwp와의 비교

| 항목 | webhwp | rhwp |
|------|--------|------|
| 렌더링 | Canvas 2D (lineTo, fillRect) | Canvas 2D (fillRect, strokeRect) |
| 테두리 유형 | 6+ 유형 (실선, 파선, 이중 등) | 기본 실선 위주 |
| 패턴 채우기 | createPattern + 8가지 패턴 | 미구현 |
| 셀 병합 | rowSpan/colSpan + M1n 계산 | rowSpan/colSpan 지원 |
| 편집 | 분할/병합/삽입/삭제 전체 지원 | 미구현 |
| 폭 배분 | 고정/자동 + 비례 배분 | 고정 폭만 |
| 중첩 표 | 셀 내용(iWn)에 포함 가능 | 미구현 |
| 색상 변환 | `CREFtoRGB()` | `ColorRef` → hex |

---

*분석 일자: 2026-02-09*
