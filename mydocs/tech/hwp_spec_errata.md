# HWP 5.0 스펙 문서 정오표

공식 스펙 문서([hwp_spec_5.0.md](hwp_spec_5.0.md))와 실제 바이너리 구현 간 불일치 사항을 기록한다.
각 항목은 역공학 검증을 통해 확인된 것이며, 본 프로젝트의 파서/렌더러는 실제 바이너리 기준으로 구현한다.

> **새 기능 구현 전 반드시 본 문서를 확인할 것.** 스펙을 그대로 따르면 파일 손상이 발생할 수 있다.

---

## 1. BorderFill 직렬화 순서

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 28 (테두리/배경) |
| 스펙 기술 | 순차 배열: `line_type[4]`, `width[4]`, `color[4]` |
| 실제 구현 | 인터리브: `(line_type + width + color) × 4` (좌, 우, 상, 하) |
| 검증 방법 | `eprintln!`으로 border color 덤프 → `#000000` 정확히 나와야 함. 순차 배열로 읽으면 `#010100` 등 오정렬 발생 |
| 수정 파일 | `src/parser/doc_info.rs` |
| 발견일 | 2026-02-05 |

---

## 2. BorderLineType 열거값

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 28 (테두리 선 종류) |
| 스펙 기술 | 0부터 시작하여 0=Solid 암시 |
| 실제 구현 | 0=None(선 없음), 1=Solid, 2=Dash, 3=Dot, 4=DashDot, 5=DashDotDot, 6=LongDash, 7=Circle, 8=Double, 9=ThinThickDouble, 10=ThickThinDouble, 11=ThinThickThinTriple, 12=Wave, ... |
| 검증 방법 | `eprintln!`으로 line_type 값 덤프, 실제 HWP 파일에서 "선 없음" 테두리가 0인지 확인 |
| 수정 파일 | `src/parser/doc_info.rs`, `src/model/style.rs` |
| 발견일 | 2026-02-05 |

---

## 3. LIST_HEADER 속성 비트 위치

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 67 (문단 리스트 헤더) |
| 스펙 기술 | UINT32 속성 필드에서 bit 0\~2=텍스트 방향, bit 3\~4=줄바꿈, bit 5\~6=세로 정렬 |
| 실제 구현 | 비트 위치가 **상위 16비트**에 존재: bit 16\~18=텍스트 방향, bit 19\~20=줄바꿈, bit 21\~22=세로 정렬 |
| 검증 방법 | `list_attr=0x00200000` (bit 21 설정) → 세로정렬=Center(1). 스펙대로 bit 5\~6을 읽으면 항상 0(Top)이 됨 |
| 추출 공식 | `text_direction = (list_attr >> 16) & 0x07`, `vertical_align = (list_attr >> 21) & 0x03` |
| 수정 파일 | `src/parser/control.rs` |
| 발견일 | 2026-02-06 |

---

## 4. FootnoteShape 레코드 크기

| 항목 | 내용 |
|------|------|
| 스펙 위치 | HWPTAG_FOOTNOTE_SHAPE |
| 스펙 기술 | 26바이트 |
| 실제 구현 | 28바이트. `note_spacing`과 `separator_line_type` 사이에 미문서화된 2바이트 필드 존재 |
| 검증 방법 | 레코드 데이터 길이 확인 및 `separator_color` 값 검증 (2바이트 건너뛰지 않으면 색상값 오정렬) |
| 수정 파일 | `src/parser/body_text.rs` |
| 발견일 | 2026-02-06 |

---

## 5. PARA_HEADER char_count MSB — 문단 리스트 종료 마커

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 60 (문단 헤더) |
| 스펙 기술 | `if (nchars & 0x80000000) { nchars &= 0x7fffffff; }` — MSB 마스킹으로 실제 글자 수를 구한다 (의미 미설명) |
| 실제 의미 | **MSB = 현재 문단 리스트(스코프)의 마지막 문단**. MSB=1이면 해당 문단이 섹션/셀/텍스트박스 등 현재 스코프의 마지막 문단임을 표시 |
| 검증 데이터 | k-water-rfp.hwp Section0(57문단): idx 0~55 MSB=0, idx 56만 MSB=1. 셀 내 26개 문단: p[0..24] MSB=0, p[25]만 MSB=1 |
| 위반 시 결과 | 마지막이 아닌 문단에 MSB=1 → 이후 문단이 사라짐. 마지막 문단에 MSB=0 → "파일 손상" 오류 |
| 수정 파일 | `src/wasm_api.rs`, `src/serializer/body_text.rs` |
| 참조 문서 | `troubleshootings/table_paste_file_corruption.md` FIX-1 |
| 발견일 | 2026-01-20 |

---

## 6. 빈 문단(cc=1)의 PARA_TEXT 레코드 금지

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 60 (문단 헤더), 표 62 (문단 텍스트) |
| 스펙 기술 | PARA_TEXT 존재 조건에 대한 명시적 기술 없음 |
| 실제 규칙 | `char_count=1`(문단 끝 마커만 존재)이고 컨트롤이 없는 문단에서 PARA_TEXT 레코드가 존재하면 한컴은 "파일 손상" 판정 |
| 정상 구조 | cc=1 빈 문단: `PARA_HEADER → PARA_CHAR_SHAPE → PARA_LINE_SEG` (PARA_TEXT 없음) |
| 위반 구조 | cc=1 빈 문단: `PARA_HEADER → PARA_TEXT([0x000D]) → PARA_CHAR_SHAPE → PARA_LINE_SEG` → **파일 손상** |
| 수정 파일 | `src/model/paragraph.rs`, `src/serializer/body_text.rs` |
| 참조 문서 | `troubleshootings/table_paste_file_corruption.md` FIX-3, `troubleshootings/cell_split_save_corruption.md` FIX-2 |
| 발견일 | 2026-01-22 |

---

## 7. control_mask 필드 — 직렬화 시 재계산 필수

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 60 (문단 헤더) |
| 스펙 기술 | "컨트롤 마스크" (상세 미설명) |
| 실제 규칙 | `control_mask`는 문단의 `controls[]` 배열에 실제로 존재하는 컨트롤의 char_code 비트 OR. 편집 후 `controls[]`가 변경되면 `control_mask`도 재계산 필요 |
| 비트 매핑 | `0x04` = SectionDef/ColumnDef(코드2), `0x800` = Table/Shape/Picture(코드11), `0x10000` = Header/Footer(코드16) |
| 위반 시 결과 | `control_mask=0x800`인데 TABLE 레코드 부재 → 한컴 파서 혼란 → "파일 손상" |
| 수정 | 직렬화 시 `compute_control_mask(controls)` 재계산 → 모델값 무시 |
| 수정 파일 | `src/serializer/body_text.rs` |
| 참조 문서 | `troubleshootings/cell_split_save_corruption.md` FIX-1 |
| 발견일 | 2026-02-19 |

---

## 8. 채우기 투명도(alpha) 바이트 — 스펙 미기재

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 30 (채우기 정보) |
| 스펙 기술 | "추가 채우기 속성 길이(DWORD) + 추가 채우기 속성(BYTE[size])" 이후 **기술 없음** |
| 실제 구조 | 추가 속성 이후 fill_type 비트별로 1바이트씩 alpha 값이 존재 |
| 바이트 규칙 | bit0(단색)=1 → 1바이트 alpha, bit2(그라데이션)=1 → 1바이트 alpha, bit1(이미지)=1 → 1바이트 alpha |
| alpha 해석 | 0=미설정(불투명으로 처리), 1~254=반투명(opacity=alpha/255), 255=불투명 |
| 검증 데이터 | Worldcup_FIFA2010_32.hwp 도형: alpha=0xA3(163) → opacity=0.639 |
| HWPX 대응 | `<winBrush alpha="0.64">` 형태의 명시적 float |
| 부작용 | alpha 바이트 미소비 → 후속 필드(shadow info 등) 바이트 정렬 붕괴 |
| 수정 파일 | `src/parser/doc_info.rs`, `src/renderer/layout.rs`, `src/renderer/svg.rs`, `src/renderer/web_canvas.rs` |
| 참조 문서 | `troubleshootings/shape_fill_transparency.md` |
| 발견일 | 2026-02-17 |

---

## 9. 확장 제어문자 크기 — WCHAR 단위 (바이트 아님)

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 6 (제어 문자), 표 62 (문단 텍스트) |
| 스펙 기술 | 확장 제어문자 "크기 = 8" |
| 실제 크기 | 8 **WCHAR** = 8 × 2바이트 = **16바이트** |
| 구조 (16바이트) | `코드(2B) + 컨트롤 타입(4B, ASCII) + 추가 정보(8B) + 코드 반복(2B)` |
| 컨트롤 타입 예시 | `' lbt'` → 'tbl '(표), `' osg'` → 'gso '(그리기), `'deqe'` → 'eqed'(수식) — little-endian 역순 |
| 검증 방법 | hwplib `ForParaText.java`: `return 16;`, `byte[] addition = new byte[12];` |
| 수정 파일 | `hwp_semantic/record_parser.py` |
| 참조 문서 | `troubleshootings/task_56_hwp_control_char_bytes.md` |
| 발견일 | 2026-01-06 |

---

## 10. 셀 제목 행(is_header) 비트 — HWP 5.0 미기재

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 67 (문단 리스트 헤더), 표 82 (셀 속성) |
| 스펙 기술 | **정의 없음** (HWP 5.0 스펙에 누락) |
| HWPML 3.0 스펙 | `Header` 속성으로 문서화: "제목 셀인지 여부, default=false" |
| 실제 비트 위치 | LIST_HEADER의 확장 속성(bytes 6-7)에서 **bit 2** (= hwplib property bit 18) |
| hwplib 매핑 | `ListHeaderPropertyForCell`: bit16=안여백지정, bit17=셀보호, **bit18=제목셀**, bit19=양식모드 |
| 동작 규칙 | `表.repeat_header=true` AND 행0에 `is_header=true` 셀이 있어야 제목행 반복 |
| 우리 코드에서의 추출 | `cell.is_header = (cell.list_header_width_ref & 0x04) != 0;` |
| 수정 파일 | `src/model/table.rs`, `src/parser/control.rs`, `src/renderer/layout.rs` |
| 참조 문서 | `troubleshootings/repeat_header_image_duplication.md` |
| 발견일 | 2026-02-10 |

**추가 조사 (2026-07-03, #1831)**: bytes 6-7 의 **상위 바이트(b7)** 에도 하위 바이트와
동형의 비트 패턴(0x01/0x04/0x05 — 안여백/제목셀 유사)이 코퍼스에 대량 존재한다
(452 통제셋 .hwp 에서 b7≠0 셀 5,639개 vs b6 의 제목셀 비트 103개, 한 셀에 양쪽
공존도 흔함). **인과 플립 실험 결과 한글 2022 는 b7 비트를 렌더링·UI(COM Header/
HasMargin) 모두에서 무시**하며, 재저장 시 원값 그대로 보존 재기록한다
(2448877=`samples/float-stack-defer.hwp` 실험, 패처 `tools/patch_cell_flags.py`).
결론: b7 은 레이아웃 불활성
보존 데이터 — is_header 추출은 현행 b6(bit 2) 기준이 옳고, rhwp 는
`list_header_width_ref` u16 원값 보존으로 라운드트립 요건을 이미 충족한다.

---

## 11. 단 정의(ColumnDef) 너비/간격 — 비례값 인코딩

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 140 (단 정의) |
| 스펙 기술 | 너비 값을 HWPUNIT 절대값으로 암시 |
| 실제 인코딩 | **비례값** — 합계가 32768(=2^15)이 되도록 비례 분배 |
| 변환 공식 | `실제값 = 비례값 / 합계(32768) × body_width` |
| 검증 데이터 | KTX.hwp: w0=13722, g0=590, w1=18456, g1=0 → 합계=32768. col0_width=13722/32768×79652=33363 HU=117.7mm (HWP 대화상자와 일치) |
| 바이트 순서 | 스펙: `[attr][spacing][widths...]`, 실제(hwplib): same_width=false → `[attr][attr2][w0+g0][w1+g1]...` |
| HWPML 3.0 차이 | HWPX에서는 Width, Gap이 HWPUNIT 절대값 |
| 수정 파일 | `src/parser/body_text.rs`, `src/model/page.rs`, `src/renderer/page_layout.rs`, `src/serializer/control.rs` |
| 참조 문서 | `troubleshootings/column_def_proportional_widths.md` |
| 발견일 | 2026-02-16 |

---

## 12. 쪽 번호 위치(PageNumberPos) — 교차 참조 오류

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 149 (쪽 번호 위치) → "속성(표 148 참조)" |
| 스펙 오류 | 표 148은 "홀/짝수 조정"(bit 0~1만 정의). **올바른 참조는 표 150** |
| 표 150 내용 | bit 0\~7: 번호 모양(format), bit 8\~11: 표시 위치(position) |
| 실제 영향 | 표 148을 참조하면 bit 0\~3만 사용 → position을 bit 4\~7로 오해 → position=0(없음)으로 잘못 판독 |
| 검증 데이터 | attr=0x00000500: 올바른 format=(0x500&0xFF)=0, position=(0x500>>8)&0x0F=**5**(가운데 아래) |
| 섹션 제목 불일치 | "글자 겹침" 섹션에 쪽 번호 속성(표 150)이 위치 — 표 번호가 2~3개 밀려 있음 |
| 수정 파일 | `src/parser/control.rs` |
| 참조 문서 | `troubleshootings/task_70_page_number_false_completion.md` |
| 발견일 | 2026-02-08 |

---

## 13. CommonObjAttr — prevent_page_break 필드 미기재

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 72 (공통 객체 속성) |
| 스펙 기술 | attr(UINT32) 이후 필드 나열에 `prevent_page_break` **없음** |
| 실제 구현 | attr 이후에 INT32 `prevent_page_break` 4바이트 존재 |
| 위반 시 결과 | 이 필드를 생략하면 이후 모든 필드(개체 설명문 길이 + 데이터)의 오프셋이 4바이트 밀림 → 파일 구조 손상 |
| 수정 파일 | `src/serializer/control.rs`, `src/parser/control.rs`, `src/model/shape.rs` |
| 참조 문서 | `troubleshootings/picture_save_hancom_compatibility.md` §1 |
| 발견일 | 2026-02-15 |

---

## 14. CommonObjAttr attr bit 15~19 — 크기 기준 설정 필수

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 72 (공통 객체 속성) |
| 스펙 기술 | bit 15\~17: 오브젝트 폭 기준 (0=paper, 1=page, 2=column, 3=para, 4=absolute), bit 18\~19: 높이 기준 (0=paper, 1=page, 2=absolute) |
| 실제 해석 | 미설정(=0=paper)이면 한컴은 width/height를 **종이 대비 퍼센트**로 해석. 예: 42520 HU → 425.20% |
| 올바른 설정 | 그림 삽입 시 `(4 << 15) \| (2 << 18)` = width=absolute, height=absolute 명시 필요 |
| 수정 파일 | `src/wasm_api.rs` |
| 참조 문서 | `troubleshootings/picture_save_hancom_compatibility.md` §6 |
| 발견일 | 2026-02-15 |

---

## 15. SHAPE_COMPONENT ctrl_id — 그림은 "$pic"

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 86 (SHAPE_COMPONENT) |
| 스펙 기술 | 컨트롤 ID 필드 존재 (상세 값 미기재) |
| 실제 규칙 | 그리기 객체(GSO) = `"gso "` (0x67736F20), **그림(Picture) = `"$pic"` (0x24706963)** |
| 위반 시 결과 | 그림에 `"gso "`를 사용하면 한컴에서 이미지 미표시 |
| 수정 파일 | `src/serializer/control.rs`, `src/parser/tags.rs` |
| 참조 문서 | `troubleshootings/picture_save_hancom_compatibility.md` §2 |
| 발견일 | 2026-02-15 |

---

## 16. bin_data_id — 레코드 순번 (storage_id 아님)

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 이미지 참조 관련 |
| 스펙 기술 | bin_data_id의 정확한 의미 미기재 |
| 실제 의미 | **doc_info의 BinData 레코드 순번 (1-indexed)**. storage_id(CFB 파일명 번호)와는 별개 |
| 혼동 원인 | 대부분의 HWP 파일에서 storage_id가 1부터 순차 할당되어 순번과 우연히 일치 |
| 올바른 접근 | `bin_data_content[(bin_data_id - 1) as usize]` — 배열 인덱스로 접근 |
| native bridge API | `ImageNode.bin_data_id`는 1-based 참조값이고, `DocumentCore::get_bin_data(index)`의 `index`는 0-based `bin_data_content` 배열 인덱스다. 따라서 render tree의 `bin_data_id`로 조회할 때는 보통 `get_bin_data((bin_data_id - 1) as usize)`를 호출한다. |
| 위반 시 결과 | storage_id가 비순차인 파일(예: Worldcup_FIFA2010_32.hwp)에서 잘못된 이미지 매핑 |
| 수정 파일 | `src/renderer/layout.rs` (6곳), `src/wasm_api.rs` (1곳) |
| 참조 문서 | `troubleshootings/bin_data_id_index_mapping.md` |
| 발견일 | 2026-02-17 |

---

## 17. ShapeComponent 파싱 순서 — 그림자 정보(shadow info)

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 86~87 (SHAPE_COMPONENT) |
| 스펙 기술 | 채우기(fill) 이후의 데이터에 대한 **기술 없음** |
| 실제 순서 | `commonPart → lineInfo → fillInfo → shadowInfo(16B) → instid → skip → transparent` |
| shadow info 구조 | `shadow_type(u32) + shadow_color(u32, COLORREF) + offset_x(i32) + offset_y(i32)` = 16바이트 |
| 오파싱 시 결과 | 그림자 16바이트를 글상자 마진(8바이트)으로 읽으면 이후 필드 전부 오정렬 |
| hwplib 참조 | `ForShapeComponent.java` `shadowInfo()` 메서드 |
| 수정 파일 | `src/parser/control.rs` |
| 참조 문서 | `troubleshootings/shape_fill_transparency.md` §2차 원인 |
| 발견일 | 2026-02-17 |

---

## 18. 필드 CTRL_HEADER — memo_index 필드 미기재

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 154 (필드 컨트롤 CTRL_HEADER) |
| 스펙 기술 | `ctrl_id(4) + properties(4) + extra_properties(1) + command_len(2) + command(가변) + field_id(4)` — field_id 이후 **기술 없음** |
| 실제 구현 | field_id 이후에 **4바이트** 존재 (hwplib에서 `memoIndex`로 명명했으나, 실제로는 DocInfo의 **MemoShape 레코드 참조 인덱스**) |
| 필드 의미 | "메모 달기" 기능이 사용된 필드에서 해당 메모의 MemoShape ID를 가리킴. ClickHere(누름틀) 필드에서는 항상 0. hwplib의 `IDMappings.memoShapeCount`, `MemoShape` 레코드와 연동 |
| 위반 시 결과 | 4바이트 미직렬화 → 한컴에서 CTRL_HEADER 크기 불일치 → 누름틀 안내문이 빈 문자열로 보임 |
| 검증 방법 | hwplib `CtrlHeaderField.java`: `memoIndex = sr.readSInt4()`, `IDMappings.memoShapeCount`, 한컴 저장 파일과 바이트 비교 |
| hwplib 직렬화 | `ForControlField.java` 52~56행: `FIELD_UNKNOWN`이면 `memoIndex` 값 기록, **그 외 모든 필드 타입은 0으로 4바이트 기록**. 즉, hwplib도 이 4바이트를 항상 직렬화함 |
| hwplib 한계 | command 문자열을 있는 그대로 읽고 쓰기만 함. 내부의 `Direction:/HelpState:/Name:` 파싱·수정 기능 없음 → 누름틀 안내문/메모/이름 편집 불가 |
| 수정 파일 | `src/model/control.rs`, `src/parser/control.rs`, `src/serializer/control.rs` |
| 발견일 | 2026-03-15 |

---

## 19. 누름틀 메모 내용(M) 저장 위치 — command 문자열 내 HelpState

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 154 (필드 컨트롤), HWP 3.0 HWPML 스펙 §10.1.5 |
| 스펙 기술 | 누름틀의 메모(도움말) 저장 방식에 대한 **기술 없음** |
| 실제 구현 | command 문자열 내 `HelpState:wstring:N:텍스트` 패턴으로 저장 |
| command 전체 구조 | `Clickhere:set:{len}:Direction:wstring:{n}:{안내문} HelpState:wstring:{n}:{메모} Name:wstring:{n}:{이름} ` |
| 주의사항 | 각 wstring 값 뒤에 **공백 1개를 반드시 유지**해야 함. trim_end() 호출 시 한컴에서 안내문이 빈 문자열로 인식됨 |
| HWP 3.0 참조 | §10.1.5: "문자열 #3에 상황선에 표시할 도움말, 문자열 #2에 입력할 내용의 안내문" |
| 검증 방법 | field-01-memo.hwp: `HelpState:wstring:43:회사명은 회사이름입니다...` |
| 수정 파일 | `src/model/control.rs` (`guide_text()`, `memo_text()`, `build_clickhere_command()`) |
| 발견일 | 2026-03-15 |

---

## 20. 누름틀 필드 이름 — CTRL_DATA에 저장

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 154 (필드 컨트롤), HWP 3.0 HWPML 스펙 §8.8 |
| 스펙 기술 | 필드 이름의 저장/업데이트 메커니즘에 대한 **기술 없음** |
| 실제 구현 | CTRL_DATA 레코드에 ParameterSet(id=0x021B) → ParameterItem(id=0x4000, type=String)으로 저장 |
| command 내 Name: | command 문자열에도 `Name:wstring:N:이름`이 존재하지만, **한컴은 필드 이름 변경 시 command를 재구축하지 않음** — CTRL_DATA의 이름만 갱신 |
| 우선순위 | ① CTRL_DATA name → ② command Name: → ③ command Direction: (폴백) |
| HWP 3.0 참조 | §8.8: 추가 정보 블록에 "누름틀 필드 번호 + 필드 이름"으로 별도 저장 |
| 수정 파일 | `src/model/control.rs` (`field_name()`), `src/parser/control.rs`, `src/wasm_api.rs` |
| 발견일 | 2026-03-15 |

---

## 21. 필드 properties bit 15 — 초기 상태 여부

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 155 (필드 속성) |
| 스펙 기술 | "bit 15: 필드 내용이 수정되었는지 여부" (상세 동작 미설명) |
| 실제 동작 | bit 15 == 0: **초기 상태** (사용자 미입력). 한컴은 메모 추가 등 동작 시 안내문 텍스트를 필드 값으로 삽입하지만 bit 15는 0 유지 → 안내문으로 취급 |
| HWP 3.0 대응 | §10.1.5 바이너리 데이터: "bit 0: 사용자가 내용을 입력하지 않은 초기 상태인지 여부 (1=초기 상태)" |
| 렌더링 규칙 | bit 15 == 0이고 필드 값이 안내문과 동일 → 안내문으로 표시 (빨간 기울임). 클릭 시 텍스트 삭제하여 빈 필드화 |
| 검증 데이터 | field-01-memo.hwp: properties=0x00000001(bit 15=0), 필드 값="여기에 입력"=안내문 → 초기 상태 |
| 수정 파일 | `src/document_core/commands/document.rs` (`clear_initial_field_texts()`), `src/renderer/layout/paragraph_layout.rs` |
| 발견일 | 2026-03-16 |

---

## 22. control_mask — TAB·FIELD_END·LINE_BREAK 비트 누락 주의

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 60 (문단 헤더) |
| 스펙 기술 | "컨트롤 마스크" (어떤 제어문자를 포함해야 하는지 미설명) |
| 실제 규칙 | PARA_TEXT에 존재하는 **모든** 제어문자의 비트를 포함해야 함. controls 배열의 확장 제어문자뿐 아니라 TAB(0x0009), FIELD_END(0x0004), LINE_BREAK(0x000A)도 해당 |
| 비트 매핑 | `bit 3` = FIELD_BEGIN(0x0003), `bit 4` = FIELD_END(0x0004), `bit 9` = TAB(0x0009), `bit 10` = LINE_BREAK(0x000A) |
| 위반 시 결과 | TAB/FIELD_END 비트 누락 → 한컴 2010 비정상 종료 (한컴 2020은 관대) |
| 검증 방법 | 한컴 저장 파일의 PARA_HEADER control_mask와 바이트 비교. 예: 원본 0x00000218, 누락 시 0x00000008 |
| 수정 파일 | `src/serializer/body_text.rs` (`compute_control_mask()`) |
| 참조 | 본 문서 §7과 관련되나, §7은 확장 제어문자만 다룸. 이 항목은 인라인 제어문자까지 포함하는 확장 규칙 |
| 발견일 | 2026-03-15 |

---

## 23. PARA_TEXT 직렬화 — FIELD_BEGIN/FIELD_END 순서

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 62 (문단 텍스트) |
| 스펙 기술 | 확장 제어문자의 배치 순서에 대한 **기술 없음** |
| 실제 규칙 | 필드 컨트롤은 `FIELD_BEGIN(0x0003)` + 필드 내용 + `FIELD_END(0x0004)` 순서로 감싸야 함. 빈 필드(안내문만 표시)의 경우 FIELD_BEGIN 직후 FIELD_END |
| trailing FIELD_END | 필드 범위가 문단 텍스트 끝까지인 경우, FIELD_END는 해당 FIELD_BEGIN 컨트롤 **직후**에 배치 (다른 컨트롤보다 앞) |
| 위반 시 결과 | FIELD_END가 FIELD_BEGIN 앞에 오거나 다른 컨트롤 뒤에 밀리면 한컴에서 필드 범위 인식 실패 |
| 수정 파일 | `src/serializer/body_text.rs` (`trailing_end_after_ctrl` HashMap) |
| 발견일 | 2026-03-15 |

---

## 24. TAB 확장 데이터 — 7 code unit 보존 필수

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 62 (문단 텍스트) |
| 스펙 기술 | TAB(0x0009) 확장 제어문자 "크기 = 8" (내부 구조 미기재) |
| 실제 구조 | TAB 코드(2B) + **7개 추가 code unit(14B)** = 16바이트. 추가 데이터에 탭 너비/종류 등 정보 포함 |
| 라운드트립 규칙 | 파싱 시 7개 code unit을 보존하고, 직렬화 시 그대로 복원해야 함. 0으로 채우면 한컴에서 탭 간격이 틀어짐 |
| 수정 파일 | `src/model/paragraph.rs` (`tab_extended`), `src/parser/body_text.rs`, `src/serializer/body_text.rs` |
| 발견일 | 2026-03-15 |



## 25. 문단번호 시작 방식 — `numbering_id` 시스템의 암묵적 동작

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 40 (문단 번호), 표 45 (ParaShape의 `numbering_id`), `nwno` 컨트롤 (표 146) |
| 스펙 기술 | 시작 방식(앞 번호 이어/이전 번호 이어/새 번호 시작)에 대한 **명시적 속성 필드가 스펙에 없음** |
| 실제 동작 | 별도 구현이 아닌 `numbering_id` 시스템의 **자연스러운 부산물**. 같은 id = 이어서, 다른 id = 리셋, 이전 id 복귀 = 복원. 대화상자의 라디오 버튼은 이 동작에 이름을 붙인 것 |
| 한컴 도움말 | `format/numberbullet/numberbullet(new_number).htm` |
| 검증 파일 | `samples/para-head-num.hwp`, `samples/para-head-num-2.hwp` |
| 발견일 | 2026-03-19 |

### 저장 메커니즘

HWP는 시작 방식을 별도 필드로 저장하지 않고, **`numbering_id` 값의 변경/유지/복귀 패턴**으로 표현한다.

| 시작 방식 | 한컴 동작 | HWP 바이너리 표현 |
|----------|----------|------------------|
| **앞 번호 목록에 이어** | 직전 번호 문단에서 이어서 +1 | 이전 문단과 **같은 `numbering_id`** 유지 |
| **새 번호 목록 시작** | 카운터를 리셋하고 새 번호부터 시작 | **다른 `numbering_id`**로 변경 (새 Numbering 정의 생성) |
| **이전 번호 목록에 이어** | 중간에 다른 번호가 끼어도 이전 카운터 복원 | **이전에 사용한 `numbering_id`**로 복귀 |
| **명시적 번호 지정** | 지정 값으로 카운터 설정 | `nwno` 컨트롤 삽입 (표 146) |

### 렌더링 구현

`NumberingState`에 `history: HashMap<u16, [u32; 7]>`로 numbering_id별 카운터를 보존한다.

```
advance(numbering_id, level):
  if numbering_id 변경:
    history[old_id] ← 현재 카운터 저장
    if history[new_id] 있음:
      카운터 ← history[new_id] 복원 (이전 번호 이어)
    else:
      카운터 ← 상위 레벨(0..level) 상속 + 현재 레벨 이하 리셋 (새 번호 시작)
  카운터[level] += 1
  하위 레벨 리셋
```

### 대화상자 바인딩

읽기: `get_para_properties_at_native`에서 이전 문단 역순 스캔으로 판별
- 직전 번호 문단과 같은 id → mode=0 (앞 번호 이어)
- 이전에 같은 id 사용 이력 있음 → mode=1 (이전 번호 이어)
- 해당 id 첫 등장 → mode=2 (새 번호 시작)

쓰기: 시작 방식 변경 시 `numbering_id`를 조작
- "앞 번호 이어" → 이전 문단의 `numbering_id` 유지
- "새 번호 시작" → 새 Numbering 정의 생성 → 다른 `numbering_id` 할당
- "이전 번호 이어" → 이전에 사용한 `numbering_id`로 복귀

### 검증 예시 (para-head-num-2.hwp)

```
id=3 level=1 → counter[1]=1 → "가."   (새 번호 시작: 첫 등장)
id=3 level=1 → counter[1]=2 → "나."   (앞 번호 이어: 같은 id)
id=2 level=1 → counter[1]=1 → "가."   (새 번호 시작: id=2 첫 등장 → 리셋)
id=3 level=1 → counter[1]=3 → "다."   (이전 번호 이어: id=3 히스토리 [2] 복원 → +1)
id=4 level=1 → counter[1]=1 → "1."    (새 번호 시작: id=4 첫 등장 → 리셋, 다른 format)
id=4 level=1 → counter[1]=2 → "2."    (앞 번호 이어: 같은 id)
```


---

## 26. 표 CTRL_HEADER — Shape와 동일한 CommonObjAttr 구조

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 72 (공통 객체 속성), 표 79 (표 컨트롤) |
| 스펙 기술 | CommonObjAttr는 Shape/GSO 개체에 대해서만 기술. **표(tbl) 컨트롤의 CTRL_HEADER ctrl_data 구조는 미기재** |
| 실제 구현 | 표의 CTRL_HEADER ctrl_data는 **Shape/GSO와 동일한 CommonObjAttr 구조** 사용 |
| hwplib 검증 | `ForControlTable.java` line 62: `ForCtrlHeaderGso.read(table.getHeader(), sr)` — Shape와 동일한 reader 호출 |
| CommonObjAttr 구조 | `attr(u32) + vertical_offset(u32) + horizontal_offset(u32) + width(u32) + height(u32) + z_order(i32) + margin(i16×4) + instance_id(u32) + ...` |
| 이전 오류 | ctrl_data[0..4]를 표 전용 attr로, ctrl_data[4..]를 위치 데이터로 임의 해석 → 좌표 오프셋 4바이트 밀림 |
| 영향 범위 | `treat_as_char`, `text_wrap`, `vert_rel_to`, `horz_rel_to`, `vertical_offset`, `horizontal_offset` 등 표의 배치 속성 전체 |
| 수정 파일 | `src/parser/control.rs`, `src/renderer/layout/table_layout.rs`, `src/renderer/pagination/engine.rs`, `src/renderer/layout.rs` |
| 발견일 | 2026-03-19 |

### 표 배치 관련 추가 수정

InFrontOfText(글앞으로, text_wrap=3) / BehindText(글뒤로, text_wrap=2) 표는 **공간을 차지하지 않는 플로팅 개체**이다.

### CommonObjAttr text wrap 비트 실측 정정

`CTRL_HEADER`의 CommonObjAttr `bit 21..23`은 한컴/HWP5 실측과 PR #1015 `samples/test-image.hwp`/`samples/test-image.hwpx` fixture 기준으로 다음처럼 파싱한다.

| bits | rhwp IR | 한컴 UI 진단 라벨 |
|------|----|--------------|
| 0 | `Square` | 어울림 |
| 1 | `TopAndBottom` | 자리차지 |
| 2 | `BehindText` | 글뒤로 |
| 3 | `InFrontOfText` | 글앞으로 |

기존 스펙 표처럼 `1=Tight`, `2=Through`, `3=TopAndBottom`, `4=BehindText`, `5=InFrontOfText`로 읽으면 HWP5 원본의 저장값과 맞지 않는다. `test-image.hwp`는 각 배치 방식을 별도 페이지/문단의 단일 그림으로 저장한 fixture이며, control record 순서와 문단 라벨 순서는 `TopAndBottom(자리차지) → Square(어울림) → InFrontOfText(글앞으로) → BehindText(글뒤로)`이다.

같은 fixture의 `글자처럼 취급` 그림은 `treat_as_char=true`이고, wrap bits/HWPX `textWrap` 값은 `BehindText`로 남아 있다. 이 경우 사용자 UI의 핵심 속성은 wrap 라벨이 아니라 **글자처럼 취급** 플래그이므로, 진단기는 `배치: 글뒤로, 글자처럼=true`처럼 raw wrap과 TAC 플래그를 함께 표시해야 한다. HWPX의 `TIGHT`/`THROUGH`는 별도 XML 값으로 보존될 수 있으나, HWP5 CommonObjAttr 저장/파싱에서는 현재 `Square` 계열로 축약한다.

한컴 도움말의 “본문과의 배치” 설명 기준으로 `어울림`은 개체와 본문이 같은 줄을 나누어 쓰되 서로 자리를 침범하지 않도록 배치하는 의미이며, `자리 차지`와 별도 의미다. 따라서 HWP5 `Square/어울림` 그림은 `LINE_SEG`가 저장한 좁은 줄폭과 첫 wrap 줄의 `vertical_pos`를 함께 따라야 하고, `TopAndBottom/자리차지`로 강제 변환하지 않는다.

- **Pagination**: Shape처럼 `PageItem::Shape`로 수집 (높이 차지 없음)
- **Layout**: shapes pass에서 `layout_table` 호출하여 paper 기준 절대 좌표에 렌더링

| 기준 | Paper 수평 위치 | Paper 수직 위치 |
|------|----------------|----------------|
| ref 원점 | x=0.0 (종이 왼쪽) | y=0.0 (종이 상단) |
| 오프셋 | `common.horizontal_offset` (HWPUNIT) | `common.vertical_offset` (HWPUNIT) |

### 검증 데이터 (table-ipc.hwp)

| 표 | 한컴 속성 | 파싱 결과 |
|----|----------|----------|
| Table[3] | 종이/왼쪽/15mm, 종이/위/49mm, 68×5mm | Paper/4252=15mm, Paper/13890=49mm, 19276×1417 ✓ |
| Table[4] | 종이/왼쪽/15mm, 종이/위/54mm, 266.8×128mm | Paper/4252=15mm, Paper/15307=54mm, 75628×36288 ✓ |

---

## 27. PrvImage 스트림 — PNG 포맷 미기재

| 항목 | 내용 |
|------|------|
| 스펙 위치 | PrvImage 스트림 설명 |
| 스펙 기술 | 미리보기 이미지 (BMP 또는 GIF) |
| 실제 구현 | 최신 한컴 오피스 버전에서는 **PNG 포맷**으로 PrvImage를 저장 |
| 검증 방법 | biz_plan.hwp의 PrvImage 스트림 매직 바이트: `89 50 4E 47` (PNG 시그니처) |
| 검증 데이터 | biz_plan.hwp: PNG 724×1024, 12,097 bytes / shift-return.hwp: GIF 177×250, 1,264 bytes |
| 추정 | 구 버전에서는 GIF(확인됨), 최신 버전에서는 PNG로 변경. BMP 사용 사례는 미확인 |
| 대응 | `detect_image_format()`에서 PNG(`\x89PNG`) → BMP(`BM`) → GIF(`GIF`) 순으로 감지 |
| 수정 파일 | `src/parser/mod.rs`, `src/model/document.rs` |
| 발견일 | 2026-04-09 |

---

## 28. CFB 디렉토리 섹터 — FAT 체인 순회 미기재

| 항목 | 내용 |
|------|------|
| 스펙 위치 | HWP 5.0 파일 구조 — Compound File (OLE2) 설명 |
| 스펙 기술 | "Compound File에 대한 접근 방법은 OLE 관련 자료 또는 MSDN을 참고" (구체적 구현 미기재) |
| 실제 구현 | OLE2 표준에 따르면 디렉토리 엔트리도 데이터 스트림과 동일하게 FAT 체인으로 여러 섹터에 걸칠 수 있음. 한컴이 생성한 HWP 파일에서 실제 발생함 |
| 검증 방법 | shift-return.hwp: PrvImage 디렉토리 엔트리가 [17]번에 위치 (첫 번째 디렉토리 섹터는 512바이트 = 4개 엔트리만 수용) |
| 검증 데이터 | 첫 번째 디렉토리 섹터(sector 0): 엔트리 [0]~[3] / 이후 FAT 체인으로 연결된 섹터들: 엔트리 [4] 이상 |
| 추정 | 파일 크기가 크거나 BinData 등 스트림이 많을수록 디렉토리 엔트리가 증가하여 첫 번째 섹터를 초과할 가능성 높음 |
| 대응 | 헤더 offset 48의 dirStartSector에서 출발하여 FAT 체인(`0xFFFFFFFE` 종료)을 따라 모든 디렉토리 섹터 순회 |
| 수정 파일 | `rhwp-chrome/sw/thumbnail-extractor.js` (`extractPrvImage()` 함수) |
| 발견일 | 2026-04-12 |

---

## 29. SectionDef.flags — hide_master_page 비트 위치

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 142 (구역 정의 속성), HWP5 스펙 §4.2.10.1 |
| 스펙 기술 | 구역정의 속성 비트 필드 |
| 실제 비트 | **bit 2 (0x0004) = "바탕쪽 감춤(첫쪽)"** |
| 잘못된 구현 | 기존 코드는 bit 10 (0x0400) 으로 읽음 → 항상 `hide_master_page=false` |
| 위반 시 결과 | flags bit 2 가 SET 된 문서의 첫쪽에 바탕쪽이 본문 표 header 와 중복 렌더되어 텍스트가 두 번 그려지고, 좌측 바탕쪽 글상자가 우측 단(column) 본문을 가림 |
| 검증 데이터 | 21_언어_기출_편집가능본.hwp(`flags=0xC0080004`), exam_kor.hwp(`0xC0000004`), exam_eng.hwp(`0xC0000004`) — bit 2 SET ✓ / exam_math.hwp(`0x20000000`) — bit 2 unset, 영향 없음 |
| 추출 공식 | `hide_master_page = (sd.flags & 0x0004) != 0;` |
| 잠재 위험 | 다른 hide 비트들 (header/footer/border/fill/page_num) 도 오프셋 어긋남 가능성. 별도 이슈 후보 |
| 수정 파일 | `src/parser/body_text.rs:549` (읽기), `src/document_core/queries/rendering.rs:166` (쓰기) |
| 참조 문서 | `troubleshootings/master_page_hide_first_page.md` (예정) — Task #304/PR #305 |
| 기여자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) — 발견·수정·검증 |
| 발견일 | 2026-04-25 |

---

## 30. TabDef.position (표 7) — 데이터 좌표일 뿐, 한컴 조판 알고리즘은 비공개

**스펙은 "데이터 포맷" 만 정의 — 한컴은 그 값을 시멘틱적으로 재해석한다.**

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 7 (TabDef), HWP5 스펙 §4.2.7 |
| 스펙 기술 | TabDef.tabs[].position (UINT32, HWPUNIT) — 탭 정지 위치 |
| 실제 동작 | 한컴은 `position` 을 절대 좌표로 신뢰하지 않고, **리더 도트 (`fill_type ≠ 0`) + RIGHT 탭 (`tab_type=1`)** 인 경우 "이 줄 우측 끝까지 채움" 으로 시멘틱 재해석. 셀 안 paragraph 에서는 cell padding_right 영역 침범 금지. 들여쓰기 문단도 페이지번호 right edge 가 동일 위치에 정렬. |
| 잘못된 구현 | TabDef.position 그대로 사용 시: ① 셀 padding_right 영역 침범, ② 들여쓰기 문단 (소제목) 페이지번호가 좌측으로 어긋남, ③ 한 자리/두 자리 페이지번호 무관 leader 끝이 같은 x 라 페이지번호와 leader 겹침. |
| 위반 시 결과 | KTX.hwp 목차에서 장제목·소제목 페이지번호 정렬 어긋남, leader 가 페이지번호 안으로 침범, 페이지번호가 셀 padding 영역까지 침범. |
| 검증 데이터 | KTX.hwp 목차 (셀[10] 안 paragraph, ps_id=109 장제목 / ps_id=111 소제목, tab_def_id=5 RIGHT + fill=점선) |
| 해결 방향 | 리더 (`fill_type ≠ 0`) RIGHT 탭은 `effective_pos = effective_margin_left + available_width` (cell inner 우측 끝) 로 강제. 페이지번호 폭에 따라 leader.end_x 단축. 공백 only run 은 carry-over. |
| 수정 파일 | `src/renderer/layout/paragraph_layout.rs::resolve_last_tab_pending` + cross-run RIGHT take 분기 |
| 참조 문서 | `troubleshootings/toc_leader_right_tab_alignment.md` — Task #279, PR #282 |
| 기여자 | [@seanshin](https://github.com/seanshin) (Shin hyoun mouk) — 발견·구현 (`tab_type != 1` 클램핑 제외 + dasharray round cap), 메인테이너 — 6가지 추가 결함 식별 및 보강 |
| 발견일 | 2026-04-25 |

**핵심 교훈**: HWP 스펙은 **데이터 포맷 스펙이지 조판 알고리즘 스펙이 아니다**. 스펙대로 처리해도 한컴과 다른 결과가 나오면 한컴 시각 결과를 정답으로 삼아 의도를 역공학해야 한다. rhwp 는 "스펙 충실 구현체" 가 아니라 **"한컴 조판 결과를 재현하는 엔진"** 이어야 한다.

---

## 31. PAGE_BORDER_FILL 위치 기준 — HWP3 보정과 HWP5/HWPX 계약을 혼합하면 회귀 발생

| 항목 | 내용 |
|------|------|
| 스펙 위치 | 표 136 (쪽 테두리/배경 속성), `HWPTAG_PAGE_BORDER_FILL.attr bit 0` |
| 스펙 기술 | bit 0 `0=본문 기준`, `1=종이 기준` |
| OWPML 대응 | `hp:pageBorderFill@textBorder`: `CONTENT=본문 기준`, `PAPER=종이 기준` |
| 한컴 UI 관찰 | 한컴 에디터 UI에서는 HWP5/HWPX export 파일의 쪽 테두리 기준이 "쪽 기준"으로 표시될 수 있음. 이 UI 용어는 `CommonObjAttr::Page`와 같은 일반 개체 위치 기준이 아니며, `PAGE_BORDER_FILL`의 위치 기준 문맥에서 별도로 해석해야 한다. |
| 회귀 이력 | PR #956(`@jangster77`)은 HWP5/HWPX 다수 샘플 실측을 근거로 page border outline을 paper-based로 강제해 #952 회귀를 수정했다. 이후 Task #987에서 HWP3 sample16의 body 기준 정합을 맞추며 공통 렌더러가 다시 `attr bit0`을 존중하도록 바뀌었고, 이 과정에서 PR #956 계열 정합이 회귀될 위험이 생겼다. |
| 실제 문제 | HWP3 전용 보정과 HWP5/HWPX의 `PAGE_BORDER_FILL` 렌더링 계약을 하나의 `paper_based = (attr & 0x01) != 0` 로 처리하면 어느 한쪽이 깨진다. |
| 구현 원칙 | HWP3 parser가 필요한 호환성 힌트를 만들더라도, HWP5/HWPX page border 렌더링 계약과 섞지 않는다. `PageBorderFill`에 포맷 출처 또는 명시적 기준 enum을 추가해 HWP3 body 기준 보정과 HWP5/HWPX paper-based 렌더링을 분리해야 한다. |
| 주의 사항 | `PAGE_BORDER_FILL`의 "쪽/종이/본문" 용어와 `CommonObjAttr`의 `Paper/Page/Para/Column` 기준은 같은 이름을 쓰더라도 같은 계층의 계약이 아니다. 특히 한컴 UI의 "쪽 기준" 표시를 `CommonObjAttr::Page`로 단순 매핑하면 안 된다. |
| 관련 코드 | `src/renderer/layout.rs::build_page_borders`, `src/renderer/layout.rs::page_number_baseline_y`, `src/parser/hwp3/mod.rs` |
| 관련 문서 | `mydocs/pr/archives/pr_956_review.md`, `mydocs/report/task_m100_987_report.md`, `mydocs/working/task_m100_987_stage2.md` |
| 발견일 | 2026-05-19 |

### 권장 구조

현재 `PageBorderFill`은 `attr` raw 값만 보존하므로, 렌더러가 포맷별 의미 차이를 판단하기 어렵다.
다음과 같이 명시적인 계약 계층을 추가하는 방향이 안전하다.

```text
PageBorderFillBasis:
  - Hwp5HwpxPaperOutline
  - Hwp3BodyOutline
  - RawAttrFallback
```

또는 같은 의미를 담는 별도 hint 필드를 두어도 된다. 핵심은 HWP3 sample16을 위해 도입한 body 기준
보정이 HWP5/HWPX page border의 paper-based 정합을 다시 깨지 않도록 하는 것이다.

---

## EQEDIT (HWPTAG_EQEDIT) — `baseline` 과 `version_info` 사이 UINT2 zero 필드 누락

### 스펙 문제

`mydocs/tech/한글문서파일형식_5.0_revision1.3.md` 의 표 105 (수식 개체 속성) 가
EQEDIT record 의 byte order 를 다음과 같이 정의:

| 자료형 | 길이 | 설명 |
|--------|------|------|
| UINT32 | 4 | 속성 (스크립트 범위) |
| WORD | 2 | 스크립트 길이 |
| WCHAR array | 2×len | 글 수식 스크립트 |
| HWPUNIT | 4 | 수식 글자 크기 |
| COLORREF | 4 | 글자 색상 |
| INT16 | 2 | base line |
| WCHAR array | 2×len | 수식 버전 정보 |
| WCHAR array | 2×len | 수식 폰트 이름 |

→ baseline (INT16) 직후 바로 version_info 의 length 필드 (WORD) 가 오는 구조.

### 실제 데이터

`samples/math-001.hwp` 의 EQEDIT raw payload 분석:

```
0058: 4C 04 00 00       letter_size = 1100
      00 00 00 00       color = 0
      5D 00             baseLine = 93
      00 00             ← 스펙 표 105 누락! (UINT16 zero)
      13 00             version_info length = 19
      [Equation Version 60]
      07 00             font_name length = 7
      [HYhwpEQ]
```

baseline 과 version_info length 사이에 **UINT16 zero (2 byte)** 가 위치.

### hwplib 정합

```java
// hwplib/src/main/java/kr/dogfoot/hwplib/reader/bodytext/paragraph/control/eqed/ForEQEdit.java
eqEdit.setBaseLine(sr.readSInt2());
eqEdit.setUnknown(sr.readUInt2());        // ← 스펙 누락 영역
eqEdit.getVersionInfo().setBytes(sr.readHWPString());
eqEdit.getFontName().setBytes(sr.readHWPString());
```

hwplib 가 spec 누락 영역을 `unknown` UINT2 로 정확 처리. write 측도 동일 (`writeUInt2`).

### 누락 시 증상

UINT2 zero 를 read 하지 않으면 `[00 00]` 이 version_info length 로 오인되어:
- 첫 string = "" (length 0)
- 둘째 string = "Equation Version 60" (그 다음 `[13 00]` 을 length 19 로 읽음)

→ (version_info, font_name) 자리값 **swap**. 한컴이 잘못된 byte align 으로 후속
record (PARA_TEXT 등) 를 읽어 **본문 텍스트 미표시** (수식만 보임).

### 정정 (Task #1061)

- `src/model/control.rs::Equation` 에 `unknown: u16` 필드 추가
- `src/parser/control.rs::parse_equation_control` — baseline 후 `read_u16()` 추가
- `src/serializer/control.rs::serialize_equation_control` — baseline 후 `write_u16(eq.unknown)` 추가

### 관련 commits

Task #1061 (2026-05-22).

### 메모리 룰

- `feedback_diagnosis_layer_attribution` — EQEDIT raw byte 직접 분석으로 본질 정확 식별
- `reference_hwp2hwpx_library` — hwplib 권위 자료가 spec errata 의 결정적 근거

---

## 31. CommonObjAttr bit 13 — 한컴 UI `쪽 영역 안으로 제한`과 HWPX `flowWithText`

### 현상

한컴 개체 속성 대화상자의 `쪽 영역 안으로 제한` 체크박스는 HWP5 스펙 표 70의
CommonObjAttr `bit 13`에 대응한다. HWPX에서는 같은 값이 `<hp:pos flowWithText="1|0">`
속성으로 저장된다.

### 기준 샘플

- `samples/ta-pic-001-r-쪽영역안제한.hwp(x)`: 첫 번째 그림 `flowWithText=true`
- `samples/ta-pic-001-r-쪽영역안제한no.hwp(x)`: 첫 번째 그림 `flowWithText=false`

HWP5 `hwp5-anchor-trace` 결과도 첫 번째 그림 `CTRL_HEADER`에서 각각
`properties=0x002a2210`(bit 13 on), `properties=0x002a0210`(bit 13 off)로 갈린다.

### 정정

- `flowWithText`라는 HWPX 이름은 한컴 UI 의미와 직관적으로 맞지 않는다.
- rhwp 내부 IR은 `CommonObjAttr::flow_with_text`로 저장하되, 사용자-facing JSON/UI에서는
  한컴 UI 명칭에 맞춰 `restrictInPage`로 노출해야 한다.
- 이 값이 켜지면 한컴 UI에서 `서로 겹침 허용`은 비활성/false 취급된다.
- HWPX picture serializer는 `flowWithText`를 고정값으로 쓰지 말고 `CommonObjAttr::flow_with_text`
  값을 그대로 직렬화해야 한다.
- 렌더링 의미는 한컴 도움말의 `쪽 영역 안으로 제한` 설명을 따른다. 세로 위치 기준이
  `문단`인 개체가 편집 가능한 쪽 영역의 위/아래 끝을 벗어나면 개체를 다음 쪽으로 넘겨야
  하며, 표 셀 내부의 자리 차지 그림도 이 값이 켜져 있을 때만 개체의 세로 오프셋과 높이를
  행/셀 높이 흐름에 반영한다.

---

## 32. ParaShape attr1 bit 28/29 — HWPX `hh:border connect/ignoreMargin`

### 현상

한컴 문단 모양 대화상자의 `문단 테두리 연결`은 두 개 이상의 연속 문단을 하나의
문단 테두리로 연결하는 설정이다. HWP5에서는 `HWPTAG_PARA_SHAPE` 속성1의 bit 28에
저장되고, `문단 여백 무시`는 bit 29에 저장된다.

HWPX에서는 같은 값이 `<hh:paraPr>` 아래 `<hh:border>`의 `connect="1|0"` 및
`ignoreMargin="1|0"` 속성으로 저장된다.

### 기준 샘플

- `samples/[2027] 온새미로 1 본교재.hwp(x)` 6쪽 지문 박스 문단
- HWPX 원본 `paraPr`의 `<hh:border connect="1" ignoreMargin="1">`
- HWP5/HWPX 파싱 후 ParaShape `attr1` bit 28/29 on

### 정정

- Studio 문단 모양 속성 JSON은 `borderConnect`, `borderIgnoreMargin`을 노출해야 한다.
- 문단 모양 수정 명령은 위 값을 ParaShape attr1 bit 28/29에 반영해야 한다.
- HWPX serializer는 `connect`/`ignoreMargin`을 고정 `0`으로 쓰지 말고 ParaShape attr1
  bit 28/29에서 출력해야 한다.

---

## 검증 원칙

1. **바이너리 우선**: 스펙 문서보다 실제 바이너리 데이터를 신뢰한다
2. **3단계 교차 검증**: ① HWP 5.0 공식 스펙 → ② hwplib Java 참조 구현 → ③ 실제 HWP 파일 hex dump
3. **디버그 덤프**: `eprintln!`으로 원시 바이트를 출력하여 검증 후 제거한다
4. **색상 검증**: 색상값이 정확히 `#000000`(검정)으로 나오는지 확인한다. 근사값(`#010100` 등)은 바이트 오정렬을 의미한다
5. **다중 파일 교차 검증**: 가능하면 여러 HWP 파일에서 동일 필드를 검증한다
6. **0/null 반환 시 의심**: 파서가 0 또는 None을 반환하면 "원래 값이 0"이라는 결론은 스펙 원문과 대조 후에만 내린다
7. **읽기 관대/쓰기 엄격**: HWP 포맷은 읽기는 관대하지만 쓰기에서 1바이트라도 어긋나면 "파일 손상" 판정. 직렬화 코드는 반드시 한컴 정상 파일과 바이트 단위 비교
8. **발견 즉시 기록**: 새로운 불일치 발견 시 본 문서에 추가한다

---
