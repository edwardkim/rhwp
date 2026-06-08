# 구현계획서 — 단 구분선 이중 렌더 수정 (M100 #1333)

수행계획서: `task_m100_1333.md`

## 단계 구성 (3단계)

### Stage 1 — 혼재 케이스 사전 조사

수정 적용 전, `has_zone_specific_layout = true` 인 페이지에 `zone_layout = None` 인 다단(2단+) `cc` 가 섞일 수 있는지 확인한다.

- `current_zone_layout` 설정/해제 경로 분석:
  - `typeset.rs:651` 초기 None, `:810` reset None
  - `typeset.rs:1191`, `:6295`, `engine.rs:1049` 에서 `Some(new_layout)` 설정
- 한 페이지의 cc 들이 zone_layout None/Some 혼재 가능한지, 혼재 시 None cc 가 다단인지 판정.
- 결과에 따라 Stage 2 수정안 확정:
  - **(A) 혼재 없음/혼재 None cc 는 단일단** → 단순 조건 추가로 충분.
  - **(B) 혼재하며 None cc 가 다단** → 별도 분기(해당 cc 만 page-level 폴백) 필요.

산출: `working/task_m100_1333_stage1.md` (조사 결론 + 채택안)

### Stage 2 — 코드 수정

채택안에 따라 `src/renderer/layout.rs:2616-2622` 수정.

(A) 채택 시:
```rust
// 본 zone 이 다단 + 구분선 보유 시 종료 시점에 emit 하기 위해 기록.
// [Task #1333] zone-specific layout 이 실제로 존재할 때만 zone emit.
// zone_layout=None(페이지 전체 단일 다단)은 page-level build_column_separators
// 가 전체 높이로 그리므로 이중 렌더 방지.
if col_content.zone_layout.is_some()
    && zone_layout.column_areas.len() >= 2
    && zone_layout.separator_type > 0
{
    prev_zone_layout_for_sep = Some(zone_layout.clone());
    prev_zone_sep_y_start = current_zone_start_y.max(zone_layout.body_area.y);
} else {
    prev_zone_layout_for_sep = None;
}
```

(B) 채택 시: 위 조건 + page-level 가드(`layout.rs:1139`)를 "zone emit 으로 커버되지 않는 None 다단 cc 존재 시 그림" 형태로 보완.

산출: 소스 커밋 (`Task #1333: 내용`)

### Stage 3 — 검증 + 최종 보고

- `cargo build` / `cargo test` 통과 확인.
- `export-svg` 재실행 후 대상 문서 23쪽 단 구분선이 페이지당 1개·전체 높이(y2≈1092.3)인지 추출 검증.
- HWPX 쌍 동일 확인.
- 다단 zone 보유 샘플(있으면)로 회귀 없음 확인.
- `clippy` 경고 없음 확인 (변경 범위).

산출: `report/task_m100_1333_report.md`

## 검증 기준 (Stage 3 합격선)

1. 대상 HWP/HWPX 23쪽: 단 구분선 x=396.9 세로선이 **페이지당 정확히 1개**, y 범위 ≈ 90.7→1092.3.
2. `cargo test` 전부 통과.
3. 다단 zone(단나누기/구역전환) 샘플 단 구분선 정상 (회귀 없음).
