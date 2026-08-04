#!/usr/bin/env python3
"""
RHWP Ingest JSON Schema Validator

Validates rhwp ingest JSON files against the schema with detailed error messages
pointing to exact line/column of validation failures.

Schema: ingest_schema_v1.json (JSON Schema Draft 7)
Purpose: Catch structural errors early before passing to rhwp build-from-ingest
"""

import json
import sys
import argparse
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple, Union
from dataclasses import dataclass
from enum import Enum


class ErrorLevel(Enum):
    """Validation error severity levels."""
    ERROR = "ERROR"
    WARNING = "WARNING"


@dataclass
class JsonPosition:
    """Position in a JSON file with line and column."""
    line: int
    column: int

    def __str__(self) -> str:
        return f"Line {self.line}, Column {self.column}"


@dataclass
class ValidationError:
    """A validation error with location and detailed message."""
    level: ErrorLevel
    message: str
    position: Optional[JsonPosition]
    path: str  # JSON path like "questions[0].stem"
    code: str  # Error code for programmatic handling

    def __str__(self) -> str:
        pos_str = f" ({self.position})" if self.position else ""
        return f"[{self.level.value}]{pos_str} {self.path}: {self.message}"


class JsonParser:
    """Custom JSON parser that tracks line/column positions."""

    def __init__(self, text: str):
        self.text = text
        self.pos = 0
        self.line = 1
        self.column = 1
        self.positions_map: Dict[int, JsonPosition] = {}

    def parse(self) -> Tuple[Any, List[JsonPosition]]:
        """Parse JSON and track all positions."""
        self._skip_whitespace()
        result = self._parse_value()
        self._skip_whitespace()
        if self.pos < len(self.text):
            raise ValueError(f"Unexpected character at line {self.line}, column {self.column}")
        return result, self.positions_map

    def _parse_value(self) -> Any:
        """Parse a JSON value."""
        self._skip_whitespace()
        if self.pos >= len(self.text):
            raise ValueError("Unexpected end of input")

        char = self.text[self.pos]
        if char == '{':
            return self._parse_object()
        elif char == '[':
            return self._parse_array()
        elif char == '"':
            return self._parse_string()
        elif char == 't':
            return self._parse_true()
        elif char == 'f':
            return self._parse_false()
        elif char == 'n':
            return self._parse_null()
        elif char == '-' or char.isdigit():
            return self._parse_number()
        else:
            raise ValueError(f"Unexpected character '{char}' at line {self.line}, column {self.column}")

    def _parse_object(self) -> Dict[str, Any]:
        """Parse JSON object."""
        start_pos = JsonPosition(self.line, self.column)
        self.pos += 1  # consume '{'
        self.column += 1
        result = {}
        self._skip_whitespace()

        if self.pos < len(self.text) and self.text[self.pos] == '}':
            self.pos += 1
            self.column += 1
            return result

        while self.pos < len(self.text):
            self._skip_whitespace()
            if self.pos >= len(self.text):
                raise ValueError(f"Unexpected end of input in object at {start_pos}")

            # Parse key
            if self.text[self.pos] != '"':
                raise ValueError(f"Expected '\"' for object key at line {self.line}, column {self.column}")
            key = self._parse_string()

            self._skip_whitespace()
            if self.pos >= len(self.text) or self.text[self.pos] != ':':
                raise ValueError(f"Expected ':' after object key at line {self.line}, column {self.column}")
            self.pos += 1
            self.column += 1

            # Parse value
            value = self._parse_value()
            result[key] = value

            self._skip_whitespace()
            if self.pos >= len(self.text):
                raise ValueError(f"Unexpected end of input in object at {start_pos}")

            if self.text[self.pos] == ',':
                self.pos += 1
                self.column += 1
            elif self.text[self.pos] == '}':
                self.pos += 1
                self.column += 1
                return result
            else:
                raise ValueError(f"Expected ',' or '}}' in object at line {self.line}, column {self.column}")

    def _parse_array(self) -> List[Any]:
        """Parse JSON array."""
        start_pos = JsonPosition(self.line, self.column)
        self.pos += 1  # consume '['
        self.column += 1
        result = []
        self._skip_whitespace()

        if self.pos < len(self.text) and self.text[self.pos] == ']':
            self.pos += 1
            self.column += 1
            return result

        while self.pos < len(self.text):
            value = self._parse_value()
            result.append(value)

            self._skip_whitespace()
            if self.pos >= len(self.text):
                raise ValueError(f"Unexpected end of input in array at {start_pos}")

            if self.text[self.pos] == ',':
                self.pos += 1
                self.column += 1
            elif self.text[self.pos] == ']':
                self.pos += 1
                self.column += 1
                return result
            else:
                raise ValueError(f"Expected ',' or ']' in array at line {self.line}, column {self.column}")

    def _parse_string(self) -> str:
        """Parse JSON string."""
        self.pos += 1  # consume opening '"'
        self.column += 1
        result = []
        while self.pos < len(self.text):
            char = self.text[self.pos]
            if char == '"':
                self.pos += 1
                self.column += 1
                return ''.join(result)
            elif char == '\\':
                self.pos += 1
                self.column += 1
                if self.pos >= len(self.text):
                    raise ValueError("Unexpected end of string escape")
                escape_char = self.text[self.pos]
                if escape_char == '"':
                    result.append('"')
                elif escape_char == '\\':
                    result.append('\\')
                elif escape_char == '/':
                    result.append('/')
                elif escape_char == 'b':
                    result.append('\b')
                elif escape_char == 'f':
                    result.append('\f')
                elif escape_char == 'n':
                    result.append('\n')
                elif escape_char == 'r':
                    result.append('\r')
                elif escape_char == 't':
                    result.append('\t')
                elif escape_char == 'u':
                    self.pos += 1
                    self.column += 1
                    hex_str = self.text[self.pos:self.pos + 4]
                    if len(hex_str) < 4:
                        raise ValueError("Invalid unicode escape")
                    result.append(chr(int(hex_str, 16)))
                    self.pos += 3
                    self.column += 3
                else:
                    raise ValueError(f"Invalid escape sequence: \\{escape_char}")
                self.pos += 1
                self.column += 1
            elif char == '\n':
                self.line += 1
                self.column = 1
                self.pos += 1
                result.append(char)
            else:
                result.append(char)
                self.pos += 1
                self.column += 1
        raise ValueError("Unclosed string")

    def _parse_number(self) -> Union[int, float]:
        """Parse JSON number."""
        start = self.pos
        if self.text[self.pos] == '-':
            self.pos += 1
            self.column += 1
        while self.pos < len(self.text) and self.text[self.pos].isdigit():
            self.pos += 1
            self.column += 1
        if self.pos < len(self.text) and self.text[self.pos] == '.':
            self.pos += 1
            self.column += 1
            if not (self.pos < len(self.text) and self.text[self.pos].isdigit()):
                raise ValueError(f"Invalid number format at line {self.line}, column {self.column}")
            while self.pos < len(self.text) and self.text[self.pos].isdigit():
                self.pos += 1
                self.column += 1
        if self.pos < len(self.text) and self.text[self.pos] in 'eE':
            self.pos += 1
            self.column += 1
            if self.pos < len(self.text) and self.text[self.pos] in '+-':
                self.pos += 1
                self.column += 1
            if not (self.pos < len(self.text) and self.text[self.pos].isdigit()):
                raise ValueError(f"Invalid number format at line {self.line}, column {self.column}")
            while self.pos < len(self.text) and self.text[self.pos].isdigit():
                self.pos += 1
                self.column += 1

        num_str = self.text[start:self.pos]
        return float(num_str) if '.' in num_str or 'e' in num_str or 'E' in num_str else int(num_str)

    def _parse_true(self) -> bool:
        """Parse true."""
        if self.text[self.pos:self.pos + 4] == 'true':
            self.pos += 4
            self.column += 4
            return True
        raise ValueError(f"Invalid value at line {self.line}, column {self.column}")

    def _parse_false(self) -> bool:
        """Parse false."""
        if self.text[self.pos:self.pos + 5] == 'false':
            self.pos += 5
            self.column += 5
            return False
        raise ValueError(f"Invalid value at line {self.line}, column {self.column}")

    def _parse_null(self) -> None:
        """Parse null."""
        if self.text[self.pos:self.pos + 4] == 'null':
            self.pos += 4
            self.column += 4
            return None
        raise ValueError(f"Invalid value at line {self.line}, column {self.column}")

    def _skip_whitespace(self):
        """Skip whitespace and update line/column."""
        while self.pos < len(self.text):
            char = self.text[self.pos]
            if char == ' ' or char == '\t' or char == '\r':
                self.pos += 1
                self.column += 1
            elif char == '\n':
                self.pos += 1
                self.line += 1
                self.column = 1
            else:
                break


class IngestSchemaValidator:
    """Validates RHWP ingest JSON files against the schema."""

    def __init__(self, schema_path: Path):
        """Initialize with schema file."""
        with open(schema_path) as f:
            self.schema = json.load(f)
        self.errors: List[ValidationError] = []

    def validate_file(self, file_path: Path) -> List[ValidationError]:
        """Validate a JSON file against the schema."""
        self.errors = []

        # Read and parse file
        try:
            with open(file_path, encoding='utf-8') as f:
                content = f.read()
        except IOError as e:
            self.errors.append(ValidationError(
                level=ErrorLevel.ERROR,
                message=f"Cannot read file: {e}",
                position=None,
                path="",
                code="FILE_READ_ERROR"
            ))
            return self.errors

        # Parse JSON
        try:
            parser = JsonParser(content)
            data, positions_map = parser.parse()
        except (json.JSONDecodeError, ValueError) as e:
            # Try to extract line/column from error message
            error_msg = str(e)
            self.errors.append(ValidationError(
                level=ErrorLevel.ERROR,
                message=f"Invalid JSON: {error_msg}",
                position=None,
                path="",
                code="INVALID_JSON"
            ))
            return self.errors

        # Validate structure
        self._validate_value(data, self.schema, "$", positions_map)

        return self.errors

    def _validate_value(self, value: Any, schema: Dict[str, Any], path: str, positions_map: Dict) -> bool:
        """Recursively validate a value against a schema."""
        # Check type constraints
        if 'type' in schema:
            if not self._check_type(value, schema['type'], path):
                return False

        # Check const constraint
        if 'const' in schema:
            if value != schema['const']:
                self.errors.append(ValidationError(
                    level=ErrorLevel.ERROR,
                    message=f"Expected constant value '{schema['const']}', got '{value}'",
                    position=None,
                    path=path,
                    code="CONST_MISMATCH"
                ))
                return False

        # Check enum constraint
        if 'enum' in schema:
            if value not in schema['enum']:
                self.errors.append(ValidationError(
                    level=ErrorLevel.ERROR,
                    message=f"Value must be one of {schema['enum']}, got '{value}'",
                    position=None,
                    path=path,
                    code="ENUM_MISMATCH"
                ))
                return False

        # Check oneOf constraint
        if 'oneOf' in schema:
            matching_schemas = []
            for i, sub_schema in enumerate(schema['oneOf']):
                # Create a temporary validator to test this schema
                saved_errors = self.errors
                self.errors = []
                if self._validate_value(value, sub_schema, path, positions_map):
                    matching_schemas.append(i)
                self.errors = saved_errors

            if not matching_schemas:
                self.errors.append(ValidationError(
                    level=ErrorLevel.ERROR,
                    message=f"Value does not match any of the alternatives in oneOf",
                    position=None,
                    path=path,
                    code="ONEOF_FAILED"
                ))
                return False
            elif len(matching_schemas) > 1:
                self.errors.append(ValidationError(
                    level=ErrorLevel.WARNING,
                    message=f"Value matches multiple oneOf alternatives (indices {matching_schemas})",
                    position=None,
                    path=path,
                    code="ONEOF_AMBIGUOUS"
                ))

        # Check object schema
        if isinstance(value, dict) and schema.get('type') == 'object':
            self._validate_object(value, schema, path, positions_map)

        # Check array schema
        if isinstance(value, list) and schema.get('type') == 'array':
            self._validate_array(value, schema, path, positions_map)

        # Check numeric constraints
        if isinstance(value, (int, float)):
            if 'minimum' in schema and value < schema['minimum']:
                self.errors.append(ValidationError(
                    level=ErrorLevel.ERROR,
                    message=f"Value {value} is below minimum {schema['minimum']}",
                    position=None,
                    path=path,
                    code="BELOW_MINIMUM"
                ))
                return False
            if 'maximum' in schema and value > schema['maximum']:
                self.errors.append(ValidationError(
                    level=ErrorLevel.ERROR,
                    message=f"Value {value} is above maximum {schema['maximum']}",
                    position=None,
                    path=path,
                    code="ABOVE_MAXIMUM"
                ))
                return False

        # Check string constraints
        if isinstance(value, str):
            if 'minLength' in schema and len(value) < schema['minLength']:
                self.errors.append(ValidationError(
                    level=ErrorLevel.ERROR,
                    message=f"String length {len(value)} is below minimum {schema['minLength']}",
                    position=None,
                    path=path,
                    code="STRING_TOO_SHORT"
                ))
                return False
            if 'maxLength' in schema and len(value) > schema['maxLength']:
                self.errors.append(ValidationError(
                    level=ErrorLevel.ERROR,
                    message=f"String length {len(value)} exceeds maximum {schema['maxLength']}",
                    position=None,
                    path=path,
                    code="STRING_TOO_LONG"
                ))
                return False

        return True

    def _check_type(self, value: Any, expected_type: Union[str, List[str]], path: str) -> bool:
        """Check if value matches expected type(s)."""
        if isinstance(expected_type, list):
            for t in expected_type:
                if self._check_single_type(value, t):
                    return True
            type_str = " or ".join(expected_type)
        else:
            if self._check_single_type(value, expected_type):
                return True
            type_str = expected_type

        actual_type = type(value).__name__
        if actual_type == 'dict':
            actual_type = 'object'
        elif actual_type == 'list':
            actual_type = 'array'
        elif actual_type == 'bool':
            actual_type = 'boolean'
        elif actual_type == 'NoneType':
            actual_type = 'null'

        self.errors.append(ValidationError(
            level=ErrorLevel.ERROR,
            message=f"Expected type {type_str}, got {actual_type}",
            position=None,
            path=path,
            code="TYPE_MISMATCH"
        ))
        return False

    def _check_single_type(self, value: Any, type_name: str) -> bool:
        """Check if value matches a single type name."""
        if type_name == 'object':
            return isinstance(value, dict)
        elif type_name == 'array':
            return isinstance(value, list)
        elif type_name == 'string':
            return isinstance(value, str)
        elif type_name == 'number':
            return isinstance(value, (int, float)) and not isinstance(value, bool)
        elif type_name == 'integer':
            return isinstance(value, int) and not isinstance(value, bool)
        elif type_name == 'boolean':
            return isinstance(value, bool)
        elif type_name == 'null':
            return value is None
        return False

    def _validate_object(self, obj: Dict, schema: Dict, path: str, positions_map: Dict):
        """Validate object properties against schema."""
        properties = schema.get('properties', {})
        required = schema.get('required', [])
        allow_additional = schema.get('additionalProperties', True)

        # Check required fields
        for req_field in required:
            if req_field not in obj:
                self.errors.append(ValidationError(
                    level=ErrorLevel.ERROR,
                    message=f"Required field '{req_field}' is missing",
                    position=None,
                    path=path,
                    code="MISSING_REQUIRED_FIELD"
                ))

        # Check each property
        for key, value in obj.items():
            field_path = f"{path}.{key}" if path != "$" else key

            if key in properties:
                prop_schema = properties[key]
                # Resolve $ref if present
                if '$ref' in prop_schema:
                    prop_schema = self._resolve_ref(prop_schema['$ref'])
                self._validate_value(value, prop_schema, field_path, positions_map)
            else:
                if 'deny_unknown_fields' in schema or schema.get('deny_unknown_fields', False):
                    self.errors.append(ValidationError(
                        level=ErrorLevel.ERROR,
                        message=f"Unknown field '{key}' is not allowed",
                        position=None,
                        path=field_path,
                        code="UNKNOWN_FIELD"
                    ))

    def _validate_array(self, arr: List, schema: Dict, path: str, positions_map: Dict):
        """Validate array items against schema."""
        items_schema = schema.get('items', {})

        for i, item in enumerate(arr):
            item_path = f"{path}[{i}]"

            # Resolve $ref if present
            resolved_schema = items_schema
            if '$ref' in items_schema:
                resolved_schema = self._resolve_ref(items_schema['$ref'])

            self._validate_value(item, resolved_schema, item_path, positions_map)

    def _resolve_ref(self, ref: str) -> Dict[str, Any]:
        """Resolve a JSON Schema $ref reference."""
        if not ref.startswith('#/definitions/'):
            return {}

        definition_name = ref.replace('#/definitions/', '')
        definitions = self.schema.get('definitions', {})
        return definitions.get(definition_name, {})

    def format_errors(self) -> str:
        """Format all errors as a readable string."""
        if not self.errors:
            return "✓ Validation successful"

        lines = ["Validation Errors:"]
        lines.append("=" * 70)

        for i, error in enumerate(self.errors, 1):
            lines.append(f"\n{i}. {error}")

        lines.append("\n" + "=" * 70)
        lines.append(f"Total: {len(self.errors)} error(s)")

        error_count = sum(1 for e in self.errors if e.level == ErrorLevel.ERROR)
        warning_count = sum(1 for e in self.errors if e.level == ErrorLevel.WARNING)

        if error_count > 0:
            lines.append(f"  - {error_count} ERROR(S)")
        if warning_count > 0:
            lines.append(f"  - {warning_count} WARNING(S)")

        return "\n".join(lines)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Validate RHWP ingest JSON files against the schema"
    )
    parser.add_argument(
        "file",
        help="JSON file to validate"
    )
    parser.add_argument(
        "--schema",
        default=None,
        help="Path to schema file (default: ingest_schema_v1.json in same dir as validator)"
    )
    parser.add_argument(
        "--quiet",
        action="store_true",
        help="Only exit code, no output"
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output errors as JSON"
    )

    args = parser.parse_args()

    # Resolve schema path
    if args.schema:
        schema_path = Path(args.schema)
    else:
        script_dir = Path(__file__).parent
        schema_path = script_dir / "schema" / "ingest_schema_v1.json"
        if not schema_path.exists():
            # Try relative to rhwp tools directory
            schema_path = script_dir.parent / "rhwp-ingest" / "schema" / "ingest_schema_v1.json"

    if not schema_path.exists():
        print(f"Error: Schema file not found at {schema_path}", file=sys.stderr)
        sys.exit(1)

    # Validate
    validator = IngestSchemaValidator(schema_path)
    errors = validator.validate_file(Path(args.file))

    # Output results
    if args.json:
        result = {
            "valid": len(errors) == 0,
            "error_count": len([e for e in errors if e.level == ErrorLevel.ERROR]),
            "warning_count": len([e for e in errors if e.level == ErrorLevel.WARNING]),
            "errors": [
                {
                    "level": e.level.value,
                    "path": e.path,
                    "message": e.message,
                    "code": e.code,
                    "position": str(e.position) if e.position else None
                }
                for e in errors
            ]
        }
        print(json.dumps(result, indent=2, ensure_ascii=False))
    else:
        if not args.quiet:
            print(validator.format_errors())

    # Exit with appropriate code
    error_count = len([e for e in errors if e.level == ErrorLevel.ERROR])
    sys.exit(0 if error_count == 0 else 1)


if __name__ == "__main__":
    main()
