# Task #1029 Stage 2 완료 보고서

**Issue**: [#1029 HWP3 외곽선 paper-edge 정합 회귀](https://github.com/edwardkim/rhwp/issues/1029)
**Branch**: `local/task1029`
**작업 내용**: Stage 1 의 layout.rs 복원 후 전체 회귀 sweep 검증

---

## 1. 빌드 + 정적 분석

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✓ Finished, warning 0 |
| `cargo clippy --release --lib -- -D warnings` | ✓ Finished |
| `cargo fmt --check` | ✓ clean |

---

## 2. 테스트 sweep

### 2.1 lib 테스트

```
$ cargo test --release --lib
test result: ok. 1307 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```

### 2.2 integration 테스트 (전체)

```
$ cargo test --release --tests
running 1 test    test result: ok. 1 passed
running 2 tests   test result: ok. 2 passed
running 3 tests   test result: ok. 3 passed
running 2 tests   test result: ok. 2 passed
running 3 tests   test result: ok. 3 passed
running 1 test    test result: ok. 1 passed
running 1 test    test result: ok. 1 passed
running 1 test    test result: ok. 1 passed
running 3 tests   test result: ok. 3 passed
running 1 test    test result: ok. 1 passed
running 2 tests   test result: ok. 2 passed
running 3 tests   test result: ok. 3 passed
running 4 tests   test result: ok. 4 passed
running 3 tests   test result: ok. 3 passed
running 2 tests   test result: ok. 2 passed
running 1 test    test result: ok. 1 passed
running 2 tests   test result: ok. 2 passed
running 4 tests   test result: ok. 4 passed
running 3 tests   test result: ok. 3 passed
running 1 test    test result: ok. 1 passed
running 1 test    test result: ok. 1 passed
running 13 tests  test result: ok. 13 passed  ← issue_table_vpos_01 (Task #990 보존)
running 2 tests   test result: ok. 2 passed
running 8 tests   test result: ok. 8 passed   ← svg_snapshot 포함
running 1 test    test result: ok. 1 passed
```

→ **68 integration tests 전체 passed, FAILED 0**.

### 2.3 Task #990 보존 단언

```
$ cargo test --release --test issue_table_vpos_01_page5_cell_hit_test
test result: ok. 13 passed; 0 failed
```

→ PR #1003 의 Task #990 의도된 변경 (pi=34 30.84px 상향 이동 + has_full_para_item 가드) 보존.

---

## 3. 단일 페이지 외곽선 단언 (Stage 1 결과 재확인)

| 포맷 | attr | basis | paper_based | border_top y | 판정 |
|------|------|-------|-------------|--------------|------|
| HWP3 native | 0x00000000 | PaperBased | **true** | **17.88** | ✓ C1 충족 (paper-edge 복원) |
| HWP5 변환본 | 0x00000001 | PaperBased | true | 17.88 | ✓ C2 충족 (무변동) |
| HWPX 변환본 | 0x00000041 | PaperBased | true | 17.88 | ✓ C3 충족 (무변동) |

회귀 전 baseline (PR #1011 @ 850cfb54) 과 byte-for-byte 동일 외곽선 좌표 복원.

---

## 4. 페이지 수 sweep

| Sample | 페이지 | 비고 |
|--------|--------|------|
| hwp3-sample10.hwp | 763 | ✓ |
| hwp3-sample11.hwp | 151 | ✓ |
| hwp3-sample13.hwp | 3 | ✓ |
| hwp3-sample14.hwp | 11 | ✓ |
| hwp3-sample16.hwp | 64 | ✓ |
| hwp3-sample16-hwp5.hwp | 64 | ✓ |
| hwp3-sample16-hwp5.hwpx | 71 | ✓ |
| exam_kor.hwp | 20 | ✓ |
| exam_eng.hwp | 8 | ✓ |
| exam_math.hwp | 20 | ✓ |
| aift.hwp | 74 | ✓ (Task #990 의도된 효과 보존) |
| biz_plan.hwp | 6 | ✓ |

→ **C4 (Task #990 보존, aift 74p 유지), C5 (시험지 회귀 0) 충족**.

---

## 5. 성공 기준 충족 표

| 조건 | 기준 | 결과 |
|------|------|------|
| C1 | HWP3 sample16 border_top y = 17.88 (paper-edge 복원) | ✓ |
| C2 | HWP5 변환본 border_top y = 17.88 유지 | ✓ |
| C3 | HWPX 변환본 border_top y = 17.88 유지 | ✓ |
| C4 | PR #1003 Task #990 효과 보존 (hit_test 13 passed, aift 74p) | ✓ |
| C5 | 시험지 회귀 0 (exam_* 페이지 수) | ✓ |
| C6 | `cargo test --release --lib` 1307+ passed | ✓ (1307 passed) |
| C7 | 작업지시자 한컴 viewer 시각 검증 | (Stage 3 PR 검토 시) |

C1~C6 자동 단언 완료. C7 은 PR 검토 시 작업지시자 한컴 viewer 비교 — Stage 3 진행.

---

## 6. 다음 단계 (Stage 3)

최종 보고서 작성 + orders 갱신 + PR 생성 (작업지시자 승인 후).
