# 구현 계획서 — Task #1239: 미주 인라인 수식 줄 병합

- **이슈**: #1239 (M100 / v1.0.0)
- **브랜치**: `feature/issue-1239-equation-multiline-merge` (base: `stream/devel`)
- **수행계획서**: `plans/task_m100_1239.md`
- **작성일**: 2026-06-02

## 설계 요지

미주 문단 pi=602(S= 블록)는 텍스트가 없고(composed 줄 runs=0) 각 "=…" 줄이 **인라인 수식
(U+FFFC, treat-as-char)** 이다. 줄 개수(5)·LINE_SEG vpos(균등)는 정상이나, **두 수식이 같은
줄에 배정**되어 한 줄이 비고 다음 줄에 겹친다. 인라인 수식의 **줄 배정**(char 위치 ↔ 줄 경계
`line_break_char_indices`, `run_tacs`/`tac_ci` dispatch)을 한컴 LINE_SEG 와 정합시킨다.

## 단계 구성 (3단계)

### Stage 1 — 정확한 배정 버그 지점 특정 (코드 무변경, 조사 보고서)

**목표**: 문20 두 수식이 같은 줄로 배정되는 정확한 지점·조건 규명.

- 인라인 수식 dispatch 경로 추적: `paragraph_layout.rs` line 루프의 `run_tacs`(L2851)/
  `Control::Equation`(L3021) — 각 수식 `tac_ci` 가 어느 `line_idx` 에 배정되는지.
- `line_break_char_indices`(L609-634)가 U+FFFC(8 UTF-16 단위) 수식 placeholder 의 char 위치를
  올바른 줄 경계로 매핑하는지 — 두 수식의 char 위치가 같은 줄 범위로 떨어지는지 진단 로그로 확인.
- 한컴 line_segs[i].text_start 와 rhwp char_offsets 매핑 대조 (U+FFFC offset 정합 여부).
- **산출**: `tech/endnote_inline_eq_line_1239.md` (배정 매핑 표 + 버그 지점).
- **승인 게이트**: 원인 확정 후 수정.

### Stage 2 — 줄 배정 수정

**목표**: 각 인라인 수식을 한컴 LINE_SEG 와 동일 줄에 배정.

- Stage 1 지점 수정 — 수식 placeholder char 위치 ↔ 줄 경계 매핑 보정 (U+FFFC offset 또는
  line_break_char_indices 단조성/매핑).
- 일반 문단 회귀 방지: U+FFFC offset 정합 또는 미주 한정 조건 게이트.
- **검증**: 문20 S= 5줄 분리 SVG ↔ PDF 1차 정합.

### Stage 3 — 시각·회귀·문서화

- **시각**: 문20 S= 블록 PDF 줄별 분리 정합 + 빈 줄 해소.
- **회귀**: 골든 스냅샷 + 전체 `cargo test` + 인라인 수식 포함 문서(exam 등) 시각 점검.
- **수치**: 인라인 수식별 배정 줄(vpos) ↔ line_segs 1:1.
- **산출**: `report/task_m100_1239_report.md`.

## 단계별 커밋 정책

각 Stage 완료 시 소스 + `working/task_m100_1239_stage{N}.md` 동반 커밋. 무관 rustfmt diff 금지.

## 검증 도구 요약

| 항목 | 명령 |
|------|------|
| 시각 대조 | `rhwp export-svg samples/3-11월_실전_통합_2022.hwpx -p 12` ↔ `pdf/…2022.pdf` |
| 배정 진단 | 임시 `DBG_EQM` 로그 (composed 줄 ↔ 수식 tac_ci ↔ line_idx) |
| 회귀 | 골든 스냅샷, `cargo test` |
