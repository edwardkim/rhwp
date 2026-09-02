# Table probe axis matrix

| case | sample | outer_margin | common_attr | table_attr | table_tail | next probe |
|---|---|---:|---:|---:|---:|---|
| `T01` | `hwpx-h-01-outer-margin` | 1 | 0 | 0 | 0 | 01_ctrl_outer_margin_only |
| `T02` | `hwpx-h-01-table-attr` | 0 | 0 | 1 | 0 | 02_table_attr_only |
| `T03` | `hwpx-h-01-table-tail` | 0 | 0 | 0 | 1 | 03_table_tail_only |
| `T04` | `hwpx-h-01-common-attr` | 0 | 1 | 0 | 0 | 04_ctrl_common_attr_only |
| `T05` | `hwpx-h-01-missing-list-header` | 0 | 0 | 0 | 0 | oracle LIST_HEADER graft |
| `T06` | `hwpx-h-02-row-count` | 0 | 0 | 0 | 0 | row/col count 필드 좁히기 |
| `T07` | `hwpx-h-02-extra-cell` | 0 | 0 | 0 | 0 | extra PARA_HEADER 제거 |
| `T08` | `hwpx-h-01-all-table-axes` | 1 | 1 | 1 | 1 | 08_all_table_axes |
| `T09` | `hwpx-h-03-nested-table` | 0 | 0 | 0 | 0 | 중첩 표 LIST_HEADER 범위 |
| `T10` | `hwpx-h-header-table` | 0 | 0 | 0 | 0 | 머리말 안 표는 Header LIST_HEADER 범위 안 |
| `S01` | `hwpx-h-01-bindata-missing` | 0 | 0 | 0 | 0 | BIN_DATA + SHAPE_PICTURE 동시 대조 |
| `S02` | `hwpx-h-03-shape-matrix-f32` | 0 | 0 | 0 | 0 | f32→f64 양자화 payload 대조 |
| `S03` | `hwpx-h-03-missing-shape-component` | 0 | 0 | 0 | 0 | CTRL_HEADER(GenShape) 다음 SHAPE_COMPONENT graft |
| `S04` | `hwpx-h-03-missing-ctrl-data` | 0 | 0 | 0 | 0 | hwp5-ctrl-data-trace |
| `S05` | `hwpx-h-02-picture-in-cell` | 0 | 0 | 0 | 0 | 셀 안 그림은 TABLE 튜플 + BIN_DATA 동시 |
| `S06` | `hwpx-ole-shape` | 0 | 0 | 0 | 0 | SHAPE_OLE + BIN_DATA |
| `S07` | `hwpx-container-group` | 0 | 0 | 0 | 0 | 컨테이너 자식 개수 |
| `P01` | `hwpx-lineseg-copy` | 0 | 0 | 0 | 0 | oracle PARA_LINE_SEG 만 정답 |
| `P02` | `hwpx-para-char-count` | 0 | 0 | 0 | 0 | char_count == PARA_TEXT code units |
| `P03` | `hwpx-missing-char-shape` | 0 | 0 | 0 | 0 | PARA_CHAR_SHAPE graft |
| `P04` | `hwpx-extra-range-tag` | 0 | 0 | 0 | 0 | 과잉 RANGE_TAG 제거 |
| `P05` | `hwpx-lineseg-tac-mix` | 0 | 0 | 1 | 0 | 혼합 문단은 oracle lineSeg 우선 |
| `D01` | `hwpx-section-count` | 0 | 0 | 0 | 0 | section_count == BodyText section streams |
| `D02` | `hwpx-id-mappings-count` | 0 | 0 | 0 | 0 | ID_MAPPINGS count == 자식 record 수 |
| `D03` | `hwpx-missing-face-name` | 0 | 0 | 0 | 0 | FACE_NAME graft |
| `D04` | `hwpx-extra-char-shape` | 0 | 0 | 0 | 0 | 과잉 CHAR_SHAPE 제거 또는 매핑 수정 |
| `D05` | `hwpx-compatible-document` | 0 | 0 | 0 | 0 | 호환 문서 표지 보존 |
| `D06` | `hwpx-distribute-doc` | 0 | 0 | 0 | 0 | 배포용 문서 플래그와 데이터 동시 |
| `D07` | `hwpx-numbering-bullet` | 0 | 0 | 0 | 0 | 번호/글머리표 표 개수 |
| `D08` | `hwpx-parashape-valign` | 0 | 0 | 0 | 0 | 셀 클리핑이면 ParaShape valign bits |
| `C01` | `hwpx-equation-missing-eqedit` | 0 | 0 | 0 | 0 | CTRL_HEADER(Equation)+EQEDIT |
| `C02` | `hwpx-footnote-list-count` | 0 | 0 | 0 | 0 | 각주 문단 수 |
| `C03` | `hwpx-endnote-missing` | 0 | 0 | 0 | 0 | 미주 컨트롤 graft |
| `C04` | `hwpx-header-footer` | 0 | 0 | 0 | 0 | 머리말/꼬리말 목록 범위 |
| `C05` | `hwpx-form-object` | 0 | 0 | 0 | 0 | FORM_OBJECT graft |
| `C06` | `hwpx-memo-shape` | 0 | 0 | 0 | 0 | 메모 모양은 DocInfo, 목록은 BodyText |
| `C07` | `hwpx-chart-data-extra` | 0 | 0 | 0 | 0 | oracle 에 없는 CHART_DATA 제거 |
| `C08` | `hwpx-bookmark` | 0 | 0 | 0 | 0 | 책갈피 컨트롤 보존 |
| `F01` | `hwpx-field-clickhere` | 0 | 0 | 0 | 0 | 누름틀 %clk 정체성 보존 |
| `F02` | `hwpx-field-hyperlink` | 0 | 0 | 0 | 0 | 하이퍼링크 %hlk |
| `F03` | `hwpx-field-mailmerge` | 0 | 0 | 0 | 0 | 메일머지 %mmg |
| `F04` | `hwpx-field-proofreading` | 0 | 0 | 0 | 0 | 교정부호는 %%*d / command $RevisionDelete |
| `G01` | `hwpx-columndef-defaults` | 0 | 0 | 0 | 0 | 다단 기본값 합성 |
| `G02` | `hwpx-pagedef-missing` | 0 | 0 | 0 | 0 | PAGE_DEF graft |
| `G03` | `hwpx-page-border-fill` | 0 | 0 | 0 | 0 | 쪽 테두리 기본값 |
| `G04` | `hwpx-pagenum-pos` | 0 | 0 | 0 | 0 | 쪽번호 위치 기본값 |
| `G05` | `hwpx-pagehide` | 0 | 0 | 0 | 0 | 감추기 비트 보존 |
| `X01` | `hwpx-index-vs-lcs-insert` | 0 | 0 | 0 | 0 | 중간 삽입은 lcs 우선 |
| `X02` | `hwpx-scope-changed` | 0 | 0 | 0 | 0 | scope_path 재배치 |
| `X03` | `hwpx-control-remapped` | 0 | 0 | 0 | 0 | 컨트롤 ID 보존 |
| `X04` | `hwpx-tag-changed` | 0 | 0 | 0 | 0 | tag_changed 는 트리 재작성 신호 |
| `X05` | `hwpx-trackchange-extra` | 0 | 0 | 0 | 0 | 변경 추적 메타 과잉 삽입 금지 |
| `X06` | `hwpx-forbidden-char` | 0 | 0 | 0 | 0 | 금칙 문자 기본값 |
| `X07` | `hwpx-tab-def` | 0 | 0 | 0 | 0 | 탭 정의 표 |
| `X08` | `hwpx-doc-data` | 0 | 0 | 0 | 0 | 문서 부가 데이터 보존 |
| `X09` | `hwpx-autonumber` | 0 | 0 | 0 | 0 | 자동번호 기본 속성 |
| `X10` | `hwpx-newnumber` | 0 | 0 | 0 | 0 | 새 번호 시작값 |
| `X11` | `hwpx-index-mark` | 0 | 0 | 0 | 0 | 찾아보기 표식 보존 |
| `X12` | `hwpx-hidden-comment` | 0 | 0 | 0 | 0 | 숨은 설명 컨트롤 |
| `X13` | `hwpx-char-overlap` | 0 | 0 | 0 | 0 | 덧말 컨트롤 |
| `X14` | `hwpx-tcps` | 0 | 0 | 0 | 0 | 글자겹침 컨트롤 |
| `X15` | `hwpx-textbox-list-header` | 0 | 0 | 0 | 0 | 글상자 목록 범위 |
| `X16` | `hwpx-identical-roundtrip` | 0 | 0 | 0 | 0 | 차이 없음 — 다음 샘플로 |
