use super::{tool, tool_with_optional_args};

pub(super) fn extend(tools: &mut Vec<serde_json::Value>) {
    tools.extend([
        tool_with_optional_args(
            "hwp_insert_image",
            "[#3719 §6-5] 도장·서명 같은 그림을 쪽 좌표에 붙여 새 문서를 만든다 — 채워 넣은 서식에 직인을 얹는 실물 제출의 마지막 조각. **길이 단위는 전부 HWPUNIT(1/7200 inch)이며 픽셀이 아니다** (A4 세로 = 59528 × 84188). 용지 왼쪽 위 모서리 기준 (x, y) 에 놓는 떠 있는 그림이다. 크기를 생략하면 원본 픽셀을 96dpi 로 환산하고, 한쪽만 주면 원본 비율을 지킨다. 쪽 밖으로 나가면 자르지 않고 overflow 로 보고한다. 지원 형식은 png·jpg·jpeg·bmp·tif·tiff 이며 그 밖은 인자 오류다. 산출물은 입력 형식을 따른다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "입력 HWP/HWPX 문서 경로" },
                    "image": { "type": "string", "description": "삽입할 그림 파일 경로 (png/jpg/jpeg/bmp/tif/tiff)" },
                    "page": { "type": "integer", "minimum": 0, "description": "붙일 쪽 (0부터). 생략하면 첫 쪽" },
                    "x": { "type": "integer", "minimum": 0, "description": "용지 왼쪽 모서리에서의 가로 위치 (HWPUNIT, 1/7200 inch). 생략하면 0" },
                    "y": { "type": "integer", "minimum": 0, "description": "용지 위쪽 모서리에서의 세로 위치 (HWPUNIT, 1/7200 inch). 생략하면 0" },
                    "width": { "type": "integer", "minimum": 1, "description": "그림 너비 (HWPUNIT, 1/7200 inch). 생략하면 원본 픽셀 × 75" },
                    "height": { "type": "integer", "minimum": 1, "description": "그림 높이 (HWPUNIT, 1/7200 inch). 생략하면 원본 픽셀 × 75" },
                    "output": { "type": "string", "description": "출력 파일 경로. 생략하면 <입력명>_image.hwp (HWPX 입력이면 _image.hwpx)" },
                    "dryRun": { "type": "boolean", "description": "true 면 파일을 쓰지 않고 배치 예정만 보고" }
                },
                "required": ["path", "image"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-image", "{path}", "--image", "{image}", "--json"]),
            serde_json::json!([
                { "when": "page", "args": ["--page", "{page}"] },
                { "when": "x", "args": ["--x", "{x}"] },
                { "when": "y", "args": ["--y", "{y}"] },
                { "when": "width", "args": ["--width", "{width}"] },
                { "when": "height", "args": ["--height", "{height}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "image", "page", "x", "y", "width", "height", "binDataId", "dryRun", "overflow", "output", "outputFormat", "verify", "changedPages"],
        ),
        tool_with_optional_args(
            "hwp_insert_picture",
            "문단 좌표(search 와 같은 section/paragraph/offset, 0 기준)에 본문 그림을 끼운다. 도장·서명용 쪽 좌표 insert-image 와 다르다. 코어 insert_picture_native 배선. 그림 바이트는 파일 그대로 넘긴다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "입력 HWP/HWPX 문서 경로" },
                    "image": { "type": "string", "description": "삽입할 그림 파일 경로 (png/jpg/jpeg/bmp/tif/tiff)" },
                    "section": { "type": "integer", "minimum": 0, "description": "구역 번호 (0부터). 생략하면 0" },
                    "paragraph": { "type": "integer", "minimum": 0, "description": "문단 번호 (0부터). 생략하면 0" },
                    "offset": { "type": "integer", "minimum": 0, "description": "문단 안 문자 오프셋 (0부터). 생략하면 0" },
                    "x": { "type": "integer", "minimum": 0, "description": "용지 가로 위치 (HWPUNIT). 생략하면 0" },
                    "y": { "type": "integer", "minimum": 0, "description": "용지 세로 위치 (HWPUNIT). 생략하면 0" },
                    "width": { "type": "integer", "minimum": 1, "description": "그림 너비 (HWPUNIT). 생략하면 원본 픽셀 × 75" },
                    "height": { "type": "integer", "minimum": 1, "description": "그림 높이 (HWPUNIT). 생략하면 원본 픽셀 × 75" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "image"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-picture", "{path}", "--image", "{image}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "x", "args": ["--x", "{x}"] },
                { "when": "y", "args": ["--y", "{y}"] },
                { "when": "width", "args": ["--width", "{width}"] },
                { "when": "height", "args": ["--height", "{height}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "image", "section", "paragraph", "offset", "x", "y", "width", "height", "binDataId", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_delete_picture",
            "본문 그림 컨트롤을 지운다. section/para/ctrl 은 0 기준. 코어 delete_picture_control_native 배선.",
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
            serde_json::json!(["edit", "delete-picture", "{path}", "--section", "{section}", "--para", "{paragraph}", "--ctrl", "{ctrl}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "ctrl", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_delete_shape",
            "본문 도형 컨트롤을 지운다. section/para/ctrl 은 0 기준. 코어 delete_shape_control_native 배선.",
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
            serde_json::json!(["edit", "delete-shape", "{path}", "--section", "{section}", "--para", "{paragraph}", "--ctrl", "{ctrl}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "ctrl", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_ungroup_shape",
            "본문 GroupShape 를 풀어 자식 개체를 되돌린다. section/para/ctrl 은 0 기준. 코어 ungroup_shape_native 배선.",
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
            serde_json::json!(["edit", "ungroup-shape", "{path}", "--section", "{section}", "--para", "{paragraph}", "--ctrl", "{ctrl}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "ctrl", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_insert_text",
            "[#4990] 문단 좌표에 새 텍스트를 삽입해 새 문서를 만든다 — 기존 문자열을 바꾸는 replace-text/fill-fields/set-cell 과 달리, **없는 자리에 글자를 넣는** 축. 주소는 search 와 같다(section/paragraph/offset, 전부 0 기준). offset 이 문단 문자 수와 같으면 끝에 붙이고, 넘으면 인자 오류다. 빈 문자열은 거부한다. 산출물은 입력 형식을 따른다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "입력 HWP/HWPX 문서 경로" },
                    "text": { "type": "string", "description": "넣을 문자열 (빈 문자열 불가)" },
                    "section": { "type": "integer", "minimum": 0, "description": "구역 번호 (0부터). 생략하면 0" },
                    "paragraph": { "type": "integer", "minimum": 0, "description": "문단 번호 (0부터, 해당 구역). 생략하면 0" },
                    "offset": { "type": "integer", "minimum": 0, "description": "문단 안 문자 오프셋 (0부터). 문단 길이와 같으면 끝에 붙인다. 생략하면 0" },
                    "output": { "type": "string", "description": "출력 파일 경로. 생략하면 <입력명>_inserted.hwp (HWPX 입력이면 _inserted.hwpx)" },
                    "dryRun": { "type": "boolean", "description": "true 면 파일을 쓰지 않고 삽입 예정만 보고" }
                },
                "required": ["path", "text"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-text", "{path}", "--text", "{text}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "offset", "text", "insertedChars", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_insert_paragraph",
            "[#4992] 지정한 자리에 빈 문단을 끼워 새 문서를 만든다. 앞 문단의 서식을 상속한다(한글에서 Enter). para 가 구역 문단 수와 같으면 끝에 붙인다. 산출물은 입력 형식을 따른다.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "입력 HWP/HWPX 문서 경로" },
                    "section": { "type": "integer", "minimum": 0, "description": "구역 번호 (0부터). 생략하면 0" },
                    "paragraph": { "type": "integer", "minimum": 0, "description": "끼울 문단 번호 (0부터). 구역 문단 수와 같으면 끝에 붙인다. 생략하면 0" },
                    "output": { "type": "string", "description": "출력 파일 경로. 생략하면 <입력명>_paragraph.hwp" },
                    "dryRun": { "type": "boolean", "description": "true 면 파일을 쓰지 않고 삽입 예정만 보고" }
                },
                "required": ["path"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-paragraph", "{path}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_insert_page_break",
            "[#4993] 문단을 지정 오프셋에서 가르고 새 문단에 쪽 나눔을 넣는다. 코어 insert_page_break_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "입력 HWP/HWPX 문서 경로" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "offset": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-page-break", "{path}", "--json"]),
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
            "hwp_insert_column_break",
            "[#5019] 문단을 지정 오프셋에서 가르고 새 문단에 단 나눔을 넣는다. 코어 insert_column_break_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "입력 HWP/HWPX 문서 경로" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "offset": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-column-break", "{path}", "--json"]),
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
            "hwp_insert_table",
            "본문 좌표에 빈 표를 만든다. rows/cols 는 1 이상, 열은 256 이하. section/para/offset 은 0 기준. 코어 create_table_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "rows": { "type": "integer", "minimum": 1 },
                    "cols": { "type": "integer", "minimum": 1, "maximum": 256 },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "offset": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "rows", "cols"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-table", "{path}", "--rows", "{rows}", "--cols", "{cols}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "offset", "rows", "cols", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_numbering_restart",
            "문단 번호 매기기를 다시 시작한다. mode 0=해제, 1=이전 이어, 2=새 시작. 코어 set_numbering_restart_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "mode": { "type": "integer", "minimum": 0, "description": "0=해제, 1=이전 이어, 2=새 시작" },
                    "count": { "type": "integer", "minimum": 0, "description": "mode=2 일 때 시작 번호" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "mode"],
            }),
            "edit",
            serde_json::json!(["edit", "set-numbering-restart", "{path}", "--mode", "{mode}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "count", "args": ["--count", "{count}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "count", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_insert_row",
            "[#4994] 본문 최상위 표에 행을 끼운다. 좌표는 export-tables 의 index. below 면 지정 행 아래, 아니면 위. 코어 insert_table_row_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "row": { "type": "integer", "minimum": 0 },
                    "below": { "type": "boolean" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "row"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-row", "{path}", "--table", "{table}", "--row", "{row}", "--json"]),
            serde_json::json!([
                { "when": "below", "args": ["--below"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "below", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_insert_col",
            "[#4995] 본문 최상위 표에 열을 끼운다. 좌표는 export-tables 의 index. right 면 지정 열 오른쪽, 아니면 왼쪽. 코어 insert_table_column_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "col": { "type": "integer", "minimum": 0 },
                    "right": { "type": "boolean" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "col"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-col", "{path}", "--table", "{table}", "--col", "{col}", "--json"]),
            serde_json::json!([
                { "when": "right", "args": ["--right"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "col", "right", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_delete_row",
            "[#4996] 본문 최상위 표에서 행을 지운다. 좌표는 export-tables 의 index. 코어 delete_table_row_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "row": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "row"],
            }),
            "edit",
            serde_json::json!(["edit", "delete-row", "{path}", "--table", "{table}", "--row", "{row}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_merge_cells",
            "[#4997] 본문 최상위 표의 셀 사각형을 병합한다. 좌표는 export-tables 의 index. 코어 merge_table_cells_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "row": { "type": "integer", "minimum": 0 },
                    "col": { "type": "integer", "minimum": 0 },
                    "endRow": { "type": "integer", "minimum": 0 },
                    "endCol": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "row", "col", "endRow", "endCol"],
            }),
            "edit",
            serde_json::json!(["edit", "merge-cells", "{path}", "--table", "{table}", "--row", "{row}", "--col", "{col}", "--end-row", "{endRow}", "--end-col", "{endCol}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "col", "endRow", "endCol", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_insert_footnote",
            "[#4998] 문단 좌표에 각주를 끼운다. 주소는 search 와 같다(section/paragraph/offset, 전부 0 기준). 코어 insert_footnote_native 배선.",
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
            serde_json::json!(["edit", "insert-footnote", "{path}", "--json"]),
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
            "hwp_insert_equation",
            "본문 문단 좌표에 수식을 끼운다. 표 셀/글상자 내부는 미지원. 코어 insert_equation_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "script": { "type": "string" },
                    "section": { "type": "integer", "minimum": 0 },
                    "paragraph": { "type": "integer", "minimum": 0 },
                    "offset": { "type": "integer", "minimum": 0 },
                    "fontSize": { "type": "integer", "minimum": 1 },
                    "color": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "script"],
            }),
            "edit",
            serde_json::json!(["edit", "insert-equation", "{path}", "--script", "{script}", "--json"]),
            serde_json::json!([
                { "when": "section", "args": ["--section", "{section}"] },
                { "when": "paragraph", "args": ["--para", "{paragraph}"] },
                { "when": "offset", "args": ["--offset", "{offset}"] },
                { "when": "fontSize", "args": ["--font-size", "{fontSize}"] },
                { "when": "color", "args": ["--color", "{color}"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "offset", "script", "fontSize", "color", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_delete_col",
            "[#5009] 본문 최상위 표에서 열을 지운다. 좌표는 export-tables 의 index. 코어 delete_table_column_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "col": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "col"],
            }),
            "edit",
            serde_json::json!(["edit", "delete-col", "{path}", "--table", "{table}", "--col", "{col}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "col", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_split_cell",
            "[#5010] 본문 최상위 표의 병합 셀을 다시 나눈다. 좌표는 export-tables 의 index. 코어 split_table_cell_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "row": { "type": "integer", "minimum": 0 },
                    "col": { "type": "integer", "minimum": 0 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "row", "col"],
            }),
            "edit",
            serde_json::json!(["edit", "split-cell", "{path}", "--table", "{table}", "--row", "{row}", "--col", "{col}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "col", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_split_cell_into",
            "본문 최상위 표의 셀을 n행 × m열로 나눈다. 좌표는 export-tables 의 index. 코어 split_table_cell_into_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "row": { "type": "integer", "minimum": 0 },
                    "col": { "type": "integer", "minimum": 0 },
                    "rows": { "type": "integer", "minimum": 1, "description": "나눌 행 수" },
                    "cols": { "type": "integer", "minimum": 1, "description": "나눌 열 수" },
                    "equalRowHeight": { "type": "boolean", "description": "나눈 행 높이를 같게" },
                    "mergeFirst": { "type": "boolean", "description": "병합 셀이면 먼저 해제" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "row", "col", "rows", "cols"],
            }),
            "edit",
            serde_json::json!(["edit", "split-cell-into", "{path}", "--table", "{table}", "--row", "{row}", "--col", "{col}", "--rows", "{rows}", "--cols", "{cols}", "--json"]),
            serde_json::json!([
                { "when": "equalRowHeight", "args": ["--equal-row-height"] },
                { "when": "mergeFirst", "args": ["--merge-first"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "col", "rows", "cols", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_split_table",
            "본문 최상위 표를 지정 행에서 둘로 나눈다. --row 는 뒤 표가 시작되는 행(1 이상). 코어 split_table_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "row": { "type": "integer", "minimum": 1 },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "row"],
            }),
            "edit",
            serde_json::json!(["edit", "split-table", "{path}", "--table", "{table}", "--row", "{row}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_fit_table",
            "본문 최상위 표를 페이지 본문 폭에 맞춰 비례 축소한다. 이미 폭 안이면 그대로 둔다. 코어 fit_table_to_page_native 배선.",
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
            serde_json::json!(["edit", "fit-table", "{path}", "--table", "{table}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_resize_table",
            "본문 최상위 표의 행/열 크기를 한 걸음(283 HWPUNIT) 조절한다. 병합 칸이 있으면 네이티브가 거부한다. 코어 resize_table_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "row": { "type": "integer", "minimum": 0 },
                    "col": { "type": "integer", "minimum": 0 },
                    "vertical": { "type": "boolean", "description": "참이면 행 높이, 거짓이면 열 폭" },
                    "forward": { "type": "boolean", "description": "참이면 늘리고, 거짓이면 줄인다" },
                    "line": { "type": "boolean", "description": "참이면 경계만 옮긴다(이웃과 짝)" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "row", "col"],
            }),
            "edit",
            serde_json::json!(["edit", "resize-table", "{path}", "--table", "{table}", "--row", "{row}", "--col", "{col}", "--json"]),
            serde_json::json!([
                { "when": "vertical", "args": ["--vertical"] },
                { "when": "forward", "args": ["--forward"] },
                { "when": "line", "args": ["--line"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "col", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_resize_table_cell",
            "본문 최상위 표의 한 칸 크기를 한 걸음(283 HWPUNIT) 조절한다. 병합 칸이 있으면 네이티브가 거부한다. 코어 resize_table_cell_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "row": { "type": "integer", "minimum": 0 },
                    "col": { "type": "integer", "minimum": 0 },
                    "vertical": { "type": "boolean", "description": "참이면 행 높이, 거짓이면 열 폭" },
                    "forward": { "type": "boolean", "description": "참이면 늘리고, 거짓이면 줄인다" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "row", "col"],
            }),
            "edit",
            serde_json::json!(["edit", "resize-table-cell", "{path}", "--table", "{table}", "--row", "{row}", "--col", "{col}", "--json"]),
            serde_json::json!([
                { "when": "vertical", "args": ["--vertical"] },
                { "when": "forward", "args": ["--forward"] },
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "col", "vertical", "forward", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_cell_props",
            "본문 최상위 표 셀 속성을 고친다. 좌표는 hwp_export_tables 의 index·격자. 코어 set_cell_properties_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "row": { "type": "integer", "minimum": 0 },
                    "col": { "type": "integer", "minimum": 0 },
                    "props": { "type": "string", "description": "셀 속성 JSON (예: {\"verticalAlign\":1})" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "row", "col", "props"],
            }),
            "edit",
            serde_json::json!(["edit", "set-cell-props", "{path}", "--table", "{table}", "--row", "{row}", "--col", "{col}", "--props", "{props}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "row", "col", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_table_props",
            "본문 최상위 표 속성(칸간격·여백·글자처럼·배치 등)을 고친다. 표 번호는 hwp_export_tables 의 index. 코어 set_table_properties_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "props": { "type": "string", "description": "표 속성 JSON (예: {\"cellSpacing\":200})" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "props"],
            }),
            "edit",
            serde_json::json!(["edit", "set-table-props", "{path}", "--table", "{table}", "--props", "{props}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_move_table",
            "본문 최상위 표의 위치 오프셋을 옮긴다. dx/dy 는 HWPUNIT(양수=오른쪽/아래). 코어 move_table_offset_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "dx": { "type": "integer", "description": "가로 이동량 (HWPUNIT)" },
                    "dy": { "type": "integer", "description": "세로 이동량 (HWPUNIT)" },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "dx", "dy"],
            }),
            "edit",
            serde_json::json!(["edit", "move-table", "{path}", "--table", "{table}", "--dx", "{dx}", "--dy", "{dy}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "dx", "dy", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_merge_table",
            "본문 최상위 표에 바로 다음 표를 이어 붙인다. 사이에는 빈 문단만 허용. 코어 merge_table_with_next_native 배선.",
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
            serde_json::json!(["edit", "merge-table", "{path}", "--table", "{table}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_set_column_widths",
            "본문 최상위 표의 열 폭(HWPUNIT)을 절대값으로 설정한다. 개수는 열 수와 같아야 한다. 코어 set_table_column_widths_native 배선.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "table": { "type": "integer", "minimum": 0 },
                    "widths": {
                        "type": "array",
                        "items": { "type": "integer", "minimum": 1 },
                        "minItems": 1
                    },
                    "output": { "type": "string" },
                    "dryRun": { "type": "boolean" }
                },
                "required": ["path", "table", "widths"],
            }),
            "edit",
            serde_json::json!(["edit", "set-column-widths", "{path}", "--table", "{table}", "--widths", "{widths}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "table", "widths", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
        tool_with_optional_args(
            "hwp_delete_equation",
            "본문 수식 컨트롤을 지운다. section/para/ctrl 은 0 기준. 코어 delete_equation_control_native 배선.",
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
            serde_json::json!(["edit", "delete-equation", "{path}", "--section", "{section}", "--para", "{paragraph}", "--ctrl", "{ctrl}", "--json"]),
            serde_json::json!([
                { "when": "output", "args": ["-o", "{output}"] },
                { "when": "dryRun", "args": ["--dry-run"] }
            ]),
            &["schemaVersion", "source", "section", "paragraph", "ctrl", "dryRun", "changedPages", "output", "outputFormat", "verify"],
        ),
    ]);
}
