# 완료 보고서 — Task M100-3173

- 이슈: #3173
- 제목: 표 계산식 셀 참조에서 행 0이 와일드카드로 잘못 처리됨
- 작성일: 2026-07-23
- 브랜치: `fix/3173-table-calc-row-zero-wildcard`
- 담당 영역: `src/document_core/table_calc/`

## 1. 문제

표 계산식에서 셀 참조의 행을 `0`으로 명시하면(예: `=A0`), 스펙(`mydocs/plans/archives/task_370.md`:
"행은 1부터 시작하고, 와일드카드는 `?`만 인정")상 잘못된 입력임에도 오류 없이 **현재 셀의
행**으로 조용히 치환되어 계산되었다.

근본 원인: `tokenizer.rs`가 와일드카드 행(`?`)을 내부적으로 `0`으로 인코딩하고,
`evaluator.rs`의 `resolve_cell_ref`가 `row == 0`이면 와일드카드로 간주해
`ctx.current_row`로 치환했다. `rest.parse::<u32>().unwrap_or(0)`이 실제 입력 문자열
`"0"`도 `0`으로 파싱하므로, 명시적 0행 참조와 와일드카드 센티널 값이 충돌했다.

## 2. 재현 (RED)

`src/document_core/table_calc/evaluator.rs`에 회귀 테스트 추가:

```rust
#[test]
fn test_row_zero_is_not_wildcard() {
    let ctx = make_ctx(); // current_row = 4
    let r = evaluate_formula("=A0", &ctx, &sample_cell);
    assert!(r.is_err());
}
```

수정 전: `Ok(51.0)` (현재 행인 5행의 값) 반환 — FAIL 확인.

## 3. 수정

- `src/document_core/table_calc/tokenizer.rs`
  - 와일드카드 행 센티널을 `0`에서 `WILDCARD_ROW = u32::MAX`로 변경(공개 상수 추가).
  - 셀 참조 파싱에서 명시적 행 문자열이 `"0"`으로 파싱되면 셀 참조로 인식하지 않고
    기존 "함수 이름" fallback 경로로 넘겨 형식 오류로 자연히 처리되도록 함.
- `src/document_core/table_calc/evaluator.rs`
  - `resolve_cell_ref`의 와일드카드 판정을 `row == 0`에서 `row == WILDCARD_ROW`로 변경.
  - 회귀 테스트 `test_row_zero_is_not_wildcard` 추가.

## 4. 검증 결과 (GREEN)

- `cargo test --lib table_calc` — 29 passed (기존 28 + 신규 1), 0 failed
- `cargo fmt --check` — 변경 파일에 대해 실질적 diff 없음(레포 전역 CRLF 관련 경고만 존재,
  기존에도 발생하던 것과 동일하며 Windows 환경 특성)
- `cargo clippy --lib -- -D warnings` — 경고 없음
- `cargo test --release --lib table_calc` — 결과는 본 보고서 최신본 참고 (실행 완료 시 갱신)

## 5. 영향 범위

`src/document_core/table_calc/` 로 국한된 최소 수정이다. 공개 API 시그니처는
`WILDCARD_ROW` 상수 1개 추가 외에 변경 없다. 기존 테스트 28개 모두 그대로 통과해
회귀가 없음을 확인했다.
