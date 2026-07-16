# 수식 폰트 조사 및 선정 보고서

작성일: 2026-04-14
타스크: [#139](https://github.com/edwardkim/rhwp/issues/139)

## 목차

1. [배경](#1-배경)
2. [한컴 수식 폰트 히스토리](#2-한컴-수식-폰트-히스토리)
3. [오픈 라이선스 수식 폰트 후보 조사](#3-오픈-라이선스-수식-폰트-후보-조사)
4. [글리프 커버리지 평가](#4-글리프-커버리지-평가)
5. [Computer Modern 유사도 평가](#5-computer-modern-유사도-평가)
6. [최적 폰트 선정](#6-최적-폰트-선정)
7. [수식 내 한글 처리](#7-수식-내-한글-처리)
8. [font-family 체인 설계](#8-font-family-체인-설계)
9. [웹 폰트 배포 방식](#9-웹-폰트-배포-방식)
10. [적용 방안 설계](#10-적용-방안-설계)

---

## 1. 배경

현재 rhwp의 수식 렌더러(`svg_render.rs`, `canvas_render.rs`)는 `<text>` 요소에 `font-family` 속성을 지정하지 않는다. 브라우저 기본 폰트로 렌더링되어 수식 기호(∫, Σ, ∏ 등)와 그리스 문자의 표현 품질이 낮다.

수식에 최적화된 오픈 라이선스 폰트를 선정하여 렌더링 품질을 향상시킨다.

### 선정 기준

1. **한컴 수식 폰트(HyHwpEQ)와의 시각적 유사도** — HyHwpEQ가 Computer Modern 기반이므로 CM 계열 폰트 우선
2. **글리프 커버리지** — 그리스 문자(48종), 수학 기호(200+종), 큰 연산자, 화살표
3. **오픈 라이선스** — SIL OFL 또는 GUST Font License (웹 번들링 자유)
4. **파일 크기** — woff2 번들 시 합리적 용량
5. **OpenType MATH 테이블** — 수식 레이아웃 엔진 활용 가능성

---

## 2. 한컴 수식 폰트 히스토리

출처: [leesj.me/hwp-custom-equation](https://leesj.me/hwp-custom-equation/)

| 시기 | 폰트 | 기반 | 비고 |
|------|------|------|------|
| 한글 97~2002 | HSUSR.HFT ("수식") | 최초 수식 폰트 | HFT 형식 |
| 한글 2004~2007 | HyHwpEQ v1.10 | HY신명조 + **CMU Serif** | Computer Modern 기반 |
| 한글 2010~2022 | HyHwpEQ v1.13 | 로만체 일부 변경 | 세로폭 조정 |
| 한글 2018~ | HancomEQN | 태광서체 개발 | 새 폰트 도입 |

### HyHwpEQ 구조 분석

- **로만체**: HY신명조 + CMU Serif 혼합 배치
- **이탤릭 대문자**: CMU Serif 로만체를 **skew(기울임) 처리**로 생성 (별도 이탤릭 디자인 아님)
- **사용자 불만 사항**:
  - 로만체 알파벳과 이탤릭체의 굵기 불일치
  - 부등호, ± 기호 스타일 불균형
  - 근호(√) 기호 돌출
  - 수식 내 한글 폰트 변경 불가
  - 대괄호 크기 조정 어려움

**핵심**: 한컴 수식 폰트의 뿌리는 **Computer Modern (TeX/LaTeX 기본 폰트)**. 폰트 선정 시 CM 계열과의 호환성이 가장 중요한 기준이다.

---

## 3. 오픈 라이선스 수식 폰트 후보 조사

### 3.1 종합 비교표

| 폰트 | 라이선스 | 글리프 수 | OTF 크기 | 기반 디자인 | MATH 테이블 | CM 유사도 |
|------|---------|----------|---------|-----------|------------|----------|
| **Latin Modern Math** | GUST (=LPPL) | 4,802 | ~575 KB | **Computer Modern 직계** | ✓ | ★★★★★ |
| **New Computer Modern Math** | SIL OFL | 대규모 | ~32.5 MB (패키지) | **CM 확장판** | ✓ | ★★★★★ |
| **STIX Two Math** | SIL OFL 1.1 | 5,200+ | ~1.5 MB | Times 계열 독자 디자인 | ✓ | ★★ |
| **XITS Math** | SIL OFL 1.1 | ~3,550 | ~500 KB | STIX v1 기반 | ✓ | ★★ |
| **Libertinus Math** | SIL OFL 1.1 | ~4,000 | ~700 KB | Linux Libertine 파생 | ✓ | ★★★ |
| **Fira Math** | SIL OFL 1.1 | 2,094 | ~400 KB | Fira Sans (산세리프) | ✓ | ★ |
| **CMU Serif** | SIL OFL | ~1,000+ | ~1.6 MB | **CM 원본 유니코드 변환** | **✗** | ★★★★★ |
| **Asana Math** | SIL OFL | 대규모 | ~845 KB | Palatino 계열 | ✓ | ★★ |
| **TG Math Termes** | GUST | ~1,250 | ~400 KB | Times 계열 | ✓ | ★★ |
| **TG Math Pagella** | GUST | ~1,250 | ~400 KB | Palatino 계열 | ✓ | ★★ |
| **TG Math Bonum** | GUST | ~1,250 | ~400 KB | Bookman 계열 | ✓ | ★ |
| **TG Math Schola** | GUST | ~1,250 | ~400 KB | Century Schoolbook | ✓ | ★ |

### 3.2 개별 폰트 평가

#### Latin Modern Math ⭐ 권장

- **Computer Modern의 공식 현대화 버전** (GUST e-foundry 프로젝트)
- Donald Knuth의 CM을 OpenType으로 재구성, MATH 테이블 포함
- MathJax 4 기본 지원, MDN MathML 가이드 권장 폰트
- LaTeX 기본 수식 폰트의 웹 버전
- 4,802 글리프로 충분한 커버리지
- **HyHwpEQ와 동일한 Computer Modern 뿌리** → 시각적 일관성 최고

#### STIX Two Math ⭐ 폴백 권장

- STI Pub 컨소시엄 개발 (AMS, AIP, ACS, APS, IEEE 등)
- **가장 넓은 글리프 커버리지** (5,200+)
- macOS 13+ 기본 탑재 → Mac 사용자에게 추가 로딩 불필요
- MathJax 4 지원
- Times 계열이라 CM과는 시각적 차이가 있으나, 커버리지 보완용으로 최적

#### CMU Serif — 참고용 (사용 불가)

- CM 원본의 유니코드 변환 → HyHwpEQ와 가장 유사
- **OpenType MATH 테이블 없음** → 수식 레이아웃 엔진에서 활용 불가
- 일반 텍스트용으로만 사용 가능

#### Fira Math — 부적합

- 산세리프 디자인 → 한컴 수식(세리프)과 스타일 불일치
- 글리프 수 가장 적음 (2,094)
- 프레젠테이션용으로는 좋으나 문서 수식용으로 부적합

---

## 4. 글리프 커버리지 평가

### rhwp 수식에서 사용하는 핵심 글리프

| 분류 | 글리프 수 | 커버리지 필요 |
|------|----------|-------------|
| 그리스 소문자 | 29 | alpha~omega + 변형 |
| 그리스 대문자 | 25 | Alpha~Omega |
| 수학 연산자 | 40+ | ×, ÷, ±, ·, ∘ 등 |
| 관계 기호 | 30+ | ≠, ≤, ≥, ≈, ≡, ∝ 등 |
| 집합/논리 | 20+ | ∈, ⊂, ∪, ∩, ∀, ∃ 등 |
| 큰 연산자 | 15+ | ∫, ∬, ∭, ∮, ∑, ∏ 등 |
| 화살표 | 20+ | ←, →, ⇐, ⇒, ↦ 등 |
| 기타 기호 | 30+ | ∞, ∂, ∅, ∇, △ 등 |
| **합계** | **200+** | |

### 후보 폰트별 커버리지 (추정)

| 폰트 | 커버리지 | 평가 |
|------|---------|------|
| STIX Two Math | 99%+ | 유니코드 수학 기호 거의 전부 |
| Latin Modern Math | 95%+ | rhwp 필요 글리프 대부분 포함 |
| XITS Math | 90%+ | Mathematical Operators 97% |
| Libertinus Math | 85%+ | 일부 부족 |
| Fira Math | 70%+ | 글리프 수 최소 |

---

## 5. Computer Modern 유사도 평가

HyHwpEQ가 CMU Serif(Computer Modern) 기반이므로, CM 유사도가 곧 **한컴 수식과의 시각적 일관성**을 의미한다.

| 폰트 | CM 유사도 | 평가 |
|------|----------|------|
| **CMU Serif** | 최고 | CM 원본. MATH 테이블 없어 수식용 불가 |
| **Latin Modern Math** | 매우 높음 | CM 공식 현대화. MATH 테이블 포함. **최적** |
| **New Computer Modern Math** | 매우 높음 | Latin Modern 확장. 패키지 크기 과다 |
| **Libertinus Math** | 중간 | Linux Libertine 기반, 세리프 스타일 다름 |
| **STIX Two Math** | 낮음 | Times 계열, CM과 뚜렷한 차이 |
| **기타** | 낮음~없음 | 각 고유 디자인 계열 |

---

## 6. 최적 폰트 선정

### 6.1 선정 결과

| 순위 | 폰트 | 역할 | 근거 |
|------|------|------|------|
| **1순위** | **Latin Modern Math** | 주 수식 폰트 | CM 직계, MATH 테이블, 4,802 글리프, 적절한 크기 |
| **2순위** | **STIX Two Math** | 폴백 | 최대 글리프 커버리지, macOS 기본 탑재 |
| **3순위** | 시스템 serif | 최종 폴백 | 모든 환경에서 동작 보장 |

### 6.2 선정 근거 상세

**Latin Modern Math를 1순위로 선정한 이유**:

1. **한컴 수식 폰트(HyHwpEQ)와 동일한 Computer Modern 뿌리** → 기존 HWP 문서의 수식과 시각적으로 가장 일관적
2. **OpenType MATH 테이블 포함** → 향후 수식 레이아웃 엔진 고도화에 활용 가능
3. **LaTeX 기본 수식 폰트의 웹 버전** → LaTeX 입력 지원 시 자연스러운 렌더링
4. **MathJax 4, MDN MathML 권장** → 웹 수식 표준과 일관
5. **GUST Font License** → 웹 번들링 완전 자유
6. **합리적 파일 크기** → OTF ~575 KB, woff2로 변환 시 더 작음

**STIX Two Math를 폴백으로 선정한 이유**:

1. **가장 넓은 글리프 커버리지** → Latin Modern Math에 없는 기호 보완
2. **macOS 13+ 기본 탑재** → Mac 사용자에게 추가 로딩 불필요
3. **SIL OFL** → 번들링 자유

---

## 7. 수식 내 한글 처리

### 7.1 한글 사용 사례

수식 내에서 한글이 사용되는 경우:

| 사례 | 예시 | 빈도 |
|------|------|------|
| CASES 조건 설명 | `CASES{x+1 ~(x>0일때) # x-1 ~(x<0일때)}` | 중간 |
| 단위/주석 | `"평점"=입찰가격 OVER 예정가격` | 중간 |
| 수식 내 한글 변수 | `"속도"=거리 OVER 시간` | 낮음 |

### 7.2 현재 처리 방식

- 토크나이저(`tokenizer.rs`): 비-ASCII 연속 문자를 `TokenType::Text`로 처리
- 파서: `EqNode::Text`로 변환
- SVG 렌더러: `<text font-style="italic">한글텍스트</text>` — 이탤릭으로 렌더링
- Latin Modern Math에는 한글 글리프가 **없음** → 한글 부분은 반드시 CJK 폰트로 폴백

### 7.3 해결 방안

수식 폰트 체인에 한글 폴백을 포함해야 한다. 기존 `font_fallback_strategy.md`에서 수립한 CJK 폰트 체인을 수식에도 적용한다.

**방식: font-family 체인에 CJK 폰트 추가**

```css
font-family: "Latin Modern Math", "STIX Two Math", "Cambria Math",
             "Pretendard", "맑은 고딕", "Malgun Gothic", sans-serif;
```

- 영문/기호/그리스 문자 → Latin Modern Math에서 렌더링
- 한글 → Latin Modern Math에 글리프 없음 → Pretendard 또는 시스템 한글 폰트로 자동 폴백
- 브라우저의 font-family 체인 동작에 의해 글자별로 적절한 폰트가 자동 선택됨

### 7.4 한글 폴백 폰트 선정: Pretendard

| 기준 | Pretendard | 맑은 고딕 | 나눔고딕 |
|------|-----------|----------|---------|
| 라이선스 | SIL OFL 1.1 | MS 전용 | OFL |
| rhwp 번들 여부 | ✓ (woff2 이미 포함) | ✗ | ✗ |
| OS 커버리지 | 전체 (웹폰트) | Windows만 | 별도 설치 |
| 디자인 품질 | 우수 (Inter 기반) | 양호 | 양호 |

**Pretendard를 한글 폴백 폰트로 확정한다.** 이미 rhwp에 woff2로 번들되어 있으므로 추가 리소스 없이 Windows/Linux/Mac 모든 환경에서 일관된 한글 렌더링이 보장된다.

### 7.5 HWP 수식 폰트 속성 처리 방침

HWP 파일의 수식 컨트롤에는 `font_name` 속성이 포함되어 있다 (예: `HyHwpEQ`, `HancomEQN`). 이 속성은 **무시**하고, rhwp의 수식 전용 폰트 체인으로 통일한다.

**근거**:
1. HyHwpEQ, HancomEQN은 저작권 폰트 — 번들링/재배포 불가
2. 수식 렌더링 품질은 폰트 체인으로 충분히 보장
3. 사용자 환경에 한컴 폰트가 설치되어 있을 수 없으므로 참조해도 무의미
4. Latin Modern Math가 동일한 Computer Modern 뿌리이므로 시각적 일관성 유지

---

## 8. font-family 체인 설계

### 8.1 수식 전용 font-family

```css
/* 수식 전용 (영문/기호/그리스 + 한글 폴백) */
font-family: "Latin Modern Math", "STIX Two Math", "Cambria Math",
             "Pretendard", serif;
```

**체인 설명**:
1. `Latin Modern Math` — 1순위 오픈 폰트 (CM 계열, 영문/기호/그리스 문자)
2. `STIX Two Math` — 2순위 폴백 (macOS 기본 탑재)
3. `Cambria Math` — Windows 기본 수식 폰트 (설치되어 있으면 사용)
4. `Pretendard` — 한글 폴백 (OFL, rhwp에 이미 woff2 번들 포함, 모든 OS 커버)
5. `serif` — 최종 폴백

**동작 원리**: 브라우저는 font-family 체인에서 각 글자에 대해 글리프를 가진 첫 번째 폰트를 사용한다. 영문/기호는 Latin Modern Math에서, 한글은 Pretendard에서 자동으로 선택된다.

**HWP 수식 폰트 무시**: HWP 파일의 수식 `font_name` 속성(HyHwpEQ, HancomEQN 등)은 무시하고, 이 체인을 일률 적용한다.

### 8.2 적용 대상별 분류

| SVG LayoutKind | font-style | 설명 |
|---------------|------------|------|
| Text | italic | 변수 (x, y, z) |
| Number | normal | 숫자 (0-9) |
| Symbol | normal | 연산/관계 기호 (+, -, =) |
| MathSymbol | normal | 유니코드 수학 기호 (α, ∞, ×) |
| Function | normal | 함수명 (sin, cos, log) |
| BigOp | normal | 큰 연산자 (∫, Σ, Π) |

모든 유형에 동일한 font-family 체인을 적용한다. 수식 전용 폰트는 이미 수학적 글리프에 최적화되어 있으므로 유형별 분리가 불필요하다.

---

## 9. 웹 폰트 배포 방식

### 9.1 방식 비교

| 방식 | 장점 | 단점 | 적합 여부 |
|------|------|------|----------|
| **woff2 번들링** | 오프라인 동작, 일관된 렌더링 | 초기 로딩 크기 증가 | ✓ 권장 |
| **local() 우선 참조** | 설치된 폰트 활용, 로딩 없음 | 미설치 시 폴백 | ✓ 보조 |
| **CDN** | 캐시 공유 | 오프라인 불가, 외부 의존 | ✗ |

### 9.2 권장 방식: woff2 번들링 + local() 우선

```css
@font-face {
  font-family: "Latin Modern Math";
  src: local("Latin Modern Math"),
       url("/fonts/latinmodern-math.woff2") format("woff2");
  font-display: swap;
}
```

- `local()`: 사용자 시스템에 설치되어 있으면 네트워크 로딩 없이 사용
- `url()`: 미설치 시 번들된 woff2 로딩
- `font-display: swap`: 폰트 로딩 중에도 텍스트 표시 (FOUT 허용)

### 9.3 폰트 로딩 시점과 레이아웃 동기화

> 피드백 반영: `mydocs/feedback/task139-eq-01.md`

`font-display: swap`은 일반 텍스트에는 적합하지만, 수식 렌더러는 분수선 위치, 첨자 위치 등을 픽셀 단위로 정밀 계산한다. 웹폰트 로드 전에 기본 폰트로 레이아웃이 계산되면, 폰트 교체 후 기호가 겹치거나 여백이 틀어질 위험이 있다.

**대응 방안**: Canvas 렌더링 경로(rhwp-studio)에서는 수식 렌더링 시작 전 Web Font Loading API로 폰트 로딩 완료를 보장한다.

```javascript
// 수식 렌더링 전 폰트 로딩 보장
await document.fonts.load('1em "Latin Modern Math"');
// 이후 Canvas 수식 렌더링 시작
```

SVG 경로에서는 `<text>` 요소에 font-family만 지정하므로 브라우저가 폰트 로딩 후 자동 재렌더링한다. 단, 초기 표시 시 잠깐 폴백 폰트로 보일 수 있으므로(FOUT), 수식 영역에 대해 `font-display: block`을 고려할 수도 있다.

### 9.4 예상 번들 크기

| 폰트 | OTF 크기 | woff2 추정 | 비고 |
|------|---------|-----------|------|
| Latin Modern Math | ~575 KB | **~200-300 KB** | 주 수식 폰트 |
| STIX Two Math | ~1.5 MB | **~500-700 KB** | 폴백 (선택적) |

Latin Modern Math만 번들하면 ~200-300 KB 추가. 서브셋 추출 시 더 줄일 수 있으나, 수식에서 어떤 기호가 사용될지 예측하기 어려우므로 **전체 폰트 번들을 권장**한다.

---

## 10. 적용 방안 설계

### 10.1 SVG 렌더러 적용 (`svg_render.rs`)

현재 `<text>` 요소에 font-family가 없는 상태:

```xml
<!-- 현재 -->
<text x="10" y="20" font-size="14" fill="#000" font-style="italic">x</text>

<!-- 적용 후 -->
<text x="10" y="20" font-size="14" fill="#000" font-style="italic"
      font-family="Latin Modern Math, STIX Two Math, Cambria Math, serif">x</text>
```

**수정 대상 함수** (`svg_render.rs`의 `render_box()`):
- `LayoutKind::Text` — 이탤릭 변수
- `LayoutKind::Number` — 숫자
- `LayoutKind::Symbol` — 기호 (text-anchor="middle")
- `LayoutKind::MathSymbol` — 유니코드 수학 기호
- `LayoutKind::Function` — 함수명
- `LayoutKind::BigOp` — 큰 연산자 (기호 부분)
- `LayoutKind::Limit` — 극한 텍스트

**구현 방식**: font-family 문자열을 상수로 정의하고, 모든 `<text>` 생성 시 포함.

```rust
// 추가할 상수
const EQ_FONT_FAMILY: &str = r#"font-family="Latin Modern Math, STIX Two Math, Cambria Math, serif""#;
```

### 10.2 Canvas 렌더러 적용 (`canvas_render.rs`)

현재 `ctx.font`에 수식 폰트가 지정되지 않은 상태. Canvas 2D API에서는 `font` 속성 문자열에 font-family를 포함해야 한다.

```javascript
// 현재
ctx.font = "italic 14px serif";

// 적용 후
ctx.font = 'italic 14px "Latin Modern Math", "STIX Two Math", "Cambria Math", serif';
```

### 10.3 웹 폰트 로딩 (`rhwp-studio/`)

- `assets/fonts/` canonical source에 `LatinModernMath-Regular.woff2` 추가
- CSS 또는 JS에서 `@font-face` 선언
- 기존 `font-loader.ts` 구조에 통합

### 10.4 네이티브 SVG 출력 (`export-svg`)

네이티브 SVG 내보내기 시에도 동일한 font-family가 SVG에 포함된다. 사용자 시스템에 Latin Modern Math가 설치되어 있지 않으면 폴백 폰트로 렌더링된다.

기존 `--embed-fonts` 옵션과의 연동도 고려해야 한다:
- `--embed-fonts` 사용 시: Latin Modern Math의 사용 글리프를 서브셋 추출하여 SVG에 인라인
- `--font-style` 사용 시: `@font-face { src: local("Latin Modern Math") }` 참조 삽입

### 10.5 후속 구현 타스크

| 수정 대상 | 변경 내용 | 예상 규모 |
|----------|----------|----------|
| `svg_render.rs` | 모든 `<text>` 요소에 EQ_FONT_FAMILY 추가 | 소 (~20줄) |
| `canvas_render.rs` | `ctx.font`에 수식 폰트 반영 | 소 (~15줄) |
| `assets/fonts/` | `LatinModernMath-Regular.woff2` 추가 | 파일 1개 |
| `rhwp-studio/` | `@font-face` 선언 | 소 (~10줄) |
| 테스트 | 기존 수식 테스트 + SVG 시각 비교 | 중 |

---

## 참고 자료

- [한글 수식 커스텀 폰트 만들기](https://leesj.me/hwp-custom-equation/) — HyHwpEQ 폰트 구조 분석
- [STIX Fonts 공식](https://www.stixfonts.org/)
- [Latin Modern Math - GUST](https://www.gust.org.pl/projects/e-foundry/lm-math)
- [XITS GitHub](https://github.com/aliftype/xits)
- [Libertinus GitHub](https://github.com/alerque/libertinus)
- [MDN MathML Fonts 가이드](https://developer.mozilla.org/en-US/docs/Web/MathML/Guides/Fonts)
- [MathJax 4 Font Support](https://docs.mathjax.org/en/latest/output/fonts.html)
