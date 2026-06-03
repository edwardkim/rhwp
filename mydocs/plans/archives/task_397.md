# Task 397: 고수준 텍스트 레이아웃 기술 리뷰 (SkParagraph + cosmic-text)

## 수행 목표

rhwp의 조판 시스템 체계화를 위한 기술 리뷰. 업계 표준 텍스트 레이아웃 엔진 두 가지를 분석하고, rhwp 조판 시스템의 근본적 개선 방향을 도출한다.

- **SkParagraph**: Google Skia의 고수준 텍스트 레이아웃 모듈 (Flutter, Chrome에서 사용)
- **cosmic-text**: System76(Pop!_OS)이 개발한 순수 Rust 텍스트 레이아웃 라이브러리

## 배경

### 문제 상황

rhwp는 HWP 뷰어로 시작하여 현재 에디터 기능까지 구현된 상태이다. 기존 한컴에서 작성된 문서를 불러온 후, 문단의 수정·추가·삭제와 같이 페이지 조판이 변경되어야 하는 경우 세부적인 버그가 다수 발생하고 있다.

**근본 원인**: 뷰어 시절의 텍스트 레이아웃 구조(사전 조판된 LINE_SEG 기반)가 에디터의 동적 재조판 요구사항과 맞지 않음. 문단 편집 시 줄바꿈 재계산, 텍스트 측정, 페이지네이션이 정확하게 연동되지 않아 레이아웃 불일치가 발생한다.

이에 조판 시스템의 체계화가 필요하며, 그 첫 단계로 업계 표준 텍스트 레이아웃 엔진을 분석하여 근본적인 해결 방향을 모색한다.

### rhwp 현재 텍스트 레이아웃 구조

| 구성요소 | 현재 구현 | 한계 |
|----------|-----------|------|
| 텍스트 측정 | 582개 내장 폰트 메트릭 + WASM JS Canvas 브릿지 | 등록 안 된 폰트는 CJK=1.0, Latin=0.5 휴리스틱 |
| 텍스트 셰이핑 | 없음 (문자별 독립 측정) | 리가처, 커닝, 컨텍스트 대체 미지원 |
| 줄바꿈 | 자체 구현 (한글 음절/영문 단어/CJK 문자) | 금칙문자 처리 있으나 UAX#14 완전 준수 아님 |
| 폰트 폴백 | 없음 (내장 메트릭 기반) | 미등록 폰트 휴리스틱 폴백 |
| BiDi | 없음 | RTL 텍스트 미지원 |
| 볼드 처리 | Faux Bold 보정 (em+10)/20 | 경험적 수치, 실제 글리프 메트릭 아님 |

### 검토 대상 기술 요약

| 항목 | SkParagraph | cosmic-text |
|------|-------------|-------------|
| 언어 | C++ (skia-safe Rust 바인딩 있음) | 순수 Rust |
| 셰이핑 | HarfBuzz + ICU | harfrust (HarfBuzz Rust 포팅) |
| 줄바꿈 | ICU 기반 | unicode-linebreak + 언어별 처리 |
| BiDi | ICU BiDi | unicode-bidi |
| 폰트 폴백 | 플랫폼별 + 커스텀 | fontdb + 플랫폼별 폴백 리스트 |
| WASM | CanvasKit으로 지원 | 지원 (no_std 옵션) |
| 라이선스 | BSD-3 | Apache-2.0 / MIT |

## 구현 계획

### 1단계: SkParagraph 심층 분석 (리서치)

- SkParagraph 아키텍처 분석
  - ParagraphBuilder → Paragraph → layout() → paint() 파이프라인
  - TextStyle / ParagraphStyle / StrutStyle 속성 매핑
  - LineMetrics, getRectsForRange(), getGlyphPositionAtCoordinate() 등 조회 API
- HWP 텍스트 속성과의 매핑 분석
  - HWP CharShape ↔ TextStyle
  - HWP ParaShape ↔ ParagraphStyle/StrutStyle
  - HWP LINE_SEG ↔ LineMetrics
- skia-safe Rust 바인딩 상태 및 WASM 빌드 가능성 조사
- 장단점 정리

### 2단계: cosmic-text 심층 분석 (리서치)

- cosmic-text 아키텍처 분석
  - FontSystem → Buffer → ShapeLine → LayoutLine 파이프라인
  - Attrs (속성), Metrics (메트릭), Wrap/Align 옵션
  - Editor 계층 (기본/구문강조/Vi)
- 핵심 기능 검증
  - 한글 텍스트 셰이핑 및 줄바꿈 동작
  - 폰트 폴백 메커니즘 (fontdb 기반)
  - BiDi 지원 수준
  - 캐싱 전략 (Shape/Layout/Font 3단계)
- WASM 빌드 가능성 및 제약사항 조사
- 장단점 정리

### 3단계: 비교 분석 및 rhwp 적용 방안 도출

- SkParagraph vs cosmic-text 비교표 작성
  - 기능 커버리지, 성능, WASM 호환성, 유지보수성, 라이선스
- rhwp 적용 시나리오 분석
  - **시나리오 A**: cosmic-text를 텍스트 측정/셰이핑 엔진으로 도입
  - **시나리오 B**: SkParagraph(skia-safe) 전면 도입
  - **시나리오 C**: cosmic-text 부분 도입 (셰이핑만, 레이아웃은 rhwp 유지)
  - **시나리오 D**: 현행 유지 + 선별적 개선
- 각 시나리오별 영향 범위, 마이그레이션 난이도, 리스크 분석
- 최종 기술 리뷰 보고서 작성 (`mydocs/tech/`)

## 산출물

- `mydocs/tech/text_layout_review.md` — 기술 리뷰 보고서
- `mydocs/working/task_397_step{N}.md` — 단계별 완료 보고서

## 비고

- 본 타스크는 리서치/분석 타스크로, 코드 변경 없음
- 결론에 따라 후속 구현 타스크 등록 예정
