from pathlib import Path
import pikepdf
from PIL import ImageCms
from pikepdf import Name, Dictionary, Array
base=Path(__file__).parent
pdf=pikepdf.Pdf.new()
def page(content,spaces=None,forms=None):
 p=pdf.add_blank_page(page_size=(200,200));p.Contents=pdf.make_stream(content.encode())
 p.Resources=Dictionary(ColorSpace=Dictionary(spaces or {}))
 if forms:p.Resources.XObject=Dictionary(forms)
 return p
page('''0 g 10 10 180 180 re f
q /Named cs 1 1 1 sc 20 20 160 160 re f
q /DeviceGray cs 0.5 sc 30 30 40 40 re f Q
1 0 0 scn 90 30 70 40 re f Q
/DeviceGray cs 0.25 sc 30 100 40 40 re f
/Named cs 0 1 0 sc 90 100 70 40 re f
0 g 0.75 sc 30 150 40 20 re f
0 0 1 rg 1 0 0 sc 90 150 70 20 re f''',{'/Named':Name.DeviceRGB})
form=pdf.make_stream(b'/Named cs 1 sc 10 10 60 60 re f')
form.Type=Name.XObject;form.Subtype=Name.Form;form.BBox=Array([0,0,80,80]);form.Resources=Dictionary(ColorSpace=Dictionary(Named=Name.DeviceGray))
page('''0 g 10 10 180 180 re f
/Named cs 0 1 0 sc
q 1 0 0 1 20 20 cm /F Do Q
1 0 0 sc 100 30 60 60 re f
BT /Named cs 1 1 1 sc ET 30 120 120 40 re f
/DeviceRGB CS 1 0 0 SC 5 w 20 110 160 60 re S''',{'/Named':Name.DeviceRGB},{'/F':form})
profile=pdf.make_stream(ImageCms.ImageCmsProfile(ImageCms.createProfile('sRGB')).tobytes());profile.N=3;profile.Alternate=Name.DeviceRGB
page('''0 g 10 10 180 180 re f
/Icc cs 1 1 1 scn 20 20 160 160 re f
0 1 0 sc 30 30 40 40 re f
q /DeviceGray cs 0 sc 90 30 70 40 re f Q
1 0 0 sc 30 100 40 40 re f
/Icc CS 0 0 1 SCN 5 w 90 100 70 40 re S''',{'/Icc':Array([Name.ICCBased,profile])})
pdf.save(base/'path-colour-controls.pdf')
