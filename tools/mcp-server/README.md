# rhwp MCP Server

**Model Context Protocol (MCP) server for rhwp CLI** — enables AI agents to read, analyze, edit, and convert Korean HWP/HWPX documents through a standardized interface.

## Overview

This MCP server exposes rhwp's powerful CLI capabilities to AI assistants like Claude Desktop, allowing them to:

- **Read** HWP/HWPX documents (metadata, text, tables, structure)
- **Search** within documents with page-level precision
- **Edit** documents (fill form fields, replace text, modify table cells)
- **Convert** between HWP/HWPX formats
- **Export** to various formats (SVG, PNG, PDF, text, structured JSON)
- **Batch process** multiple documents in parallel

## Features

### MCP Tools (15 commands)

The server provides 15 tools that map directly to rhwp CLI commands:

#### Reading & Analysis
- `hwp_info` - Get document metadata (format, version, pages, fonts)
- `hwp_fields` - List all form fields (누름틀) with properties
- `hwp_search` - Search text with page numbers and context
- `hwp_export_text` - Extract plain text by page
- `hwp_export_tables` - Extract tables as structured JSON (preserves cell merging)
- `hwp_export_structure` - Extract document outline/clause hierarchy

#### Editing
- `hwp_fill_fields` - Fill form fields with values (supports dry-run)
- `hwp_replace_text` - Replace text throughout document
- `hwp_set_cell` - Set table cell values by coordinates

#### Conversion & Export
- `hwp_convert` - Convert HWP/HWPX to editable HWP
- `hwp_export_hwpx` - Convert HWP to HWPX (ZIP+XML)
- `hwp_export_svg` - Export pages to SVG
- `hwp_export_png` - Export pages to PNG (VLM-ready)
- `hwp_export_pdf` - Export to PDF

#### Batch Processing
- `hwp_batch` - Process multiple files in parallel with any read command

### MCP Resources (3 documents)

Documentation exposed as MCP resources:

- `rhwp://docs/cli-commands` - Complete CLI reference
- `rhwp://docs/agents` - AI agent task playbook
- `rhwp://docs/json-pipeline` - JSON pipeline guide

## Installation

### Prerequisites

1. **rhwp CLI** - Install from source or binary
   ```bash
   # Build from source (requires Rust)
   cd /path/to/rhwp
   cargo build --release

   # Binary will be at: ./target/release/rhwp
   ```

2. **Node.js** - Version 18.0.0 or higher
   ```bash
   node --version  # Should be >= 18.0.0
   ```

### Install MCP Server

```bash
cd tools/mcp-server
npm install
```

## Configuration

### Environment Variables

Configure the server using environment variables:

```bash
# Path to rhwp binary (default: 'rhwp' from PATH)
export RHWP_BINARY_PATH="/path/to/rhwp/target/release/rhwp"

# Command timeout in milliseconds (default: 60000 = 1 minute)
export RHWP_TIMEOUT=60000

# Maximum output size in bytes (default: 10485760 = 10MB)
export RHWP_MAX_OUTPUT_SIZE=10485760
```

### Claude Desktop Configuration

Add to your Claude Desktop configuration file:

**macOS/Linux**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "rhwp": {
      "command": "node",
      "args": ["/absolute/path/to/rhwp/tools/mcp-server/index.js"],
      "env": {
        "RHWP_BINARY_PATH": "/absolute/path/to/rhwp/target/release/rhwp"
      }
    }
  }
}
```

**Important**: Use absolute paths for both the MCP server and rhwp binary.

## Usage

### Basic Examples

Once configured in Claude Desktop, you can ask Claude to:

**Read document information:**
```
Check the metadata of document.hwp
```

**Search within documents:**
```
Search for "위임전결" in document.hwp and show me which pages it appears on
```

**Extract and analyze tables:**
```
Extract all tables from report.hwp and show me the structure
```

**Fill form fields:**
```
Fill the form fields in application.hwp:
- 성명: 홍길동
- 부서: 기술팀
- 날짜: 2026-08-04
Save as filled_application.hwp
```

**Replace text:**
```
Replace all occurrences of "2025년" with "2026년" in document.hwp
```

**Batch processing:**
```
Get metadata for all HWP files in the docs/ directory
```

### Advanced Workflows

**Form automation pipeline:**
```
1. List all fields in template.hwp
2. Fill fields with provided data
3. Export the filled document to PDF
```

**Document analysis:**
```
1. Search for specific terms in a document
2. Extract the pages where terms appear as images
3. Analyze table data on those pages
```

**Batch conversion:**
```
Convert all HWP files in a directory to HWPX format with verification
```

## Tool Schemas

### Example: hwp_search

```json
{
  "name": "hwp_search",
  "description": "Search for text in HWP/HWPX document",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": { "type": "string", "description": "Path to file" },
      "query": { "type": "string", "description": "Search query" },
      "ignoreCase": { "type": "boolean", "default": false },
      "limit": { "type": "number", "description": "Max matches" }
    },
    "required": ["path", "query"]
  }
}
```

### Example: hwp_fill_fields

```json
{
  "name": "hwp_fill_fields",
  "description": "Fill form fields in document",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": { "type": "string" },
      "data": {
        "type": "object",
        "additionalProperties": { "type": "string" }
      },
      "output": { "type": "string" },
      "dryRun": { "type": "boolean", "default": false }
    },
    "required": ["path", "data"]
  }
}
```

## Error Handling

The server follows rhwp's exit code contract:

- **0** - Success
- **1** - Runtime failure (file not found, parsing error, etc.)
- **2** - Usage error (invalid arguments)
- **3** - IR difference detected (in verify mode)
- **4** - Page count mismatch (in verify-pages mode)

Errors are returned as JSON with:
```json
{
  "error": true,
  "exitCode": 1,
  "message": "Error description"
}
```

## Testing

### Manual Testing

Test the server directly using Node.js:

```bash
# Set environment variables
export RHWP_BINARY_PATH="/path/to/rhwp"

# Run server (connects via stdio)
node tools/mcp-server/index.js
```

### Test Script

Run the included test script:

```bash
npm test
```

This validates:
- rhwp binary accessibility
- Basic command execution
- JSON parsing
- Error handling

## Development

### Project Structure

```
tools/mcp-server/
├── index.js           # Main MCP server implementation
├── package.json       # Node.js package configuration
├── README.md          # This file
├── README_EN.md       # English version
├── test.js            # Test suite
└── config.example.json # Configuration example
```

### Adding New Tools

To add a new rhwp command as an MCP tool:

1. Add tool definition to `TOOLS` array
2. Implement handler in `TOOL_HANDLERS` object
3. Update documentation

Example:
```javascript
// 1. Tool definition
{
  name: 'hwp_new_command',
  description: 'Description of the command',
  inputSchema: {
    type: 'object',
    properties: {
      path: { type: 'string', description: 'File path' }
    },
    required: ['path']
  }
}

// 2. Handler implementation
async hwp_new_command({ path }) {
  const result = await executeRhwp(['new-command', path, '--json']);
  return parseRhwpJson(result.stdout, result.exitCode);
}
```

## Troubleshooting

### Server not appearing in Claude Desktop

1. Check configuration file location and JSON syntax
2. Verify absolute paths are used
3. Restart Claude Desktop
4. Check Claude Desktop logs

### Command execution fails

1. Verify `RHWP_BINARY_PATH` points to the correct binary
2. Test rhwp directly: `rhwp --version`
3. Check file paths are accessible
4. Review timeout settings for large files

### JSON parsing errors

1. Ensure you're using `--json` compatible commands
2. Check rhwp CLI version compatibility
3. Verify output isn't truncated (check `RHWP_MAX_OUTPUT_SIZE`)

## Limitations

- **File size**: Large files may timeout (adjust `RHWP_TIMEOUT`)
- **Output size**: Limited by `RHWP_MAX_OUTPUT_SIZE` (default 10MB)
- **Concurrent requests**: Handled sequentially by MCP protocol
- **Binary features**: `export-png` requires `native-skia` feature

## Security Considerations

- **File access**: The server can read/write files accessible to the Node.js process
- **Command injection**: Arguments are passed as array to `spawn()`, preventing shell injection
- **Path validation**: File existence is checked before execution
- **Sandboxing**: Run in restricted environment for production use

## Contributing

Contributions are welcome! Please:

1. Follow existing code style
2. Add tests for new features
3. Update documentation
4. Test with Claude Desktop

## Related Documentation

- [rhwp CLI Commands](../../mydocs/manual/cli_commands.md)
- [Agent Task Playbook](../../mydocs/manual/agent_task_playbook.md)
- [CLI JSON Pipeline Guide](../../mydocs/manual/cli_json_pipeline_guide.md)
- [MCP Specification](https://modelcontextprotocol.io/)

## License

MIT License - see [LICENSE](../../LICENSE) for details.

## Support

- GitHub Issues: [edwardkim/rhwp/issues](https://github.com/edwardkim/rhwp/issues)
- Documentation: [rhwp docs](https://edwardkim.github.io/rhwp/)

---

**Version**: 0.1.0
**Last Updated**: 2026-08-04
