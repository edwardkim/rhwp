# Task #704 최종 결과 보고서

## 이슈

- 이슈 번호: #704
- 제목: 통합재정통계 페이지 분할 — TopAndBottom TAC 1×1 wrapper + 각주 환경에서 trailing 빈 paragraph 가 0.84 px 부족으로 다음 페이지로 밀림
- 마일스톤: v1.0.0 (M100)
- 처리 일자: 2026-05-13

## 결론

**이슈 #704 의 본질적 결함은 Task #676 (PR #679, 커밋 `52ae6558`) 에서 이미 정정 완료.** 본 task 는 잔존 `#[ignore]` 테스트 cleanup 만 수행.

## 처리 경위

| 시점 | 사건 |
|------|------|
| Task #703 분석 단계 | 통합재정통계 케이스를 BehindText/InFrontOfText 와 다른 본질로 판단, #704 로 분리 등록 + `tests/issue_703.rs` 의 두 테스트 `#[ignore]` 처리 |
| Task #676 (`52ae6558`, PR #679 merged `bd3b63dd`) | `typeset.rs:1182-1199` 에 trailing empty paragraph 가드 추가 — `is_last_in_section && col_count==1 && empty_para && total_h ≤ available + LAYOUT_DRIFT_SAFETY_PX → height=0 흡수`. 통합재정통계 2010.11/2011.10/2014.8 정식 GREEN 테스트 (`tests/issue_676_trailing_empty_para.rs`) 추가 |
| Task #704 (본 task, 2026-05-13) | 본질 정정 진입 사실 검증 + `tests/issue_703.rs` 의 `#[ignore]` 2 건 cleanup |

이슈 본문의 추정 정정 "옵션 B (epsilon 보강)" 과 Task #676 가드는 **동일 시멘틱** — `LAYOUT_DRIFT_SAFETY_PX` 영역 내 미세 overflow 흡수.

## 단계별 진행

| Stage | 일자 | 커밋 | 산출물 | 결과 |
|-------|------|------|--------|------|
| Stage 1 (현황 검증) | 2026-05-13 | (Stage 2 와 통합) | `task_m100_704_stage1.md` | ignored 2 건 강제 실행 GREEN, issue_676 3 건 GREEN |
| Stage 2 (cleanup) | 2026-05-13 | `2839a7c2` | `task_m100_704_stage2.md` + `tests/issue_703.rs` | `#[ignore]` 2 건 제거, 주석 Task #676 reference 갱신 |
| Stage 3 (회귀 + 보고서) | 2026-05-13 | (본 보고서 커밋) | `task_m100_704_report.md` | cargo test --release 1231 passed, 타겟 통합 테스트 GREEN |

## 변경 파일

- `tests/issue_703.rs` — `#[ignore]` 2 건 제거 + 주석 갱신
- `mydocs/plans/task_m100_704.md` — 수행계획서 (신규)
- `mydocs/working/task_m100_704_stage1.md` — Stage 1 보고서 (신규)
- `mydocs/working/task_m100_704_stage2.md` — Stage 2 보고서 (신규)
- `mydocs/report/task_m100_704_report.md` — 본 보고서 (신규)

**소스 코드 무변경.** 테스트 cleanup 만 수행.

## 검증

### (1) 타겟 통합 테스트 — `cargo test --release --test issue_703 --test issue_676_trailing_empty_para`

```
running 3 tests
test issue_676_t재정통계_2010_11_single_page ... ok
test issue_676_t재정통계_2011_10_single_page ... ok
test issue_676_t재정통계_2014_08_no_regression ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 3 tests
test issue_703_tonghap_2010_11_single_page ... ok
test issue_703_tonghap_2011_10_single_page ... ok
test issue_703_calendar_year_single_page ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### (2) 전체 회귀 — `cargo test --release`

```
test result: FAILED. 1231 passed; 1 failed; 2 ignored
failures:
    wasm_api::tests::test_empty_save_analysis  (단독 실행 시 PASS)
```

**1 failed 는 본 task 무관 기존 flaky test.** `wasm_api::tests::test_empty_save_analysis` 는 `output/empty_with_text.hwp` 파일을 다른 lib test 와 공유하여 병렬 실행 race condition 으로 간헐 실패. 동일 테스트를:
- devel HEAD 단독 실행 → PASS
- local/task704 단독 실행 → PASS

→ 본 task 변경(`tests/issue_703.rs` 통합 테스트의 ignore 해제)과 명백히 무관 (lib 내부 단위 테스트 영역).

### (3) Clippy — `cargo clippy --all-targets -- -D warnings`

```
error: could not compile `rhwp` (lib test) due to 54 previous errors
```

**54 errors 는 본 task 무관 devel HEAD 기존 위반.** devel HEAD 에서 `cargo clippy --all-targets` (no `-D warnings`) 실행 시 동일 54 warnings 발생 (`src/wasm_api/tests.rs` 의 `unused_must_use`). `-D warnings` 옵션이 warning 을 error 로 승격시켜 컴파일 실패. 본 task 변경 (`tests/issue_703.rs`) 은 통합 테스트 영역이며 lib test (`src/wasm_api/tests.rs`) 와 코드 경로 무관.

→ devel HEAD 의 별도 정리 필요 (본 task 범위 외, 후속 이슈 등록 권장).

## 수용 기준 결과

| # | 기준 | 결과 |
|---|------|------|
| 1 | `tests/issue_703.rs` 의 통합재정통계 ignore 2 건 제거 | ✅ |
| 2 | `tests/issue_676_trailing_empty_para.rs` GREEN 보존 | ✅ (3/3 ok) |
| 3 | `cargo test --release` 본 task 회귀 0 | ✅ (1 failure 기존 flaky, 본 task 무관 확인) |
| 4 | `cargo clippy --all-targets -- -D warnings` 클린 | ⚠️ devel HEAD 기존 54 warnings 잔존 — 본 task 범위 외 |
| 5 | Issue #704 close + 본질 정정 reference 명시 | (Stage 3 후속 단계에서 수행) |

## 후속 권장

- **별도 이슈 등록 권장**: `src/wasm_api/tests.rs` 의 lib test 54 `unused_must_use` 위반 정리 — devel HEAD 기존 부채.

## 관련

- Issue #676 — 본질 정정 (closed)
- PR #679 — Task #676 merge
- Issue #703 — 분석 모태
- Issue #775 — Task #703 회귀 정정

## 닫기

Issue #704 close 권장 (closes #704).
