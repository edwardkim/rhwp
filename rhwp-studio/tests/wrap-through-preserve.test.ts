import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

// 개체 속성 다이얼로그의 wrapValues 에는 core TextWrap 의 'Through'(빈 공간
// 채움)가 없어, Through 배치 개체의 속성창을 열면 배치 버튼이 전부 비활성이
// 된다. 이때 getSelectedWrap() 이 기본값 'Square' 를 반환하면 사용자가 아무
// 것도 바꾸지 않고 확인만 눌러도 textWrap diff 가 전송되어 배치가 조용히
// 변경·저장된다. 활성 버튼이 없으면 개체의 원래 배치 값을 보존해야 한다.

test('배치 버튼 미선택 시 getSelectedWrap은 원래 textWrap을 보존한다', () => {
  const dialog = source('src/ui/picture-props-dialog.ts');
  const start = dialog.indexOf('private getSelectedWrap');
  assert.notEqual(start, -1, 'getSelectedWrap not found');
  const block = dialog.slice(start, dialog.indexOf('\n  private ', start + 1));

  assert.match(
    block,
    /return this\.props\?\.textWrap \?\? 'Square';/,
    '활성 버튼이 없으면 props.textWrap 을 반환해 의도치 않은 배치 변경을 막아야 함',
  );
  assert.doesNotMatch(
    block,
    /idx >= 0 \? this\.wrapValues\[idx\] : 'Square'/,
    "무조건 'Square' 폴백은 Through 배치를 조용히 덮어쓴다",
  );
});
