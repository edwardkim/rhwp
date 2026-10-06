# Task #1591 — Stage 1 완료보고서 (조사 + RED)

**단계**: 근본원인 정밀 규명 + RED
**브랜치**: `local/task1591`

## 1. 근본원인 (확정)

`src/serializer/hwpx/section.rs:416-426` 이 **모든 Bookmark 를 문단 시작(첫 run)으로 hoisting**:

```rust
// Bookmark는 IR에 위치 정보가 없어 문단 시작(첫 run)에 배치한다.
for ctrl in &para.controls {
    if let Control::Bookmark(bm) = ctrl { ... splitter.content.push("<hp:ctrl>...bookmark...") }
}
```

para0(빈 문단, 거대 중첩표) IR controls(순서): `[SectionDef, ColumnDef, Table, PageNumberPos,
Bookmark]`. 원본에서 북마크는 **끝**(byte 46461).

**rt 재파싱 결과**(36384689):
```
cc=33 (불변)   char_shapes: pos=0 id=25,  pos=32 id=10  (원본 24 → 32, +8)
controls 순서: [SectionDef, ColumnDef, Bookmark, Table, PageNumberPos]  ← 북마크가 앞으로 이동
```

→ 북마크가 **8유닛 슬롯을 점유**(cc=33=4슬롯×8+1, line 417 주석 "char_count 미포함"은 부정확)
하며, hoisting 이 북마크를 표 앞으로 **재배치**해 후속 Table/PageNumberPos 슬롯을 +8 밀어
char_shape(후위 컨트롤에 연동) 경계를 24→32 시프트시킨다. **#1584 ColumnDef 와 동형**
(슬롯 위치 미추적 → 오배치). #1584 무관(직전 커밋 바이너리 동일) 재확인.

> roundtrip diff 는 북마크 자체는 비교 제외(roundtrip.rs:901)하나, 재배치로 인한 char_shape
> 시프트는 검출한다.

## 2. Class C 분해 (3건 → 2 클래스)

| 파일 | controls | 시프트 | 클래스 |
|------|----------|-------|--------|
| 36384689 | [..,Table,PageNum,**Bookmark**] | +8 | **C1 북마크 hoist** |
| 36385445 | [..,Table,PageNum,**Bookmark**] | +8 | **C1 북마크 hoist** (동일) |
| 36388711 | [..,Table,**Field**(ClickHere)] | −16/−8 | **C2 필드** (별개, F3 인접) |

→ 본 타스크는 **C1(2건, 북마크)** 대상. C2(36388711, 필드 ClickHere)는 별개 근본 — 분리.

## 3. RED 테스트

`task1591_bookmark_not_hoisted_before_slot`: 표 슬롯 뒤 북마크 문단 roundtrip →
컨트롤 순서 검증. 현재 `["bm","tbl"]`(hoist 로 뒤바뀜), 기대 `["tbl","bm"]` → **RED 확인**.

## 4. 수정 방향 (제안 — 승인 대상)

북마크를 hoisting 하지 말고 **컨트롤 시퀀스의 제 위치에 슬롯으로 방출**. para.controls 는 이미
올바른 순서(북마크 index 보존)이므로, 북마크를 슬롯 시스템(`is_hwpx_inline_slot`)에 포함하여
char-offset 위치대로 방출하면 순서·char_shape 보존 가능.

**위험(F3 인접)**: 
- 종전 hoist 는 "위치 추적 불가" 케이스 대비 fallback. 일부 북마크는 진짜 위치 정보 부재일 수
  있어, 슬롯 편입 시 **위치 추정 불가 케이스에서 광역 회귀** 가능(F3 2회 실패 전례).
- roundtrip.rs:901 의 북마크 비교 제외도 재검토 필요.
- **채택 게이트 = 통제 비교 순효과>0·악화0**, 악화 시 전량 롤백.

## 5. 판단 요청

Stage 2 로 진행(슬롯 편입 구현 + baseline + 통제 비교)할지, 위험 감안하여 보류할지 결정 요청.
근본원인·RED 확보됨. C2(36388711)는 별 이슈로 분리 권장.
