---
kind: investigation
status: historical
canonical: mydocs/tech/webhwp/README.md
last_verified: 2026-07-17
---

# webhwp 분석: 텍스트 렌더링

> 분석 대상: `webhwp/js/hwpApp.*.chunk.js` (5.17MB minified)
> 분석 일자: 2026-02-09

## 1. 핵심 렌더링 함수 구조

```
렌더링 호출 순서:
1. dbr() — 그림자 효과 처리
   ├─ 그림자 색상/오프셋 적용
   ├─ cbr() 호출 (그림자 위치에서)
   └─ 원래 색상 복원

2. cbr() — 문자 단위 렌더링 (핵심)
   ├─ fonts[] 배열의 각 문자에 대해:
   │   ├─ 위치 계산 (x, y, 베이스라인)
   │   ├─ 폰트 설정 (style + size + family)
   │   ├─ 장평 적용 (ctx.scale)
   │   ├─ ctx.fillText(char, x, y)
   │   ├─ 외곽선: ctx.strokeText(char, x, y)
   │   └─ 장식: fbr() 호출
   └─ ctx.restore()

3. vbr() / _br() — 취소선/이중취소선
   └─ cbr()를 다른 Y 오프셋으로 반복 호출

4. fbr() — 악센트/밑줄 장식
   ├─ obr()로 장식 유형 디코딩
   └─ 유니코드 결합 문자로 그리기
```

## 2. 문자 단위 렌더링 (cbr 함수)

### 2.1 문자 데이터 구조

```javascript
// fonts[] 배열의 각 요소
{
    char: "가",              // 문자
    mqt: 12000,             // 폰트 크기 (HWP 단위)
    bqt: 1.0,               // 장평 (1.0 = 100%)
    dx: 7200,               // 문자 advance width (HWP 단위)
    dt: 0,                  // Y 오프셋 (HWP 단위)
    position: 0,            // 위첨자/아래첨자 위치 (HWP 단위)
    fontName: "함초롬돋움"   // 폰트 이름
}
```

### 2.2 위치 계산

```javascript
// HWP 단위 → 픽셀 변환
d = si.Pjt(v[w].dx) * r;           // 문자 advance width (px)
f = si.Pjt(v[w].dt) * r;           // Y 오프셋 (px)
a = si.Pjt(v[w].position) * r;     // 위첨자/아래첨자 (px)

// 최종 위치
u = i + f;                          // x = 문단x + deltaTop
o = n + (l + a + m);                // y = 문단y + 줄높이 + position + baseline
```

### 2.3 폰트 문자열 구성

```javascript
s = e.fontStyle;                    // "bold", "italic", "bold italic", ""
s += s.length > 0 ? " " : "";
s += b + "px " + v[w].fontName;     // 크기 + 이름
// 결과: "bold 14px Arial" 또는 "italic 16px 함초롬돋움"
```

### 2.4 장평 적용

```javascript
t.scale(h, 1);                      // h = bqt (장평 비율)
t.translate(-(u - u/h), 0);         // 위치 보정
```

### 2.5 fillText 호출

```javascript
// 외곽선이 있는 경우
if (e.rqt != mt.REt) {
    t.fillText(v[w].char, u, o);        // 채우기
    t.strokeStyle = e.$Xt;
    t.lineWidth = e.lineWidth / h;
    t.strokeText(v[w].char, u, o);      // 외곽선
} else {
    t.fillText(v[w].char, u, o);        // 채우기만
}
```

## 3. 색상 처리

| 속성 | 용도 |
|------|------|
| `e.$Xt` | 기본 채우기 색상 (fillStyle) |
| `e.PXt` | 외곽선 색상 (strokeStyle) |
| `e.dqt` | 그림자 색상 1 |
| `e.vqt` | 그림자 색상 2 / 보조 외곽선 색상 |

```javascript
// HWP 색상 참조 → RGB 변환
w.$Xt = n.CREFtoRGB(l.tqt, "#");    // "#RRGGBB" 형식
w.PXt = n.CREFtoRGB(l.iqt, "#");
```

## 4. 그림자 효과 (dbr 함수)

```javascript
dbr(t, i, n, e, r) {
    if (e.eqt != mt.QEt) {              // 그림자 유형 확인
        s = si.Pjt(e.shadowOffsetX) * r;  // X 오프셋
        h = si.Pjt(e.shadowOffsetY) * r;  // Y 오프셋

        if (e.eqt == mt.XEt)
            this.cbr(t, s+i, h+n, e, r);  // 단색 그림자
        else if (e.eqt == mt.qEt)
            // 블러 그림자 (반복 렌더링)
            for (...) this.cbr(t, i+f*c, n+l*c, e, r);
    }
}
```

| 그림자 유형 | 상수 | 설명 |
|------------|------|------|
| 없음 | `mt.QEt` | 기본값 |
| 단색 | `mt.XEt` | 오프셋 위치에 1회 렌더링 |
| 블러 | `mt.qEt` | 여러 오프셋에서 반복 렌더링 |

## 5. 취소선/이중취소선

### 5.1 취소선 (vbr)

```javascript
vbr(t, i, n, e, r) {
    if (e.sqt == 1) {                   // sqt = 취소선 플래그
        // 3회 cbr() 호출 — 다른 Y 오프셋으로
        e.cqt = h;     this.cbr(...);    // 위치 1
        e.cqt = -2*h;  this.cbr(...);    // 위치 2
        e.cqt = -h;    this.cbr(...);    // 위치 3
    }
}
```

### 5.2 이중취소선 (_br)

```javascript
_br(t, i, n, e, r) {
    if (e.hqt == 1) {                   // hqt = 이중취소선 플래그
        e.cqt = -4*h;  this.cbr(...);    // 상단
        e.cqt = 0;     this.cbr(...);    // 중앙
        e.cqt = 2*h;   this.cbr(...);    // 하단
    }
}
```

## 6. 밑줄/악센트 (fbr 함수)

```javascript
fbr(t, i, n, e, r, s, h, u, o, a) {
    c = this.obr(e.uqt);               // 장식 유형 디코딩

    switch(c.pos) {
        case "left":                     // 좌측 결합 부호
            E.arc(C, g-1.5*b, b, 0, T); // 원형 악센트
            break;
        case "bottom":                   // 밑줄
            t.fillText(l, i+v, n+_);
            t.strokeText(l, i+v, n+_);
            break;
        default:                         // 상단 장식
            t.fillText(l, i+v, n+_);
            t.strokeText(l, i+v, n+_);
    }
}
```

- 유니코드 결합 문자(U+030X 범위) 사용
- Canvas `textDecoration` 미사용 — 직접 그리기

## 7. 베이스라인 처리

```javascript
if ("bottom" == e.ZXt) {
    t.textBaseline = "alphabetic";
    m = -si.Pjt(1e3) / 5 * r;
} else if ("hanging" == e.ZXt) {
    m = si.Pjt(v[w].mqt) / 5 * r;
}
```

| 베이스라인 | Canvas 속성 | 수직 보정 |
|-----------|------------|----------|
| bottom | alphabetic | -Pjt(1000)/5 |
| hanging | (기본) | +Pjt(fontSize)/5 |
| (기본) | top | 0 |

## 8. 단위 변환

```javascript
// Pjt: HWP 단위 → 픽셀
Pjt(t) = t * 96 / 7200    // = t / 75

// ABt: 픽셀 → HWP 단위
ABt(t) = t * 7200 / 96    // = t * 75
```

## 9. rhwp와의 비교

| 항목 | webhwp | rhwp |
|------|--------|------|
| 렌더링 단위 | 문자 개별 `fillText()` | run 단위 `fillText()` |
| 위치 데이터 | HWP 단위 (정수) → 렌더링 시 px 변환 | px (부동소수점) |
| 장평 | `ctx.scale(bqt, 1)` | `ctx.scale(ratio, 1)` |
| 외곽선 | `fillText + strokeText` | 미구현 |
| 그림자 | `dbr()` 전용 함수 | 미구현 |
| 취소선 | `cbr()` 반복 호출 (다른 Y) | 미구현 |
| 밑줄 | 유니코드 결합 문자 | 미구현 |
| 베이스라인 | alphabetic / hanging | top |
| 색상 변환 | `CREFtoRGB()` | `ColorRef` → hex |

## 10. 구현 참고 사항

1. **문자 단위 렌더링이 정밀도에 유리**: 커닝/합자 문제 없음
2. **HWP 단위로 위치 저장 → 렌더링 시 px 변환**: 정수 연산으로 누적 오차 최소화
3. **Canvas save/restore 빈번 사용**: 문자마다 transform 상태 변경
4. **외곽선 = fillText + strokeText 조합**: 별도 렌더링 패스 불필요

---

*분석 일자: 2026-02-09*
