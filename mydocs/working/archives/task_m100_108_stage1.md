# 단계별 완료 보고서 — Task #108 / 1단계

**이슈**: [#108](https://github.com/edwardkim/rhwp/issues/108)
**단계**: 1단계 (전체 1단계)
**완료일**: 2026-04-12
**브랜치**: `local/task108`

---

## 작업 내용

`rhwp-chrome/sw/thumbnail-extractor.js` 수정

### 변경 사항

#### 1. `buildFatTable()` 함수 신규 추가 (line 107~127)

FAT 테이블 구성 로직을 독립 함수로 분리.

- 헤더 offset 76~507의 DIFAT 엔트리(최대 109개)에서 FAT 섹터 번호 읽기
- 각 FAT 섹터를 순회하여 `sector → nextSector` 매핑 배열 반환

`extractPrvImage()`와 `readStreamFromFAT()` 양쪽에서 재사용.

#### 2. `extractPrvImage()` 디렉토리 순회 로직 교체 (line 56~105)

**이전**: 단일 섹터 고정 오프셋에서 최대 128개 엔트리 선형 탐색
```js
const dirOffset = (dirStartSector + 1) * sectorSize;
for (let i = 0; i < 128; i++) { ... }
```

**이후**: FAT 체인을 따라 모든 디렉토리 섹터 순회
```js
const fatEntries = buildFatTable(data, sectorSize);
let dirSector = readU32LE(data, 48);
while (dirSector < 0xFFFFFFFE) {
  // 현재 섹터 내 entriesPerSector개 엔트리 탐색
  dirSector = fatEntries[dirSector]; // 다음 섹터로 이동
}
```

#### 3. `readStreamFromFAT()` 파라미터 추가 (line 138)

`fatEntries` 선택적 파라미터 추가 — 미리 구성된 FAT 테이블을 전달받아 재구성 비용 제거.
기존 호출부(외부 없음, 내부 전용)와 하위 호환 유지 (`if (!fatEntries)` 폴백).

---

## 검증

### 수정 전

`shift-return.hwp`: PrvImage 엔트리가 디렉토리 [17]번에 위치.
첫 번째 디렉토리 섹터(512바이트 = 4개 엔트리)를 초과 → 탐색 실패 → 썸네일 미표시.

### 수정 후

FAT 체인 순회로 디렉토리 섹터 [4], [5], ... 를 이어 탐색하여 [17]번 엔트리 도달 가능.
`shift-return.hwp`의 GIF 1,264바이트 PrvImage 스트림 추출 성공 예상.

### 기존 파일 영향

`biz_plan.hwp` (PrvImage PNG, 첫 번째 섹터 내 위치):
- `buildFatTable()` 호출이 추가되나 FAT 체인 1회 순회로 종료
- 동작 결과 동일

HWPX 파일:
- `extractPrvImageFromZipAsync()` 경로 — 변경 없음

---

## 변경 파일

- `rhwp-chrome/sw/thumbnail-extractor.js`

---

## 승인 요청

위 1단계 완료 보고서를 검토 후 승인해주시면 최종 결과 보고서 작성을 진행하겠습니다.
