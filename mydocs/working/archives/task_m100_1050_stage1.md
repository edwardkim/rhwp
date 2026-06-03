# Task #1050 Stage 1 보고서 — Footnote/Endnote CTRL_HEADER + LIST_HEADER contract 규명

- 이슈: [#1050](https://github.com/edwardkim/rhwp/issues/1050)
- 단계: Stage 1 (정밀 진단 — 의미 규명 우선)
- 일시: 2026-05-21

## 1. 결과 요약

`hwplib` (`/home/edward/vsworks/shwp/hwplib`) 권위 자료 + 다중 fixture raw byte 분석으로 CTRL_FOOTNOTE / CTRL_ENDNOTE / LIST_HEADER (footnote/endnote 안) 의 완전한 contract 규명 완료. **raw 보존 fallback 불필요 — 모든 field 의미 식별됨**.

## 2. 다중 fixture raw byte 패턴

footnote 보유 3 fixture 전수 추출:

| Fixture | CTRL_HEADER size | LIST_HEADER size | instanceId 보유 |
|---------|-----------------|------------------|----------------|
| `samples/footnote-tbox-01.hwp` (한컴 변환본) | **20** | **16** | ✓ |
| `samples/footnote-01.hwp` (한컴 신버전) | **20** | **16** | ✓ |
| `samples/2010-01-06.hwp` (한컴 구버전) | **16** | **8** | ❌ (생략) |

→ size 20 vs 16 의 차이 = **instanceId 4 byte 의 optional 여부** (구버전 한글이 생략).

## 3. CTRL_FOOTNOTE payload contract

`hwplib::ControlFootnote::ctrlHeader`:

```java
sr.readUInt4();              // number
readWChar();                 // beforeDecorationLetter
readWChar();                 // afterDecorationLetter
sr.readUInt4();              // numberShape
if (sr.isEndOfRecord()) return;  // optional cutoff
sr.setInstanceId(sr.readUInt4());
```

| Offset | Size | Field | hwplib type | HWPX 매핑 |
|--------|------|-------|-------------|----------|
| 0 | 4 | ctrl_id ('fn  ') | u32 | (record header) |
| 4 | 4 | **number** | UInt4 | `<hp:footNote number="">` |
| 8 | 2 | **beforeDecorationLetter** | WChar (HWPString) | (HWPX 미보유, default 0) |
| 10 | 2 | **afterDecorationLetter** | WChar (HWPString) | `suffixChar` (e.g. 0x29 = ')') |
| 12 | 4 | **numberShape** | UInt4 | FootnoteShape 참조 또는 default 0 (Digit) |
| 16 | 4 | **instanceId** (optional) | UInt4 | `instId="52"` 활용 가능, default = 한컴 식별자 |

### 본 sample 의 실측 값 검증

**footnote-tbox-01.hwp 글상자 안 #1** (size=20):
```
ctrl_id=20206e66  number=01000000=1  before=0000  after=2900=0x0029=')'
numberShape=00000000=0(Digit)  instanceId=34000000=0x34
```

**footnote-tbox-01.hwp 본문 #2** (size=20):
```
ctrl_id=20206e66  number=02000000=2  before=0000  after=2900=')'
numberShape=00000000=0  instanceId=10010000=0x110
```

**footnote-01.hwp #1** (size=20):
```
ctrl_id=20206e66  number=01000000=1  before=0000  after=2900=')'
numberShape=00000000=0  instanceId=10000000=0x10
```

→ 모든 한컴 한글 신버전 footnote 의 default pattern:
- before = 0 (없음)
- after = 0x0029 = ')'
- numberShape = 0 (Digit)
- instanceId = 한컴 내부 고유 식별자 (HWPX `instId` 또는 합성)

## 4. LIST_HEADER for Footnote/Endnote contract

`hwplib::ForListHeaderForFootnodeEndnote::write`:

```java
sw.writeRecordHeader(HWPTag.LIST_HEADER, 16);
sw.writeSInt4(lh.getParaCount());       // 4 byte
sw.writeUInt4(lh.getProperty().getValue());  // 4 byte (ListHeaderProperty)
sw.writeZero(8);                         // 8 byte padding
```

| Offset | Size | Field | Type | Default |
|--------|------|-------|------|---------|
| 0 | 4 | paraCount | SInt4 | footnote.paragraphs.len() |
| 4 | 4 | property | UInt4 | 0 (TextDirection=Horizontal, defaults) |
| 8 | 8 | zero padding | bytes | `00 00 00 00 00 00 00 00` |

### 본 sample 의 실측 값

모든 한컴 footnote LIST_HEADER (size=16) payload 동일:
```
01000000 00000000 00000000 00000000
```
→ paraCount=1, property=0, padding=zero. **HWPX 출처는 paraCount = footnote.paragraphs.len() 만 다름**.

## 5. CTRL_ENDNOTE / Endnote LIST_HEADER 추정

`hwplib` 의 `ControlEndnote` + `ForControlEndnote` 동일 구조:
- CTRL_ENDNOTE ('en  ') payload = CTRL_FOOTNOTE 와 동일 (number + before + after + numberShape + optional instanceId)
- LIST_HEADER for Endnote = LIST_HEADER for Footnote 동일

→ 두 영역 동일 본질 처리 가능. Stage 2 에서 Footnote + Endnote 공통 헬퍼.

## 6. HWPX 출처 → HWP 저장 매핑 contract

### HWPX `<hp:footNote number="" suffixChar="" instId="">` → CTRL_FOOTNOTE

| HWPX 속성 | HWP CtrlHeaderFootnote 필드 | 처리 |
|-----------|---------------------------|------|
| `number` | `number` (UInt4) | 직접 매핑 |
| `suffixChar` (int, e.g. 41) | `afterDecorationLetter` (WChar) | char 변환 (41 → 0x29 ')') |
| `instId` (string) | `instanceId` (UInt4) | 파싱 + UInt4 변환 |
| (HWPX 미보유) | `beforeDecorationLetter` | default = 0 |
| (HWPX 미보유) | `numberShape` | default = 0 (Digit) 또는 header.footnote의 numberShape 참조 |

### CTRL_FOOTNOTE size 결정

- size=20 (instanceId 포함) = 현대 표준 → **본 task 채택**
- size=16 (instanceId 생략) = 구버전 호환 → 본 task 비범위

## 7. 모델 변경 안 (Stage 2)

### 기존 (`src/model/footnote.rs:8`)

```rust
pub struct Footnote {
    pub number: u16,
    pub paragraphs: Vec<Paragraph>,
}
```

### 변경 안

```rust
pub struct Footnote {
    pub number: u32,                      // u16 → u32 (UInt4)
    pub before_decoration_letter: u16,    // WChar (0 = 없음)
    pub after_decoration_letter: u16,     // WChar (default 0x29 = ')')
    pub number_shape: u32,                // UInt4 (default 0 = Digit)
    pub instance_id: u32,                 // UInt4 (한컴 고유 식별자)
    pub list_header_property: u32,        // ListHeaderProperty (default 0)
    pub paragraphs: Vec<Paragraph>,
}
// Endnote 동일 변경
```

후방 호환:
- `number: u16` → `number: u32` 변경 시 caller 수정 필요 (검색 후 일괄 update)
- 또는 새 필드만 추가 + `number: u16` 유지 + serializer 에서 u32 cast

→ **결정**: caller 수정 영향 최소화를 위해 `number: u16` 유지 + 추가 필드 신규.

```rust
pub struct Footnote {
    pub number: u16,                          // (기존)
    pub paragraphs: Vec<Paragraph>,           // (기존)
    // [Task #1050] HWP5 CTRL_FOOTNOTE payload 보존
    pub before_decoration_letter: u16,        // WChar (default 0)
    pub after_decoration_letter: u16,         // WChar (default 0x29 ')')
    pub number_shape: u32,                    // UInt4 (default 0 = Digit)
    pub instance_id: u32,                     // UInt4 (한컴 고유 식별자)
    // [Task #1050] LIST_HEADER property
    pub list_header_property: u32,            // UInt4 (default 0)
}
```

## 8. 다음 단계

Stage 2 구현:
1. `src/model/footnote.rs` Footnote/Endnote 필드 추가
2. `src/parser/control.rs` parse_footnote_control / parse_endnote_control 새 필드 파싱
3. `src/serializer/control.rs` serialize_footnote / serialize_endnote 새 필드 직렬화 + footnote LIST_HEADER 16 byte 형식
4. `src/parser/hwpx/section.rs::parse_ctrl_footnote` HWPX → HWP 매핑 (suffixChar / instId → 새 필드)
5. 본 환경 자기 라운드트립 SVG 검증 (글상자 안 각주 본문 표시) + hwp5-inventory-diff (CTRL_HEADER size=20 정합)
