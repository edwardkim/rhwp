#!/usr/bin/env python3
"""Rebuild #8183 PDF codec contract inputs with independent Pillow encoders.

Run from the repository root with venv/bin/python. Requires Poppler, MuPDF,
Ghostscript, Pillow with libtiff/OpenJPEG. Re-run Native/fresh WASM verification
before committing regenerated inputs. This script does not generate goldens.
"""
from pathlib import Path
import io
import struct
import subprocess
import tempfile

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
PDF = ROOT / "pdf/issue8183"
FIXTURES = ROOT / "tests/fixtures/issue8183"


def stream(fields, data):
    return f"<< {fields} /Length {len(data)} >>\nstream\n".encode() + data + b"\nendstream"


def write_pdf(name, data, width, height, fields, extras=(), size=(160, 80), repeats=1):
    content = " ".join([f"q {size[0]} 0 0 {size[1]} 0 0 cm /Im Do Q"] * repeats).encode()
    objects = [b"<< /Type /Catalog /Pages 2 0 R >>", b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
               f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {size[0]} {size[1]}] /Resources << /XObject << /Im 5 0 R >> >> /Contents 4 0 R >>".encode(),
               stream("", content), stream(f"/Type /XObject /Subtype /Image /Width {width} /Height {height} {fields}", data), *extras]
    output = b"%PDF-1.7\n"
    offsets = [0]
    for index, obj in enumerate(objects, 1):
        offsets.append(len(output))
        output += f"{index} 0 obj\n".encode() + obj + b"\nendobj\n"
    xref = len(output)
    output += f"xref\n0 {len(offsets)}\n0000000000 65535 f \n".encode()
    output += b"".join(f"{value:010} 00000 n \n".encode() for value in offsets[1:])
    output += f"trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
    (PDF / f"{name}.pdf").write_bytes(output)


def jpx(image, **options):
    data = io.BytesIO()
    image.save(data, format="JPEG2000", irreversible=False, **options)
    return data.getvalue()


def segment(number, kind, data):
    return struct.pack(">IBBBI", number, kind, 0, 1, len(data)) + data


def jbig2(image):
    image = image.convert("L").point(lambda value: 255 if value > 180 else 0, "1")
    width, height = image.size
    data = io.BytesIO()
    # Each TIFF strip starts a new reference line and ends with EOFB. A JBIG2
    # generic region consumes one MMR stream, so selecting just strip zero clips.
    image.save(data, format="TIFF", compression="group4", strip_size=width * height, tiffinfo={262: 0})
    data.seek(0)
    tiff = Image.open(data)
    assert len(tiff.tag_v2[273]) == 1
    at, length = tiff.tag_v2[273][0], tiff.tag_v2[279][0]
    encoded = segment(1, 48, struct.pack(">IIIIBH", width, height, 0, 0, 0, 0))
    return encoded + segment(2, 38, struct.pack(">IIIIBB", width, height, 0, 0, 0, 1) + data.getvalue()[at:at + length])


def split_jbig2(data):
    data = data[9 if data[8] & 2 else 13:]
    position, page, globals_ = 0, bytearray(), bytearray()
    while position < len(data):
        start = position
        number = int.from_bytes(data[position:position + 4], "big")
        flags = data[position + 4]
        count = data[position + 5] >> 5
        assert count <= 4
        position += 6 + count * (1 if number <= 256 else 2 if number <= 65536 else 4)
        count = 4 if flags & 64 else 1
        owner = int.from_bytes(data[position:position + count], "big")
        position += count
        length = int.from_bytes(data[position:position + 4], "big")
        position += 4
        assert length != 0xffffffff
        position += length
        if flags & 63 not in (49, 51):
            (page if owner else globals_).extend(data[start:position])
    return bytes(page), bytes(globals_)


def box(kind, data):
    return struct.pack(">I4s", len(data) + 8, kind) + data


def palette(data):
    output, position = bytearray(), 0
    while position < len(data):
        size = int.from_bytes(data[position:position + 4], "big")
        kind, content = data[position + 4:position + 8], data[position + 8:position + size]
        if kind == b"jp2h":
            children, index = [], 0
            while index < len(content):
                length = int.from_bytes(content[index:index + 4], "big")
                key, value = content[index + 4:index + 8], content[index + 8:index + length]
                if key == b"colr":
                    value = bytes([1, 0, 0]) + struct.pack(">I", 16)
                children.append(box(key, value))
                index += length
            children += [box(b"pclr", struct.pack(">HB", 2, 3) + bytes([7, 7, 7, 255, 0, 0, 0, 255, 0])),
                         box(b"cmap", b"".join(struct.pack(">HBB", 0, 1, channel) for channel in range(3)))]
            content = b"".join(children)
        output.extend(box(kind, content))
        position += size
    return bytes(output)


def main():
    PDF.mkdir(parents=True, exist_ok=True)
    (PDF / "reference").mkdir(exist_ok=True)
    FIXTURES.mkdir(exist_ok=True)
    checker = Image.new("L", (16, 8))
    checker.putdata([255 if (x // 4 + y // 2) % 2 else 0 for y in range(8) for x in range(16)])
    jb, jp = jbig2(checker), jpx(checker, num_resolutions=1)
    write_pdf("jbig2", jb, 16, 8, "/Filter /JBIG2Decode /ColorSpace /DeviceGray /BitsPerComponent 1")
    write_pdf("jpx", jp, 16, 8, "/Filter /JPXDecode /ColorSpace /DeviceGray /BitsPerComponent 8")
    rgb = Image.new("RGB", (4, 2))
    rgb.putdata([(255, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 255)] * 2)
    rgb_data = jpx(rgb, num_resolutions=1)
    alpha = rgb.convert("RGBA")
    alpha.putalpha(Image.frombytes("L", (4, 2), bytes([0, 64, 128, 255] * 2)))
    pre = Image.new("RGBA", (4, 2))
    pre.putdata([(128, 0, 0, 128), (0, 64, 0, 64), (0, 0, 255, 255), (0, 0, 0, 0)] * 2)
    for name, image, mode in [("jpx-alpha", alpha, 1), ("jpx-premult", pre, 2)]:
        write_pdf(name, jpx(image, num_resolutions=1), 4, 2, f"/Filter /JPXDecode /ColorSpace /DeviceRGB /SMaskInData {mode}")
    indices = Image.frombytes("L", (4, 2), bytes([0, 1, 0, 1] * 2))
    table = palette(jpx(indices, num_resolutions=1))
    write_pdf("jpx-palette", table, 4, 2, "/Filter /JPXDecode")
    write_pdf("jpx-indexed", table, 4, 2, "/Filter /JPXDecode /ColorSpace [/Indexed /DeviceRGB 1 <0000ff00ffff>]")
    tiles = Image.new("RGB", (64, 32))
    tiles.putdata([(x * 3 % 256, y * 7 % 256, (x + y) * 5 % 256) for y in range(32) for x in range(64)])
    write_pdf("jpx-tiles", jpx(tiles, num_resolutions=3, tile_size=(16, 16)), 64, 32, "/Filter /JPXDecode /ColorSpace /DeviceRGB /BitsPerComponent 8")
    for name, codec, data, fields, key in [("jbig2-mask", "JBIG2Decode", jb, "/ImageMask true /BitsPerComponent 1", "Mask"),
                                          ("jpx-mask", "JPXDecode", jp, "/ColorSpace /DeviceGray /BitsPerComponent 8", "SMask")]:
        mask = stream(f"/Type /XObject /Subtype /Image /Width 16 /Height 8 /Filter /{codec} {fields}", data)
        write_pdf(name, rgb_data, 4, 2, f"/Filter /JPXDecode /ColorSpace /DeviceRGB /BitsPerComponent 8 /{key} 6 0 R", extras=[mask])
    for name in ["generic_template0", "symbol_global"]:
        page, globals_ = split_jbig2((FIXTURES / "source" / f"{name}.jbig2").read_bytes())
        write_pdf("jbig2-" + name, page, 163, 109, "/Filter /JBIG2Decode /ColorSpace /DeviceGray /BitsPerComponent 1 /DecodeParms << /JBIG2Globals 6 0 R >>", extras=[stream("", globals_)])
    with tempfile.TemporaryDirectory(prefix="issue8183-scan-") as directory:
        for suffix, height in [("", 1123), ("-200", 2339)]:
            path = Path(directory) / "source"
            subprocess.run(["pdftoppm", "-f", "1", "-singlefile", "-scale-to", str(height), "-gray", "-png", str(ROOT / "pdf/hwp_table_test-2022.pdf"), str(path)], check=True)
            image = Image.open(path.with_suffix(".png")).convert("L")
            width, height = image.size
            data = jpx(image, num_resolutions=6)
            write_pdf("scan-jpx" + suffix, data, width, height, "/Filter /JPXDecode /ColorSpace /DeviceGray /BitsPerComponent 8", size=(595, 842))
            write_pdf("scan-jbig2" + suffix, jbig2(image), width, height, "/Filter /JBIG2Decode /ColorSpace /DeviceGray /BitsPerComponent 1", size=(595, 842))
            jpeg = io.BytesIO()
            image.save(jpeg, format="JPEG", quality=90)
            write_pdf("scan-jpeg-control" + suffix, jpeg.getvalue(), width, height, "/Filter /DCTDecode /ColorSpace /DeviceGray /BitsPerComponent 8", size=(595, 842))
            if suffix:
                (FIXTURES / "scan-source-gray-200.png").write_bytes(path.with_suffix(".png").read_bytes())
                image.point(lambda value: 255 if value > 180 else 0, "1").save(FIXTURES / "scan-source-mono-200.png")
                write_pdf("work-limit-jpx", data, width, height, "/Filter /JPXDecode /ColorSpace /DeviceGray /BitsPerComponent 8", size=(595, 842), repeats=3)
    for codec in ["JBIG2Decode", "JPXDecode"]:
        write_pdf("corrupt-" + codec, b"bad", 16, 8, f"/Filter /{codec} /ColorSpace /DeviceGray /BitsPerComponent 8")
    for name in ["jpx-alpha", "jpx-premult", "jpx-indexed"]:
        subprocess.run(["gs", "-q", "-dSAFER", "-dBATCH", "-dNOPAUSE", "-sDEVICE=pdfwrite", "-dCompatibilityLevel=1.4", "-dAutoFilterColorImages=false", "-dColorImageFilter=/FlateEncode", "-dPassThroughJPXImages=false", "-dPassThroughJPEGImages=false", f"-sOutputFile={PDF / 'reference' / (name + '-gs.pdf')}", str(PDF / (name + ".pdf"))], check=True)
    print("Inputs regenerated. Ghostscript metadata may change PDF hashes; revalidate and update provenance explicitly.")


if __name__ == "__main__":
    main()
