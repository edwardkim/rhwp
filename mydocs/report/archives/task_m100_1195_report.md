# 최종 결과 보고서 — Task #1195: 표 셀 안 빈 줄(줄간격 90%) + TAC 표 겹침 보정

- **이슈**: #1195
- **브랜치**: `local/task1195`
- **작성일**: 2026-06-01
- **샘플**: samples/hcar-001.hwp, samples/hwpx/hcar-001.hwpx, pdf-large/hwpx/hcar-001.pdf

## 요약

hcar-001 1쪽 신청서 표 셀[28] 안에서 "1. 개인정보 수집 및 이용 동의[필수]" 제목과
그 아래 TAC 표(4×1 "동의" 표)가 세로로 겹치는 결함을 해결. 표 앞 공백 textRun 이
선행하는 문단에서 표를 표 앞 빈 줄 다음 줄에 조판하도록 보정.

## 결함과 원인

- 셀[28] 표 anchor 문단(p[9])은 `공백 run "     "` → `TAC 표(4×1)` 순서(원본 XML 확정).
  line_segs 2개: ls[0] vpos=8424(표 앞 공백 줄), ls[1] vpos=9324(표 줄).
- `layout_table_cells`(table_layout.rs)가 셀 안 TAC 표를 `para_y_before_compose`(문단 첫 줄 y)
  기준으로 배치 → 표가 제목 줄 위(631.8)에 놓여 제목(639.8)과 8px 겹침.
- 이미지 TAC 분기는 표 앞 줄을 line_seg vpos 상대오프셋으로 처리하나, 표 분기엔 누락.

## 보정 (한컴 규칙)

작업지시자 규칙: **TAC 표는 앞 빈 여백 textRun 너비 다음에 배치하되 잔여 너비가 부족하면
다음 줄(line feed)에 조판한다.** 표는 문단 첫 줄이 아니라 표가 속한 line_seg 에 위치.

`table_layout.rs` 표 분기에서 이미지 TAC 분기와 동형으로 보정:

```rust
let table_anchor_y = if has_preceding_text && para.line_segs.len() > 1 {
    let first_vpos = para.line_segs.first().map(|f| f.vertical_pos).unwrap_or(0);
    let tbl_vpos = para.line_segs.last().map(|s| s.vertical_pos).unwrap_or(first_vpos);
    para_y_before_compose + hwpunit_to_px(tbl_vpos - first_vpos, self.dpi)
} else {
    para_y_before_compose
};
```
- `ctrl_area.y` 와 `layout_table` y_start 를 `table_anchor_y` 로 교체.
- **가드: `has_preceding_text && line_segs.len() > 1`** — 표 앞 텍스트(공백) 선행 +
  표가 다음 줄로 내려간 경우만 보정. 표가 문단 첫 요소면 기존 동작 → 무회귀
  (feedback_hancom_compat_specific_over_general: 구조 가드).
- 변경: `table_layout.rs` +28 / −3 (표 분기 1곳).

## 효과

| | 보정 전 | 보정 후 |
|---|---|---|
| 4×1 표 top y | 631.8 (제목 위, 8px 겹침) | **643.8** (제목 아래, 해소) |
| 제목 "1." y | 639.8 | 639.8 (불변) |

표가 12px(= ls[0]→ls[1] vpos 차) 하향 → 겹침 제거. CLI / native(`render_page_svg_native`)
양 경로 동일 적용.

## 회귀 테스트 (표 겹침 범위 한정)

`tests/issue_1195_cell_table_empty_line.rs`:
- native SVG 에서 4×1 표 top y(643.8) vs 제목 "[필수]" 글자 y(639.8) 추출 → **표 top > 제목 y**
  (비겹침) 단언. 표/제목 미발견 시 명시적 panic.
- **가드 유효성 입증(양방향)**: 보정 ON → pass / 보정 OFF(코드 무력화) → FAIL("표 top=631.8 ≤
  제목 y=639.8"). 결함을 실제로 잡음.
- golden 전체 스냅샷은 무관 변경에도 실패 → 표 겹침으로 범위 좁힘(작업지시자 지시).

## 검증

| 항목 | 결과 |
|------|------|
| 작업지시자 SVG 시각 판정 | ✅ 통과 |
| hcar-001 6페이지 회귀 (before/after) | ✅ p1(4×1 표)만 변경, p2~p6 무변경 |
| `cargo test --tests` | ✅ **1894 passed, 0 failed** |
| 회귀 테스트 가드 유효성 | ✅ 보정 OFF 시 FAIL 입증 |
| WASM 빌드 | ✅ |
| 5×1 표(2.위탁) | ✅ 원래 비겹침(text_len=0, 가드 미해당) — 정상 |

## 단계 경과

- Stage 1: 정밀 계측(제목 639.8 vs 표 631.8 = 8px 겹침), 결함 경로 backtrace 특정.
- Stage 2: `layout_table_cells` 표 분기 보정(이미지 TAC 동형) → 643.8 해소. 시각 판정 통과.
- Stage 3: 좌표 정밀 회귀 테스트 + 가드 유효성 입증 + hcar 샘플 픽스처(LFS).
- Stage 4: 전체 테스트 0 failed + WASM + 본 보고서.

## 비고

- **혼란 근원 규명**: 진행 중 "보정 없이도 테스트 통과" 가 반복됐는데, 보정이 Stage 2 에서
  커밋되어 `git stash <file>` 로는 워킹트리 변경이 아니라 stash 되지 않아 항상 보정본이
  출력된 착시였음. 코드 직접 무력화(`if false`)로 631.8/FAIL 을 확인해 입증.
- **issue-267 golden 충돌**: 워킹트리에 UU(고아 충돌)로 존재하나 Task #1195 와 무관(KTX vs hcar),
  devel 에 정상본 존재, 본 작업 커밋 미포함. devel 머지 후 정상본 회귀.
- 추적 경로가 여러 번 빗나갔으나(layout_inline_table_paragraph / paragraph_layout 3099·2203 /
  shape_layout 2319 / table_layout 288), backtrace 로 실제 경로(layout_table_cells)를 확정.
- 작업지시자 인사이트(좁게 집중 / 표 앞 공백 줄 / line-feed 규칙 / 표 겹침 범위 한정 /
  골든 대신 좌표)가 매 단계 정확했음.
