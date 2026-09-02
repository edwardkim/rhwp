# Stage 19: Rust CodeQL none mode 계약 보정

## 목적

PR #5190에서 가져온 Rust CodeQL `build-mode: none` 변경과 기존 workflow
contract test를 일치시켜 PR #5185의 lint gate를 복구한다.

## 원인

기존 테스트는 모든 언어가 같은 CodeQL initialize 조건을 세 번 사용하며
`build-mode`가 없어야 한다고 가정했다. 새 workflow는 JavaScript/TypeScript와
Python은 기존 기본 모드를 유지하고 Rust만 별도 initialize 단계에서
`build-mode: none`을 사용한다.

## 변경 계약

- 공통 언어 선택 조건은 checkout과 analyze 단계 두 곳에서 사용한다.
- non-Rust와 Rust initialize 단계은 각각 동일한 선택 정책을 적용한다.
- Rust initialize 단계만 `languages: rust`와 `build-mode: none`을 선언한다.
- cache, 수동 `cargo build`, 임시 결과 artifact는 계속 추가하지 않는다.

## 검증 결과

- `python3 -m unittest scripts/tests/test_codeql_workflow.py`: 14건 통과.
