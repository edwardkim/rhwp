# Task #544 v3 Stage 3 보고서

## 목적

Phase A 적용 후 광범위 회귀 검증 + Phase B 필요 여부 검증.

## p5/p13 측정 (Phase B 검증)

Stage 1 미측정이었던 p5/p13 fix 후 측정:

| 케이스 | 수정 전 gap | 수정 후 gap | 정합 |
|--------|------------|------------|------|
| p5 [10~12] pi=112→113 | 19.10 | **24.21 px** | PDF 정합 ✓ |
| p13 [25~27] pi=266→267 | 18.34 | **24.21 px** | PDF 정합 ✓ |

**5 케이스 종합**:

| 케이스 | gap (수정 전) | drift (수정 전) | gap (수정 후) | 정합 |
|--------|--------------|----------------|--------------|------|
| p2 [4~6] | 18.35 | -5.86 | 24.21 | ✓ |
| p5 [10~12] | 19.10 | -5.11 | 24.21 | ✓ |
| p10 [19~21] | 14.67 | -9.55 | 24.21 | ✓ |
| p11 [22~24] | 14.66 | -9.55 | 24.21 | ✓ |
| p13 [25~27] | 18.34 | -5.87 | 24.21 | ✓ |

→ **5 케이스 모두 PDF 한컴 2010 정합 (24.21 px = 1816 HU)**.

→ 부분 제외 (p2/5/13 -5~6 px) 와 완전 제외 (p10/11 -9.55 px) 모두 **같은 fix** (Phase A) 로 해결.

→ **Phase B 불필요** (수행계획서의 가설 A "lazy_base 일부 보정" 도 Phase A 가 흡수).

## 회귀 가드 (31/31 GREEN)

| Suite | 결과 |
|-------|------|
| issue_301 | 1 GREEN |
| issue_418 | 1 GREEN |
| issue_501 | 1 GREEN |
| issue_505 | 9 GREEN |
| issue_514 | 3 GREEN |
| issue_516 | 8 GREEN |
| issue_530 | 1 GREEN |
| issue_546 | 1 GREEN |
| svg_snapshot | 6 GREEN |

→ **31 / 31 GREEN, 회귀 0건**.

## 단위 테스트

```
cargo test --lib
test result: ok. 1125 passed; 0 failed; 2 ignored
```

- Stage 2 baseline: 1125 (변동 없음)
- 본 task 핵심 테스트:
  - test_544_v3_passage_inner_lspacing_p2_4_6: GREEN
  - test_544_v3_passage_inner_lspacing_p10_19_21: GREEN
  - test_544_v3_passage_inner_lspacing_p11_22_24: GREEN
  - test_552_passage_box_top_gap_p2_4_6 (`--ignored`): **GREEN 유지** (Task #552 무회귀)
  - test_544 / test_547 / test_548: GREEN 유지

## Clippy

```
cargo clippy --lib
```

본 task 신규 결함 **0건**.
기존 잔존 결함 2건 (`table_ops.rs:1007`, `object_ops.rs:298` `panicking_unwrap`) — orders 메모에 이미 기록, 본 task 무관.

## Release Build

```
cargo build --release
Finished `release` profile [optimized] target(s) in 47.43s
```

성공.

## 코드 영향 (누계)

| 파일 | 변경 LOC |
|------|----------|
| `src/renderer/layout.rs` | +37 (Cell + helper + 3 caller) |
| `src/renderer/layout/paragraph_layout.rs` | +6 / -1 (가드 추가) |
| `src/renderer/layout/integration_tests.rs` | +170 (3 GREEN 테스트) |

## 다음 단계 — Stage 4

최종 보고서 + orders 갱신:
- `mydocs/report/task_m100_544_v3_report.md`
- `mydocs/orders/20260503.md` 갱신 — Task #544 v2 항목에 v3 처리 기록 추가
- 작업지시자 시각 판정 1차 (SVG) → 2차 (rhwp-studio web Canvas) 게이트
