# Task m100-3034: HTML import text-decoration-line 공백 변형 누락 수정

## 이슈
#3034 — `src/document_core/commands/html_import.rs`의 CSS→CharShape 매핑에서
밑줄 인식 로직이 `text-decoration-line:underline`(공백 없음)만 검사하고
`text-decoration-line: underline`(콜론 뒤 공백) 변형은 검사하지 않음.

같은 함수 내 `font-weight`, `text-decoration` 검사는 공백 유무 두 변형을 모두
처리하는데 `text-decoration-line`만 공백 있는 변형이 빠져 있었음.

## 수정
`has_underline` 조건에 `text-decoration-line: underline`(공백 포함) 변형 추가.

- `src/document_core/commands/html_import.rs`: 조건 분기 1줄 추가.
- `src/document_core/commands/html_import.rs`: 공백 변형이 밑줄로 인식되는지
  확인하는 테스트 1건 추가.

## 검증
- `cargo check --lib` 통과.
