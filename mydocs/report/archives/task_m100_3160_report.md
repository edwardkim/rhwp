# Task m100 처리결과: #3160 html import CSS rgba()·border-width 키워드 파싱 누락 수정

## 이슈

- 이슈: https://github.com/edwardkim/rhwp/issues/3160
- 결함 클래스: HTML import의 CSS 값 파싱 변형 누락 (#3066, #3120, #3034와 동일 클래스)

## 결함 요약

`src/document_core/helpers.rs`의 CSS 값 파싱 헬퍼 2곳이 브라우저가 실제로 생성하는 표준 표기를 인식하지 못했다.

1. **`css_color_to_hwp_bgr()` — `rgba()` 미지원**
   - `css.starts_with("rgb(")`만 검사하여 `rgba(r, g, b, a)` 표기가 전부 `None`으로 떨어짐.
   - 증상: 셀 `background-color: rgba(255, 0, 0, 1)` → 배경 유실, `border: 1px solid rgba(...)` → 테두리 색 검정 폴백.
2. **`parse_css_border_shorthand()` — border-width 키워드 미지원**
   - CSS 표준 키워드 `thin`(1px)/`medium`(3px)/`thick`(5px)이 치수 파서로 흘러가 0.0이 되어 width=0 → 테두리 전체 소실.
   - `border: solid`처럼 굵기 생략 시 CSS 기본값이 `medium`이므로 실사용에서 흔히 발생.

## 수정 내용

- `css_color_to_hwp_bgr()`: `rgb`/`rgba` 공통으로 `(` 위치를 찾아 내부 성분을 파싱. 4번째 성분(alpha)이 존재하고 0 이하이면 완전 투명으로 간주해 `None` 반환.
- `parse_css_border_shorthand()`: 토큰 match에 `thin`(0.75pt)/`medium`(2.25pt)/`thick`(3.75pt) 분기 추가.
- 재현 테스트 `test_css_color_rgba_and_border_width_keywords` 추가 (`src/wasm_api/tests.rs`).

## 변경 파일

- `src/document_core/helpers.rs` — `css_color_to_hwp_bgr()`, `parse_css_border_shorthand()` 수정
- `src/wasm_api/tests.rs` — 재현 테스트 추가
- `mydocs/report/task_m100_3160_report.md` — 본 문서

## 검증 (red → green)

1. **red**: 수정 전 테스트 실행 → FAIL 확인
   ```
   assertion `left == right` failed: rgba() 불투명 빨강 → BGR
     left: None
     right: Some(255)
   ```
2. **green**: 수정 후
   ```
   test wasm_api::tests::test_css_color_rgba_and_border_width_keywords ... ok
   ```
3. **주변 테스트**: `cargo test --lib html` 27건 전부 통과, `test_table_utility_functions`/`test_html_utility_functions` 통과.
4. **CI 사전 검사**: `cargo fmt --check`(CRLF 노이즈 제외 위반 없음), `cargo clippy --lib` 경고/에러 없음.
