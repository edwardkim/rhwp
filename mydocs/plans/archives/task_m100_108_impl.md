# 구현 계획서 — Task #108

**이슈**: [#108](https://github.com/edwardkim/rhwp/issues/108)
**타이틀**: Chrome 확장 썸네일 추출 실패 — CFB 디렉토리 섹터 체인 미순회
**마일스톤**: M100
**작성일**: 2026-04-12
**브랜치**: `local/task108`

---

## 수정 대상

`rhwp-chrome/sw/thumbnail-extractor.js` — `extractPrvImage()` 함수 (line 56~96)

---

## 현재 코드 분석

### 문제 구간 (line 66~93)

```js
const dirStartSector = readU32LE(data, 48);
const dirOffset = (dirStartSector + 1) * sectorSize;

for (let i = 0; i < 128; i++) {        // ← 단일 섹터 내 128개 엔트리만 탐색
  const entryOffset = dirOffset + i * 128;
  if (entryOffset + 128 > data.length) break;
  ...
}
```

**문제점**: `dirOffset`을 고정값으로 두고 단일 섹터를 선형 순회하고 있다.  
`sectorSize = 512`이면 섹터당 4개 엔트리 (`512 / 128 = 4`)만 실제로 포함된다.  
엔트리 [4] 이상은 FAT 체인으로 연결된 다음 디렉토리 섹터에 있다.

### 기존 `readStreamFromFAT()` 패턴 (line 101~145)

FAT 체인 탐색 로직이 이미 구현되어 있다:
1. 헤더 offset 76~507에서 DIFAT 엔트리 읽기 (최대 109개 FAT 섹터 번호)
2. 각 FAT 섹터를 읽어 `fatEntries[]` 배열 구성
3. `startSector`에서 출발해 `fatEntries[sector]`를 따라 이동, `0xFFFFFFFE`에서 종료

디렉토리 섹터 탐색에도 동일한 패턴을 적용하면 된다.

---

## 구현 단계

### 1단계: FAT 테이블 추출 분리 + 디렉토리 체인 순회 구현

**변경 범위**: `extractPrvImage()` 함수 내부만 수정 (외부 API 변경 없음)

#### 1-1. FAT 테이블 추출을 별도 함수로 분리

현재 `readStreamFromFAT()` 내부에서만 FAT 테이블을 구성한다.  
`extractPrvImage()`에서도 FAT 테이블이 필요하므로 공통 함수로 분리한다.

```js
/**
 * CFB FAT 테이블을 구성하여 반환한다.
 * @returns {number[]} sector → nextSector 매핑 배열
 */
function buildFatTable(data, sectorSize) {
  const fatEntries = [];
  for (let i = 0; i < 109; i++) {
    const fatSect = readU32LE(data, 76 + i * 4);
    if (fatSect === 0xFFFFFFFE || fatSect === 0xFFFFFFFF) break;
    const fatOffset = (fatSect + 1) * sectorSize;
    const entriesPerSector = sectorSize / 4;
    for (let j = 0; j < entriesPerSector; j++) {
      const off = fatOffset + j * 4;
      if (off + 4 > data.length) break;
      fatEntries.push(readU32LE(data, off));
    }
  }
  return fatEntries;
}
```

#### 1-2. `extractPrvImage()`에서 모든 디렉토리 섹터 순회

```js
const fatEntries = buildFatTable(data, sectorSize);
let dirSector = readU32LE(data, 48);   // 헤더 offset 48: 첫 번째 디렉토리 섹터

while (dirSector < 0xFFFFFFFE) {
  const dirOffset = (dirSector + 1) * sectorSize;
  const entriesPerSector = sectorSize / 128;

  for (let i = 0; i < entriesPerSector; i++) {
    const entryOffset = dirOffset + i * 128;
    if (entryOffset + 128 > data.length) break;

    const nameLen = readU16LE(data, entryOffset + 64);
    if (nameLen === 0 || nameLen > 64) continue;

    const name = readUTF16LE(data, entryOffset, nameLen);
    if (name !== 'PrvImage') continue;

    // ... 스트림 읽기 (기존 코드 유지)
  }

  // 다음 디렉토리 섹터로 이동
  dirSector = (dirSector < fatEntries.length) ? fatEntries[dirSector] : 0xFFFFFFFE;
}
```

#### 1-3. `readStreamFromFAT()`에서 `buildFatTable()` 재사용

FAT 테이블을 내부에서 구성하던 코드를 `buildFatTable()` 호출로 교체.

---

### 2단계: 검증

#### 2-1. shift-return.hwp 썸네일 추출 확인

- 브라우저 확장에서 `shift-return.hwp` 파일 호버 → 썸네일 표시 확인
- 콘솔에서 직접 확인:
  ```js
  // Service Worker DevTools console
  const r = await fetch('/path/to/shift-return.hwp');
  const buf = await r.arrayBuffer();
  // extractPrvImage 수동 호출
  ```

#### 2-2. 기존 파일 회귀 확인

- `biz_plan.hwp` (PNG, PrvImage가 앞 섹터에 위치) — 기존과 동일하게 추출되는지 확인
- HWPX 파일 — ZIP 경로는 변경 없으므로 영향 없음

---

## 최종 diff 예상 규모

- `buildFatTable()` 함수 신규 추가: ~15줄
- `extractPrvImage()` 내부 디렉토리 순회 로직 교체: ~20줄 (순증)
- `readStreamFromFAT()` 내부 FAT 구성 코드 → `buildFatTable()` 호출로 교체: ~10줄 감소
- **순 변경**: 약 +25줄

---

## 승인 요청

위 구현 계획서를 검토 후 승인해주시면 1단계 구현을 시작하겠습니다.
