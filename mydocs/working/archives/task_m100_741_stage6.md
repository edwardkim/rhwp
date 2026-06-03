# Task #741 Stage 6 — HWP3 ParaShape tab_def 처리

## 영역

HWP3 ParaShape 의 tabs[40] 영역을 Document IR TabDef 로 변환. 페이지 번호 우측 정렬 + 가로선 leader 채움 (TOC 등) 시각 정합 확보.

## 본질 진단

### 결함 1 — `Hwp3TabDef` 구조체 필드 순서 bug

기존 ([records.rs:240-252](src/parser/hwp3/records.rs#L240)):
```rust
pub struct Hwp3TabDef {
    pub position: u16,   // (실제로는 마지막 2 byte)
    pub tab_type: u8,    // (실제로는 첫 byte)
    pub leader: u8,      // (실제로는 두 번째 byte)
}

impl Hwp3TabDef::read {
    let position = reader.read_u16::<LittleEndian>()?;  // 실제 데이터 어긋남
    let tab_type = reader.read_u8()?;
    let leader = reader.read_u8()?;
}
```

진단 도출 영역: HWP3_DIAG_TABS env 임시 추가하여 1076 paragraphs 의 raw bytes 패턴 분석.

기본 탭 default 패턴 검증:
- Slot 0: bytes [0, 0, 0xE8, 0x03] → position LE = 0x03E8 = **1000 hunit (≈14mm)**
- Slot 1: bytes [0, 0, 0xD0, 0x07] → position LE = 0x07D0 = **2000 hunit (≈28mm)**
- Slot N: position = **(N+1) × 1000 hunit** (system default tab interval)

→ 실제 byte 순서: **`tab_type(u8) → leader(u8) → position(u16 LE)` = 4 bytes**

### 결함 2 — convert_para_shape tab 변환 부재

`convert_para_shape` 가 tabs[40] 을 무시. ParaShape.tab_def_id 항상 0 (정의 없음). 결과:
- TOC entries 가로선 leader 표시 안 됨
- 페이지 번호 우측 정렬 안 됨

## 정정 영역

### 1. `src/parser/hwp3/records.rs` — Hwp3TabDef::read 필드 순서 정정

```rust
impl Hwp3TabDef::read {
    let tab_type = reader.read_u8()?;
    let leader = reader.read_u8()?;
    let position = reader.read_u16::<LittleEndian>()?;
}
```

### 2. `src/parser/hwp3/mod.rs` — convert_para_shape tab 변환 추가

- Hwp3TabDef[40] → Document IR TabItem 변환
- default tab pattern (`pos = 1000 × (slot_idx + 1)` + `tab_type=0` + `leader=0`) 제외
- empty slot (`pos=0` + `tab_type=0` + `leader=0`) 제외
- explicit user tab 만 TabItem 으로 push, find_or_create_tab_def 로 등록

### 3. parse_paragraph_list / parse_drawing_object_tree / parse_shape_list / map_to_shape_object signature 확장

`doc_tab_defs: &mut Vec<TabDef>` 매개변수 추가 (재귀 nested paragraph 영역 포함).

### 4. 최상위 entry 에서 doc_tab_defs Vec 생성 + doc.doc_info.tab_defs 할당

## 검증

### 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | 1166 passed |
| `cargo clippy --release --lib` | 신규 경고 0 |
| `scripts/svg_regression_diff.sh build 86bf0bd HEAD` | TOTAL pages=170 same=170 diff=0 (회귀 0) |

### 결함 1 검증 — paragraph 29 (TOC entry "EXPORT/IMPORT Q & A")

| 항목 | 정정 전 | 정정 후 | HWP5 변환본 |
|------|---------|---------|-------------|
| tab_def_id | 0 (정의 없음) | 3 | 1 |
| pos | (없음) | 42516 (150.0mm) | 85032 (300.0mm) |
| type | (없음) | 1 (right) | 1 (right) |
| fill | (없음) | 1 (solid) | 3 (dashed) |

본 환경 HWP3 native pos 150mm = A4 content right edge (210mm - 30mm × 2 margins) — 정합. HWP5 변환본 의 300mm 는 다른 reference frame (HWP5 변환 시 재계산) 추정.

### 시각 정합 (PDF cross-check)

`pdf/hwp3-sample10-hwp5-2022.pdf` 페이지 2 (제목차례 + TOC) 와 본 환경 HWP3 native 페이지 2 비교:

**Stage 6 정정 효과:**
- ✓ TOC entries 가로선 leader (──────) 표시
- ✓ 페이지 번호 우측 정렬 (TOC entry 끝 page_num)
- ✓ 시각 구조 PDF 정합

**잔여 영역 (Stage 6 외):**
- ═════ 제목차례 ═════ 의 양옆 ═ borders — paragraph 26 자체 cc=8 텍스트 안에 ═ 없음 (별도 paragraph 또는 ParaShape border)
- 페이지 번호 [n] box 표시 — U+F0... PUA char 폰트 fallback (한컴 사적 폰트 필요)
- HWP5 변환본의 fill=3 (dashed) 와 본 환경 fill=1 (solid) 미세 차이 — leader 값 mapping convention 추가 조사 영역

## 후속

- HWP3 leader 값 ↔ HWP5 fill_type 정밀 매핑 (Stage 7+ 영역)
- 한컴 사적 폰트 fallback (PUA glyph 영역)
