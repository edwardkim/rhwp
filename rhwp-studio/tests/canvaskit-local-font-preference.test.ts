// 검사 유형: 관계형 — 실제 설치 글꼴이나 페이지 픽셀에 의존하지 않는 face 소유 계약.
// 주석 갱신: 2026-11-15T06:46:00+09:00.
import assert from 'node:assert/strict';
import test from 'node:test';
import { closestLocalFontStyle } from '../src/view/canvaskit/local-font-style.ts';

function face(id: string, weightClass = 400, italic = false) {
  return { id, record: { family: 'Contract Family', fullName: id, postscriptName: id,
    style: 'Regular', displayName: id, aliases: ['Contract Family'], weightClass, italic } };
}

test('동일 스타일의 다른 윤곽이 먼저 준비돼도 공통 resolver의 face를 유지한다', () => {
  const original = face('Preferred-TrueType');
  const alternate = face('Alternate-OpenType');
  for (const candidates of [[alternate, original], [original, alternate]]) {
    assert.equal(closestLocalFontStyle(original, candidates, false, false), original);
    assert.equal(closestLocalFontStyle(original, candidates, true, false), original);
  }
});

test('더 정확한 굵기·기울기 face는 기존처럼 선택하며 다른 family는 호출자가 제외한다', () => {
  const regular = face('Regular');
  const bold = face('Bold', 700);
  const italic = face('Italic', 400, true);
  assert.equal(closestLocalFontStyle(regular, [bold, italic], true, false), bold);
  assert.equal(closestLocalFontStyle(regular, [bold, italic], false, true), italic);
  assert.equal(closestLocalFontStyle(null, [], false, false), null);
});
