# Batch Convert Tool

A powerful CLI tool for batch converting HWP/HWPX files to multiple formats (PDF, PNG, SVG, Text) with parallel processing, progress tracking, and comprehensive error handling.

## Features

- **Parallel Processing**: Convert multiple files simultaneously using configurable number of workers
- **Multiple Output Formats**: Support for PDF, PNG, SVG, and Text exports
- **Progress Tracking**: Real-time progress monitoring with ETA calculation
- **Error Handling**: Robust error handling with detailed error reporting
- **Configuration File**: JSON-based configuration for all conversion options
- **File Pattern Filtering**: Regex-based file selection
- **Dry Run Mode**: Test conversions without writing files
- **Retry Logic**: Configurable retry attempts for failed conversions
- **Output Organization**: Automatic directory structure creation per format
- **Logging**: Comprehensive logging for debugging and auditing

## Installation

```bash
cd /c/Users/swsz9/rhwp
cargo build --release -p batch-convert
```

The compiled binary will be available at `target/release/batch-convert` (or `batch-convert.exe` on Windows).

## Usage

### Basic Usage

Convert all HWP files in a directory to PDF and text:

```bash
batch-convert \
  --input-dir ./documents \
  --output-dir ./output
```

### With Configuration File

Use a specific configuration file:

```bash
batch-convert \
  --input-dir ./documents \
  --output-dir ./output \
  --config ./config.pdf-only.json
```

### Parallel Processing

Specify the number of parallel workers:

```bash
batch-convert \
  --input-dir ./documents \
  --output-dir ./output \
  --jobs 8
```

### File Pattern Filtering

Only convert files matching a pattern:

```bash
batch-convert \
  --input-dir ./documents \
  --output-dir ./output \
  --pattern "^report_.*\.hwp$"
```

### Dry Run Mode

Test conversion without writing files:

```bash
batch-convert \
  --input-dir ./documents \
  --output-dir ./output \
  --dry-run
```

### Verbose Logging

Enable debug logging:

```bash
batch-convert \
  --input-dir ./documents \
  --output-dir ./output \
  --verbose
```

## Command-Line Options

```
USAGE:
    batch-convert [OPTIONS] --input-dir <INPUT_DIR> --output-dir <OUTPUT_DIR>

OPTIONS:
    -i, --input-dir <INPUT_DIR>      Input directory containing HWP/HWPX files
    -o, --output-dir <OUTPUT_DIR>    Output directory for converted files
    -c, --config <CONFIG>            Configuration file (JSON) for conversion options
    -j, --jobs <JOBS>                Number of parallel workers [default: 4]
    -p, --pattern <PATTERN>          File pattern filter (regex)
    --dry-run                        Dry run mode (no files written)
    -v, --verbose                    Enable verbose logging
    -h, --help                       Print help information
```

## Configuration Files

Configuration files are JSON-based and define all conversion options.

### Default Configuration Structure

```json
{
  "formats": {
    "pdf": true,
    "png": false,
    "svg": false,
    "text": true
  },
  "pdf": {
    "color": true,
    "compression": 6,
    "include_metadata": true,
    "include_bookmarks": true
  },
  "png": {
    "dpi": 300,
    "quality": 90,
    "background": "ffffff",
    "export_all_pages": true
  },
  "svg": {
    "preserve_viewbox": true,
    "embed_fonts": false,
    "text_to_paths": false,
    "separate_layers": false
  },
  "text": {
    "include_formatting": true,
    "include_tables": true,
    "include_headers_footers": true,
    "preserve_paragraphs": true,
    "line_ending": "unix"
  },
  "behavior": {
    "overwrite": true,
    "create_format_dirs": true,
    "collect_failed": false,
    "fail_fast": false,
    "max_retries": 3,
    "skip_existing": false
  }
}
```

### Provided Configuration Examples

#### 1. `config.default.json`
Default configuration converting to PDF and text formats.

#### 2. `config.pdf-only.json`
PDF-only conversion with maximum compression and error collection.

Features:
- PDF format enabled
- Compression level: 9 (maximum)
- Collects failed files for later inspection
- Doesn't overwrite existing files

#### 3. `config.images.json`
Image format conversions (PNG and SVG) with quality focus.

Features:
- PNG and SVG formats enabled
- High quality PNG (95/100)
- Embedded fonts in SVG
- Skips already converted files

#### 4. `config.all-formats.json`
Converts to all available formats (PDF, PNG, SVG, Text).

Features:
- All formats enabled
- Collects failed conversions
- Creates separate directories per format

#### 5. `config.high-quality-png.json`
High-resolution PNG export for archival or printing.

Features:
- PNG format only
- 600 DPI resolution
- Maximum quality (100/100)
- Retry failed conversions up to 5 times
- Skips existing files

## Configuration Options

### Format Options

#### PDF Options
- `color` (boolean): Enable color output
- `compression` (0-9): Compression level (6 is default, 9 is maximum)
- `include_metadata` (boolean): Include document metadata
- `include_bookmarks` (boolean): Include bookmarks

#### PNG Options
- `dpi` (number): Dots per inch (default: 300)
- `quality` (1-100): Output quality (default: 90)
- `background` (string): Background color as hex code (default: "ffffff")
- `export_all_pages` (boolean): Export all pages or just first

#### SVG Options
- `preserve_viewbox` (boolean): Preserve viewBox attribute
- `embed_fonts` (boolean): Embed fonts in output
- `text_to_paths` (boolean): Convert text to paths
- `separate_layers` (boolean): Export as separate layers

#### Text Options
- `include_formatting` (boolean): Preserve text formatting
- `include_tables` (boolean): Include table structure
- `include_headers_footers` (boolean): Include headers and footers
- `preserve_paragraphs` (boolean): Preserve paragraph breaks
- `line_ending` (string): Line ending style ("unix", "windows", "mac")

### Behavior Options
- `overwrite` (boolean): Overwrite existing files
- `create_format_dirs` (boolean): Create subdirectories per format
- `collect_failed` (boolean): Copy failed files to separate directory
- `fail_fast` (boolean): Stop on first error
- `max_retries` (number): Maximum retry attempts for failed conversions
- `skip_existing` (boolean): Skip already converted files

## Output Structure

By default, output is organized by format:

```
output/
├── pdf/
│   ├── document1.pdf
│   └── document2.pdf
├── png/
│   ├── document1.png
│   └── document2.png
├── svg/
│   ├── document1.svg
│   └── document2.svg
└── text/
    ├── document1.txt
    └── document2.txt
```

If `create_format_dirs` is disabled, all outputs go to the root output directory.

## Examples

### Example 1: Convert All Documents to PDF

```bash
batch-convert \
  --input-dir ./documents \
  --output-dir ./output \
  --config ./config.pdf-only.json \
  --jobs 8
```

### Example 2: Export High-Quality Images

```bash
batch-convert \
  --input-dir ./reports \
  --output-dir ./images \
  --config ./config.high-quality-png.json \
  --jobs 4
```

### Example 3: Process Specific Files

```bash
batch-convert \
  --input-dir ./archive \
  --output-dir ./processed \
  --pattern "^2024.*\.hwp$" \
  --jobs 6 \
  --verbose
```

### Example 4: Test Configuration Before Processing

```bash
batch-convert \
  --input-dir ./documents \
  --output-dir ./output \
  --config ./config.all-formats.json \
  --dry-run \
  --verbose
```

### Example 5: Process with Error Collection

```bash
batch-convert \
  --input-dir ./documents \
  --output-dir ./output \
  --config ./config.all-formats.json \
  --jobs 4 \
  --verbose
```

## Exit Codes

- `0`: Successful completion
- `1`: One or more files failed to convert

## Logging

The tool provides comprehensive logging at different levels:

- **INFO**: General progress and completion messages
- **DEBUG**: Detailed conversion progress and file operations
- **WARN**: Non-fatal errors in individual file conversions

Control logging with the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug batch-convert --input-dir ./documents --output-dir ./output
```

## Performance Tuning

### Parallel Workers

The `--jobs` parameter controls how many files are converted simultaneously:

- Recommended values: 2-16 depending on available CPU cores
- More workers = faster processing but higher resource usage
- Default: 4

### Memory Usage

Monitor memory usage when processing large batches. If memory usage is high:
- Reduce the number of parallel workers
- Process files in smaller batches
- Use separate output directories for each batch

## Error Handling

The tool provides detailed error reporting for failed conversions:

1. **Error Collection**: All errors are logged with details
2. **Error Summary**: Final report shows failed files and reasons
3. **Optional Error Directory**: With `collect_failed: true`, failed files are moved to a separate directory
4. **Retry Logic**: Failed conversions can be automatically retried

### Troubleshooting

- **Out of memory**: Reduce `--jobs` parameter
- **File permission errors**: Check input/output directory permissions
- **Conversion failures**: Check verbose logs with `--verbose` flag
- **Timeout issues**: Increase retry count in configuration

## Architecture

### Main Components

1. **main.rs**: CLI argument parsing and orchestration
2. **config.rs**: Configuration loading and validation
3. **converter.rs**: Core batch conversion logic with parallel processing
4. **progress.rs**: Progress tracking and ETA calculation
5. **error.rs**: Custom error types and handling

### Key Design Decisions

- **Rayon for Parallelism**: Uses Rayon for efficient parallel processing
- **Arc<Mutex<T>>**: Thread-safe error collection and progress tracking
- **Regex Patterns**: Flexible file filtering with regex patterns
- **Modular Config**: Separate configuration modules for each format
- **Dry Run Support**: Test conversions before actual processing

## Limitations

Current implementation includes placeholder conversion logic. To integrate with rhwp library:

1. Update `converter.rs` conversion methods to use rhwp's export functions
2. Handle rhwp-specific error types
3. Support streaming for large files
4. Add memory-mapped file handling for huge documents

## Future Enhancements

- [ ] Resume partial batches
- [ ] Real-time progress UI with progress bars
- [ ] Output validation
- [ ] Format-specific output verification
- [ ] Database logging of conversions
- [ ] Webhook notifications on completion
- [ ] S3/cloud storage support
- [ ] Distributed processing across multiple machines
- [ ] Template-based batch processing
- [ ] Advanced filtering by file size, modification date, etc.

## Contributing

Contributions are welcome! Please ensure:
- All tests pass
- Code follows Rust conventions
- Configuration changes are backward compatible
- Documentation is updated

## License

MIT License - See LICENSE file for details

## Support

For issues, questions, or suggestions:
1. Check existing documentation
2. Enable verbose logging (`--verbose`)
3. Run in dry-run mode to test configuration
4. Report issues with full error logs and configuration files
