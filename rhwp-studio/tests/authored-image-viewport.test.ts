// 합성 transport 계약: 실물 문서의 시각 검증을 대신하지 않는다.
import test from 'node:test';
import assert from 'node:assert/strict';
import { collectFlowImagePaintOps, visibleFlowImageBbox, flowImageOpsFromNarrowQuery } from '../src/view/flow-image-clip.ts';

const owner = { x: 0, y: 0, width: 100, height: 50 };
const painted = { x: 200, y: 100, width: 100, height: 50 };
const clip = { x: 225, y: 110, width: 50, height: 30 };
const path: [number, number][] = [[225,110],[275,110],[275,140],[225,140]];

test('저장 viewport는 조상 clip과 교차하고 이동한 실제 그림을 보존한다', () => {
  const [image] = collectFlowImagePaintOps({kind:'clipRect',
    clip:{x:200,y:100,width:150,height:100}, child:{kind:'leaf',ops:[{
      type:'image',bbox:owner,paintBbox:painted,paintClip:clip,paintClipPath:path,
      mime:'image/png',base64:'AA==',
    }]}}, op=>op.type==='image');
  assert.deepEqual(image.clip,clip);
  assert.deepEqual(image.paintClipPath,path);
  assert.deepEqual(visibleFlowImageBbox(image),clip);
  assert.deepEqual(image.bbox,owner,'논리 개체 소속을 paint 원점으로 바꾸지 않는다');
});

test('좁은 질의도 같은 원점과 다각형을 전달한다', () => {
  const images = flowImageOpsFromNarrowQuery(JSON.stringify({cacheable:true,images:[{
    bbox:owner,paintBbox:painted,clip,paintClipPath:path,mime:'image/png',sourceImageKey:'source',
  }]}),()=>'blob:test');
  assert.ok(images);
  assert.deepEqual(images[0].paintClipPath,path);
  assert.deepEqual(visibleFlowImageBbox(images[0]),clip);
});
