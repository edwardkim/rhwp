"""Public, generated scan-sized JPEG controls; no private document pixels."""
from pathlib import Path
from io import BytesIO
from PIL import Image, ImageDraw


def generate():
    objects = [b'', b'']
    pages = []
    for number, (w, h) in enumerate([(1928, 2725)] * 3 + [(1929, 2728), (1929, 2729)], 1):
        image = Image.new('RGB', (w, h), 'white')
        draw = ImageDraw.Draw(image)
        draw.rectangle((40, 40, w - 41, h - 41), outline='black', width=5)
        colors = ['#1769aa', '#f28a18', '#16a36a', '#9c36a6', '#555555']
        for row in range(12):
            y = 160 + row * 190
            draw.rectangle((90, y, w - 91, y + 110), fill=colors[(row + number) % len(colors)])
            draw.text((115, y + 40), f'PUBLIC GENERATED JPEG BUFFER CONTROL - PAGE {number} - ROW {row + 1}', fill='white')
            draw.line((90, y + 140, w - 91, y + 140), fill='black', width=2)
        for i in range(number):
            draw.rectangle((100 + i * 80, 60, 150 + i * 80, 110), fill='black')
        jpeg = BytesIO()
        image.save(jpeg, format='JPEG', quality=90, subsampling=0, progressive=False, optimize=False)
        data = jpeg.getvalue()
        image_id = len(objects) + 1
        objects.append(f'<< /Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {len(data)} >>\nstream\n'.encode() + data + b'\nendstream')
        width = 756 * w / h
        content = f'q {width:.8f} 0 0 756 {(612-width)/2:.8f} 18 cm /Im Do Q'.encode()
        content_id = len(objects) + 1
        objects.append(f'<< /Length {len(content)} >>\nstream\n'.encode() + content + b'\nendstream')
        pages.append(len(objects) + 1)
        objects.append(f'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /XObject << /Im {image_id} 0 R >> >> /Contents {content_id} 0 R >>'.encode())
    objects[0] = b'<< /Type /Catalog /Pages 2 0 R >>'
    objects[1] = f'<< /Type /Pages /Count {len(pages)} /Kids [ {" ".join(f"{i} 0 R" for i in pages)} ] >>'.encode()
    output = bytearray(b'%PDF-1.7\n%\xe2\xe3\xcf\xd3\n')
    offsets = []
    for i, value in enumerate(objects, 1):
        offsets.append(len(output))
        output.extend(f'{i} 0 obj\n'.encode() + value + b'\nendobj\n')
    xref = len(output)
    output.extend(f'xref\n0 {len(objects) + 1}\n0000000000 65535 f \n'.encode())
    for offset in offsets:
        output.extend(f'{offset:010d} 00000 n \n'.encode())
    output.extend(f'trailer\n<< /Size {len(objects)+1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode())
    return bytes(output)


if __name__ == '__main__':
    path = Path(__file__).with_name('scan-jpeg-buffer-controls.pdf')
    path.write_bytes(generate())
    print(path, path.stat().st_size)
