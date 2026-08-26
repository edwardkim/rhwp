use super::{tool, tool_with_optional_args};

pub(super) fn extend(tools: &mut Vec<serde_json::Value>) {
    tools.extend([
        tool_with_optional_args(
            "hwp_delete_text",
            "[#5011] 문단 좌표에서 글자를 지운다. 주소는 search 와 같다. 코어 delete_text_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "offset": { "type": "integer", "minimum": 0 },
                    "count": { "type": "integer", "minimum": 1 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "count"],
            }),
            "edit",
            serde_json::json!(["edit", "delete-text", "{path}", "--count", "{count}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "offset", "count", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_delete_paragraph",
            "[#5012] 지정 문단을 지운다. 구역의 마지막 문단은 코어가 거부한다. 코어 delete_paragraph_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path"],
            }),
            "edit",
            serde_json::json!(["edit", "delete-paragraph", "{path}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_insert_endnote",
            "[#5013] 문단 좌표에 미주를 끼운다. 주소는 search 와 같다. 코어 insert_endnote_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "offset": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-endnote", "{path}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "offset", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_merge_paragraph",
            "[#5018] 지정 문단을 바로 앞 문단에 합친다. para 는 합쳐질 문단(1 이상). 코어 merge_paragraph_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 1 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path"],
            }),
            "edit",
            serde_json::json!(["edit", "merge-paragraph", "{path}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_delete_footnote",
            "[#5017] 본문 각주/미주 컨트롤을 지운다. section/para/ctrl 은 0 기준. 코어 delete_footnote_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "ctrl": { "type": "integer", "minimum": 0, "description": "문단 안 컨트롤 인덱스 (0부터)" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "section", "paragraph", "ctrl"],
            }),
            "edit",
            serde_json::json!(["edit", "delete-footnote", "{path}", "--section", "{section}", "--para", "{paragraph}", "--ctrl", "{ctrl}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "ctrl", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_delete_text_in_footnote",
            "각주/미주 문단에서 글자를 지운다. section/para/ctrl/fnPara/offset 은 0 기준. 코어 delete_text_in_footnote_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "ctrl": { "type": "integer", "minimum": 0, "description": "문단 안 각주/미주 컨트롤 인덱스 (0부터)" },
                    "fnPara": { "type": "integer", "minimum": 0, "description": "각주/미주 안 문단 인덱스 (0부터)" },
                    "offset": { "type": "integer", "minimum": 0 },
                    "count": { "type": "integer", "minimum": 1, "description": "지울 글자 수 (1 이상)" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "count"],
            }),
            "edit",
            serde_json::json!(["edit", "delete-text-in-footnote", "{path}", "--count", "{count}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "ctrl", "args": ["--ctrl", "{ctrl}"] },
                { "when": "fnPara", "args": ["--fn-para", "{fnPara}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "ctrl", "fnPara", "offset", "count", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_group_shapes",
            "같은 구역의 도형/그림을 묶는다. targets 는 \"para,ctrl;para,ctrl\" (2개 이상). 코어 group_shapes_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "targets": { "type": "string", "description": "para,ctrl;para,ctrl 형식의 도형 좌표 목록" },
                    "section": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "targets"],
            }),
            "edit",
            serde_json::json!(["edit", "group-shapes", "{path}", "--targets", "{targets}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "ctrl", "count", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_page_def",
            "구역의 용지 설정(PageDef)을 바꾼다. --props 는 width/height/marginLeft 등 HWPUNIT JSON. 코어 set_page_def_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "props": { "type": "string", "description": "용지 설정 JSON (HWPUNIT)" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "props"],
            }),
            "edit",
            serde_json::json!(["edit", "set-page-def", "{path}", "--props", "{props}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "props", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_section_def",
            "구역 정의(SectionDef)를 바꾼다. --props 는 hideHeader/columnSpacing/pageNum 등 JSON. 코어 set_section_def_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "props": { "type": "string", "description": "구역 정의 JSON (예: {\"hideHeader\":true})" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "props"],
            }),
            "edit",
            serde_json::json!(["edit", "set-section-def", "{path}", "--props", "{props}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "props", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_add_bookmark",
            "[#5026] 지정 좌표에 책갈피를 넣는다. 같은 이름은 거부. 코어 add_bookmark_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "name": {
                        "type": "string",
                        "pattern": r".*\S.*",
                        "description": "책갈피 이름 (공백만으로 구성할 수 없음)"
                    },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "offset": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "name"],
            }),
            "edit",
            serde_json::json!(["edit", "add-bookmark", "{path}", "--name", "{name}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "offset", "name", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_delete_bookmark",
            "[#5027] 책갈피 컨트롤을 지운다. section/para/ctrl 은 0 기준. 코어 delete_bookmark_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "ctrl": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "section", "paragraph", "ctrl"],
            }),
            "edit",
            serde_json::json!(["edit", "delete-bookmark", "{path}", "--section", "{section}", "--para", "{paragraph}", "--ctrl", "{ctrl}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "ctrl", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_rename_bookmark",
            "[#5033] 책갈피 이름을 바꾼다. section/para/ctrl 은 0 기준. 코어 rename_bookmark_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "ctrl": { "type": "integer", "minimum": 0 },
                    "name": { "type": "string", "description": "새 책갈피 이름 (빈 문자열 불가)" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "section", "paragraph", "ctrl", "name"],
            }),
            "edit",
            serde_json::json!(["edit", "rename-bookmark", "{path}", "--section", "{section}", "--para", "{paragraph}", "--ctrl", "{ctrl}", "--name", "{name}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "ctrl", "name", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_delete_header_footer",
            "[#5039] 머리말/꼬리말 컨트롤을 지운다. --header 또는 --footer 필수. applyTo 는 0 양쪽·1 짝수·2 홀수. 코어 delete_header_footer_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "header": { "type": "boolean" },
                    "footer": { "type": "boolean" },
                    "applyTo": { "type": "integer", "minimum": 0, "maximum": 2, "description": "0 양쪽 / 1 짝수 / 2 홀수" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path"],
            }),
            "edit",
            serde_json::json!(["edit", "delete-header-footer", "{path}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "header", "args": ["--header"] },
                { "when": "footer", "args": ["--footer"] },
                { "when": "applyTo", "args": ["--apply-to", "{applyTo}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "isHeader", "applyTo", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_header_footer_text",
            "기존 머리말/꼬리말 문단 텍스트를 통째로 바꾼다. --header 또는 --footer 필수. applyTo 는 0 양쪽·1 짝수·2 홀수. 코어 delete_text_in_header_footer_native + insert_text_in_header_footer_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "text": { "type": "string", "description": "바꿀 문자열 (빈 문자열 불가)" },
                    "section": { "type": "integer", "minimum": 0 },
                    "header": { "type": "boolean" },
                    "footer": { "type": "boolean" },
                    "applyTo": { "type": "integer", "minimum": 0, "maximum": 2, "description": "0 양쪽 / 1 짝수 / 2 홀수" },
                    "paragraph": { "type": "integer", "minimum": 0, "description": "머리말/꼬리말 안 문단 인덱스 (0부터)" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "text"],
            }),
            "edit",
            serde_json::json!(["edit", "set-header-footer-text", "{path}", "--text", "{text}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "header", "args": ["--header"] },
                { "when": "footer", "args": ["--footer"] },
                { "when": "applyTo", "args": ["--apply-to", "{applyTo}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "isHeader", "applyTo", "paragraph", "text", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_delete_table",
            "[#5028] 본문 최상위 표를 지운다. 좌표는 export-tables 의 index. 코어 delete_table_control_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table"],
            }),
            "edit",
            serde_json::json!(["edit", "delete-table", "{path}", "--table", "{table}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_insert_header_footer_text",
            "기존 머리말/꼬리말 문단에 텍스트를 끼운다. --header 또는 --footer 필수. applyTo 는 0 양쪽·1 짝수·2 홀수. 코어 insert_text_in_header_footer_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "text": { "type": "string", "description": "넣을 문자열 (빈 문자열 불가)" },
                    "section": { "type": "integer", "minimum": 0 },
                    "header": { "type": "boolean" },
                    "footer": { "type": "boolean" },
                    "applyTo": { "type": "integer", "minimum": 0, "maximum": 2, "description": "0 양쪽 / 1 짝수 / 2 홀수" },
                    "paragraph": { "type": "integer", "minimum": 0, "description": "머리말/꼬리말 안 문단 인덱스 (0부터)" },
                    "offset": { "type": "integer", "minimum": 0, "description": "문단 안 문자 오프셋 (0부터)" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "text"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-header-footer-text", "{path}", "--text", "{text}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "header", "args": ["--header"] },
                { "when": "footer", "args": ["--footer"] },
                { "when": "applyTo", "args": ["--apply-to", "{applyTo}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "isHeader", "applyTo", "paragraph", "offset", "text", "insertedChars", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_insert_header_footer",
            "[#5036] 머리말/꼬리말을 만든다. 같은 applyTo 가 있으면 거부. 코어 create_header_footer_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "header": { "type": "boolean", "description": "머리말을 만든다 (footer 와 동시에 쓰지 않는다)" },
                    "footer": { "type": "boolean", "description": "꼬리말을 만든다 (header 와 동시에 쓰지 않는다)" },
                    "section": { "type": "integer", "minimum": 0 },
                    "applyTo": { "type": "integer", "minimum": 0, "maximum": 2, "description": "0 양쪽, 1 짝수, 2 홀수" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path"],
                "oneOf": [
                    {
                        "properties": {
                            "header": { "const": true },
                            "footer": { "not": { "const": true } }
                        },
                        "required": ["header"]
                    },
                    {
                        "properties": {
                            "header": { "not": { "const": true } },
                            "footer": { "const": true }
                        },
                        "required": ["footer"]
                    }
                ],
            }),
            "edit",
            serde_json::json!(["edit", "insert-header-footer", "{path}", "--json"]),
            serde_json::json!([
                { "when": "header", "args": ["--header"] },
                { "when": "footer", "args": ["--footer"] },
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "applyTo", "args": ["--apply-to", "{applyTo}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "isHeader", "applyTo", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_insert_field_in_hf",
            "기존 머리말/꼬리말 문단에 필드 마커를 넣는다. --header 또는 --footer 필수. fieldType 1 쪽번호 / 2 총쪽수 / 3 파일이름. 코어 insert_field_in_hf_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "header": { "type": "boolean", "description": "머리말에 삽입 (footer 와 동시에 쓰지 않는다)" },
                    "footer": { "type": "boolean", "description": "꼬리말에 삽입 (header 와 동시에 쓰지 않는다)" },
                    "fieldType": { "type": "integer", "minimum": 1, "maximum": 3, "description": "1 쪽번호 / 2 총쪽수 / 3 파일이름" },
                    "section": { "type": "integer", "minimum": 0 },
                    "applyTo": { "type": "integer", "minimum": 0, "maximum": 2, "description": "0 양쪽 / 1 짝수 / 2 홀수" },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "offset": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "fieldType"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-field-in-hf", "{path}", "--field-type", "{fieldType}", "--json"]),
            serde_json::json!([
                { "when": "header", "args": ["--header"] },
                { "when": "footer", "args": ["--footer"] },
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "applyTo", "args": ["--apply-to", "{applyTo}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "isHeader", "applyTo", "paragraph", "offset", "fieldType", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_column_def",
            "[#5081] 구역의 단(다단) 정의를 바꾼다. count 는 단 수. type 0 일반 / 1 배분 / 2 평행. 코어 set_column_def_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "count": { "type": "integer", "minimum": 1, "description": "단 수" },
                    "section": { "type": "integer", "minimum": 0 },
                    "columnType": { "type": "integer", "minimum": 0, "maximum": 2, "description": "0 일반 / 1 배분 / 2 평행" },
                    "sameWidth": { "type": "boolean", "description": "단 너비 동일 (기본 true)" },
                    "spacing": { "type": "integer", "description": "단 간격 (HWPUNIT)" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "count"],
            }),
            "edit",
            serde_json::json!(["edit", "set-column-def", "{path}", "--count", "{count}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "columnType", "args": ["--type", "{columnType}"] },
                { "when": "sameWidth", "args": ["--same-width"] },
                { "when": "spacing", "args": ["--spacing", "{spacing}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "columnCount", "columnType", "sameWidth", "spacing", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_delete_hf_text",
            "기존 머리말/꼬리말 문단에서 글자를 지운다. --header 또는 --footer 와 --count 필수. applyTo 는 0 양쪽·1 짝수·2 홀수. 코어 delete_text_in_header_footer_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "count": { "type": "integer", "minimum": 1, "description": "지울 글자 수 (1 이상)" },
                    "section": { "type": "integer", "minimum": 0 },
                    "header": { "type": "boolean" },
                    "footer": { "type": "boolean" },
                    "applyTo": { "type": "integer", "minimum": 0, "maximum": 2, "description": "0 양쪽 / 1 짝수 / 2 홀수" },
                    "paragraph": { "type": "integer", "minimum": 0, "description": "머리말/꼬리말 안 문단 인덱스 (0부터)" },
                    "offset": { "type": "integer", "minimum": 0, "description": "문단 안 문자 오프셋 (0부터)" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "count"],
            }),
            "edit",
            serde_json::json!(["edit", "delete-hf-text", "{path}", "--count", "{count}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "header", "args": ["--header"] },
                { "when": "footer", "args": ["--footer"] },
                { "when": "applyTo", "args": ["--apply-to", "{applyTo}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "isHeader", "applyTo", "paragraph", "offset", "count", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_split_paragraph_in_hf",
            "기존 머리말/꼬리말 문단을 오프셋에서 나눈다. --header 또는 --footer 필수. applyTo 는 0 양쪽·1 짝수·2 홀수. 코어 split_paragraph_in_header_footer_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "header": { "type": "boolean" },
                    "footer": { "type": "boolean" },
                    "applyTo": { "type": "integer", "minimum": 0, "maximum": 2, "description": "0 양쪽 / 1 짝수 / 2 홀수" },
                    "paragraph": { "type": "integer", "minimum": 0, "description": "머리말/꼬리말 안 문단 인덱스 (0부터)" },
                    "offset": { "type": "integer", "minimum": 0, "description": "문단 안 문자 오프셋 (0부터)" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path"],
            }),
            "edit",
            serde_json::json!(["edit", "split-paragraph-in-hf", "{path}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "header", "args": ["--header"] },
                { "when": "footer", "args": ["--footer"] },
                { "when": "applyTo", "args": ["--apply-to", "{applyTo}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "isHeader", "applyTo", "paragraph", "offset", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_merge_paragraph_in_hf",
            "머리말/꼬리말 문단을 바로 앞 문단과 합친다. --header 또는 --footer 필수. --para 는 합쳐질 문단(1 이상). applyTo 는 0 양쪽·1 짝수·2 홀수. 코어 merge_paragraph_in_header_footer_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "header": { "type": "boolean" },
                    "footer": { "type": "boolean" },
                    "applyTo": { "type": "integer", "minimum": 0, "maximum": 2, "description": "0 양쪽 / 1 짝수 / 2 홀수" },
                    "paragraph": { "type": "integer", "minimum": 1, "description": "합쳐질 머리말/꼬리말 문단 인덱스 (1부터)" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path"],
            }),
            "edit",
            serde_json::json!(["edit", "merge-paragraph-in-hf", "{path}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "header", "args": ["--header"] },
                { "when": "footer", "args": ["--footer"] },
                { "when": "applyTo", "args": ["--apply-to", "{applyTo}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "isHeader", "applyTo", "paragraph", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_split_paragraph_in_cell",
            "표 셀 문단을 오프셋에서 나눈다. export-tables 격자와 같은 --table/--row/--col. 코어 split_paragraph_in_cell_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "row": { "type": "integer", "minimum": 0 },
                    "col": { "type": "integer", "minimum": 0 },
                    "cellPara": { "type": "integer", "minimum": 0 },
                    "offset": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "row", "col"],
            }),
            "edit",
            serde_json::json!(["edit", "split-paragraph-in-cell", "{path}", "--table", "{table}", "--row", "{row}", "--col", "{col}", "--json"]),
            serde_json::json!([
                { "when": "cellPara", "args": ["--cell-para", "{cellPara}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "col", "paragraph", "offset", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_merge_paragraph_in_cell",
            "표 셀 문단을 바로 앞 문단과 합친다. --cell-para 는 합쳐질 문단(1 이상). export-tables 격자와 같은 --table/--row/--col. 코어 merge_paragraph_in_cell_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "row": { "type": "integer", "minimum": 0 },
                    "col": { "type": "integer", "minimum": 0 },
                    "cellPara": { "type": "integer", "minimum": 1 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "row", "col"],
            }),
            "edit",
            serde_json::json!(["edit", "merge-paragraph-in-cell", "{path}", "--table", "{table}", "--row", "{row}", "--col", "{col}", "--json"]),
            serde_json::json!([
                { "when": "cellPara", "args": ["--cell-para", "{cellPara}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "col", "paragraph", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_split_paragraph",
            "[#5082] 본문 문단을 지정 오프셋에서 가른다. 주소는 search 와 같다. 코어 split_paragraph_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "offset": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path"],
            }),
            "edit",
            serde_json::json!(["edit", "split-paragraph", "{path}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "offset", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_page_hide",
            "[#5083] 문단에 쪽 감추기(PageHide) 컨트롤을 넣거나 갱신한다. 플래그를 모두 끄면 제거. 코어 set_page_hide_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "hideHeader": { "type": "boolean" },
                    "hideFooter": { "type": "boolean" },
                    "hideMasterPage": { "type": "boolean" },
                    "hideBorder": { "type": "boolean" },
                    "hideFill": { "type": "boolean" },
                    "hidePageNum": { "type": "boolean" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path"],
            }),
            "edit",
            serde_json::json!(["edit", "set-page-hide", "{path}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "hideHeader", "args": ["--hide-header"] },
                { "when": "hideFooter", "args": ["--hide-footer"] },
                { "when": "hideMasterPage", "args": ["--hide-master"] },
                { "when": "hideBorder", "args": ["--hide-border"] },
                { "when": "hideFill", "args": ["--hide-fill"] },
                { "when": "hidePageNum", "args": ["--hide-page-num"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "hideHeader", "hideFooter", "hideMasterPage", "hideBorder", "hideFill", "hidePageNum", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_transpose_table",
            "[#5108] 본문 최상위 표의 행/열을 제자리에서 바꾼다. 병합 셀이 있으면 거부. 좌표는 export-tables 의 index. 코어 transpose_table_cells_in_place_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table"],
            }),
            "edit",
            serde_json::json!(["edit", "transpose-table", "{path}", "--table", "{table}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "section", "paragraph", "ctrl", "sourceRows", "sourceCols", "targetRows", "targetCols", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
    ]);
}
