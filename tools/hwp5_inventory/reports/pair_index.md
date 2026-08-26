# Contract case index

| id | sample | family | class | judgment | status | index Δ | lcs Δ | construct |
|---|---|---|---|---|---|---:|---:|---|
| `T01` | `hwpx-h-01-outer-margin` | `table` | `E` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:tbl / CTRL_HEADER outer margin |
| `T02` | `hwpx-h-01-table-attr` | `table` | `E` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:tbl / TABLE.table_attr |
| `T03` | `hwpx-h-01-table-tail` | `table` | `C` | 파일 손상 | `violated` | 1 | 1 | hp:tbl / TABLE tail after 0x16 |
| `T04` | `hwpx-h-01-common-attr` | `table` | `E` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:tbl / CTRL_HEADER.common_attr |
| `T05` | `hwpx-h-01-missing-list-header` | `table` | `B` | 파일 손상 | `violated` | 5 | 5 | hp:tbl / LIST_HEADER subtree |
| `T06` | `hwpx-h-02-row-count` | `table` | `C` | 파일 손상 | `violated` | 1 | 1 | hp:tbl / TABLE.rows vs LIST_HEADER count |
| `T07` | `hwpx-h-02-extra-cell` | `table` | `B` | 파일 손상 | `violated` | 4 | 8 | hp:tbl / extra cell paragraph |
| `T08` | `hwpx-h-01-all-table-axes` | `table` | `E` | 열림 + 조판 실패 | `violated` | 2 | 2 | hp:tbl / four table-probe axes together |
| `T09` | `hwpx-h-03-nested-table` | `table` | `B` | 파일 손상 | `violated` | 1 | 1 | hp:tbl inside hp:tbl |
| `T10` | `hwpx-h-header-table` | `table` | `B` | 파일 손상 | `violated` | 1 | 1 | hp:header / hp:tbl |
| `S01` | `hwpx-h-01-bindata-missing` | `shape` | `D` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:pic / DocInfo BIN_DATA |
| `S02` | `hwpx-h-03-shape-matrix-f32` | `shape` | `C` | 파일 손상 | `violated` | 1 | 1 | hp:pic / SHAPE_COMPONENT rendering matrix |
| `S03` | `hwpx-h-03-missing-shape-component` | `shape` | `B` | 파일 손상 | `violated` | 1 | 1 | hp:pic / SHAPE_COMPONENT after GenShape |
| `S04` | `hwpx-h-03-missing-ctrl-data` | `shape` | `B` | 파일 손상 | `violated` | 1 | 1 | hp:pic / CTRL_DATA ParameterSet |
| `S05` | `hwpx-h-02-picture-in-cell` | `shape` | `D` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:tbl cell / hp:pic |
| `S06` | `hwpx-ole-shape` | `shape` | `B` | 파일 손상 | `violated` | 1 | 1 | hp:ole / SHAPE_OLE |
| `S07` | `hwpx-container-group` | `shape` | `B` | 파일 손상 | `violated` | 1 | 1 | hp:container / SHAPE_CONTAINER |
| `P01` | `hwpx-lineseg-copy` | `para` | `F` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:p / lineSegArray copied into PARA_LINE_SEG |
| `P02` | `hwpx-para-char-count` | `para` | `C` | 파일 손상 | `violated` | 4 | 4 | hp:p / PARA_HEADER.char_count |
| `P03` | `hwpx-missing-char-shape` | `para` | `B` | 파일 손상 | `violated` | 4 | 1 | hp:p / PARA_CHAR_SHAPE |
| `P04` | `hwpx-extra-range-tag` | `para` | `B` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:p / extra PARA_RANGE_TAG |
| `P05` | `hwpx-lineseg-tac-mix` | `para` | `F` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:p + TAC table mix / PARA_LINE_SEG |
| `D01` | `hwpx-section-count` | `docinfo` | `A` | 열림 + 조판 실패 | `violated` | 1 | 1 | hsz:secPr / DOCUMENT_PROPERTIES.section_count |
| `D02` | `hwpx-id-mappings-count` | `docinfo` | `C` | 파일 읽기 오류 | `violated` | 1 | 1 | DocInfo / ID_MAPPINGS counts |
| `D03` | `hwpx-missing-face-name` | `docinfo` | `D` | 파일 읽기 오류 | `violated` | 1 | 1 | DocInfo / FACE_NAME |
| `D04` | `hwpx-extra-char-shape` | `docinfo` | `D` | 열림 + 조판 실패 | `violated` | 1 | 1 | DocInfo / extra CHAR_SHAPE |
| `D05` | `hwpx-compatible-document` | `docinfo` | `A` | 파일 읽기 오류 | `violated` | 2 | 2 | DocInfo / COMPATIBLE_DOCUMENT |
| `D06` | `hwpx-distribute-doc` | `docinfo` | `A` | 파일 읽기 오류 | `violated` | 1 | 1 | DocInfo / DISTRIBUTE_DOC_DATA |
| `D07` | `hwpx-numbering-bullet` | `docinfo` | `D` | 열림 + 조판 실패 | `violated` | 2 | 2 | DocInfo / NUMBERING + BULLET |
| `D08` | `hwpx-parashape-valign` | `docinfo` | `E` | 열림 + 조판 실패 | `violated` | 1 | 1 | DocInfo / PARA_SHAPE.attr1 valign |
| `C01` | `hwpx-equation-missing-eqedit` | `equation` | `B` | 파일 손상 | `violated` | 1 | 1 | hp:equation / EQEDIT |
| `C02` | `hwpx-footnote-list-count` | `note` | `C` | 파일 손상 | `violated` | 1 | 1 | hp:fn / LIST_HEADER paragraph count |
| `C03` | `hwpx-endnote-missing` | `note` | `B` | 파일 손상 | `violated` | 1 | 1 | hp:en / Endnote control |
| `C04` | `hwpx-header-footer` | `note` | `B` | 파일 손상 | `violated` | 1 | 1 | hp:header + hp:footer |
| `C05` | `hwpx-form-object` | `form` | `B` | 파일 손상 | `violated` | 1 | 1 | hp:form / FORM_OBJECT |
| `C06` | `hwpx-memo-shape` | `note` | `B` | 파일 손상 | `violated` | 2 | 2 | hp:memo / MEMO_SHAPE + MEMO_LIST |
| `C07` | `hwpx-chart-data-extra` | `shape` | `B` | 파일 손상 | `violated` | 1 | 1 | hp:chart / extra CHART_DATA |
| `C08` | `hwpx-bookmark` | `field` | `B` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:bookmark / CTRL_HEADER bokm |
| `F01` | `hwpx-field-clickhere` | `field` | `B` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:fieldBegin type=CLICKHERE |
| `F02` | `hwpx-field-hyperlink` | `field` | `B` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:fieldBegin type=HYPERLINK |
| `F03` | `hwpx-field-mailmerge` | `field` | `B` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:fieldBegin type=MAILMERGE |
| `F04` | `hwpx-field-proofreading` | `field` | `C` | 열림 + 조판 실패 | `violated` | 3 | 2 | hp:fieldBegin type=PROOFREADING_MARKS_DELETE |
| `G01` | `hwpx-columndef-defaults` | `page` | `E` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:colPr / ColumnDef defaults |
| `G02` | `hwpx-pagedef-missing` | `page` | `B` | 파일 읽기 오류 | `violated` | 1 | 1 | hp:secPr / PAGE_DEF |
| `G03` | `hwpx-page-border-fill` | `page` | `E` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:pagePr / PAGE_BORDER_FILL defaults |
| `G04` | `hwpx-pagenum-pos` | `page` | `E` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:pageNum / PageNumPos |
| `G05` | `hwpx-pagehide` | `page` | `E` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:pageHide / PageHide |
| `X01` | `hwpx-index-vs-lcs-insert` | `para` | `B` | 파일 손상 | `violated` | 8 | 4 | 중간 삽입에서 index vs lcs |
| `X02` | `hwpx-scope-changed` | `table` | `B` | 파일 손상 | `violated` | 1 | 0 | 같은 uid / 다른 scope_path |
| `X03` | `hwpx-control-remapped` | `ctrl` | `C` | 파일 손상 | `violated` | 2 | 2 | CTRL_HEADER fourcc remapped |
| `X04` | `hwpx-tag-changed` | `para` | `B` | 파일 손상 | `violated` | 7 | 5 | 같은 uid / 다른 tag |
| `X05` | `hwpx-trackchange-extra` | `docinfo` | `B` | 열림 + 조판 실패 | `violated` | 1 | 1 | DocInfo / extra TRACKCHANGE |
| `X06` | `hwpx-forbidden-char` | `docinfo` | `E` | 열림 + 조판 실패 | `violated` | 1 | 1 | DocInfo / FORBIDDEN_CHAR defaults |
| `X07` | `hwpx-tab-def` | `docinfo` | `D` | 열림 + 조판 실패 | `violated` | 1 | 1 | DocInfo / TAB_DEF |
| `X08` | `hwpx-doc-data` | `docinfo` | `A` | 파일 읽기 오류 | `violated` | 1 | 1 | DocInfo / DOC_DATA ParameterSet |
| `X09` | `hwpx-autonumber` | `field` | `E` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:autoNum / AutoNumber |
| `X10` | `hwpx-newnumber` | `field` | `E` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:newNum / NewNumber |
| `X11` | `hwpx-index-mark` | `field` | `B` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:indexmark / IndexMark |
| `X12` | `hwpx-hidden-comment` | `note` | `B` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:hiddenComment / HiddenComment |
| `X13` | `hwpx-char-overlap` | `field` | `B` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:dutmal / CharOverlap |
| `X14` | `hwpx-tcps` | `field` | `B` | 열림 + 조판 실패 | `violated` | 1 | 1 | hp:compose / Tcps |
| `X15` | `hwpx-textbox-list-header` | `shape` | `B` | 파일 손상 | `violated` | 1 | 1 | hp:textbox / LIST_HEADER |
| `X16` | `hwpx-identical-roundtrip` | `para` | `A` | 성공 | `satisfied` | 0 | 0 | oracle == generated sentinel |
