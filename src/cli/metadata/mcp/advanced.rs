use super::{path_schema, tool, tool_with_optional_args};

pub(super) fn extend(tools: &mut Vec<serde_json::Value>) {
    tools.extend([
        tool_with_optional_args(
            "hwp_render_diff",
            "두 렌더의 페이지별 bbox 변위(px)를 재어 시각 회귀를 판정한다. pathB 를 주면 두 문서 직접 비교, 없으면 자기 라운드트립(원본 IR vs 직렬화→재로드 IR, via 로 경유 포맷 선택)이다. 판정은 status(PASS/WARN_TEXTRUN/OVER/STRUCT_MISMATCH/PAGE_MISMATCH)와 regression 으로 읽고, maxDisp·pages[].topDeltas 로 어디가 얼마나 밀렸는지 좁힌다. 회귀를 찾으면 종료 코드 3 이지만 봉투는 정상 산출된다(도구 실패가 아니라 검출이다).",
            path_schema(serde_json::json!({
                "pathB": { "type": "string", "description": "비교 대상 문서 경로. 주면 pair 모드(라운드트립 아님), 생략하면 자기 라운드트립" },
                "via": { "type": "string", "enum": ["hwpx", "hwp"], "description": "자기 라운드트립 경유 포맷. 기본 hwpx. pathB 를 준 pair 모드에서는 무의미하다" },
                "page": { "type": "integer", "minimum": 0, "description": "특정 페이지만 (0 기준). 비교 범위 밖이면 usage error(2)" },
                "maxDisp": { "type": "number", "minimum": 0, "description": "변위 임계값(px). 기본 1.0 — 초과 페이지가 있으면 status=OVER" }
            })),
            "render-diff",
            serde_json::json!(["render-diff", "--json", "{path}"]),
            serde_json::json!([
                { "when": "pathB", "args": ["{pathB}"] },
                { "when": "via", "args": ["--via", "{via}"] },
                { "when": "page", "args": ["-p", "{page}"] },
                { "when": "maxDisp", "args": ["--max-disp", "{maxDisp}"] }
            ]),
            &[
                "schemaVersion", "mode", "sourceA", "sourceB", "via", "pageFilter", "threshold",
                "pageCountA", "pageCountB", "pageCountMismatch", "maxDisp", "worstPage",
                "overPages", "structPages", "hardStructPages", "status", "regression", "pages",
            ],
        ),
        tool_with_optional_args(
            "hwp_layout_anomaly",
            "렌더 한 장의 기하에서 overflow·off-canvas·overlap·text-overlap·중간 빈 쪽 이상 신호를 찾는다. render-diff가 두 렌더 사이의 변위를 재는 것과 달리, 이 도구는 단일 렌더 자체가 정상적인지 판정한다. overflow는 본문 여백(Body) 밖, off-canvas는 페이지 상자 밖 또는 y<0이며, text-overlap은 텍스트 런 bbox 교차(글자끼리)만 보므로 표·이미지 겹침과 다르다. 기본은 발견 결과를 데이터로 보고 성공하며, strict를 주면 overflow·off-canvas·overlap·text-overlap 확정 신호가 있을 때 종료 코드 3을 반환한다(빈 쪽은 가능성 신호라 strict에서도 실패시키지 않는다).",
            path_schema(serde_json::json!({
                "page": { "type": "integer", "minimum": 0, "description": "특정 페이지만 검사하는 0 기준 번호. 생략하면 전체 문서" },
                "strict": { "type": "boolean", "description": "참이면 overflow·off-canvas·overlap·text-overlap 확정 신호가 발견될 때 검증 실패(exit 3)로 처리. 빈 쪽 신호는 실패시키지 않음" },
                "overflowTolerance": { "type": "number", "minimum": 0, "description": "본문 여백(overflow) 또는 페이지 상자(off-canvas) 밖으로 벗어난 요소를 잡을 최소 거리(px). 기본 1.0. 음수 y도 이 허용치를 쓴다" },
                "overlapTolerance": { "type": "number", "minimum": 0, "description": "두 요소를 overlap으로 볼 최소 겹침 폭과 높이(px). 기본 2.0" },
                "types": { "type": "string", "description": "overflow/overlap 검사 대상 노드 타입. 쉼표 구분(예: Table,Image,TextLine). off-canvas·text-overlap·empty_page 는 영향 없음" },
                "batch": { "type": "boolean", "description": "참이면 path 를 폴더로 보고 .hwp/.hwpx 를 재귀 스윕. stdout 은 NDJSON 한 줄 한 파일, 로드 실패는 error 레코드" }
            })),
            "layout-anomaly",
            serde_json::json!(["layout-anomaly", "--json", "{path}"]),
            serde_json::json!([
                { "when": "page", "args": ["-p", "{page}"] },
                { "when": "strict", "args": ["--strict"] },
                { "when": "overflowTolerance", "args": ["--overflow-tolerance", "{overflowTolerance}"] },
                { "when": "overlapTolerance", "args": ["--overlap-tolerance", "{overlapTolerance}"] },
                { "when": "types", "args": ["--types", "{types}"] },
                { "when": "batch", "args": ["--batch"] }
            ]),
            &[
                "schemaVersion", "mode", "source", "pageCount", "pageFilter", "overflowTolerancePx",
                "overlapTolerancePx", "types", "strict", "overflowCount", "offCanvasCount", "overlapCount",
                "textOverlapCount",
                "emptyPageCount", "hasSignal", "pages",
            ],
        ),
        tool_with_optional_args(
            "hwp_set_chart_data",
            "문서 순번 차트의 숫자 데이터를 JSON 으로 바꾼다. 코어 set_chart_data_by_index_native 배선. chart 는 문서 순서 1부터. data.structure=true 면 행렬이 목표 상태다 — 행 추가·삭제(꼬리 기준), 계열 추가·삭제(이름·값 대응이 서면 정체 보존: insertSeries 가 새 계열을 지정 자리에 기본 스타일로 끼우고 removeSeries 가 요소째 들어내 잔여 계열이 자기 스타일을 지킨다. 대응이 모호하면 꼬리 기준 폴백), 계열명·라벨 변경까지 쓴다(주식형 캔들 양끝 계열 고정·마지막 1점/1계열 삭제 거부). 없으면 값만 바꾸고 치수·이름·라벨 불일치는 invalid 로 거부. dryRun 도 코어 검증을 거쳐 거부 사유와 changed[].op 를 돌려준다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "chart": { "type": "integer", "minimum": 1, "description": "차트 번호(문서 순서, 1부터)" },
                    "data": { "type": "string", "description": "편집 JSON (labels?, series[{name?, values[]}], structure?: boolean, dryRun?: boolean). structure=true 면 행렬이 목표 상태(행·열 증감·계열명·라벨 변경 허용)" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "chart", "data"],
            }),
            "edit",
            serde_json::json!(["edit", "set-chart-data", "{path}", "--chart", "{chart}", "--data", "{data}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &[
                "schemaVersion", "source", "count", "dryRun", "changedCount", "changed", "wrote", "invalid",
                "changedPages", "output", "outputFormat", "verify",
            ],
        ),
        tool_with_optional_args(
            "hwp_insert_number",
            "문단 좌표에 쪽 새 번호로 시작 컨트롤을 넣는다. 코어 insert_new_number_native 배선. count 는 시작 번호(1~65535, 기본 1).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "offset": { "type": "integer", "minimum": 0 },
                    "count": { "type": "integer", "minimum": 1, "maximum": 65535, "description": "시작 쪽 번호" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-number", "{path}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "count", "args": ["--count", "{count}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "offset", "count", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_insert_shape",
            "본문 문단에 도형(기본 사각형)을 끼운다. 길이 단위는 HWPUNIT(1/7200 inch). 코어 create_shape_control_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "offset": { "type": "integer", "minimum": 0 },
                    "width": { "type": "integer", "minimum": 0, "description": "너비 HWPUNIT. height 와 동시에 0 이면 거부" },
                    "height": { "type": "integer", "minimum": 0, "description": "높이 HWPUNIT" },
                    "x": { "type": "integer", "minimum": 0, "description": "가로 오프셋 HWPUNIT" },
                    "y": { "type": "integer", "minimum": 0, "description": "세로 오프셋 HWPUNIT" },
                    "shape": { "type": "string", "description": "rectangle|ellipse|line|textbox|polygon|arc. 기본 rectangle" },
                    "wrap": { "type": "string", "description": "Square|Tight|Through|TopAndBottom|BehindText|InFrontOfText. 기본 InFrontOfText" },
                    "treatAsChar": { "type": "boolean" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "width", "height"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-shape", "{path}", "--width", "{width}", "--height", "{height}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "x", "args": ["--x", "{x}"] },
                { "when": "y", "args": ["--y", "{y}"] },
                { "when": "shape", "args": ["--shape", "{shape}"] },
                { "when": "wrap", "args": ["--wrap", "{wrap}"] },
                { "when": "treatAsChar", "args": ["--treat-as-char"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "offset", "width", "height", "x", "y", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_picture",
            "본문 그림 속성을 JSON 으로 바꾼다. section/para/ctrl 은 0 기준. 코어 set_picture_properties_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "ctrl": { "type": "integer", "minimum": 0 },
                    "props": { "type": "string", "description": "그림 속성 JSON (예: {\"brightness\":50,\"treatAsChar\":true})" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "section", "paragraph", "ctrl", "props"],
            }),
            "edit",
            serde_json::json!(["edit", "set-picture", "{path}", "--section", "{section}", "--para", "{paragraph}", "--ctrl", "{ctrl}", "--props", "{props}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "ctrl", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_form_value",
            "본문 양식 컨트롤의 값을 JSON으로 바꾼다. section/para/ctrl 은 0 기준.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "ctrl": { "type": "integer", "minimum": 0 },
                    "value": { "type": "string", "description": "양식 값 JSON (예: {\"text\":\"값\"})" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "section", "paragraph", "ctrl", "value"],
            }),
            "edit",
            serde_json::json!(["edit", "set-form-value", "{path}", "--section", "{section}", "--para", "{paragraph}", "--ctrl", "{ctrl}", "--value", "{value}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "ctrl", "value", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_form_value_in_cell",
            "표 셀 안 양식 컨트롤의 값을 JSON으로 바꾼다. 모든 좌표는 0 기준.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "tablePara": { "type": "integer", "minimum": 0 },
                    "tableCi": { "type": "integer", "minimum": 0 },
                    "cell": { "type": "integer", "minimum": 0 },
                    "cellPara": { "type": "integer", "minimum": 0 },
                    "ctrl": { "type": "integer", "minimum": 0 },
                    "value": { "type": "string", "description": "양식 값 JSON (예: {\"value\":1})" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "section", "tablePara", "tableCi", "cell", "cellPara", "ctrl", "value"],
            }),
            "edit",
            serde_json::json!(["edit", "set-form-value-in-cell", "{path}", "--section", "{section}", "--table-para", "{tablePara}", "--table-ci", "{tableCi}", "--cell", "{cell}", "--cell-para", "{cellPara}", "--ctrl", "{ctrl}", "--value", "{value}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "tablePara", "tableCi", "cell", "cellPara", "ctrl", "value", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_equation_properties",
            "본문 수식 속성을 바꾼다. --section/--para/--ctrl 은 수식 컨트롤 좌표. --props 는 script 등 JSON. 코어 set_equation_properties_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "ctrl": { "type": "integer", "minimum": 0 },
                    "props": { "type": "string", "description": "수식 속성 JSON (예: {\"script\":\"x^2\"})" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "section", "paragraph", "ctrl", "props"],
            }),
            "edit",
            serde_json::json!(["edit", "set-equation-properties", "{path}", "--section", "{section}", "--para", "{paragraph}", "--ctrl", "{ctrl}", "--props", "{props}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "ctrl", "text", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_page_border_fill",
            "구역의 쪽 테두리/배경을 바꾼다. --props 는 spacingLeft/fillColor 등 JSON. 코어 set_page_border_fill_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "props": { "type": "string", "description": "쪽 테두리/배경 JSON (예: {\"spacingLeft\":100})" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "props"],
            }),
            "edit",
            serde_json::json!(["edit", "set-page-border-fill", "{path}", "--props", "{props}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "props", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
    ]);

    super::add_edit_tool(
        tools,
        "hwp_set_hf_picture",
        "머리말/꼬리말 안 그림 속성을 바꾼다. para/ctrl은 머리말·꼬리말 컨트롤, innerPara/innerCtrl은 그 안 그림이다.",
        serde_json::json!({
            "path": { "type": "string" },
            "section": { "type": "integer", "minimum": 0 },
            "paragraph": { "type": "integer", "minimum": 0 },
            "ctrl": { "type": "integer", "minimum": 0 },
            "innerPara": { "type": "integer", "minimum": 0 },
            "innerCtrl": { "type": "integer", "minimum": 0 },
            "props": { "type": "string", "description": "그림 속성 JSON" },
            "output": { "type": "string" },
            "dryRun": { "type": "boolean" }
        }),
        &["path", "section", "paragraph", "ctrl", "innerPara", "innerCtrl", "props"],
        serde_json::json!(["edit", "set-hf-picture", "{path}", "--section", "{section}", "--para", "{paragraph}", "--ctrl", "{ctrl}", "--inner-para", "{innerPara}", "--inner-ctrl", "{innerCtrl}", "--props", "{props}", "--json"]),
        serde_json::json!([
            { "when": "output", "args": ["-o", "{output}"] },
            { "when": "dryRun", "args": ["--dry-run"] }
        ]),
        &["schemaVersion", "source", "section", "paragraph", "ctrl", "innerPara", "innerCtrl", "props", "dryRun", "changedPages", "output", "outputFormat", "verify"],
    );
    super::add_edit_tool(
        tools,
        "hwp_apply_hf_template",
        "머리말/꼬리말에 마당(템플릿)을 적용한다. template은 0부터 10까지다.",
        serde_json::json!({
            "path": { "type": "string" },
            "header": { "type": "boolean" },
            "footer": { "type": "boolean" },
            "template": { "type": "integer", "minimum": 0, "maximum": 10 },
            "section": { "type": "integer", "minimum": 0 },
            "applyTo": { "type": "integer", "minimum": 0, "maximum": 2 },
            "output": { "type": "string" },
            "dryRun": { "type": "boolean" }
        }),
        &["path", "template"],
        serde_json::json!([
            "edit",
            "apply-hf-template",
            "{path}",
            "--template",
            "{template}",
            "--json"
        ]),
        serde_json::json!([
            { "when": "header", "args": ["--header"] },
            { "when": "footer", "args": ["--footer"] },
            { "when": "section", "args": ["--section", "{section}"] },
            { "when": "applyTo", "args": ["--apply-to", "{applyTo}"] },
            { "when": "output", "args": ["-o", "{output}"] },
            { "when": "dryRun", "args": ["--dry-run"] }
        ]),
        &[
            "schemaVersion",
            "source",
            "section",
            "isHeader",
            "applyTo",
            "templateId",
            "dryRun",
            "changedPages",
            "output",
            "outputFormat",
            "verify",
        ],
    );
    super::add_edit_tool(
        tools,
        "hwp_toggle_hide_hf",
        "지정 쪽의 머리말 또는 꼬리말 감추기를 토글한다. page는 0부터다.",
        serde_json::json!({
            "path": { "type": "string" },
            "header": { "type": "boolean" },
            "footer": { "type": "boolean" },
            "page": { "type": "integer", "minimum": 0 },
            "output": { "type": "string" },
            "dryRun": { "type": "boolean" }
        }),
        &["path"],
        serde_json::json!(["edit", "toggle-hide-hf", "{path}", "--json"]),
        serde_json::json!([
            { "when": "header", "args": ["--header"] },
            { "when": "footer", "args": ["--footer"] },
            { "when": "page", "args": ["--page", "{page}"] },
            { "when": "output", "args": ["-o", "{output}"] },
            { "when": "dryRun", "args": ["--dry-run"] }
        ]),
        &[
            "schemaVersion",
            "source",
            "page",
            "isHeader",
            "hidden",
            "dryRun",
            "changedPages",
            "output",
            "outputFormat",
            "verify",
        ],
    );
    super::add_edit_tool(
        tools,
        "hwp_apply_para_format_in_hf",
        "머리말/꼬리말 문단에 문단 서식 JSON을 적용한다.",
        serde_json::json!({
            "path": { "type": "string" },
            "header": { "type": "boolean" },
            "footer": { "type": "boolean" },
            "applyTo": { "type": "integer", "minimum": 0, "maximum": 2 },
            "section": { "type": "integer", "minimum": 0 },
            "paragraph": { "type": "integer", "minimum": 0 },
            "props": { "type": "string", "description": "문단 서식 JSON" },
            "output": { "type": "string" },
            "dryRun": { "type": "boolean" }
        }),
        &["path", "props"],
        serde_json::json!([
            "edit",
            "apply-para-format-in-hf",
            "{path}",
            "--props",
            "{props}",
            "--json"
        ]),
        serde_json::json!([
            { "when": "header", "args": ["--header"] },
            { "when": "footer", "args": ["--footer"] },
            { "when": "applyTo", "args": ["--apply-to", "{applyTo}"] },
            { "when": "section", "args": ["--section", "{section}"] },
            { "when": "paragraph", "args": ["--para", "{paragraph}"] },
            { "when": "output", "args": ["-o", "{output}"] },
            { "when": "dryRun", "args": ["--dry-run"] }
        ]),
        &[
            "schemaVersion",
            "source",
            "section",
            "isHeader",
            "applyTo",
            "paragraph",
            "props",
            "dryRun",
            "changedPages",
            "output",
            "outputFormat",
            "verify",
        ],
    );
    super::add_edit_tool(
        tools,
        "hwp_apply_endnote_shape",
        "구역의 미주 모양 JSON을 적용한다.",
        serde_json::json!({
            "path": { "type": "string" },
            "section": { "type": "integer", "minimum": 0 },
            "props": { "type": "string", "description": "미주 모양 JSON" },
            "output": { "type": "string" },
            "dryRun": { "type": "boolean" }
        }),
        &["path", "props"],
        serde_json::json!([
            "edit",
            "apply-endnote-shape",
            "{path}",
            "--props",
            "{props}",
            "--json"
        ]),
        serde_json::json!([
            { "when": "section", "args": ["--section", "{section}"] },
            { "when": "output", "args": ["-o", "{output}"] },
            { "when": "dryRun", "args": ["--dry-run"] }
        ]),
        &[
            "schemaVersion",
            "source",
            "section",
            "dryRun",
            "changedPages",
            "output",
            "outputFormat",
            "verify",
        ],
    );
    super::add_edit_tool(
        tools,
        "hwp_insert_footnote_text",
        "각주 문단에 텍스트를 넣는다. ctrl은 본문 문단의 각주 컨트롤 인덱스다.",
        serde_json::json!({
            "path": { "type": "string" },
            "text": { "type": "string" },
            "section": { "type": "integer", "minimum": 0 },
            "paragraph": { "type": "integer", "minimum": 0 },
            "ctrl": { "type": "integer", "minimum": 0 },
            "fnPara": { "type": "integer", "minimum": 0 },
            "offset": { "type": "integer", "minimum": 0 },
            "output": { "type": "string" },
            "dryRun": { "type": "boolean" }
        }),
        &["path", "text", "ctrl"],
        serde_json::json!([
            "edit",
            "insert-footnote-text",
            "{path}",
            "--ctrl",
            "{ctrl}",
            "--text",
            "{text}",
            "--json"
        ]),
        serde_json::json!([
            { "when": "section", "args": ["--section", "{section}"] },
            { "when": "paragraph", "args": ["--para", "{paragraph}"] },
            { "when": "fnPara", "args": ["--fn-para", "{fnPara}"] },
            { "when": "offset", "args": ["--offset", "{offset}"] },
            { "when": "output", "args": ["-o", "{output}"] },
            { "when": "dryRun", "args": ["--dry-run"] }
        ]),
        &[
            "schemaVersion",
            "source",
            "section",
            "paragraph",
            "ctrl",
            "fnPara",
            "offset",
            "text",
            "dryRun",
            "changedPages",
            "output",
            "outputFormat",
            "verify",
        ],
    );
    super::add_edit_tool(
        tools,
        "hwp_split_paragraph_in_footnote",
        "각주/미주 문단을 오프셋에서 나눈다.",
        serde_json::json!({
            "path": { "type": "string" },
            "section": { "type": "integer", "minimum": 0 },
            "paragraph": { "type": "integer", "minimum": 0 },
            "ctrl": { "type": "integer", "minimum": 0 },
            "fnPara": { "type": "integer", "minimum": 0 },
            "offset": { "type": "integer", "minimum": 0 },
            "output": { "type": "string" },
            "dryRun": { "type": "boolean" }
        }),
        &["path"],
        serde_json::json!(["edit", "split-paragraph-in-footnote", "{path}", "--json"]),
        serde_json::json!([
            { "when": "section", "args": ["--section", "{section}"] },
            { "when": "paragraph", "args": ["--para", "{paragraph}"] },
            { "when": "ctrl", "args": ["--ctrl", "{ctrl}"] },
            { "when": "fnPara", "args": ["--fn-para", "{fnPara}"] },
            { "when": "offset", "args": ["--offset", "{offset}"] },
            { "when": "output", "args": ["-o", "{output}"] },
            { "when": "dryRun", "args": ["--dry-run"] }
        ]),
        &[
            "schemaVersion",
            "source",
            "section",
            "paragraph",
            "ctrl",
            "fnPara",
            "offset",
            "dryRun",
            "changedPages",
            "output",
            "outputFormat",
            "verify",
        ],
    );
    super::add_edit_tool(
        tools,
        "hwp_merge_paragraph_in_footnote",
        "각주/미주 문단을 바로 앞 문단에 합친다. fnPara는 합쳐질 문단이다.",
        serde_json::json!({
            "path": { "type": "string" },
            "section": { "type": "integer", "minimum": 0 },
            "paragraph": { "type": "integer", "minimum": 0 },
            "ctrl": { "type": "integer", "minimum": 0 },
            "fnPara": { "type": "integer", "minimum": 1 },
            "output": { "type": "string" },
            "dryRun": { "type": "boolean" }
        }),
        &["path"],
        serde_json::json!(["edit", "merge-paragraph-in-footnote", "{path}", "--json"]),
        serde_json::json!([
            { "when": "section", "args": ["--section", "{section}"] },
            { "when": "paragraph", "args": ["--para", "{paragraph}"] },
            { "when": "ctrl", "args": ["--ctrl", "{ctrl}"] },
            { "when": "fnPara", "args": ["--fn-para", "{fnPara}"] },
            { "when": "output", "args": ["-o", "{output}"] },
            { "when": "dryRun", "args": ["--dry-run"] }
        ]),
        &[
            "schemaVersion",
            "source",
            "section",
            "paragraph",
            "ctrl",
            "fnPara",
            "dryRun",
            "changedPages",
            "output",
            "outputFormat",
            "verify",
        ],
    );
    super::add_edit_tool(
        tools,
        "hwp_apply_para_format_in_footnote",
        "각주/미주 문단에 문단 서식 JSON을 적용한다.",
        serde_json::json!({
            "path": { "type": "string" },
            "section": { "type": "integer", "minimum": 0 },
            "paragraph": { "type": "integer", "minimum": 0 },
            "ctrl": { "type": "integer", "minimum": 0 },
            "fnPara": { "type": "integer", "minimum": 0 },
            "props": { "type": "string", "description": "문단 서식 JSON" },
            "output": { "type": "string" },
            "dryRun": { "type": "boolean" }
        }),
        &["path", "section", "paragraph", "ctrl", "props"],
        serde_json::json!([
            "edit",
            "apply-para-format-in-footnote",
            "{path}",
            "--section",
            "{section}",
            "--para",
            "{paragraph}",
            "--ctrl",
            "{ctrl}",
            "--props",
            "{props}",
            "--json"
        ]),
        serde_json::json!([
            { "when": "fnPara", "args": ["--fn-para", "{fnPara}"] },
            { "when": "output", "args": ["-o", "{output}"] },
            { "when": "dryRun", "args": ["--dry-run"] }
        ]),
        &[
            "schemaVersion",
            "source",
            "section",
            "paragraph",
            "ctrl",
            "fnPara",
            "props",
            "dryRun",
            "changedPages",
            "output",
            "outputFormat",
            "verify",
        ],
    );
}
