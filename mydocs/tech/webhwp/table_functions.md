---
kind: investigation
status: historical
canonical: mydocs/tech/webhwp/README.md
last_verified: 2026-07-17
---

# 한컴 웹기안기 — 표(Table) 관련 JS 함수 목록

> 분석 대상: `webhwp/js/hwpApp.*.chunk.js` (5.17MB minified)
> 작성일: 2026-02-21
> 참조: [표 컨트롤 분석](02_table.md)

## 1. 테두리(Border) 렌더링

| 함수 | 역할 | Canvas API |
|------|------|-----------|
| `bbr()` | 기본 선 그리기. `moveTo`→`lineTo`→`stroke` + `setLineDash`로 대시 패턴 적용 | `moveTo`, `lineTo`, `stroke`, `setLineDash` |
| `Cbr()` | 모서리 테두리 렌더링. 클리핑 영역으로 교차점 처리 | `clip`, `stroke` |
| `Tbr()` | 범용 테두리 디스패처. 좌표 변환 + 줌 적용 후 `bbr()` 또는 `Cbr()` 호출 | — |
| `gbr()` | 이중선 모서리 처리. 2회 평행선 렌더 | `moveTo`, `lineTo`, `stroke` |
| `xbr()` | 삼중선 처리. 3회 평행선 렌더 | `moveTo`, `lineTo`, `stroke` |

### 테두리 선 유형 상수 (mt)

| 상수 | 유형 | 대시 패턴 |
|------|------|----------|
| `mt.NEt` | 실선 (Solid) | 없음 |
| `mt.kEt` | 파선 (Dashed) | `[2, 1.2]` |
| `mt.HEt` | 일점쇄선 (DashDot) | `[10, 1.2, 2, 1.2]` |
| `mt.PEt` | 이점쇄선 (DashDotDot) | `[10, 1.2, 2, 1.2, 2, 1.2]` |
| `mt.xEt` | 특수선 | — |
| `mt.OEt` | 기본선/없음 | — |

### 테두리 방향 상수

| 상수 | 방향 | 참조 횟수 |
|------|------|----------|
| `mt.OCt` | 위쪽 (Top) | 308 |
| `mt.kCt` | 아래쪽 (Bottom) | 257 |
| `mt.ACt` | 왼쪽 (Left) | 321 |
| `mt.RCt` | 오른쪽 (Right) | 257 |

### 테두리 속성 객체

```javascript
{
    uqt: borderType,     // 선 유형 (NEt, kEt 등)
    rqt: linePattern,    // 선 스타일 패턴
    strokeStyle: color,  // RGB 색상 (#rrggbb)
    lineWidth: width,    // 두께 (px)
    ec: capStyle,        // "round" | "square"
    lw: lineWidthValue,  // 선 폭 원본값
    lc: lineColor,       // CREF 형식 색상
    lt: lineType         // 선 종류
}
```

## 2. 셀 배경/채우기 렌더링

| 함수 | 역할 | Canvas API |
|------|------|-----------|
| `CREFtoRGB()` | HWP COLORREF → RGB hex 변환 (`#rrggbb`) | — |
| `Ubr()` | 패턴 색상 변환. `globalCompositeOperation = "source-in"`으로 패턴 캔버스에 색상 적용 | `globalCompositeOperation`, `fillRect` |
| (인라인) | 단색 채우기: `fillStyle = CREFtoRGB(color)` → `fillRect(x, y, w, h)` | `fillStyle`, `fillRect` |
| (인라인) | 패턴 채우기: `createPattern(patternCanvas, "repeat")` → `fillRect` | `createPattern`, `fillRect` |

### 패턴 유형 (8가지)

```javascript
e[mt.eyt]   // 격자 (5×5 교차선)
// 그 외: 가로줄, 세로줄, 역슬래시, 슬래시, 십자, X자, 점 패턴
```

## 3. 표 레이아웃 계산

| 함수 | 역할 | 설명 |
|------|------|------|
| `M1n()` | 열 폭 배분 | 각 열의 `columnWidth` 합산 → 총 폭(`gUn`)과의 차이 비례 조정 → 병합 셀 폭 재계산 |
| `B1n()` | 다중 스팬 공간 배분 | `colSpan`/`rowSpan > 1`인 셀의 공간을 관련 열/행에 분배 |
| `K1n()` | 희소 테이블 빈 셀 채우기 | 불규칙 테이블 구조에서 누락된 셀 생성 |
| `nzn()` | 비례 폭 계산 | `columnWidth × (targetWidth / totalWidth)` 비례 배분 |

### 폭/높이 모드

```javascript
// 고정 폭 (gUn > 0): 표 전체 폭이 지정됨
if (i.gUn > 0) {
    // 비례 배분: (gUn - 사용폭) / (VUn - 할당열수)
}

// 자동 폭 (gUn <= 0): 셀 내용 기반 측정
// columnWidth로부터 자동 계산

// 고정 높이 (jUn > 0): (jUn - 사용높이) / (zUn - 할당행수)
// 자동 높이 (jUn <= 0): 셀 내용 + 패딩
```

## 4. 셀 데이터 구조

### 셀 (qUn) — 223회 참조

| 속성 | 참조 | 설명 |
|------|------|------|
| `.JUn` | 23 | 행 인덱스 |
| `.ZUn` | 28 | 열 인덱스 |
| `.$Un` | 21 | 셀 폭 (-1 = 자동) |
| `.tWn` | 20 | 셀 높이 (-1 = 자동) |
| `.Bun` | 56 | 주소 배열 `[colAddr, rowAddr]` |
| `.Fun` | 23 | 병합 범위 `[colSpan, rowSpan]` |
| `.iWn` | 17 | 셀 내용 (텍스트/HTML) |
| `.nWn` | 14 | 플래그 배열 |
| `.eWn` | 19 | 스타일 속성 배열 |
| `.rWn` | 16 | 스타일 값 배열 |
| `.MUn` | 13 | 레이아웃 후 측정된 폭 |

### 열 메타데이터 (LUn/QUn) — 23회 참조

| 속성 | 설명 |
|------|------|
| `.columnWidth` | 열 폭 (-1 = 자동/미설정) |
| `.MUn` | 측정된 폭 |
| `.BUn` | 테두리에 의한 추가 폭 |
| `.FUn` | 행 병합 누적값 |

### 행 메타데이터 (UUn)

| 속성 | 참조 | 설명 |
|------|------|------|
| `.WUn` | 28 | 행 높이 (-1 = 자동) |
| `.GUn` | 5 | 폭 조정값 |
| `.KUn` | 4 | 추가 높이 |

### 표 전체 구조 (YUn)

| 속성 | 참조 | 설명 |
|------|------|------|
| `.jUn` | — | 표 전체 높이 |
| `.gUn` | 10 | 표 전체 폭 |
| `.VUn` | 24 | 열 개수 |
| `.zUn` | 16 | 행 개수 |

### 셀 접근 함수

| 함수 | 역할 |
|------|------|
| `r.Chn(h, u)` | 행 `h`, 셀 `u` 접근 |
| `r.Phn(h)` | 행 `h`의 셀 개수 반환 |

## 5. 셀 병합/분할

| 함수 | 역할 | 단축키 |
|------|------|--------|
| `MergeCell(t)` | 선택된 직사각형 셀 범위를 병합. `Fun[colSpan, rowSpan]` 갱신 → `QUn` 매트릭스 갱신 → 레이아웃 재계산 | Alt+M |
| `SplitCell(t, i)` | 병합된 셀을 분할. `ZRe()`로 편집 가능 여부 확인 → `nse()`로 셀 참조 → 분할 마킹 → 레이아웃 재계산 | Alt+S |
| `unMergeCell` | 이전 병합을 되돌리기 (MergeCell 역연산) | — |

### 병합 감지 로직

```javascript
rowSpan = v.Fun[1];
colSpan = v.Fun[0];
// 단일 셀: 1 == colSpan && $Un > 0
// 병합 셀: 1 == colSpan && $Un > QUn[ZUn].columnWidth

// 병합된 열의 폭 합산
for (let n = 0; n < e.colSpan; n++) {
    if (e.ZUn + n < i.VUn)
        t += i.QUn[e.ZUn + n].columnWidth;
}
e.$Un = t;
```

## 6. 행/열 삽입/삭제

| 함수 (메뉴 커맨드) | 역할 | 조작 타입 상수 |
|-------------------|------|---------------|
| `tRowInsert` | 행 삽입 | `Jw` |
| `tCellInsert` | 셀 삽입 | — |
| `insertRow` | 행 삽입 (대체) | — |
| `insertColumn` | 열 삽입 | `Kw` |
| `insertCell` | 셀 삽입 (DOM: `f.insertCell(t)`) | — |
| `tRowDelete` | 행 삭제 | `$w` |
| `tCellDelete` | 셀 삭제 | — |
| `deleteCell` | 셀 제거 | — |
| `deleteColumn` | 열 삭제 | `Lw` |
| `deleteTable` | 표 전체 삭제 | — |

## 7. 크기 균등 배분 / 리사이즈

| 함수 | 역할 | 단축키 |
|------|------|--------|
| `equalTableRow` | 행 높이 균등 배분 | Alt+H |
| `equalTableCol` | 열 폭 균등 배분 | Alt+W |
| `EqualCellWidth` (`yi.z3t`) | 선택된 셀 폭 균등화 | — |
| `EqualCellHeight` (`yi.jLt`) | 선택된 셀 높이 균등화 | — |

### 드래그 리사이즈 핸들

| CSS 클래스 / ID | 역할 |
|-----------------|------|
| `hcwo_table_resize_dragger` / `hcwoTableResizeDragger` | 표 전체 리사이즈 핸들 |
| `hcwo_table_row_resize_dragger` / `hcwoTableRowResizeDragger` | 행 높이 리사이즈 핸들 |
| `hcwo_table_col_resize_dragger` / `hcwoTableColResizeDragger` | 열 폭 리사이즈 핸들 |

## 8. 셀 선택 / 내비게이션

| 함수 (메뉴 커맨드) | 역할 |
|-------------------|------|
| `selectTable` | 표 전체 선택 |
| `selectCell` | 개별 셀 선택 |
| `selectRow` | 행 전체 선택 |
| `selectColumn` | 열 전체 선택 |

### CSS 클래스

| 클래스 | 용도 |
|--------|------|
| `hcwo_table` | 표 컨테이너 |
| `hcwo_hwp_table_grid` | HWP 스타일 표 그리드 |
| `hcwo_table_grid` | 표 그리드 표시 |
| `hcwo_selected_cell` | 선택된 셀 표시 |
| `hcwo_vmerge_cell` | 세로 병합 셀 표시 |

### 키보드 내비게이션

| 기능 | 설명 |
|------|------|
| `TabMoveCell` | Tab키로 다음 셀 이동 |
| 화살표 키 | 셀 간 이동 |
| Enter | 셀 내 줄바꿈 또는 행 추가 |

## 9. 셀 서식 / 정렬

| 함수 (메뉴 커맨드) | 역할 |
|-------------------|------|
| `alignCell` | 셀 내용 정렬 (수평/수직) |
| `borderCell` | 셀 테두리 설정 |
| `highlightColorCell` | 셀 배경색/강조색 |
| `tableLine` | 표 선 설정 |
| `formatCopy` | 서식 복사 |

### 정렬 속성

```javascript
// 수평 정렬
textAlign: "left" | "center" | "right"   // r.TIr 참조

// 수직 정렬
textBaseline: "top" | "middle" | "bottom" // r.vAlign 참조
```

## 10. 대화상자 (Dialog)

| 대화상자 | 디스패처 | 역할 |
|----------|---------|------|
| `TableCreateDialog` | — | 새 표 생성 (행/열 수, 크기 설정) |
| `TablePropertyDialog` | `F9s()` | 표 속성 (폭/높이, 여백, 배치) |
| `TableSplitCellDialog` | `F6s()` | 셀 분할 설정 (행/열 분할 수) |
| `TableCellBorderFillDialog` | — | 셀 테두리/배경 설정 |
| `PasteCellDialog` | — | 셀 붙여넣기 옵션 |

## 11. 조작 타입 상수 (Undo/Redo용)

| 상수 | 함수 | 설명 |
|------|------|------|
| `yi.W3t` | MergeCell | 셀 병합 |
| `yi.zLt` | SplitCell | 셀 분할 |
| `yi.VLt` | unMergeCell | 병합 해제 |
| `yi.P3t` | Row/Col 조작 | 행/열 삽입/삭제 |
| `yi.H3t` | EqualCellHeight | 높이 균등 |
| `yi.z3t` | EqualCellWidth | 폭 균등 |
| `yi.jLt` | CellHeight | 셀 높이 |
| `yi.C3t` | Cell 조작 | 셀 관련 |
| `yi.L3t` | Left/Indent | 들여쓰기 |

## 12. 렌더링 파이프라인 요약

```
서버 JSON 수신
  ↓
표 구조 검증 → M1n() 열 폭 배분
  ↓
for each cell:
  ├─ 위치 계산: JUn(행), ZUn(열), Bun[colAddr, rowAddr]
  ├─ 병합 크기: Fun[colSpan, rowSpan] → 열 폭 합산
  ├─ 배경 채우기:
  │   ├─ 단색: CREFtoRGB() → fillStyle → fillRect()
  │   └─ 패턴: Ubr() 색상 적용 → createPattern() → fillRect()
  ├─ 테두리 (4변 각각):
  │   ├─ 방향: OCt(위)/kCt(아래)/ACt(좌)/RCt(우)
  │   ├─ 유형: eWn/rWn에서 선 스타일 조회
  │   ├─ 디스패치: Tbr() → bbr()(기본) 또는 Cbr()(모서리)
  │   └─ 대시: setLineDash() + strokeStyle + lineWidth
  └─ 텍스트: textAlign/textBaseline 설정 → fillText()
```

## 13. Canvas 2D API 사용 빈도

| API | 횟수 | 용도 |
|-----|------|------|
| `lineWidth` | 80 | 테두리 두께 |
| `strokeStyle` | 55 | 테두리 색상 |
| `fillStyle` | 47 | 배경 색상/패턴 |
| `setLineDash()` | 23 | 대시 패턴 |
| `globalCompositeOperation` | 16 | 패턴 색상 합성 |
| `fillRect()` | 11 | 배경 채우기 |
| `createPattern()` | 6 | 패턴 채우기 |
| `strokeRect()` | 4 | 사각형 테두리 |
| `textBaseline` | 4 | 수직 정렬 |

## 14. rhwp 대비 구현 현황

| 기능 | 한컴 webhwp | rhwp 현재 | 비고 |
|------|-----------|-----------|------|
| 셀 테두리 렌더링 | 6+ 유형 (실선/파선/이중/삼중 등) | 실선 위주 | `bbr`/`Cbr`/`gbr`/`xbr` 참조 |
| 패턴 채우기 | 8가지 패턴 + `createPattern` | 미구현 | `Ubr` 색상 합성 방식 참조 |
| 셀 병합/분할 | `MergeCell`/`SplitCell` 완전 지원 | 병합 렌더링만 | 편집은 미구현 |
| 행/열 삽입/삭제 | 6종 커맨드 | 미구현 | `tRowInsert` 등 |
| 드래그 리사이즈 | 행/열/표 3종 핸들 | 미구현 | CSS 드래거 참조 |
| 크기 균등 배분 | `equalTableRow`/`equalTableCol` | 미구현 | |
| 표 생성 대화상자 | `TableCreateDialog` | 미구현 | |
| 셀 선택 | 개별/행/열/전체 4종 | 개별/전체 | |
| 키보드 내비게이션 | Tab 셀 이동 지원 | Tab 지원 | |
| 폭 배분 알고리즘 | `M1n`/`B1n`/`K1n` | 고정 폭만 | 자동 폭 미지원 |
| 중첩 표 | `iWn` 셀 내용에 포함 | 지원 | |

---

*분석 소스: `webhwp/js/hwpApp.1827379d2f5132ffd00b.chunk.js` (5.17MB)*
