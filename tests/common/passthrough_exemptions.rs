/// 무효화하지 않는 `pub fn (&mut self)` 전수 목록 — (파일, 함수, 분류, 근거).
///
/// 파일 경로는 [`SCAN_ROOT`] 기준 상대 경로다. 병합 `devel` 기준 46건(2026-08-30 동결).
const EXEMPT: &[(&str, &str, Exempt, &str)] = &[
    (
        "commands/border_paint_edit.rs",
        "discard_border_paint_undo_native",
        Exempt::SessionState,
        "#7998: history 소유 undo journal만 폐기하며 문서 IR·저장 raw stream을 변경하지 않는다. 복원 시 실제 편집은 별도 commit 경로에서 무효화한다.",
    ),
    (
        "native_pua_fonts.rs",
        "prepare_native_pua_fonts_with",
        Exempt::SessionState,
        "공급한 원 PUA 프로그램과 파생 측정·배치 캐시만 교체한다. 문서 IR을 편집하지 않으며 native_pua_host_font_protocol에서 세션 해제 전후 IR·저장 HWP 바이트 보존을 검사한다.",
    ),
    (
        "native_pua_fonts.rs",
        "register_native_pua_font",
        Exempt::SessionState,
        "family/style별 원 PUA 세션 프로그램과 파생 배치만 갱신한다. 저장 문서의 face·문자·raw를 바꾸지 않으며 공통 refresh_native_pua_font_layout 경로를 쓴다.",
    ),
    (
        "native_pua_fonts.rs",
        "register_literal_figure_space_font",
        Exempt::SessionState,
        "리터럴 공백의 실제 공급 프로그램과 파생 조판 캐시만 갱신한다. native_pua_host_font_protocol에서 등록·해제 전후 명시 exact 소유권·문서 IR·저장 HWP 바이트 보존을 검사한다.",
    ),
    (
        "native_pua_fonts.rs",
        "clear_native_pua_fonts",
        Exempt::SessionState,
        "원 PUA 공급과 파생 캐시만 해제한다. native_pua_host_font_protocol에서 명시 exact 소유권·원본 IR·실제 저장 HWP 바이트 보존을 검사한다.",
    ),
    (
        "commands/document/metafile_fonts.rs",
        "register_metafile_font_native",
        Exempt::SessionState,
        "메타파일 paint의 host font와 파생 cache만 바꾼다. metafile_font_session_contract가 등록·중복 등록·해제·재등록 전후 문서 IR·저장 bytes 보존을 검사한다.",
    ),
    (
        "commands/document/metafile_fonts.rs",
        "clear_metafile_fonts_native",
        Exempt::SessionState,
        "문서 IR을 바꾸지 않고 host font 세대와 paint cache만 해제한다. metafile_font_session_contract가 저장 bytes와 원본 IR 보존을 검사한다.",
    ),
    (
        "commands/document/hft_outline_fonts.rs",
        "register_hft_outline_font_native",
        Exempt::SessionState,
        "#7902: hft_outline_fonts의 paint 전용 프로그램만 등록·교체한다. 문서 IR과 저장 HWP 바이트 보존은 hft_outline_font_contract의 public API 실행으로 검사한다.",
    ),
    (
        "font_environment.rs",
        "set_font_environment",
        Exempt::SessionState,
        "#6389: 명시적 폰트 환경과 파생 조판·측정 캐시만 교체한다. issue_6389_environment_changes_measurement_and_paint_without_editing_document에서 적용·해제 전후 저장 bytes와 원본 face 보존을 검사한다.",
    ),
    // #7084 render-session APIs do not mutate serialized document state.
    // issue_7084_canvas_metric_session checks original HWP/HWPX IR and portable SVG preservation.
    (
        "canvas_metric_requests.rs",
        "collect_canvas_metric_requests",
        Exempt::SessionState,
        "#7084: 원문에서 측정 요청을 읽고 세션의 pending ticket만 갱신한다. 원문 IR 보존은 collected_original_requests_register_and_preserve_ir_and_portable_svg에서 검사한다.",
    ),
    (
        "canvas_metric_requests.rs",
        "register_canvas_metric_replies",
        Exempt::SessionState,
        "#7084: 요청 세대·descriptor 검증 후 렌더 전용 측정 snapshot을 등록한다. 원문·원본 스트림은 변경하지 않으며 실패 원자성과 정상 IR 보존을 issue_7084_canvas_metric_session에서 검사한다.",
    ),
    (
        "canvas_metrics.rs",
        "begin_canvas_metric_session",
        Exempt::SessionState,
        "#7084: 문서/폰트/backend 세대의 측정 소유자와 파생 조판 상태만 교체한다. canvas_session_switch_rebuilds_positions_and_protects_portable_output에서 IR 무변경을 검사한다.",
    ),
    (
        "canvas_metrics.rs",
        "register_canvas_metrics",
        Exempt::SessionState,
        "#7084: 보충 진행폭 store와 파생 조판만 갱신한다. canvas_session_switch_rebuilds_positions_and_protects_portable_output에서 IR·portable 출력 보존을 검사한다.",
    ),
    (
        "canvas_metrics.rs",
        "select_canvas_metrics",
        Exempt::SessionState,
        "#7084: Canvas/portable 렌더 문맥만 전환하고 저장 원본은 보존한다. collected_original_requests_register_and_preserve_ir_and_portable_svg에서 전환 전후를 대조한다.",
    ),
    (
        "hyperlink.rs",
        "insert_hyperlink_native",
        Exempt::DelegatesTo("commit_hyperlink_paragraph"),
        "검증한 필드 삽입 후보를 공통 commit helper에 넘긴다. helper가 구역 raw_stream과 페이지 캐시를 무효화한다 (#6963).",
    ),
    (
        "hyperlink.rs",
        "update_hyperlink_native",
        Exempt::DelegatesTo("commit_hyperlink_paragraph"),
        "주소가 바뀐 경우에만 공통 commit helper로 적용한다. helper가 구역 raw_stream을 무효화하며 동일 주소는 무변경이다 (#6963).",
    ),
    (
        "hyperlink.rs",
        "replace_hyperlink_text_native",
        Exempt::DelegatesTo("commit_hyperlink_paragraph"),
        "표시 문자열 교체 후보의 raw_stream 및 페이지 무효화를 공통 commit helper에 위임한다 (#6963).",
    ),
    (
        "hyperlink.rs",
        "remove_hyperlink_native",
        Exempt::DelegatesTo("remove_hyperlink_with_format_native"),
        "서식 복원 없이 필드만 제거하는 호환 래퍼. 복원 옵션을 받는 제거 경로와 공통 commit helper를 거쳐 무효화한다 (#6963).",
    ),
    (
        "hyperlink.rs",
        "remove_hyperlink_with_format_native",
        Exempt::DelegatesTo("commit_hyperlink_paragraph"),
        "표시 글자를 보존한 제거 후보를 공통 commit helper로 적용한다. helper가 구역 raw_stream과 페이지 캐시를 무효화한다 (#6963).",
    ),
    (
        "commands/paragraph_block/import_json.rs",
        "import_paragraph_block_json_native",
        Exempt::DelegatesTo("import_paragraph_block_native"),
        "#3587: JSON 어댑터의 dry-run은 무변경, 실행은 native 가져오기의 DocInfo/구역 무효화 경로에 위임한다. issue_3587_template_operation과 실제 WASM import-wasm-contract에서 실패 무변경 및 저장 재열기를 검사한다.",
    ),
    (
        "commands/paragraph_block/import.rs",
        "import_paragraph_block_native",
        Exempt::DelegatesTo("commit_block_content"),
        "#3587: 자원 준비 후 Resources::commit은 DocInfo를 dirty로 표시하고, 공통 commit_block_content가 삽입한 구역 raw_stream을 무효화한다. 실패 무변경 및 HWP/HWPX 재열기는 issue_3587_block_import에서 검사한다.",
    ),
    (
        "commands/paragraph_block/repeat.rs",
        "repeat_paragraph_block_native",
        Exempt::DelegatesTo("repeat_paragraph_block_prepared"),
        "#3587: 준비 후 commit_paragraph_block이 삽입과 대상 구역 raw_stream 무효화를 함께 수행한다.",
    ),
    (
        "commands/paragraph_block/template.rs",
        "repeat_and_fill_paragraph_block_native",
        Exempt::DelegatesTo("commit_paragraph_block"),
        "#3587: detached 채우기 준비 후 공통 commit에서 삽입·구역 원본 스트림 무효화를 수행한다.",
    ),
    (
        "commands/paragraph_block/template_operation.rs",
        "apply_template_operation_json_native",
        Exempt::DelegatesTo("execute_template_operation_native"),
        "#3587: dry-run은 무변경 조회, 실제 실행은 typed 연산 디스패처로 위임한다.",
    ),
    (
        "commands/paragraph_block/template_operation.rs",
        "execute_template_operation_native",
        Exempt::DelegatesTo("repeat_and_fill_paragraph_block_native"),
        "#3587: 문단 복제는 공통 commit, 나머지 분기는 fill_template_native·repeat_and_fill_table_rows_native가 직접 구역 원본 스트림을 무효화한다. 세 경로의 저장·재열기는 issue_3587_template_plan 및 template_operation에서 검증한다.",
    ),
    (
        "commands/object_ops/table.rs",
        "delete_cell_picture_control_by_path_native",
        Exempt::DelegatesTo("delete_cell_control_by_path_native"),
        "그림 종류를 지정하는 래퍼. 공통 helper가 삭제와 구역 raw_stream 무효화를 수행한다.",
    ),
    (
        "commands/object_ops/table.rs",
        "delete_cell_table_control_by_path_native",
        Exempt::DelegatesTo("delete_cell_control_by_path_native"),
        "표 종류를 지정하는 래퍼. 공통 helper가 삭제와 구역 raw_stream 무효화를 수행한다.",
    ),
    // ── 세션/캐시 상태만 변경 (문서 IR 비변경) ──────────────────────────────
    (
        "mod.rs",
        "set_dpi",
        Exempt::SessionState,
        "렌더 DPI·해소 스타일·페이지네이션만 갱신. 문서 IR 무변경.",
    ),
    (
        "mod.rs",
        "set_hangul2024_compat",
        Exempt::SessionState,
        "[#5524] 세션 호환 모드 플래그·재페이지네이션만 갱신. 문서 IR 무변경.",
    ),
    (
        "queries/rendering.rs",
        "begin_deferred_pagination",
        Exempt::SessionState,
        "편집이 만든 descriptor로 shadow pagination job과 측정 캐시만 준비. 문서 IR 무변경.",
    ),
    (
        "queries/rendering.rs",
        "step_deferred_pagination",
        Exempt::SessionState,
        "shadow 결과를 pagination·측정·dirty 캐시에 commit. `Table::dirty`는 직렬화 비대상 런타임 플래그.",
    ),
    (
        "queries/rendering.rs",
        "cancel_deferred_pagination",
        Exempt::SessionState,
        "pending shadow job만 폐기. 문서 IR과 패스스루 무변경.",
    ),
    (
        "queries/rendering.rs",
        "flush_deferred_pagination",
        Exempt::SessionState,
        "pending job을 drain하거나 기존 paginate로 fallback. 편집 IR은 선행 뮤테이터가 이미 무효화.",
    ),
    (
        "queries/rendering.rs",
        "repaginate_if_needed",
        Exempt::SessionState,
        "dirty 구역을 다시 쪽으로 나눌 뿐(pagination·측정 캐시). 문서 IR 무변경 — \
         선행 편집 뮤테이터가 이미 무효화했다.",
    ),
    (
        "queries/memo_query.rs",
        "insert_memo_json_native",
        Exempt::DelegatesTo("insert_memo_at"),
        "insertMemo의 {author,text} JSON을 읽은 뒤 insert_memo_at에 넘긴다. 구역 raw_stream 무효화는 insert_memo_at이 한다 (#8017).",
    ),
    (
        "queries/memo_query.rs",
        "insert_memo_by_path_json_native",
        Exempt::DelegatesTo("insert_memo_by_path"),
        "insertMemoByPath의 셀 경로와 JSON을 읽은 뒤 insert_memo_by_path에 넘긴다. 구역 raw_stream 무효화는 insert_memo_by_path가 한다 (#8017).",
    ),
    (
        "queries/changed_pages.rs",
        "pages_covering_paragraphs",
        Exempt::SessionState,
        "조판 커버리지를 읽어 페이지 번호만 돌려주는 조회. `&mut` 는 paginate_if_needed \
         한 줄 때문이며 문서 IR 무변경 — 편집 IR은 선행 뮤테이터가 이미 무효화.",
    ),
    (
        "commands/clipboard.rs",
        "clear_clipboard_native",
        Exempt::SessionState,
        "`self.clipboard = None` 뿐.",
    ),
    (
        "commands/clipboard.rs",
        "copy_selection_native",
        Exempt::SessionState,
        "복사 — 읽기 후 `self.clipboard` 에만 기록.",
    ),
    (
        "commands/clipboard.rs",
        "copy_selection_logical_native",
        Exempt::SessionState,
        "[#7444] 논리 오프셋 복사 — 읽기 후 `self.clipboard` 에만 기록.",
    ),
    (
        "commands/clipboard.rs",
        "copy_selection_in_cell_native",
        Exempt::SessionState,
        "복사 — 읽기 후 `self.clipboard` 에만 기록.",
    ),
    (
        "commands/clipboard.rs",
        "copy_selection_in_cell_by_path_native",
        Exempt::SessionState,
        "경로 기반 복사 — 읽기 후 `self.clipboard` 에만 기록.",
    ),
    (
        "commands/formatting_runs.rs",
        "get_char_shape_runs_in_cell_by_path_native",
        Exempt::SessionState,
        "[#6788] 구간 검증과 모양 목록 직렬화만 하는 순수 조회. `&mut` 는 가변 셀 접근자 재사용 때문이며 문서 IR 비변경.",
    ),
    (
        "commands/header_footer_ops.rs",
        "copy_selection_in_header_footer_native",
        Exempt::SessionState,
        "머리말/꼬리말 복사 — 읽기 후 `self.clipboard` 에만 기록. 문서 IR 비변경.",
    ),
    (
        "commands/clipboard.rs",
        "copy_control_native",
        Exempt::SessionState,
        "복사 — `self.clipboard` / `self.paste_cascade_count` 만 변경.",
    ),
    (
        "commands/document.rs",
        "begin_batch_native",
        Exempt::SessionState,
        "`batch_mode` 플래그와 이벤트 로그만 조작.",
    ),
    (
        "commands/document.rs",
        "end_batch_native",
        Exempt::SessionState,
        "`batch_mode` 해제 + 재페이지네이션 + 이벤트 로그 직렬화.",
    ),
    (
        "commands/document.rs",
        "register_exact_font_source_native",
        Exempt::SessionState,
        "[#4968] exact font source registry·kerning measurement context·layout caches만 갱신. 문서 IR·직렬화 대상 무변경.",
    ),
    (
        "commands/document.rs",
        "set_exact_font_instance_native",
        Exempt::SessionState,
        "[#4969] exact slot의 variable-font instance request와 조판 caches만 갱신. 문서 IR·직렬화 대상 무변경.",
    ),
    (
        "commands/document.rs",
        "clear_exact_font_instance_native",
        Exempt::SessionState,
        "[#4969] exact slot의 instance request를 제거하고 조판 caches만 무효화. 문서 IR·직렬화 대상 무변경.",
    ),
    (
        "commands/document.rs",
        "save_snapshot_native",
        Exempt::SessionState,
        "문서를 clone 해 `snapshot_store` 에 적재. 원본 IR 무변경.",
    ),
    (
        "commands/document.rs",
        "discard_snapshot_native",
        Exempt::SessionState,
        "`snapshot_store` 에서 항목 제거.",
    ),
    (
        "commands/delete_fragment.rs",
        "capture_delete_range_native",
        Exempt::SessionState,
        "[#5769] 삭제 직전 문단 범위를 `fragment_store` 에 적재. 원본 IR 을 읽기만 하고 바꾸지 않는다.",
    ),
    (
        "commands/delete_fragment.rs",
        "discard_delete_fragment_native",
        Exempt::SessionState,
        "[#5769] `fragment_store` 에서 조각 제거. 문서 IR 비변경.",
    ),
    (
        "commands/section_raw_journal.rs",
        "capture_section_raw_native",
        Exempt::SessionState,
        "[#5769] 속성 변경 직전 구역 raw+봉인을 저널에 적재. IR 비변경(읽기 전용).",
    ),
    (
        "commands/section_raw_journal.rs",
        "discard_section_raw_native",
        Exempt::SessionState,
        "[#5769] 구역 raw 저널에서 항목 제거. 문서 IR 비변경.",
    ),
    (
        "commands/picture_transform_journal.rs",
        "capture_picture_transform_native",
        Exempt::SessionState,
        "[#6806] 그림 common/shape 변환 상태를 복제해 Undo 저널과 ID만 갱신. 원본 문서 IR·구역 raw는 읽기만 한다.",
    ),
    (
        "commands/picture_transform_journal.rs",
        "discard_picture_transform_native",
        Exempt::SessionState,
        "[#6806] 그림 변환 Undo 저널에서 handle만 제거. 문서 IR·구역 raw는 변경하지 않는다.",
    ),
    (
        "commands/formatting.rs",
        "get_cell_char_properties_at_by_path",
        Exempt::SessionState,
        "순수 조회. `&mut` 는 가변 접근자(`get_cell_paragraph_mut_by_path`) 재사용 때문.",
    ),
    (
        "commands/header_footer_ops.rs",
        "toggle_hide_header_footer_native",
        Exempt::SessionState,
        "세션 집합 `hidden_header_footer` + 렌더 트리 캐시만 변경. 직렬화 비대상.",
    ),
    (
        "commands/table_ops.rs",
        "copy_table_cells_transposed_native",
        Exempt::SessionState,
        "복사 — `table_transpose_clipboard` 에만 기록.",
    ),
    (
        "queries/field_query.rs",
        "set_active_field",
        Exempt::SessionState,
        "편집 세션의 활성 필드 포커스 + 렌더 캐시 무효화. 직렬화 비대상.",
    ),
    (
        "queries/field_query.rs",
        "set_active_field_in_cell",
        Exempt::SessionState,
        "편집 세션의 활성 필드 포커스. 직렬화 비대상.",
    ),
    (
        "queries/field_query.rs",
        "set_active_field_by_path",
        Exempt::SessionState,
        "편집 세션의 활성 필드 포커스. 직렬화 비대상.",
    ),
    (
        "queries/field_query.rs",
        "clear_active_field",
        Exempt::SessionState,
        "활성 필드 해제 + 렌더 캐시 무효화. 직렬화 비대상.",
    ),
    // ── 문서 전체 교체·생성 ────────────────────────────────────────────────
    (
        "commands/document.rs",
        "create_blank_document_native",
        Exempt::WholeDocument,
        "내장 템플릿을 파싱해 `self.document` 를 통째로 교체. 패스스루도 함께 새로 설정된다.",
    ),
    (
        "commands/document.rs",
        "set_document",
        Exempt::WholeDocument,
        "주입된 `Document` 로 통째 교체. 패스스루는 주입 측 IR 의 것을 그대로 따른다.",
    ),
    (
        "commands/document.rs",
        "restore_snapshot_native",
        Exempt::WholeDocument,
        "스냅샷 문서로 통째 복원. 스냅샷은 저장 시점의 패스스루 상태를 그대로 담고 있다.",
    ),
    (
        "commands/delete_fragment.rs",
        "restore_delete_fragment_native",
        Exempt::WholeDocument,
        "[#5769] 조각 복원 — 캡처 시점의 section raw_stream·raw_provenance·캐럿(DocProperties)을 \
         통째로 되돌린다. 스냅샷 복원과 같은 원리로, 복원 뒤 패스스루는 되돌려진 IR 과 다시 일치한다.",
    ),
    (
        "commands/section_raw_journal.rs",
        "restore_section_raw_native",
        Exempt::WholeDocument,
        "[#5769] 구역 raw 복원 — 캡처 시점의 raw_stream·봉인을 되돌려 passthrough 와 IR 을 \
         다시 일치시킨다. 문단 등 IR 은 건드리지 않는다(속성 복원은 setter 호출부 몫).",
    ),
    // ── 무효화 대신 원본 스트림 직접 수술 ──────────────────────────────────
    (
        "commands/document.rs",
        "convert_to_editable_native",
        Exempt::SurgicalRawEdit,
        "`Document::convert_to_editable` 이 `header.raw_data = None` + \
         `distribute_doc_data_removed` 를 세우고, 직렬화기가 원본 스트림에서 \
         DISTRIBUTE_DOC_DATA 만 surgical remove 한다(`serializer/doc_info.rs:27-30`).",
    ),
    (
        "commands/formatting.rs",
        "find_or_create_font_id_native",
        Exempt::SurgicalRawEdit,
        "DocInfo 를 dirty 로 만드는 대신 `surgical_insert_font_all_langs` 로 원본 \
         스트림에 FACE_NAME 을 직접 삽입한다(전체 재직렬화 회피 — FIX-4 계열 위험).",
    ),
    (
        "commands/formatting.rs",
        "find_or_create_font_id_for_lang",
        Exempt::SurgicalRawEdit,
        "위와 동일 — `surgical_insert_font_all_langs` 로 원본 스트림 직접 반영.",
    ),
    // ── 패스스루 부재 ──────────────────────────────────────────────────────
    (
        "commands/document.rs",
        "populate_external_images_from_dir",
        Exempt::NoPassthrough,
        "HWP3 외부 경로 그림 전용(비-wasm CLI 경로). HWP3 파서는 \
         `raw_stream: None`(`parser/hwp3/mod.rs:3241`)이라 무효화할 패스스루가 없고, \
         적재 대상 `bin_data_content` 는 DocInfo 레코드가 아니라 BinData 저장소다.",
    ),
    (
        "commands/object_ops/chart.rs",
        "set_chart_data_native",
        Exempt::NoPassthrough,
        "[#4100] 차트 값 편집. 바꾸는 것은 `bin_data_content` 슬롯 바이트뿐이고 그것은 \
         BodyText·DocInfo 스트림이 아니라 **BinData 저장소**라 패스스루 대상이 아니다 \
         (`serializer/cfb_writer.rs`가 IR 에서 매번 재방출, HWPX 는 \
         `serializer/hwpx/mod.rs`가 `Chart/chartN.xml` 로 무가공 방출). 문단·컨트롤·\
         DocInfo 레코드는 건드리지 않는다. 근거는 말이 아니라 판정으로 둔다 — \
         `tests/issue_4100_chart_data_edit.rs::the_edit_survives_hwp5_save_despite_stream_passthrough` \
         가 HWP5 저장→재파스에서 편집 생존을 확인한다.",
    ),
    (
        "commands/object_ops/chart.rs",
        "set_chart_data_by_index_native",
        Exempt::NoPassthrough,
        "[#4100] 위와 같다 — 문서 순번으로 지목하는 정본 주소 경로이고 기록 대상은 동일한 \
         `bin_data_content` 슬롯이다.",
    ),
    // ── 호출자 책임 ────────────────────────────────────────────────────────
    (
        "commands/document.rs",
        "document_mut",
        Exempt::CallerResponsibility,
        "`&mut Document` 를 그대로 넘기는 탈출구. 스스로는 아무것도 바꾸지 않으므로 \
         무효화 책임은 전적으로 호출자에게 있다.",
    ),
    // ── 위임 ───────────────────────────────────────────────────────────────
    (
        "commands/object_ops/table.rs",
        "create_table_native",
        Exempt::DelegatesTo("create_table_with_options_native"),
        "[#5819] 기본 옵션을 전달하는 래퍼. 공통 표 생성 함수가 구역 raw_stream을 무효화한다.",
    ),
    (
        "commands/table_ops.rs",
        "paste_table_cells_transposed_as_new_table_native",
        Exempt::DelegatesTo("create_table_native"),
        "표 생성 경로를 그대로 태운다.",
    ),
    (
        "commands/table_ops.rs",
        "fit_table_to_page_native",
        Exempt::DelegatesTo("set_table_column_widths_native"),
        "열 폭 계산만 하고 실제 반영은 열 폭 설정 뮤테이터가 한다.",
    ),
    (
        "commands/table_ops.rs",
        "evaluate_table_formula",
        Exempt::DelegatesTo("replace_text_in_cell_native_impl"),
        "[#4135] 계산만 할 때는 IR 비변경. 결과 기록은 일반 셀 텍스트 치환 경로에 위임하며 \
         그 경로가 section raw passthrough를 무효화한다.",
    ),
    (
        "commands/table_ops.rs",
        "remove_border_fill_tails_native",
        Exempt::CallerResponsibility,
        "[#5959] doc_info.border_fills 의 고아 꼬리 절단 + dirty 플래그 원복만 한다 — \
         섹션 본문 IR 무변경이며 passthrough 무효화·복원은 호출자(TS undo 의 \
         applyIds → restoreSectionRaw 3단)가 담당한다.",
    ),
    (
        "commands/text_editing.rs",
        "insert_text_native",
        Exempt::DelegatesTo("insert_text_at_caret_native"),
        "[#7444] 얇은 래퍼 — 개체 뒤 여부만 false 로 넘기고 삽입·무효화는 위임 대상이 한다.",
    ),
    (
        "commands/text_editing.rs",
        "delete_text_native",
        Exempt::DelegatesTo("apply_body_text_edit"),
        "[#7866] 본문 글자 편집의 공통 꼬리(그림 밴드 분기·raw_stream 무효화·재조판·캐럿)를 \
         `apply_body_text_edit` 로 추출했다. 이 함수는 삭제 클로저와 이벤트만 더한다.",
    ),
    (
        "commands/text_editing.rs",
        "replace_text_inheriting_char_shapes_native",
        Exempt::DelegatesTo("apply_body_text_edit"),
        "[#7866] 지운 글자의 모양을 잇는 한 단계 바꾸기 — 무효화·재조판은 `delete_text_native` \
         와 같은 `apply_body_text_edit` 가 수행한다. issue_7866_replace_inherits_char_shapes 가 \
         저장·재열기 뒤 모양 보존을 검사한다.",
    ),
    (
        "commands/text_editing.rs",
        "insert_text_in_cell_native",
        Exempt::DelegatesTo("replace_text_in_cell_native_impl"),
        "얇은 래퍼 — 실제 삽입·무효화는 `_impl` 이 수행.",
    ),
    (
        "commands/page_extract.rs",
        "extract_page_range",
        Exempt::DelegatesTo("delete_paragraph_native"),
        "[#3565] 지우는 일은 전부 문단 삭제 뮤테이터에 위임하고, 그쪽이 손댄 구역의 \
         `raw_stream` 을 무효화한다. 한 문단도 지우지 않은 구역은 원본이 그대로 \
         유효하므로 통과를 남기는 것이 맞다.",
    ),
    (
        "commands/text_editing.rs",
        "insert_text_in_cell_native_deferred_pagination",
        Exempt::DelegatesTo("replace_text_in_cell_native_impl"),
        "페이지네이션 지연 플래그만 다른 래퍼 — 본체는 `_impl`.",
    ),
    (
        "commands/text_editing.rs",
        "replace_text_in_cell_native_deferred_pagination",
        Exempt::DelegatesTo("replace_text_in_cell_native_impl"),
        "원자 치환 래퍼 — 실제 치환·무효화는 `_impl` 이 수행.",
    ),
    (
        "commands/text_editing.rs",
        "delete_text_in_cell_native",
        Exempt::DelegatesTo("delete_text_in_cell_native_impl"),
        "얇은 래퍼 — 실제 삭제·무효화는 `_impl` 이 수행.",
    ),
    (
        "commands/text_editing.rs",
        "delete_text_in_cell_native_deferred_pagination",
        Exempt::DelegatesTo("delete_text_in_cell_native_impl"),
        "페이지네이션 지연 플래그만 다른 래퍼 — 본체는 `_impl`.",
    ),
    (
        "queries/field_query.rs",
        "set_field_value_by_name",
        Exempt::DelegatesTo("set_field_value_by_name_at"),
        "첫 occurrence를 선택하는 호환 래퍼. 실제 필드 치환·section raw_stream 무효화는 occurrence 경로가 수행.",
    ),
    (
        "queries/rendering.rs",
        "set_section_def_native",
        Exempt::DelegatesTo("apply_section_def_json"),
        "JSON 파싱·적용은 공통 헬퍼가 담당.",
    ),
    (
        "queries/rendering.rs",
        "set_section_def_all_native",
        Exempt::DelegatesTo("apply_section_def_json"),
        "전 구역 루프 — 구역별 적용·무효화는 공통 헬퍼가 담당.",
    ),
    (
        "queries/search_query.rs",
        "replace_text_native",
        Exempt::DelegatesTo("delete_text_native"),
        "삭제 + 삽입 조합. 두 뮤테이터가 각각 무효화한다.",
    ),
    (
        "queries/search_query.rs",
        "replace_one_native",
        Exempt::DelegatesTo("delete_text_native"),
        "검색 후 삭제 + 삽입 조합. 두 뮤테이터가 각각 무효화한다.",
    ),
    (
        "queries/search_query.rs",
        "replace_all_native",
        Exempt::DelegatesTo("replace_matches_native"),
        "[#3395] 전량 치환 몸통이 공통 헬퍼로 이관됨. 무효화(`raw_stream = None`)는 헬퍼가 수행.",
    ),
    (
        "queries/search_query.rs",
        "replace_nth_native",
        Exempt::DelegatesTo("replace_matches_native"),
        "[#3395] k번째 매치 치환 — replace_all_native 와 같은 공통 헬퍼에 위임. 무효화는 헬퍼가 수행.",
    ),
    (
        "queries/field_query.rs",
        "insert_click_here_field_at_cursor",
        Exempt::DelegatesTo("insert_click_here_field_at"),
        "웹한글컨트롤 커서 좌표(list/para/pos)를 구역·문단·글자 번호로 옮겨 넘길 뿐이다. \
         삽입과 무효화는 본문 경로(`insert_click_here_field_at`)와 셀 경로 \
         (`insert_click_here_field_at_by_path`)가 한다.",
    ),
    (
        "queries/hwpctrl_sets.rs",
        "apply_char_format_at_cursor",
        Exempt::DelegatesTo("apply_char_format_native"),
        "좌표만 옮긴다(코드 유닛 → 글자 번호). 서식 적용과 무효화는 본문 경로 \
         (`apply_char_format_native`)와 셀 경로(`apply_char_format_in_cell_by_path`)가 한다.",
    ),
    (
        "queries/hwpctrl_sets.rs",
        "split_para_at_cursor",
        Exempt::DelegatesTo("split_paragraph_native"),
        "좌표만 옮긴다(코드 유닛 → 글자 번호). 가르기와 무효화는 본문 경로 \
         (`split_paragraph_native`)와 셀 경로(`split_paragraph_in_cell_by_path`)가 한다.",
    ),
    (
        "commands/table_ops.rs",
        "delete_table_control_native",
        Exempt::DelegatesTo("delete_control_native_impl"),
        "표만 받는지 검사하고 넘긴다. 지우기와 무효화는 `delete_control_native_impl` 이 한다.",
    ),
    (
        "commands/table_ops.rs",
        "delete_control_native",
        Exempt::DelegatesTo("delete_control_native_impl"),
        "갈래 검사 없이 넘길 뿐이다. 지우기와 무효화는 `delete_control_native_impl` 이 한다.",
    ),
    (
        "queries/hwpctrl_sets.rs",
        "delete_control_at",
        Exempt::DelegatesTo("delete_control_native"),
        "본문 문단 번호를 구역·문단으로 풀어 넘길 뿐이다. 지우기와 무효화는 아래가 한다.",
    ),
    (
        "queries/hwpctrl_sets.rs",
        "insert_text_at_cursor",
        Exempt::DelegatesTo("insert_text_native"),
        "좌표만 옮긴다(코드 유닛 → 글자 번호). 끼우기와 무효화는 본문 경로 \
         (`insert_text_native`)와 셀 경로(`insert_text_in_cell_by_path`)가 한다.",
    ),
    (
        "queries/hwpctrl_sets.rs",
        "table_merge_at_cursor",
        Exempt::DelegatesTo("merge_table_cells_native"),
        "리스트 아이디를 구역·문단·컨트롤·행·열로 풀어 넘길 뿐이다. 합치는 것과 무효화는 \
         `merge_table_cells_native` 가 한다.",
    ),
    (
        "queries/hwpctrl_sets.rs",
        "table_edit_at_cursor",
        Exempt::DelegatesTo("insert_table_row_native"),
        "리스트 아이디를 구역·문단·컨트롤·행·열로 풀어 넘길 뿐이다. 표를 고치는 것과 무효화는 \
         `insert_table_row_native`·`delete_table_row_native` 같은 표 편집 API 가 한다.",
    ),
    (
        "queries/hwpctrl_sets.rs",
        "delete_at_cursor",
        Exempt::DelegatesTo("delete_text_native"),
        "좌표만 옮긴다(코드 유닛 → 글자 번호). 삭제와 무효화는 본문 경로 \
         (`delete_text_native`)와 셀 경로(`delete_range_in_cell_by_path`)가 한다.",
    ),
    (
        "queries/hwpctrl_sets.rs",
        "apply_para_format_at_cursor",
        Exempt::DelegatesTo("apply_para_format_native"),
        "리스트 아이디를 구역·문단으로 풀어 넘길 뿐이다. 서식 적용과 무효화는 본문 경로 \
         (`apply_para_format_native`)와 셀 경로(`apply_para_format_in_cell_native`)가 한다.",
    ),
    // ── 판정 보류 ──────────────────────────────────────────────────────────
    (
        "commands/document.rs",
        "reflow_linesegs_on_demand",
        Exempt::Pending,
        "[#2724 3.4] wasm 공개 API `reflowLinesegs`. `para.line_segs` 를 재작성하지만 \
         `raw_stream` 을 비우지 않아 `raw_stream` 보유 문서에서는 보정이 저장되지 않는다. \
         다만 docstring 이 효과를 렌더 기준으로 서술하고, studio 는 #2527 이후 이 API 를 \
         호출하지 않아 관측된 유실이 없다. 구역 전체 재직렬화(FIX-4 계열) 위험이 있어 \
         추정으로 고치지 않고 지속성 계약 확정까지 보류한다.",
    ),
];
