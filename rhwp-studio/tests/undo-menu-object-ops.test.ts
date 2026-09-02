import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { balancedFrom, callsOf, functionBodyFrom, matchesOutside } from './support/source-guard.ts';

// [Task #2343] 메뉴/도구상자 개체 조작 히스토리 라우팅 소스 가드.
//
// 개체 삭제·정렬(z순서)·묶기/풀기·회전/대칭을 메뉴로 실행해도 undo 되도록, 해당 뮤테이션이
// executeOperation({kind:'snapshot'}) 로 라우팅되는지 정적으로 핀한다. 뮤테이션 표면 원장
// (mutation-routing-guard)은 '표면 증가'만 잡고 '라우팅 누락'은 못 잡으므로(라우팅해도
// wasm.X( 텍스트는 그대로 남음) 이 가드로 재발을 차단한다. 행위 증명은 브라우저 왕복(PR 검증).
//
// [Task #2370 클러스터 D] 견고성 보강 — 종전 가드는 뮤테이터가 파일 어딘가에 `wasm.X(`
// 로 존재하기만 하면 통과해서, `const wasm = services.wasm` 별칭으로 라우팅 밖에서
// 호출해도 green 이었다. 이제 **호출 위치가 recordObjectMutation 인자 안인지**를 본다.
// 회전/대칭 검사도 고정 창(slice(0, 700)) 대신 중괄호 매칭으로 함수 본문을 잡는다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const insertSrc = readFileSync(join(rootDir, 'src/command/commands/insert.ts'), 'utf8');

// recordObjectMutation 경유로 라우팅돼야 하는 개체 뮤테이터(속성 setter 는 setProps 공유·별건).
// [#5769 후속] changeShapeZOrder 는 이 목록에서 뺐다 — recordObjectMutation(스냅샷) 대신
// SetZOrderCommand(kind:'command') 역연산으로 기록되며, 아래 전용 가드가 배선을 핀한다.
const OBJECT_MUTATORS = [
  'deleteShapeControl',
  'deleteEquationControl',
  'deleteCellPictureControlByPath',
  'deletePictureControl',
  'groupShapes',
  'ungroupShape',
];

test('recordObjectMutation 은 executeOperation snapshot 으로 위임한다', () => {
  const block = functionBodyFrom(insertSrc, 'function recordObjectMutation');
  assert.match(block, /ih\.executeOperation\(/, 'executeOperation 에 위임');
  assert.match(block, /kind:\s*'snapshot'/, 'snapshot 커맨드로 기록(undo/redo 보장)');
});

test('개체 조작 뮤테이터는 recordObjectMutation 인자 안에서만 호출된다', () => {
  const routed = callsOf(insertSrc, 'recordObjectMutation');
  assert.ok(routed.length > 0, 'recordObjectMutation 호출부가 있어야 함');

  for (const m of OBJECT_MUTATORS) {
    // 미라우팅 흔적: services.wasm.<mutator>( 가 남아있으면 회귀.
    assert.doesNotMatch(
      insertSrc,
      new RegExp(`services\\.wasm\\.${m}\\s*\\(`),
      `${m} 는 recordObjectMutation 경유여야 함(services.wasm 직접 호출 금지 — 히스토리 우회)`,
    );
    // 라우팅된 호출: 모든 호출이 recordObjectMutation 인자 범위 안에 있어야 한다.
    const pattern = new RegExp(`\\bwasm\\s*\\.\\s*${m}\\s*\\(`, 'g');
    assert.ok(
      pattern.test(insertSrc),
      `${m} 뮤테이션 자체는 operation 콜백에 존재해야 함`,
    );
    assert.deepEqual(
      matchesOutside(insertSrc, new RegExp(pattern.source, 'g'), routed),
      [],
      `${m} 가 recordObjectMutation 밖에서 호출됨 — 히스토리를 우회한다`,
    );
  }
});

// [#5769 후속] z순서도 회전/대칭과 같은 축이다 — 스냅샷에서 역연산으로 옮겼다.
// 되돌릴 것은 대상+교환 이웃의 z 스칼라 1~2개뿐이므로 문서 클론 2개가 스택에
// 얹힐 이유가 없다. 배선 상세는 issue-5769-zorder-inverse.test.ts 가 담당하고,
// 여기선 "insert.ts 에 직접 호출이 없다" 는 우회 차단만 본다.
test('z순서는 역연산 커맨드로 기록한다 — insert.ts 에 직접 뮤테이션이 남지 않는다', () => {
  assert.doesNotMatch(
    insertSrc,
    /\bwasm\s*\.\s*changeShapeZOrder\s*\(/,
    'changeShapeZOrder 직접 호출은 히스토리를 우회한다 — SetZOrderCommand 로 기록',
  );
  const funnel = functionBodyFrom(insertSrc, 'function changeZOrder');
  assert.match(funnel, /new SetZOrderCommand\(/, '정렬 메뉴는 속성쌍 커맨드로 기록');
  assert.match(funnel, /kind:\s*'command'/, '양식 모드 게이트 통과를 위해 command 라우터 경유');
});

test('services.wasm 을 별칭으로 빼내 가드를 우회할 수 없다', () => {
  // `const wasm = services.wasm` 같은 별칭이 생기면 위 `services.wasm.X(` 금지가 무력화된다.
  // (operation 콜백의 파라미터 이름 `wasm` 은 별칭이 아니라 주입값이므로 무관하다.)
  const aliases = [...insertSrc.matchAll(/(?:const|let|var)\s+\w+\s*=\s*services\s*\.\s*wasm\b(?!\s*\.)/g)];
  assert.deepEqual(
    aliases.map((m) => m[0]),
    [],
    'services.wasm 별칭 금지 — 라우팅 가드를 우회하는 통로가 된다',
  );
});

// [Task #3230] 회전/대칭은 스냅샷에서 **역연산**으로 옮겼다. 가드의 뜻은 그대로다 —
// "히스토리를 우회하지 않는다". 우회 형태만 달라졌으므로(직접 setProps 호출) 그것을 본다.
test('회전/대칭은 역연산으로 기록한다', () => {
  const rot = balancedFrom(insertSrc, 'function applyRotationDelta', '{');
  assert.match(rot, /applyAbsolutePropChange\(/, '회전을 히스토리에 기록');
  assert.doesNotMatch(
    rot,
    /\bsetProps\s*\(|setObjectProps\s*\(/,
    '적용은 기록 헬퍼 안에서만 — 직접 적용하면 히스토리를 우회한다',
  );
  const flip = balancedFrom(insertSrc, 'function toggleFlip', '{');
  assert.match(flip, /applyAbsolutePropChange\(/, '대칭을 히스토리에 기록');
  assert.doesNotMatch(
    flip,
    /\bsetProps\s*\(|setObjectProps\s*\(/,
    '적용은 기록 헬퍼 안에서만 — 직접 적용하면 히스토리를 우회한다',
  );
});

// [Task #3230 후속] 역연산은 **왕복이 참인 대상에만** 쓴다. 그림·OLE 는
// `refresh_picture_rotation_layout_for_save` 가 각도와 무관하게 `rotate_image`·flip 0x80000 을
// 세우고 되돌리는 경로가 없어(object_ops/picture.rs:242-243) 속성 bag 으로 복원되지 않는다.
test('그림·OLE 는 스냅샷을 유지한다 — 속성 왕복이 참인 역연산이 아니다', () => {
  const gate = functionBodyFrom(insertSrc, 'function absolutePropChangeIsInvertible');
  assert.match(
    gate,
    /ref\.type === 'shape'/,
    "역연산 대상은 도형뿐 — 그림·OLE 로 넓히면 rotate_image 가 되돌아가지 않는다",
  );

  const apply = functionBodyFrom(insertSrc, 'function applyAbsolutePropChange');
  assert.match(apply, /absolutePropChangeIsInvertible\(ref\)/, '대상 판정을 거쳐야 한다');
  assert.match(apply, /executeAbsolutePropChange\(/, '참이면 역연산 커맨드');
  assert.match(apply, /recordObjectMutation\(/, '아니면 종전 스냅샷');
});

test('executeAbsolutePropChange 는 라우터에서 역연산 커맨드를 실행한다', () => {
  const block = functionBodyFrom(insertSrc, 'function executeAbsolutePropChange');
  assert.match(block, /kind:\s*'command'/, '속성 적용도 편집 허용 검사를 거치는 command 라우터에서 실행');
  assert.match(block, /new SetObjectPropsCommand\(/, 'before/after 를 든 커맨드로 기록');
  assert.match(
    block,
    /refresh:\s*'full'/,
    '개체 회전/대칭 뒤 전체 화면 갱신을 유지한다',
  );
  assert.doesNotMatch(block, /\bsetProps\s*\(/, 'setter를 라우터 전에 호출하면 양식 모드 편집 차단을 우회한다');
});
