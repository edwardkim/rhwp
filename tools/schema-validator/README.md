# RHWP Ingest JSON Schema Validator

A comprehensive JSON schema validator for RHWP ingest files with detailed error reporting that points to exact line and column positions of validation failures.

## Overview

This tool validates RHWP ingest JSON files against the official `ingest_schema_v1.json` schema before passing them to `rhwp build-from-ingest`. It provides:

- **Detailed error messages** with specific guidance on what's wrong
- **Line and column tracking** for precise error location identification
- **Multiple output formats** (human-readable and JSON)
- **Comprehensive validation** of all schema constraints:
  - Type validation (object, array, string, number, integer, boolean, null)
  - Required fields detection
  - Unknown field detection
  - Enum and const constraints
  - OneOf alternatives
  - Numeric ranges (minimum/maximum)
  - String length constraints
  - Schema references ($ref)

## Installation

The validator is a standalone Python 3 script with no external dependencies beyond the standard library.

```bash
# Make executable (Unix/Linux/WSL)
chmod +x tools/schema-validator/schema_validator.py
```

## Usage

### Basic Validation

```bash
python tools/schema-validator/schema_validator.py path/to/ingest.json
```

### With Custom Schema Path

```bash
python tools/schema-validator/schema_validator.py path/to/ingest.json \
  --schema path/to/custom_schema.json
```

### JSON Output (for scripting)

```bash
python tools/schema-validator/schema_validator.py path/to/ingest.json --json
```

Output:
```json
{
  "valid": false,
  "error_count": 2,
  "warning_count": 1,
  "errors": [
    {
      "level": "ERROR",
      "path": "questions[0].stem",
      "message": "Expected type string, got number",
      "code": "TYPE_MISMATCH",
      "position": "Line 15, Column 10"
    },
    ...
  ]
}
```

### Quiet Mode (exit code only)

```bash
python tools/schema-validator/schema_validator.py path/to/ingest.json --quiet
echo $?  # 0 = valid, 1 = errors
```

## Exit Codes

- **0**: File is valid (no errors)
- **1**: Validation failed (one or more errors)

## Error Types

### Common Error Codes

| Code | Meaning |
|------|---------|
| `INVALID_JSON` | File is not valid JSON |
| `TYPE_MISMATCH` | Value type doesn't match schema |
| `MISSING_REQUIRED_FIELD` | Required field is missing |
| `UNKNOWN_FIELD` | Field not allowed by schema |
| `CONST_MISMATCH` | Value doesn't match const requirement |
| `ENUM_MISMATCH` | Value not in allowed enum |
| `ONEOF_FAILED` | Value doesn't match any alternative |
| `BELOW_MINIMUM` | Number below minimum |
| `ABOVE_MAXIMUM` | Number above maximum |
| `STRING_TOO_SHORT` | String shorter than minLength |
| `STRING_TOO_LONG` | String longer than maxLength |

## Schema Structure

The RHWP ingest schema v1 defines:

- **version** (required): String constant "1"
- **page_size**: Object with width_mm and height_mm (defaults to A4)
- **default_font**: String (default: "함초롬바탕")
- **header_text**: Optional string
- **footer_text**: Optional string
- **form_label**: Optional string (e.g., "홀수형")
- **passages**: Array of Passage objects (shared question stems)
  - Each has `id` and optional `blocks` (array of StemBlock)
- **questions** (required): Array of Question objects
  - Required fields: `number`, `stem`, `choices`
  - Optional: `passage_ref`, `stem_blocks`, `media`, `auto_number`

### Block Types (StemBlock)

Blocks within passages, questions, and boxed elements can be:

1. **text** block
   - Required: `type` ("text"), `text` (string content)
   - Not allowed: `ref`, `placement`, `title`, `blocks`

2. **image** block
   - Required: `type` ("image"), `ref` (media ID)
   - Optional: `placement` (between|above|below|inline)
   - Not allowed: `text`, `title`, `blocks`

3. **boxed** block
   - Required: `type` ("boxed")
   - Optional: `title`, `blocks`
   - Not allowed: `text`, `ref`, `placement`

## Examples

### Minimal Valid File

```json
{
  "version": "1",
  "questions": [
    {
      "number": 1,
      "stem": "첫 번째 문제입니다.",
      "choices": [
        {"label": "①", "text": "선택지 1"},
        {"label": "②", "text": "선택지 2"}
      ]
    }
  ]
}
```

### With Passages and Media

```json
{
  "version": "1",
  "header_text": "국어 영역",
  "form_label": "홀수형",
  "passages": [
    {
      "id": "p1-2",
      "blocks": [
        {
          "type": "text",
          "text": "[1~2] 다음 글을 읽고 물음에 답하시오."
        }
      ]
    }
  ],
  "questions": [
    {
      "number": 1,
      "passage_ref": "p1-2",
      "stem": "다음 글의 주제는?",
      "stem_blocks": [
        {"type": "text", "text": "다음 글의 주제는?"},
        {
          "type": "image",
          "ref": "img/diagram.png",
          "placement": "below"
        }
      ],
      "choices": [
        {"label": "①", "text": "환경 보호"},
        {"label": "②", "text": "도시 생활"},
        {"label": "③", "text": "전통 음식"},
        {"label": "④", "text": "기술 발전"},
        {"label": "⑤", "text": "진로 탐색"}
      ],
      "media": [
        {
          "id": "img/diagram.png",
          "natural_w": 640,
          "natural_h": 480,
          "target_w_mm": 80.0
        }
      ]
    }
  ]
}
```

### Common Validation Errors

**Error: Missing version**
```
questions[0].stem: Required field 'version' is missing
```

**Error: Wrong type**
```
questions[0].number: Expected type integer, got string
```

**Error: Invalid block configuration**
```
questions[0].stem_blocks[1]: boxed 블록에 허용되지 않는 필드 'text' — blocks 배열의 text 블록으로 넣으세요
```

**Error: Unknown field**
```
questions[0]: Unknown field 'choice' is not allowed
```

## Integration

### In Build Scripts

```bash
#!/bin/bash
set -e

# Validate all ingest files
for ingest_file in *.ingest.json; do
    python tools/schema-validator/schema_validator.py "$ingest_file" || {
        echo "Validation failed for $ingest_file"
        exit 1
    }
done

# Proceed with build
rhwp build-from-ingest ...
```

### In CI/CD Pipelines

```yaml
# GitHub Actions example
- name: Validate ingest JSON files
  run: |
    python tools/schema-validator/schema_validator.py ingest.json --json \
      | python -m json.tool
```

### Programmatic Use

```python
from pathlib import Path
from schema_validator import IngestSchemaValidator

validator = IngestSchemaValidator(Path("schema/ingest_schema_v1.json"))
errors = validator.validate_file(Path("my_ingest.json"))

if not errors:
    print("Valid!")
else:
    for error in errors:
        print(error)
```

## Development

### Running Tests

```bash
python -m pytest tests/test_schema_validator.py -v
```

### Adding Custom Validation Rules

Extend `IngestSchemaValidator` class and override `_validate_value()`:

```python
class CustomValidator(IngestSchemaValidator):
    def _validate_value(self, value, schema, path, positions_map):
        super()._validate_value(value, schema, path, positions_map)

        # Add custom logic
        if path.startswith("questions["):
            # Custom question validation
            pass
```

## Limitations

- Line/column positions are approximate (calculated during JSON parsing)
- Does not validate media file existence or accessibility
- Does not validate image dimensions or format
- Does not cross-validate passage_ref IDs with actual passages
- Does not validate media[].id paths against filesystem

## See Also

- [ingest_schema_v1.json](schema/ingest_schema_v1.json) - Complete JSON Schema
- [Sample Files](../rhwp-ingest/schema/) - Example ingest files
- `rhwp build-from-ingest --help` - Usage of the build command
