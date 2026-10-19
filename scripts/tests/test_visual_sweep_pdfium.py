"""PDFium 후보가 색·쪽 번호·실제 Print·가는 Type3 획을 보존하는지 검사한다."""

import hashlib
import math
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from PIL import Image

from scripts import visual_sweep as SWEEP
from scripts import visual_sweep_pdf as PDFIUM
import test_visual_sweep_pdf_tiled_fill as tiled
import test_visual_sweep_pdf_uniform_color as uniform
import test_visual_sweep_pdf_print_raster as print_raster


def thin_type3_pdf():
    """PDF 좌표로 폭0.2pt·높이7.2pt를 갖는 가는 글꼴 획을 만든다."""
    char = b"500 0 0 0 500 600 d1 0 0 16.6666667 600 re f"
    stream = b"BT /F 12 Tf 1 0 0 1 10.6 10 Tm (I) Tj ET"
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 72 72] "
        b"/Resources << /Font << /F 5 0 R >> >> /Contents 4 0 R >>",
        f"<< /Length {len(stream)} >>\nstream\n".encode() + stream + b"\nendstream",
        b"<< /Type /Font /Subtype /Type3 /FontBBox [0 0 500 600] "
        b"/FontMatrix [.001 0 0 .001 0 0] /CharProcs << /I 6 0 R >> "
        b"/Encoding << /Type /Encoding /Differences [73 /I] >> "
        b"/FirstChar 73 /LastChar 73 /Widths [500] /Resources << >> >>",
        f"<< /Length {len(char)} >>\nstream\n".encode() + char + b"\nendstream",
    ]
    data = bytearray(b"%PDF-1.4\n")
    offsets = [0]
    for index, value in enumerate(objects, 1):
        offsets.append(len(data))
        data.extend(f"{index} 0 obj\n".encode() + value + b"\nendobj\n")
    xref = len(data)
    data.extend(f"xref\n0 {len(offsets)}\n0000000000 65535 f \n".encode())
    for offset in offsets[1:]:
        data.extend(f"{offset:010d} 00000 n \n".encode())
    data.extend((f"trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\n"
                 f"startxref\n{xref}\n%%EOF\n").encode())
    return bytes(data)


class PdfiumIndependentContracts(
    uniform.UniformPdfColorTests,
    tiled.TiledPdfFillTests,
    print_raster.HancomPrintRasterTests,
):
    def setUp(self):
        # 기존 독립 RGB 기대값과 Print 검사는 바꾸지 않고 후보 명령만 주입한다.
        for name in ("PDF_RASTERIZER", "pdf_raster_commands"):
            mocked = patch.object(SWEEP, name, getattr(PDFIUM, name))
            mocked.start()
            self.addCleanup(mocked.stop)

    def test_type3_thin_stroke_remains_visible_at_supported_dpi(self):
        for dpi in (96, 144):
            with self.subTest(dpi=dpi), tempfile.TemporaryDirectory() as directory:
                folder = Path(directory)
                pdf = folder / "type3.pdf"
                pdf.write_bytes(thin_type3_pdf())
                for command in PDFIUM.pdf_raster_commands(pdf, dpi, folder / "page", None):
                    subprocess.run(command, check=True, capture_output=True, timeout=30)
                with Image.open(next(folder.glob("page-*.png"))) as image:
                    rgb = image.convert("RGB")
                    self.assertEqual(rgb.size, (dpi, dpi))
                    # 글리프 내부에 실제 검정 잉크가 있다. 전체 소실을 허용하지 않는다.
                    colors = rgb.getcolors(rgb.width * rgb.height)
                    self.assertTrue(any(min(color) < 232 for _, color in colors),
                                    "PDF Type3의 가는 글꼴 획을 래스터 단계에서 지우면 안 됨")

    def test_pdfium_provenance_identifies_actual_library_and_helper(self):
        identity = PDFIUM.pdf_rasterizer_provenance()
        self.assertTrue(identity["available"])
        self.assertEqual(identity["backend"], "pdfium")
        self.assertEqual(identity["script_sha256"],
                         hashlib.sha256(Path(PDFIUM.__file__).read_bytes()).hexdigest())
        library = Path(identity["library"])
        self.assertEqual(identity["library_sha256"], hashlib.sha256(library.read_bytes()).hexdigest())
        self.assertEqual(identity["antialias"], {"text": True, "image": True, "path": True})

    def test_out_of_range_page_fails_without_rendering_a_substitute(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            pdf = folder / "one.pdf"
            pdf.write_bytes(uniform.uniform_color_pdf()[0])
            command = PDFIUM.pdf_raster_commands(pdf, 96, folder / "page", [2])[0]
            result = subprocess.run(command, capture_output=True, timeout=30)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(list(folder.glob("page-*.png")), [])

    def test_invalid_dpi_fails_before_rendering(self):
        for dpi in (0, -1, math.inf, math.nan):
            with self.subTest(dpi=dpi):
                with self.assertRaises(ValueError):
                    PDFIUM.rasterize("not-opened.pdf", dpi, "not-written")


if __name__ == "__main__":
    unittest.main()
