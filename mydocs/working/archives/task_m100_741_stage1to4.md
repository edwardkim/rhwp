# Task #741 Stage 1~4 단계별 통합 보고서

## 진행 단계 요약

| Stage | 영역 | 상태 |
|-------|------|------|
| 1 | 본질 진단 — Document IR Picture + HWP3 파서 그림 + 폰트 추적 | 완료 |
| 2 | image placeholder Document IR 확장 + 본 환경 정정 | 완료 |
| 3 | TAC 그림 paragraph line_spacing 정합 (HWP3 파서) | 완료 |
| 4 | HWP5 변환본 paragraph 26 페이지 분할 정합 (vpos-reset 후속 가드) | 완료 |

## Stage 1 — 본질 진단

### 결함 1 — HWP3 외부 file path 그림 IR 누락

- `src/model/image.rs::ImageAttr` — `external_path` 필드 부재
- `src/parser/hwp3/mod.rs:844~857` — `pic_name` 파싱 ✓, `bin_data_id` 부여 ✓, IR 미전달 ✗

### 결함 2 — TAC 그림 paragraph line_spacing

| 항목 | HWP3 native | HWP5 변환본 (한컴 정합) |
|------|-------------|------------------------|
| paragraph 12 ls | **9379 HU (33mm)** | **600 HU (2mm)** |
| paragraph height | 25011 HU (88mm) | 16232 HU (57mm) |

본 환경 HWP3 파서가 TAC 그림 paragraph 의 ls 를 `th × (line_spacing_ratio - 100) / 100` (image height × 0.6) 큰 값으로 계산. 한컴 정합 작은 ls=600.

### 폰트 매핑

paragraph 4 ("Technical Bulletins") HWP3 native = "신명 디나루" / HWP5 변환본 = "돋움". 한컴이 HWP3 → HWP5 변환 시 font_faces array 자체를 재구성. HWP3 native 결과는 **spec 정합** (한컴 viewer 다이얼로그 폰트 dropdown 도 "신명 디나루" 표시).

### 결함 3 — HWP5 변환본 paragraph 26 페이지 분할

paragraph 26 ("● 제목차례 ●") vpos=0 hint 인데 본 환경 typeset 가 페이지 1 안에 잘못 표시. paragraph 22 anchor (cs=11084) active 유지로 vpos-reset 가드 (Task #724 anchor cs=0 한정) 미발현.

## Stage 2 — image placeholder 정정

### 정정 영역

| 파일 | 변경 |
|------|------|
| `src/model/image.rs` | `ImageAttr.external_path: Option<String>` 추가 |
| `src/parser/hwp3/mod.rs:844` | `pic_type=0/1/2` 시 `pic_name` → `external_path` 전달 |
| `src/renderer/render_tree.rs` | `ImageNode.external_path` 추가 |
| `src/renderer/layout/picture_footnote.rs` | external_path 전달 |
| `src/renderer/layout/table_cell_content.rs` | external_path 전달 |
| `src/renderer/svg.rs:1069~` | 빈 binary + external_path placeholder (점선 사각형 + file path 텍스트) |
| `src/main.rs` | dump 출력 external_path |
| `src/paint/json.rs`, `src/serializer/hwpx/picture.rs`, `src/document_core/commands/object_ops.rs` | ImageAttr 구성 시 external_path: None |

### 검증

paragraph 1 image (`D:\Work\Gwjang\temp\images\oracle.gif`) placeholder 표시 정합 (회색 사각형 + 점선) — 한컴 viewer 정합.

## Stage 3 — TAC line_spacing 정정

### 정정 영역 (`src/parser/hwp3/mod.rs:1395~1413`)

```rust
let has_tac_picture = para.controls.iter().any(|c| {
    match c {
        Control::Picture(p) => p.common.treat_as_char,
        Control::Shape(s) => {
            if let crate::model::shape::ShapeObject::Picture(p) = s.as_ref() {
                p.common.treat_as_char
            } else { false }
        }
        _ => false,
    }
});
ls = if has_tac_picture {
    600
} else {
    th * (line_spacing_ratio - 100) / 100
};
```

### 검증

- paragraph 12 ls: 9379 → 600 ✓
- paragraph 13 vpos: 50931 → 42152 ✓
- 페이지 1 = 29 items (페이지 분할 정합)
- 총 페이지: 764 → 763 (HWP5 변환본 정합)

## Stage 4 — vpos-reset 후속 가드

### 정정 영역 (`src/renderer/typeset.rs:594~620`)

매칭 실패 분기 (line 594~) 후 vpos-reset trigger 추가 가드. wrap_around active 종료 후 paragraph vpos=0 hint 발현 시 advance_column_or_new_page.

```rust
} else {
    st.wrap_around_cs = -1;
    st.wrap_around_sw = -1;
    st.wrap_around_any_seg = false;
    // [Task #741 Stage 4] 매칭 실패 paragraph 의 vpos=0 hint (page break 의도)
    if para_idx > 0 && !st.current_items.is_empty() {
        let prev_para = &paragraphs[para_idx - 1];
        let curr_first_vpos = para.line_segs.first().map(|s| s.vertical_pos);
        let prev_last_vpos = prev_para.line_segs.last().map(|s| s.vertical_pos);
        if let (Some(cv), Some(pv)) = (curr_first_vpos, prev_last_vpos) {
            let trigger = if st.col_count > 1 {
                cv < pv && pv > 5000
            } else {
                cv == 0 && pv > 5000
            };
            if trigger {
                st.advance_column_or_new_page();
            }
        }
    }
}
```

### 검증

- HWP5 변환본 페이지 1 끝: paragraph 24 (vpos=62312)
- HWP5 변환본 페이지 2 시작: paragraph 26 ("제목차례") — 한컴 정합 ✓
- 광범위 sweep: 회귀 0 (kps-ai 등 영향 없음)

## 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | 1166 passed |
| `cargo clippy --release` | 신규 경고 0 |
| 광범위 sweep | DIFF 0 (회귀 0) |

## 시각 판정 (한컴 viewer 정합 ★)

- HWP3 native 페이지 1: image placeholder 3개 + Technical Bulletins + 창원대학교 (한컴 정합)
- HWP5 변환본 페이지 1: image placeholder 3개 + Technical Bulletins + 창원대학교 (한컴 정합)
- HWP5 변환본 페이지 2: paragraph 26 ("제목차례") 시작 (한컴 정합)

## Stage 5 영역

추가 진단 발견 (사용자 시각 판정):
- HWP3 native paragraph 26 의 가로선 char "═" + ■ 표시자 누락 — `src/parser/hwp3/johab.rs` 미지원 char skip 결함
- HWP3 native ParaShape tab_def 누락 — 페이지 번호 우측 정렬 + 가로선 채움 미지원

본 task #741 안에서 Stage 5+ 진행 (HWP3 char encoding + ParaShape tab).
