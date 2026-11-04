import test from 'node:test';
import assert from 'node:assert/strict';
import { evaluateFormAction, type FormActionContext } from '../../rhwp-shared/forms/action-runtime.ts';

const context = (body: string): FormActionContext => ({ schemaVersion: 1,
  button: { section: 0, para: 0, control: 0, cellPath: [] }, trigger: 'OnSubmit_Click',
  header: 'var Documents=XHwpDocuments; var Document=Documents.Active_XHwpDocument; var Result=Document.XHwpFormEditControls.ItemFromName("Result");',
  source: `function OnSubmit_Click(){${body}}`, forms: [
    { address: { section: 0, para: 0, control: 0, cellPath: [] }, name: 'Submit', formType: 'PushButton', text: '', caption: '계산', value: 0, enabled: true, items: [] },
    { address: { section: 0, para: 1, control: 0, cellPath: [[0, 0, 0]] }, name: 'Result', formType: 'Edit', text: 'before', caption: '', value: 0, enabled: true, items: [] },
  ] });

test('form scripts calculate real values and emit addressed updates without executing host code', () => {
  const input = context('var sum=0; for(var i=1;i<=4;i++){sum+=i;} if(sum===10){Result.Text="합계 "+sum;this.Caption="완료";}');
  const plan = evaluateFormAction(input);
  assert.equal(plan.updates.find(value => value.address.para === 1)?.text, '합계 10');
  assert.equal(plan.updates.find(value => value.address.para === 0)?.caption, '완료');
  assert.equal(input.forms[1].text, 'before'); assert.equal(plan.commands.length, 0);
});

test('script host edit commands preserve their order and explicit parameters', () => {
  const plan = evaluateFormAction(context('HAction.Run("MoveDocEnd"); HAction.GetDefault("InsertText",HParameterSet.HInsertText.HSet); HParameterSet.HInsertText.Text="완료"; HAction.Execute("InsertText",HParameterSet.HInsertText.HSet);'));
  assert.deepEqual(plan.commands, [{ name: 'MoveDocEnd' }, { name: 'InsertText', text: '완료' }]);
});

test('unbounded loops, prototype access, host I/O and arbitrary JavaScript fail before any update', () => {
  for (const body of ['while(true){}', 'Result.Text="bad"; while(true){}', 'window.location="https://example.com";',
    'Result.constructor("return window")();', 'eval("1")', 'HAction.Run("ShellExecute");', 'var x="x";while(true){x+=x;}']) {
    const input = context(body); assert.throws(() => evaluateFormAction(input), /FORM_ACTION_SCRIPT/); assert.equal(input.forms[1].text, 'before');
  }
});

test('function locals are isolated and global values follow lexical scope', () => {
  const input = context('increment(); Result.Text=String(total);');
  input.source = 'var total=1;function increment(){total+=2;}'+input.source;
  assert.equal(evaluateFormAction(input).updates[0].text, '3');
});
