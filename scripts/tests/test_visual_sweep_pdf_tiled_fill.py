"""같은 색으로 빈틈없이 이은 PDF 면에 흰 래스터 이음매를 만들지 않는다."""

import math
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

from PIL import Image

from scripts import visual_sweep as SWEEP


def tiled_fill_pdf():
    """PDF 좌표의 동일 RGB 상자를 단일 면과 인접한 작은 띠로 각각 그린다."""
    commands = ["0.25 0.5 0.75 rg", "8.25 12.5 24 50 re f"]
    # 각 띠의 끝과 다음 띠의 시작은 정확히 같다. 흰 배경은 끼어 있지 않다.
    for index in range(200):
        bottom = 12.5 + index * 0.25
        commands.append(f"40.25 {bottom:.2f} 24 0.25 re f")
    stream = "\n".join(commands).encode("ascii")
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        b"<< /Type /Pages /Count 1 /Kids [3 0 R] >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 72 72] /Contents 4 0 R >>",
        f"<< /Length {len(stream)} >>\nstream\n".encode() + stream + b"\nendstream",
    ]
    document = bytearray(b"%PDF-1.4\n")
    offsets = [0]
    for index, value in enumerate(objects, 1):
        offsets.append(len(document))
        document.extend(f"{index} 0 obj\n".encode() + value + b"\nendobj\n")
    xref = len(document)
    document.extend(b"xref\n0 5\n0000000000 65535 f \n")
    for offset in offsets[1:]:
        document.extend(f"{offset:010d} 00000 n \n".encode())
    document.extend(
        f"trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
    )
    return bytes(document)


class TiledPdfFillTests(unittest.TestCase):
    def test_tiled_and_single_pdf_faces_keep_the_same_interior_rgb(self):
        self.assertIsNotNone(shutil.which(SWEEP.PDF_RASTERIZER))
        for dpi in (96, 144):
            with self.subTest(dpi=dpi), tempfile.TemporaryDirectory() as directory:
                folder = Path(directory)
                pdf = folder / "tiled.pdf"
                pdf.write_bytes(tiled_fill_pdf())
                for command in SWEEP.pdf_raster_commands(pdf, dpi, folder / "page", None):
                    subprocess.run(command, check=True, capture_output=True, timeout=30)
                with Image.open(next(folder.glob("page-*.png"))) as image:
                    rgb = image.convert("RGB")
                    self.assertEqual(rgb.size, (math.ceil(dpi), math.ceil(dpi)))
                    # 바깥 테두리를 피하고 PDF 좌표에서 두 면의 내부를 직접 선택한다.
                    left = rgb.crop(tuple(round(v * dpi / 72) for v in (12, 15, 28, 55)))
                    right = rgb.crop(tuple(round(v * dpi / 72) for v in (44, 15, 60, 55)))
                    expected = {(64, 128, 191)}
                    self.assertEqual({color for _, color in left.getcolors(left.width * left.height)}, expected)
                    self.assertEqual(
                        {color for _, color in right.getcolors(right.width * right.height)}, expected,
                        "같은 RGB의 인접 띠를 따로 raster하면 생기는 흰 이음매를 금지한다",
                    )


if __name__ == "__main__":
    unittest.main()
