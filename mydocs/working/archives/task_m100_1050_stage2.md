# Task #1050 Stage 2 보고서 — Model + Parser + Serializer 정정

- 이슈: [#1050](https://github.com/edwardkim/rhwp/issues/1050)
- 단계: Stage 2 (구현)
- 일시: 2026-05-21

## 1. 결과 요약

Stage 1 contract 규명 결과 (`hwplib` 권위) 에 따라 Footnote + Endnote 의 CTRL_HEADER + LIST_HEADER 직렬화 정정 완료. **본 환경 자기 라운드트립 정합 입증** (글상자 안 각주 본문 "글상자 내부 각주" 페이지 하단 표시).

## 2. 변경 사항

### 2.1 `src/model/footnote.rs` (+10 라인)

```rust
pub struct Footnote {
    pub number: u16,
    pub paragraphs: Vec<Paragraph>,
    // [Task #1050] HWP5 CTRL_FOOTNOTE payload 보존
    pub before_decoration_letter: u16,   // WChar
    pub after_decoration_letter: u16,    // WChar (default 0x0029 ')')
    pub number_shape: u32,               // UInt4 (default 0 = Digit)
    pub instance_id: u32,                // UInt4
    pub list_header_property: u32,       // ListHeaderProperty
}
// Endnote 동일
```

### 2.2 `src/parser/control.rs` (+30 라인)

`parse_footnote_control` / `parse_endnote_control`:
- `number` 를 UInt4 (4 byte) 로 변경 (기존 UInt2 = 2 byte)
- ctrl_data 길이별 점진 파싱 (4/8/12/16 byte) — 한컴 구버전 size=12 호환

`find_list_header_property_for_footnote_endnote` 추가 — LIST_HEADER property field 추출.

### 2.3 `src/serializer/control.rs` (+35 라인)

`serialize_footnote` / `serialize_endnote`:
- CTRL_FOOTNOTE 17 byte payload 직렬화 (record header 제외, size=16):
  number(UInt4) + before(WChar) + after(WChar) + numberShape(UInt4) + instanceId(UInt4)
- `after_decoration_letter == 0` → default `0x0029 ')'`

`serialize_footnote_endnote_list_header` 신규 — LIST_HEADER 16 byte 형식:
  paraCount(SInt4) + property(UInt4) + 8 byte zero padding

### 2.4 `src/parser/hwpx/section.rs` (+30 라인)

`parse_ctrl_footnote` / `parse_ctrl_endnote`:
- HWPX `suffixChar` 속성 → `after_decoration_letter` 매핑
- HWPX `instId` 속성 → `instance_id` 매핑
- 기본값 `after_decoration_letter = 0x0029`

### 2.5 caller 영향 정정

- `src/document_core/commands/object_ops.rs` Footnote 생성자
- `src/serializer/control/tests.rs` 테스트 fixture (Footnote default suffix)
- `src/serializer/hwpx/mod.rs` 테스트 fixture (Footnote/Endnote)
- `src/parser/control/tests.rs::test_parse_footnote_control` payload 정합 (UInt4 number + 16 byte)

## 3. 정량 입증 — 자기 라운드트립

### 3.1 본 sample HWPX → HWP 저장 → 재로드 → SVG

| 항목 | Task #1050 이전 | Task #1050 Stage 2 후 |
|------|----------------|---------------------|
| 본문 각주 "일반 문단내 각주" | ✓ | ✓ (회귀 부재) |
| **글상자 안 각주 "글상자 내부 각주"** | ❌ 누락 | ✓ **표시** |
| SVG text element 수 | 42 | 51 (+9) |

### 3.2 한컴 정답지 vs Stage 2 후 — hwp5-inventory-diff

| 위치 | 정답지 | Stage 0 (Task #1050 이전) | Stage 2 후 |
|------|--------|--------------------------|-----------|
| CTRL_FOOTNOTE size | 20 | **6** | **20** ✓ |
| Footnote LIST_HEADER size | 16 | **6** | **16** ✓ |

→ Footnote 관련 diff 0 (한컴 정답지 완전 정합).

## 4. 다음 단계

Stage 3 (회귀 가드 + sweep + WASM) + Stage 4 (작업지시자 한컴 시각 판정 + 머지).
