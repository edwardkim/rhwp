# 구현 계획서 — Task #1236: 미주(해설) 영역 줄간격 압축

- **이슈**: #1236 (M100 / v1.0.0)
- **브랜치**: `feature/issue-1236-endnote-line-spacing` (base: `stream/devel` f83c43b5)
- **수행계획서**: `plans/task_m100_1236.md`
- **작성일**: 2026-06-02

## 설계 요지

줄간격(line spacing)·vpos 누적은 `src/renderer/typeset.rs`(페이지네이션, `lh_sum`/`ls_sum`/
`trail_ls`)와 라인 레이아웃 경로에서 계산된다. 일반 문단은 ParaShape 의 line spacing 으로
줄높이(lh)+줄간격(ls)을 누적해 PDF 와 정합하나, **미주(해설) 영역은 줄간격이 좁다**(조사 확인).
미주 경로의 줄간격 계산을 일반 문단 규칙과 정합시킨다. 미주 한정 조건 게이트로 일반 문단·
각주 회귀를 차단한다.

## 단계 구성 (3단계)

### Stage 1 — 원인 특정 + 회귀 여부 (코드 무변경, 조사 보고서)

**목표**: 미주 줄간격이 좁아지는 정확한 코드 위치와 일반 문단과의 차이를 규명.

- 미주 라인 레이아웃 경로 추적: `paragraph_layout.rs` / `picture_footnote.rs` /
  `typeset.rs` 중 미주 문단의 lh/ls(line spacing) 계산 지점 특정.
- 일반 문단 vs 미주 문단의 줄간격 계산 차이를 코드/`dump-pages` 로 대조
  (일반: `lh=14 ls=6` → line=20; 미주: 동일 ParaShape 인데 ls 누락/축소 여부 확인).
- **회귀 여부 bisect**: `git bisect` 로 미주 줄간격이 좁아진 커밋(있으면) 특정. 장기 버그면 명시.
- **HWPX 파싱 점검**: 동일 문서 HWP 판과 `ir-diff` 로 ParaShape line spacing 파싱 차이 확인
  (HWPX 미주 ParaShape 가 잘못 파싱되는지).
- **산출**: `tech/endnote_line_spacing_1236.md` (원인 위치 + 일반/미주 차이 + 회귀 결과).
- **승인 게이트**: 원인 확정 후 수정 단계 진행.

### Stage 2 — 미주 줄간격 정합 수정

**목표**: 미주 줄간격을 일반 문단 규칙(ParaShape line spacing)으로 계산.

- Stage 1 에서 특정한 지점 수정 — 미주 문단도 lh+ls 누적이 일반 문단과 동일하게 적용.
- **미주 한정 조건 게이트**: 일반 문단·각주에 영향 없도록 미주 경로에만 적용
  (메모리 `tech_lazy_base_trailing_ls_gate` 교훈 — 무조건 적용/제거는 양방향 회귀).
- **검증**: 12쪽 문19 미주 줄간격 SVG ↔ PDF 1차 정합 확인.

### Stage 3 — 시각·수치 검증 + 회귀 + 문서화

**목표**: 전 지적 지점 정합 + 회귀 없음 증빙.

- **시각 정합**: 지적 페이지(10·11·12·14쪽) 미주 줄간격 ↔ PDF 대조(문8·문11·문15·문19·문22·문24).
- **수치 정합**: `dump-pages` 로 미주 문단 vpos 델타가 일반 문단 규칙과 동일.
- **회귀**: 문제 페이지(1~9쪽) 줄간격 불변(전후 `dump-pages`/SVG 동일) + 전체 `cargo test --lib`.
- **다문서**: 다른 미주 포함 문서 1~2종 추가 확인(있으면).
- **산출**: `report/task_m100_1236_report.md`.

## 단계별 커밋 정책

각 Stage 완료 시 소스 + `working/task_m100_1236_stage{N}.md` 동반 커밋. 무관 rustfmt diff 금지.

## 검증 도구 요약

| 항목 | 명령 |
|------|------|
| 시각 대조 | `rhwp export-svg samples/3-11월_실전_통합_2022.hwpx -p {9,10,11,13}` ↔ `pdf/…2022.pdf` |
| 수치 줄간격 | `rhwp dump-pages … -p N` (미주 vpos 델타) |
| 회귀 bisect | `git bisect` (미주 줄간격 기준) |
| 파싱 점검 | `rhwp ir-diff sample.hwpx sample.hwp` (ParaShape line spacing) |
| 회귀 | `cargo test --lib` |
