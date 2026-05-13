# Task #704 Stage 2 완료 보고서 — cleanup (#[ignore] 해제)

## 단계 목표

`tests/issue_703.rs` 의 통합재정통계 `#[ignore = "Issue #704 ..."]` 2 건을 제거하여 정식 테스트로 승격. 주석은 Task #676 본질 정정 reference 로 갱신.

## 변경 내역

### `tests/issue_703.rs` (line 37-54)

| 항목 | 변경 전 | 변경 후 |
|------|--------|--------|
| 주석 (line 37-40) | "Issue #704 로 별도 분리 ... 후 #[ignore]." | "Task #676 (PR #679, 커밋 52ae6558) ... 으로 이미 본질 정정. Task #704 cleanup 단계에서 #[ignore] 해제." |
| `issue_703_tonghap_2010_11_single_page` (line 42-46) | `#[ignore = "Issue #704 ..."]` | 정식 `#[test]` |
| `issue_703_tonghap_2011_10_single_page` (line 49-54) | `#[ignore = "Issue #704 ..."]` | 정식 `#[test]` |

### diff 요약

- `-#[ignore = "Issue #704 별도 task — ..."]` × 2 줄 제거
- 주석 4 줄 → 7 줄 (Task #676/PR #679 reference 추가)
- 소스 코드 (typeset.rs 등) 변경 없음

## 검증

```
$ cargo test --test issue_703

running 3 tests
test issue_703_tonghap_2010_11_single_page ... ok
test issue_703_tonghap_2011_10_single_page ... ok
test issue_703_calendar_year_single_page ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

→ 통합재정통계 2 케이스 정식 GREEN, calendar_year 회귀 0, ignored 0.

## 다음 단계 (Stage 3)

1. `cargo test --release` 전체 회귀 검증 (1250+ 테스트)
2. `cargo clippy --all-targets -- -D warnings` 클린 확인
3. 최종 보고서 (`mydocs/report/task_m100_704_report.md`) 작성
4. 오늘할일 (`mydocs/orders/20260513.md`) 갱신
5. `local/task704` → `local/devel` merge → `devel` merge + push
6. `gh issue close #704`

## 환경 정보

- 브랜치: `local/task704`
- 변경 파일: `tests/issue_703.rs` (소스 무변경, 테스트 cleanup 만)
- 커밋: Stage 1 보고서 + Stage 2 보고서 + 본 변경 통합 1 커밋 예정
