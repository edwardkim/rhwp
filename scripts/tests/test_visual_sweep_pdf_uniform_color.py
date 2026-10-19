"""PDF의 상수 RGB 면은 분수 좌표에서 축소해도 같은 내부 색을 유지해야 한다."""

import math
import shutil
import subprocess
import tempfile
import unittest
import zlib
from pathlib import Path

from PIL import Image

from scripts import visual_sweep as SWEEP


def uniform_color_pdf(page_count=1):
    """손실 없는 DeviceRGB 표본과 독립 PDF 단위로만 정한 상자들을 작성한다."""
    values = [0, 64, 128, 192, 231, 232, 233, 255]
    image_start = 3 + 2 * page_count
    resources = " ".join(f"/I{i} {image_start+i} 0 R" for i in range(len(values)))
    pages = " ".join(f"{3+2*i} 0 R" for i in range(page_count))
    objects = [b"<< /Type /Catalog /Pages 2 0 R >>",
               f"<< /Type /Pages /Count {page_count} /Kids [{pages}] >>".encode()]
    for page in range(page_count):
        objects.append((f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 841] "
                        f"/Resources << /XObject << {resources} >> >> "
                        f"/Contents {4+2*page} 0 R >>").encode())
        commands = []
        for i in range(len(values)):
            x, y = 140.325 + (i % 2) * 260, 148.997 + (i // 2) * 180
            commands.append(f"q 153.038 0 0 83.668 {x:.3f} {841-y-83.668:.3f} cm /I{i} Do Q")
        stream = "\n".join(commands).encode()
        objects.append(f"<< /Length {len(stream)} >>\nstream\n".encode()
                       + stream + b"\nendstream")
    for value in values:
        stream = zlib.compress(bytes([value]) * (1201 * 657 * 3))
        objects.append((f"<< /Type /XObject /Subtype /Image /Width 1201 /Height 657 "
                        f"/BitsPerComponent 8 /ColorSpace /DeviceRGB /Filter /FlateDecode "
                        f"/Length {len(stream)} >>\nstream\n").encode()
                       + stream + b"\nendstream")
    data = bytearray(b"%PDF-1.4\n")
    offsets = [0]
    for i, obj in enumerate(objects, 1):
        offsets.append(len(data))
        data.extend(f"{i} 0 obj\n".encode() + obj + b"\nendobj\n")
    xref = len(data)
    data.extend(f"xref\n0 {len(offsets)}\n0000000000 65535 f \n".encode())
    for offset in offsets[1:]:
        data.extend(f"{offset:010d} 00000 n \n".encode())
    data.extend((f"trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\n"
                 f"startxref\n{xref}\n%%EOF\n").encode())
    return bytes(data), values


class UniformPdfColorTests(unittest.TestCase):
    def raster(self, directory, dpi, selected_pages, page_count):
        pdf = directory / "uniform.pdf"
        contents, values = uniform_color_pdf(page_count)
        pdf.write_bytes(contents)
        prefix = directory / "reference"
        commands = SWEEP.pdf_raster_commands(pdf, dpi, prefix, selected_pages)
        for command in commands:
            self.assertIsNotNone(shutil.which(command[0]), f"필수 PDF 도구: {command[0]}")
            subprocess.run(command, check=True, capture_output=True, timeout=30)
        return sorted(directory.glob("reference-*.png")), values

    def test_minification_preserves_uniform_rgb_without_reclassifying_ink(self):
        for dpi in [96, 144]:
            with self.subTest(dpi=dpi), tempfile.TemporaryDirectory() as temporary:
                paths, values = self.raster(Path(temporary), dpi, None, 1)
                self.assertEqual(len(paths), 1)
                with Image.open(paths[0]) as raster:
                    image = raster.convert("RGB")
                    self.assertEqual(image.size, (math.ceil(595*dpi/72), math.ceil(841*dpi/72)))
                    for i, value in enumerate(values):
                        # 상자의 경계가 아닌 내부를 PDF 좌표에서 직접 선택한다.
                        x = round((140.325 + (i % 2)*260 + 50)*dpi/72)
                        y = round((148.997 + (i // 2)*180 + 40)*dpi/72)
                        self.assertEqual(image.getpixel((x, y)), (value,)*3,
                                         f"균일 RGB {value}의 축소·배경 색을 바꾸면 안 됨")

    def test_selection_preserves_original_page_number_and_physical_size(self):
        with tempfile.TemporaryDirectory() as temporary:
            paths, _ = self.raster(Path(temporary), 96, [3], 3)
            self.assertEqual([SWEEP.page_num(path) for path in paths], [3])
            with Image.open(paths[0]) as image:
                self.assertEqual(image.size, (math.ceil(595*96/72), math.ceil(841*96/72)))


if __name__ == "__main__":
    unittest.main()
