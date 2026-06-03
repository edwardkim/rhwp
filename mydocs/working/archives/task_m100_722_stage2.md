# Task #722 Stage 2 단계별 보고서 — 본질 정정 (정정안 E)

## 개요

Stage 1 본질 재진단 (PDF 권위 시각 판정) 결과 D'' 가설 폐기 후 정정안 E 적용:
**`src/renderer/typeset.rs` wrap_around state machine 에서 anchor host paragraph (자기 자신) 도 `current_column_wrap_anchors` 에 등록**.

## 1. Rollback (D'' 가설 기반 정정 전부 제거)

다음 변경 전부 rollback:
- `src/renderer/layout/paragraph_layout.rs` — ComposedLine merge 분기, host_pic_vertical_offset 가드, wrap_anchor.anchor_vpos 가드 (130+ 줄)
- `src/renderer/pagination.rs` — `WrapAnchorRef.anchor_vpos` 필드 추가
- `src/renderer/typeset.rs` — anchor_vpos 추출 + DEBUG_TASK722 로그

`git checkout --` 으로 rollback 후 git status 깨끗.

## 2. 본질 정정 — `src/renderer/typeset.rs:687-697`

기존: paragraph 175 (anchor host) 처리 시점에 `st.wrap_around_*` state 만 설정. paragraph 175 자체는 `current_column_wrap_anchors` 에 미등록.

정정: paragraph 175 (anchor host) 자기 자신을 `current_column_wrap_anchors` 에 등록.

```rust
if has_non_tac_pic_square {
    let anchor_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
    let anchor_sw = para.line_segs.first().map(|s| s.segment_width as i32).unwrap_or(0);
    if anchor_cs > 0 || anchor_sw > 0 {
        st.wrap_around_cs = anchor_cs;
        st.wrap_around_sw = anchor_sw;
        st.wrap_around_table_para = para_idx;
        st.wrap_around_any_seg = true;
        // [Task #722] anchor host paragraph 자체도 wrap_anchors 등록
        st.current_column_wrap_anchors.insert(
            para_idx,
            crate::renderer::pagination::WrapAnchorRef {
                anchor_para_index: para_idx,
                anchor_cs,
                anchor_sw,
            },
        );
    }
}
```

11 줄 추가 (한 곳 단일 분기).

## 3. 시각 판정 (rsvg-convert PNG 변환 자체 판정)

### Before (Stage 2 D'' 정정 적용 상태)

`output/task722_compare/current_p8.png` — paragraph 175 가 col_area 전체 폭 (image 영역 침범) 위치, image z-order 후 그려져 텍스트 가려짐.

### After (정정안 E 적용)

`output/task722_compare/fixed_p8.png` — paragraph 175 "아래에 디렉토리 트리 각 부분의 역할에 대하여 설명하였다." 가 image 우측 wrap zone 첫 줄에 정확히 배치.

### PDF 권위 자료 (한컴 2022)

`output/task722_compare/pdf_p8-08.png` — paragraph 175 image 우측 wrap zone 첫 줄. 본 환경 After 정합.

## 4. 결정적 검증 (release 모드)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | **1165 passed; 0 failed** (회귀 0) |
| `cargo test --release` | 전체 GREEN |
| `cargo clippy --release` | 신규 경고 0 |

svg_snapshot, issue_546, issue_554 영역 전부 통과.

## 5. 회귀 위험 영역 확인

- typeset.rs 단일 분기 (`has_non_tac_pic_square` 발현 영역만 발현)
- IR 무수정 (HWP3 파서 / Document model 무영향)
- 기존 `current_column_wrap_anchors` 데이터 모델 그대로 사용
- wrap=Square 그림이 없는 paragraph 무영향

## 6. Stage 3 진행 승인 요청

광범위 페이지네이션 sweep + 최종 검증 진행 승인 요청.
