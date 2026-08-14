# Stage 242: upstream 리베이스 후 Clippy 표현 보정

## 목표

최신 `upstream/devel` 리베이스 후 Rust Clippy가 보고한 `obfuscated-if-else` 경고를 제거한다. 저장된 첫 fragment가 다음 행을 수용하는 기존 계산 계약과 렌더링 동작은 변경하지 않는다.

## 원인

`bool::then(...).unwrap_or(0.0)` 표현이 조건에 따라 overflow 값을 선택하는 로직을 간접적으로 나타내어 Clippy의 명료성 검사를 통과하지 못했다.

## 변경

- 조건이 참이면 계산한 overflow를 사용하고, 거짓이면 `0.0`을 사용하는 명시적 `if/else`로 바꾼다.
- 문서별 수치나 페이지별 예외는 추가하지 않는다.
- 계산 조건과 결과는 기존 표현과 동일하게 유지한다.

## 검증

- 리베이스 후 전체 라이브러리 회귀: `3609 passed; 0 failed; 13 ignored`
- 리베이스 후 전체 integration 회귀: 종료 코드 `0`
- 최종 코드 `cargo clippy -- -D warnings`: 종료 코드 `0`
- 최종 코드 `issue_3820_rowbreak_rowspan_band`: `4 passed; 0 failed`
