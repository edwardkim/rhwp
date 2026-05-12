# Task #850 Stage 4 완료보고서

## 단계

Stage 4 — 회귀 검증

## 검증 범위

1. #850 신규 회귀 테스트
2. #717 기존 hit-test 회귀 테스트
3. 기존 중첩 표 path 기반 WASM API 테스트
4. 전체 `cargo test`

## 실행 결과

### #850 신규 회귀 테스트

명령:

```bash
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
```

결과:

```text
running 2 tests
test issue_850_exam_science_answer_sheet_name_cell_keeps_outer_path ... ok
test issue_850_exam_social_answer_sheet_name_cell_keeps_outer_path ... ok

test result: ok. 2 passed; 0 failed
```

### #717 기존 hit-test 테스트

명령:

```bash
cargo test --test issue_717_table_cell_hit_test -- --nocapture
```

결과:

```text
running 3 tests
test issue_717_exam_social_title_empty_area_stays_in_clicked_table ... ok
test issue_717_exam_social_view_table_empty_area_stays_in_clicked_table ... ok
test issue_717_exam_social_nested_header_empty_area_returns_editable_path ... ok

test result: ok. 3 passed; 0 failed
```

### 기존 중첩 표 path API 테스트

명령:

```bash
cargo test --lib test_task105_nested_table_path_api -- --nocapture
```

결과:

```text
test wasm_api::tests::test_task105_nested_table_path_api ... ok

test result: ok. 1 passed; 0 failed
```

### 전체 cargo test

명령:

```bash
cargo test
```

첫 실행은 sandbox 네트워크 제한으로 `static.crates.io` DNS 조회에 실패했다.

```text
failed to download from `https://static.crates.io/crates/web-sys/0.3.95/download`
Could not resolve host: static.crates.io
```

승인 후 같은 명령을 재실행하여 `web-sys v0.3.95` 다운로드와 전체 테스트를 완료했다.

주요 결과:

```text
test result: ok. 1232 passed; 0 failed; 2 ignored
```

통합 테스트 및 doc-test까지 모두 통과했다.

전체 테스트 중 #717, #850도 다시 통과했다.

```text
Running tests/issue_717_table_cell_hit_test.rs
test result: ok. 3 passed; 0 failed

Running tests/issue_850_answer_sheet_name_hit_test.rs
test result: ok. 2 passed; 0 failed
```

## 경고

전체 테스트 중 기존 warning 6건이 출력되었다.

- `src/renderer/equation/parser.rs`: duplicated attribute
- `src/renderer/layout/integration_tests.rs`: unnecessary parentheses
- `src/serializer/hwpx/field.rs`: non-snake-case test name
- `src/wasm_api/tests.rs`: non-snake-case test name 1건
- `src/wasm_api/tests.rs`: unused Result 2건

모두 기존 테스트 경고이며 #850 수정과 직접 관련된 실패는 아니다.

## 판정

Stage 4 목표 달성.

- #850 신규 회귀 테스트 GREEN
- #717 기존 회귀 테스트 GREEN
- 기존 중첩 표 path 기반 API 테스트 GREEN
- 전체 `cargo test` GREEN
- Studio TypeScript 변경 없이 Rust hit-test 경로 보정만으로 해결 확인

## 다음 단계

Stage 5에서 최종 정리를 진행한다.

예정 작업:

- 최종 결과보고서 작성: `mydocs/report/task_m100_850_report.md`
- 오늘할일 문서 상태 갱신
- 최종 `git status` 확인
- 커밋 가능한 변경 범위 정리

## 승인 요청

Stage 5 진행 승인 요청.

