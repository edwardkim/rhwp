# 최종 보고서 — Task #1070: HWPX TAC 표 문단 후속 본문 표 높이만큼 하강 overflow

- 이슈: edwardkim/rhwp#1070
- 브랜치: `local/task1070` (stream/devel `be2a71c4` 기준)
- 수정 파일: `src/renderer/typeset.rs` (1 곳) + 회귀 가드 `tests/issue_1070_*.rs` (신규)

## 증상
거의 한 페이지 크기 `treat_as_char`(TAC) 표가 첫 줄에 있고 그 뒤에 본문 줄이 있는 문단에서,
표는 거의 fit하나 후속 본문 텍스트가 표 높이만큼 추가 하강해 편집영역 하단을 348~472px 초과.

| 파일 | 문단 | overflow(전→후) |
|------|------|------|
| `2025년 기부·답례품…양식.hwpx` | pi=25 (page2) | 472 → 해소(표잔여 5.6) |
| `hwpx/hwpx-h-02.hwpx` | pi=51 (page2) | 348 → 7.4 |
| `hwpx/2025년 2분기 해외직접투자 (최종).hwpx` | pi=51 (page2) | 348 → 7.4 |

## 근본 원인
`place_table_with_text`(`typeset.rs:2517`) 의 `post_table_start` 산식이 `attr & 0x01`(HWP5
TAC 비트)에만 의존. HWPX TAC 표는 비트0=0 → 마지막 else → `post_table_start = pre_end(=0)`
→ 표줄(line0)이 post-text PartialParagraph(0..total_lines)에 포함 → 후속 본문 줄이 표 높이
만큼 하강. HWP5 TAC(attr&0x01)는 `pre_end.max(1)` 로 이미 표줄 제외 → **HWPX 한정 갭**.

#1068(제목줄 LINE_SEG lh over-inflation)과 외형 유사하나 별개 원인. #1068 PR 의
`already_covered` 가드로 lh 보정이 안 되는 표(실제 linesegarray 보유)에서 발생.

## 수정
```rust
} else if table.attr & 0x01 != 0 {
    pre_table_end_line.max(1)
} else if table.common.treat_as_char && total_lines > pre_table_end_line + 1 {
    // 표줄 다음에 실제 본문 줄이 있는 HWPX TAC 표: 표줄을 post-text 에서 제외
    pre_table_end_line + 1
} else if is_last_table && !is_first_table {
    0
} else {
    pre_table_end_line
};
```
`treat_as_char && total_lines > pre_end + 1`(표줄 뒤에 실제 본문 줄) 일 때만 표줄 제외.
**단일줄 TAC 표는 불변** — 단순 blanket 확장(`|| treat_as_char`)은 단일줄 표의 post-text 까지
제거해 mel-001 +5 회귀(Stage 1 실험으로 확정), 정밀 게이트로 회피.

## 검증
- 재현 3파일 본문 overflow 해소. dump-pages 구조 정합(`lines=0..2 → 1..2`).
- 전수 sweep: 3057→3043 lines / 382815→376707px (−14/−6108, **회귀 0**, sample16 35→15·aift
  5→4·tac-img-02.hwpx 7→6 개선).
- 회귀 가드 3 신규(SVG text max_y ≤ 페이지 높이) 통과.
- 골든 SVG 8/8, cargo test --release lib 1324 + 통합 0 failed, clippy/fmt clean.

## 경계 안전성 (Stage 2 페이퍼 검증)
신규 arm 은 `treat_as_char && pre_end==0 && total_lines>1` 만 발동. tac_wrap_split / HWP5
attr&0x01 / 단일줄 / vertical_offset / Square wrap / 비-TAC / page-larger 와 모순 없음(전수
sweep 0 회귀로 실증).

## 후속
A군 해소. 동일 인벤토리의 B군(kps-ai PartialTable 758px), C군(교육/실전 통합 누적 드리프트),
D군(pr-149 Shape)은 별도 타스크.
