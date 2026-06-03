# Task #1195 Stage 2 완료 보고서 — 셀 안 TAC 표 앞 빈 줄 높이 반영 보정

- **이슈**: #1195
- **브랜치**: `local/task1195`
- **단계**: Stage 2 / 4

## 결함 근원 (Stage 2 에서 호출 경로 확정)

Stage 1 에서 의심한 여러 경로(`layout_inline_table_paragraph`, `paragraph_layout.rs:3099`,
`shape_layout.rs:2319`)는 모두 이 표를 지나지 않음을 디버그로 확인. backtrace 로 실제 경로 확정:

```
4×1 표 ← layout_table_cells (table_layout.rs:2745) ← 외곽 12×7 표 layout_table ← layout.rs:4279
```

- 셀[28] 내용을 그리는 `layout_table_cells` 가 셀 안 TAC 표를 `layout_table(y_start =
  para_y_before_compose)` 로 호출.
- `para_y_before_compose` = 문단 compose 전 y = **표 앞 빈 줄("     ") 높이 미반영** →
  표가 제목 줄 위(631.8)에 배치 → 제목(639.8)과 8px 겹침.
- **이미지 TAC 분기(L2231)는 동일 문제를 이미 해결**(표/이미지 line_seg vpos 상대오프셋 가산)
  하지만 표 분기에는 그 처리가 누락 — 이것이 결함.

## 보정 (한컴 규칙)

작업지시자 규칙: **TAC 표는 앞 빈 여백 textRun 너비 다음에 배치하되, 잔여 너비가 부족하면
다음 줄(line feed)에 조판한다.** 즉 표는 문단 첫 줄이 아니라 표가 속한 line_seg(표 앞 빈 줄 다음).

`table_layout.rs` 표 분기에서 이미지 TAC 분기(L2231)와 동형으로 보정:

```rust
let table_anchor_y = if has_preceding_text && para.line_segs.len() > 1 {
    let first_vpos = para.line_segs.first().map(|f| f.vertical_pos).unwrap_or(0);
    let tbl_vpos = para.line_segs.last().map(|s| s.vertical_pos).unwrap_or(first_vpos);
    para_y_before_compose + hwpunit_to_px(tbl_vpos - first_vpos, self.dpi)
} else {
    para_y_before_compose
};
```
- `ctrl_area.y` 와 `layout_table` y_start 를 `para_y_before_compose` → `table_anchor_y` 로 교체.
- **가드: `has_preceding_text && line_segs.len() > 1`** — 표 앞에 텍스트(공백)가 선행하고
  line_seg 가 둘 이상(=표가 다음 줄로 내려간 경우)일 때만 보정 → 표가 문단 첫 요소인 기존
  케이스 무회귀 (feedback_hancom_compat_specific_over_general: 구조 가드).

변경: `table_layout.rs` +28 / −3 (표 분기 1곳).

## 효과 (계측)

| | 보정 전 | 보정 후 |
|---|---|---|
| 제목 "1." 글자 y | 639.8 | 639.8 (불변) |
| **4×1 표 top y** | **631.8** (제목 위, 8px 겹침) | **643.8** (제목 아래, 겹침 해소) |

표가 12px(= p[9] ls[0]→ls[1] vpos 차 900HU) 하향 → 겹침 제거.

## 회귀 점검 (전체 페이지)

| 검증 | 결과 |
|------|------|
| hcar-001 6페이지 before/after 좌표 비교 | ✅ **p1(4×1 표)만 변경, p2~p6 rect/text 좌표 차이 0** |
| `cargo test --tests` | ✅ **1893 passed, 0 failed** (98 스위트) |
| 작업지시자 SVG 시각 판정 | ✅ **통과** |
| WASM 빌드 | ✅ (`pkg/rhwp_bg.wasm` 11:08) |

- 가드가 표 앞 빈 줄 케이스만 한정 → 다른 페이지·다른 표·기존 샘플 전부 무회귀 확인.

## 산출물
- 보정 후 SVG: `output/poc/issue1195/stage2_fix2/hcar-001_001.svg`
- 전/후 비교: `output/poc/issue1195/before/`, `output/poc/issue1195/after/`

## 다음 단계 (Stage 3)

- 회귀 테스트 `tests/issue_1195_*.rs` 추가 (hcar-001 셀[28] 제목·표 비겹침 좌표 단언 + 기존 표 보호).
- 5×1 표(2.위탁, p[12] text_len=0)는 가드 미해당으로 변화 없음 — 겹침 여부 별도 확인.
