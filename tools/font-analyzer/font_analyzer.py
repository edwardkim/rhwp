#!/usr/bin/env python3
"""Comprehensive font analysis tool for HWP documents.

Identifies all fonts used in HWP documents, checks for missing fonts,
provides recommendations for font substitution, and generates detailed reports.

Usage:
    python font_analyzer.py <hwp_file> [--output report.json] [--check-system]
    python font_analyzer.py --analyze-dir C:/path/to/hwp/documents --report summary.csv

Features:
    - Extracts font names from HWP document structure
    - Detects installed system fonts
    - Identifies missing/unavailable fonts
    - Provides font substitution recommendations
    - Generates comprehensive JSON/CSV reports
    - Batch analysis of multiple documents
"""

from __future__ import annotations

import argparse
import csv
import json
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any
from xml.etree import ElementTree as ET
import zipfile
import logging

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


@dataclass
class FontInfo:
    """Information about a font found in document."""
    name: str
    family: str = ""
    style: str = "regular"
    usage_count: int = 1
    source: str = "unknown"  # paragraph, table, header, footer, shape, etc.
    locations: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)


@dataclass
class FontAnalysisResult:
    """Complete font analysis result for a document."""
    file_path: str
    document_name: str
    fonts_found: list[FontInfo] = field(default_factory=list)
    fonts_by_name: dict[str, FontInfo] = field(default_factory=dict)
    missing_fonts: list[FontInfo] = field(default_factory=list)
    substitution_recommendations: dict[str, str] = field(default_factory=dict)
    system_fonts_available: list[str] = field(default_factory=list)
    coverage_percentage: float = 0.0
    total_font_instances: int = 0
    unique_fonts: int = 0
    analysis_timestamp: str = ""

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        return {
            'file_path': self.file_path,
            'document_name': self.document_name,
            'fonts_found': [f.to_dict() for f in self.fonts_found],
            'fonts_by_name': {k: v.to_dict() for k, v in self.fonts_by_name.items()},
            'missing_fonts': [f.to_dict() for f in self.missing_fonts],
            'substitution_recommendations': self.substitution_recommendations,
            'system_fonts_available': self.system_fonts_available,
            'coverage_percentage': self.coverage_percentage,
            'total_font_instances': self.total_font_instances,
            'unique_fonts': self.unique_fonts,
            'analysis_timestamp': self.analysis_timestamp,
        }


class FontSubstitutionMapper:
    """Maps missing fonts to suitable substitutes."""

    # Font families and their characteristics
    FONT_FAMILIES = {
        # Korean fonts
        '나눔바른고딕': ['Noto Sans CJK KR', 'Gothic', '굴림'],
        '나눔고딕': ['Noto Sans CJK KR', 'Gothic', '굴림'],
        '나눔명조': ['Noto Serif CJK KR', 'Mincho', '바탕'],
        '나눔손글씨': ['Marker Felt', 'Casual'],
        '맑은고딕': ['Segoe UI', 'Noto Sans CJK KR', 'Gothic'],
        '굴림': ['Noto Sans CJK KR', '나눔고딕', 'Gothic'],
        '돋움': ['Noto Sans CJK KR', '나눔바른고딕', 'Gothic'],
        '바탕': ['Noto Serif CJK KR', '나눔명조', 'Mincho'],
        '궁서': ['Noto Serif CJK KR', 'Serif', '바탕'],
        '한컴손글씨': ['Marker Felt', 'Casual'],

        # English fonts - Sans Serif
        'Arial': ['Segoe UI', 'Helvetica', 'Trebuchet MS'],
        'Helvetica': ['Arial', 'Segoe UI', 'Trebuchet MS'],
        'Segoe UI': ['Arial', 'Helvetica', 'Tahoma'],
        'Trebuchet MS': ['Arial', 'Segoe UI', 'Helvetica'],
        'Verdana': ['Segoe UI', 'Arial', 'Tahoma'],
        'Tahoma': ['Segoe UI', 'Arial', 'Verdana'],
        'Calibri': ['Segoe UI', 'Arial', 'Helvetica'],
        'Courier New': ['Consolas', 'Monaco', 'Monospace'],
        'Consolas': ['Courier New', 'Monaco', 'Monospace'],
        'Georgia': ['Times New Roman', 'Serif'],
        'Times New Roman': ['Georgia', 'Serif', 'Garamond'],
        'Garamond': ['Times New Roman', 'Georgia', 'Serif'],
    }

    # System-independent fallback fonts
    FALLBACK_SUBSTITUTES = {
        'serif': 'Times New Roman',
        'sans-serif': 'Arial',
        'monospace': 'Courier New',
        'cursive': 'Lucida Handwriting',
        'fantasy': 'Impact',
    }

    @classmethod
    def get_substitutes(cls, font_name: str) -> list[str]:
        """Get list of substitute fonts for given font name.

        Args:
            font_name: Name of the font to find substitutes for

        Returns:
            List of substitute font names, ordered by preference
        """
        font_name = font_name.strip()

        # Check if exact match exists
        if font_name in cls.FONT_FAMILIES:
            return cls.FONT_FAMILIES[font_name]

        # Check case-insensitive match
        for key, subs in cls.FONT_FAMILIES.items():
            if key.lower() == font_name.lower():
                return subs

        # Heuristic-based suggestions
        suggestions = []

        # Detect language/type based on name
        if any(char.encode('utf-8').decode('utf-8', errors='ignore') for char in font_name
               if ord(char) > 127):
            # Korean font - suggest CJK fonts
            if 'gothic' in font_name.lower() or 'sans' in font_name.lower():
                suggestions = ['Noto Sans CJK KR', 'Gothic', '굴림']
            elif 'serif' in font_name.lower() or 'mincho' in font_name.lower():
                suggestions = ['Noto Serif CJK KR', '바탕']
            else:
                suggestions = ['Noto Sans CJK KR', '굴림']
        else:
            # English font - suggest based on patterns
            if 'mono' in font_name.lower():
                suggestions = ['Courier New', 'Consolas', 'Monaco']
            elif 'serif' in font_name.lower():
                suggestions = ['Times New Roman', 'Georgia', 'Garamond']
            else:
                suggestions = ['Arial', 'Segoe UI', 'Helvetica']

        return suggestions or ['Arial']


class HWPFontAnalyzer:
    """Analyzes fonts in HWP documents."""

    # Namespaces used in HWP XML
    NAMESPACES = {
        'o': 'urn:schemas-microsoft-com:office:office',
        'v': 'urn:schemas-microsoft-com:vml',
        'w': 'http://schemas.openxmlformats.org/wordprocessingml/2006/main',
        'wp': 'http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing',
    }

    def __init__(self, check_system_fonts: bool = True):
        """Initialize analyzer.

        Args:
            check_system_fonts: Whether to check for system fonts
        """
        self.check_system_fonts = check_system_fonts
        self.system_fonts = set()
        if check_system_fonts:
            self._detect_system_fonts()

    def _detect_system_fonts(self) -> None:
        """Detect installed system fonts."""
        system_fonts = []

        # Try different methods based on OS
        try:
            import platform
            system_name = platform.system()

            if system_name == 'Windows':
                system_fonts.extend(self._get_windows_fonts())
            elif system_name == 'Darwin':
                system_fonts.extend(self._get_macos_fonts())
            elif system_name == 'Linux':
                system_fonts.extend(self._get_linux_fonts())

            self.system_fonts = set(f.lower() for f in system_fonts)
            logger.info(f"Detected {len(self.system_fonts)} system fonts")
        except Exception as e:
            logger.warning(f"Failed to detect system fonts: {e}")

    @staticmethod
    def _get_windows_fonts() -> list[str]:
        """Get Windows system fonts."""
        fonts = []
        try:
            import winreg
            with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE,
                              r'SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts') as key:
                for i in range(winreg.QueryInfoKey(key)[1]):
                    try:
                        font_name, _ = winreg.EnumValue(key, i)
                        # Extract font name (remove " (TrueType)" suffix)
                        name = font_name.split(' (')[0]
                        fonts.append(name)
                    except Exception:
                        pass
        except ImportError:
            logger.warning("winreg not available, skipping Windows font detection")

        # Add common Windows fonts as fallback
        common_fonts = [
            'Arial', 'Times New Roman', 'Courier New', 'Verdana',
            'Georgia', 'Trebuchet MS', 'Comic Sans MS', 'Impact',
            'Palatino Linotype', 'Garamond', 'Book Antiqua', 'Tahoma',
            'Segoe UI', 'Calibri', 'Cambria', 'Consolas',
            'Arial Black', 'Arial Narrow', 'Arial Unicode MS',
        ]
        fonts.extend(common_fonts)
        return fonts

    @staticmethod
    def _get_macos_fonts() -> list[str]:
        """Get macOS system fonts."""
        fonts = []
        try:
            from pathlib import Path
            font_dirs = [
                Path('/Library/Fonts'),
                Path('/System/Library/Fonts'),
                Path.home() / 'Library/Fonts',
            ]
            for font_dir in font_dirs:
                if font_dir.exists():
                    for font_file in font_dir.glob('*'):
                        if font_file.suffix.lower() in ('.ttf', '.otf'):
                            fonts.append(font_file.stem)
        except Exception:
            pass
        return fonts

    @staticmethod
    def _get_linux_fonts() -> list[str]:
        """Get Linux system fonts."""
        fonts = []
        try:
            import subprocess
            result = subprocess.run(['fc-list', ':'], capture_output=True, text=True)
            for line in result.stdout.split('\n'):
                if ':' in line:
                    font_info = line.split(':')[1].strip()
                    for font_name in font_info.split(','):
                        fonts.append(font_name.strip())
        except Exception:
            pass
        return fonts

    def analyze_hwp_file(self, file_path: Path) -> FontAnalysisResult:
        """Analyze a single HWP file for fonts.

        Args:
            file_path: Path to HWP file

        Returns:
            FontAnalysisResult containing analysis details
        """
        result = FontAnalysisResult(
            file_path=str(file_path),
            document_name=file_path.stem,
            analysis_timestamp=self._get_timestamp()
        )

        try:
            # Handle both HWPX and HWP formats
            if file_path.suffix.lower() == '.hwpx':
                fonts = self._extract_hwpx_fonts(file_path)
            elif file_path.suffix.lower() == '.hwp':
                fonts = self._extract_hwp_fonts(file_path)
            else:
                logger.warning(f"Unsupported file format: {file_path.suffix}")
                return result

            # Process fonts
            result.fonts_found = fonts
            result.total_font_instances = sum(f.usage_count for f in fonts)
            result.unique_fonts = len({f.name for f in fonts})

            # Organize by name
            for font in fonts:
                if font.name not in result.fonts_by_name:
                    result.fonts_by_name[font.name] = font
                else:
                    result.fonts_by_name[font.name].usage_count += font.usage_count

            # Check for missing fonts and get substitutes
            available_count = 0
            for font in result.fonts_found:
                font_lower = font.name.lower()
                if self.system_fonts and font_lower not in self.system_fonts:
                    result.missing_fonts.append(font)
                    # Get substitution recommendations
                    substitutes = FontSubstitutionMapper.get_substitutes(font.name)
                    if substitutes:
                        result.substitution_recommendations[font.name] = substitutes[0]
                else:
                    available_count += 1
                    result.system_fonts_available.append(font.name)

            # Calculate coverage
            if result.total_font_instances > 0:
                result.coverage_percentage = (available_count / result.total_font_instances) * 100

            logger.info(f"Analysis complete: {file_path.name} - "
                       f"{result.unique_fonts} unique fonts, "
                       f"{len(result.missing_fonts)} missing, "
                       f"{result.coverage_percentage:.1f}% coverage")

        except Exception as e:
            logger.error(f"Error analyzing {file_path}: {e}")

        return result

    def _extract_hwpx_fonts(self, file_path: Path) -> list[FontInfo]:
        """Extract fonts from HWPX file (Office Open XML format)."""
        fonts = []
        try:
            with zipfile.ZipFile(file_path, 'r') as hwpx:
                # Try to read document.xml
                try:
                    doc_xml = hwpx.read('word/document.xml').decode('utf-8')
                    fonts.extend(self._parse_xml_fonts(doc_xml, 'document'))
                except KeyError:
                    pass

                # Try to read styles.xml
                try:
                    styles_xml = hwpx.read('word/styles.xml').decode('utf-8')
                    fonts.extend(self._parse_xml_fonts(styles_xml, 'styles'))
                except KeyError:
                    pass

                # Try other content files
                for name in hwpx.namelist():
                    if name.endswith('.xml') and name.startswith('word/'):
                        try:
                            content = hwpx.read(name).decode('utf-8')
                            fonts.extend(self._parse_xml_fonts(content, Path(name).stem))
                        except Exception:
                            pass

        except Exception as e:
            logger.error(f"Failed to extract HWPX fonts: {e}")

        return self._deduplicate_fonts(fonts)

    def _extract_hwp_fonts(self, file_path: Path) -> list[FontInfo]:
        """Extract fonts from HWP file (OLE format).

        Note: This attempts to parse OLE structures. Full HWP support
        requires the pyhwpx library.
        """
        fonts = []
        try:
            with zipfile.ZipFile(file_path, 'r') as hwp:
                # HWP files stored as ZIP may contain XML sections
                for name in hwp.namelist():
                    if name.endswith('.xml'):
                        try:
                            content = hwp.read(name).decode('utf-8', errors='ignore')
                            fonts.extend(self._parse_xml_fonts(content, name))
                        except Exception:
                            pass
        except zipfile.BadZipFile:
            # Try OLE format using cfb if available
            try:
                import cfb
                with cfb.open(file_path) as cf:
                    # Extract font info from OLE streams if available
                    logger.info(f"HWP file {file_path.name} requires pyhwpx for full parsing")
            except ImportError:
                logger.warning(
                    f"Install cfb and pyhwpx for full HWP support: "
                    f"pip install pyhwpx cfb"
                )
        except Exception as e:
            logger.error(f"Failed to extract HWP fonts: {e}")

        return self._deduplicate_fonts(fonts)

    def _parse_xml_fonts(self, xml_content: str, source: str) -> list[FontInfo]:
        """Parse font information from XML content."""
        fonts = []
        try:
            # Extract font references from XML
            # Look for common font element patterns
            import re

            # Pattern for rFonts elements
            rfonts_pattern = r'<w:rFonts\s+w:ascii="([^"]+)"(?:\s+w:hAnsi="([^"]+)")?'
            for match in re.finditer(rfonts_pattern, xml_content):
                font_name = match.group(1)
                if font_name:
                    fonts.append(FontInfo(
                        name=font_name,
                        source=source,
                        locations=[f'line {match.start()}']
                    ))

            # Pattern for theme font names
            theme_pattern = r'<w:eastAsia\s+w:val="([^"]+)"'
            for match in re.finditer(theme_pattern, xml_content):
                font_name = match.group(1)
                if font_name:
                    fonts.append(FontInfo(
                        name=font_name,
                        source=f"{source}:theme",
                        locations=[f'line {match.start()}']
                    ))

            # Pattern for font family definitions (Korean)
            family_pattern = r'<fontfamily\s+(?:[^>])*name="([^"]+)"'
            for match in re.finditer(family_pattern, xml_content, re.IGNORECASE):
                font_name = match.group(1)
                if font_name:
                    fonts.append(FontInfo(
                        name=font_name,
                        source=f"{source}:fontfamily",
                        locations=[f'line {match.start()}']
                    ))

        except Exception as e:
            logger.warning(f"Error parsing XML fonts from {source}: {e}")

        return fonts

    @staticmethod
    def _deduplicate_fonts(fonts: list[FontInfo]) -> list[FontInfo]:
        """Remove duplicate fonts, merging usage counts."""
        font_map: dict[str, FontInfo] = {}
        for font in fonts:
            key = font.name.lower()
            if key in font_map:
                font_map[key].usage_count += font.usage_count
                font_map[key].locations.extend(font.locations)
            else:
                font_map[key] = font

        return list(font_map.values())

    @staticmethod
    def _get_timestamp() -> str:
        """Get current timestamp."""
        from datetime import datetime
        return datetime.now().isoformat()


def format_report_text(result: FontAnalysisResult) -> str:
    """Format analysis result as readable text report."""
    lines = []
    lines.append("=" * 80)
    lines.append(f"FONT ANALYSIS REPORT: {result.document_name}")
    lines.append("=" * 80)
    lines.append(f"File: {result.file_path}")
    lines.append(f"Timestamp: {result.analysis_timestamp}")
    lines.append("")

    lines.append("SUMMARY")
    lines.append("-" * 80)
    lines.append(f"Total Font Instances: {result.total_font_instances}")
    lines.append(f"Unique Fonts: {result.unique_fonts}")
    lines.append(f"Available Fonts: {len(result.system_fonts_available)}")
    lines.append(f"Missing Fonts: {len(result.missing_fonts)}")
    lines.append(f"Coverage: {result.coverage_percentage:.1f}%")
    lines.append("")

    if result.fonts_by_name:
        lines.append("FONTS FOUND")
        lines.append("-" * 80)
        for font_name, font_info in sorted(result.fonts_by_name.items()):
            status = "✓" if font_info in result.system_fonts_available else "✗"
            lines.append(f"  {status} {font_name}")
            lines.append(f"      Usage: {font_info.usage_count} instances")
            lines.append(f"      Source: {font_info.source}")
        lines.append("")

    if result.missing_fonts:
        lines.append("MISSING FONTS & RECOMMENDATIONS")
        lines.append("-" * 80)
        for font_info in sorted(result.missing_fonts, key=lambda f: f.name):
            lines.append(f"  Font: {font_info.name}")
            if font_info.name in result.substitution_recommendations:
                lines.append(
                    f"  Substitute: {result.substitution_recommendations[font_info.name]}"
                )
            lines.append("")

    lines.append("=" * 80)
    return "\n".join(lines)


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description='Analyze fonts in HWP documents',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__
    )

    parser.add_argument(
        'input',
        nargs='?',
        help='HWP file to analyze'
    )
    parser.add_argument(
        '--analyze-dir',
        type=Path,
        help='Analyze all HWP files in directory'
    )
    parser.add_argument(
        '--output', '-o',
        type=Path,
        help='Output report file (JSON, CSV, or TXT)'
    )
    parser.add_argument(
        '--check-system',
        action='store_true',
        help='Check system fonts availability'
    )
    parser.add_argument(
        '--format',
        choices=['json', 'csv', 'txt'],
        default='txt',
        help='Output format'
    )
    parser.add_argument(
        '--recursive', '-r',
        action='store_true',
        help='Recursively search for HWP files in directory'
    )

    args = parser.parse_args()

    if not args.input and not args.analyze_dir:
        parser.print_help()
        return 1

    analyzer = HWPFontAnalyzer(check_system_fonts=True)
    results = []

    # Analyze single file
    if args.input:
        input_path = Path(args.input)
        if not input_path.exists():
            logger.error(f"File not found: {input_path}")
            return 1

        logger.info(f"Analyzing: {input_path}")
        result = analyzer.analyze_hwp_file(input_path)
        results.append(result)

        # Print text report if no output file specified
        if not args.output:
            print(format_report_text(result))

    # Analyze directory
    elif args.analyze_dir:
        if not args.analyze_dir.exists():
            logger.error(f"Directory not found: {args.analyze_dir}")
            return 1

        pattern = '**/*.hw*' if args.recursive else '*.hw*'
        hw_files = list(args.analyze_dir.glob(pattern))

        if not hw_files:
            logger.warning(f"No HWP files found in {args.analyze_dir}")
            return 1

        logger.info(f"Found {len(hw_files)} HWP files")
        for hwp_file in sorted(hw_files):
            logger.info(f"Analyzing: {hwp_file.name}")
            result = analyzer.analyze_hwp_file(hwp_file)
            results.append(result)

    # Write output
    if args.output and results:
        args.output.parent.mkdir(parents=True, exist_ok=True)

        if args.format == 'json' or args.output.suffix == '.json':
            with open(args.output, 'w', encoding='utf-8') as f:
                json.dump([r.to_dict() for r in results], f, indent=2, ensure_ascii=False)
            logger.info(f"Report saved to: {args.output}")

        elif args.format == 'csv' or args.output.suffix == '.csv':
            with open(args.output, 'w', encoding='utf-8', newline='') as f:
                if results:
                    writer = csv.writer(f)
                    # Header
                    writer.writerow([
                        'file_path', 'document_name', 'unique_fonts', 'missing_fonts',
                        'coverage_percentage', 'fonts_found', 'substitution_needed'
                    ])
                    # Rows
                    for result in results:
                        writer.writerow([
                            result.file_path,
                            result.document_name,
                            result.unique_fonts,
                            len(result.missing_fonts),
                            f"{result.coverage_percentage:.1f}%",
                            '; '.join(sorted(result.fonts_by_name.keys())),
                            '; '.join(sorted([f.name for f in result.missing_fonts]))
                            if result.missing_fonts else ''
                        ])
            logger.info(f"Report saved to: {args.output}")

        elif args.format == 'txt' or args.output.suffix == '.txt':
            with open(args.output, 'w', encoding='utf-8') as f:
                for result in results:
                    f.write(format_report_text(result))
                    f.write("\n\n")
            logger.info(f"Report saved to: {args.output}")

    return 0


if __name__ == '__main__':
    sys.exit(main())
