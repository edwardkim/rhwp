---
kind: reference
status: active
canonical: mydocs/tech/hwp_spec_errata.md
last_verified: 2026-07-17
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

### 3. SectionDef (secd) ⚪ raw round-trip 보존

- hwplib: `ForControlSectionDefine.java` → CtrlData 읽기
- 구역 설정의 부가 메타데이터
- **현재 영향**: 없음 (raw bytes 보존으로 round-trip 정상)

### 4. Table (tbl) ⚪ raw round-trip 보존

- hwplib: `ForControlTable.java` → CtrlData 읽기
- 표의 부가 메타데이터
- **현재 영향**: 없음

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
| CTRL_DATA raw bytes 보존 (round-trip) | ✅ `para.ctrl_data_records` |
| Bookmark 이름 추출 | ✅ `parse_ctrl_data_field_name()` |
| Field 이름 추출 | ✅ `field.ctrl_data_name` |
| 새 Bookmark CTRL_DATA 생성 | ✅ `build_bookmark_ctrl_data()` |
| Bookmark 삭제/이름변경 시 동기화 | ✅ |
| 기타 컨트롤 구조적 파싱 | ⚪ 불필요 (raw 보존으로 충분) |
| 새 컨트롤 생성 시 CTRL_DATA 생성 | ⚠️ Bookmark만 구현, 기타 미구현 |

## 향후 고도화 대상

1. **표 셀 내 커서 이동**: BookmarkInfo에 cellPath 추가 → 표 안 책갈피 정확한 위치 이동
2. **FIELD_BOOKMARK(%bmk)**: 현재 대상 파일에서 미발견, 필요 시 파싱 추가
3. ~~새 컨트롤 생성~~: 표/그림 등은 CTRL_DATA 없이 삽입 → 직렬화 → 한컴 로드 성공 확인됨. CTRL_DATA는 선택적(optional) 레코드.
