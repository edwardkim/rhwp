import test from 'node:test';
import assert from 'node:assert/strict';
import { rustMutatingExports } from './helpers/rust-mutating-exports.ts';

test('exported impl inventories every public mutable method without camelCase guessing', () => {
  assert.deepEqual(rustMutatingExports(`
    #[wasm_bindgen] impl Doc {
      pub fn save_snapshot(&mut self) {}
      pub async fn other_action(&mut self) {}
      pub unsafe fn unsafe_action(&mut self) {}
      #[wasm_bindgen(js_name = namedAction)]
      ${'/// Long documentation.\n'.repeat(100)}
      pub fn named_action(&mut self) {}
      #[wasm_bindgen(js_name = "stringName")]
      pub fn string_name(&'a mut self) {}
    }
  `), ['save_snapshot', 'other_action', 'unsafe_action', 'namedAction', 'stringName']);
});

test('comments, strings, private methods and non-exported impls cannot create exports', () => {
  assert.deepEqual(rustMutatingExports(`
    /* nested /* #[wasm_bindgen] impl Fake {} */ comment */
    const TEXT: &str = r###"#[wasm_bindgen] impl Fake { pub fn fake(&mut self) {} }"###;
    impl Internal { pub fn internal(&mut self) {} }
    #[wasm_bindgen] impl Doc {
      fn private(&mut self) {}
      pub fn query(&self) {}
      pub fn factory() {}
      #[wasm_bindgen(skip)] pub fn skipped(&mut self) {}
      pub fn actual(&mut self) {
        let ch = '}'; let text = "}";
        fn nested() {}
      }
    }
  `), ['actual']);
});

test('module and cfg attributes preserve the enclosing exported impl context', () => {
  assert.deepEqual(rustMutatingExports(`
    mod api {
      #[wasm_bindgen] #[cfg(feature = "web")]
      impl Doc {
        #[cfg(feature = "web")] pub fn r#reset(&mut self) {}
        #[wasm_bindgen(js_name = "second")] pub fn second_action(&mut self) {}
      }
    }
  `), ['reset', 'second']);
});

test('empty, malformed and unsupported mutable inventories fail closed', () => {
  for (const src of [
    'impl Internal { pub fn internal(&mut self) {} }',
    '#[wasm_bindgen] impl Doc {',
    '#[wasm_bindgen] impl Doc { pub fn typed(self: &mut Self) {} }',
    '#[wasm_bindgen] impl Doc { #[wasm_bindgen(js_name)] pub fn bad(&mut self) {} }',
    '/* unterminated',
    '#[cfg_attr(feature = "web", wasm_bindgen)] impl Doc { pub fn reset(&mut self) {} }',
    '#[wasm_bindgen] impl Doc { pub(crate) fn reset(&mut self) {} }',
  ]) assert.throws(() => rustMutatingExports(src));
});
