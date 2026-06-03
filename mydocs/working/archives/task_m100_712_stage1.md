# Task #712 Stage 1 (RED) 완료 보고서

**Issue**: [#712](https://github.com/edwardkim/rhwp/issues/712)
**Stage**: 1 — TDD RED
**작성일**: 2026-05-08

---

## 산출물

- **신규 회귀 테스트**: `tests/issue_712.rs`
- **단언 대상**: pi=586 (12x5 일정 표) 외곽 상단 y ≥ pi=585 (1x3 TAC 제목 표) 외곽 하단 y - 0.5px(rounding 허용)
- **빌드 독립**: PAGE_INDEX 하드코딩 제거. pi=585 / pi=586 가 동시 등장하는 페이지를 동적 탐색 → Task #643 적용 여부와 무관하게 결함 검증.

## 테스트 실행 결과 (RED — 의도된 FAIL)

```
$ cargo test --test issue_712 -- --nocapture
...
[issue_712] page_index=35 (page_count=40) pi585=[98.25..137.11] pi586=[124.93..1004.31]
panicked at tests/issue_712.rs:81:
pi=586 12x5 표가 pi=585 1x3 표 안쪽으로 침범.
pi585=[98.25..137.11] pi586=[124.93..1004.31] 침범=12.17 px
test issue_712_pi586_table_does_not_invade_pi585_outer_box ... FAILED
```

## 측정 결과 (현재 빌드: local/task712 = devel = stream/devel + Task #703)

| 항목 | y (px) | 비고 |
|------|--------|------|
| body_top | 94.48 | |
| pi=585 cell 상단 | 98.25 | outer_top 1mm = 3.77 px |
| pi=585 cell 하단 | 137.11 | (size 38.85 px) |
| pi=585 outer 하단 (이론치) | 140.87 | + outer_bottom 3.77 px |
| **pi=586 12x5 표 시작** | **124.93** | ← 침범 |
| 침범량 (cell 기준) | 12.17 px | 실측 |
| 침범량 (outer 기준) | ~15.94 px | outer_bottom 추가 시 |

→ Stage 0 수행 계획서의 측정값과 일치 확인.

## 빌드 환경 차이 메모

| 빌드 | page_count | pi=585/586 등장 page_index |
|------|-----------|----------------------------|
| stream/devel (Task #643 미적용) | 40 | 35 |
| devel (현재, Task #703 적용 + Task #643 미적용) | 40 | 35 |
| pr-task644-rebase (Task #643 적용) | 35 | 30 |

→ 회귀 테스트는 페이지 인덱스를 동적 탐색하므로 빌드 환경 무관.

## 다음 단계 (Stage 2 — 분석)

1. `RHWP_TASK712_DEBUG=1` 환경변수로 `layout.rs` 에 트레이스 인스트루먼트 일시 추가
2. pi=585 → pi=586 진입 시 y_offset 진행 시퀀스 수집
3. 가설 H1 (vpos-reset base 오류) / H2 (vert_offset 이중 적용) / H3 (pt_y_start 가드 음수 누락) 중 root cause 확정
4. 보고서 `task_m100_712_stage2.md` 작성

## 승인 요청

Stage 1 RED 단계 완료. Stage 2 (분석) 진행 승인 요청.
