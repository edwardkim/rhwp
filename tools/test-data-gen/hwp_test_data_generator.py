#!/usr/bin/env python
"""HWP Document Test Data Generator

Generates synthetic HWP documents with configurable complexity levels for testing purposes.
Supports creation of documents with various features:
  - Text content (basic paragraphs)
  - Tables (configurable dimensions, cell content)
  - Images (synthetic PNG/JPEG generation)
  - Formatting and styles (bold, italic, colors, fonts)
  - Multiple pages and sections
  - Complex nested structures

Usage:
    python hwp_test_data_generator.py --complexity <low|medium|high> --output <path.hwpx>
    python hwp_test_data_generator.py --preset simple --count 10 --output-dir ./test_docs/
    python hwp_test_data_generator.py --help

Requirements:
    - python-pptx (for document creation patterns)
    - Pillow (for image generation)
    - zipfile (standard library)
"""

import argparse
import io
import json
import os
import sys
import zipfile
from dataclasses import dataclass, asdict
from datetime import datetime
from enum import Enum
from pathlib import Path
from typing import Optional, List, Dict, Any
from xml.etree import ElementTree as ET

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    Image = None
    ImageDraw = None


class Complexity(Enum):
    """Document complexity levels."""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"


@dataclass
class ComplexityConfig:
    """Configuration for document complexity."""
    num_paragraphs: int
    num_tables: int
    num_images: int
    table_rows: int
    table_cols: int
    text_length: int  # Average words per paragraph
    use_styles: bool
    use_colors: bool
    include_complex_shapes: bool


COMPLEXITY_PRESETS = {
    Complexity.LOW: ComplexityConfig(
        num_paragraphs=3,
        num_tables=0,
        num_images=0,
        table_rows=3,
        table_cols=3,
        text_length=50,
        use_styles=False,
        use_colors=False,
        include_complex_shapes=False,
    ),
    Complexity.MEDIUM: ComplexityConfig(
        num_paragraphs=10,
        num_tables=2,
        num_images=1,
        table_rows=5,
        table_cols=4,
        text_length=100,
        use_styles=True,
        use_colors=True,
        include_complex_shapes=False,
    ),
    Complexity.HIGH: ComplexityConfig(
        num_paragraphs=20,
        num_tables=5,
        num_images=3,
        table_rows=10,
        table_cols=6,
        text_length=150,
        use_styles=True,
        use_colors=True,
        include_complex_shapes=True,
    ),
}


class TestDataGenerator:
    """Generator for synthetic HWP test documents."""

    def __init__(self, complexity: Complexity = Complexity.MEDIUM, seed: Optional[int] = None):
        """Initialize generator with complexity level.

        Args:
            complexity: Document complexity level
            seed: Random seed for reproducible generation
        """
        self.complexity = complexity
        self.config = COMPLEXITY_PRESETS[complexity]
        self.seed = seed or 42
        self._init_random(seed)

    def _init_random(self, seed: Optional[int]) -> None:
        """Initialize random number generator."""
        import random
        random.seed(seed)

    def _generate_paragraph_text(self, paragraph_num: int, word_count: Optional[int] = None) -> str:
        """Generate sample text for a paragraph.

        Args:
            paragraph_num: Paragraph number for variation
            word_count: Number of words to generate (uses config default if None)

        Returns:
            Generated paragraph text
        """
        import random

        if word_count is None:
            word_count = self.config.text_length

        sample_words = [
            "Lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit",
            "한글", "테스트", "문서", "생성기", "복잡도", "설정", "가능", "예제",
            "데이터", "검증", "구조", "포맷", "내용", "스타일", "테이블", "이미지",
            "텍스트", "형식", "단락", "섹션", "페이지", "문단", "구성", "배치",
        ]

        words = random.choices(sample_words, k=word_count)
        text = " ".join(words)
        return f"Paragraph {paragraph_num}: {text}"

    def _generate_table_content(self, row: int, col: int) -> str:
        """Generate content for a table cell.

        Args:
            row: Row index
            col: Column index

        Returns:
            Cell content string
        """
        headers = ["Header", "Data", "Value", "Name", "Info"]
        col_name = headers[col % len(headers)]
        return f"{col_name} R{row}C{col}"

    def _generate_image(self, image_num: int, width: int = 200, height: int = 150) -> bytes:
        """Generate a synthetic image.

        Args:
            image_num: Image number for variation
            width: Image width in pixels
            height: Image height in pixels

        Returns:
            PNG image bytes
        """
        if Image is None:
            # Return empty bytes if PIL not available
            return b""

        import random

        # Create image with random color
        color = tuple(random.randint(100, 255) for _ in range(3))
        img = Image.new("RGB", (width, height), color=color)

        if ImageDraw:
            draw = ImageDraw.Draw(img)
            # Draw a simple pattern
            for i in range(0, width, 20):
                draw.line([(i, 0), (i, height)], fill=(255, 255, 255), width=1)
            for i in range(0, height, 20):
                draw.line([(0, i), (width, i)], fill=(255, 255, 255), width=1)

            # Add text
            text = f"Image {image_num}"
            try:
                draw.text((10, 10), text, fill=(0, 0, 0))
            except Exception:
                # Font might not be available, skip text
                pass

        # Convert to PNG bytes
        buffer = io.BytesIO()
        img.save(buffer, format="PNG")
        buffer.seek(0)
        return buffer.getvalue()

    def generate_hwpx(self, output_path: str) -> None:
        """Generate a synthetic HWPX document.

        Args:
            output_path: Path to save HWPX file
        """
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        with zipfile.ZipFile(output_path, 'w', zipfile.ZIP_DEFLATED) as hwpx:
            # Add required manifest
            hwpx.writestr('mimetype', 'application/vnd.hancom.hwpx')

            # Create META-INF/manifest.xml
            manifest = self._create_manifest()
            hwpx.writestr('META-INF/manifest.xml', manifest)

            # Create contents
            hwpx.writestr('Contents/content.hpf', self._create_content_hpf())
            hwpx.writestr('Contents/docInfo.xml', self._create_docinfo_xml())
            hwpx.writestr('Contents/styles.xml', self._create_styles_xml())

            # Add images if needed
            for i in range(self.config.num_images):
                image_data = self._generate_image(i + 1)
                if image_data:
                    hwpx.writestr(f'Contents/images/image{i+1}.png', image_data)

            # Create settings.xml
            hwpx.writestr('Contents/settings.xml', self._create_settings_xml())

            # Create version info
            hwpx.writestr('version.xml', self._create_version_xml())

    def _create_manifest(self) -> str:
        """Create META-INF/manifest.xml content."""
        manifest_root = ET.Element("Manifest")
        manifest_root.set("xmlns", "urn:oasis:names:tc:opendocument:xmlns:manifest:1.0")

        files = [
            ("application/vnd.hancom.hwpx+zip", "/"),
            ("application/vnd.hancom.hwpx-officedocument+xml", "/Contents/content.hpf"),
            ("application/vnd.hancom.hwpx-officedocument+xml", "/Contents/docInfo.xml"),
            ("text/xml", "/Contents/styles.xml"),
            ("text/xml", "/Contents/settings.xml"),
            ("text/xml", "/version.xml"),
        ]

        # Add image files
        for i in range(self.config.num_images):
            files.append(("image/png", f"/Contents/images/image{i+1}.png"))

        for media_type, path in files:
            file_elem = ET.SubElement(manifest_root, "File")
            file_elem.set("MediaType", media_type)
            file_elem.set("FullPath", path)

        return ET.tostring(manifest_root, encoding='unicode')

    def _create_content_hpf(self) -> str:
        """Create Contents/content.hpf content."""
        root = ET.Element("HwpMLDocument")
        root.set("xmlns", "http://www.hancom.co.kr/hwpml/2.1")

        # Body section
        body = ET.SubElement(root, "Body")

        # Add paragraphs
        section = ET.SubElement(body, "Section")
        for para_num in range(self.config.num_paragraphs):
            para = ET.SubElement(section, "P")
            para.set("id", f"p{para_num+1}")

            # Text content
            text = self._generate_paragraph_text(para_num + 1)
            text_elem = ET.SubElement(para, "Text")
            text_elem.text = text

            # Add run properties if using styles
            if self.config.use_styles:
                run_props = ET.SubElement(para, "RunProps")
                if para_num % 3 == 0:
                    run_props.set("Bold", "true")
                if para_num % 3 == 1:
                    run_props.set("Italic", "true")

        # Add tables
        for table_num in range(self.config.num_tables):
            table = ET.SubElement(section, "Table")
            table.set("id", f"table{table_num+1}")

            # Table properties
            tbl_props = ET.SubElement(table, "TblProps")
            tbl_props.set("Width", "9500")

            # Table rows
            for row in range(self.config.table_rows):
                tr = ET.SubElement(table, "Tr")
                for col in range(self.config.table_cols):
                    cell = ET.SubElement(tr, "Td")
                    cell_content = self._generate_table_content(row, col)
                    cell_text = ET.SubElement(cell, "P")
                    cell_para_text = ET.SubElement(cell_text, "Text")
                    cell_para_text.text = cell_content

        # Add images (simple reference)
        for img_num in range(self.config.num_images):
            para = ET.SubElement(section, "P")
            para.set("id", f"img_para_{img_num+1}")
            img_elem = ET.SubElement(para, "Image")
            img_elem.set("href", f"images/image{img_num+1}.png")
            img_elem.set("Width", "1000")
            img_elem.set("Height", "750")

        return ET.tostring(root, encoding='unicode')

    def _create_styles_xml(self) -> str:
        """Create Contents/styles.xml content."""
        root = ET.Element("Styles")
        root.set("xmlns", "http://www.hancom.co.kr/hwpml/2.1")

        # Define some basic styles
        if self.config.use_styles:
            # Character styles
            char_style = ET.SubElement(root, "CharStyle")
            char_style.set("name", "Bold")
            char_style.set("fontweight", "bold")

            # Paragraph styles
            para_style = ET.SubElement(root, "ParaStyle")
            para_style.set("name", "Normal")
            para_style.set("fontsize", "10")

        return ET.tostring(root, encoding='unicode')

    def _create_docinfo_xml(self) -> str:
        """Create Contents/docInfo.xml content."""
        root = ET.Element("DocInfo")
        root.set("xmlns", "http://www.hancom.co.kr/hwpml/2.1")

        # Title
        title = ET.SubElement(root, "Title")
        title.text = f"Test Document - {self.complexity.value} complexity"

        # Subject
        subject = ET.SubElement(root, "Subject")
        subject.text = "Generated synthetic HWP test document"

        # Creator
        creator = ET.SubElement(root, "Creator")
        creator.text = "HWP Test Data Generator"

        # Created date
        created = ET.SubElement(root, "Created")
        created.text = datetime.now().isoformat()

        # Last saved by
        lastsaveby = ET.SubElement(root, "LastSavedBy")
        lastsaveby.text = "TestGenerator"

        # Modified
        modified = ET.SubElement(root, "Modified")
        modified.text = datetime.now().isoformat()

        return ET.tostring(root, encoding='unicode')

    def _create_settings_xml(self) -> str:
        """Create Contents/settings.xml content."""
        root = ET.Element("Settings")
        root.set("xmlns", "http://www.hancom.co.kr/hwpml/2.1")

        # View properties
        view = ET.SubElement(root, "ViewProperties")
        view.set("zoom", "100")
        view.set("displaymode", "page")

        return ET.tostring(root, encoding='unicode')

    def _create_version_xml(self) -> str:
        """Create version.xml content."""
        root = ET.Element("Version")
        root.set("app", "HWP")
        root.set("major", "5")
        root.set("minor", "0")
        root.set("revision", "0")
        root.set("build", "0")

        return ET.tostring(root, encoding='unicode')


class TestDocumentBatch:
    """Generate batch of test documents with varying complexity."""

    def __init__(self, output_dir: str):
        """Initialize batch generator.

        Args:
            output_dir: Directory to save generated documents
        """
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)

    def generate_preset_batch(self, count_per_complexity: int = 5) -> List[Path]:
        """Generate batch with all complexity levels.

        Args:
            count_per_complexity: Number of documents per complexity level

        Returns:
            List of generated file paths
        """
        generated_files = []

        for complexity in Complexity:
            for i in range(count_per_complexity):
                generator = TestDataGenerator(complexity=complexity, seed=hash((complexity.value, i)))
                filename = f"test_{complexity.value}_{i:02d}.hwpx"
                filepath = self.output_dir / filename

                print(f"Generating {filepath.name}...", end=" ", flush=True)
                generator.generate_hwpx(str(filepath))
                generated_files.append(filepath)
                print("Done")

        return generated_files

    def generate_custom_batch(
        self,
        count: int,
        complexity: Complexity = Complexity.MEDIUM,
        seed_start: int = 0
    ) -> List[Path]:
        """Generate batch with custom parameters.

        Args:
            count: Number of documents to generate
            complexity: Document complexity level
            seed_start: Starting seed value for reproducibility

        Returns:
            List of generated file paths
        """
        generated_files = []

        for i in range(count):
            generator = TestDataGenerator(
                complexity=complexity,
                seed=seed_start + i
            )
            filename = f"test_{complexity.value}_{seed_start + i:04d}.hwpx"
            filepath = self.output_dir / filename

            print(f"Generating {filepath.name}...", end=" ", flush=True)
            generator.generate_hwpx(str(filepath))
            generated_files.append(filepath)
            print("Done")

        return generated_files


def main() -> int:
    """Main entry point for command-line interface."""
    parser = argparse.ArgumentParser(
        description="Generate synthetic HWP test documents with configurable complexity"
    )

    subparsers = parser.add_subparsers(dest="command", help="Generation mode")

    # Single document generation
    single_parser = subparsers.add_parser("single", help="Generate single document")
    single_parser.add_argument(
        "--complexity",
        choices=[c.value for c in Complexity],
        default="medium",
        help="Document complexity level (default: medium)"
    )
    single_parser.add_argument(
        "--output",
        required=True,
        help="Output file path (.hwpx)"
    )
    single_parser.add_argument(
        "--seed",
        type=int,
        help="Random seed for reproducibility"
    )

    # Batch generation
    batch_parser = subparsers.add_parser("batch", help="Generate batch of documents")
    batch_parser.add_argument(
        "--output-dir",
        required=True,
        help="Output directory for generated documents"
    )
    batch_parser.add_argument(
        "--preset",
        choices=["simple", "all"],
        default="all",
        help="Preset batch type (default: all)"
    )
    batch_parser.add_argument(
        "--count",
        type=int,
        default=5,
        help="Documents per complexity level (default: 5)"
    )

    # Custom batch
    custom_parser = subparsers.add_parser("custom", help="Generate custom batch")
    custom_parser.add_argument(
        "--count",
        type=int,
        required=True,
        help="Number of documents to generate"
    )
    custom_parser.add_argument(
        "--complexity",
        choices=[c.value for c in Complexity],
        default="medium",
        help="Document complexity level (default: medium)"
    )
    custom_parser.add_argument(
        "--output-dir",
        required=True,
        help="Output directory for generated documents"
    )
    custom_parser.add_argument(
        "--seed-start",
        type=int,
        default=0,
        help="Starting seed value (default: 0)"
    )

    args = parser.parse_args()

    if args.command == "single":
        complexity = Complexity(args.complexity)
        generator = TestDataGenerator(complexity=complexity, seed=args.seed)
        print(f"Generating {complexity.value} complexity document...")
        generator.generate_hwpx(args.output)
        print(f"Document saved to: {args.output}")
        return 0

    elif args.command == "batch":
        batch = TestDocumentBatch(args.output_dir)
        print(f"Generating batch to: {args.output_dir}")
        if args.preset == "simple":
            files = batch.generate_custom_batch(1, Complexity.LOW)
        else:
            files = batch.generate_preset_batch(args.count)
        print(f"\nGenerated {len(files)} documents")
        return 0

    elif args.command == "custom":
        complexity = Complexity(args.complexity)
        batch = TestDocumentBatch(args.output_dir)
        print(f"Generating {args.count} {complexity.value} complexity documents...")
        files = batch.generate_custom_batch(
            args.count,
            complexity=complexity,
            seed_start=args.seed_start
        )
        print(f"\nGenerated {len(files)} documents")
        return 0

    else:
        parser.print_help()
        return 1


if __name__ == "__main__":
    sys.exit(main())
