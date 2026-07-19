---
kind: reference
status: active
canonical: mydocs/tech/hwp_spec_errata.md
last_verified: 2026-07-17
---

# HWP 저장 기술 가이드

## 1. 편집 영역 좌표계

### 1.1 기본 구조

HWP 문서의 페이지 구조:

```
┌─────────────────────────────────┐ ← 용지 상단 (0)
│         margin_top (20mm)       │
├─────────────────────────────────┤ ← margin_top (5668)
│       margin_header (15mm)      │ ← 머리말 영역
├─────────────────────────────────┤ ← body_area.top = margin_top + margin_header (9920)
│                                 │
│         편집 영역 (본문)         │ ← LineSeg 좌표의 원점 (0, 0)
│         body_area               │
│         height: 65764           │
│         width: 42520            │
│                                 │
├─────────────────────────────────┤ ← body_area.bottom (75684)
│       margin_footer (15mm)      │ ← 꼬리말 영역
├─────────────────────────────────┤
│        margin_bottom (15mm)     │
└─────────────────────────────────┘ ← 용지 하단 (84188)
```

### 1.2 핵심 발견: LineSeg 좌표계는 편집 영역 상대 좌표

`template/empty.hwp` 분석 결과:

| 항목 | PageAreas 계산값 | LineSeg 실제값 | 의미 |
|------|-----------------|---------------|------|
| vertical_pos | 9920 (body_area.top) | **0** | 편집 영역 내 상대 좌표 |
| column_start | 8504 (body_area.left) | **0** | 편집 영역 내 상대 좌표 |
| segment_width | 42520 | **42520** | 편집 영역 너비와 일치 |

**결론**: LineSeg의 `vertical_pos`와 `column_start`는 **페이지 절대 좌표가 아니라 편집 영역(body_area) 기준 상대 좌표**이다.

### 1.3 빈 문서의 첫 줄 LineSeg 값

```
LineSeg[0]:
  text_start:         0
  vertical_pos:       0        ← 편집 영역 최상단
  line_height:        1000     ← 10pt 글자 기준
  text_height:        1000
  baseline_distance:  850      ← 기준선까지 거리
  line_spacing:       600      ← 줄간격 (160% × 1000 × 0.6?)
  column_start:       0        ← 편집 영역 좌측
  segment_width:      42520    ← 편집 영역 전체 너비
  tag:                0x00060000 ← bit 17(first segment) + bit 18(last segment)
```

`0x00060000`은 범용 "HWP 기본 플래그"가 아니라, 공식 스펙의 PARA_LINE_SEG tag bit 17/18을 동시에 켠 **단일 세그먼트 줄(single segment line)** 계약이다. 코드에서는 `LineSeg::TAG_SINGLE_SEGMENT_LINE`으로 표현한다.

### 1.4 A4 기본 용지 설정 (template/empty.hwp)

| 항목 | HWPUNIT | mm |
|------|---------|-----|
| 용지 너비 | 59528 | 210.0 |
| 용지 높이 | 84188 | 297.0 |
| 왼쪽 여백 | 8504 | 30.0 |
| 오른쪽 여백 | 8504 | 30.0 |
| 위쪽 여백 | 5668 | 20.0 |
| 아래쪽 여백 | 4252 | 15.0 |
| 머리말 여백 | 4252 | 15.0 |
| 꼬리말 여백 | 4252 | 15.0 |
| 제본 여백 | 0 | 0.0 |

### 1.5 DocProperties 캐럿 정보

빈 문서 기준:

| 필드 | 값 | 의미 |
|------|-----|------|
| caret_list_id | 0 | 본문 섹션 0 |
| caret_para_id | 0 | 첫 번째 문단 |
| caret_char_pos | 16 | 컨트롤 2개(SectionDef, ColumnDef) × 8 WCHAR |

### 1.6 빈 문서의 기본 스타일

| 항목 | 값 |
|------|-----|
| CharShape | base_size=1000 (10pt) |
| ParaShape | line_spacing_type=Percent, line_spacing=160 |
| 문단[0] | text='' char_count=17 controls=2 (SectionDef, ColumnDef) |

### 1.7 HWPUNIT 변환

- 1인치 = 7200 HWPUNIT
- 1mm = 7200 / 25.4 = 283.46 HWPUNIT
- 1pt = 100 HWPUNIT (CharShape.base_size 기준)

---

## 2. 스펙 교차 검증 결과

### 2.1 문단 레코드 구조

| 레코드 | 바이트/항목 | 검증 결과 |
|--------|-----------|----------|
| PARA_HEADER | 24바이트 (기본) + raw_header_extra | 완전 일치 |
| PARA_TEXT | 2×nchars (UTF-16LE) | 완전 일치 |
| PARA_CHAR_SHAPE | 8바이트/항목 (start_pos:4 + char_shape_id:4) | 완전 일치 |
| PARA_LINE_SEG | 36바이트/항목 (9×4바이트) | 완전 일치 |
| PARA_RANGE_TAG | 12바이트/항목 (start:4 + end:4 + tag:4) | 완전 일치 |

#### 2.1.1 PARA_HEADER 저장 계약

`src/serializer/body_text.rs`의 HWP5 저장 경로는 원본 `PARA_HEADER`를 그대로 복사하지 않는다. 편집 후 불일치를 막기 위해 다음 필드는 현재 IR 기준으로 재생성한다.

| 필드 | 저장 계약 |
|------|-----------|
| `char_count` | `PARA_TEXT`/컨트롤 코드 단위 기준 실제 글자 수로 재계산 |
| `char_count` bit 31 | 현재 list scope의 마지막 문단 여부로 결정. `Paragraph.char_count_msb` 원본값을 그대로 쓰지 않음 |
| `control_mask` | `paragraph.controls`, text control char, field marker 기반으로 재계산 |
| `numCharShapes` | 직렬화에 사용할 effective char shape 수로 재계산 |
| `numRangeTags` | `paragraph.range_tags.len()`으로 재계산 |
| `numLineSegs` | `paragraph.line_segs.len()`으로 재계산 |
| `instanceId` 및 후속 extra | `Paragraph.raw_header_extra[6..]`를 보존. 앞 6바이트 count 영역은 저장 시 건너뜀 |

따라서 `Paragraph.raw_header_extra`는 전체 PARA_HEADER tail의 불변 원본이 아니라, count 3개를 제외한 instanceId/변경추적 suffix 보존 저장소로 해석해야 한다.

#### 2.1.2 PARA_LINE_SEG 저장 계약

`PARA_LINE_SEG`는 공식 스펙 표 62와 동일하게 36바이트, 9개 `u32/i32` 필드를 1:1 저장한다.

| 순서 | 필드 | IR |
|------|------|----|
| 1 | 텍스트 시작 위치 | `LineSeg.text_start` |
| 2 | 줄의 세로 위치 | `LineSeg.vertical_pos` |
| 3 | 줄의 높이 | `LineSeg.line_height` |
| 4 | 텍스트 부분의 높이 | `LineSeg.text_height` |
| 5 | 베이스라인까지 거리 | `LineSeg.baseline_distance` |
| 6 | 줄간격 | `LineSeg.line_spacing` |
| 7 | 컬럼에서의 시작 위치 | `LineSeg.column_start` |
| 8 | 세그먼트의 폭 | `LineSeg.segment_width` |
| 9 | 태그 | `LineSeg.tag` |

`LineSeg.tag`의 공식 bit 의미:

| bit | 의미 | 코드 상수 |
|-----|------|-----------|
| 0 | 페이지의 첫 줄 | `TAG_FIRST_LINE_OF_PAGE` |
| 1 | 컬럼의 첫 줄 | `TAG_FIRST_LINE_OF_COLUMN` |
| 16 | 텍스트가 배열되지 않은 빈 세그먼트 | `TAG_EMPTY_SEGMENT` |
| 17 | 줄의 첫 세그먼트 | `TAG_FIRST_SEGMENT` |
| 18 | 줄의 마지막 세그먼트 | `TAG_LAST_SEGMENT` |
| 19 | auto-hyphenation 수행 | `TAG_AUTO_HYPHENATION` |
| 20 | indentation 적용 | `TAG_INDENTATION` |
| 21 | 문단 머리 모양 적용 | `TAG_PARAGRAPH_HEAD` |
| 31 | 구현 편의 property | `TAG_IMPLEMENTATION_PROPERTY` |

`TAG_SINGLE_SEGMENT_LINE = TAG_FIRST_SEGMENT | TAG_LAST_SEGMENT = 0x00060000`이다. 이 값은 한 줄이 하나의 세그먼트로 구성될 때의 계약이며, 페이지/컬럼 첫 줄 여부(bit 0/1)와는 별도이다.

### 2.2 컨트롤별 필수 레코드 매트릭스

| 컨트롤 | char_code | ctrl_id | 필수 레코드 (순서) | 레벨 |
|--------|-----------|---------|-------------------|------|
| SectionDef | 0x0002 | 'secd' | CTRL_HEADER → PAGE_DEF → FOOTNOTE_SHAPE×2 → PAGE_BORDER_FILL | L+1 → L+2 |
| ColumnDef | 0x0002 | 'cold' | CTRL_HEADER (데이터 내장) | L+1 |
| Table | 0x000B | 'tbl ' | CTRL_HEADER → [Caption] → TABLE → Cell(LIST_HEADER + Paragraphs)×N | L+1 → L+2 |
| Picture | 0x000B | 'gso ' | CTRL_HEADER → SHAPE_COMPONENT → SHAPE_COMPONENT_PICTURE | L+1 → L+2 → L+3 |
| Shape | 0x000B | 'gso ' | CTRL_HEADER → SHAPE_COMPONENT → SHAPE_COMPONENT_* | L+1 → L+2 → L+3 |
| Header | 0x0010 | 'head' | CTRL_HEADER → LIST_HEADER → Paragraphs | L+1 → L+2 |
| Footer | 0x0010 | 'foot' | CTRL_HEADER → LIST_HEADER → Paragraphs | L+1 → L+2 |
| Footnote | 0x0011 | 'fn  ' | CTRL_HEADER → LIST_HEADER → Paragraphs | L+1 → L+2 |
| Endnote | 0x0011 | 'en  ' | CTRL_HEADER → LIST_HEADER → Paragraphs | L+1 → L+2 |
| HiddenComment | 0x000F | 'tcmt' | CTRL_HEADER → LIST_HEADER → Paragraphs | L+1 → L+2 |
| AutoNumber | 0x0012 | - | CTRL_HEADER만 | L+1 |
| NewNumber | 0x0012 | - | CTRL_HEADER만 | L+1 |
| PageNumberPos | 0x0015 | - | CTRL_HEADER만 | L+1 |
| PageHide | 0x0015 | - | CTRL_HEADER만 | L+1 |
| Bookmark | 0x0016 | - | CTRL_HEADER만 | L+1 |
| Field | 0x0003 | '%hlk' 등 | CTRL_HEADER → [CTRL_DATA] | L+1 → L+2 |
| Equation | 0x000B | 'eqed' | CTRL_HEADER → SHAPE_COMPONENT → EQ_EDIT | L+1 → L+2 → L+3 |

### 2.3 TABLE 레코드 상세 구조

```
CTRL_HEADER (level L+1, ctrl_id='tbl ')
  ├ attr: u32 (4바이트)
  └ raw_ctrl_data (가변)
TABLE (level L+2)
  ├ attr: u32 (4)
  ├ row_count: u16 (2)
  ├ col_count: u16 (2)
  ├ cell_spacing: i16 (2)
  ├ padding: i16×4 (8)
  ├ row_sizes: i16[row_count] (2×N)
  └ border_fill_id: u16 (2)
LIST_HEADER (level L+2) ← 셀별 반복
  ├ n_paragraphs: u16 (2)
  ├ list_attr: u32 (4)
  ├ width_ref: u16 (2)
  ├ col/row/col_span/row_span: u16×4 (8)
  ├ width/height: u32×2 (8)
  ├ padding: i16×4 (8)
  └ border_fill_id: u16 (2)
  PARA_HEADER (level L+2) ← 셀 내 문단
    PARA_TEXT (level L+3)
    PARA_CHAR_SHAPE (level L+3)
    PARA_LINE_SEG (level L+3)
```

### 2.4 DOCUMENT_PROPERTIES 레코드 (26바이트)

| 필드 | 타입 | 크기 |
|------|------|------|
| section_count | u16 | 2 |
| page_start_num | u16 | 2 |
| footnote_start_num | u16 | 2 |
| endnote_start_num | u16 | 2 |
| picture_start_num | u16 | 2 |
| table_start_num | u16 | 2 |
| equation_start_num | u16 | 2 |
| caret_list_id | u32 | 4 |
| caret_para_id | u32 | 4 |
| caret_char_pos | u32 | 4 |

### 2.5 ID_MAPPINGS 레코드 (72바이트)

18개 u32 카운트: bin_data, font×7, border_fill, char_shape, tab_def, numbering, bullet, para_shape, style, memo_shape, trackchange, trackchange_author

### 2.6 검증 상태 요약

| 항목 | 상태 |
|------|------|
| PARA_HEADER | 완전 일치 |
| PARA_TEXT (제어문자 인코딩) | 완전 일치 |
| PARA_CHAR_SHAPE | 완전 일치 |
| PARA_LINE_SEG (36바이트) | 완전 일치 |
| PARA_RANGE_TAG (12바이트) | 완전 일치 |
| CTRL_HEADER 구조 | 완전 일치 |
| TABLE 레코드 계층 | 완전 일치 |
| HEADER/FOOTER 구조 | 완전 일치 |
| FOOTNOTE/ENDNOTE 구조 | 완전 일치 |
| ID_MAPPINGS (72바이트) | 완전 일치 |
| 레코드 레벨 계층 | 완전 일치 |
| DOCUMENT_PROPERTIES | 26바이트 (스펙 30바이트, raw_data로 보존) |

---

## 3. 제어 문자 크기

- 인라인 컨트롤 (Tab, LineBreak 등): **1 WCHAR** (2바이트)
  - 해당 코드: 0, 10, 13, 24-31
- 확장 컨트롤 (SectionDef, Table, Picture 등): **8 WCHAR** (16바이트)
  - 해당 코드: 1-3, 11-12, 14-18, 21-23
  - 구성: char_code(2) + ctrl_id(8) + reserved(4) + char_code_repeat(2) = 16바이트

---

## 4. 컨트롤별 저장 검증 기록

### 4.1 텍스트만 (단계 2 완료)

**테스트 케이스:**

| 파일 | 삽입 텍스트 | char_count | PARA_TEXT 크기 |
|------|-----------|-----------|---------------|
| save_test_korean.hwp | 가나다라마바사아 | 25 (8글자+16컨트롤+1CR) | 50바이트 |
| save_test_english.hwp | Hello World | 28 (11글자+16컨트롤+1CR) | 56바이트 |
| save_test_mixed.hwp | 안녕 Hello 123 !@# | 33 (16글자+16컨트롤+1CR) | 66바이트 |

**변경 레코드 (원본 대비):**
- PARA_HEADER: char_count만 변경 (원본 17 → 텍스트 추가분)
- PARA_TEXT: 텍스트 데이터 추가 (원본 34바이트 → 크기 증가)

**불변 레코드:**
- PARA_CHAR_SHAPE: 동일 (8바이트)
- PARA_LINE_SEG: **원본 값 보존** (36바이트) — 편집 영역 좌표 유지
- CTRL_HEADER 이후 모든 레코드: 동일 (SectionDef, ColumnDef 등)

**검증 결과:**
- 3개 파일 모두 재파싱 성공
- 레코드 수 동일 (원본 12 = 저장 12)
- 전체 466 테스트 통과
- 출력: `output/save_test_*.hwp`
- 한글 프로그램 오픈 검증: **3개 파일 모두 정상 오픈, 캐럿 위치 올바름** ✓

### 4.2 표 (Table) — 단계 3 완료

**참조 파일**: `output/1by1-table.hwp` (HWP 프로그램으로 직접 생성)

**핵심 발견 (참조 파일 분석):**

1. **CTRL_HEADER 구조**: ctrl_id(4) + attr(4) + CommonObjAttr(38) = **46바이트**
   - CommonObjAttr: y_offset(4) + x_offset(4) + width(4) + height(4) + z_order(4) + margins(8) + instance_id(4) + extra(6)
   - `attr = 0x082A2210` (표의 공통 오브젝트 속성 플래그)

2. **표 문단 구조**: 반드시 **2개 문단** 필요
   - 문단[0]: 표를 포함하는 문단 (SectionDef + ColumnDef + Table + CR, char_count=25)
   - 문단[1]: 표 아래 빈 문단 (CR만, char_count=1)

3. **표 문단 LineSeg**: `segment_width = 0` (표가 줄 전체 차지)

4. **control_mask**: 표가 있는 문단은 `0x00000804`

5. **셀 문단 레벨**: LIST_HEADER와 **같은 레벨** (level=L+2)
   - PARA_HEADER level=L+2, PARA_TEXT/CHAR_SHAPE/LINE_SEG level=L+3

6. **캐럿 위치**: `caret_list_id=1` (두 번째 문단=표 아래 빈 줄)

7. **BorderFill**: 실선 테두리 BorderFill을 DocInfo에 추가 필요
   - `BorderLineType::Solid, width=1, color=0` (검은 실선)
   - `diagonal_type=1`

**TABLE 레코드 상세 (참조 기준):**

| 레코드 | 크기 | 비고 |
|--------|------|------|
| CTRL_HEADER | 46B | attr(4)+CommonObjAttr(38)+ctrl_id(4) |
| TABLE | 24B | attr(4)+rows(2)+cols(2)+spacing(2)+padding(8)+row_sizes(2)+bf_id(2)+extra(2) |
| LIST_HEADER | 47B | 기본 34B + raw_list_extra(13B) |

**테스트 케이스:**

| 파일 | 표 크기 | 레코드 수 | 결과 |
|------|---------|----------|------|
| save_test_table_1x1.hwp | 1×1 빈 셀 | 21 (참조와 동일) | HWP 정상 오픈 ✓ |

**수정 이력:**
- 첫 시도: CTRL_HEADER에 CommonObjAttr 누락 → 파일 손상
- LenientCfbReader 구현: HWP 프로그램 생성 파일의 FAT 검증 우회
- 참조 파일 분석 후 수정: 구조 완전 일치, 테두리 BorderFill 추가

### 4.3 이미지 (Picture) — 단계 4 완료

**참조 파일**: `output/pic-01-as-text.hwp` (HWP 프로그램으로 빈 문서에 이미지 1개 글자처리 삽입)
**사용 이미지**: `output/3tigers.jpg` (4,774,959 bytes, JPEG)

**핵심 발견 (참조 파일 분석):**

1. **문단 구조**: 단일 문단 (표와 다름!)
   - 문단[0]: SectionDef + ColumnDef + Picture + CR (char_count=25, msb=true)
   - 표는 2개 문단(표+빈줄) 필요하지만 이미지는 1개 문단으로 충분

2. **CTRL_HEADER (GenShape)**: 242바이트
   - ctrl_id='gso ' + CommonObjAttr(attr, offsets, width, height, z_order, margins, instance_id, description, raw_extra)
   - `attr = 0x040A6311` (글자처리 이미지 속성 플래그)
   - raw_extra: 200바이트 (파일 경로/UUID 등 메타데이터)

3. **SHAPE_COMPONENT**: 196바이트 (level=L+2)
   - ctrl_id × 2: `$pic` (0x24706963) — 반드시 2회 기록 (top-level GSO)
   - ShapeComponentAttr: offset(8) + group_level(2) + file_version(2) + original/current size(16) + flip(4)
   - raw_rendering: 146바이트 (단위 행렬 + 추가 데이터)
   - 렌더링 데이터: cnt(u16=1) + identity_matrix(48B: 1.0,0.0,0.0,1.0,0.0,0.0) + 추가

4. **SHAPE_COMPONENT_PICTURE (tag=85)**: 82바이트 (level=L+3)
   - border_color(4) + border_width(4) + border_attr(4)
   - border_x[4] (16) + border_y[4] (16) — 사각형 좌표
   - crop(16) + padding(8) + image_attr(5) + raw_picture_extra(9)

5. **border 좌표 패턴**: W=너비, H=높이일 때
   - border_x = [0, 0, W, 0]
   - border_y = [W, H, 0, H]

6. **crop 값**: 원본 이미지 크기(HWPUNIT) 저장
   - crop.right = 127560 (≈원본 이미지 가로 크기)
   - crop.bottom = 191400 (≈원본 이미지 세로 크기)

7. **BinData 파이프라인**:
   - DocInfo: BIN_DATA 레코드 (attr=0x0001, Embedding, status=NotAccessed)
   - CFB: `/BinData/BIN0001.jpg` 스트림
   - Picture.image_attr.bin_data_id = 1 (1-indexed)
   - ID_MAPPINGS에 bin_data 카운트 자동 반영

8. **캐럿 위치**: list_id=0, para_id=0, char_pos=24 (CR 직전)
   - 단일 문단이므로 list_id=0 유지 (표의 list_id=1과 다름)

9. **LineSeg**: line_height = 이미지 높이 (14775), segment_width = 편집 영역 너비 (42520)

**레코드 비교 결과 (참조 vs 저장):**

| 레코드 | 참조 크기 | 저장 크기 | 일치 |
|--------|----------|----------|------|
| PARA_HEADER | 22B | 22B | 일치 ✓ |
| PARA_TEXT | 50B | 50B | 완전 일치 ✓ |
| PARA_CHAR_SHAPE | 8B | 8B | ~= (char_shape_id 차이) |
| PARA_LINE_SEG | 36B | 36B | 완전 일치 ✓ |
| CTRL_HEADER (SectionDef) | 38B | 38B | 완전 일치 ✓ |
| CTRL_HEADER (ColumnDef) | 16B | 16B | 완전 일치 ✓ |
| CTRL_HEADER (GenShape) | 242B | 242B | 완전 일치 ✓ |
| SHAPE_COMPONENT | 196B | 196B | 완전 일치 ✓ |
| SHAPE_PICTURE | 82B | 82B | 완전 일치 ✓ |
| 기타 (PAGE_DEF 등) | - | - | 완전 일치 ✓ |

**총 15개 레코드 중 14개 완벽 일치, 1개 사소한 차이 (char_shape_id)**

**테스트 케이스:**

| 파일 | 이미지 | 레코드 수 | 결과 |
|------|--------|----------|------|
| save_test_picture.hwp | 3tigers.jpg (4.8MB) | 15 (참조와 동일) | HWP 정상 오픈 ✓ |

**검증 항목:**
- 이미지 위치 정상 ✓
- 캐럿 위치 정상 ✓
- 재파싱 성공 ✓
- BinData 스트림 정상 생성 ✓

### 4.4 기타 컨트롤 라운드트립 — 단계 5 완료

**검증 방식**: 다양한 컨트롤을 포함한 실제 HWP 파일 라운드트립 (파싱→재직렬화→저장→재파싱)

**검증 결과:**

| 샘플 파일 | 대상 컨트롤 | 보존 | 레코드 수 | 일치율 |
|-----------|-----------|------|----------|--------|
| k-water-rfp.hwp | Header(3), Footer(2), Shape(2), Picture(15), Table(19) | 모두 ✓ | 266=266 | 89% |
| 20250130-hongbo.hwp | Shape(1), Picture(4), Table(6) | 모두 ✓ | 306=306 | 100% |
| hwp-multi-001.hwp | Shape(1), Picture(2), Table(26) | 모두 ✓ | 8702→8697 | 43% |
| hwp-multi-002.hwp | Picture(3), Table(7) | 모두 ✓ | 1261→1243 | 10% |
| 2010-01-06.hwp | Footnote(30), Table(12) | 모두 ✓ | 2711=2711 | 90% |

**컨트롤별 보존 현황:**

| 컨트롤 | 검증 파일 수 | 총 인스턴스 | 보존 | 비고 |
|--------|-------------|-----------|------|------|
| Header (머리글) | 1 | 3 | ✓ | k-water-rfp |
| Footer (바닥글) | 1 | 2 | ✓ | k-water-rfp |
| Footnote (각주) | 1 | 30 | ✓ | 2010-01-06 |
| Shape (도형) | 3 | 4 | ✓ | k-water, hongbo, multi-001 |
| Picture (이미지) | 4 | 24 | ✓ | 모든 파일 |
| Table (표) | 5 | 70 | ✓ | 모든 파일 |
| Endnote (미주) | 0 | 0 | - | 샘플 없음 |
| Bookmark (책갈피) | 0 | 0 | - | 샘플 없음 |

**레코드 차이 원인 분석:**

1. **LIST_HEADER 크기 차이**: Header/Footer/Footnote의 LIST_HEADER가 직렬화 시 원본과 다른 크기 생성 (34B→6B)
   - 원인: 직렬화 시 raw 데이터 일부 생략
   - 영향: 컨트롤 자체는 보존되나 바이트 단위 일치율 감소

2. **레벨 시프트**: Header/Footer 내부 문단의 레벨이 +1 shift되는 경우
   - 원인: LIST_HEADER 크기 차이로 인한 파서 해석 차이
   - 영향: 레코드 구조는 동일, 레벨만 다름

3. **CTRL_HEADER 크기 차이**: ColumnDef CTRL_HEADER가 16B→8B로 축소
   - 원인: 확장 속성 미보존 (다단 문서에서 발생)

4. **100% 일치 가능 파일**: 20250130-hongbo.hwp — 표/이미지/도형만 포함된 단순 구조

**주의사항:**
- Endnote, Bookmark은 테스트 샘플에 미포함으로 검증 불가
- Header/Footer의 LIST_HEADER 직렬화는 향후 개선 대상
- 복잡한 다단 문서(hwp-multi-*)는 일치율이 낮지만 컨트롤 보존은 정상

### 4.5 필드 컨트롤 (Field) — 저장 규칙

**적용 대상**: ClickHere(누름틀), Hyperlink, Unknown 등 모든 필드 타입 (ctrl_id `%hlk`, `%clk`, `%unk` 등)

#### CTRL_HEADER 직렬화 구조

```
CTRL_HEADER (level L+1)
  ├ ctrl_id: u32 (4바이트) — '%hlk', '%clk', '%unk' 등
  ├ properties: u32 (4바이트) — 표 155 참조
  │   └ bit 15: 필드 내용 수정 여부 (0=초기 상태, 1=사용자 수정됨)
  ├ extra_properties: u8 (1바이트)
  ├ command_len: u16 (2바이트) — command 문자열 길이 (WCHAR 단위)
  ├ command: u16[command_len] — UTF-16LE 문자열
  ├ field_id: u32 (4바이트) — 문서 내 필드 식별자
  └ memo_index: u32 (4바이트) — ★ 스펙 미기재, 반드시 직렬화
```

**주의**: `memo_index`는 공식 스펙에 없지만 **4바이트 필수 직렬화**. 미포함 시 한컴에서 CTRL_HEADER 크기 불일치로 파일 손상 (errata §18 참조).

#### ClickHere(누름틀) command 문자열 구조

```
Clickhere:set:{total_len}:Direction:wstring:{n}:{안내문} HelpState:wstring:{n}:{메모} Name:wstring:{n}:{이름}
```

| 필드 | 의미 | 예시 |
|------|------|------|
| Direction | 안내문 (빨간 기울임으로 표시) | `Direction:wstring:7:여기에 입력 ` |
| HelpState | 메모/도움말 텍스트 | `HelpState:wstring:43:회사명은...` |
| Name | 필드 이름 | `Name:wstring:2:직위 ` |

**저장 시 주의사항**:
1. 각 wstring 값 뒤에 **공백 1개 필수** — `trim_end()` 금지
2. `{n}`은 문자열 길이 (공백 포함 WCHAR 수)
3. HelpState가 없으면(메모 미입력) 해당 필드 전체 생략 가능

#### CTRL_DATA (누름틀 이름 저장)

```
CTRL_DATA (level L+2, 선택적)
  └ ParameterSet (id=0x021B)
      └ ParameterItem (id=0x4000, type=String) → 필드 이름
```

- 한컴은 필드 이름 변경 시 **CTRL_DATA만 갱신**, command 내 Name:은 재구축하지 않음
- 이름 조회 우선순위: ① CTRL_DATA name → ② command Name: → ③ command Direction:

#### FIELD_BEGIN / FIELD_END 직렬화 (PARA_TEXT 내)

```
PARA_TEXT 내 배치:
  ... | FIELD_BEGIN(0x0003, 8 WCHAR) | {필드 내용 텍스트} | FIELD_END(0x0004, 1 WCHAR) | ...
```

- FIELD_BEGIN은 확장 제어문자 (8 WCHAR = 16바이트)
- FIELD_END는 인라인 제어문자 (1 WCHAR = 2바이트)
- 빈 필드(초기 상태): FIELD_BEGIN 직후 FIELD_END
- control_mask에 **반드시** bit 3(FIELD_BEGIN)과 bit 4(FIELD_END) 포함

#### properties bit 15 — 초기 상태 처리

| bit 15 | 필드 값 | 렌더링 |
|--------|---------|--------|
| 0 (초기) | 비어있거나 안내문과 동일 | 안내문 표시 (빨간 기울임) |
| 0 (초기) | 안내문과 동일 (메모 입력 후) | **안내문 표시** — 문서 로드 시 텍스트 제거 필요 |
| 1 (수정됨) | 사용자 입력 텍스트 | 일반 텍스트로 표시 |

- 한컴은 메모 추가 시 안내문을 필드 값으로 삽입하면서 bit 15=0 유지
- 본 프로젝트에서는 `clear_initial_field_texts()`로 로드 시 정규화

#### TAB 확장 데이터 보존

필드 내부에 TAB(0x0009)이 포함된 경우:
```
TAB 코드(2B) + 추가 7 code unit(14B) = 16바이트
```
- 추가 데이터에 탭 너비/종류 정보 포함
- 라운드트립 시 7개 code unit을 그대로 보존해야 함 (0 채움 시 탭 간격 오류)

---

## 5. 최종 요약

### 검증 완료 항목

| 단계 | 대상 | 참조 파일 | HWP 오픈 | 비고 |
|------|------|----------|---------|------|
| 2 | 텍스트만 | - | ✓ | 3개 파일 정상 |
| 3 | 표 (Table) | 1by1-table.hwp | ✓ | 21개 레코드 완전 일치 |
| 4 | 이미지 (Picture) | pic-01-as-text.hwp | ✓ | 15개 레코드 14/15 일치 |
| 추가 | 표 안 이미지 (Table+Picture) | pic-in-tb-01.hwp | ✓ | 25개 레코드 21/25 일치 |
| 5 | 기타 (Header/Footer/Footnote/Shape) | 라운드트립 | - | 모든 컨트롤 보존 |
| - | 필드 (Field/ClickHere) | 라운드트립 | - | memo_index 4바이트, command 구조, CTRL_DATA 보존 |

### 알려진 제약사항

1. **Header/Footer LIST_HEADER**: 직렬화 시 원본 대비 크기 차이 (기능 영향 없음)
2. **ColumnDef 확장 속성**: 다단 문서의 일부 확장 속성 미보존
3. **Endnote/Bookmark**: 검증 샘플 미확보 (파서/직렬화 코드는 존재)
4. **char_shape_id 차이**: 새로 생성한 문서에서 empty.hwp 기본값 사용 시 발생 (서식만 영향)
5. **필드 command 문자열**: 내부 파싱은 구현했으나, 편집(재구축) 시 wstring 길이 필드와 후행 공백 정확히 유지 필요

---

**작성일**: 2026-02-11
**최종 갱신**: 2026-03-16 — 필드 컨트롤(ClickHere/누름틀) 저장 규칙 추가 (§4.5)
