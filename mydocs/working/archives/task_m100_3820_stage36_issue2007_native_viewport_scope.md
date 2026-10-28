---
kind: investigation
status: in_progress
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 36 — issue2007 native HWP continuation viewport 범위 복구

## 출발점

Stage 35의 전체 `cargo test --profile release-test --tests`는
`issue_2007_nested_cell_pagination` 9개 중 4개 실패에서 멈췄다. 이 Stage에서는
회귀 테스트의 기대값이 틀렸을 가능성도 함께 열어 두고, 독립된 한컴 PDF를 먼저 다시
대조했다.

- `samples/basic/issue2007_nested_cell_pagination_42065.hwp`가 Hancom 2020 PDF의 17쪽 대신
  18쪽으로 과분할된다.
- PDF p11의 continuation 첫 줄은 physical nested-cell clip 안에 있어야 하나 현재 tree에서는
  clip보다 위(`y=115.8`)에 남는다.
- PDF p12/p15의 source owner가 한 physical fragment 뒤로 밀린다.

독립 기준은 이미 보관된
`pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`다. 새 HWP 2020 MCP 변환은 이
기준을 대체하거나 더 정확하게 만들지 않으므로 생성하지 않는다. 기준 PDF p16은
`2) 대안선택 및 근거`에서 끝나며 `3) 선호된 대안의 기대효과`는 p17에만 있다. 따라서
기존 회귀의 “p16에는 없고 p17에는 있어야 한다”는 계약은 정확하다.

## 이전 정상 상태와 차이

Stage 29 기록은 같은 native HWP와 PDF p10–p17에서 9개 focused regression green을 확인했다.
그 뒤 #3637 HWPX 보정을 넣은 `0ec8a14eb`은 `NestedTableSplit::content_offset`과
`fragment_cut_units`를 도입했다.

`fragment_cut_units`는 `single_row_fragment && row_filter.is_some() && !split_terminal`만
확인해 HWPX 전용 source-unit viewport를 **native HWP에도** 적용한다. 동시에 normal cell
layout이 그 `NestedTableSplit`을 child table로 전달한다. native HWP RowBreak wrapper에는 이미
바깥 Cell의 물리 clip과 누적 vpos가 있으므로, HWPX unit viewport까지 중복 전달되면 child
viewport가 한 unit 앞선다. 그 결과 p16에 p17 제목이 미리 paint되고 이후 source owner도
밀린다.

반면 #3637의 필요 조건은 `hwpx_stored_layout()`이며, Stage 35에서 이 HWPX 축은 PDF p26→p27 및
issue1949의 115쪽 HWP 2020/2024 오라클로 별도 확인됐다.

## 수정 가설

`fragment_cut_units`와 normal cell layout의 child `NestedTableSplit` 전달을
`hwpx_stored_layout()`로 한정한다. 또한 native HWP RowBreak 1×1 wrapper는 offset=0인 첫
조각도 바깥 Cell을 viewport로 쓰므로, descendant y를 그 조각 하단에 clamp하지 않는다.

- HWPX: #3637 source-unit owner 보정을 유지한다.
- native HWP: Stage 29에서 검증된 기존 row-filter/physical-cell-clip 경로를 그대로 쓴다.

## 판정

1. `issue_2007_nested_cell_pagination` 9개가 17쪽과 p10–p17 owner/clip 계약을 회복한다.
2. #3637 HWPX focused regression과 issue1949 #2214/#2424 deferred contract가 유지된다.
3. `fidelity_compare.py` direct pair로 issue2007 p10–p17을 다시 대조해 PDF physical page
   ownership 후보를 확인한다. 이 macOS SVG raster는 문서가 요구하는 일부 logical font family를
   Chrome이 두부로 fallback하므로 pixel diff 수치만으로 fidelity 합격을 선언하지 않는다. 대신
   동일 실행의 PDF text owner 후보, focused render-tree 계약, 기준 PDF p16/p17의 실제 내용을
   교차한다.
4. 새 결함이 남으면 이 stage는 분석·증적만 커밋하고 다음 stage에서 별도 수정한다.

## 결과

- `issue_2007_nested_cell_pagination`: 9/9 통과. 페이지 수 17, p10–p17의 owner/clip 계약과
  p16→p17 제목 경계가 모두 회복됐다.
- `issue_3637_nested_table_starts_inside_parent_cell` 및
  `issue_3637_split_cell_nested_table_vpos`: HWPX nested-table viewport 회귀 없이 통과했다.
- `issue_2214_page_local_repaint`: HWP/HWPX deferred pagination cache 계약 3/3 통과.
- `issue_2424_pagination_subphase_probe --ignored`: HWP/HWPX 모두 115 fragment step과 115쪽을
  유지했다.
- direct pair `fidelity_compare.py` p10–p17은 완료됐고, `text-owner-shift-candidates.tsv`와
  `text-owner-sequence-candidates.tsv`에는 후보가 없다. SVG glyph font fallback 때문에
  `visible-text-excess-candidates.tsv`는 후보로만 남으며, PDF 대조를 대체하지 않는다.

이 수정은 회귀 기준을 완화하지 않는다. 한컴 2020 PDF가 확인한 native HWP의 물리 Cell 계약을
복원하고, HWPX 전용 source-unit viewport는 HWPX 경로에 그대로 보존한다.
