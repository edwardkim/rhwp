---
kind: working
status: active
issue: 6360
---

# #6360 Stage 3: 장시간 회귀 테스트 단축 구현 계획

## 기준

- 선행 분석: `mydocs/working/task_m100_6360_stage2_regression_runtime_root_cause.md`
- 기준 branch: `fix/pdf-reference-fast-pass-20260829`
- 기준 HEAD: `3dc1fd382e0bac6c336fbed6ffaf626208c05e20`

Stage 2에서 확인한 병목은 archive 배분 문제가 아니라, nextest가 더 쪼갤 수 없는
단일 testcase의 시간이 너무 긴 문제다. 이번 stage는 사용자 지시 범위인 1, 2, 3만
코드로 줄인다.

## 구현 방침

### 1. 전수형 corpus sweep partition 확대

대상 테스트는 문서 묶음을 deterministic bucket으로 나눈다. 파일명 순 modulo가 아니라,
파일 크기 내림차순으로 정렬한 뒤 현재 합산 크기가 가장 작은 bucket에 넣어 큰 fixture가
한 partition에 몰리는 현상을 줄인다.

대상:

- `tests/convert_verify_corpus_ratchet.rs`
- `tests/hwp5_roundtrip_baseline.rs`
- `tests/overflow_cell_baseline.rs`
- `tests/cases/text_overlap_baseline.rs`
- `tests/cases/off_canvas_baseline.rs`
- `tests/cases/oracle_page_count_baseline.rs`

dump 파일을 쓰는 baseline 테스트는 partition별 파일명을 사용해 같은 env dump 경로를
여러 testcase가 동시에 덮어쓰지 않도록 한다.

### 2. 보안 sweep의 반복 CLI 호출 제거

정상 corpus를 검사하는 clean sweep은 같은 문서를 숨김 텍스트, injection, unicode scan
명령으로 각각 다시 여는 구조다. 검증 의미는 유지하되, 테스트 내부에서 문서를 한 번
로드하고 in-process query를 호출하도록 바꾼다.

대상:

- `tests/security_corpus_regression.rs`
- `tests/injection_scan_contract.rs`

CLI schema와 개별 fixture 검증은 그대로 남긴다. 반복 clean sweep만 in-process와
partition으로 줄인다.

### 3. `issue2063_huge_cellbreak_table.hwp` page count 중복 제거

같은 대형 문서를 다음 세 테스트가 각각 다시 열고 `page_count()`를 수행한다.

- `tests/issue_2063.rs`
- `tests/issue_1842.rs`
- `tests/issue_2070_rowbreak_density.rs`

가장 엄격한 현재 pin은 161쪽이다. 하나의 sentinel에서 161쪽 pin을 검증하고, 나머지
테스트는 동일 문서의 중복 조판을 수행하지 않도록 정리한다. 이슈 문맥은 주석과 테스트
이름에 남긴다.

## 검증 기준

코드 수정 뒤에는 다음을 확인한다.

1. `cargo fmt --all`
2. 변경한 직접 integration test target
3. `tests/cases` 기반 generated regression suite 준비 및 관련 suite 실행
4. `ci-duration-observation` 프로필로 최장 testcase 시간이 기존 816.636초보다 낮아졌는지 확인

측정 결과는 별도 stage 문서에 기록한다. 전체 PR 생성은 별도 승인 전에는 수행하지 않는다.
