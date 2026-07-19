---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-397/README.md
last_verified: 2026-07-17
---

# 텍스트 레이아웃 기술 리뷰 보고서

> Task 397 | 2026-03-28 | 고수준 텍스트 레이아웃 기술 리뷰 (SkParagraph + cosmic-text)

## 1. 목적

rhwp의 조판 시스템 체계화를 위한 업계 표준 텍스트 레이아웃 엔진 기술 리뷰.

**문제 상황**: rhwp는 HWP 뷰어로 시작하여 에디터 기능까지 구현되었으나, 문단 편집(수정·추가·삭제) 시 페이지 조판 변경 과정에서 세부 버그가 다수 발생. 뷰어 시절의 사전 조판(LINE_SEG) 기반 구조가 에디터의 동적 재조판 요구사항과 맞지 않는 것이 근본 원인.

## 2. 검토 대상

| 기술 | 설명 | 버전 |
|------|------|------|
| **SkParagraph** | Google Skia 텍스트 레이아웃 모듈 (C++, Flutter/Chrome 사용) | Skia HEAD |
| **cosmic-text** | System76 순수 Rust 텍스트 레이아웃 (COSMIC 데스크톱) | v0.18.2 |
| **parley** | linebender 순수 Rust 리치 텍스트 레이아웃 (NLnet 지원) | v0.8.0 |

## 3. rhwp 현행 구조와 한계

### 아키텍처

```
텍스트 입력 → 내장 폰트 메트릭(582개) → 문자별 독립 폭 측정
           → 자체 줄바꿈 (한글 음절/영문 단어/CJK 문자)
           → LINE_SEG 기반 줄 배치
           → 문단 레이아웃 (여백, 들여쓰기, 정렬, 탭)
           → 페이지네이션
           → SVG/Canvas 렌더링
```

### 핵심 한계

| 구성요소 | 현재 | 한계 |
|----------|------|------|
| 텍스트 측정 | 내장 메트릭 + WASM JS Canvas | 미등록 폰트 → 휴리스틱 (CJK=1.0, Latin=0.5) |
| 셰이핑 | 없음 | 커닝, 리가처, 컨텍스트 대체 미지원 |
| 줄바꿈 | 자체 구현 | UAX#14 불완전 |
| 폰트 폴백 | 없음 | 미등록 폰트 사용 불가 |
| BiDi | 없음 | RTL 미지원 |
| 볼드 | Faux Bold 경험적 보정 | 실제 글리프 메트릭 아님 |
| 편집 시 재조판 | LINE_SEG 재계산 | 줄바꿈 재계산과 LINE_SEG 재생성 간 불일치 |

## 4. 기술 비교

### 4.1 텍스트 셰이핑

| | SkParagraph | cosmic-text | parley | rhwp |
|-|-------------|-------------|--------|------|
| 엔진 | HarfBuzz (C++) | harfrust (Rust) | harfrust (Rust) | 없음 |
| 커닝 | ◎ | ◎ | ◎ | ✗ |
| 리가처 | ◎ | ◎ | ◎ | ✗ |
| 복잡 스크립트 | ◎ | ◎ | ◎ | ✗ |
| OpenType 기능 | ◎ | ◎ | ◎ | ✗ |

### 4.2 줄바꿈 / BiDi

| | SkParagraph | cosmic-text | parley | rhwp |
|-|-------------|-------------|--------|------|
| 줄바꿈 | ICU (C++) | unicode-linebreak | ICU4X (Rust) | 자체 구현 |
| BiDi | ICU BiDi | unicode-bidi | ICU4X | 없음 |
| 한글 줄바꿈 | ◎ | ○ | ◎ (ICU4X) | ○ (자체) |

### 4.3 WASM 호환성

| | SkParagraph | cosmic-text | parley | rhwp |
|-|-------------|-------------|--------|------|
| wasm32-unknown-unknown | ✗ | △ | △ | ◎ |
| wasm-pack 호환 | ✗ | △ | △ | ◎ |
| 순수 Rust | ✗ (C++ FFI) | ◎ | ◎ | ◎ |
| 빌드 복잡도 | 극도로 높음 | 낮음 | 낮음 | — |

### 4.4 HWP 고유 기능 호환

| HWP 기능 | SkParagraph | cosmic-text | parley | rhwp |
|----------|-------------|-------------|--------|------|
| 장평 (ratios) | ✗ | ✗ | ✗ | ◎ |
| 언어별 자간 (spacings[7]) | ✗ | ✗ | ✗ | ◎ |
| 언어별 폰트 (font_ids[7]) | 폴백 의존 | 폴백 의존 | 폴백 의존 | ◎ |
| 4종 줄간격 | ✗ | ✗ | ✗ | ◎ |
| 문단 여백/들여쓰기 | ✗ | ✗ | ✗ | ◎ |
| 문단 간격 | ✗ | ✗ | ✗ | ◎ |
| 커스텀 탭 스톱 | 제한적 | 고정만 | 미확인 | ◎ |
| 번호/글머리표 | ✗ | ✗ | ✗ | ◎ |
| 강조점/양각/음각 | ✗ | ✗ | ✗ | ◎ |

## 5. 적용 시나리오

### 시나리오 A: cosmic-text 전면 도입 → **비권장**
- HWP 고유 기능 대부분 재구현 필요, WASM 폰트 로딩 문제, 기존 테스트 파괴

### 시나리오 B: SkParagraph 전면 도입 → **불가**
- `wasm32-unknown-unknown` 미지원, WASM 빌드 불가

### 시나리오 C: 셰이핑 엔진(harfrust)만 선별 도입 → **중기 권장**
- TextMeasurer에 harfrust 기반 구현체 추가
- 줄바꿈/문단 레이아웃/페이지네이션은 rhwp 자체 구현 유지
- 핵심 문제(텍스트 측정 부정확)를 최소 영향으로 해결
- WASM: harfrust + 폰트 번들 or JS Canvas 하이브리드

### 시나리오 D: 현행 유지 + 선별적 개선 → **단기 권장**
- LINE_SEG 재계산 버그 수정에 집중
- 내장 폰트 메트릭 보강
- 근본적 한계(셰이핑 없음)는 해소 불가

## 6. 최종 판단

### 외부 엔진 도입 부적합 판단

검토 대상 3종(SkParagraph, cosmic-text, parley)은 모두 **소스코드 에디터 또는 RTF 수준의 텍스트 위젯**을 위한 라이브러리로, HWP 문서 조판 엔진이 요구하는 수준과 범위가 다르다.

**HWP 포맷의 본질적 특성:**
- HWP는 문자별 독립 측정 방식 (CharShape의 언어별 font_ids/spacings/ratios가 이를 전제)
- OpenType 셰이핑(커닝/리가처)이 아닌 자간/장평 기반 폭 제어
- 따라서 harfrust 등 셰이핑 엔진 도입은 HWP 호환성 관점에서 불필요

**문서 조판 엔진과의 격차:**
- 검토 대상들은 단일 텍스트 블록의 셰이핑+줄바꿈+렌더링이 목적
- HWP가 요구하는 페이지네이션, 표 행 분할, 다단 레이아웃, 머리말/꼬리말/바탕쪽, 각주/미주, 개체 배치(TopAndBottom/TAC) 등은 모두 범위 밖
- 비교 대상이 되려면 LibreOffice Writer 엔진, TeX, typst 수준이어야 함

### 자체 구현 결정

rhwp의 조판 시스템은 **자체 구현을 유지·강화**한다.

**근거:**
1. HWP의 문자별 독립 측정 방식에는 현행 내장 폰트 메트릭 기반이 적합
2. textRun → 줄 정렬 → 줄 개행 → 페이지네이션으로 이어지는 파이프라인이 HWP 고유 규칙(4종 줄간격, 문단 간격, 금칙문자, 탭 정의, 번호/글머리표 등)과 밀접하게 결합되어 있어 외부 엔진으로 분리 불가
3. 편집 시 발생하는 조판 버그의 근본 원인은 셰이핑 부재가 아니라 **LINE_SEG 재계산과 페이지네이션 연동 로직**의 문제

### 후속 방향

편집 시 조판 버그 해결을 위한 **LINE_SEG 재계산 + 페이지네이션 재조판 로직 체계화**를 후속 타스크로 진행한다.

- LINE_SEG 재생성 시 내장 폰트 메트릭과 원본 LINE_SEG 간 정합성 보장
- 문단 편집(추가/삭제/수정) 후 영향 범위 산정 및 점진적 재조판
- 페이지네이션 재계산과 레이아웃 동기화

## 7. 참고 자료

- [Skia Text API Overview](https://skia.org/docs/dev/design/text_overview/)
- [SkParagraph 소스](https://github.com/google/skia/tree/main/modules/skparagraph)
- [skia-safe Rust 바인딩](https://rust-skia.github.io/doc/skia_safe/textlayout/type.Paragraph.html)
- [cosmic-text GitHub](https://github.com/pop-os/cosmic-text)
- [cosmic-text API 문서](https://docs.rs/cosmic-text)
- [cosmic-text DeepWiki](https://deepwiki.com/pop-os/cosmic-text)
- [parley GitHub](https://github.com/linebender/parley)
- [W3C 한국어 텍스트 레이아웃 요구사항 (klreq)](https://www.w3.org/TR/klreq/)
- [skia-safe WASM 이슈 #855](https://github.com/rust-skia/rust-skia/issues/855)
- [skia-safe WASM 이슈 #1078](https://github.com/rust-skia/rust-skia/issues/1078)
