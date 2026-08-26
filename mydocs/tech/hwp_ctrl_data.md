---
kind: reference
status: active
canonical: mydocs/tech/hwp_spec_errata.md
last_verified: 2026-08-10
---

# HWPTAG_CTRL_DATA 분석 (hwplib 크로스 체크)

## 개요

HWPTAG_CTRL_DATA (tag = HWPTAG_BEGIN + 71)는 컨트롤의 부가 데이터를 ParameterSet 형태로 저장하는 레코드.
CTRL_HEADER 바로 다음에 위치하며, 컨트롤의 이름/속성 등을 담는다.

## ParameterSet 바이너리 구조

```
offset  size  설명
0       2     ps_id (UINT16) - ParameterSet ID (예: 0x021B = 필드/책갈피 이름)
2       2     count (INT16) - 파라미터 아이템 수
4       2     dummy (UINT16) - 예약
6+      가변   ParameterItem[] × count
```

### ParameterItem 구조

```
offset  size  설명
0       2     item_id (UINT16) - 아이템 ID (예: 0x4000 = 이름)
2       2     item_type (UINT16) - 데이터 타입
4+      가변   value - 타입에 따라 다름
```

### ParameterType 값

| 값 | 이름 | 크기 |
|---|------|------|
| 0x0000 | NULL | 0 |
| 0x0001 | String | 2 + N*2 (len + UTF-16LE) |
| 0x0002 | Integer1 | 4 |
| 0x0003 | Integer2 | 4 |
| 0x0004 | Integer4 | 4 |
| 0x0005 | Integer | 4 |
| 0x0006 | UnsignedInteger1 | 4 |
| 0x0007 | UnsignedInteger2 | 4 |
| 0x0008 | UnsignedInteger4 | 4 |
| 0x0009 | UnsignedInteger | 4 |
| 0x8000 | ParameterSet | 재귀 (중첩 ParameterSet) |
| 0x8001 | Array | 2(count) + 2(id) + items... |
| 0x8002 | BINDataID | 2 |

## CTRL_DATA를 사용하는 컨트롤 (hwplib 기준 7종)

### 1. Bookmark (bokm) ✅ 구현 완료

- **파싱**: CTRL_DATA → ParameterSet → 이름 추출
- **생성**: `build_bookmark_ctrl_data(name)` → ParameterSet 바이너리 생성
- **ps_id**: 0x021B, item_id: 0x4000, type: String
- **예시**: `1b 02 01 00 00 00 00 40 01 00 10 00 [UTF-16LE name]`

### 2. Field (%clk, %hlk 등) ✅ 구현 완료

- **파싱**: CTRL_DATA → `field.ctrl_data_name` 추출
- 같은 ParameterSet 구조 (ps_id=0x021B, item_id=0x4000)
- 필드 이름이 command 문자열과 별도로 저장됨

### 3. SectionDef (secd) ✅ canonical raw owner

- hwplib: `ForControlSectionDefine.java` → CtrlData 읽기
- 구역 설정의 부가 메타데이터
- 첫 중첩 `CTRL_HEADER` 전에 나타나는 첫 직접 자식 `CTRL_DATA`는
  `Paragraph.ctrl_data_records`가 소유한다.
- `SectionDef.extra_child_records`에는 같은 첫 레코드를 중복 보존하지 않는다.
- 추가 직접 자식, 중첩 control의 `CTRL_DATA`, 중첩 header 뒤의 직접 자식
  `CTRL_DATA`는 `extra_child_records`에 raw 보존한다.
- 저장 시 `CTRL_HEADER(secd)` 직후에 canonical payload를 한 번만 출력한다. 과거 이중 소유 IR은
  동일 level·payload의 exact duplicate만 제거한다.
- **호환 근거**: #3507 — 동일 payload를 두 번 쓰면 rhwp 재로드는 성공하지만 macOS 한컴 Viewer와
  Windows 한글 2024가 파일 손상으로 거부한다.

### 4. Table (tbl) ✅ raw round-trip + 제한된 HWPX→HWP materialization

- hwplib: `ForControlTable.java` → CtrlData 읽기
- 표의 부가 메타데이터
- HWP5 파서의 canonical owner는 해당 control index의 `Paragraph.ctrl_data_records`다.
  serializer는 payload를 해석하거나 item id를 다시 배정하지 않고 raw bytes를 출력한다.
- HWPX→HWP adapter는 `3×2 + repeat_header + RowBreak` 조건에서만 다음 104바이트
  table-layout payload를 materialize한다.

```text
ParameterSet 0x021B
  item 0x0242 (ParameterSet)
    ParameterSet 0x0242
      Integer4 items:
        (0x4000, 3826), (0x4001, 1048), (0x4002, 28346),
        (0x4003, 8475), (0x4004, 708),  (0x4005, 0),
        (0x4006, 2),    (0x4007, 9),    (0x4008, 0),
        (0x4009, 59528),(0x400A, 84188)
```

- 위 item id는 이 opaque payload에서 함께 관찰된 정확한 값이다. `0x4000 + index`를
  범용 할당 규칙이나 확장 가능한 사설 namespace로 취급하지 않는다. 인접 ID에 새 의미를
  배정하지 말고, 새 payload는 독립된 바이너리 근거와 실제 소비자를 먼저 확정한다.
- **근거와 한계**: #1064와 #1099에서 같은 104바이트 payload가 관찰됐지만, 당시 결과에는
  다른 HWP5 저장 계약도 함께 작용했다. #4438의 합성 exact/item-id 역순/CTRL_DATA 없음
  단일 변화축은 외부 reader에서 모두 열리고 Save As됐다. exact와 item-id 역순 입력은 같은
  2쪽 구조를 렌더링했고, CTRL_DATA가 없으면 1쪽이었다. 저장 출력은 각 입력의 11개
  `(item_id, value)` 결합을 유지한 채 item record 순서를 반전했으며, 없음 입력은 계속
  CTRL_DATA가 없었다. 따라서 이 fixture에서는 payload 존재가 관찰된 layout에 영향을 주지만
  item-id 치환만으로는 차이가 관찰되지 않았다. 이는 개별 item의 일반 의미·무시 여부,
  namespace 또는 다른 문서의 호환성을 확정하지 않으므로, adapter가 재현하는 opaque bytes와
  rhwp 내부 보존 경계만 계약으로 삼는다.
- source IR 불변이 필요한 저장 경계는
  `DocumentCore::export_hwp_with_adapter_snapshot()`으로 adapter를 `Document` 복제본에만
  적용한다. 현재 세션 저장 경로가 이 API를 소비하며, materialized raw bytes와 출처 marker는
  저장 snapshot에만 존재한다. 반대로 `export_hwp_with_adapter()`는 live IR을 정규화하는
  in-place API이므로 source-nonmutation 계약으로 해석하지 않는다.

### 5. Picture ($pic) ⚪ raw round-trip 보존

- hwplib: `ForControlPicture.java` → CtrlData 읽기 (GSO 공통)
- 그림 개체의 부가 메타데이터
- **현재 영향**: 없음

### 6. Rectangle ($rec) ⚪ raw round-trip 보존

- hwplib: `ForControlRectangle.java` → CtrlData 읽기 (GSO 공통)
- 사각형/글상자의 부가 메타데이터
- **현재 영향**: 없음

### 7. 기타 GSO (선/원/호/다각형/곡선/OLE/묶음) ⚪ raw round-trip 보존

- hwplib: `ForGsoControl.java` → captionAndCtrlData() 공통 처리
- **현재 영향**: 없음

## 우리 구현 현황

| 항목 | 상태 |
|------|------|
| CTRL_DATA raw bytes 보존 (round-trip) | ✅ `para.ctrl_data_records` canonical owner + control별 raw extra |
| Bookmark 이름 추출 | ✅ `parse_ctrl_data_field_name()` |
| Field 이름 추출 | ✅ `field.ctrl_data_name` |
| 새 Bookmark CTRL_DATA 생성 | ✅ `build_bookmark_ctrl_data()` |
| Bookmark 삭제/이름변경 시 동기화 | ✅ |
| 기타 컨트롤 구조적 파싱 | ⚪ 불필요 (raw 보존으로 충분) |
| 새 컨트롤 생성 시 CTRL_DATA 생성 | ⚠️ Bookmark와 위 제한된 table-layout 계약만 구현, 기타 미구현 |
| SectionDef exact duplicate 방어 | ✅ 첫 중첩 header 전의 첫 직접 자식은 단일 owner, legacy 이중 소유 IR은 serializer에서 1회 출력 |

## 향후 고도화 대상

1. **표 셀 내 커서 이동**: BookmarkInfo에 cellPath 추가 → 표 안 책갈피 정확한 위치 이동
2. **FIELD_BOOKMARK(%bmk)**: 현재 대상 파일에서 미발견, 필요 시 파싱 추가
3. ~~새 컨트롤 생성~~: 표/그림 등은 CTRL_DATA 없이 삽입 → 직렬화 → 한컴 로드 성공 확인됨. CTRL_DATA는 선택적(optional) 레코드.
