# Task #291 Stage 2 — 구현

## 변경 내역

`src/renderer/layout.rs:1991~2013` — TAC 표 분기에 ParaShape `alignment` 반영 추가:

```rust
} else if is_tac {
    let leading = composed.get(para_index)
        .map(|c| compute_tac_leading_width(c, control_index, styles))
        .unwrap_or(0.0);
    let base_x = col_area.x + effective_margin + leading;
    // [Issue #291] ParaShape align 반영:
    // TAC 표가 inline_shape_position 미설정 상태에서 단/문단 좌측에
    // 붙어버리는 회귀를 막는다. ParaShape align=Right 인 경우 표를
    // 단의 우측 끝 - 표 폭 - margin_right 위치로 이동시켜 한컴과 일치.
    // align=Center 도 동일 원리로 처리.
    let aligned_x = match para_style.map(|s| s.alignment) {
        Some(crate::model::style::Alignment::Right) => {
            let tbl_w = hwpunit_to_px(t.common.width as i32, self.dpi);
            let avail_right = col_area.x + col_area.width - margin_right;
            (avail_right - tbl_w).max(base_x)
        }
        Some(crate::model::style::Alignment::Center) => {
            let tbl_w = hwpunit_to_px(t.common.width as i32, self.dpi);
            let center = col_area.x + (col_area.width - tbl_w) / 2.0;
            center.max(base_x)
        }
        _ => base_x,
    };
    Some(aligned_x)
}
```

### 핵심 설계 요소

1. **`base_x` 분리**: 기존 leading 계산을 그대로 두고 그 위에 align 분기 추가
2. **`.max(base_x)` 안전장치**: leading 이 있는 경우 align 정렬로 위치가 후퇴하지 않도록 방어
3. **`_ => base_x`**: Justify/Left/Distribute/Split 등 기존 동작 보존
4. **`crate::model::style::Alignment`**: 절대경로로 enum 참조 (use 추가 회피)

## 빌드 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ 25.57s |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ clean |

## 단위/통합 테스트

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **992 passed / 0 failed / 1 ignored** |
| `cargo test --test svg_snapshot` | ✅ 6 passed (golden 무회귀) |
| `cargo test --test issue_301` | ✅ 1 passed (z-table 회귀 없음) |
| `cargo test --test tab_cross_run` | ✅ 1 passed (#290 회귀 없음) |

## KTX.hwp 좌표 변화

| 표 | Before | After | 한컴 기대 | 평가 |
|----|--------|-------|-----------|------|
| pi=29 (비-TAC) | 744.71 | **744.71** | (변화 없음) | ✅ 영향 없음 |
| **pi=31 (TAC)** | 494.10 | **518.16** | 518.56 | ✅ 오차 0.4px |
| **pi=32 (TAC)** | 494.10 | **517.95** | 518.56 | ✅ 오차 0.6px |

이동 거리: 24.06px / 23.85px (≈ 6.4mm)

## Stage 2 요약

- 변경 파일: 1개 (`src/renderer/layout.rs`)
- 변경 라인: ~+15 / -1
- 빌드 + clippy + wasm32 + 단위/통합 테스트: 전부 통과
- KTX.hwp 좌표 회귀: 한컴 기대값과 오차 1px 이내 일치
