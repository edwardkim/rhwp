# Stage 2 (재개) 완료 보고서 — Task #853 (M100) — 섹션-top 제목 spacing_before 클램프

GitHub Issue: edwardkim/rhwp#853 · 브랜치: `local/task853`
배경: Stage 2(초안)에서 부분 정정 시도 후 작업지시자 옵션 B 로 일시 revert. 이후 작업지시자가 범위를 "제목 + band 간격 + overflow 전부"(옵션 C)로 확대 → 본 정정 재적용.

## 변경

`src/renderer/layout/paragraph_layout.rs::layout_composed_paragraph` (745-748 부근):
```rust
let is_column_top = (y - col_area.y).abs() < 1.0;
if start_line == 0 && spacing_before > 0.0 {
    if !is_column_top {
        y += spacing_before;
    } else if para_index == 0 {
        let vpos0_px = para
            .and_then(|p| p.line_segs.first())
            .map(|ls| hwpunit_to_px(ls.vertical_pos, self.dpi))
            .unwrap_or(0.0);
        y += spacing_before.min(vpos0_px.max(0.0));
    }
}
```
- column-top(`is_column_top`)이면서 **섹션 첫 문단(`para_index == 0`)** 인 경우, `spacing_before` 를 그 문단 첫 LINE_SEG 의 `vertical_pos`(한컴이 파일에 기록한 실제 렌더 첫 줄 위치)로 상한 클램프해 적용. 페이지 break 후 이어진 column-top(`para_index > 0`, `vertical_pos` 가 원래 레이아웃 위치를 담아 0 이 아닐 수 있음)은 종전대로 0.

## 결과

- ✅ shortcut.hwp 제목 "글 2010 단축키 일람표" baseline y=79.4 → **105.8** (+26.4px). top ≈ 83.8px ≈ 한컴 PDF top 83.6px. 한컴 기록값 `vertical_pos=1984 HU (26.45px)` 와 정합. `height_measurer`(이미 26.5px 포함) ↔ `paragraph_layout` 비대칭 해소.
- ✅ `cargo test --release` 전건 통과(34 test suites ok, 0 failed).
- 🔄 svg_snapshot 2건 golden 갱신(`UPDATE_GOLDEN=1`): `tests/golden_svg/issue-267/ktx-toc-page.svg`(목차 제목 y 129.0 → 132.8), `tests/golden_svg/issue-617/exam-kor-page5.svg`(셀 "6" y 179.1 → 186.7, "홀수형" y 169.9 → 174.8). 두 문서의 섹션-시작 문단도 LINE_SEG.vertical_pos 기준으로 재배치 — 한컴이 파일에 기록한 위치와 정합하므로 개선으로 판단. (`is_column_top` 예외가 한컴이 *드롭*한 경우엔 `vertical_pos==0` 이라 `min(sb,0)=0` 으로 무변화 — 자기 정합적.)
- ⚠ shortcut.hwp 페이지 수 7 → 8. 제목 +26.4px 가 1쪽 band 하나를 2쪽으로 밀어 연쇄. 한컴은 7쪽이므로 rhwp 2~8쪽이 한컴보다 짧지 않다는 뜻 — band-transition deficit(Stage 3)이 미해결이라 한쪽으로만 늘어남. + 기존 3쪽 overflow(pre-change 버그)가 page break 로 풀리며 +1쪽 기여. Stage 3(band 정정) 후 7쪽 회복 가능 여부 재확인.

## 다음 (Stage 3)

`다단나누기` 구분 칸 band 위·아래 간격(~17px zone gap + ~20px 헤더 띠 텍스트 line0 흡수) + 3쪽 overflow(#768 패턴). composer/table layout 변경 영역이라 회귀 위험 큼 → Stage 3-1(추가 진단·설계) → 승인 → Stage 3-2(구현) 로 분리. 상세는 구현 계획서 v3 (`mydocs/plans/task_m100_853_impl_v3.md`).

## 첨부
- 변경 후 SVG: `output/svg/sc853/` (8개)
