---
kind: reference
status: active
canonical: mydocs/tech/hwpx_hancom_reference.md
last_verified: 2026-07-17
---

# 한컴 공식 OWPML 모델 참조 가이드

- **작성일**: 2026-04-17
- **대상 프로젝트**: [hancom-io/hwpx-owpml-model](https://github.com/hancom-io/hwpx-owpml-model)
- **원본 저장소**: <https://github.com/hancom-io/hwpx-owpml-model>
- **라이선스**: Apache License 2.0

## 1. 프로젝트 정체

한컴이 공개한 **HWPX(OWPML) 공식 오픈소스 모델**이다. Microsoft Visual Studio C++ 프로젝트로 구성되어 있으며, Expat XML 파서와 libzip을 내장한 정적 라이브러리 형태다.

### 규모

| 항목 | 수치 |
|---|---|
| 소스 파일 | 683개 (.h/.cpp) |
| 총 라인 | 약 91,054 LOC |
| 클래스 수 | 292개 |
| 열거형 정의 | 약 100개 (`enumdef.h` 2,245줄) |

### 폴더 구조

```
OWPML/
├─ Base/            — SAX 파싱, XMLSerializer, CExtObject 기반 클래스
├─ Class/
│  ├─ Head/  (107) — header.xml 요소: 폰트, 스타일, 마진, 테두리
│  ├─ Para/  (134) — section{N}.xml 요소: 문단, 표, 이미지, 도형
│  ├─ Core/  (10)  — 공통 타입: Color, HWPValue, FillBrush 등
│  ├─ Etc/   (36)  — container/version/manifest 메타
│  └─ RDF/   (1)   — RDF 메타데이터
├─ Expat/           — libexpat 2.0.1/2.1.0 (XML 파서)
├─ Zip/             — SDZip (ZIP 컨테이너)
└─ Hwp/include/    — HwpMetatagDef.h 등
```

## 2. 핵심 한계 — "읽기 전용"

`README.md`와 `OWPMLTest/OWPMLTest.cpp` 확인 결과 이 공개 프로젝트는 **텍스트 추출 수준의 읽기 전용**이다.

| 기능 | 지원 |
|---|---|
| HWPX 파싱 (OWPML 객체 모델로 로드) | ✅ |
| 텍스트 추출 (`GetSectionText()`) | ✅ |
| 객체 모델 순회 | ✅ |
| **편집** | ❌ |
| **라운드트립(수정 후 저장)** | ❌ |
| **렌더링 엔진** | ❌ |
| **변경 추적(diff)** | ❌ |

즉 **한컴 공개본은 rhwp의 경쟁자가 아니라 rhwp의 스펙 참조원이다**. 실제 한컴2020 내부 엔진이 아니라 API 호환성을 위한 **공개용 축약본**으로 파악된다.

이 한계는 역설적으로 rhwp의 위치를 분명히 한다:
- **rhwp는 한컴 공개본이 안 하는 영역(편집·라운드트립·렌더링·WASM)을 다룬다**
- **rhwp는 한컴 공개본이 잘 하는 영역(속성 커버리지·기본값·enum 정의)을 참조한다**

## 3. 참조 가치가 있는 파일

### 3.1 `enumdef.h` — HWPX enum 값의 정답

[`OWPML/Class/enumdef.h`](https://github.com/hancom-io/hwpx-owpml-model/blob/main/OWPML/Class/enumdef.h)
(2,245줄)

HWPX 속성 값에 쓰이는 **모든 enum의 공식 정의**. rhwp의 `src/model/*.rs`에 있는 enum들이 이와 일치하는지 검증하는 기준.

예시:

```cpp
// SYMBOLMARKTYPE (enumdef.h:83)
enum {
    SMT_NONE = 0,
    SMT_DOT_ABOVE,
    SMT_RING_ABOVE,
    SMT_TILDE,
    // ...
};

// LSTYPE — lineSpacing 종류 (enumdef.h:587)
enum {
    LST_PERCENT = 0,
    LST_FIXED,
    LST_BETWEEN_LINES,
    LST_AT_LEAST,
};

// LINETYPE2 — border line 종류 (enumdef.h:133)
enum {
    LT2_NONE = 0, LT2_SOLID, LT2_DOT, LT2_DASH,
    LT2_DASH_DOT, LT2_DASH_DOT_DOT, LT2_LONG_DASH,
    LT2_CIRCLE, LT2_DOUBLE_SLIM, LT2_SLIM_THICK,
    // ...
};

// ALIGN
enum {
    AH_JUSTIFY = 0, AH_LEFT, AH_RIGHT, AH_CENTER,
    AH_DISTRIBUTE, AH_THAI_DISTRIBUTED,
    AV_BASELINE = 0, AV_TOP, AV_CENTER, AV_BOTTOM,
};
```

### 3.2 `Class/Head/*.cpp` — 뼈대 층 기본값의 정답

각 클래스의 constructor 초기화 리스트가 **HWPX 속성의 공식 기본값**이다.

**예: `CharShapeType.cpp:31`**
```cpp
CCharShapeType::CCharShapeType() : CExtObject(ID_HEAD_CharShapeType),
    m_nHeight(1000),
    m_cTextColor(0x000000),
    m_cShadeColor(0xFFFFFF),
    m_bUseFontSpace(false),
    m_bUseKerning(false),
    m_uSymMark(SMT_NONE),
    m_uId(0),
    m_uBorderFillIDRef(0)
```

**예: `ParaShapeType.cpp:31`**
```cpp
CParaShapeType::CParaShapeType() : CExtObject(ID_HEAD_ParaShapeType),
    m_uCondense(0),
    m_bFontLineHeight(false),
    m_bSnapToGrid(true),           // ← 유일한 true 기본값
    m_bSuppressLineNumbers(false),
    m_bChecked(false),
    m_uTabPrIDRef(0)
```

**핵심 규칙**:
- 대부분의 bool 속성: **`false`**
- 대부분의 UINT 속성: **`0`**
- 대부분의 enum 속성: **첫 번째 값 (보통 NONE/0)**
- 색상: **`0x000000` (textColor)**, **`0xFFFFFF` (shadeColor)**
- **예외**: `snapToGrid = true` — 이것만 기본값이 `true`

### 3.3 `Class/` 하위 클래스 집합 — 태그 전체 목록

`find OWPML/Class -name "*.h" | xargs -n1 basename | sed 's/\.h$//'` 을 실행하면 292개 클래스명이 나온다. 이것이 **HWPX 포맷의 공식 태그 전체 집합**이며, rhwp 파서 커버리지의 상한선이다.

## 4. "필수 vs 선택" 구분 규칙

한컴 코드는 속성의 필수/선택을 **명시적으로 구분하지 않는다**:

1. 모든 속성이 constructor에서 기본값으로 초기화됨
2. `ReadAttribute()`: XML 속성이 누락되면 **constructor 기본값 유지**
3. `WriteElement()`: **모든 속성을 항상 출력** (값이 0이든 false든)

**따라서**:
- **실질적으로 모든 속성이 "선택"**이다
- **누락 시 한컴 기본값이 쓰인다**
- rhwp 직렬화는 **한컴과 동일하게 모든 속성을 항상 출력**하면 안전하다

## 5. rhwp와의 구조 비교

| 관점 | 한컴 OWPML | rhwp |
|---|---|---|
| 언어 | C++ | Rust |
| 패러다임 | OOP (292개 전용 클래스) | IR + 함수형 Writer |
| 파싱 | SAX (Expat) | SAX (quick-xml) |
| 직렬화 | `CExtObject` 가상 함수 기반 | `HwpxZipWriter` + `utils.rs` 헬퍼 |
| 메모리 모델 | 객체 트리 | IR 구조체 + ID 참조 테이블 |
| 편집 지원 | ❌ | ✅ |
| 렌더링 | ❌ | ✅ (SVG/Canvas) |
| 라운드트립 | ❌ | 목표 (#182) |
| WASM | ❌ | ✅ |

**중요**: rhwp의 **IR 기반 설계가 한컴 OWPML보다 우수한 점**이 있다. 한컴은 XML 구조를 클래스로 1:1 매핑해 메모리 비효율적이다. rhwp는 스타일을 ID로 참조하고 테이블에서 관리 → **메모리 효율 + 일관성 보장**.

## 6. 태그 커버리지 현황

조사 결과, rhwp 파서는 292개 중 **약 283개를 다루고 있음**으로 추정된다 (`src/parser/hwpx/section.rs`, `header.rs`의 매칭 태그 기준).

### 한컴만 정의, rhwp 미다룸 (216개 중)

| 분류 | 개수 | rhwp 로드맵 위치 |
|---|---|---|
| 추상 타입 (`Abstract*Type`) | 37 | 내부 사용, 직렬화 불필요 |
| 고급 렌더링 효과 (EffectsType, Scale, Skew) | 12 | 2단계 |
| 변경 추적 (Diff, Insert, Update, HWPMLHistory) | 7 | 3단계 (협업 기능) |
| 양식 컨트롤 (ComboBox, ListBox, ScrollBar) | 6 | 2단계 |
| 차트·OLE·수식 고급 | 6 | 2/3단계 |
| 메타데이터·MasterPage | 5 | 2단계 |
| 고급 도형 (ConnectLine, Container) | 10 | 2단계 |
| 호환성 플래그 (`do*`, `apply*`, `use*`) | 130+ | 생략 가능 (한컴2020 내부 호환모드) |

**1단계(#182) 목표 범위**:
- **HWP 수준** = 기본 텍스트·문단·표·이미지·스타일·글꼴
- 한컴 공개본 수준을 이미 상회하는 영역 (편집·라운드트립·렌더링)

## 7. 라이선스적 고려

### Apache 2.0의 의미

1. **상업 사용 가능** — rhwp는 오픈소스이나 향후 상업 파생물도 법적으로 가능
2. **수정 표기 필수** — 한컴 코드를 직접 사용·수정·재배포 시 Apache 2.0 고지
3. **특허 조항** — 한컴이 해당 코드에 대한 특허 소송을 제기하지 못하도록 보호

### rhwp에서의 활용 범위

| 활용 유형 | 허용 여부 | 비고 |
|---|---|---|
| 코드 직접 복사·변환 (C++ → Rust) | ✅ (Apache 2.0 고지 필수) | 실제로는 의미 낮음 — 언어·패러다임 다름 |
| **클래스명·속성명 참조** | ✅ | XML 포맷 스펙의 공적 정보 |
| **기본값 참조** (constructor 초기화) | ✅ | 스펙 정보로 간주 |
| **enum 값 참조** | ✅ | 호환성 보장 |
| **아키텍처 설계 참조** | ✅ | 공적 아이디어 |

### 표기 방침

rhwp가 한컴 소스에서 추출한 기본값·enum 값을 쓸 때는 **파일 헤더 주석**에 다음을 명시한다:

```rust
//! Default values and enum definitions referenced from
//! hancom-io/hwpx-owpml-model (Apache License 2.0).
//! See mydocs/tech/hwpx_hancom_reference.md for details.
```

**전용 파일**: `src/serializer/hwpx/canonical_defaults.rs` (1단계 Stage 0에서 추가)

## 8. 시사점 요약

### rhwp 관점의 한컴 프로젝트 가치

1. **스펙 참조원**: enum·기본값·태그 전체 집합의 권위 있는 공식 출처
2. **커버리지 검증 도구**: rhwp의 태그 커버리지 실측 가능 (292개 기준)
3. **뼈대 층 정답**: Stage 0의 `canonical_defaults.rs`에 추출할 기본값 정본

### 한컴 프로젝트로 **대체되지 않는** rhwp 영역

1. **편집·라운드트립** — 한컴 공개본 미지원
2. **렌더링 엔진** — 한컴 공개본 미지원
3. **WASM/웹 에디터** — 한컴 공개본 미지원
4. **Rust 생태계 통합** — 한컴은 C++
5. **리눅스·macOS 지원** — 한컴은 Windows 전용

### 1단계(#182)에서의 활용 방법

| 영역 | 활용 |
|---|---|
| Stage 0 | `canonical_defaults.rs`에 한컴 기본값 추출 |
| Stage 1 (header.xml) | 한컴 `Class/Head/*.cpp` 속성 목록 대조 |
| Stage 2 (section.xml) | 한컴 `Class/Para/*.cpp` 속성 목록 대조 |
| Stage 3/4 (표·이미지) | 한컴 `tbl.cpp`, `pic.cpp` 속성 커버리지 검증 |
| Stage 5 | 한컴 미구현 영역(편집) = rhwp 차별화 강점으로 문서화 |

## 9. 관련 자료

- **프로젝트 URL**: https://github.com/hancom-io/hwpx-owpml-model
- **OWPML 공식 스펙 문서**: https://www.hancom.com/etc/hwpDownload.do
- **rhwp 관련 문서**:
  - `mydocs/tech/hwp_hwpx_ir_differences.md` — HWP↔HWPX IR 차이점
  - `mydocs/plans/task_m100_182.md` — HWPX Serializer 완성 수행계획
  - Discussion [#183](https://github.com/edwardkim/rhwp/discussions/183) — HWPX 포맷 관찰
  - Discussion [#184](https://github.com/edwardkim/rhwp/discussions/184) — LLM 기반 HWPX 생성의 함정
