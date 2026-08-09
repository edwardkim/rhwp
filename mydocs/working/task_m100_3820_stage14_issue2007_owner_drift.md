---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage13.md
last_verified: 2026-08-05
---

# Task #3820 Stage 14 — issue2007 p10–p16 nested continuation owner drift

## 재개 사유

Stage 13의 `p10–p16` TextLine overlap 0은 수용 기준으로 충분하지 않았다. 최신 WASM build의 p10은
중첩 RowBreak table 안 본문을 페이지 하단으로 밀어 큰 빈 영역을 만들며, 현재 native release-test SVG도
동일하다. 따라서 browser/WASM wrapper 문제가 아니라 공통 Rust layout의 continuation owner/offset 결함이다.

## 최초 재현

- HWP: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준 PDF: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- native: `target/task-3820-3821-fidelity/release-test/rhwp`
- p10: 0-based page index 9, outer source anchor `pi=7, ci=1`

수정 전 native render tree에서 해당 outer table의 bbox는 `y=113.4, h=918.1`이고 첫 visible 본문
`독점규제 및 공정거래에 관한 법률`은 y=458.9에서 시작한다. 같은 기준 PDF의 첫 본문은 표 상단 바로 뒤
y≈201에서 시작한다. p10에서만 축소·잘림이 아니라 p10–p16 continuation 모두의 owner가 잘못된 상태로
판정한다.

`c42d177ee`는 중간 `LINE_SEG vpos=0` reset을 cell top anchor로 재사용해 본문이 서로 겹치는 결함을
막았으나, 동시에 1×1 nested continuation의 `offset_within_start`를 전체 누적 `offset`으로 바꿨다. 이 값은
현재 조각의 물리 origin이 아니라 이전 쪽에서 소비한 전체 콘텐츠 위치다. page 10 이후 이를 화면 origin에
그대로 쓰면 본문이 하단으로 밀린다. 반대로 p8에는 직전 첫 visible unit의 재paint를 막아야 하는 별도
계약이 있어, 단순히 이전 식을 전역 복원하지 않는다.

## 수정·검증 계약

1. p8의 직전 쪽 마지막 줄 재paint 방지와 p10–p16의 정상적인 top continuation을 별도의 조건으로
   계측·보정한다.
2. p10–p16의 first visible body line과 기준 PDF의 table-content start를 직접 비교하고, TextLine
   overlap 0만으로 합격 처리하지 않는다.
3. native render tree 회귀와 release-test fixture를 고정하고, 사용자가 수행한 WASM build의 공통 경로가
   같은 Rust layout으로 회복되도록 한다.
4. 구현 전후 p10–p16 PDF/SVG raster와 p8·p4·p2 기존 safety 회귀를 재검증한다.

## 보정 A — 부모 continuation viewport 안의 손자 cell valign

`layout_table_cells`의 cell vertical-align 판정에 부모 viewport clip을 추가했다. 중첩(`depth > 0`)·비
자리차지 표에서 현재 cell이 전달받은 `col_area` 하단을 실제로 넘을 때만 `Center`/`Bottom`을 `Top`으로
수렴시킨다. `nested_split`을 직접 받지 않는 손자 1×1 표도 부모 RowBreak continuation 안에서 물리적으로
잘릴 수 있다는 점이 핵심이다. 최상위 표와 완전 cell에는 적용하지 않으므로 일반 수직 정렬 계약은 유지한다.

회귀 `issue_2007_continuation_viewport_does_not_center_nested_cell_content`는 p10의
`독점규제 및 공정거래에 관한 법률` 첫 TextRun이 해당 손자 표 상단에서 40px 이내에 있어야 한다고 고정한다.
수정 전에는 표 상단보다 약 250px 아래(`text_y=458.9`)였다. 이 조건은 TextLine 겹침 0만으로는 검출할 수
없는 대형 공백/owner drift를 직접 막는다.

## 보정 B — p2–p3 partial wrapper의 완성된 nested-table 우측선

보정 A 뒤 전수 direct-pair ledger에서 p2의 4×2·9×2 표와 p3 continuation 9×2 표의 우측선이 모두
wrapper Cell clip 밖에 있다는 후보를 찾았다. 기준 PDF에는 세 outer vertical stroke가 보이지만 rhwp SVG는
각각 `clip_right=729.44`, border right `733.44` 이상으로 전부 잘랐다.

기존 p4 보정은 일반 table cell loop에서 직접 자식 표의 이미 emit된 `Line`만 읽었다. 하지만 RowBreak
partial-table 경로는 edge를 fragment cell loop 뒤에 추가하므로 p2–p3에서 이 시점이 너무 이르다. 이제
완성된 table subtree를 post-order로 순회해 **직접** nested Table의 outer vertical stroke만 가로 clip에
포함한다. 아직 edge `Line`이 붙기 전인 partial path는 completed Table bbox와 1px stroke 여유를 fallback으로
쓴다. 세로 clip은 전혀 확대하지 않아, p5 이후 1×1 continuation의 다음 쪽 text tail은 계속 숨긴다.

회귀 `issue_2007_wrapper_clip_keeps_completed_nested_table_right_borders`는 p2의 두 outer stroke와 p3의
한 outer stroke가 각 wrapper clip 안에 있어야 함을 고정한다. 수정 후 p2–p4
`svg-table-border-clip-candidates.tsv`는 header만 남아 후보 0건이다.

## 보정 C — 1×1 continuation의 이미 예약된 첫 단위 재도색 방지

contact sheet만으로는 p10–p17의 각 조각이 어느 source window를 실제로 소유하는지 판정할 수 없어,
각 PDF/SVG pair를 분리해 다시 확인했다. p12는 PDF 기준으로 `중앙선거관리위원회`에서 시작해야 하지만,
기존에는 직전 조각의 `진술을 하거나 그 직무집행을 거부 또는 기피한 자`를 다시 칠한 뒤 이후 heading을
아래로 밀었다. 반대로 p16은 마지막 non-terminal 조각인데 p17 소유 heading을 미리 그릴 수 있었다.

원인은 1×1 host 안의 1×1 nested table에서 첫 visible unit이 physical flow reservation에는 이미 반영됐지만,
다음 조각 content origin과 mixed-flow extra에는 다시 남아 있던 이중 계산이다. non-terminal 1×1
continuation에서만 다음 content origin을 그 첫 unit만큼 전진시키고, 같은 reservation/extra는 제거했다.
terminal 조각은 남은 tail을 보존해야 하므로 적용 대상에서 제외했다. 따라서 p8의 재paint 방지 계약이나
일반 multi-row nested table의 row cut은 바꾸지 않는다.

회귀 `issue_2007_single_cell_continuation_does_not_repaint_boundary_fragments`는 다음을 고정한다.

- p12에는 `중앙선거관리위원회`가 있고 직전 조각의 마지막 문장은 없다.
- p16에는 p17 소유 `선호된 대안의 기대효과`가 없고, p17에만 있다.

같은 post-order clip 경로에서 table bbox가 직접 Cell의 수평 paint extent를 전달하게 하여 p10–p16의
깊은 nested table 우측 stroke도 상위 Cell/Body clip에 잘리지 않도록 했다. 이 계약은
`issue_2007_continuation_ancestor_clip_keeps_deep_right_border`로 고정한다.

## 재검증과 현재 판정

- 최신 native release-test 출력은 기준 PDF와 동일하게 **17쪽**이다. 전수 page-count ledger의 PDF/SVG/render
  tree 모두 17쪽이다.
- p2·p3 우측선, p4 우측선, p8 중복 line, p10–p16 visible-content top과 cell-local `vpos=0` reset,
  p10 continuation ancestor clip, p12/p16/p17 source-window 경계를 포함한
  `issue_2007_nested_cell_pagination` focused executable은 **7 passed, 0 failed**다.
- 17쪽 `--text-only --export-all-svg --layout-ledger` 결과는 text owner-shift 0건,
  text-owner-sequence 0건, float owner-shift 0건, TableCell TextLine overlap 0건이다.
- p5–p17의 `table_footer`/`table_outside_frame` 및 SVG border-clip 보조 후보는 1×1 RowBreak 표의 page 밖
  continuation tail이다. 이들을 표시하려고 vertical cell clip을 넓히면 p8의 replay 결함이 재발하므로,
  자동 후보로 보관하되 결함으로 승격하지 않았다.
- PDF pair를 p10–p17 각각 분리해 직접 확인했다. p10–p16은 수정 전의 대형 상단 공백·문단 겹침 없이 기준과
  같은 continuation 흐름이고 p12의 이전 source 재표시, p16의 p17 source 누출도 없다. p2·p3의 표 우측
  경계도 보인다. p10–p16 raw pixel diff는 한컴 PDF와 native 글꼴 raster/metrics 차이를 포함하므로 단독
  합격 기준으로 쓰지 않았다. 다만 p10–p17의 outer border·page owner·text order·page count는 별도
  ledger와 focused 회귀로 확인했다.

HWP fixture와 한컴 2020 PDF는 각각 `samples/basic/`와 `pdf/basic/` canonical 경로에 보관되어 있다.
생성 증적은 `mydocs/pr/assets/task_m100_3820_stage14_issue2007_continuation/`에 보관했다.

## 증적

- [p2–p4 border-clip 후보 0건](../pr/assets/task_m100_3820_stage14_issue2007_continuation/p002_p004_svg_table_border_clip_candidates_after.tsv)
- [p2 수정 후 PDF pair](../pr/assets/task_m100_3820_stage14_issue2007_continuation/review_p002_after_border.png), [p3 수정 후 PDF pair](../pr/assets/task_m100_3820_stage14_issue2007_continuation/review_p003_after_border.png), [p4 guard pair](../pr/assets/task_m100_3820_stage14_issue2007_continuation/review_p004_after_border.png)
- [p1–p9 직접 대조 sheet](../pr/assets/task_m100_3820_stage14_issue2007_continuation/review_p001_p009_contact.png), [p10–p17 직접 대조 sheet](../pr/assets/task_m100_3820_stage14_issue2007_continuation/review_p010_p017_contact.png)
- p10–p17 분리 PDF pair: [p10](../pr/assets/task_m100_3820_stage14_issue2007_continuation/review_p010_after_window.png), [p11](../pr/assets/task_m100_3820_stage14_issue2007_continuation/review_p011_after_window.png), [p12](../pr/assets/task_m100_3820_stage14_issue2007_continuation/review_p012_after_window.png), [p13](../pr/assets/task_m100_3820_stage14_issue2007_continuation/review_p013_after_window.png), [p14](../pr/assets/task_m100_3820_stage14_issue2007_continuation/review_p014_after_window.png), [p15](../pr/assets/task_m100_3820_stage14_issue2007_continuation/review_p015_after_window.png), [p16](../pr/assets/task_m100_3820_stage14_issue2007_continuation/review_p016_after_window.png), [p17](../pr/assets/task_m100_3820_stage14_issue2007_continuation/review_p017_after_window.png)
- p10–p17 [pixel report](../pr/assets/task_m100_3820_stage14_issue2007_continuation/p010_p017_pixel_report_after.tsv), [page-count ledger](../pr/assets/task_m100_3820_stage14_issue2007_continuation/p010_p017_page_count_after.tsv), [SVG border-clip 후보](../pr/assets/task_m100_3820_stage14_issue2007_continuation/p010_p017_svg_table_border_clip_candidates_after.tsv), [provenance](../pr/assets/task_m100_3820_stage14_issue2007_continuation/p010_p017_provenance_after.tsv)
- [전수 page-count ledger](../pr/assets/task_m100_3820_stage14_issue2007_continuation/all_pages_page_count_after.tsv), [전수 TextLine overlap ledger](../pr/assets/task_m100_3820_stage14_issue2007_continuation/all_pages_table_cell_text_overlap_after.tsv), [전수 border-clip 후보](../pr/assets/task_m100_3820_stage14_issue2007_continuation/all_pages_svg_table_border_clip_candidates_after.tsv), [전수 layout 후보](../pr/assets/task_m100_3820_stage14_issue2007_continuation/all_pages_layout_candidates_after.tsv)
