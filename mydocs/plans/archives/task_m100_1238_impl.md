# 구현 계획서 — Task #1238: 미주 between-notes margin 누락

- **이슈**: #1238 (M100 / v1.0.0)
- **브랜치**: `feature/issue-1238-between-notes-margin` (base: `stream/devel`)
- **수행계획서**: `plans/task_m100_1238.md`
- **Stage 1 조사**: `tech/between_notes_multiline_1238.md`
- **작성일**: 2026-06-02

## 설계 요지 (Stage 1 확정)

between-notes(미주 사이 7mm=1984HU)는 `typeset.rs:2055` 에서 **직전 미주 마지막 문단의
`last_seg.line_spacing = 1984`** 주입으로만 적용된다(pagination_gap=0). 그러나
`paragraph_layout.rs:4156` 다줄 미주 문단(`endnote_line_vpos_base`) 경로가 **마지막 줄 trailing 을
0.0 으로 버려** 주입값을 무시 → 다줄로 끝나는 미주 다음 제목이 붙는다(문22 11.3px). 단일줄
경로(L4173)는 line_spacing 을 포함하므로 정상.

**수정**: 다줄 경로 마지막 줄 trailing 을, **다른 미주가 뒤따르는 마지막 문단**(=주입 위치)에서만
`line_spacing_px` 로 복원. 단일줄 경로·같은 미주 내부·문서 마지막 미주는 무영향 → 이중가산 없음.

## 단계 구성 (3단계)

### Stage 1 — 원인 특정 (완료, 코드 무변경)
- 산출: `tech/between_notes_multiline_1238.md`, `working/task_m100_1238_stage1.md`.

### Stage 2 — 회귀 없는 게이트 수정
- `layout.rs` 에 헬퍼 `endnote_para_has_different_endnote_successor(para_index) -> bool` 추가:
  `endnote_para_sources` 의 local idx 기준, 다음 문단의 (section_index, para_index, control_index)
  가 현재와 **다르면** true(다음 local 없으면 false = 문서 마지막 미주).
- `paragraph_layout.rs:4156` 다줄 경로 trailing 분기:
  ```rust
  let trailing = if line_idx + 1 < end {
      line_spacing_px
  } else if self.endnote_para_has_different_endnote_successor(para_index) {
      line_spacing_px   // #1238 between-notes margin (주입된 last_seg.line_spacing)
  } else {
      0.0
  };
  ```
- **검증**: 문22 above-gap ~27~38px, 골든 스냅샷, 전체 `cargo test` (issue_1139/1189 포함) 무회귀.

### Stage 3 — 시각·회귀·문서화
- **시각**: 문22(외 다줄로 끝나는 미주) above-gap PDF(한글 2022) 14쪽 정합 + 단일줄 경계 비회귀.
- **회귀**: 골든 + 전체 `cargo test`, 특히 issue_1139(3-09월 7mm/20mm)·1189 페이지 수 불변.
- **산출**: `report/task_m100_1238_report.md`.

## 단계별 커밋 정책

각 Stage 소스 + `working/task_m100_1238_stage{N}.md` 동반 커밋. 무관 rustfmt diff 금지.

## 회귀 가드

| 위험 | 방어 |
|------|------|
| 3-09월 between-notes 이중가산 | 게이트가 주입 위치(=다른 미주 successor)와 정확 일치, 단일줄 경로 불변 |
| 페이지 수 변동 | issue_1139 page-count 테스트 + 골든 스냅샷 |
| #1236(PR#1240) 머지 충돌 | L4156 의미 직교(미주 내부 vs 미주 사이) → OR 결합 |
