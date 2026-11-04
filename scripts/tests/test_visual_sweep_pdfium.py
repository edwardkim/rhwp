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


def one_inch_square_pdf(width, height, rotation, origin, annotation=False):
    """구현과 독립적인 72pt 정사각형을 CropBox 좌상단에서 9pt 떨어뜨린다."""
    left, bottom = origin
    stream = f"0 g {left+9} {bottom+height-81} 72 72 re f".encode()
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>",
        (f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {left+width} {bottom+height}] "
         f"/CropBox [{left} {bottom} {left+width} {bottom+height}] /Rotate {rotation} "
         f"/Resources << >> /Contents 4 0 R "
         f"{'/Annots [5 0 R]' if annotation else ''} >>").encode(),
        f"<< /Length {len(stream)} >>\nstream\n".encode() + stream + b"\nendstream",
    ]
    if annotation:
        appearance = b"1 0 0 rg 0 0 12 12 re f"
        objects.extend([
            b"<< /Type /Annot /Subtype /Square /Rect [100 78 112 90] /F 4 /AP << /N 6 0 R >> >>",
            (f"<< /Type /XObject /Subtype /Form /BBox [0 0 12 12] /Resources << >> "
             f"/Length {len(appearance)} >>\nstream\n").encode() + appearance + b"\nendstream",
        ])
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

    def check_physical_geometry(self, origins, exact_pixel_length):
        for width, height in [(200, 100), (200.25, 100.5)]:
            for rotation in (0, 90, 180, 270):
                for dpi in (72, 96, 144):
                    for origin in origins:
                        with self.subTest(size=(width, height), rotation=rotation,
                                          dpi=dpi, origin=origin), tempfile.TemporaryDirectory() as directory:
                            folder = Path(directory)
                            pdf = folder / "one-inch.pdf"
                            pdf.write_bytes(one_inch_square_pdf(width, height, rotation, origin))
                            PDFIUM.rasterize(pdf, dpi, folder / "page", [1])
                            with Image.open(folder / "page-001.png") as image:
                                mask = image.convert("L").point([255 if v < 128 else 0 for v in range(256)])
                                bbox = mask.getbbox()
                                self.assertIsNotNone(bbox)
                                if exact_pixel_length:
                                    self.assertEqual((bbox[2]-bbox[0], bbox[3]-bbox[1]), (dpi, dpi),
                                                     "72pt 도형은 용지의 출력 크기 올림과 무관하게 정확히 1인치다")
                                if rotation == 0:
                                    expected = (9, 9, 81, 81)
                                elif rotation == 90:
                                    expected = (height-81, 9, height-9, 81)
                                elif rotation == 180:
                                    expected = (width-81, height-81, width-9, height-9)
                                else:
                                    expected = (9, width-81, 81, width-9)
                                for actual, point in zip(bbox, expected):
                                    self.assertLessEqual(abs(actual-point*dpi/72), 1,
                                                         "회전·CropBox의 원점과 내용 소속 보존")
                                page_size = (height, width) if rotation in (90, 270) else (width, height)
                                self.assertEqual(image.size, tuple(math.ceil(v*dpi/72) for v in page_size))

    def test_physical_dpi_preserves_one_inch_on_fractional_pixel_pages(self):
        self.check_physical_geometry([(0, 0)], exact_pixel_length=True)

    def test_fractional_crop_origin_and_rotation_keep_source_geometry(self):
        # 소수 원점의 float32 연산과 채움의 바깥 픽셀 정수화는 1픽셀 안에서 판독한다.
        # 정확한 인치 길이는 위의 픽셀 정렬된 독립 대조군이 별도로 검사한다.
        self.check_physical_geometry([(11.25, 7.5)], exact_pixel_length=False)

    def test_source_annotation_appearance_remains_visible(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            pdf = folder / "annotation.pdf"
            pdf.write_bytes(one_inch_square_pdf(200, 100, 0, (0, 0), annotation=True))
            PDFIUM.rasterize(pdf, 72, folder / "page", [1])
            with Image.open(folder / "page-001.png") as image:
                self.assertEqual(image.convert("RGB").getpixel((105, 15)), (255, 0, 0),
                                 "기존 annotation 표시 계약을 DPI 보정에서 지우지 않는다")

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
