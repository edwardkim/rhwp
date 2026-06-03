# 타스크 90: HWPX 파서 정확도 개선 — 최종 결과 보고서

## 배경
타스크 89에서 HWPX 파서를 구현한 후, python-hwpx 참조 파서 및 OWPML 스키마와 비교한 결과
다수의 파싱 누락·오류가 확인되어 정확도를 개선하는 작업을 수행하였다.

## 수행 단계

### 1단계: 공통 유틸리티 추출 + charPr/paraPr 보완
- `utils.rs` 신규 생성 — 공통 유틸리티 함수 13개 + 테스트 3개
- header.rs/section.rs 중복 함수 120줄 제거
- charPr: emboss(양각), engrave(음각) 비트 파싱 추가
- paraPr: breakSetting(widowOrphan, keepWithNext, keepLines, pageBreakBefore),
  autoSpacing(eAsianEng, eAsianNum), border offset(좌/우/상/하) 파싱 추가

### 2단계: section.rs 이미지/표/특수문자 보완
- **parse_picture 대폭 개선**: `<hp:pic>` 요소 속성, `<hp:pos>` 위치 속성,
  `<hp:outMargin>`/`<hp:inMargin>` 여백, `<hp:imgClip>` 크롭, `<hp:img>` effect 파싱 추가
- `<hp:columnBreak/>` 특수문자 처리 추가
- cellPr 속성 파싱 (borderFillIDRef, textDirection, vAlign)

### 3단계: borderFill 보완 + 글꼴 언어 매핑
- **글꼴 언어 그룹 수정**: `<hh:fontface lang="...">` 컨텍스트 추적으로
  7개 언어 그룹(한글/영어/한자/일어/기타/기호/사용자)에 정확히 매핑
- borderFill gradation 색상 목록 파싱
- imgBrush 이미지 배경 파싱 (fill mode, bin_data_id 등)
- slash(대각선) 파싱

### 4단계: 빌드 + SVG 검증 + 보고서
- 532개 Rust 테스트 통과
- WASM 빌드 + Vite 빌드 성공
- 5개 HWPX 샘플 SVG 내보내기 정상
- **이미지 0×0 버그 수정**: curSz/sz의 0값이 orgSz의 유효값을 덮어쓰는 문제 해결

## 개선된 파싱 항목 요약

| 영역 | 수정 전 | 수정 후 |
|------|---------|---------|
| 이미지 크기 | 일부 0×0 | orgSz 폴백으로 모든 이미지 정상 표시 |
| 이미지 위치 | 위치/텍스트흐름 미파싱 | pos, textWrap, outMargin, inMargin, imgClip 파싱 |
| 글꼴 매핑 | 모든 글꼴이 한글 그룹 | 7개 언어 그룹 정확 매핑 |
| paraPr | align, margin, lineSpacing만 | breakSetting, autoSpacing, border offset 추가 |
| charPr | 기본 속성만 | emboss, engrave 비트 추가 |
| borderFill | 4방향 선 + 단색만 | gradation 색상, imgBrush, slash 추가 |
| 특수문자 | lineBreak, tab | columnBreak 추가 |
| 표 셀 | cellPr 스킵 | cellPr 속성 파싱 |
| 코드 품질 | header/section 중복 함수 | utils.rs 공통 모듈 |

## 수정 파일

| 파일 | 변경 유형 | 변경 내용 |
|------|----------|----------|
| `src/parser/hwpx/utils.rs` | 신규 | 공통 유틸리티 13개 함수 + 테스트 |
| `src/parser/hwpx/mod.rs` | 수정 | `pub mod utils;` 추가 |
| `src/parser/hwpx/header.rs` | 수정 | fontface lang, charPr/paraPr/borderFill 보완 |
| `src/parser/hwpx/section.rs` | 수정 | 이미지/표셀/특수문자 보완 |

## 검증 결과
- `docker compose run --rm test` — **532개** Rust 테스트 통과
- `docker compose run --rm wasm` — WASM 빌드 성공
- `npm run build` — Vite 빌드 성공
- 5개 HWPX 샘플 SVG 내보내기 — 에러 없이 45페이지 정상 생성
- 0×0 이미지 — 수정 전 3개 → 수정 후 **0개**
