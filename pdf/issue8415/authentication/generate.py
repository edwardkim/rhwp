"""Generate public synthetic authentication controls; requires pikepdf.

Usage: python3 generate.py OUTPUT_DIRECTORY
Random encryption salts/IVs make regenerated PDF hashes differ. No external
fonts or user documents are read; the checked-in manifest identifies the inputs.
"""
import pathlib
import sys
import pikepdf

out = pathlib.Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)
doc = pikepdf.Pdf.new()
page = doc.add_blank_page(page_size=(300, 200))
font = pikepdf.Dictionary(Type=pikepdf.Name.Font, Subtype=pikepdf.Name.Type1,
                         BaseFont=pikepdf.Name.Helvetica)
image = doc.make_stream(b'\xff\x00\x00')
image.Type = pikepdf.Name.XObject
image.Subtype = pikepdf.Name.Image
image.Width = image.Height = 1
image.BitsPerComponent = 8
image.ColorSpace = pikepdf.Name.DeviceRGB
page.Resources = pikepdf.Dictionary(Font=pikepdf.Dictionary(F1=font),
                                    XObject=pikepdf.Dictionary(Im1=image))
page.Contents = doc.make_stream(
    b'BT /F1 18 Tf 20 140 Td (Password import control) Tj ET '
    b'q 40 0 0 40 20 60 cm /Im1 Do Q')
doc.save(out / 'password-control.pdf')
for revision in range(2, 7):
    doc.save(out / f'password-r{revision}.pdf',
             encryption=pikepdf.Encryption(user='synthetic-open', owner='synthetic-owner',
                                            R=revision, aes=revision >= 4, metadata=revision >= 4),
             object_stream_mode=pikepdf.ObjectStreamMode.generate if revision >= 4
             else pikepdf.ObjectStreamMode.disable)
permissions = pikepdf.Permissions(accessibility=False, extract=False, modify_annotation=False,
    modify_assembly=False, modify_form=False, modify_other=False, print_lowres=False, print_highres=False)
doc.save(out / 'password-empty.pdf', encryption=pikepdf.Encryption(
    user='', owner='synthetic-owner', R=6, allow=permissions))
doc.save(out / 'password-unicode.pdf', encryption=pikepdf.Encryption(
    user='합성-암호-8415', owner='synthetic-owner', R=6))
doc.save(out / 'password-metadata-clear.pdf', encryption=pikepdf.Encryption(
    user='synthetic-open', owner='synthetic-owner', R=6, metadata=False),
    object_stream_mode=pikepdf.ObjectStreamMode.generate)

doc.save(out / 'password-long-unicode.pdf', encryption=pikepdf.Encryption(
    user=('가' * 43).encode('utf-8')[:127], owner='synthetic-owner', R=6))

# Preserve ciphertext and append a new encryption dictionary revision. V5 key
# derivation consumes fixed prefixes; QPDF accepts longer strings. This models
# padded producer fields without copying any user document or encryption key.
source = (out / 'password-r6.pdf').read_bytes()
with pikepdf.open(out / 'password-r6.pdf', password='synthetic-open') as encrypted:
    encryption = pikepdf.Dictionary(encrypted.trailer.Encrypt)
    for key in ['/O', '/U', '/OE', '/UE', '/Perms']:
        encryption[key] = pikepdf.String(bytes(encryption[key]) + b'\0' * 79)
    obj, gen = encrypted.trailer.Encrypt.objgen
    root, root_gen = encrypted.Root.objgen
    previous = int(source.rsplit(b'startxref', 1)[1].split()[0])
    offset = len(source) + 1
    extra = f'\n{obj} {gen} obj\n'.encode() + encryption.unparse() + b'\nendobj\n'
    xref = len(source) + len(extra)
    extra += (f'xref\n{obj} 1\n{offset:010d} {gen:05d} n \ntrailer\n'
              f'<< /Size {encrypted.trailer.Size} /Root {root} {root_gen} R '
              f'/Encrypt {obj} {gen} R /Prev {previous} /ID ').encode()
    extra += encrypted.trailer.ID.unparse() + f' >>\nstartxref\n{xref}\n%%EOF\n'.encode()
    (out / 'password-padded-fields.pdf').write_bytes(source + extra)
