# Task 400 — 완료 보고서: 텍스트 폭 측정 정합성

> 2026-03-28~29

## 수정 사항

### 1. 윈도우 시스템 폰트 메트릭 추가
- Batang(바탕체), Gulim(굴림) 메트릭 내장 DB에 추가
- font-metric-gen .ttc 파일 지원 추가
- 별칭 매핑: 바탕체→Batang, 굴림/굴림체/돋움체→Gulim

### 2. 글자 단위 한글/CJK 줄바꿈 가능 지점 처리
- `fill_lines()`에서 단일 문자 CJK/한글 토큰을 break point로 처리
- `korean_break_unit=1`(글자 단위)일 때만 한글 break point 설정
- CJK 한자/일본어는 항상 break point

### 3. 내어쓰기(indent<0) 줄바꿈 폭 계산 수정
- 이후 줄의 effective_width = available_width + indent (좁아짐)
- 이전: indent<0 무시 (`.max(0.0)` 클램프)

### 4. HWPUNIT 정수 기반 줄바꿈 엔진
- `fill_lines()`의 `line_width`를 f64(px) → i32(HWPUNIT) 완전 전환
- 한컴의 HWPUNIT 정수 연산과 동일한 줄바꿈 판정
- `measure_char_width_embedded`: round → truncate

### 5. 토큰화에 unrounded 폭 사용
- `tokenize_paragraph`에서 `estimate_text_width_unrounded` 사용
- 단일 문자 round(6.667→7.0) 문제 해소 → 공백 525→500 HU 정확화

### 6. 줄바꿈 허용 오차 15 HU
- 한컴의 HWPUNIT 양자화 미세 차이를 허용 (15 HU ≈ 0.2mm)

## 일치율 변화

### 통제 샘플 (lseg-01 ~ lseg-06)

| 샘플 | 줄 수 (전→후) | 줄바꿈 (전→후) | 전체 (전→후) |
|------|-------------|---------------|-------------|
| lseg-01-basic | 100→**100** | 0→**100** | 0→**100** |
| lseg-02-mixed | 100→100 | 100→100 | 100→100 |
| lseg-03-spacing | **0→100** | 0→**100** | 0→**100** |
| lseg-04-indent | **75→100** | 0→25 | 0→25 |
| lseg-05-tab | 100→100 | 0→**100** | 0→**100** |
| lseg-06-multisize | 100→100 | 0→**100** | 0→**60** |

### 기존 샘플 (8개 파일, 45개 문단)

| 지표 | 전 | 후 |
|------|-----|-----|
| 줄 수 일치율 | 88.9% | 86.7% |
| 줄바꿈 위치 일치율 | 64.4% | 60.0% |
| 전체 필드 일치율 | 2.2% | 2.2% |

기존 샘플 하락 원인: Batang/Gulim 메트릭 추가로 hongbo.hwp 문단 17의 라틴 폭이 변경. HFT 폰트 vs 윈도우 폰트 메트릭 충돌.

## 잔여 문제

| No | 문제 | 상태 |
|----|------|------|
| B-018 | 탭+양쪽정렬 공백 분배 오버플로 | 백로그 등록 |
| - | lseg-04 text_start ±1~3 차이 (내어쓰기 문단) | Task 399 |
| - | lseg-06 line_height/baseline 줄별 차이 | Task 401 |
| - | 기존 샘플 HFT 폰트 메트릭 정밀도 | 추가 조사 |
