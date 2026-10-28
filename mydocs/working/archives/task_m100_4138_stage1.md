---
kind: working
status: active
last_verified: 2026-08-07
---

# Task #4138 Stage 1 — 셀 나누기 후 stale line_segs·vpos 사다리 붕괴 수정

Issue: [#4138](https://github.com/edwardkim/rhwp/issues/4138) (외부 리포트: glyph truncation)
브랜치: `fix/issue-4138-split-cell-stale-linesegs` (기준: `upstream/devel` 9f564bbee)

## 증상과 원인 (2단)

재현: `samples/issue1949_giant_cell_nested_tables_perf.hwp` 에서 셀 나누기(2행 셀을 1×2 분할).

1. **glyph truncation** — `split_table_cell_into_native`(src/document_core/commands/table_ops.rs)가
   `Table::split_cell_into`(src/model/table.rs)로 셀 폭을 44790→22395 로 바꾸면서 저장
   line_segs 재계산을 생략한다. 렌더러(`LayoutEngine::layout_paragraph`,
   src/renderer/paragraph_layout.rs)는 저장 줄을 그대로 그리므로 옛 폭(seg 폭 44508) 기준
   줄이 새 셀 클립 경계에서 잘린다. 실측: 분할 뒤 4,321 seg 전부 stale, 새 sibling 셀도
   원본 line_segs 를 클론해 같은 폭을 물려받음.
2. **페이지 과소 적재** — 1에 per-para reflow 만 배선하면 `reflow_line_segs` 가 각 문단의
   vpos 원점을 보존한 채 내부 줄만 다시 싸므로(4,321→7,484 seg), 줄 수가 늘어난 문단 뒤에서
   사다리가 역행한다(실측: para[5] 끝 22920 뒤 para[6] 시작 17160). 컷 기계는 저장 vpos
   역행을 RowBreak hard break 신호로 읽어 페이지를 과소 적재한다(118→222쪽; 한컴 오라클
   기대 이미지는 1.1.1 뒤에 1.1.2 가 같은 쪽에 이어짐).

## 수정

- `reflow_stale_cells_after_split`(table_ops.rs, 신규): 저장 seg 폭이 셀 폭을 넘는
  셀(= 옛 폭 기준 stale — seg 폭은 항상 패딩만큼 셀 폭보다 작게 계산되므로 초과는 확정
  증거)을 골라 전 문단을 `reflow_cell_paragraph` 로 재래핑한 뒤, 셀 단위로 vpos 사다리를
  단조 재구축한다. `resize_table_cells_native` 의 reflow 계약을 분할 3형제 전부에 적용:
  - `split_table_cell_native` (병합 셀 복원 분할)
  - `split_table_cell_into_native` (N×M 분할)
  - `split_table_cells_in_range_native` (범위 분할)
- `rebuild_table_cell_vpos_ladder_native`(text_editing.rs, 신규): 셀 문단 전 구간을 정지
  없이 재배치한다. `recalculate_cell_paragraph_vpos` 는 저장 vpos 역행을 RowBreak 조각
  경계 신호로 존중해 그 앞에서 멈추지만, 셀 폭이 바뀌어 모든 문단을 재래핑한 직후에는
  저장 경계 자체가 옛 폭 기준이라 신호가 아니다. 간격 계산(문단 간격 boundary_gaps +
  줄간격)은 기존 적용 루프를 `apply_cell_vpos_ladder` 로 분리해 텍스트 편집 경로와
  공유한다 — 초기 실험의 ad-hoc 누적 커서(간격 무시)를 대체.

## undo 대칭 (남은 항목 c — 해소)

셀 나누기는 studio 에서 스냅샷 undo 로 라우팅된다. 증거 체인:
`table:cell-split`(rhwp-studio/src/command/commands/table.ts, `kind: 'snapshot'`) →
`SnapshotCommand`(rhwp-studio/src/engine/command.ts) → `save_snapshot_native` /
`restore_snapshot_native`(src/document_core/commands/document.rs)가 `Document` 전체를
클론·복원한다. line_segs 는 `Document` 안에 있으므로 undo 시 구 line_segs 가 자동
복원된다. 코어 측 추가 작업 불요.

## 실측 (native release, 동일 표본 1×2 분할)

| 구성 | stale | 사다리 역행 | 쪽수 |
| --- | --- | --- | --- |
| 수정 전 | 2,498문단 / 4,321seg | — | 118 |
| reflow만 | 0 | 발생 | 222 |
| reflow + 사다리 재구축(본 수정) | 0 | 0 | 195 |

- 195쪽이 한컴 실측 쪽수와 일치하는지는 **미확인** (남은 항목 — 오라클 필요).
- 회귀 테스트: `tests/issue_4138_split_cell_stale_linesegs.rs` 2건 green,
  수정 원복 시 stale 2,498 로 red 확인(red→green).

## 성능 계측 (남은 항목 d — 원인 규명, 본 수정 밖으로 판정)

`RHWP_2424_PROFILE=1` + 임시 프로브 실측 (native release):

| 단계 | 시간 |
| --- | --- |
| 파싱 + 초기 페이지네이션(115쪽) | 1.12s (`typeset_ms=1095`) |
| 분할 호출 전체 | 3.43s |
| ├ 분할 후 재조판(195쪽) | 3.33s (`typeset_ms=3308`, `RHWP_2424_BLOCK_TABLE_PROFILE`) |
| └ reflow + 사다리 재구축 + recompose (**본 수정이 추가한 비용**) | **≈0.10s** |

과제 설명의 "3.97s 성능 주의"는 본 수정의 reflow 가 아니라 **분할 뒤 195쪽 거대 표
전체 재조판**(`typeset_block_table_inner`, 쪽수 비례 ~17ms/쪽)이다. 이는 Task #8
(거대 셀 fragment 단위 증분 재조판)의 대상과 동일한 병목이므로 이 과제에서 더
최적화하지 않는다.

## 오라클 대조 (2026-08-07, issue #4138 첨부 이미지)

이슈의 한컴 기대 이미지(분할 후)와 본 수정 후 렌더(0쪽)를 대조했다:

- 기대 이미지: 분할된 왼쪽 셀에서 1.1.1·1.1.2 본문이 좁은 폭으로 완전히 재래핑되고
  1.1.2 가 1.1.1 과 같은 쪽에 이어진다. 실제(결함) 이미지: 1.1.2 줄이 옛 폭으로
  그려져 셀 경계에서 잘린다("치수는 ㅈ", "해양수ㅅ").
- 본 수정 후 렌더는 1.1.2 문단의 줄바꿈 위치가 기대 이미지와 줄 단위로 일치한다
  ("각 부재 치수는 / 직접강도 계산에 따라 결정하며, / 계산에 사용되는 자료와 그 결과
  / 를 해양수산부장관에 제출하여야 …"). 1.1.1 은 1개 줄바꿈 위치가 근소하게 다르다
  (기대 "모두 만족/하여야" vs 렌더 "모두 만/족하여야") — 최종 판정은 거버넌스에 따라
  작업지시자 권위.
- 첨부 이미지는 중첩 표 영역을 포함하지 않아 (a)의 중첩 표 리스케일 여부는 이
  오라클로 판정 불가 — 계속 미해결.

## 시각 증적 (mydocs/pr/assets/)

`HwpDocument::render_page_svg_native` 로 생성 (0–1쪽), qlmanage 로 PNG 변환:

- `pr_4138_presplit_p0.{svg,png}` — 분할 전 원본
- `pr_4138_before_p{0,1}.{svg,png}` — 분할 후, 수정 원복(HEAD~1 소스) — 잘림 재현
- `pr_4138_after_p{0,1}.{svg,png}` — 분할 후, 본 수정 — 기대 이미지와 정합

## 남은 항목

- (a) 중첩 표 host 문단의 잔존 stale seg: `reflow_line_segs` 는 텍스트 없이 컨트롤만
  호스팅하는 문단에서 원본 seg 폭 template 을 보존한다. 한컴이 분할 시 중첩 표를
  리스케일하는지 오라클 확인 필요(이슈 첨부 이미지 범위 밖). 회귀 테스트는 이 문단들을
  계약 밖으로 명시 제외.
- (f) PR #4122(#4069 canonical cell unit·frame 경계 reset 해석) 2026-08-07 **merge 됨**
  (`accebdb20`) — 본 브랜치를 merge 후 devel 로 rebase(cherry-pick `148bff3ef`)하고
  회귀 테스트·게이트를 재실행해 재검증 (검증 기록 참조).
- 후속 후보: `merge_table_cells` / 폭이 **넓어지는** 방향(병합)은 seg 폭 < 셀 폭이라
  본 판별(초과 검사)에 걸리지 않고, 병합 경로에도 reflow 배선이 없다 — 줄이 좁게 남는
  cosmetic 결함. 별도 이슈로 분리 검토.
- 한컴 실측 쪽수(분할 후 전체 문서) 대조는 한컴 편집기 환경 필요 — 작업지시자 판정
  대상으로 이관.

## 검증 기록

- `cargo test --release --test issue_4138_split_cell_stale_linesegs` — 2 passed (4.7s)
- red 확인: table_ops.rs 수정만 stash 후 동일 테스트 → stale 2,498 로 FAILED, pop 복원
- 인접 focused (release, 전부 green — 20 tests):
  issue_1073_nested_table_split(3) · issue_1750_split_guard_spacing_before(1) ·
  issue_2158_hwpx_vpos_reset_preserve(2) · issue_2299_edit_vpos_reset_preserve(8) ·
  issue_3236_split_single_cell_table(1) · issue_3593_cell_para_vpos_anchor(2) ·
  issue_3595_nested_split_row_identity(2) · issue_3637_split_cell_nested_table_vpos(1)
  — 2158/2299/3593 은 이번에 분리한 `apply_cell_vpos_ladder` 경유 reset-preserve
  의미론을 직접 핀하는 스위트.
- `cargo fmt --check`: 본 과제 변경 파일 clean (잔여 diff 는 타 세션의 임시 프로브
  `tests/tmp_bold_probe.rs` 뿐 — 본 과제 밖).
- 전체 게이트 (2026-08-07, 작업지시자 완주 지시로 실행 — merge 된 #4122 포함
  devel 기준 rebase 커밋, 격리 worktree): fmt PASS · clippy(-D warnings) PASS ·
  release-test `--tests` PASS(**5,350 passed / 0 failed**, 473 targets) ·
  native-skia 3종 PASS · wasm-pack build PASS.
- 시각 증적: `render_page_svg_native` 로 네이티브 확보 (위 "시각 증적" 절) —
  studio 브라우저 경로 불요했음. studio wasm 도 재빌드해 실기 확인 경로 제공.
