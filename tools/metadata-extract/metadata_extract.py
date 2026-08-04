#!/usr/bin/env python3
"""HWP Document Metadata Extraction Tool

Extracts and exports document metadata from HWP/HWPX files including:
- Document properties (author, creation date, modification history)
- File information (size, format, compression, encryption)
- Document statistics (page count, word count, etc.)
- Generator/Application information

Output formats: JSON, CSV

Usage:
    python metadata_extract.py <input_file.hwp|input_file.hwpx> [--output OUTPUT_DIR] [--format {json,csv,both}]

Examples:
    python metadata_extract.py document.hwp
    python metadata_extract.py document.hwp --format json --output ./exports
    python metadata_extract.py *.hwp --format csv
"""

import argparse
import csv
import json
import os
import re
import struct
import sys
import zipfile
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

# Try to import olefile for HWP5 support
try:
    import olefile
    HAS_OLEFILE = True
except ImportError:
    HAS_OLEFILE = False


class HWPMetadataExtractor:
    """Extracts metadata from HWP and HWPX files."""

    def __init__(self, file_path: str):
        """Initialize extractor with file path."""
        self.file_path = Path(file_path)
        self.file_size = self.file_path.stat().st_size
        self.file_modified = datetime.fromtimestamp(self.file_path.stat().st_mtime)
        self.metadata: Dict[str, Any] = {
            "file_info": self._get_file_info(),
        }

    def _get_file_info(self) -> Dict[str, Any]:
        """Extract basic file information."""
        return {
            "file_path": str(self.file_path.absolute()),
            "file_name": self.file_path.name,
            "file_size_bytes": self.file_size,
            "file_size_mb": round(self.file_size / (1024 * 1024), 2),
            "file_modified": self.file_modified.isoformat(),
            "file_format": self._detect_format(),
        }

    def _detect_format(self) -> str:
        """Detect file format (HWP5 or HWPX)."""
        suffix = self.file_path.suffix.lower()
        if suffix == ".hwpx":
            return "HWPX"

        # Check if it's OLE format (HWP5)
        if HAS_OLEFILE and olefile.isOleFile(str(self.file_path)):
            return "HWP5"

        # Fallback to extension
        if suffix == ".hwp":
            return "HWP5 (extension-based)"

        return "Unknown"

    def extract_hwp5_metadata(self) -> bool:
        """Extract metadata from HWP5 (OLE format) files."""
        if not HAS_OLEFILE:
            self.metadata["error"] = "olefile not installed. Install with: pip install olefile"
            return False

        try:
            ole = olefile.OleFileIO(str(self.file_path))

            # Extract header information
            self._extract_hwp5_header(ole)

            # Extract summary information
            self._extract_hwp5_summary(ole)

            # Extract document properties
            self._extract_hwp5_doc_properties(ole)

            ole.close()
            return True
        except Exception as e:
            self.metadata["error"] = f"Failed to parse HWP5: {str(e)}"
            return False

    def _extract_hwp5_header(self, ole: "olefile.OleFileIO") -> None:
        """Extract HWP5 file header information."""
        try:
            header_data = ole.openstream("FileHeader").read()

            if len(header_data) >= 40:
                # Parse header structure
                signature = header_data[0:32].rstrip(b'\x00').decode('utf-8', errors='ignore')
                revision, build, minor, major = struct.unpack('<BBBB', header_data[32:36])
                flags = struct.unpack('<I', header_data[36:40])[0]

                self.metadata["hwp5_header"] = {
                    "signature": signature,
                    "version": f"{major}.{minor}.{build}.{revision}",
                    "flags": {
                        "compressed": bool(flags & 0x01),
                        "encrypted": bool(flags & 0x02),
                        "distribution": bool(flags & 0x04),
                        "script": bool(flags & 0x08),
                        "drm": bool(flags & 0x10),
                        "xml_template": bool(flags & 0x20),
                        "document_history": bool(flags & 0x40),
                        "digital_signature": bool(flags & 0x80),
                        "public_key_encrypted": bool(flags & 0x100),
                        "modified_certificate": bool(flags & 0x200),
                        "prepare_distribution": bool(flags & 0x400),
                    }
                }
        except Exception as e:
            self.metadata["hwp5_header"] = {"error": str(e)}

    def _extract_hwp5_summary(self, ole: "olefile.OleFileIO") -> None:
        """Extract HWP5 summary information (HwpSummaryInformation)."""
        try:
            # Try various stream names
            raw = None
            for stream_name in ["/\x05HwpSummaryInformation", "\x05HwpSummaryInformation", "HwpSummaryInformation"]:
                try:
                    raw = ole.openstream(stream_name).read()
                    break
                except:
                    continue

            if raw:
                text = raw.decode('utf-16-le', errors='replace')

                summary = {
                    "raw_text": text[:500],  # First 500 chars
                    "extracted_fields": self._parse_summary_fields(text)
                }

                # Detect generator/application
                generator = self._detect_generator_hwp5(text)
                if generator:
                    summary["generator"] = generator

                # Detect creation date hints
                date_hints = self._extract_date_hints(text)
                if date_hints:
                    summary["date_hints"] = date_hints

                self.metadata["hwp5_summary"] = summary
        except Exception as e:
            self.metadata["hwp5_summary"] = {"error": str(e)}

    def _extract_hwp5_doc_properties(self, ole: "olefile.OleFileIO") -> None:
        """Extract document properties from HWP5."""
        try:
            doc_props = {}

            # Try to extract DocInfo stream
            try:
                doc_info_raw = ole.openstream("DocInfo").read()
                doc_props["doc_info_size"] = len(doc_info_raw)
            except:
                pass

            # Try to extract BodyText stream
            try:
                body_text_raw = ole.openstream("BodyText").read()
                doc_props["body_text_size"] = len(body_text_raw)
            except:
                pass

            # List all streams
            doc_props["streams"] = ole.listdir()

            if doc_props:
                self.metadata["hwp5_doc_properties"] = doc_props
        except Exception as e:
            self.metadata["hwp5_doc_properties"] = {"error": str(e)}

    def extract_hwpx_metadata(self) -> bool:
        """Extract metadata from HWPX (ZIP format) files."""
        try:
            with zipfile.ZipFile(str(self.file_path)) as zf:
                self.metadata["hwpx"] = {}

                # Extract manifest information
                self._extract_hwpx_manifest(zf)

                # Extract content.hpf metadata
                self._extract_hwpx_content_metadata(zf)

                # Extract settings.xml metadata
                self._extract_hwpx_settings(zf)

                # List document structure
                self._extract_hwpx_structure(zf)

            return True
        except Exception as e:
            self.metadata["error"] = f"Failed to parse HWPX: {str(e)}"
            return False

    def _extract_hwpx_manifest(self, zf: zipfile.ZipFile) -> None:
        """Extract HWPX manifest information."""
        try:
            manifest_data = zf.read("META-INF/manifest.xml").decode('utf-8')
            self.metadata["hwpx"]["manifest"] = {
                "content": manifest_data[:500],  # First 500 chars
                "file_count": len([n for n in zf.namelist() if not n.endswith('/')])
            }
        except:
            pass

    def _extract_hwpx_content_metadata(self, zf: zipfile.ZipFile) -> None:
        """Extract metadata from content.hpf."""
        try:
            content_data = zf.read("Contents/content.hpf").decode('utf-8', errors='replace')
            content_meta = {}

            # Extract lastsaveby
            m = re.search(r'lastsaveby"\s+content="text">([^<]*)', content_data)
            if m:
                content_meta["last_saved_by"] = m.group(1)

            # Extract creator/author
            m = re.search(r'creator"\s+content="text">([^<]*)', content_data)
            if m:
                content_meta["creator"] = m.group(1)

            # Extract title
            m = re.search(r'title"\s+content="text">([^<]*)', content_data)
            if m:
                content_meta["title"] = m.group(1)

            # Extract subject
            m = re.search(r'subject"\s+content="text">([^<]*)', content_data)
            if m:
                content_meta["subject"] = m.group(1)

            # Extract description
            m = re.search(r'description"\s+content="text">([^<]*)', content_data)
            if m:
                content_meta["description"] = m.group(1)

            # Extract creation date
            m = re.search(r'creatdttm"\s+content="text">([^<]*)', content_data)
            if m:
                content_meta["creation_date"] = m.group(1)

            # Extract modification date
            m = re.search(r'savedttm"\s+content="text">([^<]*)', content_data)
            if m:
                content_meta["last_saved_date"] = m.group(1)

            # Extract generator
            m = re.search(r'generator"\s+content="text">([^<]*)', content_data)
            if m:
                content_meta["generator"] = m.group(1)

            if content_meta:
                self.metadata["hwpx"]["content_metadata"] = content_meta
        except:
            pass

    def _extract_hwpx_settings(self, zf: zipfile.ZipFile) -> None:
        """Extract settings from settings.xml."""
        try:
            settings_data = zf.read("Settings/settings.xml").decode('utf-8', errors='replace')
            self.metadata["hwpx"]["settings"] = {
                "content": settings_data[:500]
            }
        except:
            pass

    def _extract_hwpx_structure(self, zf: zipfile.ZipFile) -> None:
        """Extract document structure information."""
        try:
            namelist = zf.namelist()
            structure = {
                "total_files": len(namelist),
                "directories": sorted(set(n.rsplit('/', 1)[0] for n in namelist if '/' in n)),
                "file_types": {}
            }

            # Count file types
            for name in namelist:
                ext = Path(name).suffix or "directory"
                structure["file_types"][ext] = structure["file_types"].get(ext, 0) + 1

            self.metadata["hwpx"]["structure"] = structure
        except:
            pass

    def _parse_summary_fields(self, text: str) -> Dict[str, str]:
        """Parse common fields from summary text."""
        fields = {}

        # Extract common patterns
        patterns = {
            "author": [r"작성자:\s*([^\n]+)", r"Author:\s*([^\n]+)"],
            "company": [r"회사:\s*([^\n]+)", r"Company:\s*([^\n]+)"],
            "subject": [r"주제:\s*([^\n]+)", r"Subject:\s*([^\n]+)"],
            "category": [r"카테고리:\s*([^\n]+)", r"Category:\s*([^\n]+)"],
            "comments": [r"설명:\s*([^\n]+)", r"Comments:\s*([^\n]+)"],
        }

        for field_name, pattern_list in patterns.items():
            for pattern in pattern_list:
                match = re.search(pattern, text)
                if match:
                    fields[field_name] = match.group(1).strip()
                    break

        return fields

    def _detect_generator_hwp5(self, text: str) -> Optional[str]:
        """Detect document generator/application."""
        markers = [
            ("opendoc", "OpenDoc (Government e-approval)"),
            ("법령안편집기", "Korean Government Legislation Editor"),
            ("정부입법지원센터", "Government Legislative Support Center"),
            ("한국법령정보원", "Korea Legislation Research Institute"),
            ("국가법령정보센터", "National Legal Information Center"),
            ("한글", "Hangul (HWP)"),
            ("LibreOffice", "LibreOffice"),
            ("Microsoft", "Microsoft Office"),
        ]

        text_lower = text.lower()
        for marker, name in markers:
            if marker.lower() in text_lower:
                return name

        return None

    def _extract_date_hints(self, text: str) -> List[str]:
        """Extract date hints from summary text."""
        dates = []

        # Match various date formats
        patterns = [
            r"\d{4}년\s*\d{1,2}월\s*\d{1,2}일",  # YYYY년 MM월 DD일
            r"\d{4}-\d{1,2}-\d{1,2}",  # YYYY-MM-DD
            r"\d{4}/\d{1,2}/\d{1,2}",  # YYYY/MM/DD
            r"\d{1,2}:\d{2}:\d{2}",  # HH:MM:SS
        ]

        for pattern in patterns:
            matches = re.findall(pattern, text)
            dates.extend(matches)

        return list(set(dates[:10]))  # Return unique dates, max 10

    def extract(self) -> Dict[str, Any]:
        """Extract metadata based on file format."""
        file_format = self.metadata["file_info"]["file_format"]

        if "HWPX" in file_format:
            self.extract_hwpx_metadata()
        elif "HWP5" in file_format or "Unknown" in file_format:
            if not self.extract_hwp5_metadata():
                self.extract_hwpx_metadata()

        return self.metadata

    def to_json(self) -> str:
        """Convert metadata to JSON string."""
        return json.dumps(self.metadata, indent=2, ensure_ascii=False)

    def to_csv_rows(self) -> List[Dict[str, str]]:
        """Convert metadata to CSV-friendly format."""
        rows = []

        # Flatten nested structure for CSV
        flat_data = self._flatten_dict(self.metadata)

        # Create a single row with all flattened keys
        row = {
            "file_name": self.metadata["file_info"]["file_name"],
            "file_path": self.metadata["file_info"]["file_path"],
            "file_size_bytes": self.metadata["file_info"]["file_size_bytes"],
            "file_size_mb": self.metadata["file_info"]["file_size_mb"],
            "file_format": self.metadata["file_info"]["file_format"],
            "file_modified": self.metadata["file_info"]["file_modified"],
        }

        # Add extracted metadata
        for key, value in flat_data.items():
            if key != "file_info":
                row[key] = str(value)

        rows.append(row)
        return rows

    def _flatten_dict(self, d: Dict[str, Any], parent_key: str = '', sep: str = '.') -> Dict[str, Any]:
        """Flatten nested dictionary."""
        items = []
        for k, v in d.items():
            new_key = f"{parent_key}{sep}{k}" if parent_key else k
            if isinstance(v, dict):
                items.extend(self._flatten_dict(v, new_key, sep=sep).items())
            elif isinstance(v, (list, tuple)):
                items.append((new_key, str(v)))
            else:
                items.append((new_key, v))
        return dict(items)


def extract_from_files(file_paths: List[str], output_dir: Optional[str] = None,
                       output_format: str = "json") -> None:
    """Extract metadata from multiple files."""

    if output_dir:
        output_path = Path(output_dir)
        output_path.mkdir(parents=True, exist_ok=True)
    else:
        output_path = Path.cwd()

    all_metadata = []
    all_csv_rows = []

    for file_path in file_paths:
        try:
            print(f"Processing: {file_path}", file=sys.stderr)
            extractor = HWPMetadataExtractor(file_path)
            metadata = extractor.extract()
            all_metadata.append(metadata)
            all_csv_rows.extend(extractor.to_csv_rows())

            # Save individual JSON file
            if output_format in ("json", "both"):
                json_file = output_path / f"{Path(file_path).stem}_metadata.json"
                with open(json_file, 'w', encoding='utf-8') as f:
                    f.write(json.dumps(metadata, indent=2, ensure_ascii=False))
                print(f"  Saved: {json_file}")

        except Exception as e:
            print(f"  ERROR: {str(e)}", file=sys.stderr)

    # Save combined outputs
    if output_format in ("json", "both"):
        combined_json = output_path / "metadata_combined.json"
        with open(combined_json, 'w', encoding='utf-8') as f:
            f.write(json.dumps(all_metadata, indent=2, ensure_ascii=False))
        print(f"Saved combined JSON: {combined_json}")

    if output_format in ("csv", "both"):
        csv_file = output_path / "metadata_combined.csv"
        if all_csv_rows:
            with open(csv_file, 'w', newline='', encoding='utf-8') as f:
                writer = csv.DictWriter(f, fieldnames=all_csv_rows[0].keys())
                writer.writeheader()
                writer.writerows(all_csv_rows)
            print(f"Saved combined CSV: {csv_file}")


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter
    )

    parser.add_argument(
        "files",
        nargs="+",
        help="HWP/HWPX file path(s) or glob pattern"
    )
    parser.add_argument(
        "--output", "-o",
        default=None,
        help="Output directory (default: current directory)"
    )
    parser.add_argument(
        "--format", "-f",
        choices=["json", "csv", "both"],
        default="json",
        help="Output format (default: json)"
    )

    args = parser.parse_args()

    # Expand glob patterns and collect files
    file_list = []
    for pattern in args.files:
        files = list(Path(".").glob(pattern))
        if files:
            file_list.extend(str(f) for f in files if f.is_file())
        else:
            # Try as direct path
            if Path(pattern).is_file():
                file_list.append(pattern)

    if not file_list:
        print("No files found.", file=sys.stderr)
        return 1

    extract_from_files(file_list, args.output, args.format)
    return 0


if __name__ == "__main__":
    sys.exit(main())
