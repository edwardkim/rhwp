// 관계형: 인쇄/화면 메트릭의 복원과 묶음 선택의 실제 소유 주소를 검사한다.
// 실물 페이지의 고정 px를 정답으로 복사하지 않는 합성 소비 경로 검사다.
// 주석 갱신: 2026-11-04 14:55 KST.
import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';

for (const mode of ['print', 'group']) {
  test(`${mode}: 실제 제품 소비 경로의 상태·주소 계약`, () => {
    const result = spawnSync(process.execPath, ['--experimental-transform-types', fileURLToPath(new URL('./support/print-group-context.runner.mjs', import.meta.url)), mode], {encoding: 'utf8'});
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.match(result.stdout, /PASS/);
  });
}
