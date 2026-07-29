import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #bookmark] 책갈피 추가/삭제/이름변경 히스토리 라우팅 소스 가드.
//
// 책갈피 관리(추가/삭제/이름변경)가 wasm 뮤테이터를 직접 호출하면 미기록되어 undo 불가.
// this.services.getInputHandler() 로 라우터에 도달하므로 executeOperation snapshot 으로
// 라우팅했는지 정적으로 핀한다. 행위 증명은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/ui/bookmark-dialog.ts'), 'utf8');

const OPS = ['addBookmark', 'deleteBookmark', 'renameBookmark'];

test('책갈피 관리 op 는 wasm 직접 호출이 아니라 snapshot 으로 라우팅한다', () => {
  for (const op of OPS) {
    // 미라우팅 흔적: this.services.wasm.<op>( 직접 호출 금지.
    assert.doesNotMatch(src, new RegExp(`this\\.services\\.wasm\\.${op}\\s*\\(`),
      `${op} 는 executeOperation 경유여야 함(this.services.wasm 직접 호출 금지)`);
    // 라우팅 마커 + operation 콜백 안의 실제 뮤테이션.
    assert.match(src, new RegExp(`operationType:\\s*'${op}'`), `${op} snapshot 라우팅`);
    assert.match(src, new RegExp(`\\bwasm\\.${op}\\s*\\(`), `${op} 뮤테이션은 operation 콜백에 존재`);
  }
});
