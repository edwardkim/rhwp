---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 표 붙여넣기 후 저장 파일 손상 분석

## 증상

웹 뷰어에서 HTML 표를 붙여넣기한 후 HWP로 저장하면 한컴오피스에서 "파일이 손상되었습니다" 오류가 발생한다.
표 붙여넣기 이전 내용까지만 렌더링되고, 붙여넣기된 표 이후 내용은 표시되지 않는다.

## 테스트 파일

### 간단한 케이스 (step1)
| 파일 | 설명 | 상태 |
|------|------|------|
| `template/empty.hwp` | HWP 빈 문서 | 원본 |
| `template/empty-step1.hwp` | 2x2 테이블 수동 생성 | 원본 |
| `template/empty-step1-p.hwp` | HWP 프로그램 간 붙여넣기 | 정상 (참조용) |
| `template/empty-step1_saved.hwp` | 우리 뷰어 붙여넣기 | **손상** |
| `template/empty-step1_saved-a.hwp` | 손상 파일을 HWP에서 다른이름 저장 | 정상 |
| `template/empty-step1_saved_add_ccmsb.hwp` | msb 수정 후 뷰어 붙여넣기 | **정상** |

### 복잡한 케이스 (step2)
| 파일 | 설명 | 상태 |
|------|------|------|
| `template/empty-step2.hwp` | 원본 (2x2 테이블 포함) | 원본 |
| `template/empty-step2-p.hwp` | HWP 프로그램 간 복잡한 표 붙여넣기 | 정상 (참조용) |
| `template/empty-step2_saved_err.hwp` | 우리 뷰어 복잡한 표 붙여넣기 | **손상** |

### 실제 문서 케이스
| 파일 | 설명 | 상태 |
|------|------|------|
| `pasts/20250130-hongbo-p2.hwp` | 원본 | 원본 |
| `pasts/20250130-hongbo_saved-rp-006.hwp` | 뷰어 붙여넣기 | **손상** |
| `template/20250130-hongbo_saved_err.hwp` | 뷰어 붙여넣기 (복잡) | **손상** |
| `template/111111.hwp` | 손상 파일을 HWP에서 다른이름 저장 | 정상 (일부 누락) |

---

## 발견된 차이점 목록

### 이미 수정 완료된 항목

#### [FIX-1] char_count_msb 플래그 (간단한 케이스 해결 → 복잡한 케이스에서 재수정)

- **위치**: 표 컨트롤을 포함하는 문단 (PARA_HEADER)
- **증상**: `char_count` 의 비트 31 (MSB) 설정에 따라 동작이 달라짐
- **간단한 케이스 (빈 문서)**: MSB = 1 (true) 필요 → MSB=0이면 "파일 손상" 오류
- **복잡한 케이스 (기존 내용 있는 문서)**: MSB = 0 (false) 필요 → MSB=1이면 표 이후 내용이 사라짐
- **원인 분석**:
  - HWP 스펙: `if (nchars & 0x80000000) { nchars &= 0x7fffffff; }` — MSB를 마스킹하여 실제 글자 수를 구하는 코드만 제시, 의미는 미설명
  - 빈 문서에서는 MSB=1이 필수 (표 컨트롤이 유일한 콘텐츠)
  - 기존 내용이 있는 문서에서는 MSB=0이어야 표 이후 문단이 정상 렌더링됨
  - 실제 HWP 프로그램이 생성한 표 문단의 MSB 값으로 확인:
    - `template/empty-step1-p.hwp`: 빈 문서에 표 삽입 → MSB=1
    - `samples/20250130-hongbo.hwp`: 기존 표 문단 → MSB=0

- **근본 원인 규명** (`samples/k-water-rfp.hwp` 전수 조사):

  MSB는 **"현재 문단 리스트(스코프)의 마지막 문단"** 을 표시하는 종료 마커이다.

  | MSB 값 | 의미 |
  |--------|------|
  | 0 | 현재 스코프에 후속 문단이 더 있음 |
  | 1 | 현재 스코프의 마지막 문단 |

  검증 데이터:
  - Section 0: 57개 문단 → idx 0~55 전부 MSB=0, idx 56(마지막)만 MSB=1
  - Section 1: 265개 문단 → idx 0~263 전부 MSB=0, idx 264(마지막)만 MSB=1
  - 셀 내 복수 문단: 마지막 문단만 MSB=1 (예: cell[11]의 26개 문단 중 p[25]만 MSB=1)
  - **문단 스타일(ParaShape)이 바뀌어도 MSB에는 영향 없음** — 오직 위치로만 결정

  이 규칙은 섹션, 셀, 텍스트 박스, 머리글/꼬리글 등 모든 문단 리스트에 동일하게 적용된다.

  따라서 빈 문서 케이스에서 MSB=1이 필요했던 이유: 표 문단이 섹션의 유일한(=마지막) 문단이었기 때문이다. 기존 문서에서 MSB=0이 필요한 이유: 표 문단 뒤에 더 많은 문단이 있기 때문이다.

- **최종 수정**: `parse_table_html()` 에서 `char_count_msb: false` 설정
  - 셀 내부 문단은 `char_count_msb: true` 유지 (셀 보정 코드에서 설정, 셀당 문단이 1개이므로 항상 마지막)
  - 표 문단 자체(바깥 컨테이너)는 `false` (삽입 위치가 마지막이 아닌 한)
  - 향후 개선: 삽입 위치에 따라 MSB를 동적으로 설정해야 함

```
[빈 문서 케이스]
step1_saved (손상): rec[34] PARA_HEADER cc=9 msb=0  ← 마지막 문단인데 MSB=0 → 손상
step1_saved-a (HWP수정): rec[34] PARA_HEADER cc=9 msb=1  ← 마지막 문단 MSB=1 → 정상

[기존 문서 케이스 - 바이트 비교]
복제 표 (정상): PARA_HEADER [09, 00, 00, 00, ...] → cc=9, MSB=0  ← 중간 문단이므로 MSB=0 → 정상
생성 표 (실패): PARA_HEADER [09, 00, 00, 80, ...] → cc=9, MSB=1  ← 중간 문단인데 MSB=1 → 이후 내용 사라짐
생성 표 (수정): PARA_HEADER [09, 00, 00, 00, ...] → cc=9, MSB=0  ← 중간 문단 MSB=0 → 정상

[k-water-rfp.hwp 전수 조사]
Section 0 (57 paras): MSB_T=1 MSB_F=56  → 마지막만 MSB=1
Section 1 (265 paras): MSB_T=1 MSB_F=264 → 마지막만 MSB=1
cell[42] (3 paras): p[0] MSB=F, p[1] MSB=F, p[2] MSB=T  → 마지막만 MSB=1
cell[11] (26 paras): p[0..24] MSB=F, p[25] MSB=T  → 마지막만 MSB=1
```

#### [FIX-2] DIFF-1~8 일괄 수정 (타스크 41)

수정된 DIFF 항목:

| 항목 | 수정 내용 |
|------|----------|
| DIFF-1 빈 셀 공백 | `html_to_plain_text()` 에서 `&nbsp;` → 빈 셀로 처리 (cc=1, PARA_TEXT 없음) |
| DIFF-2 CharShape ID | 빈 char_shapes에 기본 CharShapeRef(id=0) 추가 |
| DIFF-3 ParaShape ID | 기본 "본문" ParaShape(id=0) 사용 |
| DIFF-4 BorderFill ID | `create_border_fill_from_css()` + surgical insert로 올바른 1-based ID |
| DIFF-5 TABLE attr | `raw_table_record_attr = 0x04000006` (bit 1 셀분리금지 항상 설정) |
| DIFF-6 LineSeg 메트릭 | tag=0x00060000, seg_width=셀폭-좌우패딩 |
| DIFF-7 인스턴스 ID | 해시 기반 비-0 instance_id 생성 |
| DIFF-8 컨테이너 LineSeg | total_height, total_width 기반 |

#### [FIX-3] 엔터 2회 후 저장 시 파일 손상 (타스크 42)

- **증상**: 기존 HWP를 웹 뷰어에서 열고, 문단 중간에서 엔터를 2회 연속 입력한 후 저장하면 HWP 프로그램에서 "파일이 손상되었습니다" 오류 발생
- **재현 조건**: 엔터 1회 → 저장 = 정상, 엔터 2회 → 저장 = 손상
- **원인**: `split_at(0)` 호출 시 텍스트가 비어있는 문단(cc=1)이 생성되는데, `has_para_text=true`가 유지되어 직렬화 시 불필요한 PARA_TEXT 레코드([0x000D] 1 code unit)가 포함됨
  - 원본 HWP 파일의 빈 문단(cc=1)은 PARA_TEXT가 **없음** (`has_para_text=false`)
  - HWP 프로그램은 cc=1 문단에 PARA_TEXT가 있으면 레코드 구조 불일치로 판단하여 파일 손상 오류 발생
  - 우리 뷰어 파서는 더 관대하여 재파싱은 성공하지만, HWP 프로그램에서만 오류 발생

```
[수정 전 - 분할 생성 빈 문단]
PARA_HEADER: cc=1
PARA_TEXT: 1 code_unit (0x000D)   ← HWP 프로그램이 거부
PARA_CHAR_SHAPE
PARA_LINE_SEG

[수정 후 - 원본 HWP와 동일]
PARA_HEADER: cc=1
PARA_CHAR_SHAPE                   ← PARA_TEXT 없음
PARA_LINE_SEG
```

- **수정**: `split_at()` 후 텍스트가 비어있고 컨트롤이 없으면 `has_para_text = false`로 설정
- **수정 파일**: `src/model/paragraph.rs`
- **검증**: 474개 테스트 통과, HWP 프로그램에서 정상 오픈 확인

#### [FIX-4] DocInfo 재직렬화 버그 발견 (별도 이슈)

- **증상**: `doc_info.raw_stream = None` 설정 시 복잡한 문서에서 "파일 손상" 오류
- **원인**: DocInfo 재직렬화 코드가 일부 레코드를 완전히 재현하지 못함
- **워크어라운드**: DocInfo raw_stream 유지, Section만 재직렬화
- **검증**:
  - `save_test_section_only.hwp` (Section만 재직렬화) → 정상
  - `save_test_docinfo_only.hwp` (DocInfo만 재직렬화) → 파일 손상
  - `save_test_roundtrip.hwp` (전체 재직렬화) → 파일 손상
- **상태**: 워크어라운드 적용 중 (surgical insert로 DocInfo 수정 시에만 raw_stream 부분 변경)

#### [FIX-5] 텍스트 편집 후 저장 시 DocInfo raw_stream 무효화로 파일 손상

- **증상**: rhwp-studio에서 HWP 문서를 열고 텍스트를 추가/삭제한 후 Ctrl+S로 저장하면 한컴오피스에서 "파일이 손상되었습니다" 오류 발생
- **재현 조건**: 문서 열기 → 텍스트 입력 또는 삭제 → 저장 = 손상. 열고 그대로 저장 = 정상
- **원인**: `insert_text_native()`, `delete_text_native()`, `delete_range_native()`에서 캐럿 위치 업데이트를 위해 `doc_info.raw_stream = None`을 설정하여 FIX-4의 DocInfo 재직렬화 버그가 발동
  - 캐럿 위치(caret_list_id, caret_para_id, caret_char_pos)는 DocInfo의 DOCUMENT_PROPERTIES 레코드 offset 14~25에 위치 (각 u32, 12바이트)
  - `raw_stream = None` 설정 시 전체 DocInfo가 재직렬화되어 불완전한 레코드가 생성됨
- **수정**: `surgical_update_caret()` 함수 구현 (src/serializer/doc_info.rs)
  - DocInfo raw_stream 내 DOCUMENT_PROPERTIES 레코드를 `scan_records()`로 검색
  - 해당 레코드의 offset 14~25 (12바이트)만 직접 수정하여 캐럿 위치 업데이트
  - raw_stream 전체를 유지하므로 재직렬화가 발생하지 않음
- **수정 파일**: `src/serializer/doc_info.rs`, `src/wasm_api.rs` (3곳)
- **검증**: 488개 테스트 통과, 텍스트 추가 후 저장 정상, 표 붙여넣기 후 저장 정상

---

### 미수정 항목 (복잡한 케이스)

step2 비교 기준:
- **VALID** = `template/empty-step2-p.hwp` (HWP 프로그램 간 붙여넣기, 정상)
- **DAMAGED** = `template/empty-step2_saved_err.hwp` (우리 뷰어 붙여넣기, 손상)

#### [DIFF-1] 빈 셀에 공백 문자 삽입 (레코드 구조 왜곡)

- **위치**: 표 셀 내부 빈 문단 (PARA_HEADER + PARA_TEXT)
- **심각도**: **높음** — 추가 레코드가 이후 전체 레코드를 밀어냄
- **증상**:
  - VALID: 빈 셀 → `char_count=1` (줄바꿈만), PARA_TEXT 레코드 없음
  - DAMAGED: 빈 셀 → `char_count=6` (공백5 + 줄바꿈), PARA_TEXT 레코드 추가됨

```
VALID  rec[46]: PARA_HEADER cc=1 → PARA_CHAR_SHAPE → PARA_LINE_SEG
DAMAGED rec[46]: PARA_HEADER cc=6 → PARA_TEXT("     \r") → PARA_CHAR_SHAPE → PARA_LINE_SEG
```

- **원인**: HTML 셀의 `&nbsp;` 가 공백으로 변환된 후 빈 셀로 인식되지 않음
  - `decode_html_entities()` 에서 `&nbsp;` → 스페이스 변환
  - `pc.content_html.trim().is_empty()` 가 `false` 로 평가 → `parse_html_to_paragraphs()` 호출
  - 결과적으로 빈 셀에 `"     "` 텍스트가 포함됨
- **코드 위치**: `wasm_api.rs:3870`

#### [DIFF-2] CharShape ID 손실 (CS=0 획일화)

- **위치**: 표 셀 내부 문단의 PARA_CHAR_SHAPE 레코드
- **심각도**: **중간** — 문자 서식 정보 손실, 손상 원인 가능성
- **증상**:
  - VALID: 셀마다 다양한 CharShape ID (CS5, CS6, CS7, CS8, ... CS19)
  - DAMAGED: 모든 셀이 `CS_id=0` 으로 통일됨

```
VALID DocInfo:  CS=20 (원본5 + 붙여넣기15)
DAMAGED DocInfo: CS=8  (원본5 + 붙여넣기3)
```

- **원인**: HTML에서 CSS 스타일을 파싱하여 CharShape를 생성하지만, 네이티브 HWP 붙여넣기가 원본 문서의 CharShape를 그대로 복사하는 것과 달리 우리 코드는 CSS 기반으로 재생성하므로 세밀한 서식 차이가 반영되지 않음
- **코드 위치**: `parse_html_to_paragraphs()` → `parse_inline_content()` 에서 CharShape 할당

#### [DIFF-3] ParaShape ID 오프셋 (ps=12 vs ps=13)

- **위치**: 표 셀 내부 모든 PARA_HEADER의 `para_shape_id` 필드
- **심각도**: **낮음~중간** — 잘못된 문단 서식 참조
- **증상**:
  - VALID: `para_shape_id = 13` (새로 추가된 ParaShape)
  - DAMAGED: `para_shape_id = 12` (기존 ParaShape)

- **원인**: `cell_para_shape_id` 결정 로직이 기존 문서의 첫 번째 표 셀의 ParaShape를 재사용하는데, 정상 파일에서는 붙여넣기용 새 ParaShape(ID=13)를 생성함
- **코드 위치**: `wasm_api.rs:3882-3904`

#### [DIFF-4] BorderFill ID 오프셋 (전체적으로 1 부족)

- **위치**: 모든 셀의 LIST_HEADER `border_fill_id` 필드
- **심각도**: **중간** — 잘못된 테두리/배경 참조
- **증상**:
  - VALID: `borderFillId = 4, 5, 9, 6, 7, 8, ...`
  - DAMAGED: `borderFillId = 3, 4, 5, 6, 7, ...` (1씩 부족)

```
VALID DocInfo:  BF=17
DAMAGED DocInfo: BF=15
```

- **원인**: `create_border_fill_from_css()` 에서 BorderFill을 생성할 때 ID 할당 방식이 다를 수 있음. HWP 프로그램은 원본 문서의 BorderFill을 그대로 복사하고, 우리 코드는 CSS에서 재생성
- **코드 위치**: `wasm_api.rs` `create_border_fill_from_css()`

#### [DIFF-5] TABLE 레코드 attr 플래그 차이

- **위치**: HWPTAG_TABLE 레코드의 첫 번째 u32 attr 필드
- **심각도**: **낮음~중간**
- **증상**:
  - VALID: `attr = 0x04000006` (bit 1,2 set: 셀 분리 금지 + repeat_header)
  - DAMAGED: `attr = 0x04000004` (bit 2 only: repeat_header만)

```
VALID:   low bits = 110 (bit1=셀분리금지, bit2=repeat_header)
DAMAGED: low bits = 100 (bit2=repeat_header만)
```

- **원인**: `raw_table_record_attr` 생성 시 bit 1 (셀 분리 금지) 이 설정되지 않음
- **코드 위치**: `wasm_api.rs` 표 생성 부분

#### [DIFF-6] PARA_LINE_SEG 메트릭 0 초기화

- **위치**: 모든 셀 문단의 PARA_LINE_SEG 레코드
- **심각도**: **낮음** — 레이아웃 정보 손실 (렌더링 영향)
- **증상**:
  - VALID: `segWidth=적절값, flags=0x00060000`
  - DAMAGED: `segWidth=0, flags=0x00000000`

- **원인**: 셀 문단 생성 시 LineSeg의 `seg_width`와 `flags`가 기본값(0)으로 설정됨. 정상 파일에서는 `flags=0x00060000` (bit 17,18 = 라인 타입 플래그)
- **코드 위치**: `wasm_api.rs:3931-3959` 셀 문단 보정 코드

#### [DIFF-7] CTRL_HEADER 인스턴스 ID 0 초기화

- **위치**: 표 컨트롤의 CTRL_HEADER 레코드 (offset 36-39)
- **심각도**: **낮음**
- **증상**:
  - VALID: `instance_id = 0x7c154b69` (유니크한 값)
  - DAMAGED: `instance_id = 0x00000000` (0으로 초기화)

- **원인**: `raw_ctrl_data`에 instance_id 필드가 포함되어야 하지만 0으로 채워짐
- **코드 위치**: 표 컨트롤의 `raw_ctrl_data` 생성 부분

#### [DIFF-8] 표 컨테이너 PARA_LINE_SEG 부정확

- **위치**: 표 컨트롤을 포함하는 문단 (rec[37]) 의 PARA_LINE_SEG
- **심각도**: **낮음**
- **증상**:
  - VALID: `yPos=3130, height=26990, textHeight=26990, baseline=22942, segWidth=42520, flags=0x00060000`
  - DAMAGED: `yPos=0, height=400, textHeight=400, baseline=320, segWidth=0, flags=0x00000000`

- **원인**: 표 문단의 LineSeg 높이가 표 전체 높이를 반영해야 하지만, 기본 폰트 크기(400) 기반으로 설정됨
- **코드 위치**: `wasm_api.rs:4131-4137` 표 문단 LineSeg 생성

#### [DIFF-9] 첫 번째 표 (원본) 의 BorderFill ID 차이

- **위치**: 원본 2x2 테이블의 CTRL_HEADER 및 셀 LIST_HEADER
- **심각도**: **낮음** — 원본 표 데이터가 재직렬화 시 달라짐
- **증상**:
  - VALID: `borderFillId = 3`
  - DAMAGED: `borderFillId = 2`

- **원인**: `section.raw_stream = None` 설정으로 전체 섹션이 재직렬화될 때, 원본 표의 BorderFill 참조가 DocInfo에 새로 추가된 BorderFill 인덱스 오프셋의 영향을 받는 것으로 추정

---

## 레코드 구조 순서 비교

### HWP 표 컨트롤 기본 구조 (정상 파일 기준)

```
L0: PARA_HEADER (표 포함 문단, cc=9, msb=1, cm=0x800)
L1:   PARA_TEXT (확장제어문자 + 종료마커)
L1:   PARA_CHAR_SHAPE
L1:   PARA_LINE_SEG (표 전체 높이 반영)
L1:   CTRL_HEADER (tbl)
L2:     TABLE (attr, row_count, col_count, spacing, padding, row_sizes, borderFillId)
L2:     LIST_HEADER (cell[0]: n_para, list_attr, col, row, colspan, rowspan, w, h, padding, borderFillId)
L2:     PARA_HEADER (cell[0] para[0], cc, msb=1)
L3:       PARA_TEXT (셀 텍스트)      ← 빈 셀이면 이 레코드 없음!
L3:       PARA_CHAR_SHAPE
L3:       PARA_LINE_SEG
L2:     LIST_HEADER (cell[1])
L2:     PARA_HEADER (cell[1] para[0])
L3:       ...
```

### 우리 코드의 직렬화 순서

동일한 순서로 생성됨. **순서 자체는 정상.**

```
serialize_table() → CTRL_HEADER(tbl) [level]
                   → TABLE [level+1]
                   → serialize_cell() [level+1]
                     → LIST_HEADER [level+1]
                     → serialize_paragraph_list() [level+1]
                       → PARA_HEADER [level+1]
                         → PARA_TEXT [level+2]    ← 빈 셀에도 생성됨 (DIFF-1)
                         → PARA_CHAR_SHAPE [level+2]
                         → PARA_LINE_SEG [level+2]
```

**핵심 구조적 차이**: 빈 셀에 불필요한 PARA_TEXT 레코드가 추가되어 이후 모든 레코드가 밀림

---

## 우선순위 판단

| 순위 | 항목 | 이유 |
|------|------|------|
| 1 | DIFF-1 빈 셀 공백 | 레코드 구조를 왜곡하여 파서 오류 유발 가능성 높음 |
| 2 | DIFF-2 CharShape ID | ID 범위 밖 참조 시 직접 손상 원인 |
| 3 | DIFF-4 BorderFill ID | ID 범위 밖 참조 시 직접 손상 원인 |
| 4 | DIFF-5 TABLE attr | 표 속성 플래그 차이 |
| 5 | DIFF-3 ParaShape ID | 문단 서식 참조 오류 |
| 6 | DIFF-6 LINE_SEG 메트릭 | 레이아웃 품질 (직접 손상 원인은 아닐 수 있음) |
| 7 | DIFF-7 인스턴스 ID | 품질 (직접 손상 원인은 아닐 수 있음) |
| 8 | DIFF-8 표 컨테이너 LINE_SEG | 레이아웃 품질 |
| 9 | DIFF-9 원본 표 BF ID | 재직렬화 부작용 |

---

## 관련 코드 위치

| 파일 | 라인 | 설명 |
|------|------|------|
| `src/wasm_api.rs` | 3870 | 셀 내용 파싱 (빈 셀 판단) |
| `src/wasm_api.rs` | 3907-3960 | 셀 문단 보정 코드 |
| `src/wasm_api.rs` | 4122-4145 | 표 문단 생성 |
| `src/wasm_api.rs` | 4648-4676 | HTML 엔티티 디코딩, html_to_plain_text |
| `src/serializer/control.rs` | 305-332 | serialize_table (레코드 순서) |
| `src/serializer/control.rs` | 377-421 | serialize_cell |
| `src/serializer/body_text.rs` | 50-111 | serialize_paragraph (PARA_TEXT 생성 조건) |

---

## 테스트 코드

| 테스트명 | 설명 |
|----------|------|
| `test_template_comparison` | step1 파일들 레코드 덤프 비교 |
| `test_step2_comparison` | step2 파일들 DocInfo + BodyText 비교 |
| `test_step2_paste_area` | step2 붙여넣기 영역 바이트 레벨 비교 |
| `test_complex_comparison` | 복잡한 파일 (hongbo) 비교 |
| `test_rp006_dangling_references` | rp-006 CharShape/ParaShape 참조 범위 검증 |
