"""Generate public resource admission controls; no user document is read."""
import pathlib
import sys

import pikepdf


def main():
    out = pathlib.Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    pdf = pikepdf.Pdf.new()
    page = pdf.add_blank_page(page_size=(300, 200))
    pdf.Root.Probe = pikepdf.Array([0] * 60000)
    page.Contents = pikepdf.Stream(pdf, b"q Q")
    pdf.save(out / "resource-objects-nonempty.pdf")

    pdf = pikepdf.Pdf.new()
    page = pdf.add_blank_page(page_size=(300, 200))
    page.Contents = pikepdf.Stream(pdf, b"q Q\n" * 320000)
    pdf.save(out / "resource-dense.pdf")

    pdf = pikepdf.Pdf.new()
    image = pikepdf.Stream(pdf, bytes([137]) * (7680 * 4320))
    image.Type = pikepdf.Name.XObject
    image.Subtype = pikepdf.Name.Image
    image.Width = 7680
    image.Height = 4320
    image.ColorSpace = pikepdf.Name.DeviceGray
    image.BitsPerComponent = 8
    for _ in range(2):
        page = pdf.add_blank_page(page_size=(300, 200))
        page.Resources = pikepdf.Dictionary(XObject=pikepdf.Dictionary(Im=image))
        page.Contents = pikepdf.Stream(pdf, b"q 240 0 0 135 30 30 cm /Im Do Q")
    pdf.save(out / "resource-8k.pdf")


if __name__ == "__main__":
    main()
