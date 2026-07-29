import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #number-dialogs] 새 번호·미주 모양 다이얼로그 히스토리 라우팅 소스 가드.
//
// new-number/endnote-shape 다이얼로그는 (wasm,eventBus) 로만 생성돼 편집 라우터에 도달 못 했다
// → 새 번호 삽입/미주 모양 변경이 미기록(undo 불가). services 주입 + executeOperation snapshot
// 라우팅 + services 미주입 fallback 을 정적으로 핀한다. 행위 증명은 브라우저 왕복.
//
// (numbering-dialog 은 별건 — 문단 적용 ih.applyNumbering 이 이미 라우팅됨; DEF 생성 orphan 은
//  보이지 않는 잔여라 이 PR 범위 밖.)

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, `src/ui/${rel}`), 'utf8');

const DIALOGS: Array<{ file: string; op: string }> = [
  { file: 'new-number-dialog.ts', op: 'insertNewNumber' },
  { file: 'endnote-shape-dialog.ts', op: 'endnoteShape' },
];

for (const { file, op } of DIALOGS) {
  test(`${file} 는 services 주입 + snapshot 라우팅 + fallback 을 갖춘다`, () => {
    const s = src(file);
    assert.match(s, /services\?:\s*CommandServices/, `${file}: 생성자에 services 주입`);
    assert.match(s, /import type \{ CommandServices \}/, `${file}: CommandServices import`);
    assert.match(s, /this\.services\?\.getInputHandler\(\)/, `${file}: getInputHandler 로 라우터 도달`);
    assert.match(s, new RegExp(`operationType:\\s*'${op}'`), `${file}: ${op} snapshot 라우팅`);
    assert.match(s, /this\.eventBus\.emit\('document-changed'\)/, `${file}: fallback emit 유지`);
  });
}
