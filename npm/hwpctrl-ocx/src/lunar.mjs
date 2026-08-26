/**
 * 음·양력 변환 — 규격 §8.3.57~§8.3.60.
 *
 * ## 표의 출처
 *
 * 한국천문연구원(KASI)의 음양력 정보를 공공데이터포털 Open API 로 받아 접었다. 이용허락범위
 * 제한 없음이라 MIT 인 이 저장소에 실을 수 있다. 해마다 열일곱 비트다 — 위 4비트가 윤달
 * 번호(없으면 0), 아래 13비트가 달마다 30일이면 1인 자리다(윤달이 있는 해는 열세 달).
 * 첫 해 설날의 양력 하나만 적어 두고 나머지 날짜는 달 길이를 더해 얻는다.
 *
 * 표가 맞는지는 두 갈래로 봤다. 긁어 온 달 머리 2,511개가 이 표에서 모두 그 자리에 오고,
 * 왕복(음→양→음)이 어긋나는 자리가 없다.
 *
 * ## 한글과 어긋나는 자리가 있다 — 일부러 그렇게 두었다
 *
 * 한글 2022 의 표는 이 공공자료와 **1841~2043 의 2,051일 표본 중 35일(1.71%)** 을 다르게
 * 답한다. 그중 스물둘은 한글이 29일짜리 달의 "30일"을 말하는 곳이라 이 달력에 아예 없는
 * 날이고, 넷은 2033년 윤달 배치가 갈린 자리다. 어긋난 목록은 `spec/lunar_divergence.json`
 * 에 양쪽 답을 함께 적어 두었다.
 *
 * 그래서 이 넷은 **오라클이 판정자가 될 수 없는 항목**(`substituted`)이다. 우리는 국가
 * 기관이 펴낸 달력을 따른다.
 *
 * ## 경계는 한글에 맞췄다
 *
 * 한글이 답하는 범위는 실측으로 양력 **1841-01-23 ~ 2043-12-31**, 음력 해 **1841~2043**
 * 이다(`tools/hwpctrl_compat/probes/pC-lunar-edge.json`). 표를 더 넓게 만들 수 있었지만
 * 범위 밖에서의 거동까지 같아야 호환이라 여기서 끊었다.
 */

export const LUNAR_FIRST_YEAR = 1841;
export const LUNAR_LAST_YEAR = 2043;

/** 음력 1841년 1월 1일에 해당하는 양력 날짜. */
const EPOCH_UTC = Date.UTC(1841, 0, 23);

/**
 * 양→음이 답하는 마지막 양력 날짜.
 *
 * **두 방향의 끝이 서로 다르다**(실측). 양→음은 양력 2043-12-31 에서 끊기는데, 음→양은
 * 음력 2043년이면 답한다 — 음력 2043/12/15 를 물으면 2044-01-14 를 준다. 음력 해로만
 * 자르면 2044-01-01 에도 답해 버려 한글과 갈린다.
 */
const LAST_SOLAR_UTC = Date.UTC(2043, 11, 31);

/** 해마다 열일곱 비트, 16진수 다섯 자씩. */
const PACKED =
  '06a5601a540fd2a01aaa00b540b55a012ba0095c094ab0149a11a4b01652016aa0ead5005b4012ba' +
  '0a95700936014960764b00d52115a900d6a0056c0b2b60126e0092e08c9601c9415d4a01b5200b5a' +
  '0c56d0055c0125c0b92d0192a01a9407b4a016d20eada00ab6004ba0b25b012560152a09a9501694' +
  '016aa04ad500ab60c4b7004ae00a560b52a01d2a00d54075aa0156a1096d0095c014ae0aa4d01a4c' +
  '01b2a08d5500ad40135a0495d0095c0d49b0149a01a4a0bb29016aa00ad4052da012ba0e95b00936' +
  '014960b64b00d52015a8096b50056c012b6049370092e0cc9601c9401d4a0ada900b6a0056c072ae' +
  '0125c0f92d0192a01a940db4a016d200ada0855b004ba0125a0592b0152a0fa9501694016aa0aad5' +
  '009b6004b60725700a561152b00d2a00d540d5aa0156a0096c094ae014ae00a4e06d2601b2a0ed55' +
  '00ad40135a0a95d0095c0149c09a4d01a4a11aa9016a801ad40d2da012b6009360949b014961564b' +
  '00d4a00da80d6b50056c012b60a9370092e00c9606d4a01d4a10da900b5a0056c0b26e0125c0192c' +
  '09c9501a9401b4a04b5500ad80f55b004ba0125a0b92b0152a016940774a016aa12ab500974014b6' +
  '0aa5700a560152a0969500d54015aa04ab50096c0d4ae014ae00a4e0ad2601b2600b540756a012da' +
  '1695d0095c0149a0da4d01a4a01aa40bb54016d4012da0495b00936';

const CODES = [];
for (let i = 0; i < PACKED.length; i += 5) {
  CODES.push(parseInt(PACKED.slice(i, i + 5), 16));
}

const DAY_MS = 86400000;

/** 그 해의 달들을 차례대로. 윤달은 제 달 바로 뒤에 낀다 — 1..L, 윤L, L+1..12. */
function monthsOf(year) {
  const code = CODES[year - LUNAR_FIRST_YEAR];
  const leap = code >> 13;
  const count = leap ? 13 : 12;
  const out = [];
  for (let i = 0; i < count; i += 1) {
    const len = (code >> (12 - i)) & 1 ? 30 : 29;
    if (leap && i === leap) out.push({ month: leap, leap: true, len });
    else out.push({ month: leap && i > leap ? i : i + 1, leap: false, len });
  }
  return out;
}

function yearLength(year) {
  let total = 0;
  for (const m of monthsOf(year)) total += m.len;
  return total;
}

/**
 * 양력 → 음력. 표 밖이면 `null`.
 *
 * 달·날이 넘치면 그냥 날짜 산술로 넘긴다 — 한글도 그렇게 답한다(2026년 13월 1일은
 * 2027년 1월 1일이다).
 */
export function solarToLunar(solarYear, solarMonth, solarDay) {
  const utc = Date.UTC(solarYear, solarMonth - 1, solarDay);
  if (!Number.isFinite(utc)) return null;
  if (utc < EPOCH_UTC || utc > LAST_SOLAR_UTC) return null;
  let days = Math.round((utc - EPOCH_UTC) / DAY_MS);
  let year = LUNAR_FIRST_YEAR;
  for (;;) {
    if (year > LUNAR_LAST_YEAR) return null;
    const len = yearLength(year);
    if (days < len) break;
    days -= len;
    year += 1;
  }
  for (const m of monthsOf(year)) {
    if (days < m.len) return { year, month: m.month, day: days + 1, leap: m.leap };
    days -= m.len;
  }
  return null;
}

/**
 * 음력 → 양력. 표 밖이거나 없는 날이면 `null`.
 *
 * 그 해에 윤달이 없으면 `leap` 은 무시한다 — 한글도 그렇게 답한다(2026년 윤6월을 물어도
 * 평6월과 같은 날을 준다).
 */
export function lunarToSolar(lunarYear, lunarMonth, lunarDay, leap = false) {
  if (!Number.isInteger(lunarYear) || lunarYear < LUNAR_FIRST_YEAR || lunarYear > LUNAR_LAST_YEAR) {
    return null;
  }
  let days = 0;
  for (let year = LUNAR_FIRST_YEAR; year < lunarYear; year += 1) days += yearLength(year);
  const months = monthsOf(lunarYear);
  const wanted = months.find((m) => m.month === lunarMonth && m.leap === Boolean(leap))
    ?? months.find((m) => m.month === lunarMonth && !m.leap);
  if (!wanted) return null;
  for (const m of months) {
    if (m === wanted) break;
    days += m.len;
  }
  if (!Number.isInteger(lunarDay) || lunarDay < 1 || lunarDay > wanted.len) return null;
  const at = new Date(EPOCH_UTC + (days + lunarDay - 1) * DAY_MS);
  return { year: at.getUTCFullYear(), month: at.getUTCMonth() + 1, day: at.getUTCDate() };
}
