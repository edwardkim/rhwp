# Task #712 Stage 3 (GREEN) 완료 보고서

**Issue**: [#712](https://github.com/edwardkim/rhwp/issues/712)
**Stage**: 3 — TDD GREEN (정정 적용)
**작성일**: 2026-05-08

---

## 1. 정정 내용

### 변경 파일

1. **`src/renderer/layout/table_partial.rs`** (PartialTable 내부 게이트)
2. **`src/renderer/layout.rs`** (`pt_y_start` 게이트)

### 변경 본질

`HwpUnit = u32` 타입 특성상 음수 값(예: -1796 HU)이 비트표현 그대로 unsigned 양수(4294965500u32)로 저장되어, `vertical_offset > 0` unsigned 비교 게이트가 음수도 통과시킴 → 후속 `as i32` 캐스트에서 음수가 적용되어 표가 위로 점프, 직전 인라인 표 영역 침범.

**정정 (`table_partial.rs:62-71`)**:

```rust
// Before
&& table.common.vertical_offset > 0   // u32 비교 — 음수 비트표현도 통과

// After
let vert_off_signed = table.common.vertical_offset as i32;
&& vert_off_signed > 0                // i32 비교 — 음수 차단
```

`y_start + hwpunit_to_px(table.common.vertical_offset as i32, ...)` → `hwpunit_to_px(vert_off_signed, ...)` 도 동일하게 signed 변수 사용.

**정정 (`layout.rs:2673-2685` `pt_y_start` 게이트)** 도 동일 패턴으로 수정.

### 인스트루먼트 정리

Stage 2 분석용 `RHWP_TASK712_DEBUG` 디버그 인쇄 모두 제거 (5 위치).

## 2. 검증

### 회귀 테스트 PASS

```
$ cargo test --test issue_712 -- --nocapture
[issue_712] page_index=35 (page_count=40) pi585=[98.25..137.11] pi586=[148.88..1028.25]
test issue_712_pi586_table_does_not_invade_pi585_outer_box ... ok
test result: ok. 1 passed; 0 failed
```

### Before/After 비교

| 항목 | Before (Stage 1 RED) | After (Stage 3 GREEN) | 변화 |
|------|---------------------|----------------------|------|
| pi=585 cell | [98.25..137.11] | [98.25..137.11] | 동일 |
| **pi=586 12x5 표** | **[124.93..1004.31]** | **[148.88..1028.25]** | **+23.95 px (정상화)** |
| pi=585 cell 하단 → pi=586 시작 간격 | -12.17 px (침범) | **+11.77 px** | 정상 (= outer_margin_bottom 3.77 + ls 8.0) |

### 페이지 37 (분할 표 연결 페이지) 회귀 확인

```
$ ./target/debug/rhwp export-svg ... -p 36 --debug-overlay
s0:pi=586 ci=0 12x5 y=94.5    (PartialTable 잔여 rows 8..12, body_top 정상)
s0:pi=586 ci=1 1x3 y=966.9    (말미 1x3 inline TAC, 정상)
```

`is_continuation=true` 분기는 게이트의 `!is_continuation` 조건으로 그대로 통과 → 분할 연결 페이지 회귀 0 확인.

### 전체 테스트 (`cargo test --release`)

- **1252 passed, 0 failed, 5 ignored** — 회귀 0
- `tests/issue_703.rs` (BehindText/InFrontOfText), `tests/issue_154.rs` 등 관련 회귀 모두 PASS

## 3. 변경 사항 (diff 요약)

```
src/renderer/layout.rs            | +5 -2 (gate signed 비교 + 주석)
src/renderer/layout/table_partial.rs | +9 -3 (gate signed 비교 + 주석)
```

## 4. 다음 단계 (Stage 4 — 회귀 검증)

광범위 회귀 검증 + 골든 SVG 비교 (이미 cargo test 에서 부분 통과 확인). 추가 횡단 검증 단계.

## 승인 요청

Stage 3 GREEN 완료 — 회귀 테스트 PASS, 전체 테스트 1252/1252. Stage 4 (광범위 회귀 검증) 진행 승인 요청.
