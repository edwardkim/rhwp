import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2374 후속] 셀 내부 양식 개체 쓰기 경로의 locator 일치 가드.
//
// 셀 안 양식 개체는 hit 결과의 para 가 "표를 담은 최상위 문단", ci 가 "셀 문단 안의 컨트롤
// 인덱스"다(form_query.rs get_form_object_at_native). 따라서 flat setFormValue(sec, para, ci)
// 는 표 컨트롤 슬롯을 가리켜 set_form_value_native 의 `not a form object` 로 조용히 실패한다.
//
//   양식 모드에서 표 셀 안 Edit 필드 클릭 → 입력 → Enter
//   기대: 값이 저장되고 Ctrl+Z 로 되돌아감
//   실제: 값이 저장되지 않고(반환값 미확인), inCell 없는 기록만 남아 유령 undo 엔트리 발생
//
// CheckBox 분기는 setFormValueInCell + record 의 inCell 을 모두 갖추고 있었고 Edit 커밋만
// 누락돼 있었다. 두 경로가 같은 locator 를 쓰는지 정적으로 핀한다.
// 행위 증명(셀 안 Edit 왕복)은 브라우저 왕복(PR 검증).

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/engine/input-handler.ts'), 'utf8');

/** 메서드 시작부터 다음 최상위 메서드 전까지 본문을 자른다. */
function methodBlock(name: string): string {
  const start = src.indexOf(`  private ${name}(`) >= 0
    ? src.indexOf(`  private ${name}(`)
    : src.indexOf(`  ${name}(`);
  assert.notEqual(start, -1, `${name} 메서드 not found`);
  const rest = src.slice(start + 1);
  const end = rest.search(/\n {2}(private |public )?[a-zA-Z_]\w*\s*\(/);
  return end === -1 ? rest : rest.slice(0, end);
}

test('셀 내부 locator 계산이 단일 헬퍼로 공유된다', () => {
  assert.match(src, /private formInCellLoc\s*\(/,
    'locator 조건이 여러 곳에 복제되면 한쪽만 고쳐지는 회귀가 재발한다');

  // 인라인 복제본이 남아 있으면 헬퍼가 무력화된다.
  const inlineCopies = src.match(/formHit\.inCell && formHit\.tablePara !== undefined/g) ?? [];
  assert.equal(inlineCopies.length, 1,
    `locator 조건은 헬퍼 안에만 있어야 함(현재 ${inlineCopies.length}곳)`);
});

test('Edit 오버레이 커밋이 셀 내부 경로로 분기한다', () => {
  const block = methodBlock('showEditOverlay');

  assert.match(block, /formInCellLoc\s*\(/, 'Edit 커밋도 셀 내부 locator 를 계산해야 함');
  assert.match(block, /setFormValueInCell\s*\(/,
    '셀 안에서는 setFormValueInCell 로 써야 표 컨트롤 슬롯을 피한다');
  assert.match(block, /inCell: inCellLoc/,
    '기록에 inCell 을 실어야 undo 가 같은 슬롯을 되돌린다');
});

test('CheckBox 분기의 셀 내부 처리가 유지된다', () => {
  // 선례가 사라지면 Edit 쪽 수정의 근거도 사라진다.
  const block = methodBlock('handleFormObjectClick');
  assert.match(block, /setFormValueInCell\s*\(/, 'CheckBox 셀 내부 쓰기 유지');
  assert.match(block, /inCell: inCellLoc/, 'CheckBox 기록의 inCell 유지');
});
