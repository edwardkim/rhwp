# 최종 결과 보고서 — Task #108

**이슈**: [#108](https://github.com/edwardkim/rhwp/issues/108)
**타이틀**: Chrome 확장 썸네일 추출 실패 — CFB 디렉토리 섹터 체인 미순회
**마일스톤**: M100
**완료일**: 2026-04-12
**브랜치**: `local/task108`

---

## 결과 요약

`shift-return.hwp` 호버 시 썸네일이 표시되지 않던 문제를 수정 완료하였다.
원인은 두 가지 CFB 파싱 누락이었다.

---

## 원인 분석

### 원인 1: 디렉토리 섹터 FAT 체인 미순회

기존 코드는 첫 번째 디렉토리 섹터만 탐색하였다.
`sectorSize=512`이면 섹터당 4개 엔트리만 수용하므로, 엔트리 [4] 이상은 탐색 불가능하였다.

`shift-return.hwp`의 디렉토리 구조:
- 섹터 2 (엔트리 [0]~[3]): Root Entry, FileHeader, DocInfo, BodyText
- 섹터 4 (엔트리 [4]~): HwpSummaryInformation, **PrvImage** ← 탐색 실패

### 원인 2: Mini Stream 미지원

OLE2 CFB 스펙에 따르면 크기가 `miniStreamCutoff`(4096바이트) 미만인 스트림은
일반 FAT 섹터가 아닌 **Mini Stream**에 저장된다.

`shift-return.hwp`의 PrvImage: 1264바이트 GIF → Mini Stream 저장
기존 코드는 일반 FAT 체인으로만 읽어 매직 바이트 불일치 → `parseImageData` null 반환

---

## 수정 내용

### `rhwp-chrome/sw/thumbnail-extractor.js`

| 함수 | 변경 |
|------|------|
| `buildFatTable()` | FAT 테이블 구성 로직을 독립 함수로 분리 (신규) |
| `buildMiniFatTable()` | Mini FAT 테이블 구성 함수 추가 (신규) |
| `readStreamFromMini()` | Mini Stream 미니 섹터 체인 읽기 함수 추가 (신규) |
| `extractPrvImage()` | 디렉토리 섹터 while 루프 순회 + Mini Stream 분기 구현 |
| `readStreamFromFAT()` | `fatEntries` 선택적 파라미터 추가 (재사용 지원) |

#### `extractPrvImage()` 핵심 로직

```js
// 디렉토리 섹터 FAT 체인 전체 순회
while (dirSector < 0xFFFFFFFE) {
  // 현재 섹터 엔트리 탐색 ...
  dirSector = fatEntries[dirSector]; // 다음 섹터로 이동
}

// PrvImage 발견 시 Mini Stream 분기
if (streamSize < miniStreamCutoff && miniStreamData) {
  streamData = readStreamFromMini(...);   // Mini Stream
} else {
  streamData = readStreamFromFAT(...);    // 일반 FAT
}
```

---

## 검증

| 파일 | PrvImage | 저장 위치 | 결과 |
|------|----------|-----------|------|
| shift-return.hwp | GIF 177×250, 1,264 bytes | Mini Stream | ✅ 썸네일 표시 |
| biz_plan.hwp | PNG 724×1024, 12,097 bytes | 일반 FAT | ✅ 기존 동일 |
| (다수 HWP 파일) | PNG 724×1024 | 일반 FAT | ✅ 정상 동작 확인 |

---

## 부산물

- 정오표 **#28** 추가 (`mydocs/tech/hwp_spec_errata.md`):
  CFB 디렉토리 섹터 FAT 체인 순회 미기재
  - OLE2 표준 위임만 기술되어 있고 구체 구현 없음
  - Mini Stream 관련 내용도 미기재

---

## 커밋 목록

| 커밋 | 내용 |
|------|------|
| `f52f121` | CFB 디렉토리 섹터 FAT 체인 순회 구현 + 정오표 #28 |
| `b000e67` | CFB Mini Stream 지원 추가 — PrvImage 썸네일 완전 수정 |
