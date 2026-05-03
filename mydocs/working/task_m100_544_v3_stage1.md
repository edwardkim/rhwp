# Task #544 v3 Stage 1 보고서

## 목적

박스 안 sequential paragraph 사이 line-spacing 좁음 (5+ 케이스) 의 정밀 진단 + TDD RED + Phase A 방법 결정.

## 측정 결과 (5 케이스)

본 task #544 v2 와 무관 (byte-identical baseline 검증 완료, paragraph y 무영향).

| 케이스 | prev paragraph (줄수) | gap (px) | drift (px) | drift (HU) | 패턴 |
|--------|---------------------|---------|------------|------------|------|
| p2 [4~6] pi=46→47 | "15세기 초..." (5줄) | 18.35 | -5.86 | -440 | 부분 제외 |
| p5 [10~12] pi=112→113 | "살펴보건대..." (9줄) | 19.10 | -5.11 | -383 | 부분 제외 |
| **p10 [19~21] pi=208→209** | **"조선 시대를..." (5줄)** | **14.67** | **-9.55** | **-716** | **완전 제외** |
| **p11 [22~24] pi=234→235** | **"빈곤 퇴치..." (12줄)** | **14.66** | **-9.55** | **-716** | **완전 제외** |
| p13 [25~27] pi=266→267 | "암세포의 대사..." (3줄) | 18.34 | -5.87 | -440 | 부분 제외 |

PDF 기대값: 24.21 px (= 1816 HU = 1 line spacing).

## 본질 가설 (확정)

**박스 안 sequential paragraph (prev/next 둘 다 같은 박스 outline) 사이에서 Task #479 trailing-ls 제외 메커니즘이 작동**:

- `paragraph_layout.rs:2645~2664` 의 `is_full_paragraph_end && cell_ctx.is_none() && !next_starts_border` 분기에서 `y += line_height` (trailing-ls 미가산)
- Task #552 의 `next_para_starts_visible_border` 는 **새로운 visible border 시작** 시에만 set. 박스 안 sequential paragraph 는 **set 안 함** → 가드 미작동 → trailing-ls 누락

### 두 가지 패턴

- **완전 제외 (-9.55 px)**: p10, p11 — trailing-ls 716 HU 전부 누락
- **부분 제외 (-5~6 px ≈ 276~332 HU 누락)**: p2, p5, p13 — Task #537 의 `lazy_base` 보정이 일부 작동하여 부분 회복 가능성 (가설 A — Stage 2 적용 후 재검증)

p2/p13 의 정확히 동일한 drift (-440 HU) 는 우연 아닐 것 — `lazy_base` 의 보정값이 일정 케이스에서 일관되게 일부 적용됨.

## TDD RED 추가

`src/renderer/layout/integration_tests.rs` 끝에 3 테스트 추가:

| 테스트 | 페이지 | 검증 |
|--------|--------|------|
| `test_544_v3_passage_inner_lspacing_p2_4_6` | 2 (col 1) | pi=46→47 gap = 24.21 ±2 px |
| `test_544_v3_passage_inner_lspacing_p10_19_21` | 10 (col 0) | pi=208→209 gap = 24.21 ±2 px |
| `test_544_v3_passage_inner_lspacing_p11_22_24` | 11 (col 1) | pi=234→235 gap = 24.21 ±2 px |

각 `#[ignore = "Task #544 v3 RED — fix 적용 전 실패 expected"]`.

검증:
- baseline `cargo test --lib`: 1122 passed / 5 ignored / 0 failed
- `--ignored` 단독 실행: 3 RED 모두 정확히 fail (측정값 표기)

## Phase A 방법 결정 (방법 1)

### 방법 1: 새 Cell `next_para_continues_visible_border` 도입 (확정)

```rust
/// [Task #544 v3] 박스 안 sequential paragraph (prev/next 둘 다 같은
/// border_fill_id 진행 중) 시 prev paragraph 의 trailing-ls 보존.
/// next_para_starts_visible_border (Task #552, "border 시작") 와 의미 분리.
next_para_continues_visible_border: std::cell::Cell<bool>,
```

paragraph_layout.rs is_full_paragraph_end 분기:

```rust
let next_starts_border = self.next_para_starts_visible_border.get();
let next_continues_border = self.next_para_continues_visible_border.get();
if is_cell_last_line && cell_ctx.is_some() {
    y += line_height;
} else if is_full_paragraph_end && cell_ctx.is_none()
    && !next_starts_border && !next_continues_border {
    y += line_height;
} else {
    y += line_height + line_spacing_px;
}
```

### 방법 2 (대안 — 기각)

`next_starts_border` 의미 확장 — Task #552 의 의미 변경 → test_552 영향 가능. 안전성 위해 방법 1 채택.

## set caller 위치 (Stage 2 작업)

layout.rs 의 `next_para_starts_visible_border.set(...)` 호출 6 곳 (line 1985/2002/2041/2056/2153/2170) 와 동등한 분기 위치에 `next_para_continues_visible_border.set(...)` 추가:

- 다음 paragraph 의 `border_fill_id > 0` + visible stroke + **prev paragraph 의 border_fill_id 와 동일** 이면 set
- 호출 직후 reset

## 산출

- `src/renderer/layout/integration_tests.rs` (+170 LOC, 3 RED 테스트)
- `mydocs/working/task_m100_544_v3_stage1.md` (본 문서)

## 다음 단계 — Stage 2

방법 1 적용:
1. layout.rs LayoutEngine struct 에 `next_para_continues_visible_border: Cell<bool>` 추가
2. layout.rs caller 6 곳에 set 분기 추가 (`next_starts_border` set 와 동등 위치, 다른 조건)
3. paragraph_layout.rs is_full_paragraph_end 분기에 가드 추가
4. test_544_v3_*_p2_4_6 / _p10_19_21 / _p11_22_24 의 `#[ignore]` 제거 → GREEN

### 위험 가드

- test_552_passage_box_top_gap_p2_4_6 (`--ignored`) GREEN 유지 (Task #552 무회귀)
- test_544 / test_547 / test_548 GREEN 유지
- 회귀 가드 24/24 GREEN 유지
