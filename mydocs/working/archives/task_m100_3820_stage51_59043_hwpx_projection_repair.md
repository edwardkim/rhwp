---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 51 — 59043 저장 경계와 direct HWPX 투영 분리

## 목적

[Stage 50](task_m100_3820_stage50_full_regression.md)의 전체 회귀에서 확인한
`59043_regulatory_analysis.hwp` p35→p36 source 소실을 고치되, PR #4122 통합 뒤
32쪽으로 늘어난 direct HWPX #3637의 페이지·source-owner 계약을 함께 복원한다.

## 확정 원인

### 59043 p35→p36

원본 HWP5의 1×1 자식 표는 물리 높이가 한 페이지를 넘지만 저장 reset은
`7540HU → 0` 한 번뿐이다. 첫 fragment가 현재 body의 절반보다 짧아 일반
`stored_frame_break_before` 판정을 통과하지 못했고, legacy mixed fallback이 reset을
지웠다. 그 결과 p36의 첫 source 줄이 p35에 흡수되고, p36 부모 행 높이에는 재귀 child
cursor와 scalar continuation 보정이 동시에 더해져 27.7px 중복이 생겼다.

### #3637 31→32쪽 회귀

history를 격리 검증한 결과 다음 경계에서 회귀가 시작됐다.

- 리베이스 전 Stage 45 `405a8e4c7`: 31쪽, focused 계약 통과
- PR #4122 충돌 해소 첫 체크포인트 `8fc0e2ef8`: 32쪽, 페이지 수 실패

direct HWPX의 반복 cell-local vpos reset까지 HWP5 canonical child cursor로 투영한 것이
원인이다. 단순히 옛 tuple fallback으로 내리면 p26의 마지막 줄이 p27로 밀리고, 같은
문단의 줄 높이와 nested 높이를 `max`로만 줄이면 30쪽으로 과보정됐다. PR #4122 이전
fallback이 보존하던 재귀 단위 세분성과 scalar viewport 의미를 함께 복원해야 했다.

## 구현

1. native HWP5 1×1 표에서 물리 높이가 body를 넘고 reset이 정확히 하나이면 canonical
   `CellUnit`을 부모 fragment로 투영한다. 투영된 단일 reset만 authoritative 저장 경계로
   표시한다.
2. 재귀 fragment로만 구성된 native HWP5 run은 child `RowCut`이 source cursor와 viewport를
   이미 소유하므로 `mixed_nested_flow_extra_from_cut`의 scalar 첫 줄 보정을 더하지 않는다.
3. canonical hard/stored cursor는 native HWP5와 HWP5-origin HWPX에만 적용한다.
4. direct HWPX는 canonical `CellUnit`의 높이·가시 단위·재귀 세분성을 측정 원장으로
   재사용하되 `hard_break_before`, `stored_frame_break_before`, recursive child cursor를
   제거해 기존 scalar viewport로 투영한다. 이 조합이 #3637의 31쪽과 p26→p28 source-owner를
   동시에 복원한다.

고정 px, 문장, 문단 번호, 페이지 번호는 구현 predicate에 사용하지 않았다.

## 회귀 고정

- `test_multi_page_single_cell_nested_reset_is_authoritative_in_parent_projection`
  - native HWP5 물리 multi-page 단일 reset이 부모의 authoritative 경계가 되는지 고정한다.
- `test_direct_hwpx_nested_resets_keep_legacy_parent_projection`
  - direct HWPX 반복 local reset이 HWP5 recursive cursor로 승격되지 않는지 고정한다.
- 기존 #3637 integration은 31쪽뿐 아니라 p26 마지막 줄, p27 첫 줄, p28 page 밖
  `TextLine`, deepest nested overflow를 함께 검사한다.

## 집중 검증 결과

전용 `target/task-3820-3821-fidelity-rebase`와 `CARGO_INCREMENTAL=0`을 사용했다.

```text
새 단위 회귀:                              2 passed / 0 failed
issue_1921_59043_pagination_pin:           5 passed / 0 failed
issue_2007_nested_cell_pagination:         15 passed / 0 failed
issue_2279_layout_oracles:                  4 passed / 0 failed
issue_2430_cell_rewrap_threshold:           2 passed / 0 failed
issue_3637 세 focused binary:               3 passed / 0 failed
issue_1891:                                 3 passed / 0 failed
issue_2308 derived/guard:                   5 passed / 0 failed
cargo fmt --all:                            통과
git diff --check:                           통과
```

#3637의 복원된 결과는 31쪽이며 p26 `시간당 근로임금은`, p27
`사업체노동력조사`, p28 page-bottom 안쪽 child line 계약이 모두 통과했다. 59043은 37쪽을
유지하면서 p36의 중첩 source와 후속 제목을 복원했다.

## 다음 단계

이 변경을 독립 checkpoint로 커밋한 뒤 새 Stage에서 전체
`cargo test --profile release-test --tests`를 끝까지 실행한다. 통과 후 같은 HEAD로 full
Clippy와 native Skia 3개 회귀를 다시 실행한다. 실패 시 전체 회귀를 반복하지 않고 최초
실패 로그의 focused test부터 원인을 수정한다.
