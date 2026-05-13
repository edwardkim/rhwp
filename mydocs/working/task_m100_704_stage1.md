# Task #704 Stage 1 완료 보고서 — 현황 검증

## 단계 목표

수행계획서 (`mydocs/plans/task_m100_704.md`) 의 진단 — "Task #676 가드로 이미 본질 정정 완료" — 를 실제 cargo test 실행으로 검증.

## 검증 결과

### (1) `tests/issue_703.rs` — `#[ignore = "Issue #704 ..."]` 강제 실행

```
$ cargo test --test issue_703 -- --ignored

running 2 tests
test issue_703_tonghap_2011_10_single_page ... ok
test issue_703_tonghap_2010_11_single_page ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
```

→ 이슈 #704 분리 시점에 ignore 처리됐던 두 테스트가 **이미 GREEN**.

### (2) `tests/issue_676_trailing_empty_para.rs` — Task #676 GREEN 재확인

```
$ cargo test --test issue_676_trailing_empty_para

running 3 tests
test issue_676_t재정통계_2010_11_single_page ... ok
test issue_676_t재정통계_2011_10_single_page ... ok
test issue_676_t재정통계_2014_08_no_regression ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

→ Task #676 본질 정정 GREEN 유지.

## 결론

수행계획서의 진단 확정:

- 이슈 #704 의 본질적 결함 (TopAndBottom TAC 1×1 wrapper + 각주 환경 trailing empty paragraph 0.84 px borderline) 은 **Task #676 (PR #679, 커밋 `52ae6558`) 의 `trailing empty paragraph 가드` (typeset.rs:1182-1199) 로 이미 정정됨**.
- 이슈 본문의 추정 정정 "옵션 B (epsilon 보강)" 와 본질적으로 동일 시멘틱 (`LAYOUT_DRIFT_SAFETY_PX = 4.0 px` 영역 내 미세 overflow 흡수).
- 잔존 부정합은 `tests/issue_703.rs` 의 `#[ignore]` 2 건이 풀리지 않은 **테스트 cleanup 누락**.

## 다음 단계 (Stage 2)

`tests/issue_703.rs` 의 `#[ignore]` 2 건 제거 + 주석을 "Task #676 (PR #679) 으로 해결됨" 으로 갱신.

소스 코드 변경 없음. 테스트 cleanup 만 수행.

## 환경 정보

- 브랜치: `local/task704` (← `local/devel` ← `devel` @ `03fc6565`)
- Working tree clean (Stage 1 단계 미커밋)
- 실행 일자: 2026-05-13
