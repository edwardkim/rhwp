import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';

// 제품 대화상자 메서드를 그대로 실행한다. DOM 입력과 부수 미리보기만 대역이다.
const ui = fileURLToPath(new URL('../src/ui/', import.meta.url));
class Option {
  value = ''; textContent = ''; dataset: Record<string,string> = {};
  parent: Select | null = null;
  remove() { if (this.parent) this.parent.options = this.parent.options.filter(x => x !== this); }
}
class Select {
  options: Option[] = [];
  disabled = false;
  private selected = '';
  constructor() { for (const value of ['none','solid']) { const o=new Option(); o.value=value; this.appendChild(o); } }
  get value() { return this.selected; }
  set value(value: string) { this.selected=this.options.some(o=>o.value===value)?value:''; }
  appendChild(o: Option) { o.parent=this; this.options.push(o); return o; }
}
const noop=()=>{};
function control() { return {value:'',checked:false,disabled:false,textContent:'',classList:{contains:()=>false,add:noop,remove:noop,toggle:noop}}; }
function controls(): any { return new Proxy({}, {get(o: any,k) { return o[k]??=(control()); }}); }
const context=vm.createContext({i18nText:(s:string)=>s,REGISTERED_FONTS:new Set(),console,document:{createElement:(name:string)=>{assert.equal(name,'option');return new Option();}}});
function loadSource(file: string) {
  const source=readFileSync(ui+file+'.ts','utf8').replace(/^import\s[\s\S]*?\sfrom\s+['"][^'"]+['"];\s*$/gm,'');
  return stripTypeScriptTypes(source,{mode:'transform'}).replace(/\bexport\s+/g,'');
}
if (existsSync(ui+'fill-selection.ts')) vm.runInContext(loadSource('fill-selection'),context);
function dialog(file: string, name: string, kind: string) {
  const C=vm.runInContext(`{${loadSource(file)};${name}}`,context);
  const target=Object.create(C.prototype);
  const instance=new Proxy(target,{get(o,k) { if(k in o)return o[k];return o[k]=control(); }});
  const p={fontSize:1000,shadowOffsetX:10,shadowOffsetY:10,fillType:kind,fillColor:'#ffffff',patternColor:'#000000',patternType:0,borderFillId:42};
  Object.assign(target,{props:structuredClone(p),initialProps:structuredClone(p),attrBtns:{},alignBtns:{justify:control()},currentLang:0,shadowRadios:[control()],firstLineRadios:[control(),control(),control()],headTypeRadios:[Object.assign(control(),{value:'None'})],borderStates:{},bdSideToggles:{},tabState:{currentTabStops:[],deletedTabStops:[]},tabResult:controls(),borderResult:controls(),saveLangFields:noop,updateLangFields:noop,updateBorderPreview:noop,updatePreview:noop,hide:noop});
  target.tabResult.renderTabList=noop; target.tabResult.renderDeletedTabList=noop;
  target.borderResult.bdSpacingInputs=Array.from({length:4},control); target.borderResult.updateBdPreview=noop;
  target.faceColorSelect=new Select(); target.borderResult.bgFillSelect=new Select();
  const calls: any[]=[];target.onApply=(mods:unknown)=>calls.push(structuredClone(mods));
  C.prototype.populateFromProps.call(instance);
  return {instance,C,calls,select:file==='char-shape-dialog'?target.faceColorSelect:target.borderResult.bgFillSelect};
}
for (const [file,name] of [['char-shape-dialog','CharShapeDialog'],['para-shape-dialog','ParaShapeDialog']]) {
  for (const kind of ['gradient','image']) {
    test(`${name}: ${kind} 무변경 확인은 원래 채우기를 보존한다`,()=>{
      const {instance,C,calls,select}=dialog(file,name,kind);
      assert.equal(select.value,kind);
      C.prototype.handleOk.call(instance);assert.deepEqual(calls,[]);
    });
    test(`${name}: ${kind}에서 다른 서식 변경은 채우기를 쓰지 않는다`,()=>{
      const {instance,C,calls}=dialog(file,name,kind);
      if(file==='char-shape-dialog')instance.baseSizeInput.value='11';else instance.marginLeftInput.value='2';
      C.prototype.handleOk.call(instance);assert.equal(calls.length,1);
      assert.equal(calls[0].fillType,undefined);assert.equal(calls[0].borderFillId,undefined);
      assert.ok(file==='char-shape-dialog'?calls[0].fontSize===1100:calls[0].marginLeft===400);
    });
    test(`${name}: ${kind}의 명시적 삭제는 원래 정의를 기준으로 한다`,()=>{
      const {instance,C,calls,select}=dialog(file,name,kind);select.value='none';
      C.prototype.handleOk.call(instance);assert.equal(calls[0].fillType,'none');assert.equal(calls[0].borderFillId,42);
    });
    test(`${name}: ${kind}를 단색으로 바꾸어도 원래 테두리 정의를 전달한다`,()=>{
      const {instance,C,calls,select}=dialog(file,name,kind);select.value='solid';
      if(file==='char-shape-dialog')instance.faceColorPicker.value='#123456';else instance.borderResult.bgFillPicker.value='#123456';
      C.prototype.handleOk.call(instance);assert.equal(calls[0].fillType,'solid');assert.equal(calls[0].fillColor,'#123456');assert.equal(calls[0].borderFillId,42);
    });
    test(`${name}: ${kind} 유지 중 단색 컨트롤의 잔류값을 보내지 않는다`,()=>{
      const {instance,C,calls}=dialog(file,name,kind);
      if(file==='char-shape-dialog')instance.faceColorPicker.value='#123456';else instance.borderResult.bgFillPicker.value='#123456';
      C.prototype.handleOk.call(instance);assert.deepEqual(calls,[]);
    });
  }
  for(const kind of ['none','solid'])test(`${name}: ${kind} 무변경 대조군`,()=>{
    const {instance,C,calls,select}=dialog(file,name,kind);assert.equal(select.value,kind);C.prototype.handleOk.call(instance);assert.deepEqual(calls,[]);
  });
  test(`${name}: 단색 무늬만 바꾸어도 기존 면색과 무늬 종류를 보존한다`,()=>{
    const {instance,C,calls}=dialog(file,name,'solid');
    Object.assign(instance.props,{fillColor:'#ffdd00',patternColor:'#aa0000',patternType:2});
    instance.initialProps=structuredClone(instance.props);C.prototype.populateFromProps.call(instance);
    if(file==='char-shape-dialog')instance.patColorInput.value='#0011ff';else instance.borderResult.bgPatColorInput.value='#0011ff';
    C.prototype.handleOk.call(instance);
    assert.equal(calls[0].fillType,'solid');assert.equal(calls[0].fillColor,'#ffdd00');
    assert.equal(calls[0].patternType,2);assert.equal(calls[0].patternColor,'#0011ff');assert.equal(calls[0].borderFillId,42);
  });
  test(`${name}: 테두리만 바꾸면 미지원 배경 정의를 상속한다`,()=>{
    const {instance,C,calls}=dialog(file,name,'gradient');
    if(file==='char-shape-dialog')instance.borderTypeSelect.value='1';else instance.borderStates.left.type=1;
    C.prototype.handleOk.call(instance);assert.equal(calls[0].fillType,undefined);assert.equal(calls[0].borderFillId,42);
  });
  test(`${name}: 대화상자 재사용 시 이전 유지 선택지를 제거한다`,()=>{
    const {instance,C,select}=dialog(file,name,'gradient');instance.props.fillType='image';instance.initialProps.fillType='image';C.prototype.populateFromProps.call(instance);
    assert.equal(select.value,'image');assert.equal(select.options.filter((o:Option)=>o.dataset.preservedFill).length,1);
    instance.props.fillType='none';instance.initialProps.fillType='none';C.prototype.populateFromProps.call(instance);assert.equal(select.options.filter((o:Option)=>o.dataset.preservedFill).length,0);
  });
}
