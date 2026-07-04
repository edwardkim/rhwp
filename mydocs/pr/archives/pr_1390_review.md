# PR #1390 검토 — 미주 빈수식 spacer 발산 진단·종결 기록 (#1377 문서)

## 1. PR 개요

- PR: https://github.com/edwardkim/rhwp/pull/1390
- 작성자: `planet6897` (Jaeuk Ryu) — 미주(endnote)·수식 영역 핵심 컨트리뷰터 (누적 30+ PR)
- 상태: open / 라벨 `documentation`
- base: `devel` ✓ (head: `task1377-docs`, 본질 커밋 `44a58cbf` + devel 동기화 merge 6개)
- 연결 이슈: #1377 (**이미 CLOSED** — 2026-06-12)
- 변경: **12파일 +682/-0, 전부 `mydocs/` 문서. 코드 변경 0.**

## 2. 변경 요약

#1377(미주 단 render↔typeset 발산 ~15px 제거 시도) 조사·**종결**의 계획서·단계별·최종
보고서 일괄 반영. #1377 은 이미 close 됐으나 그 진단 문서가 devel 에 없었다 → 본 PR 이 채움.

- `plans/`: task_m100_1377.md, _v2.md, _v2_impl.md
- `working/`: _stage2~5, _v2_stage1~3
- `report/`: _report.md, _v2_report.md

핵심 결론(문서):
- 발산 근원 = sep2020 pi=1128 빈 TAC-수식 spacer 가 between-notes 마커(1984HU)를 trailing
  line_spacing 으로 가산 → render 54.1px vs typeset 33.6px, 이후 +20px offset 상속.
- sep2020 은 **compact 가 PDF 정답**(픽셀·SVG bbox·문단내 3중 입증, 한글 2022 PDF p22).
- 그러나 compact-정답(sep2020)과 gap-정답(미주사이20·2022/2023 8건)을 가를 신뢰 신호가
  parsed FootnoteShape·line_seg·saved-vpos 어디에도 없음 → `tech_trailing_model_no_ssot`
  의 #1246 입력 모호성으로 **종결**. plumbing 시도 코드는 회귀(19→8건)로 보류, 본 PR 미포함.

## 3. 정합성 검증

- **명명·폴더 규칙**: CLAUDE.md 준수 — plans/working/report 분리, `task_m100_1377` 패턴,
  `_v2`/`_stage{N}`/`_report` 접미어 정확. 무관 변경 0.
- **devel 코드 상태 정합**: report 가 참조하는 plumbing 커밋(b9bf5ce6 등)은 fork 로컬이라
  devel 미반영(보류 결정과 일치). v2_impl 이 가리키는 코드 위치(`paragraph_layout.rs` 미주
  trailing 분기)와 헬퍼(`endnote_para_has_same_endnote_successor`, `endnote_between_notes_hu`,
  `measure_endnote_para_advance`)는 devel 에 실재 — 진단 정확. clamp 미적용(보류) 상태와 정합.
- **참조 자료 실재**: `mydocs/tech/trailing_model_render_vs_pagination_1248.md` 존재. 메모리
  `tech_trailing_model_no_ssot`(#1246 입력 모호성) 정합.
- **merge 정합**: BEHIND 이나 PR 에 devel 동기화 merge 가 포함돼 GitHub 3-way merge 결과가
  깨끗(12 문서 +682, 비문서·무관 변경 0, 충돌 없음). 로컬 시뮬레이션으로 확인.
- 코드 변경 0 → 빌드/테스트 영향 없음.

## 4. 평가

- 진단 충실·정확(근원 국소화 + PDF 3중 입증 + 신호 부재 종결 논리). devel 현실 정확 반영.
- 문서 규칙 완벽 준수, 무관 변경 0, merge 깨끗.
- CLOSED 타스크의 종결 기록 보존 — 후속 타스크(파서 raw_unknown 의미 변형·note-boundary
  그룹핑) 출발점으로 가치.

## 5. 판단

**merge 권고**. 순수 문서, 규칙 준수, devel 정합, merge 깨끗. base 가 `devel` 이고 동기화
merge 가 들어 있어 GitHub UI merge 또는 cherry-pick 둘 다 깨끗. 세부는 `pr_1390_report.md`.
