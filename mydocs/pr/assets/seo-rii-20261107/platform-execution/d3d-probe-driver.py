import ctypes, json, os, sys
from pathlib import Path
sys.path.insert(0, '\\\\wsl.localhost\\Ubuntu-24.04\\home\\edward\\mygithub\\rhwp-worktrees\\review-integration-536-seo-rii-native-hosts\\bindings\\Native\\examples')
from native_worker import NativeWorker
base = Path('C:\\Users\\edward\\AppData\\Local\\Temp\\rhwp-pr-review-seo-rii-20261107\\native-d3d')
os.add_dll_directory(str(base))
probe = ctypes.CDLL(str(base / 'rhwp_review_d3d_probe.dll'))
fn = probe.rhwp_review_d3d_probe
fn.argtypes = [ctypes.c_char_p, ctypes.c_void_p, ctypes.c_size_t]
fn.restype = ctypes.c_int
with NativeWorker(base / 'rhwp_native_ffi.dll') as worker:
 def accepted(command):
  r=worker.wait(worker.submit(command));assert r['ok'],r;return r
 info=accepted({'op':'new','replace':None})['info']
 stamp={'documentId':info['documentId'],'revision':info['revision'],'fontGeneration':info['fontGeneration'],'viewGeneration':1,'deviceGeneration':1,'deviceScale':1.0}
 worker.host.host_request(op='presentationConfigure',worker=worker.worker,stamp=stamp)
 snapshot=accepted({'op':'exportRenderSnapshot','target':{'documentId':info['documentId'],'revision':info['revision']},'fontGeneration':info['fontGeneration'],'page':0,'profile':'screen'})['snapshot']
 try:
  request={'stamp':stamp,'target':{'width':4,'height':4,'deviceGeneration':1},'clearColor':0xff336699,'placements':[{'snapshot':snapshot,'scale':1,'originX':1,'originY':1,'backgroundColor':0xffffffff}]}
  output=ctypes.create_string_buffer(65536)
  code=fn(json.dumps(request).encode(),output,len(output))
  text=output.value.decode();print('GPU_PROBE_RESPONSE '+text,flush=True)
  assert code==0,(code,text)
  result=json.loads(text)
  for y in range(4):
   for x in range(4):
    expected=[153,102,51,255] if x==0 or y==0 else [255,255,255,255]
    assert result['bgraSamples'][y*4+x]==expected,(x,y,result['bgraSamples'][y*4+x],expected)
  print('D3D12 actual producer: BGRA channels, top-left origin, target background/placement, producer fence and verification readback PASS')
 finally:
  worker.host.presentation_request(op='snapshotRelease',snapshot=snapshot)
