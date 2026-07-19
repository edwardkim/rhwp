---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-124/README.md
last_verified: 2026-07-17
---

# 한컴 웹기안기 폰트 시스템 소스 분석

작성일: 2026-02-21
분석 대상: `webgian/hancomgian_files/main-hwpapp.js.download` (4.7MB)

## 1. 개요

한컴 웹기안기의 폰트 처리 시스템은 세 가지 핵심 요소로 구성된다:

1. **임베디드 폰트 메트릭 (.hft)** — 387개 모듈, 342개 폰트에 대한 글리프 폭 테이블
2. **폰트 패밀리 매핑** — HWP 폰트명 → CSS font-family 문자열 변환
3. **웹폰트 동적 로딩** — FontFace API를 사용한 woff2 폰트 로딩

## 2. 임베디드 폰트 메트릭 (.hft)

### 2.1 규모

| 항목 | 수치 |
|------|------|
| .hft 모듈 수 | 387개 (hc_fj ~ hc_mr) |
| 폰트 이름 엔트리 수 | 342개 |
| 언어/스크립트 타입 | 7종 (한글, 영문, 한자, 일본어, 옛한글, 간체, 특수) |

### 2.2 데이터 구조

각 .hft 모듈은 AMD 모듈로 정의되며, 아래 구조를 반환한다:

```javascript
{
  fileName: "enbaskvl.hft",   // 원본 .hft 파일명
  v73: 1,                      // 언어/스크립트 타입 인덱스
  emsize: 1000,                // em 단위 (디자인 유닛/em)
  _73: [                       // 코드 범위별 폭 데이터 배열
    {
      type: 0 | 1 | 2,        // 룩업 타입
      y73: 32768,              // 시작 코드포인트 (HWP 내부 인코딩)
      I73: 65535,              // 끝 코드포인트
      D73: [...]               // 폭 값 배열
    }
  ]
}
```

### 2.3 세 가지 룩업 타입

#### type: 0 — 균일 폭 (Uniform Width)

모든 글자가 동일한 폭:
```javascript
{ type: 0, y73: 32768, I73: 65535, D73: [880] }
// → 해당 범위 모든 문자의 폭 = 880 (emsize 기준)
```

**사용 예**: 시스템 고정폭 한글 폰트 (hgsys.hft)

#### type: 1 — 글자별 개별 폭 (Per-Character Width)

코드포인트별 개별 폭 배열:
```javascript
{
  type: 1, y73: 32, I73: 126,
  D73: [274, 301, 331, 769, 549, 798, ...]
  // D73[charCode - y73] = 해당 글자의 폭
}
```

**사용 예**: 영문 프로포셔널 폰트 (enbaskvl.hft — Baskerville)
- `D73[0]` = 274 → 공백 (U+0020)
- `D73[1]` = 301 → 느낌표 (U+0021)
- 총 95개 값 (ASCII 32~126)

#### type: 2 — 한글 음절 분해 (Hangul Syllable Decomposition)

초성/중성/종성 조합으로 폭을 계산하는 압축 방식:
```javascript
{
  type: 2, y73: 32768, I73: 65535,
  g73: [2, 4, 1],        // [초성 그룹수, 중성 그룹수, 종성 그룹수]
  m73: [0,0,1,0,...],    // 32개: 초성 자모 → 그룹 인덱스
  E73: [0,0,3,0,...],    // 32개: 중성 자모 → 그룹 인덱스
  H73: [0,0,0,0,...],    // 32개: 종성 자모 → 그룹 인덱스
  D73: [900, 850, 590, 950, 900, 850, 635, 950]
  // D73[cho_group * jung_groups * jong_groups + jung_group * jong_groups + jong_group]
}
```

**핵심 아이디어**: 한글 11,172개 음절을 개별 저장하면 11,172개 값이 필요하지만, 초/중/종성을 형태 그룹으로 분류하면 `2×4×1 = 8`개 값만으로 표현 가능.

**사용 예**: 복숭아체 (hgpeach.hft)
- 초성 2그룹: 좁은 자음(ㄱ,ㄴ,ㄷ...) vs 넓은 자음(ㅁ,ㅂ...)
- 중성 4그룹: 세로모음(ㅏ,ㅓ) / 가로모음(ㅗ,ㅜ) / 복합(ㅘ,ㅙ) / 기타
- 종성 1그룹: 차이 없음

### 2.4 언어/스크립트 타입 (v73)

| v73 값 | 언어/스크립트 | 모듈 수 |
|--------|-------------|---------|
| 0 | 한글 (Hangul) | 110 |
| 1 | 영문 (Latin) | 172 |
| 2 | 한자 (Hanja) | 37 |
| 3 | 일본어 (Japanese) | 28 |
| 4 | 옛한글 (Old Hangul) | 3 |
| 5 | 간체 (Simplified Chinese) | 35 |
| 6 | 특수 (Special) | 2 |

### 2.5 emsize 분포

| emsize | 모듈 수 | 대표 폰트 |
|--------|---------|-----------|
| 1000 | 295 | 대부분의 한글/영문 폰트 |
| 1200 | 34 | 명조, 고딕 등 구형 HWP 폰트 |
| 512 | 48 | 일본어/중국어/특수 폰트 |
| 1024 | 8 | 일부 특수 폰트 |

### 2.6 폰트 매핑 테이블 (R.data)

7개 하위 배열로 구성되며, 각 엔트리는:

```javascript
{
  fontName: "명조",
  fontData: [normalHft, boldHft, italicHft, boldItalicHft]
  // 4슬롯: 일반/볼드/이탤릭/볼드이탤릭 .hft 참조
}
```

**스타일 폴백 순서** (L73):
```
일반     → 일반, 이탤릭, 볼드, 볼드이탤릭
볼드     → 볼드, 일반, 볼드이탤릭, 이탤릭
이탤릭   → 이탤릭, 일반, 볼드이탤릭, 볼드
볼드이탤릭 → 볼드이탤릭, 이탤릭, 볼드, 일반
```

### 2.7 주요 폰트명 ↔ .hft 파일 매핑

| 폰트명 | 한글 .hft | 영문 .hft |
|--------|-----------|-----------|
| 명조 | hgmj.hft | enmj.hft |
| 고딕 | hggt.hft | engt.hft |
| 시스템 | hgsys.hft | ensys.hft |
| 한양신명조 | hgsmj.hft | ensmj.hft |
| 한양견명조 | hggmj.hft | engmj.hft |
| 한양중고딕 | hgjgt.hft | enjgt.hft |
| 한양견고딕 | hgggt.hft | enggt.hft |
| 한양그래픽 | hggrp.hft | engrp.hft |
| 한양궁서 | hggs.hft | engs.hft |
| 휴먼명조 | hmksm.hft | — |
| 휴먼고딕 | hmkmg.hft | — |
| HY둥근고딕 | hyhggl.hft | hyengl.hft |
| 문화바탕 | hgbt.hft | — |
| 문화돋움 | hgdu.hft | — |
| #세명조 | hchgsemj.hft | hcensemj.hft |
| #신명조 | hchgsmj.hft | hcensmj.hft |
| 신명 세명조 | tesemhg.hft | tesemen.hft |
| 양재 다운명조M | yjhgdnmj.hft | yjendnmj.hft |

**파일명 접두어 규칙**:

| 접두어 | 의미 |
|--------|------|
| `hg` | 한글 글리프 (표준 폰트) |
| `en` | 영문 글리프 |
| `hchg` | 한글 글리프 (#접두어 폰트) |
| `hcen` | 영문 글리프 (#접두어 폰트) |
| `te` | 신명(Shinmyeong) 폰트 |
| `yj` | 양재(Yangjae) 폰트 |
| `hme` | HCI 꽃이름 폰트 |
| `hmk` | 휴먼(Humanist) 한글 폰트 |
| `han`/`khan` | 한/공한 폰트 |
| `jp`/`sp` | 일본어/간체 시스템 폰트 |
| `hy` | HY 폰트 |
| `fl` | 풀어쓰기 폰트 |

## 3. 폰트 패밀리 매핑 (CSS font-family 생성)

### 3.1 매핑 함수 (XTt)

HWP 폰트명을 CSS `font-family` 문자열로 변환:

```javascript
XTt: function(fontName, isSpecialChar, isSymbol) {
  var alias = qTt[fontName.toUpperCase()];  // 별칭 테이블 조회
  return alias
    ? "'" + fontName + "'," + alias + "," + fallbacks
    : "'" + fontName + "'," + fallbacks;
}
// 결과 예: "'함초롬바탕','HCR Batang','serif'"
```

### 3.2 폰트 별칭 테이블 (qTt)

한글 폰트의 다양한 이름 변형을 매핑:

```javascript
qTt: {
  "HY헤드라인M":  "'HYHeadLine M','HYHeadline medium','HYHeadline'",
  "HYHEADLINE M": "'HY헤드라인M','HYHeadline medium','HYHeadline'",
  "HY궁서B":     "'HYGungSo B','HYGungSo black','HYGungSo'",
  // ... 다수 엔트리
}
```

### 3.3 폴백 체인

```
요청 폰트 → 별칭 폰트 → 기본 한글 폰트 → 시스템 폴백
'함초롬바탕' → 'HCR Batang' → 'Malgun Gothic','맑은 고딕' → serif
```

## 4. 웹폰트 동적 로딩

### 4.1 FontFace API (Chrome 전용)

```javascript
// 등록된 웹폰트
fontRegistry = {
  "HY헤드라인M":  { url: baseUrl + "/hygtre.woff2",  pending: false, loaded: false },
  "HY견고딕":     { url: baseUrl + "/hygtre.woff2",  pending: false, loaded: false },
  "HY그래픽":     { url: baseUrl + "/hygprm.woff2",  pending: false, loaded: false },
  "HY견명조":     { url: baseUrl + "/hymjre.woff2",  pending: false, loaded: false },
  "SpoqaHanSans": { url: baseUrl + "/SpoqaHanSans-Regular.woff2", pending: false, loaded: false }
};

// 지연 로딩 흐름
if (isChrome && fontRegistry[fontName] && !fontRegistry[fontName].loaded) {
  new FontFace(fontName, "url(" + fontRegistry[fontName].url + ")")
    .load()
    .then(function() {
      fontRegistry[fontName].loaded = true;
      triggerPageRerender();  // 폰트 로드 후 재렌더링
    });
  fontRegistry[fontName].pending = true;
}
```

**참고**: 5종의 웹폰트만 동적 로딩. 나머지는 사용자 시스템에 설치된 폰트에 의존.

## 5. 폰트 메트릭 사용 — 텍스트 폭 측정 파이프라인

### 5.1 2단계 측정 전략

```
┌─────────────────────────────────────────┐
│  1차: .hft 메트릭 기반 측정 (서버 독립)    │
│  lr3(char, fontName, style) → true?      │
│    → cr3(char, fontName, style)          │
│    → 폭 = D73[index] × fontSize / emsize │
├─────────────────────────────────────────┤
│  2차: 브라우저 측정 (폴백)                │
│  lr3(char, fontName, style) → false?     │
│    → Canvas: ctx.font="1000pt font"      │
│      ctx.measureText(char).width / 100   │
│    → 또는 DOM: span.style.fontSize="400pt"│
│      span.getBoundingClientRect().width/40│
└─────────────────────────────────────────┘
```

### 5.2 .hft 기반 측정 함수 (cr3)

```javascript
function cr3(charCode, fontName, styleFlags) {
  // 1. 언어 타입 판별: AAt(charCode) → 한글/영문/한자/...
  var langType = AAt(charCode);

  // 2. 폰트 찾기: R.data[langType]에서 fontName 검색
  var fontEntry = S73(fontName, langType);
  if (!fontEntry) return fallback;

  // 3. 스타일 폴백: L73으로 normal/bold/italic/bolditalic 순서 탐색
  var hftData = U73(fontEntry, styleFlags);

  // 4. 폭 조회: W73(charCode, hftData)
  var width = W73(charCode, hftData);

  // 5. 볼드 보정
  if (isBold) width += parseInt((emsize + 10) / 20);

  // 6. 위첨자/아래첨자 보정 (64% 축소)
  if (isSuperOrSub) width = _2(width, 16, 25);

  // 7. 최종 픽셀 폭 = width × (fontSize/4) / (100 × emsize)
  return _2(width * (fontSize / 4), ratio, 100 * emsize);
}
```

### 5.3 type:2 한글 음절 폭 계산 (W73 내부)

```javascript
function W73(code, hftData) {
  for (var entry of hftData._73) {
    if (code < entry.y73 || code > entry.I73) continue;

    switch (entry.type) {
      case 0: return entry.D73[0];              // 균일 폭
      case 1: return entry.D73[code - entry.y73]; // 개별 폭
      case 2:
        // 한글 음절 분해
        var syllable = code - entry.y73;  // HWP 내부 코드 기준
        var cho = getChoseong(syllable);  // 초성 인덱스
        var jung = getJungseong(syllable); // 중성 인덱스
        var jong = getJongseong(syllable); // 종성 인덱스
        var choGroup  = entry.m73[cho];
        var jungGroup = entry.E73[jung];
        var jongGroup = entry.H73[jong];
        var idx = choGroup * entry.g73[1] * entry.g73[2]
                + jungGroup * entry.g73[2]
                + jongGroup;
        return entry.D73[idx];
    }
  }
  return emsize; // 기본값: 전각
}
```

### 5.4 측정값 캐싱

```javascript
// LRU 캐시 (128 엔트리)
// 키: charString + fontName + (4096 * fontSize + ratio)
var cache = new LRUCache(128);
```

### 5.5 폰트 대체 체인 (폴백)

.hft에 해당 폰트가 없을 때:

```
1. lr3(char, fontName, style) → false
2. VTt/YTt 폰트 대체 테이블에서 대안 폰트 검색
3. 대안 폰트의 .hft 존재 → cr3 사용
4. 대안도 없음 → MTt()로 기본 폰트명 획득
5. 최종 폴백: 브라우저 measureText/DOM 측정
```

## 6. Canvas fillText에서의 폰트 적용

### 6.1 CSS 폰트 문자열 구성 (J13 텍스트 렌더링)

```javascript
// 폰트 스타일 + 크기 + 패밀리
var fontStr = fontStyle + " " + fontSize + "px " + fontFamily;
// 예: "bold 12px '함초롬바탕','HCR Batang','serif'"

ctx.font = fontStr;
ctx.fillText(char, x, y);
```

### 6.2 폰트 크기 계산 (GRt)

```javascript
GRt: function(baseHeight, scalePercent, isSpecial, isNarrow) {
  var height = baseHeight;
  if (isSpecial || isNarrow) height = 16 * height / 25;  // 0.64 비율
  return height + height * ((scalePercent || 100) - 100) / 100;
}
```

### 6.3 장평(가로 비율) 적용

```javascript
ctx.save();
ctx.scale(hScale, 1);              // hScale = 장평 비율 (0.5~2.0)
ctx.translate(-(x - x / hScale), 0); // 위치 보정
ctx.fillText(char, x, y);
ctx.restore();
```

### 6.4 폰트 스타일 구성 (KRt)

```javascript
KRt: function(isItalic, isBold) {
  var style = "";
  if (isItalic) style += "italic";
  if (isBold) style += (style.length > 0 ? " " : "") + "bold";
  return style;  // "", "bold", "italic", "bold italic"
}
```

## 7. rhwp와의 비교

### 7.1 현재 rhwp 폰트 측정 방식

```rust
// layout.rs — WASM 환경
#[wasm_bindgen(js_namespace = globalThis, js_name = "measureTextWidth")]
fn js_measure_text_width(font: &str, text: &str) -> f64;

// 측정 파이프라인:
// 1. 1000px 크기로 Canvas measureText 호출
// 2. font_size/1000 스케일링
// 3. HWP 단위(×75) 양자화 → 정수 반올림 → px 변환
fn measure_char_width_hwp(font: &str, c: char, hangul_hwp: i32, font_size: f64) -> f64 {
    if c.is_hangul_syllable() {
        return hangul_hwp as f64 / 75.0;  // '가' 대리 측정값 재사용
    }
    let raw_px = js_measure_text_width(font, &c.to_string());
    let actual_px = raw_px * font_size / 1000.0;
    let hwp = (actual_px * 75.0).round() as i32;
    hwp as f64 / 75.0
}
```

### 7.2 비교표

| 항목 | 한컴 웹기안기 | rhwp |
|------|-------------|------|
| 1차 측정 | .hft 임베디드 메트릭 (서버 독립) | — (없음) |
| 2차 측정 | Canvas measureText / DOM span | Canvas measureText (WASM→JS) |
| 한글 최적화 | type:2 음절분해 (초/중/종성 그룹) | '가' 대리 측정 (모든 한글 동일) |
| 측정 정밀도 | emsize 단위 정수 연산 | HWP 단위(×75) 양자화 |
| 캐싱 | LRU 128 엔트리 | 없음 (매 호출 JS 브릿지) |
| 폰트 수 | 342개 커버 | 시스템 폰트 의존 |
| 볼드 보정 | emsize/20 추가 폭 | 없음 (CSS bold 의존) |
| 장평 보정 | 측정 + 렌더 모두 적용 | 렌더만 적용 |

### 7.3 rhwp에 .hft를 활용할 수 있는가?

**직접 사용: 불가** — 한컴 코드에서 추출한 .hft 데이터를 그대로 사용하면 저작권 문제.

**간접 활용 가능한 점**:

1. **구조 참고**: type:2 한글 음절 분해 알고리즘은 공개 유니코드 자모 분해 원리에 기반. 동일한 알고리즘을 독자 구현 가능.

2. **독자 메트릭 생성**: 실제 폰트 파일(.ttf/.otf)의 `hmtx` 테이블에서 글리프 폭을 읽어 우리만의 메트릭 DB 생성 가능. 이는 FreeType 없이도 가능 (OS/2, hmtx 헤더만 파싱).

3. **측정 캐시 도입**: 한컴의 LRU 128 캐시 패턴을 참고하여 우리의 WASM→JS measureText 호출에 캐시 레이어 도입 가능. JS 브릿지 호출 횟수를 대폭 줄일 수 있다.

4. **한글 최적화**: 현재 '가' 하나로 모든 한글 음절 폭을 대표하는 방식 대신, 초/중/종성 그룹별 폭 차이를 반영하면 정밀도 향상.

### 7.4 권장 개선 방향

| 우선순위 | 개선 사항 | 효과 | 비용 |
|---------|-----------|------|------|
| 1 | measureText 캐시 도입 | JS 브릿지 호출 50%+ 감소 | 낮음 |
| 2 | TTF hmtx 파싱으로 독자 메트릭 DB | 서버/오프라인 측정 가능 | 중간 |
| 3 | 한글 음절 분해 측정 | 프로포셔널 한글 폰트 정밀도 | 중간 |
| 4 | 볼드/이탤릭 폭 보정 | 볼드 텍스트 레이아웃 정확도 | 낮음 |
