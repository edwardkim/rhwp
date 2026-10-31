from PIL import Image
from pathlib import Path
import subprocess,json,hashlib

out=Path(__file__).resolve().parent
items=[x for x in json.loads((out/'jpeg-inputs.json').read_text()) if 'sampling' not in x]
for progressive in [False,True]:
    for w,h in [(1,1),(1,9),(9,1),(9,9),(17,17),(33,1),(33,19),(65,33)]:
        ppm=out/f'mixed-{w}x{h}.ppm'
        image=Image.new('RGB',(w,h))
        image.putdata([((x*31)%256,(y*19)%256,((x+y)*23)%256) for y in range(h) for x in range(w)])
        image.save(ppm)
        name=f'{"progressive" if progressive else "baseline"}-{w}x{h}-mixed-Y1x2-Cb2x2-Cr2x2.jpg'
        dest=out/'jpeg-inputs'/name
        cmd=['cjpeg','-quality','90','-sample','1x2,2x2,2x2']
        cmd+=['-progressive'] if progressive else []
        cmd+=['-outfile',str(dest),str(ppm)]
        result=subprocess.run(cmd,capture_output=True)
        assert result.returncode==0,result.stderr
        check=subprocess.run(['djpeg','-outfile',str(out/(name+'.ppm')),str(dest)],capture_output=True)
        assert check.returncode==0,check.stderr
        items.append({'name':name,'width':w,'height':h,'sampling':['1x2','2x2','2x2'],'progressive':progressive,'bytes':dest.stat().st_size,'sha256':hashlib.sha256(dest.read_bytes()).hexdigest(),'generator_command':cmd,'independent_djpeg_exit':check.returncode})
(out/'jpeg-inputs.json').write_text(json.dumps(items,indent=2)+'\n')
print('independently generated and decoded mixed JPEGs:',sum('sampling' in x for x in items))
