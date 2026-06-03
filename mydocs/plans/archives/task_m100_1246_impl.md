# 구현 계획서 — Task #1246: HeightCursor 미주 min-gap + pagination 정합

- **이슈**: #1246 (M100 / v1.0.0) — #1238 흡수
- **브랜치**: `feature/issue-1246-endnote-vpos-anchor`
- **수행계획서**: `plans/task_m100_1246.md`
- **Stage 1 조사**: `tech/endnote_vpos_anchor_1246.md`, `working/task_m100_1246_stage1.md`
- **작성일**: 2026-06-02

## 설계 요지 (Stage 1 확정)

전면 좌표 재작성 불필요. render 는 이미 `HeightCursor.vpos_adjust` 로 vpos→y 매핑하며
compact endnote between-notes 특례(forward-cap/backtrack/safe-vpos-backtrack #1209)를 보유한다.
**유일한 사각지대 = gap 이 부족할 때(stored vpos gap < between_notes, 특히 0=문22) 끌어올리는
min-gap 로직 부재.** 수정은 이 한 케이스 추가 + pagination 동일 예약(국소).

핵심 불변식:
- min-gap 은 **forward 흐름 & gap 부족 & 비-backtrack/비-rewind/비-stale-forward** 에만 적용.
  기존 분기(backtrack/forward-cap)가 **우선** → `issue_1209`(문12)·`issue_1189` 무회귀.
- render 가 gap 만큼 end_y 를 늘리면 pagination 도 같은 gap 을 예약해야 pi=475 overflow 가 없다.

## 단계 구성 (3단계: Stage 2~4)

### Stage 2 — HeightCursor min-gap (render)
1. `HeightCursor` 에 `endnote_between_notes_hu: i32` 필드 + 생성자 인자 추가. `build_single_column`
   에서 현재 섹션 미주 shape 의 between-notes 마진(HU)을 주입(미주 흐름 컬럼만, 그 외 0).
2. `vpos_adjust` compact endnote 분기(`suppress_large_forward_jump`)에 **min-gap 케이스** 추가:
   - 조건: 새 미주 제목(`compact_endnote_question_title` 재사용) & forward(`end_y >= y_offset`) &
     `end_y − prev_content_bottom_y < between_notes_px` & **기존 backtrack/rewind/stale-forward/
     forward-cap 케이스 미해당**.
   - 동작: `end_y = prev_content_bottom_y + between_notes_px`.
   - `shift_vpos_base_for_rendered_delta` 정합(후속 줄이 gap 복원 안 하도록 base 이동).
3. **신규 단위테스트**: stored gap 0/부족 → between_notes 로 확대, backtrack/forward-cap 우선 확인.
4. **검증**: HeightCursor 단위테스트(28+) 무회귀, 문22 0→~26.5px, `issue_1209_2022_sep_page10_q12`·
   `issue_1139` 미주군 무회귀.

### Stage 3 — pagination 정합 (compute_en_metrics)
1. `typeset.rs` 새 미주 첫 문단(forward & 비-rewind) fit/advance 에 동일 min-gap deficit 예약 —
   render 의 end_y 확대와 동일 위치·크기. (#1238 §5.2 에서 실패한 full-gap/blanket 방식이 아니라,
   render 와 정확히 같은 조건·deficit 으로 한정.)
2. **검증**: `issue_1189_2022_nov_pages10_12`(pi=475) overflow 해소, 전체 `cargo test` 무회귀
   (특히 issue_1209·1189·1139 미주군, 페이지 수 불변).

### Stage 4 — 시각·회귀·문서화·PR
- **시각**: 한컴 2022 PDF 정합 — 3-11월 page10/14/17, 3-09·10월 미주 경계 (`pdftoppm` 크롭 대조).
- **회귀**: 골든 스냅샷 + 전체 `cargo test`.
- **산출**: `report/task_m100_1246_report.md`. stream/devel 기준 squash PR.

## 단계별 커밋 정책

각 Stage 소스 + `working/task_m100_1246_stage{N}.md` 동반 커밋. 무관 rustfmt diff 금지.
#1238 render-클램프 코드는 본 브랜치에 잔존(레퍼런스)하므로, Stage 2 착수 시 **제거**하고
HeightCursor min-gap 으로 대체한다(클램프 hack 잔존 금지 — 수행계획서 §8).

## 회귀 가드

| 위험 | 방어 |
|------|------|
| #1209 backtrack(문12) 침범 | min-gap 은 비-backtrack & forward & gap부족 에만, 기존 분기 우선 |
| pi=475 overflow | render+pagination 동일 deficit 예약 (Stage 2+3 동시) |
| HeightCursor 단위테스트 회귀 | 신규 케이스 좁은 조건 + 신규 단위테스트, 28+ 기존 무영향 |
| 문29/30 late-tail, 2024 미주모양, 다단 | issue_1139/1189/1209 전체 + 골든 |
| between_notes_hu 주입 누락(미주 외 컬럼) | 미주 흐름 컬럼만 주입, 그 외 0 → 본문 무영향 |

## #1238 와의 관계

#1238 의 render-클램프(별 접근)는 폐기·제거. 본 타스크가 동일 목표(문22 외 between-notes)를
HeightCursor min-gap 으로 회귀 없이 달성하며 #1238 을 종결한다(closes #1238 후보).
