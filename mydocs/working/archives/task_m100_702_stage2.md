# Task #702: Stage 2 단계별 보고서 — 본질 1 정정

## 작업 개요

Stage 1 진단에서 식별한 본질 1 (`SectionColumnDef` 후속 갱신 누락) 의 실제 결함을 정정. 진단 단계에서 결함이 두 가지 본질 결함의 복합임을 발견.

## 정정 결함 (2건)

### 결함 A — Distribute 다단의 짧은 컬럼 vpos-reset 검출 임계값

**위치**: `src/renderer/typeset.rs:430-446` (수정 전 410-417)

**원인**:
```rust
// 수정 전
let trigger = if st.col_count > 1 {
    cv < pv && pv > 5000   // 다단 임계값 (Task #470)
} else {
    cv == 0 && pv > 5000   // 단단 임계값 (Task #321)
};
```

`pv > 5000` 임계값은 Task #321/#470 도입 시 partial-table split 잔재 (Issue #418) 와의 false positive 회피용. 그러나 짧은 Distribute 컬럼 (예: 지우기 3+3 분배) 에서 마지막 paragraph vpos=3000 < 5000 으로 trigger 미발동.

**정정**:
```rust
// 수정 후 ([Task #702])
let is_distribute = st.col_count > 1
    && matches!(st.current_zone_column_type, ColumnType::Distribute);
let trigger = if st.col_count > 1 {
    if is_distribute {
        cv < pv && pv > 0   // Distribute: 짧은 컬럼 허용
    } else {
        cv < pv && pv > 5000   // Normal: 기존 임계값 유지
    }
} else {
    cv == 0 && pv > 5000
};
```

`ColumnType::Distribute` (HWPX `BalancedNewspaper`) 한정 임계값 완화. Normal/단단은 영향 없음.

**전제 조건 — `current_zone_column_type` 전파**:
- `TypesetState` 필드 추가
- `TypesetState::new` 시그니처에 `column_type: ColumnType` 추가 → `typeset_section` 에서 `column_def.column_type` 전달
- `process_multicolumn_break` 에서 새 ColumnDef 매칭 시 `current_zone_column_type = cd.column_type` 갱신

### 결함 B — Page/Column break + 새 ColumnDef 미적용

**위치**: `src/renderer/typeset.rs:396-441`

**원인 (Stage 1 진단 시 미발견, Stage 2 진행 중 발견)**:

shortcut.hwp p2 의 파일/미리보기/편집 sections 는 다음 패턴 사용:
- `[쪽나누기] + 단정의:1단 + 표(header)` (header zone)
- `[단나누기] + 단정의:2단 배분` (content zone)

기존 코드:
- `MultiColumn` break 만 `process_multicolumn_break` 호출 → ColumnDef 적용
- `Page` / `Column` break 는 단순 page-break / column-advance 만 처리 → 동반된 ColumnDef 무시

결과: 페이지 2 에서 col_count 가 이전 zone 의 2단 유지 → 파일 right column 진입 시 `advance_column_or_new_page` 가 새 페이지 강제 → 페이지 분기 폭주 (10쪽).

**정정**:

`Page`/`Column` break 처리 전 새 ColumnDef 검출:
```rust
let new_col_def_opt: Option<ColumnDef> = para.controls.iter().find_map(|c| {
    if let Control::ColumnDef(cd) = c { Some(cd.clone()) } else { None }
});
let has_diff_col_def = new_col_def_opt.as_ref().map(|cd| {
    cd.column_count.max(1) != st.col_count
        || cd.column_type != st.current_zone_column_type
}).unwrap_or(false);
```

처리:
- `MultiColumn`: 기존대로 `process_multicolumn_break`
- `Column + has_diff_col_def`: `process_multicolumn_break` 호출 (zone 재정의)
- `Column + 동일 ColumnDef`: 기존 `advance_column_or_new_page`
- `Page/Section + has_diff_col_def`: `force_new_page` 후 새 ColumnDef 적용 (col_count, layout, column_type 갱신)

## 변경 파일

- `src/renderer/typeset.rs`:
  - import `ColumnType` 추가
  - `TypesetState` 필드 `current_zone_column_type` 추가
  - `TypesetState::new` 시그니처 변경 (column_type 인자)
  - `typeset_section` 에서 `column_def.column_type` 전달
  - 메인 루프 vpos-reset trigger 분기 수정 (Distribute 완화)
  - 메인 루프 Page/Column break 핸들러 + 새 ColumnDef 적용
  - `process_multicolumn_break` 에서 ColumnDef 적용 시 `current_zone_column_type` 갱신
- `tests/issue_702.rs` (신규):
  - `shortcut_distribute_short_column_split`: 페이지 수 ≤ 8 검증
  - `shortcut_page2_has_three_sections`: 페이지 2 SVG 에 파일/편집 헤더 모두 존재 검증

## 검증 결과

### shortcut.hwp dump-pages

수정 전 (Stage 1 시점):
- 총 10페이지
- 페이지 1 단 5: items=6 (지우기 6항목 1단 적층)
- 페이지 2 단 0/1: items=2/5 (파일 header + left col 5항목만)
- LAYOUT_OVERFLOW: 다수 (페이지 8 에서 40~60px overflow)

수정 후:
- 총 8페이지 (PDF 7쪽 vs SVG 8쪽, 1쪽 차이)
- 페이지 1 단 5/6: items=3/3 ✓ (지우기 정상 분할)
- 페이지 2: 단 0~8 (파일 header + 2단 5+5 + 미리보기 header + 2단 5+4 + 편집 header + 2단 12+11) ✓
- LAYOUT_OVERFLOW: 0 (export-svg 시 1건만 발생, 페이지 8 마지막 부분)

### 시각 검증 (qlmanage 비교)

- Page 1 SVG: 커서 이동 (2단 14+13) + 지우기 (2단 3+3) ✓ PDF 정합
- Page 2 SVG: 파일/미리보기/편집 3 섹션 동일 페이지 ✓ PDF 정합

### `cargo test --release` 전체 회귀

```
1248+ tests run, 0 failed
- exam_eng_multicolumn: 14 passed (Task #470 회귀 차단 ✓)
- issue_418: 1 passed (단단 partial-table 잔재 ✓)
- svg_snapshot: 7 passed (시각 회귀 ✓)
- issue_702 (신규): 2 passed
```

핵심 회귀 차단 테스트 모두 통과. Distribute 한정 정정으로 Normal/단단 분기 영향 없음.

## 잔여 이슈

- 페이지 8 마지막 5항목 (그림 그리기 후반부) 가 페이지 8 로 넘침 — PDF 는 페이지 7 에 모두 fit. 1쪽 차이는 column height 정합 미세 조정 영역 (본질 2 영역).
- 페이지 1 커서 이동 right col 의 단축키 텍스트 (Ctrl+(회색)5, Home 등) 가 SVG 에 누락되어 보이는 현상 — 별개 결함으로 분리 (text rendering / column right-edge 정합).
- 부수 결함 (탭 leader, PUA 글자 "한글 2010" → "ㅎ글2010", 바탕쪽 자동번호) — 본 사이클 범위 외, 별도 이슈 후보.

## 승인 요청

본 단계별 보고서대로 Stage 3 (광범위 회귀 검증 + 시각 판정) 진입 가능 여부 승인 부탁드립니다.

Stage 3 작업 항목:
1. 광범위 샘플 export-svg 회귀 (KTX, aift, hwp-multi-001, kps-ai, hwpx/aift)
2. shortcut.hwp 시각 정합 페이지별 비교 (qlmanage)
3. 한컴 2010/2020 정답지 비교 (가능한 경우)
