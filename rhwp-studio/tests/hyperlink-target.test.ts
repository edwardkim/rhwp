import test from 'node:test';
import assert from 'node:assert/strict';
import { hyperlinkTarget, hyperlinkRange, selectedHyperlink } from '../src/core/hyperlink.ts';

const pos = { sectionIndex: 0, paragraphIndex: 3, charOffset: 2 };
test('하이퍼링크 주소는 중첩 셀 전체 경로를 보존한다', () => {
  assert.deepEqual(hyperlinkTarget({ ...pos, parentParaIndex: 8, cellPath: [
    { controlIndex: 1, cellIndex: 2, cellParaIndex: 3 },
    { controlIndex: 4, cellIndex: 5, cellParaIndex: 6 },
  ] }), { section: 0, para: 8, cellPath: [[1, 2, 3], [4, 5, 6]] });
});
test('캡션·불완전 셀 경로·다른 문단 선택을 본문에 적용하지 않는다', () => {
  assert.throws(() => hyperlinkTarget({ ...pos, parentParaIndex: 1 }));
  assert.throws(() => hyperlinkTarget({ ...pos, parentParaIndex: 1, controlIndex: 0, cellIndex: 65534, cellParaIndex: 0 }));
  assert.throws(() => hyperlinkRange(pos, { start: pos, end: { ...pos, paragraphIndex: 4 } }));
  assert.throws(() => hyperlinkTarget({ ...pos, charOffset: -1 }));
});
test('본문/글상자 주소와 역방향 문자 범위가 안정적이다', () => {
  assert.deepEqual(hyperlinkTarget(pos), { section: 0, para: 3, cellPath: [] });
  assert.deepEqual(hyperlinkTarget({ ...pos, parentParaIndex: 8, controlIndex: 0, cellIndex: 0, cellParaIndex: 1, isTextBox: true }),
    { section: 0, para: 8, cellPath: [[0, 0, 1]] });
  const result = hyperlinkRange(pos, { start: { ...pos, charOffset: 5 }, end: pos });
  assert.equal(result.start, 2);
  assert.equal(result.end, 5);
});
test('인접 링크 경계·여러 링크 선택·링크 밖 범위를 구분한다', () => {
  const links = [
    { fieldId: 1, start: 0, end: 3, text: '첫😀', uri: 'https://a.test' },
    { fieldId: 2, start: 3, end: 5, text: '둘째', uri: 'https://b.test' },
  ];
  assert.equal(selectedHyperlink(links, 3, 3)?.fieldId, 2);
  assert.equal(selectedHyperlink(links, 5, 5), undefined);
  assert.equal(selectedHyperlink(links, 1, 2)?.fieldId, 1);
  assert.throws(() => selectedHyperlink(links, 1, 4));
  assert.throws(() => selectedHyperlink(links, 4, 6));
});
