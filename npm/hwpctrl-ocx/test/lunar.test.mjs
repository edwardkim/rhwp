/**
 * 음력 표가 조용히 틀어지는 것을 막는다.
 *
 * 표는 해마다 열일곱 비트로 접혀 있어 **한 달의 길이만 틀어져도 그 뒤가 통째로 하루씩
 * 밀린다**. 그래서 양 끝을 못박고 사이에 닻을 여럿 박는다 — 실제로 접기 자릿수를 잘못
 * 잡았을 때(윤달 번호가 13비트 위로 올라가 열일곱 비트가 되는 것을 놓쳤다) 1900년부터
 * 전부 밀렸고, 이 방식의 검사가 그것을 잡았다.
 */

import assert from 'node:assert/strict';
import test from 'node:test';

import {
  LUNAR_FIRST_YEAR,
  LUNAR_LAST_YEAR,
  lunarToSolar,
  solarToLunar,
} from '../src/lunar.mjs';

/** 한국천문연구원 자료에서 뽑은 음력 달 머리들 — `[양력, 해, 달, 날, 윤달]`. */
const ANCHORS = [
  ['1841-01-23', 1841, 1, 1, false], // 표의 첫날
  ['1841-04-21', 1841, 3, 1, true],
  ['1848-12-26', 1848, 12, 1, false],
  ['1850-06-10', 1850, 5, 1, false],
  ['1870-11-23', 1870, 10, 1, true],
  ['1880-10-04', 1880, 9, 1, false],
  ['1883-02-08', 1883, 1, 1, false],
  ['1888-03-13', 1888, 2, 1, false],
  ['1893-04-16', 1893, 3, 1, false],
  ['1900-09-24', 1900, 8, 1, true],
  ['1902-07-05', 1902, 6, 1, false],
  ['1930-07-26', 1930, 6, 1, true],
  ['1932-09-30', 1932, 9, 1, false],
  ['1948-02-10', 1948, 1, 1, false],
  ['1953-05-13', 1953, 4, 1, false],
  ['1960-07-24', 1960, 6, 1, true],
  ['1967-08-06', 1967, 7, 1, false],
  ['1971-05-24', 1971, 5, 1, false],
  ['1979-10-21', 1979, 9, 1, false],
  ['1990-06-23', 1990, 5, 1, true],
  ['1997-09-02', 1997, 8, 1, false],
  ['2000-02-05', 2000, 1, 1, false],
  ['2009-12-16', 2009, 11, 1, false],
  ['2020-05-23', 2020, 4, 1, true],
  ['2032-10-04', 2032, 9, 1, false],
  ['2043-12-31', 2043, 12, 1, false], // 표의 마지막 날
];

test('음력 표의 닻이 제자리에 있다', () => {
  for (const [iso, year, month, day, leap] of ANCHORS) {
    const [y, m, d] = iso.split('-').map(Number);
    assert.deepEqual(solarToLunar(y, m, d), { year, month, day, leap }, iso);
    assert.deepEqual(lunarToSolar(year, month, day, leap), { year: y, month: m, day: d }, iso);
  }
});

test('표 전체에서 음→양→음 왕복이 제자리로 돌아온다', () => {
  let checked = 0;
  for (let year = LUNAR_FIRST_YEAR; year <= LUNAR_LAST_YEAR; year += 1) {
    for (let month = 1; month <= 12; month += 1) {
      for (const leap of [false, true]) {
        for (const day of [1, 15, 29]) {
          const solar = lunarToSolar(year, month, day, leap);
          if (!solar) continue;
          // 양→음은 양력 2043-12-31 에서 끊긴다. 음력 2043년 섣달은 그 뒤로 넘어가므로
          // 되돌아올 자리가 없다 — 그 비대칭은 아래 경계 검사에서 따로 본다.
          if (solar.year > LUNAR_LAST_YEAR) continue;
          const back = solarToLunar(solar.year, solar.month, solar.day);
          assert.ok(back, `${year}/${month}/${day}`);
          // 그 해에 윤달이 없으면 `leap` 은 무시되므로 되돌아온 값이 평달일 수 있다.
          assert.equal(back.year, year);
          assert.equal(back.month, month);
          assert.equal(back.day, day);
          checked += 1;
        }
      }
    }
  }
  assert.ok(checked > 12000, `왕복을 ${checked}건밖에 못 봤다`);
});

test('표 밖에서는 답하지 않는다 — 두 방향의 끝이 다르다', () => {
  assert.equal(solarToLunar(1841, 1, 22), null);
  assert.equal(solarToLunar(2044, 1, 1), null);
  assert.ok(solarToLunar(2043, 12, 31));
  // 음→양은 음력 해로 자른다. 2043년 섣달은 양력 2044년으로 넘어가지만 답한다.
  assert.deepEqual(lunarToSolar(2043, 12, 15, false), { year: 2044, month: 1, day: 14 });
  assert.equal(lunarToSolar(1840, 12, 15, false), null);
  assert.equal(lunarToSolar(2044, 1, 1, false), null);
});

test('그 해에 윤달이 없으면 윤달 표시를 무시한다', () => {
  // 2026년에는 윤6월이 없다. 한글도 평6월과 같은 날을 준다.
  assert.deepEqual(lunarToSolar(2026, 6, 26, true), lunarToSolar(2026, 6, 26, false));
  // 있는 해에서는 갈린다.
  assert.notDeepEqual(lunarToSolar(2020, 4, 1, true), lunarToSolar(2020, 4, 1, false));
});

test('없는 날은 답하지 않는다', () => {
  const june = lunarToSolar(2026, 6, 30, false);
  assert.ok(june, '2026년 6월은 서른 날이다');
  assert.equal(lunarToSolar(2026, 6, 31, false), null);
  assert.equal(lunarToSolar(2026, 6, 0, false), null);
});
