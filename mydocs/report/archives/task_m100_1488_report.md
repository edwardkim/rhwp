# Task M100 #1488 최종 결과 보고서

- 이슈: #1488 [HWPX] Rowbreak 표 페이지네이션 여분 페이지/겹침
- 브랜치: `local/task_m100_1488`
- 작성일: 2026-06-25
- 상태: 핵심 결함 해소, 검증 완료

## 1. 문제

`samples/rowbreak-problem-pages.hwpx` 가 RowBreak 표 분할에서 여분의 거의 빈 연속
페이지와 본문/도식 겹침을 발생시켰다. clean devel(`4538a02c`) 기준 **22페이지**
(정답지 한글 2024 PDF는 **18페이지**), 이슈 제기 시점(`e678104`)에는 24페이지.

## 2. 근본 원인

섹션 1·문단 28의 `1×1 RowBreak 표`: 단일 셀에 본문 텍스트(p[0..5]) + **빈 오버레이
스페이서 문단(p[6..15])** + TAC 사각형 다이어그램(p[16..17])이 같은 세로 영역에 겹쳐
배치된 구조. 셀 높이가 페이지를 초과해 분할이 필요했다.

`cell_units`(`src/renderer/layout/table_layout.rs`)가 빈 오버레이 문단의 vpos 리셋
(동일/역방향 vpos)을 `hard_break_before` 로 표시 → `advance_row_cut` 가 가용 예산과
무관하게 리셋마다 컷을 종료. 그 결과 954px 가용 페이지에 32~85px 만 배치하고 페이지를
넘기는 거의 빈 연속 페이지를 5장 양산했다.

## 3. 해결

**비가시(빈 텍스트) 오버레이 문단이 만든 vpos 리셋을 하드 브레이크에서 제외**한다.
가시 텍스트 문단 사이 리셋(Task #993 의도)은 그대로 보존한다.

- `cell_units`: `para_has_visible_text` 게이트(`c > U+001F && c != U+FFFC`) 추가.
  - 텍스트줄 유닛: `line_reset_before(li) && para_has_visible_text`
  - 빈/원자 유닛: `reset_before && (has_table_in_para || para_has_visible_text)`
- 빈 오버레이 문단(`internal_reset`)과 #1488 의 `p[6..15]` 는 구조적으로 동일하므로,
  가시성(텍스트 유무)을 유일한 판별 신호로 채택.

## 4. 결과

| 항목 | before | after |
|------|--------|-------|
| 페이지 수 | 22 | **18** (정답지 PDF 일치) |
| pi=28 표 분할 | 8페이지(빈 페이지 5장) | 3페이지(15~17) |
| fragment 소비 | 85/33/62/62/62px | 186/945/261px |
| 2p 본문·도식 겹침 | 있음 | 해소 |
| 내용 손실 | — | 없음(18p 전수 대조) |

## 5. 검증

- 전체 `cargo test`: lib 1938 + 통합 전부 통과, 0 실패.
- `hwpx_roundtrip_baseline`: 4/4 통과.
- 단위 회귀: `test_advance_row_cut_empty_overlay_reset_no_hard_break` (+기존 2개 가시
  문단으로 갱신, rewind-orphan/Task #993 커버리지 보존).
- 통합 회귀: `tests/issue_1488_rowbreak_empty_overlay_pages.rs` (18p + pi=28 ≤4페이지).
- 시각: 한글 2024 PDF 18페이지 컨택트시트 대조 — 구조 일치, 결함 없음.

## 6. 잔여 사항 (ROOT B — 별도 후속 권장)

표 하단 분할 `LAYOUT_OVERFLOW`(sec0 p2/p3/p6, sec1 p1/p2 등, 2.7~76px 마진 스필)는:

- **모두 기존(clean devel) 문제** (이슈 본문 로그와 동일). 본 수정이 sec1 p14
  para28 13.5px overflow 를 해소했고, 신규 overflow 는 미발생.
- **내용 손실 없음** — 초과 fragment 의 나머지는 다음 페이지로 정상 연속.
- 근본은 한컴의 페이지 하단 "표 밀기 vs 부분 배치" 패리티(`typeset.rs`
  Task #1025/#1086/#1486/#1105 계보)로, 단일 수정이 다수 샘플 회귀를 유발할 수 있는
  고위험 영역. 본 이슈 핵심(여분 빈 페이지)과 분리하여 **별도 후속 이슈**로 등록 권장.

## 7. 변경 파일

- `src/renderer/layout/table_layout.rs` (cell_units 게이트 + 단위 테스트 갱신/추가)
- `tests/issue_1488_rowbreak_empty_overlay_pages.rs` (신규 통합 회귀 테스트)
- `mydocs/plans/task_m100_1488*.md`, `mydocs/working/task_m100_1488_stage{1,2,3}.md`,
  본 보고서
