# Task 304 구현계획서: 쪽 번호 매기기 위치 기능 검증 및 버그 수정

## 1. 현황 분석

### 1.1 현재 구현 상태 (전문가 검토로 확인)
- **이미 구현됨**: `build_page_number()` (layout.rs:841), `format_page_number()` (utils.rs:85)
- 10가지 위치별 좌표 계산, 안쪽/바깥쪽 홀짝수 분기: 구현됨
- PageHide.hide_page_num 체크: 구현됨
- prefix/suffix/dash_char 처리: 구현됨

### 1.2 발견된 버그
1. **format 매핑 불일치**: `NumberFormat::from_hwp_format`이 HWP 스펙 표 136과 다름 (format 6~16)
2. **text_width 계산**: `String::len()`(UTF-8 바이트 수) 사용 → 한글/로마자에서 부정확
3. **dash_char 공백**: 한컴은 "- 1 -" 형태인데 현재 "-1-"로 출력될 수 있음

## 2. 구현 계획

### 2.1 단계 1: format 매핑 수정
- `NumberFormat::from_hwp_format()` 또는 `format_page_number()` 내부의 매핑을 표 136에 맞춤
- 검증: 다양한 format 값으로 쪽 번호 출력 확인

### 2.2 단계 2: text_width 및 dash 공백 수정
- `text_width = page_num_text.len()` → `page_num_text.chars().count()` 수정
- dash_char 전후 공백 추가 여부 한컴 동작과 비교

### 2.3 단계 3: 예제 파일로 렌더링 검증
- 쪽 번호가 있는 예제 파일로 SVG/WASM 렌더링 확인
- 위치별 (상/하/좌/중/우) 정상 배치 확인

## 3. 영향 범위
- `src/renderer/layout.rs` — build_page_number 내 text_width 수정
- `src/renderer/layout/utils.rs` — format_page_number dash 처리
- `src/renderer/mod.rs` — NumberFormat::from_hwp_format 매핑 수정

## 4. 검토 이력
- 전문가 검토: "미구현"이 아닌 "이미 구현됨" 확인 → 범위를 버그 수정으로 조정
- format 매핑 불일치, text_width 바이트 수 버그, dash 공백 누락 지적
