# HWP Metadata Extraction Tool

A comprehensive Python tool for extracting and exporting document metadata from HWP (Hangul Word Processor) files in both HWP5 and HWPX formats.

## Features

### Metadata Extraction

- **File Information**
  - File path, name, size (bytes and MB)
  - File format detection (HWP5 vs HWPX)
  - File modification timestamp

- **HWP5 Specific (OLE Format)**
  - File header information and version
  - Document header flags (compression, encryption, DRM, digital signature, etc.)
  - Summary information stream (HwpSummaryInformation)
  - Document properties and statistics
  - Generator/application detection
  - Creation and modification date hints
  - All embedded streams listing

- **HWPX Specific (ZIP Format)**
  - Manifest information
  - Content metadata (author, title, subject, description)
  - Creator and last saved by information
  - Creation and modification dates
  - Generator/application information
  - Document structure and file listing
  - Settings and configuration details

### Output Formats

- **JSON**: Hierarchical, detailed metadata structure (default)
- **CSV**: Flattened structure for spreadsheet analysis
- **Both**: Generate both formats simultaneously

## Installation

### Requirements

- Python 3.7+
- Standard library only (for HWPX)
- Optional: `olefile` for HWP5 support

### Install olefile (Optional but Recommended)

```bash
pip install olefile
```

## Usage

### Basic Usage

Extract metadata from a single HWP file (outputs JSON):

```bash
python metadata_extract.py document.hwp
```

### Advanced Usage

#### Extract to specific format

```bash
# JSON format (default)
python metadata_extract.py document.hwp --format json

# CSV format
python metadata_extract.py document.hwp --format csv

# Both formats
python metadata_extract.py document.hwp --format both
```

#### Specify output directory

```bash
python metadata_extract.py document.hwp --output ./exports
python metadata_extract.py document.hwp -o /path/to/output
```

#### Process multiple files

```bash
# Multiple files
python metadata_extract.py file1.hwp file2.hwp file3.hwpx

# Using glob patterns
python metadata_extract.py *.hwp
python metadata_extract.py **/*.hwp  # Recursive
```

#### Complete example

```bash
python metadata_extract.py documents/*.hwp --format both --output ./metadata_export
```

## Output

### JSON Output

Individual file: `document_metadata.json`
Combined output: `metadata_combined.json`

Example structure:
```json
{
  "file_info": {
    "file_path": "/path/to/document.hwp",
    "file_name": "document.hwp",
    "file_size_bytes": 1024000,
    "file_size_mb": 1.0,
    "file_format": "HWP5",
    "file_modified": "2024-01-15T10:30:45.123456"
  },
  "hwp5_header": {
    "signature": "HWP Document File",
    "version": "5.0.2.0",
    "flags": {
      "compressed": true,
      "encrypted": false,
      "digital_signature": false,
      ...
    }
  },
  "hwp5_summary": {
    "generator": "Hangul (HWP)",
    "extracted_fields": {
      "author": "John Doe",
      "company": "ACME Corp",
      "subject": "Project Report"
    },
    "date_hints": ["2024-01-15", "10:30:45"]
  }
}
```

### CSV Output

Combined output: `metadata_combined.csv`

Flattened structure for easy import into spreadsheets:
- file_name
- file_path
- file_size_bytes
- file_size_mb
- file_format
- file_modified
- hwp5_header.version
- hwp5_header.flags.compressed
- hwp5_header.flags.encrypted
- hwp5_summary.generator
- hwpx.content_metadata.title
- hwpx.content_metadata.creator
- hwpx.content_metadata.creation_date
- ... (all extracted metadata as columns)

## Supported Generators

The tool can detect the following document generators:

- OpenDoc (Government e-approval system)
- Korean Government Legislation Editor (법령안편집기)
- Government Legislative Support Center (정부입법지원센터)
- Korea Legislation Research Institute (한국법령정보원)
- National Legal Information Center (국가법령정보센터)
- Hangul (HWP) - Official HWP editor
- LibreOffice
- Microsoft Office

## Metadata Fields

### HWP5 (OLE Format)

| Field | Description |
|-------|-------------|
| signature | File signature from header |
| version | HWP version (major.minor.build.revision) |
| compressed | Whether document is compressed |
| encrypted | Whether document is encrypted |
| distribution | Distribution flag |
| drm | Digital Rights Management flag |
| digital_signature | Electronic signature flag |
| generator | Detected application/generator |
| author | Document author |
| company | Author's company |
| creation_date_hints | Detected creation dates |
| modification_date_hints | Detected modification dates |

### HWPX (ZIP Format)

| Field | Description |
|-------|-------------|
| creator | Document creator |
| title | Document title |
| subject | Document subject |
| description | Document description |
| last_saved_by | Person who last saved |
| creation_date | Document creation timestamp |
| last_saved_date | Last modification timestamp |
| generator | Application that created document |

## Error Handling

The tool gracefully handles various error scenarios:

- Missing `olefile` library: Attempts HWPX parsing or reports error
- Corrupted files: Logs error and continues with other files
- Unsupported formats: Detects and reports
- Missing streams: Safely skips unavailable data

## Examples

### Extract metadata from quarterly reports

```bash
python metadata_extract.py reports/Q1_2024.hwp reports/Q2_2024.hwp \
    --format both --output ./reports_metadata
```

### Batch process directory

```bash
python metadata_extract.py documents/*.hwp --format csv --output ./exports
```

### Detect document generators and authors

```bash
python metadata_extract.py contracts/*.hwp --format json | \
    grep -A 5 '"generator"\|"author"\|"creator"'
```

### Find recent documents

```bash
python metadata_extract.py *.hwp --format json --output ./temp
# Then examine file_modified timestamps
```

## Performance Considerations

- **File size**: Works efficiently with files up to several MB
- **Batch processing**: Can process hundreds of files with minimal memory overhead
- **Output size**: JSON output typically 5-10x smaller than original file

## Limitations

1. **Content extraction**: Only extracts metadata, not document content
2. **Encrypted files**: Some encrypted file metadata may be inaccessible
3. **Custom properties**: User-defined properties may not be fully extracted
4. **HWP3 format**: Only HWP5 and HWPX formats are supported

## Dependencies

### Required
- Python 3.7+

### Optional
- `olefile`: For enhanced HWP5 metadata extraction
  ```bash
  pip install olefile
  ```

## Troubleshooting

### olefile not found
```bash
pip install olefile
```

### File encoding issues
The tool handles UTF-8, UTF-16LE, and Korean encodings automatically.

### Permission denied
Ensure you have read permissions for input files and write permissions for output directory.

### No metadata extracted
- File may be corrupted
- Try with `--format csv` for diagnostic output
- Check file permissions and integrity

## File Size Guidelines

| File Size | Processing Time | Output Size |
|-----------|-----------------|-------------|
| < 1 MB | < 1 second | < 50 KB |
| 1-10 MB | 1-5 seconds | 50-500 KB |
| > 10 MB | 5-30 seconds | > 500 KB |

## License

Part of the RHWP project - See main project LICENSE

## See Also

- [HWP Format Documentation](../README.md)
- Related tools: `hwp_generator_probe.py`
