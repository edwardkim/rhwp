# CJK 폰트 폴백 전략 보고서

작성일: 2026-04-07

## 목차

1. [현황 분석](#1-현황-분석)
2. [HWP 문서 주요 한글 폰트 라이선스 분석](#2-hwp-문서-주요-한글-폰트-라이선스-분석)
3. [오픈소스 폰트 대체 매핑](#3-오픈소스-폰트-대체-매핑)
4. [OS별 기본 폰트 및 우선권 전략](#4-os별-기본-폰트-및-우선권-전략)
5. [CSS font-family 체인 설계](#5-css-font-family-체인-설계)
6. [Canvas 2D 폰트 감지](#6-canvas-2d-폰트-감지)
7. [웹폰트 번들링 전략](#7-웹폰트-번들링-전략)
8. [구현 로드맵](#8-구현-로드맵)
9. [결론 및 권장안](#9-결론-및-권장안)

---

## 1. 현황 분석

### 1.1 현재 폰트 사용 구조

rhwp는 두 가지 렌더링 경로에서 폰트를 사용한다:

| 렌더링 경로 | 폰트 사용 방식 | 관련 코드 |
|------------|---------------|----------|
| SVG 출력 | `font-family` 속성에 폰트명 직접 지정 | `src/renderer/svg.rs` |
| Canvas 2D | `ctx.font` 문자열에 폰트명 설정 | `src/renderer/web_canvas.rs` |

### 1.2 현재 번들링 현황

`web/fonts/` 디렉토리에 17개의 woff2 파일이 존재한다:

**오픈 라이선스 (Git 포함 가능, 17파일):**
- Pretendard 9종 (OFL)
- Cafe24 2종 (무료 배포)
- Happiness Sans 4종 (무료 배포)
- SpoqaHanSans 1종 (OFL)

**저작권 폰트 (Git 미포함, 로컬 배치 필요):**
- 함초롬바탕/돋움 (hamchob-r.woff2, hamchod-r.woff2) -- 한컴 저작물
- HY 시리즈 (h2hdrm, hygprm, hygtre, hymjre) -- 한컴/한양 저작물
- MS 폰트 (Arial, Calibri, Courier New, Tahoma, Times New Roman, Verdana, Malgun Gothic, Webdings, Wingdings)

### 1.3 현재 치환 체계

`font-substitution.ts`에서 webhwp 기반 7개 언어별 치환 테이블을 운용한다. 치환 체인의 최종 종착점은 대부분 **함초롬바탕/함초롬돋움**이며, 이 두 폰트가 사실상의 최종 폴백 역할을 한다.

### 1.4 문제점

1. **함초롬바탕/돋움 woff2**를 번들링하여 배포하는 것은 한컴 라이선스 위반 소지
2. **MS 폰트 woff2** (Arial, Calibri 등)도 재배포 불가
3. 폴백 체인이 저작권 폰트에 의존하여, 해당 파일 제거 시 렌더링 품질 저하
4. `font-loader.ts`에서 40+개 폰트명을 모두 저작권 woff2에 매핑

---

## 2. HWP 문서 주요 한글 폰트 라이선스 분석

### 2.1 HWP 문서에서 자주 사용되는 폰트

HWP 문서 작성 시 기본 지정되거나 자주 선택되는 폰트를 빈도순으로 정리한다.

| 폰트명 | 분류 | 라이선스 | 재배포 가능 | 비고 |
|--------|------|---------|-----------|------|
| 한컴바탕 | Serif | 한컴 전용 | **불가** | HWP 기본 바탕체 |
| 한컴돋움 | Sans-serif | 한컴 전용 | **불가** | HWP 기본 돋움체 |
| 함초롬바탕 | Serif | 한컴 전용 | **불가** | 한컴 오피스 번들 |
| 함초롬돋움 | Sans-serif | 한컴 전용 | **불가** | 한컴 오피스 번들 |
| 맑은 고딕 | Sans-serif | MS 전용 | **불가** | Windows 번들 |
| 굴림 / 굴림체 | Sans-serif (둥근) | MS 전용 | **불가** | Windows 번들 |
| 바탕 / 바탕체 | Serif | MS 전용 | **불가** | Windows 번들 |
| 돋움 / 돋움체 | Sans-serif | MS 전용 | **불가** | Windows 번들 |
| 궁서 / 궁서체 | Serif (필기) | MS 전용 | **불가** | Windows 번들 |
| HY신명조 | Serif | 한양 전용 | **불가** | HWP 번들 |
| HY중고딕 | Sans-serif | 한양 전용 | **불가** | HWP 번들 |
| HY견명조 | Serif | 한양 전용 | **불가** | HWP 번들 |
| HY견고딕 | Sans-serif | 한양 전용 | **불가** | HWP 번들 |
| HY헤드라인M | Display | 한양 전용 | **불가** | HWP 번들 |
| HY그래픽 | Display | 한양 전용 | **불가** | HWP 번들 |
| 새바탕 | Serif | 한컴 전용 | **불가** | HWP 번들 |
| 새돋움 | Sans-serif | 한컴 전용 | **불가** | HWP 번들 |
| 새굴림 | Sans-serif | 한컴 전용 | **불가** | HWP 번들 |
| 휴먼명조 | Serif | 휴먼 전용 | **불가** | HWP 번들 |

### 2.2 이미 사용 중인 오픈소스 폰트

| 폰트명 | 분류 | 라이선스 | 배포 형태 |
|--------|------|---------|----------|
| Pretendard | Sans-serif | SIL OFL 1.1 | woff2 번들 |
| SpoqaHanSans | Sans-serif | SIL OFL 1.1 | woff2 번들 |
| Happiness Sans | Sans-serif | 무료 배포 (비상업 포함) | woff2 번들 |
| Cafe24 Ssurround | Display | 무료 배포 | woff2 번들 |

---

## 3. 오픈소스 폰트 대체 매핑

### 3.1 대체 후보 오픈소스 폰트 목록

#### Serif 계열 (바탕/명조 대체)

| 오픈소스 폰트 | 라이선스 | 메트릭 유사성 | 한글 글리프 수 | woff2 크기 | 추천도 |
|-------------|---------|------------|-------------|-----------|-------|
| **Noto Serif KR** | SIL OFL 1.1 | ★★★★ | 11,172+ | ~4.5MB (전체), ~1.2MB (서브셋) | **최우선** |
| **나눔명조 (NanumMyeongjo)** | SIL OFL 1.1 | ★★★☆ | 11,172 | ~2.5MB | 차선 |
| KoPub 바탕체 | 무료 배포 | ★★★☆ | 11,172 | ~3.0MB | 보조 |
| 본명조 (Source Han Serif) | SIL OFL 1.1 | ★★★★ | 44,000+ (CJK) | ~8MB | CJK 통합 필요 시 |

**메트릭 유사성 평가 기준:**
- 한컴바탕/함초롬바탕과의 글자폭 차이율
- 줄높이(line-height) 비율
- 한글 완성형 11,172자 커버리지

Noto Serif KR은 Google에서 제작하여 한글 명조체 메트릭이 가장 표준적이며, 한컴바탕/바탕체와의 글자폭 편차가 5% 이내로 가장 유사하다.

#### Sans-serif 계열 (돋움/고딕 대체)

| 오픈소스 폰트 | 라이선스 | 메트릭 유사성 | 한글 글리프 수 | woff2 크기 | 추천도 |
|-------------|---------|------------|-------------|-----------|-------|
| **Noto Sans KR** | SIL OFL 1.1 | ★★★★ | 11,172+ | ~4.2MB (전체), ~1.1MB (서브셋) | **최우선** |
| **Pretendard** (현재 사용) | SIL OFL 1.1 | ★★★☆ | 11,172 | ~2.8MB | 이미 번들 중 |
| 나눔고딕 (NanumGothic) | SIL OFL 1.1 | ★★★☆ | 11,172 | ~2.4MB | 차선 |
| 본고딕 (Source Han Sans) | SIL OFL 1.1 | ★★★★ | 44,000+ | ~8MB | CJK 통합 필요 시 |
| IBM Plex Sans KR | SIL OFL 1.1 | ★★☆☆ | 11,172 | ~2.6MB | 보조 |

Pretendard는 Inter 기반에 Noto Sans KR 한글을 합성한 폰트로, 맑은 고딕과의 호환성이 높다. 이미 번들 중이므로 Sans-serif 기본 폴백으로 유지한다.

#### Monospace 계열 (고정폭 대체)

| 오픈소스 폰트 | 라이선스 | 한글 지원 | woff2 크기 | 추천도 |
|-------------|---------|----------|-----------|-------|
| **D2Coding** | SIL OFL 1.1 | O (2,350자) | ~1.2MB | **최우선** |
| Noto Sans Mono CJK KR | SIL OFL 1.1 | O (전체) | ~4.5MB | 무거움 |
| Sarasa Gothic | SIL OFL 1.1 | O (전체) | ~5MB | 무거움 |

HWP 문서에서 고정폭 한글 폰트 사용빈도는 낮으므로 D2Coding 하나로 충분하다.

#### Display/필기 계열

| 원본 폰트 | 대체 오픈소스 | 라이선스 | 비고 |
|----------|------------|---------|------|
| HY헤드라인M | **Pretendard Bold** 또는 나눔스퀘어 Bold | OFL | 제목용 고딕 |
| HY그래픽 | Pretendard Medium | OFL | 그래픽 장식체 |
| 궁서 | Noto Serif KR | OFL | 필기체 대체 없음, 명조로 폴백 |
| 양재튼튼체B | Pretendard ExtraBold | OFL | 두꺼운 고딕 계열 |

### 3.2 최종 폰트 대체 매핑 테이블

```
┌──────────────────────────┬──────────────────────────┬───────────────┐
│ HWP 원본 폰트             │ 오픈소스 대체              │ 분류           │
├──────────────────────────┼──────────────────────────┼───────────────┤
│ 한컴바탕, 새바탕           │ Noto Serif KR            │ Serif         │
│ 함초롬바탕, 함초롱바탕       │ Noto Serif KR            │ Serif         │
│ 바탕, 바탕체               │ Noto Serif KR            │ Serif         │
│ HY신명조, HY견명조         │ Noto Serif KR            │ Serif         │
│ 휴먼명조                   │ Noto Serif KR            │ Serif         │
│ 궁서, 궁서체, 새궁서        │ Noto Serif KR            │ Serif         │
├──────────────────────────┼──────────────────────────┼───────────────┤
│ 한컴돋움, 새돋움           │ Noto Sans KR / Pretendard│ Sans-serif    │
│ 함초롬돋움, 함초롱돋움       │ Noto Sans KR / Pretendard│ Sans-serif    │
│ 돋움, 돋움체               │ Noto Sans KR / Pretendard│ Sans-serif    │
│ 굴림, 굴림체, 새굴림        │ Noto Sans KR / Pretendard│ Sans-serif    │
│ 맑은 고딕                  │ Pretendard               │ Sans-serif    │
│ HY중고딕, HY견고딕         │ Noto Sans KR / Pretendard│ Sans-serif    │
├──────────────────────────┼──────────────────────────┼───────────────┤
│ HY헤드라인M               │ Pretendard Bold          │ Display       │
│ HY그래픽                  │ Pretendard Medium        │ Display       │
├──────────────────────────┼──────────────────────────┼───────────────┤
│ Times New Roman           │ Noto Serif (Latin)       │ Serif (영문)   │
│ Arial, Calibri, Tahoma   │ Pretendard               │ Sans (영문)    │
│ Verdana                   │ Pretendard               │ Sans (영문)    │
│ Courier New               │ D2Coding                 │ Monospace     │
└──────────────────────────┴──────────────────────────┴───────────────┘
```

---

## 4. OS별 기본 폰트 및 우선권 전략

### 4.1 OS별 기본 설치 한글 폰트

#### Windows

| 폰트명 | Windows 버전 | 분류 | 비고 |
|--------|------------|------|------|
| 맑은 고딕 (Malgun Gothic) | Vista+ | Sans-serif | **기본 UI 폰트** |
| 바탕 (Batang) | XP+ | Serif | 시스템 한글 명조 |
| 바탕체 (BatangChe) | XP+ | Serif Mono | 고정폭 명조 |
| 돋움 (Dotum) | XP+ | Sans-serif | 시스템 한글 고딕 |
| 돋움체 (DotumChe) | XP+ | Sans Mono | 고정폭 고딕 |
| 굴림 (Gulim) | XP+ | Sans-serif | 둥근 고딕 |
| 굴림체 (GulimChe) | XP+ | Sans Mono | 고정폭 둥근 고딕 |
| 궁서 (Gungsuh) | XP+ | Serif | 필기체 |
| 궁서체 (GungsuhChe) | XP+ | Serif Mono | 고정폭 필기체 |

#### macOS

| 폰트명 | macOS 버전 | 분류 | 비고 |
|--------|----------|------|------|
| Apple SD Gothic Neo | 10.8+ | Sans-serif | **기본 UI 폰트** |
| AppleMyungjo | 10.3+ | Serif | 시스템 명조 |
| Nanum Gothic | 10.7+ | Sans-serif | 기본 포함 |
| Nanum Myeongjo | 10.7+ | Serif | 기본 포함 |
| Apple Gothic | 10.3+ | Sans-serif | 구 버전 호환 |

#### Linux

| 폰트명 | 배포판 | 분류 | 비고 |
|--------|------|------|------|
| Noto Sans CJK KR | Ubuntu 18.04+ | Sans-serif | Google Noto 패키지 |
| Noto Serif CJK KR | Ubuntu 18.04+ | Serif | Google Noto 패키지 |
| 나눔고딕 (NanumGothic) | Ubuntu 12.04+ | Sans-serif | `fonts-nanum` 패키지 |
| 나눔명조 (NanumMyeongjo) | Ubuntu 12.04+ | Serif | `fonts-nanum` 패키지 |
| 은바탕 (UnBatang) | 일부 | Serif | `fonts-unfonts-core` |
| 은돋움 (UnDotum) | 일부 | Sans-serif | `fonts-unfonts-core` |
| 백묵바탕 (Baekmuk Batang) | 구 배포판 | Serif | 레거시 |

### 4.2 OS 폰트 우선권 전략

**핵심 원칙**: OS에 이미 설치된 폰트를 먼저 시도하고, 없을 때만 웹폰트를 로드한다.

```
[1순위] OS 기본 폰트  →  네트워크 비용 0, 렌더링 즉시
[2순위] 오픈소스 웹폰트  →  CDN/번들에서 로드
[3순위] CSS generic  →  최종 폴백 (serif / sans-serif)
```

이 전략의 장점:
- Windows 사용자는 "맑은 고딕", "바탕" 등 친숙한 폰트로 즉시 렌더링
- macOS 사용자는 "Apple SD Gothic Neo" 등으로 네이티브 느낌 유지
- 웹폰트 다운로드량 최소화 (OS에 폰트가 있으면 woff2 로드 안 함)

---

## 5. CSS font-family 체인 설계

### 5.1 Serif 체인 (바탕/명조 계열)

```css
/* HWP 원본: 한컴바탕, 함초롬바탕, 바탕, HY신명조 등 */
font-family:
  "한컴바탕",                    /* 한컴 오피스 설치 시 */
  "함초롬바탕",                   /* 한컴 오피스 설치 시 */
  "바탕", "Batang",              /* Windows */
  "AppleMyungjo",               /* macOS */
  "Nanum Myeongjo",             /* macOS / Linux */
  "Noto Serif KR",              /* 웹폰트 폴백 (오픈소스) */
  serif;
```

### 5.2 Sans-serif 체인 (돋움/고딕 계열)

```css
/* HWP 원본: 한컴돋움, 함초롬돋움, 돋움, 굴림, 맑은 고딕 등 */
font-family:
  "한컴돋움",                    /* 한컴 오피스 설치 시 */
  "함초롬돋움",                   /* 한컴 오피스 설치 시 */
  "맑은 고딕", "Malgun Gothic",  /* Windows */
  "Apple SD Gothic Neo",        /* macOS */
  "Nanum Gothic",               /* macOS / Linux */
  "Pretendard",                 /* 웹폰트 폴백 (오픈소스, 이미 번들) */
  "Noto Sans KR",               /* 웹폰트 폴백 (오픈소스) */
  sans-serif;
```

### 5.3 Monospace 체인

```css
font-family:
  "돋움체", "DotumChe",         /* Windows */
  "D2Coding",                   /* 웹폰트 폴백 */
  "Noto Sans Mono CJK KR",     /* Linux */
  monospace;
```

### 5.4 @font-face 선언 전략

```css
/* 오픈소스 Serif 웹폰트 */
@font-face {
  font-family: "Noto Serif KR";
  src: url("fonts/NotoSerifKR-Regular.woff2") format("woff2");
  font-weight: 400;
  font-display: swap;
  unicode-range: U+AC00-D7AF, U+1100-11FF, U+3130-318F,  /* 한글 */
                 U+A960-A97F, U+D7B0-D7FF;                /* 한글 확장 */
}

@font-face {
  font-family: "Noto Serif KR";
  src: url("fonts/NotoSerifKR-Bold.woff2") format("woff2");
  font-weight: 700;
  font-display: swap;
  unicode-range: U+AC00-D7AF, U+1100-11FF, U+3130-318F,
                 U+A960-A97F, U+D7B0-D7FF;
}

/* 오픈소스 Sans-serif 웹폰트 (Pretendard 이미 등록, Noto Sans KR 추가) */
@font-face {
  font-family: "Noto Sans KR";
  src: url("fonts/NotoSansKR-Regular.woff2") format("woff2");
  font-weight: 400;
  font-display: swap;
  unicode-range: U+AC00-D7AF, U+1100-11FF, U+3130-318F,
                 U+A960-A97F, U+D7B0-D7FF;
}
```

**`font-display: swap` 사용 이유:**
- 웹폰트 로드 전에 시스템 폰트로 즉시 렌더링 (FOUT 허용)
- HWP 뷰어 특성상 "내용이 안 보이는 것"보다 "폰트가 바뀌는 것"이 낫다

**`unicode-range` 활용:**
- 한글 영역만 분리 선언하면, 영문만 있는 페이지에서는 한글 woff2를 다운로드하지 않음
- 라틴, 한글, CJK 통합한자를 별도 @font-face로 분리하여 필요한 범위만 로드

---

## 6. Canvas 2D 폰트 감지

### 6.1 폰트 설치 여부 감지 방법

Canvas 2D 렌더링에서는 CSS font-family 체인이 자동으로 작동하지 않는다. `ctx.font`에 지정한 폰트가 실제로 사용 가능한지 확인해야 한다.

#### 방법 1: FontFace API `check()` (권장)

```typescript
function isFontAvailable(fontName: string): boolean {
  return document.fonts.check(`12px "${fontName}"`);
}
```

**장점**: 표준 API, 정확도 높음, 비동기 불필요
**단점**: @font-face에 등록되지 않은 시스템 폰트도 감지 가능하나 일부 브라우저에서 불안정

#### 방법 2: Canvas measureText 비교

```typescript
function isFontAvailable(fontName: string): boolean {
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d')!;
  const testStr = '가나다ABCabc';

  ctx.font = `72px monospace`;
  const fallbackWidth = ctx.measureText(testStr).width;

  ctx.font = `72px "${fontName}", monospace`;
  const testWidth = ctx.measureText(testStr).width;

  return Math.abs(testWidth - fallbackWidth) > 0.01;
}
```

**장점**: 모든 브라우저 호환, 시스템 폰트도 정확히 감지
**단점**: 동기적 Canvas 연산 필요

#### 방법 3: Local Font Access API (Chrome 103+)

```typescript
async function getSystemFonts(): Promise<Set<string>> {
  if ('queryLocalFonts' in window) {
    const fonts = await (window as any).queryLocalFonts();
    return new Set(fonts.map((f: any) => f.family));
  }
  return new Set();
}
```

**장점**: 시스템 설치 폰트 전체 목록 확보
**단점**: Chrome 전용, 사용자 권한 요청 필요

### 6.2 권장 감지 전략

```
문서 로드 시:
  1. HWP 문서에서 사용된 폰트 목록 추출
  2. 각 폰트에 대해 document.fonts.check() 또는 measureText 비교로 감지
  3. 사용 가능 → 그대로 사용
  4. 사용 불가 → 대체 매핑 테이블에서 오픈소스 폰트 결정
  5. 오픈소스 폰트 woff2 로드 → FontFace API로 등록
  6. Canvas ctx.font에 최종 폰트명 설정
```

### 6.3 font-loader.ts 개선안

현재 `font-loader.ts`는 모든 폰트를 무조건 woff2에서 로드하는 구조다. 개선안:

```typescript
async function loadFontsWithOsFallback(docFonts: string[]): Promise<void> {
  for (const fontName of docFonts) {
    // 1. OS 폰트 존재 확인
    if (document.fonts.check(`12px "${fontName}"`)) {
      continue; // OS에 있으면 스킵
    }

    // 2. 오픈소스 대체 폰트 결정
    const substitute = getOpenSourceSubstitute(fontName);

    // 3. 대체 폰트도 OS에 있는지 확인
    if (document.fonts.check(`12px "${substitute}"`)) {
      // CSS에서 원본→대체 alias 등록
      registerFontAlias(fontName, substitute);
      continue;
    }

    // 4. 웹폰트 로드 (오픈소스만)
    await loadWebFont(substitute);
    registerFontAlias(fontName, substitute);
  }
}
```

---

## 7. 웹폰트 번들링 전략

### 7.1 파일 크기 분석

| 폰트 | Regular woff2 | Bold woff2 | 합계 | 글리프 수 |
|------|-------------|-----------|------|----------|
| Noto Serif KR | ~4.5MB | ~4.5MB | ~9MB | 전체 |
| Noto Serif KR (한글 서브셋) | ~1.2MB | ~1.2MB | ~2.4MB | 한글+기본라틴 |
| Noto Sans KR | ~4.2MB | ~4.2MB | ~8.4MB | 전체 |
| Noto Sans KR (한글 서브셋) | ~1.1MB | ~1.1MB | ~2.2MB | 한글+기본라틴 |
| Pretendard (현재) | ~2.8MB | ~2.8MB | ~5.6MB | 전체 |
| Pretendard (한글 서브셋) | ~0.9MB | ~0.9MB | ~1.8MB | 한글+기본라틴 |

### 7.2 서브셋 전략

#### Google Fonts 방식 (unicode-range 슬라이스)

Google Fonts는 Noto Sans KR을 120개 이상의 슬라이스로 분할한다. 각 슬라이스는 50~100KB이며, 브라우저가 실제 사용하는 유니코드 범위에 해당하는 슬라이스만 다운로드한다.

```css
/* 슬라이스 예시 — 자주 쓰이는 한글 음절 */
@font-face {
  font-family: 'Noto Sans KR';
  src: url('NotoSansKR-Regular.s1.woff2') format('woff2');
  unicode-range: U+AC00-AD0B;  /* 가~갋 */
}

@font-face {
  font-family: 'Noto Sans KR';
  src: url('NotoSansKR-Regular.s2.woff2') format('woff2');
  unicode-range: U+AD0C-AE17;  /* 갌~긗 */
}
/* ... 120+ 슬라이스 */
```

**장점**: 사용된 글자에 해당하는 슬라이스만 로드, 초기 로딩 최소화
**단점**: 관리 복잡, 오프라인 환경에서 불가

#### 실용적 서브셋 (3분할 권장)

| 슬라이스 | 유니코드 범위 | 예상 크기 | 로딩 시점 |
|---------|------------|----------|----------|
| **Latin** | U+0020-007E, U+00A0-00FF | ~30KB | 즉시 |
| **한글 음절** | U+AC00-D7AF | ~0.8-1.0MB | 한글 첫 등장 시 |
| **한글 자모 + CJK** | U+1100-11FF, U+3130-318F, U+4E00-9FFF | ~2.0MB | 해당 범위 등장 시 |

### 7.3 CDN vs 자체 호스팅 vs 번들링 비교

| 방식 | 장점 | 단점 | 적합한 경우 |
|------|------|------|-----------|
| **Google Fonts CDN** | 무료, 글로벌 캐시, 자동 서브셋 | 중국 차단, 개인정보(IP 전송), 오프라인 불가 | 일반 웹 배포 |
| **자체 CDN 호스팅** | 제어권 확보, 오프라인 가능(SW) | 인프라 비용, 서브셋 직접 관리 | 기업 배포 |
| **NPM 번들링** | 오프라인 완전 지원, VSCode 확장 호환 | 패키지 크기 증가 | **VSCode 확장** |
| **하이브리드** | CDN 우선 + 번들 폴백 | 구현 복잡도 | **권장안** |

### 7.4 권장 번들링 전략

```
┌─────────────────────────────────────────────────┐
│ 배포 환경별 폰트 전략                              │
├────────────────┬────────────────────────────────┤
│ VSCode 확장     │ Pretendard Regular/Bold 번들     │
│                │ + Noto Serif KR Regular 번들      │
│                │ (한글 서브셋, 합계 ~3MB)           │
├────────────────┼────────────────────────────────┤
│ 웹 배포        │ 1순위: OS 시스템 폰트             │
│                │ 2순위: Google Fonts CDN           │
│                │ 3순위: 자체 호스팅 woff2 폴백      │
├────────────────┼────────────────────────────────┤
│ 오프라인 웹     │ Service Worker로 폰트 캐시         │
│                │ + 최소 번들 (Pretendard Regular)   │
└────────────────┴────────────────────────────────┘
```

### 7.5 지연 로딩 전략

현재 `font-loader.ts`의 BATCH=4 병렬 로딩을 유지하되, 우선순위를 도입한다:

```
Phase 1 (Critical, 즉시): Pretendard Regular — 대부분의 UI + 기본 폴백
Phase 2 (High, 문서 로드 시): 문서에서 사용된 폰트의 대체 폰트
Phase 3 (Low, 유휴 시): Bold/Italic 변형, 드물게 쓰이는 폰트
```

```typescript
// requestIdleCallback 활용
function loadPhase3Fonts(): void {
  if ('requestIdleCallback' in window) {
    requestIdleCallback(() => loadNonCriticalFonts());
  } else {
    setTimeout(() => loadNonCriticalFonts(), 3000);
  }
}
```

---

## 8. 구현 로드맵

### Phase 1: 즉시 실행 가능 (1-2일)

**목표**: 저작권 폰트 제거, 기본 폴백 체인 구축

1. `font-loader.ts`에서 저작권 woff2 매핑 제거
   - hamchob-r.woff2, hamchod-r.woff2 매핑 제거
   - HY 시리즈 woff2 매핑 제거
   - MS 폰트 woff2 매핑 제거

2. Noto Serif KR Regular woff2 (한글 서브셋) 추가
   - Google Fonts에서 다운로드 (OFL 라이선스)
   - `web/fonts/`에 배치, Git 포함

3. `font-substitution.ts` 치환 체인 종착점 변경
   - 함초롬바탕 → Noto Serif KR
   - 함초롬돋움 → Pretendard

4. `src/renderer/mod.rs`의 `generic_fallback()` 업데이트
   - Serif: `'Noto Serif KR','Batang','바탕',serif`
   - Sans: `'Pretendard','Malgun Gothic','맑은 고딕',sans-serif`

### Phase 2: OS 폰트 감지 도입 (3-5일)

**목표**: OS 폰트 우선 사용, 불필요한 웹폰트 로드 방지

1. 폰트 감지 유틸리티 구현 (`font-detector.ts`)
   - `document.fonts.check()` 기반 감지
   - Canvas measureText 비교 폴백

2. `font-loader.ts` 개선
   - 문서 폰트 목록 → OS 감지 → 필요한 것만 로드
   - CRITICAL_FONTS를 Pretendard + Noto Serif KR로 변경

3. CSS font-family 체인 개선
   - OS 폰트 → 오픈소스 폰트 → generic 3단계 체인 적용

### Phase 3: 서브셋 최적화 (1주)

**목표**: 웹폰트 로딩 성능 최적화

1. Noto Serif KR, Noto Sans KR을 unicode-range 기반 3분할
2. @font-face에 unicode-range 선언 추가
3. 지연 로딩 3단계(Critical/High/Low) 구현

### Phase 4: 메트릭 보정 (2주)

**목표**: 대체 폰트로 인한 레이아웃 차이 최소화

1. 주요 폰트 쌍의 메트릭 비교 데이터 수집
   - 한컴바탕 vs Noto Serif KR: 글자폭, ascent, descent
   - 함초롬돋움 vs Pretendard: 글자폭, ascent, descent

2. 메트릭 보정 계수(scale factor) 도입
   - `font-substitution.ts`에 보정 계수 테이블 추가
   - 레이아웃 계산 시 보정 적용

3. E2E 테스트로 렌더링 품질 검증
   - 기준 문서 세트로 비교 렌더링
   - 줄 바꿈 위치 차이 측정

### 검증 방법

| 단계 | 검증 방법 |
|------|----------|
| Phase 1 | `cargo test` + 기존 SVG 비교 (줄 바꿈 위치 변화 확인) |
| Phase 2 | 각 OS에서 Chrome DevTools > Network 탭으로 폰트 로드 확인 |
| Phase 3 | Lighthouse 성능 점수 + woff2 전송량 측정 |
| Phase 4 | 기준 HWP 문서 10종의 페이지 수 변화 여부 확인 |

---

## 9. 결론 및 권장안

### 9.1 핵심 권장사항

#### 즉시 조치

1. **저작권 woff2 파일을 Git에서 완전히 제거**한다. `web/fonts/`에는 오픈소스 라이선스 폰트만 유지한다.

2. **Serif 최종 폴백을 Noto Serif KR로 교체**한다. 함초롬바탕 대신 Noto Serif KR (SIL OFL) woff2를 번들링한다.

3. **Sans-serif 최종 폴백은 Pretendard 유지**한다. 이미 OFL 라이선스로 번들 중이며, 맑은 고딕과의 메트릭 호환성이 양호하다.

#### 중기 조치

4. **OS 폰트 감지를 도입**하여, Windows의 "맑은 고딕"/"바탕" 등 시스템 폰트가 있으면 웹폰트 로드를 건너뛴다. 이는 Windows 사용자 경험을 크게 개선한다.

5. **unicode-range 기반 분할 로딩**으로 초기 페이지 로드 시 전체 CJK 폰트를 다운로드하지 않도록 한다.

### 9.2 최종 폰트 스택

```
Serif 계열:
  [OS] 한컴바탕/함초롬바탕 → 바탕/Batang → AppleMyungjo
  [Web] Noto Serif KR (OFL, 번들)
  [Generic] serif

Sans-serif 계열:
  [OS] 한컴돋움/함초롬돋움 → 맑은 고딕/Malgun Gothic → Apple SD Gothic Neo
  [Web] Pretendard (OFL, 번들) → Noto Sans KR (OFL, CDN 또는 번들)
  [Generic] sans-serif

Monospace 계열:
  [OS] 돋움체/DotumChe → Menlo
  [Web] D2Coding (OFL, 필요 시 번들)
  [Generic] monospace
```

### 9.3 번들 크기 영향 예측

| 항목 | 현재 | 변경 후 | 차이 |
|------|------|--------|------|
| 저작권 woff2 (Git 미포함, 로컬) | ~15MB | 0 | -15MB |
| 오픈소스 woff2 (Git 포함) | ~10MB (Pretendard 등) | ~13MB (+Noto Serif KR) | +3MB |
| 실제 전송량 (서브셋 적용 시) | — | ~2-3MB (첫 로드) | — |

### 9.4 라이선스 요약

변경 후 번들에 포함되는 폰트의 라이선스:

| 폰트 | 라이선스 | 상업적 사용 | 재배포 | 수정 |
|------|---------|-----------|-------|------|
| Pretendard | SIL OFL 1.1 | O | O | O |
| Noto Serif KR | SIL OFL 1.1 | O | O | O |
| Noto Sans KR | SIL OFL 1.1 | O | O | O |
| SpoqaHanSans | SIL OFL 1.1 | O | O | O |
| D2Coding | SIL OFL 1.1 | O | O | O |
| Happiness Sans | 무료 배포 (상업 포함) | O | O | X |
| Cafe24 | 무료 배포 (상업 포함) | O | O | X |

**SIL OFL 1.1의 주요 조건:**
- 폰트 파일 단독 판매 금지 (소프트웨어에 포함하여 배포하는 것은 허용)
- 원본 저작권 고지 유지
- 파생물도 OFL 라이선스 적용

이 조건은 rhwp 프로젝트(오픈소스 HWP 뷰어)의 사용 방식과 완전히 호환된다.

---

## 부록 A. `resolve_metric_alias` 2-계층 폰트 이름 해석 (Task #259)

### A.1 문제

본 문서 §3~§7 의 CSS font-family 체인과 서브셋 번들링은 **브라우저 렌더** 단계의 폴백.
그러나 rhwp 내부에는 **SVG 좌표 계산** 을 위한 별도 폰트 메트릭 조회 경로가 있다:

```
HWP 파일: "HY중고딕" (또는 별칭)
  ↓ [Layer 1] style_resolver.rs: 한국어 별칭 → 한국어 정규명 (예: 한양중고딕 → HY중고딕)
  ↓ [Layer 2] font_metrics_data.rs::resolve_metric_alias: 한국어 정규명 → 영문 DB 이름 (예: HY중고딕 → HYGothic-Medium)
  ↓ font_metrics_data.rs::find_metric: FONT_METRICS 에서 영문 이름으로 조회
  ↓ None 반환 시 기본 폭 (fallback) 사용 → SVG 에서 글자 겹침
```

Layer 1 은 구현되어 있었으나 Layer 2 가 HY / 본한글 계열에 대해 누락되어 있었다 (Task #259).

### A.2 HY 계열 매핑 (7건)

| 한국어 정규명 | 영문 DB 이름 | em_size | 비고 |
|---|---|---|---|
| HY중고딕 | HYGothic-Medium | 1000 | Regular 만 (bold 요청 시 bold_fallback) |
| HY견고딕 | HYGothic-Extra | 1000 | |
| HY헤드라인M | HYHeadLine-Medium | 1000 | |
| HY견명조 | HYMyeongJo-Extra | 1000 | |
| HY신명조 | HYSinMyeongJo-Medium | 1000 | |
| HY그래픽 | HYGraphic-Medium | 1000 | |
| HY궁서 | HYGungSo-Bold | 1000 | |

### A.3 본한글 / 본명조 근사 매핑 정책

HWP 문서에서 다음 폰트명들은 FONT_METRICS DB 에 정식 엔트리가 없음:
- **본한글 / 본한글vf / 본고딕 계열** (Source Han Sans KR)
- **본명조 계열** (Source Han Serif KR)

정식 DB 엔트리 추가 (TTF → `extract_metrics` 파이프라인) 는 별도 대형 작업이므로, 현재는 **한글 원천이 동일한 오픈소스 폰트로 근사**:

| 원본 폰트 계열 | 매핑 대상 | 근거 |
|---|---|---|
| Source Han Sans 계열 (본한글 · 본고딕 · Noto Sans CJK KR) | **Pretendard** | Pretendard 한글 글리프는 Source Han Sans KR 합성 · OFL 호환 · 이미 번들 |
| Source Han Serif 계열 (본명조 · Noto Serif CJK KR) | **Noto Serif KR** | 같은 serif 한글 원천 · OFL 호환 · 이미 번들 |

### A.4 근사 한계

1. **Latin 폭 차이**: Pretendard Latin 은 Inter 기반. 본한글 Latin 과 미세 차이.
2. **Weight 축 근사**: Pretendard 메트릭은 Regular / Bold 2단계. 본한글 ExtraLight/Light/Medium/Heavy, 본한글vf 의 임의 wght 는 Regular/Bold 중 가까운 쪽으로 근사. CJK 는 weight 별 한글 폭 차이가 작아 실무 허용.
3. **정식 DB 엔트리 추가는 별도 이슈**.

### A.5 유지보수 체크리스트

**새 한글 폰트 추가 시 반드시 확인**:

- [ ] `style_resolver.rs` 에 Layer 1 (별칭 → 정규명) 등록
- [ ] `font_metrics_data.rs::resolve_metric_alias` 에 Layer 2 (정규명 → 영문 DB 이름) 등록
- [ ] FONT_METRICS 배열에 영문 DB 이름으로 엔트리 존재하는지 확인. 없으면:
  - (A) `extract_metrics` 로 TTF 추가 (정식), 또는
  - (B) 기존 유사 폰트로 근사 매핑 (본한글 → Pretendard 사례)
- [ ] 단위 테스트 추가 (`mod tests` in `font_metrics_data.rs`)

Layer 2 누락 시 증상: `find_metric` None 반환 → 기본 폭 → SVG 에서 글자 겹침.


---

## 10. 옛한글 (Old Hangul) Fallback (Task #528)

### 10.1 본질

한/글 2010 이전 버전에서 입력된 옛한글은 PUA 영역 (U+E0BC ~ U+F8F7) 에 저장된다. 한컴 자체 폰트 (함초롬바탕 LVT 등) 는 PUA 글리프를 직접 보유하나, OFL 폰트는 이 영역을 미지원.

해결: PUA → KS X 1026-1:2007 자모 시퀀스 변환 + 변환 결과를 합자 (CCMP/LJMO/VJMO/TJMO) 렌더링하는 폰트 fallback.

### 10.2 채택 폰트

**Source Han Serif K Old Hangul subset** (Adobe + Google, **SIL OFL 1.1**)

- 출처: https://github.com/adobe-fonts/source-han-serif (Adobe-Fonts/source-han-serif)
- 원본: 23 MB (`SourceHanSerifK-Regular.otf`)
- subset: **234 KB woff2** (옛한글 자모 영역만 + 합자 피처 보존)
- 라이선스 동봉: `rhwp-studio/public/fonts/SourceHanSerifK-OFL.txt`

### 10.3 Subset 절차

```bash
# 원본 다운로드
curl -L -O https://github.com/adobe-fonts/source-han-serif/raw/release/OTF/Korean/SourceHanSerifK-Regular.otf

# Old Hangul 영역 + 합자 피처 보존 subset
pyftsubset SourceHanSerifK-Regular.otf \
    --unicodes='U+1100-11FF,U+A960-A97F,U+D7B0-D7FF' \
    --layout-features='*' \
    --output-file=SourceHanSerifK-OldHangul-subset.woff2 \
    --flavor=woff2 --no-hinting
```

검증:
- 357/368 옛한글 자모 codepoints 커버 (KTUG 매핑 target jamo 357/357 100%)
- GSUB features: ccmp + ljmo + vjmo + tjmo (모든 합자 피처 보존)

### 10.4 적용 위치

**WASM 웹 빌드만**:
- `rhwp-studio/public/fonts/SourceHanSerifK-OldHangul-subset.woff2`
- `rhwp-studio/src/core/font-loader.ts` 의 `FONT_LIST` 에 등록
- `unicode-range: U+1100-11FF, U+A960-A97F, U+D7B0-D7FF` 으로 옛한글 영역만 매칭 → 일반 한글 미영향

**네이티브 SVG 출력**:
- `src/renderer/mod.rs::generic_fallback` 의 한글 serif/sans-serif 체인 말단에 `'Source Han Serif K Old Hangul'` 추가
- 단독 SVG 사용 시 `--font-style` / `--embed-fonts` 옵션 또는 시스템에 폰트 설치 필요

### 10.5 변환 파이프라인

```
[원본 IR: PUA U+E38A]
       ↓ Composer / Renderer (Task #528)
[map_pua_old_hangul] — KTUG 매핑 표 룩업
       ↓
[KS X 1026-1:2007 자모 시퀀스: U+1103 U+119E]
       ↓
[font-family 체인 — 일반 한글 폰트는 cmap 부재로 fallback]
       ↓
[Source Han Serif K Old Hangul (unicode-range 매칭)]
       ↓
[CCMP/LJMO/VJMO/TJMO 합자 → 단일 음절 글리프]
```

### 10.6 매핑 표

KTUG HanyangPuaTableProject (Public Domain) — 5,660 매핑 (BMP PUA U+E0BC ~ U+F8F7).

자세한 내용: `mydocs/tech/pua_oldhangul_mapping_sources.md`.

### 10.7 미커버 영역

| 영역 | 처리 |
|------|------|
| Supplementary PUA-A (U+F0854/F0855 책괄호 등) | 본 task 외 — 별도 issue 권장 |
| 한컴 자체 PUA 기호 | Task #509 패턴 정합 (별도 매핑 필요) |
