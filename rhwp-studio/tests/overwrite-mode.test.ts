import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// [#7489] Insert 키로 수정 모드를 켜면 입력이 캐럿 뒤 글자를 덮어써야 한다(한컴과 같다).
// 종전에는 상태 표시줄 문구만 바뀌고 입력은 항상 끼워 넣었다. 실제 입력 핸들러와
// InsertTextCommand·CommandHistory 를 러너에서 실행해 문서 결과와 되돌리기, 링크·누름틀
// 범위와 글자 모양 복원, 양식 모드 보호, 이모지 뒤 scalar 오프셋, 조합 중 문서 교체의
// 문단 조각 해제를 검증한다.
//
// 지원 여부를 allowedNodeEnvironmentFlags 로 미리 짐작하면 그 판단이 틀린 Node 에서 검사가
// 조용히 빠진다. review-runtime-contracts·mixed-char-format 러너처럼 바로 띄우고, 못 띄우면
// 실패로 드러낸다.

const runner = fileURLToPath(new URL('./support/overwrite-mode.runner.mjs', import.meta.url));

test('수정 모드 입력은 캐럿 뒤 일반 글자를 덮어쓰고 한 번에 되돌린다 (자식 프로세스)', () => {
  const res = spawnSync(
    process.execPath,
    ['--experimental-transform-types', '--no-warnings', runner],
    { encoding: 'utf8' },
  );
  assert.ifError(res.error);
  assert.equal(res.status, 0,
    `러너가 비정상 종료했습니다.\n--- stdout ---\n${res.stdout}\n--- stderr ---\n${res.stderr}`);
  assert.match(res.stdout, /OVERWRITE_MODE_OK/, '행위 검증 성공 마커가 있어야 함');
});
