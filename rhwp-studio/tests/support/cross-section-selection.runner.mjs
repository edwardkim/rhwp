import { registerHooks } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { dirname, resolve } from 'node:path';
import assert from 'node:assert/strict';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const src = resolve(root, 'src');
const toast = 'data:text/javascript,export function showToast() {}';
const wasmStub = 'data:text/javascript,export default async function init() {} export class HwpDocument {} export function version() { return "0"; }';
registerHooks({ resolve(specifier, context, next) {
  if (specifier === '@/ui/toast') return { url: toast, shortCircuit: true };
  if (specifier === '@wasm/rhwp.js') return { url: wasmStub, shortCircuit: true };
  if (specifier.startsWith('@/')) return { url: pathToFileURL(resolve(src, specifier.slice(2) + '.ts')).href, shortCircuit: true };
  if (specifier.startsWith('.') && !/\.[cm]?[jt]s$/.test(specifier)) return { url: new URL(specifier + '.ts', context.parentURL).href, shortCircuit: true };
  return next(specifier, context);
} });
const { DeleteSelectionCommand } = await import(pathToFileURL(resolve(src, 'engine/command.ts')));
const { InputHandler } = await import(pathToFileURL(resolve(src, 'engine/input-handler.ts')));
const { onKeyDown, onCut, onPaste } = await import(pathToFileURL(resolve(src, 'engine/input-handler-keyboard.ts')));
const { onInput } = await import(pathToFileURL(resolve(src, 'engine/input-handler-text.ts')));
const start = { sectionIndex: 0, paragraphIndex: 0, charOffset: 1 };
const end = { sectionIndex: 1, paragraphIndex: 0, charOffset: 3 };
assert.throws(() => new DeleteSelectionCommand(start, end), /구역/);
assert.doesNotThrow(() => new DeleteSelectionCommand(start, { ...end, sectionIndex: 0 }));
const forbidden = () => { throw new Error('unsupported selection mutated'); };
const host = {
  active: true, editMode: 'edit', isFormMode: () => false,
  cursor: { getSelectionOrdered: () => ({ start, end }), hasSelection: () => true,
    isInHeaderFooter: () => false, isInFootnote: () => false, getPosition: () => end,
    clearSelection: forbidden },
  textarea: { value: 'X' }, executeOperation: forbidden, deleteSelection: forbidden,
};
host.canDeleteSelectionInFormMode = () => InputHandler.prototype.canDeleteSelectionInFormMode.call(host);
assert.equal(host.canDeleteSelectionInFormMode(), false);
for (const key of ['Delete', 'Backspace', 'Enter', 'Tab']) {
  let prevented = false;
  onKeyDown.call(host, { key, preventDefault: () => { prevented = true; } });
  assert.equal(prevented, true, key);
}
for (const operation of [onCut, onPaste]) {
  let prevented = false;
  operation.call(host, { preventDefault: () => { prevented = true; } });
  assert.equal(prevented, true, operation.name);
}
onInput.call(host);
assert.equal(host.textarea.value, '');
assert.equal(host.cursor.hasSelection(), true);
host.cursor.getSelectionOrdered = () => ({ start, end: { ...end, sectionIndex: 0 } });
assert.equal(host.canDeleteSelectionInFormMode(), true, 'same-section edit stays supported');
console.log('CROSS_SECTION_SELECTION_OK');
