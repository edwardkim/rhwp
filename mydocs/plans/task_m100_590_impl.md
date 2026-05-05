# Task #590: 구현 계획서

## 패치 위치

`src/renderer/layout.rs:2285-2300`

## 패치 내용 (최소 핀셋)

현재 분기는 `horz_rel_to` 무관하게 모든 Square-wrap 표를 `col_area.x + effective_margin` 기준으로 강제 배치한다.

수정 방침: **`horz_rel_to=Para` 인 경우에만** 본 분기 발동. 그 외(`Column`, `Page`, `Paper`)는 fall-through 하여 `compute_table_x_position` 의 기본 분기에서 명세대로 처리.

```rust
} else if !is_tac && tbl_is_square
    && matches!(t.common.horz_rel_to, crate::model::shape::HorzRelTo::Para) {
    // [Issue #480 / #590] horz_rel_to=Para 인 Square wrap 표만 paragraph 영역 기준 정렬.
    // horz_rel_to=Column/Page/Paper 는 compute_table_x_position 의 기본 분기에서
    // 명세대로 처리 (col_area / body_area / paper 기준).
    let tbl_w = hwpunit_to_px(t.common.width as i32, self.dpi);
    let area_x = col_area.x + effective_margin;
    let area_w = (col_area.width - effective_margin - margin_right).max(0.0);
    let x = match t.common.horz_align {
        crate::model::shape::HorzAlign::Right | crate::model::shape::HorzAlign::Outside =>
            area_x + (area_w - tbl_w).max(0.0),
        crate::model::shape::HorzAlign::Center =>
            area_x + (area_w - tbl_w).max(0.0) / 2.0,
        _ => area_x,
    };
    Some(x)
}
```

## 단계

### 1단계: 패치 + 빌드

- `src/renderer/layout.rs:2285` 분기 가드 추가
- `cargo build --release --bin rhwp` GREEN

### 2단계: 단위 테스트

- `cargo test --lib` 전체 GREEN
- 신규 회귀 검출 시 → 한컴 정답지 대조 후 의도 정정/회귀 분류

### 3단계: 17쪽 [A] 실측 검증

- `rhwp export-svg samples/exam_kor.hwp -p 16 -o output/svg/`
- `[A]` 셀 좌측 = 약 126.6 px (col_area.x + h_offset) 일치 확인

### 4단계: 14쪽 [A] 회귀 검증

- `exam_kor.hwp` 14쪽 [A] 박스도 `horz=단(708)` 이므로 동일 분기 변경 영향 받음
- 새 위치 = 단 좌측 + 9.44 px (이전: 단 좌측 + 20.77 px)
- 한컴 2010/2020 / hancomdocs PDF 와 비교하여 정합 여부 확인

### 5단계: 광범위 sweep

- baseline: `/tmp/rhwp590_baseline/` (5 샘플 56 페이지)
- 패치 후 동일 출력 → diff
- 변경 페이지마다 의도된 정정 / 회귀 분류

### 6단계: clippy

- `cargo clippy -- -D warnings` 신규 경고 0

### 7단계: 보고서 + 머지

- `mydocs/working/task_m100_590_stage{1..6}.md` (단계별 완료 보고서)
- `mydocs/report/task_m100_590_report.md` (최종 보고서)
- `mydocs/orders/20260504.md` 갱신 (완료)
- `local/task590` → `local/devel` merge → push origin/devel

## 검증 기준

| 항목 | 기준 |
|---|---|
| exam_kor.hwp p17 [A] 좌측 | ≈ 126.6 px (단 좌측 + 9.44 px) |
| exam_kor.hwp p14 [A] 좌측 | 한컴 정답지 정합 (변경 후 11.33 px 좌측 이동) |
| `cargo test --lib` | 전체 GREEN |
| `cargo clippy` | 신규 경고 0 |
| 5 샘플 sweep 회귀 | 한컴 정답지 미정합 변경 0 |

## 회귀 위험 메모

- 본 변경은 Square wrap 표의 좌표 기준점 정정 (본질 정정).
- 14쪽 [A] 가 `horz=단` 인데 #480 분기로 정합화되어 있었다면, 새 분기에서 11.33 px 좌측 이동 발생 → 한컴 정답지 비교 필수.
- 다른 샘플의 `horz=단 + Square wrap` 조합 광범위 점검.

## 베이스라인 스냅샷

`/tmp/rhwp590_baseline/`
- exam_kor (20 페이지)
- exam_eng (8 페이지)
- exam_math (20 페이지)
- exam_science (4 페이지)
- exam_social (4 페이지)

총 56 페이지.
