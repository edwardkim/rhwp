import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
const pkg=path.resolve(process.argv[2]);
const {default:init,HwpDocument}=await import(pathToFileURL(path.join(pkg,'rhwp.js')));
await init({module_or_path:fs.readFileSync(path.join(pkg,'rhwp_bg.wasm'))});
const d=HwpDocument.createEmpty();const checks=[];
try {
 d.createBlankDocument();const last=d.getParagraphCount(0)-1;
 const made=JSON.parse(d.createTable(0,last,0,2,3));const para=made.paraIdx,ctrl=made.controlIdx;
 const dims=p=>JSON.parse(d.getTableDimensionsByPath(0,para,JSON.stringify(p)));
 const full=[{controlIndex:ctrl,cellIndex:0,cellParaIndex:0}];
 const expected=dims(full);assert.equal(expected.rowCount,2);assert.equal(expected.colCount,3);assert.equal(expected.cellCount,6);
 for (const step of [{controlIndex:ctrl},{controlIndex:ctrl,cellIndex:0},{controlIndex:ctrl,cellIndex:99,cellParaIndex:99}])assert.deepEqual(dims([step]),expected);
 checks.push('last-step optional cell fields');
 const shortBox=d.getTableCellBboxesByPath(0,para,JSON.stringify([{controlIndex:ctrl}]));assert.equal(shortBox,d.getTableCellBboxesByPath(0,para,JSON.stringify(full)));assert.equal(JSON.parse(shortBox).length,6);checks.push('actual cell bboxes identical');
 for(const bad of [[],[{}],[{controlIndex:999999}],[{controlIndex:ctrl,cellIndex:'x'}]])assert.throws(()=>dims(bad));checks.push('invalid paths rejected');
 const outer=[{controlIndex:ctrl,cellIndex:4,cellParaIndex:0}];const inner=JSON.parse(d.createTableInCellByPath(0,para,JSON.stringify(outer),0,JSON.stringify({rowCount:3,colCount:4,treatAsChar:true})));
 const first={controlIndex:ctrl,cellIndex:4,cellParaIndex:inner.cellParaIdx};const nested=[first,{controlIndex:inner.controlIdx}];const nestedFull=[first,{controlIndex:inner.controlIdx,cellIndex:0,cellParaIndex:0}];assert.deepEqual(dims(nested),dims(nestedFull));assert.equal(dims(nested).rowCount,3);assert.equal(dims(nested).colCount,4);assert.equal(dims(nested).cellCount,12);
 for(const bad of [[{controlIndex:ctrl,cellIndex:4},{controlIndex:inner.controlIdx}],[{controlIndex:ctrl},{controlIndex:inner.controlIdx}]])assert.throws(()=>dims(bad));checks.push('nested table and required outer steps');
 console.log(JSON.stringify({status:'passed',checks,dimensions:expected,nestedDimensions:dims(nested),bboxCount:JSON.parse(shortBox).length},null,2));
}finally{d.free();}
