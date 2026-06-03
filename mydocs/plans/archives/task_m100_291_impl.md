# Task #291 구현계획서

## 변경 위치

`src/renderer/layout.rs:1991~2003` — TAC 표 분기 (`tbl_inline_x` 계산)

## 변경 내용

### Before

```rust
} else if is_tac {
    let leading = composed.get(para_index)
        .map(|c| compute_tac_leading_width(c, control_index, styles))
        .unwrap_or(0.0);
    Some(col_area.x + effective_margin + leading)
}
```

### After

```rust
} else if is_tac {
    let leading = composed.get(para_index)
        .map(|c| compute_tac_leading_width(c, control_index, styles))
        .unwrap_or(0.0);
    let base_x = col_area.x + effective_margin + leading;
    // [Issue #291] ParaShape align 반영
    let aligned_x = match para_style.map(|s| s.alignment) {
        Some(Alignment::Right) => {
            let tbl_w = hwpunit_to_px(t.common.width as i32, self.dpi);
            let avail_right = col_area.x + col_area.width - margin_right;
            (avail_right - tbl_w).max(base_x)
        }
        Some(Alignment::Center) => {
            let tbl_w = hwpunit_to_px(t.common.width as i32, self.dpi);
            let center = col_area.x + (col_area.width - tbl_w) / 2.0;
            center.max(base_x)
        }
        _ => base_x,
    };
    Some(aligned_x)
}
```

## 설계 근거

### `.max(base_x)` 안전장치

- `base_x = col_area.x + effective_margin + leading`
- leading 이 있는 경우 (TAC 표 앞에 텍스트가 있는 경우) align 정렬 위치가 leading 으로 이미 이동한 위치보다 왼쪽이 되면 안 됨
- `.max(base_x)` 로 leading 보전

### ParaShape `alignment` vs `t.common.horz_align`

본 이슈의 케이스 (KTX.hwp pi=31/32) 는 `t.common.horz_align = 문단(0=0.0mm)` 으로 무지정. 따라서 ParaShape `alignment` 만 고려해도 충분. 두 값이 충돌하는 케이스는 본 PR 범위 외.

### Justify/Left 등 기존 동작 보존

`_ => base_x` 로 default 처리 → 기존 모든 케이스 회귀 0.

## 단계별 진행

### 2-1: 코드 수정 (Edit)
- `src/renderer/layout.rs:1991~2003` 의 TAC 분기 교체
- `crate::model::style::Alignment` 경로로 enum 참조

### 2-2: 빌드 검증
- `cargo build --release` 성공
- `cargo clippy --lib -- -D warnings` 통과
- `cargo check --target wasm32-unknown-unknown --lib` 통과

### 2-3: 단위/통합 테스트
- `cargo test --lib` 992 passed 유지
- `cargo test --test svg_snapshot` 6 passed (golden 무회귀)
- `cargo test --test issue_301` 1 passed
- `cargo test --test tab_cross_run` 1 passed

### 2-4: KTX.hwp 좌표 변화 직접 확인
- pi=31 좌측 x: 494.10 → 518.16 (오차 ± 1px)
- pi=32 좌측 x: 494.10 → 517.95
- pi=29 (비-TAC) 좌측 x: 변화 없음 (744.71 유지)

### 2-5: 회귀 스캔 (5개 샘플)
- KTX.hwp / exam_math / 21_언어 / aift / biz_plan
- 변경된 페이지에 대해 ParaShape align 분석 → 의도/회귀 판정

### 2-6: WASM 빌드 + 브라우저 시각 검증

## 산출물 (예상)

- `mydocs/plans/task_m100_291{,_impl}.md`
- `mydocs/working/task_m100_291_stage{1,2,3}.md`
- `mydocs/report/task_m100_291_report.md`
- `mydocs/troubleshootings/tac_table_align.md` (재발 방지)
- `mydocs/orders/20260425.md` 갱신

## 검증 우선순위

1. KTX.hwp 좌표 회귀 (목표 달성)
2. cargo test 회귀 0
3. 5샘플 byte-diff (지표)
4. 브라우저 시각 검증 (사용자 가시 검증)
