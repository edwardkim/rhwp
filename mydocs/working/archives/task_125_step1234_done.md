# 타스크 125 — 1~4단계 통합 완료 보고서

## 완료 내용

### 1단계: TTF 테이블 파싱 도구 (font_metric_gen.rs)
- 순수 Rust로 TTF 바이너리 파싱 구현 (외부 크레이트 없음)
- 파싱 테이블: head (unitsPerEm, macStyle), maxp (numGlyphs), cmap (Format 4/12), hmtx, hhea, name
- TTC (TrueType Collection) 지원
- bold/italic 속성 추출 (head.macStyle)
- 601개 TTF 파일 전부 파싱 성공 (실패 0)

### 2단계: 한글 음절분해 압축
- 초(19)×중(21)×종(28) 그룹 K-means 클러스터링
- 고정폭 폰트(대부분): 1×1×1 = 1 대표 폭 (오차 0)
- 가변폭 폰트(Yj 시리즈 등): 최대 4×6×3 = 72 대표 폭 (오차 < 1%)
- 160개 한글 폰트 메트릭 생성

### 3단계: Rust 소스코드 생성 + WASM 내장
- `font_metrics_data.rs` 자동 생성 (~9,800줄)
- 582개 폰트 엔트리 (중복 제거: 601 → 582)
- 구조체: FontMetric, HangulMetric, LatinRange
- 조회 함수: `find_metric(name, bold, italic)` — 정확매칭 → bold매칭 → Regular 폴백
- HWP 기본 폰트 + 태블릿 폰트 배열 최상단 배치 (검색 최적화)

### 4단계: layout.rs 측정 파이프라인 교체
- `measure_char_width_embedded()` 신규 함수: 내장 메트릭으로 즉시 반환
- `measure_char_width_hwp()`: 내장 메트릭 1차 → JS 브릿지 2차 폴백
- `measure_hangul_width_hwp()`: 내장 메트릭 1차 → JS 폴백
- `estimate_text_width()`, `compute_char_positions()`: 양쪽 모두 내장 메트릭 사용
- 네이티브 빌드: 히우리스틱 대신 내장 메트릭 사용

### 추가: Noto CJK 지원
- Noto Sans KR (Regular + Bold), Noto Serif KR (Regular + Bold) 추가
- Android/Chrome OS/태블릿 환경 대응
- 23,174 글리프, 한글 11,172 전수 커버
- 고정폭 한글 → 메트릭 데이터 극소 (+3 KB WASM 증분)

## 변경 파일

| 파일 | 변경 | 상태 |
|------|------|------|
| `src/tools/font_metric_gen.rs` | TTF 파싱 CLI 도구 | 신규 |
| `src/renderer/font_metrics_data.rs` | 582 폰트 메트릭 DB (자동 생성) | 신규 |
| `src/renderer/mod.rs` | font_metrics_data 모듈 등록 | 수정 |
| `src/renderer/layout.rs` | 내장 메트릭 우선 사용 파이프라인 | 수정 |
| `Cargo.toml` | font-metric-gen 바이너리 타겟 | 수정 |
| `mydocs/tech/investigations/issue-124/font_metrics_size_comparison.md` | 3사 크기 비교 (Noto 포함) | 신규 |

## 검증 결과

| 항목 | 결과 |
|------|------|
| 571개 회귀 테스트 | 전부 통과 |
| WASM 빌드 | 성공 |
| 네이티브 빌드 | 성공 |
| WASM 크기 | 1.83 MB (+475 KB, +34%) |

## 크기 비교 요약

| | rhwp | 한컴 | 일부 상용 웹오피스 |
|--|------|------|-------------------|
| WASM | **1.83 MB** | N/A | ~19 MB |
| 폰트 패밀리 | **386개** | 342개 | 미상 |
| 폰트 변형 | **582개** | 387개 | 미상 |
| 한글 메트릭 | **160개** | ~120개 | 미상 |

## 우선 배치 폰트 (배열 최상단)

1. HCR Batang/Dotum (함초롬바탕/돋움) — HWP 기본
2. Malgun Gothic (맑은 고딕) — Windows 시스템 기본
3. Haansoft Batang/Dotum — 한소프트 레거시
4. NanumGothic/NanumMyeongjo — 인기 한글 폰트
5. Noto Sans KR / Noto Serif KR — Android/태블릿 기본
6. Arial, Times New Roman, Calibri, Verdana, Tahoma — 주요 영문
7. Batang, Dotum, Gulim, Gungsuh — 레거시 한글

## 기대 효과
- 등록 폰트(582개) 사용 시 JS 브릿지 호출 **0회** (100% 절감)
- 네이티브 빌드에서도 정확한 텍스트 폭 측정 가능
- 한글 음절별 개별 폭 제공 (기존: '가' 하나로 대표)
- Android/Chrome OS 태블릿 환경 Noto CJK 지원
