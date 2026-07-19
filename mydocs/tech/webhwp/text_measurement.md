---
kind: investigation
status: historical
canonical: mydocs/tech/webhwp/README.md
last_verified: 2026-07-17
---

# 한컴 webhwp 텍스트 측정 시스템 분석

> 분석 대상: `/webhwp/js/hwpApp.*.chunk.js` (minified webpack bundle, ~5MB)
> 분석 일자: 2026-02-09

## 1. 전체 파이프라인 개요

```
문자 + 폰트 + 크기 + 장평
        ↓
    캐시 조회 (LRU 128)
    ├─ 히트: 캐시값 반환
    └─ 미스: 측정 진행
        ↓
    문자 유형 판별 (한글/ASCII/제로폭/기타)
    ├─ 제로폭: 0 반환
    └─ 기타: 측정
        ↓
    Canvas 측정 (1000pt) 또는 DOM 측정 (400pt)
    ├─ 한글(44032-55203): '가' 대리 측정
    ├─ ASCII(<127): 'A' 대리 측정
    └─ 기타: 실제 문자 직접 측정
        ↓
    정규화: width / 100 (Canvas) 또는 / 40 (DOM)
        ↓
    소수점 2자리 반올림
        ↓
    HWP 단위 변환: ABt(value) = value × 7200 / 96
        ↓
    폰트별 스케일링: ZRt(u, iYt, 1000) = round(u × iYt / 1000)
        ↓
    최종 폭 (HWP 단위) 반환 + 캐시 저장
```

## 2. Canvas 기반 측정 (주 방식)

### 2.1 핵심 코드 (복원)

```javascript
// 위치: hwpApp chunk, offset ~3295900
gqt(char, fontName, sizeCode, variant, context) {
    // 1. 캐시 키 생성 및 조회
    const cacheKey = String(char) + fontName + String(4096 * sizeCode + variant);
    const cached = this.$yr(cacheKey);
    if (cached !== 0) return cached;

    // 2. 폰트 속성 조회
    const fontProps = Zt.Djt(fontName);       // { iYt, hYt, ... }
    const charType = $t.kjt(char.charCodeAt(0));  // 문자 유형
    const resolvedFont = $t.Ojt(char, context, fontName, defaultType);

    // 3. Canvas 폰트 설정 (변경 시에만)
    if (this._lastFont[0] !== fontName ||
        this._lastFont[1] !== charType ||
        this._lastFont[2] !== resolvedFont) {
        this.ctx.font = "1000pt " + $t.Ajt(fontName, charType, resolvedFont);
        this._lastFont = [fontName, charType, resolvedFont];
    }

    // 4. 문자 유형별 측정
    let measured;
    const code = char.charCodeAt(0);

    if (fontProps && (4 & fontProps.hYt || 8 & fontProps.hYt)
        && code >= 44032 && code <= 55203) {
        // 한글 음절 → '가'(U+AC00) 대리 측정
        measured = this.ctx.measureText("\uAC00").width / 100;
    } else if (fontProps && (4 & fontProps.hYt) && code < 127) {
        // ASCII → 'A' 대리 측정
        measured = this.ctx.measureText("A").width / 100;
    } else if (isZeroWidth(code)) {
        measured = 0;
    } else {
        // 기타 문자 → 직접 측정
        measured = this.ctx.measureText(char).width / 100;
    }

    // 5. 반올림 (소수점 2자리)
    measured = Math.round(100 * measured) / 100;

    // 6. HWP 단위 변환
    let hwpWidth = ABt(measured);  // = measured × 7200 / 96

    // 7. 폰트별 advance width 스케일링
    const advBase = fontProps ? fontProps.iYt : 1024;
    let scaled = ZRt(hwpWidth, advBase, 1000);  // = round(hwpWidth × advBase / 1000)

    // 8. 최종 계산
    const quarterSize = parseInt(sizeCode / 4);
    let result;
    if (variant !== 100) {
        result = 4 * parseInt(scaled * quarterSize * variant / (100 * advBase));
    } else {
        result = 4 * ZRt(scaled, quarterSize, advBase);
    }

    // 9. 캐시 저장
    this.Zyr(cacheKey, result);
    return result;
}
```

### 2.2 왜 1000pt인가?

| 측정 폰트 크기 | '가' measureText().width | 정밀도 |
|---|---|---|
| 10pt | ~10px | 정수 단위, 오차 ±0.5px = ±5% |
| 100pt | ~100px | 소수점 1자리, 오차 ±0.05px = ±0.05% |
| 1000pt | ~1000px | 소수점 2자리, 오차 ±0.005px = ±0.0005% |

1000pt로 측정 후 /100으로 나누면 **소수점 2자리 정밀도**를 확보한다.

### 2.3 한글 대리 측정 원리

대부분의 한글 폰트에서 **모든 한글 음절(가~힣, 11,172자)은 동일한 advance width**를 가진다. 이는 한글 폰트의 등폭 특성 때문이다.

따라서:
- 한글 '가' 한 번 측정 → 모든 한글 음절에 동일 폭 적용
- ASCII 'A' 한 번 측정은 **사용하지 않음** → ASCII는 비례폭이므로 실제로는 각 문자를 개별 측정해야 함
- 단, `4 & hYt` 플래그가 있는 폰트(composite CJK 폰트)에서만 대리 측정 적용

## 3. DOM 기반 측정 (폴백 방식)

### 3.1 핵심 코드 (복원)

```javascript
// 위치: hwpApp chunk, offset ~3296362
// 오프스크린 <span> 요소 생성
const dom = document.createElement("span");
dom.id = "DomForMeasureElement";
dom.style.cssText = "margin:0;padding:0;white-space:nowrap;" +
                    "position:absolute;left:-10000px;";
document.body.appendChild(dom);

// 측정
dom.style.fontFamily = resolvedFontFamily;
dom.style.fontSize = "400pt";
dom.textContent = char;

const width = dom.getBoundingClientRect().width / 40;
```

### 3.2 Canvas vs DOM 비교

| 항목 | Canvas | DOM |
|---|---|---|
| 폰트 크기 | 1000pt | 400pt |
| 정규화 | /100 | /40 |
| API | `measureText().width` | `getBoundingClientRect().width` |
| 캐시 키 | `char + font + (4096*size + variant)` | `char_font_size_variant` |
| 용도 | 주 방식 (Chrome) | 폴백 (Canvas 불안정 시) |

## 4. 단위 변환 체계

### 4.1 핵심 상수

```javascript
const DPI = 96;              // 화면 DPI
const HWP_PER_INCH = 7200;   // 1인치 = 7200 HWPUNIT
const ROTATION_DIV = 4;       // 회전 분할자
```

### 4.2 변환 함수

| 함수 | 수식 | 용도 |
|---|---|---|
| `ABt(px)` | `px × 7200 / 96` (= px × 75) | 픽셀 → HWP 단위 |
| `Pjt(hwp)` | `hwp × 96 / 7200` (= hwp / 75) | HWP 단위 → 픽셀 |
| `Fjt(hwp)` | `hwp / 7200` | HWP 단위 → 인치 |
| `ZRt(t, i, n)` | `round((t × i) / n)` | 스케일링 + 반올림 |

### 4.3 최종 폭 계산 공식

```
quarterSize = parseInt(fontSize / 4)

variant == 100 (기본 장평):
  result = 4 × ZRt(scaled, quarterSize, advBase)
         = 4 × round(scaled × quarterSize / advBase)

variant != 100 (비표준 장평):
  result = 4 × parseInt(scaled × quarterSize × variant / (100 × advBase))
```

## 5. 폰트 메타데이터

### 5.1 폰트 속성 구조 (Xt.tYt[fontName])

```javascript
{
    iYt: 1024,    // advance width base (기본값 1024)
    hYt: 0,       // 폰트 플래그
                   //   bit 2 (4): composite CJK 폰트
                   //   bit 3 (8): 세로쓰기 CJK 폰트
    eYt: ...,     // 추가 속성
    rYt: ...      // 예약 속성
}
```

### 5.2 `iYt` (advance width base)

- 각 폰트의 **기준 advance width** 값
- 대부분 1024 (TrueType 일반적 units per em의 축약)
- 측정값을 이 값으로 스케일링: `round(hwpWidth × iYt / 1000)`
- 폰트마다 다를 수 있어 `Djt(fontName)`으로 조회

### 5.3 번들 웹폰트 목록

| 파일명 | 폰트 이름 | 경로 |
|---|---|---|
| `h2hdrm.woff2` | 함초롬돋움 | `commonFrame/font/` |
| `hygtre.woff2` | HY헤드라인M / HY광고딩 | `commonFrame/font/` |
| `hygprm.woff2` | HY그래픽 | `commonFrame/font/` |
| `hymjre.woff2` | HY광명조 | `commonFrame/font/` |
| `MalgunGothicW35-Regular.woff2` | 맑은 고딕 | `commonFrame/font/` |
| `SpoqaHanSans-Regular.woff2` | Spoqa Han Sans | `commonFrame/font/` |
| `TimesNewRomanW05-Regular.woff2` | Times New Roman | `commonFrame/font/` |
| `ArialW05-Regular.woff2` | Arial | `commonFrame/font/` |
| `CourierNewW05-Regular.woff2` | Courier New | `commonFrame/font/` |
| `Calibri.woff2` | Calibri | `commonFrame/font/` |
| `TahomaW05-Regular.woff2` | Tahoma | `commonFrame/font/` |
| `VerdanaW05-Regular.woff2` | Verdana | `commonFrame/font/` |

### 5.4 폰트 로딩 방식

```javascript
// FontFace API로 lazy 로딩
new FontFace(fontName, "url(" + fontUrl + ")").load().then(() => {
    // 로딩 완료 시 문서 재렌더링 트리거
    fontInfo.loaded = true;
    document.fEe();  // re-render
});
```

- Chrome에서만 웹폰트 로딩 적용 (`isChrome` 체크)
- 폰트가 로딩되기 전에는 캐시된 측정값을 사용하지 않음 (`tIr()` 가드)
- 로딩 완료 후 전체 문서 재렌더링

## 6. 캐싱 시스템

### 6.1 LRU 캐시 구현 (Vo 클래스)

```javascript
class LRUCache {
    constructor(maxCapacity = 128) {
        this.maxCapacity = maxCapacity;
        this.evictThreshold = parseInt(75 * maxCapacity / 100);  // 75%
        this.count = 0;
        this.buffer = {};        // key → node 해시맵
        this.head = sentinel;    // 이중 연결 리스트 (LRU)
        this.end = sentinel;     // 이중 연결 리스트 (MRU)
    }

    set(key, value) {
        // 용량 초과 시 가장 오래된 25% 제거
        if (this.count >= this.maxCapacity) {
            const toDelete = Math.round(this.maxCapacity - this.evictThreshold);
            // head부터 toDelete개 노드 삭제
        }
        // 새 노드를 end(MRU)에 삽입
    }

    get(key) {
        return this.buffer[key].value;  // O(1) 조회
    }

    contains(key) {
        return !!this.buffer[key];
    }
}
```

### 6.2 캐시 특성

| 항목 | 값 |
|---|---|
| 최대 용량 | 128 엔트리 |
| 퇴거 기준 | 용량 100% 도달 시 |
| 퇴거량 | 가장 오래된 ~25% (= 128 - 96 = 32개) |
| 자료구조 | 이중 연결 리스트 + 해시맵 |
| 조회 복잡도 | O(1) |

### 6.3 캐시 키 형식

**Canvas 방식:**
```
key = String(char) + fontName + String(4096 × sizeCode + variant)
예: "가함초롬돋움40960"
```

**DOM 방식:**
```
key = char + "_" + fontName + "_" + size + "_" + variant
예: "가_함초롬돋움_10_100"
```

## 7. 텍스트 렌더링

### 7.1 렌더링 방식

한컴 webhwp는 **Canvas `fillText()`로 개별 문자 단위 렌더링**한다.

```javascript
for (let w = 0; w < charCount; w++) {
    if (chars[w].char === " ") continue;  // 공백 건너뛰기

    const pixelSize = Pjt(chars[w].mqt * scale);  // 폰트 크기 → px
    const charWidth = Pjt(chars[w].dx) * scale;   // 문자 폭 → px
    const xPos = Pjt(chars[w].dt) * scale;        // X 위치 → px

    ctx.save();
    ctx.scale(chars[w].bqt, 1);      // 장평 적용
    ctx.translate(-(x - x / bqt), 0); // 스케일 보정
    ctx.font = fontString;
    ctx.fillText(chars[w].char, x + xPos, y + yPos);
    ctx.restore();
}
```

### 7.2 핵심 포인트

- **문자 단위 렌더링**: 각 문자를 개별 `fillText()` 호출로 그림 (run 단위가 아님)
- **HWP 단위 기반 위치**: 문자별 위치(`dt`)와 폭(`dx`)이 모두 HWP 단위로 저장
- **렌더링 직전 px 변환**: `Pjt()`로 HWP → px 변환 후 Canvas에 그림
- **장평은 `ctx.scale()`로 적용**: 우리와 동일한 방식

## 8. 우리 구현과의 비교 및 개선 방향

### 8.1 차이점 요약

| 항목 | 한컴 webhwp | 우리 (rhwp) |
|---|---|---|
| 측정 단위 | HWP 단위 (정수) | px (부동소수점) |
| 측정 정밀도 | 1000pt → /100 | 실제 fontSize |
| 한글 처리 | '가' 대리 (등폭 가정) | 개별 문자 측정 |
| 렌더링 단위 | 문자 개별 `fillText()` | run 단위 `fillText()` |
| 위치 계산 | HWP 단위로 누적 → 렌더링 시 px 변환 | px로 직접 누적 |
| 폰트 파일 | woff2 번들 (확정적) | 시스템 폰트 (불확정) |
| 캐싱 | LRU 128 | 없음 |

### 8.2 가장 큰 불일치 원인

1. **run 단위 vs 문자 단위 렌더링**: 우리는 run 전체를 `fillText(text, x, y)`로 한 번에 그리지만, charX는 접두사(prefix) 단위로 `measureText()`한다. Canvas의 커닝/합자 처리로 인해 `measureText("AB").width ≠ measureText("A").width + measureText("B").width`가 될 수 있다. 한컴은 문자 단위로 그리므로 이 문제가 없다.

2. **폰트 불일치**: HWP 파일의 폰트(함초롬돋움 등)가 시스템에 없으면 fallback 폰트가 사용되는데, 측정과 렌더링에서 동일한 fallback이 적용되더라도 커닝 테이블 차이로 미세한 불일치 발생 가능.

3. **정밀도**: 실제 fontSize(10px)로 측정하면 서브픽셀 반올림 오차가 크다.

### 8.3 개선 방향 (향후 참조)

**단기 (캐럿 정확도 우선)**:
- 한글 등폭 가정 도입: 한글 문자는 '가' 한 번 측정으로 통일
- 1000pt 고정밀 측정 도입
- 캐싱 도입

**중기 (렌더링-측정 일치)**:
- 문자 단위 `fillText()` 렌더링으로 전환
- 또는 측정 시 개별 문자 폭 사용 (prefix 방식 대신)

**장기 (완전한 일치)**:
- HWP 폰트 woff2 번들링
- HWP 단위 기반 위치 계산 체계로 전환

## 9. 참조용 변수명 매핑

| 난독화 이름 | 원래 의미 | 위치 (offset) |
|---|---|---|
| `Vo` | LRU Cache 클래스 | ~3293685 |
| `gqt` | 문자 폭 측정 함수 | ~3295900 |
| `$yr` | 캐시 조회 (get) | ~3295705 |
| `Zyr` | 캐시 저장 (set) | ~3295705 |
| `Qyr` | 캐시 인스턴스 | ~3295700 |
| `Jyr` | 마지막 설정 폰트 (캐시) | ~3296000 |
| `qyr` | Canvas 2D context | ~3295850 |
| `Djt` | 폰트 속성 조회 | ~2140052 |
| `Ajt` | CSS font-family 문자열 생성 | ~3295900 |
| `kjt` | 문자 유형 판별 | ~990628 |
| `Ojt` | 폰트 대체 해석 | ~3295775 |
| `FDt` | 제로폭 문자 판별 | ~990226 |
| `ABt` | px → HWP 단위 | ~516676 |
| `Pjt` | HWP 단위 → px | ~516233 |
| `ZRt` | 스케일링 반올림 | ~516048 |
| `iYt` | 폰트 advance width base | ~506568 |
| `hYt` | 폰트 플래그 (CJK 등) | ~506568 |
| `tIr` | 폰트 로딩 완료 체크 | ~3300790 |
| `DomForMeasureElement` | DOM 측정용 오프스크린 span | ~3296362 |
