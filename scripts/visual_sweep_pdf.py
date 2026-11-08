"""PDF의 색·가는 Type3 획을 보존하는 PDFium 래스터와 실행 신원."""

import argparse
import ctypes
import hashlib
import importlib.util
import math
import sys
from functools import lru_cache
from pathlib import Path

PDF_RASTERIZER = sys.executable
PDF_RASTER_POLICY = "pdfium_smooth_rgb_exact_dpi_v2"


def pdf_raster_commands(pdf, dpi, pdf_prefix, selected_pages):
    """선택 유무에 관계없이 같은 PDFium·DPI·원본 쪽 번호를 사용한다."""
    command = [sys.executable, str(Path(__file__).resolve()),
               "--pdf", str(pdf), "--dpi", str(dpi), "--prefix", str(pdf_prefix)]
    if selected_pages:
        command += ["--pages", ",".join(str(page) for page in selected_pages)]
    return [command]


@lru_cache(maxsize=1)
def pdf_rasterizer_provenance():
    identity = {
        "program": "pypdfium2",
        "backend": "pdfium",
        "policy": PDF_RASTER_POLICY,
        "antialias": {"text": True, "image": True, "path": True},
        "coordinate_transform": {
            "scale": "dpi/72 on both axes",
            "buffer_extent": "ceil(page_points*dpi/72)",
            "annotations": True,
        },
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "available": importlib.util.find_spec("pypdfium2") is not None,
    }
    if not identity["available"]:
        return identity
    import pypdfium2_raw
    from pypdfium2.version import PDFIUM_INFO, PYPDFIUM_INFO

    package = Path(pypdfium2_raw.__file__).resolve().parent
    libraries = [package / name for name in ("libpdfium.so", "libpdfium.dylib", "pdfium.dll")]
    library = next((path for path in libraries if path.is_file()), None)
    if library is None:
        identity.update(available=False, reason="PDFium 실행 라이브러리 신원 확인 불가")
        return identity
    python = Path(sys.executable).resolve()
    identity.update(
        executable=str(python),
        executable_sha256=hashlib.sha256(python.read_bytes()).hexdigest(),
        library=str(library),
        library_sha256=hashlib.sha256(library.read_bytes()).hexdigest(),
        version=str(PDFIUM_INFO),
        wrapper_version=str(PYPDFIUM_INFO),
        version_exit_code=0,
    )
    return identity


def rasterize(pdf, dpi, prefix, selected_pages=None):
    """기준 PDF의 페이지를 흰 배경 RGB PNG로 출력한다. 실패 시 대체하지 않는다."""
    if not math.isfinite(dpi) or dpi <= 0:
        raise ValueError("DPI는 유한한 양수여야 합니다")
    if not pdf_rasterizer_provenance()["available"]:
        raise RuntimeError("실행 신원이 확인되는 pypdfium2가 필요합니다")
    import pypdfium2
    import pypdfium2_raw

    prefix = Path(prefix)
    prefix.parent.mkdir(parents=True, exist_ok=True)
    with pypdfium2.PdfDocument(pdf) as document:
        pages = list(selected_pages) if selected_pages else list(range(1, len(document) + 1))
        if any(page < 1 or page > len(document) for page in pages):
            raise ValueError("선택 페이지가 기준 PDF 범위 밖입니다")
        for page_number in pages:
            page = document[page_number - 1]
            try:
                scale = dpi / 72
                width = math.ceil(page.get_width() * scale)
                height = math.ceil(page.get_height() * scale)
                bitmap = pypdfium2.PdfBitmap.new_native(
                    width, height, pypdfium2_raw.FPDFBitmap_BGR,
                )
                try:
                    bitmap.fill_rect((255, 255, 255, 255), 0, 0, width, height)
                    # 버퍼 크기 올림으로 용지를 늘리지 않는다. PDFium의 source
                    # rotation/CropBox display matrix 뒤에 같은 물리 배율을 결합한다.
                    matrix = pypdfium2_raw.FS_MATRIX(scale, 0, 0, scale, 0, 0)
                    clip = pypdfium2_raw.FS_RECTF(0, 0, width, height)
                    # 기존 page.render의 annotation·smooth 기본값을 보존한다.
                    pypdfium2_raw.FPDF_RenderPageBitmapWithMatrix(
                        bitmap, page, ctypes.byref(matrix), ctypes.byref(clip),
                        pypdfium2_raw.FPDF_ANNOT,
                    )
                    # PDFium 소유 버퍼가 닫히기 전에 별도의 RGB 이미지로 복사한다.
                    with bitmap.to_pil().convert("RGB") as image:
                        image.save(prefix.parent / f"{prefix.name}-{page_number:03d}.png")
                finally:
                    bitmap.close()
            finally:
                page.close()



def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pdf", required=True)
    parser.add_argument("--dpi", type=float, required=True)
    parser.add_argument("--prefix", required=True)
    parser.add_argument("--pages")
    arguments = parser.parse_args()
    pages = [int(value) for value in arguments.pages.split(",")] if arguments.pages else None
    rasterize(arguments.pdf, arguments.dpi, arguments.prefix, pages)


if __name__ == "__main__":
    main()
