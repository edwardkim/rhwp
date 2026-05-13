# Task #873 Stage A 본질 정밀 진단

**작성일**: 2026-05-13
**브랜치**: `local/task873`

## 범위 확장

사용자 추가 지시: **HWPX 도 동일 결함 보유** — 본 task 에 포함.

## A.1 DocInfo BinData dump 결과

진단용 임시 print 추가 (`src/main.rs` 의 dump 명령) 후 결과:

### HWP5 (hwp3-sample10-hwp5.hwp)

```
=== DocInfo BinData ===
  [1] type=Link storage_id=0 ext=None abs=Some("D:\\Work\\Gwjang\\temp\\images\\oracle.gif") rel=Some("") status=NotAccessed
  [2] type=Link storage_id=0 ext=None abs=Some("D:\\Work\\Gwjang\\temp\\images\\rdb02.gif") rel=Some("") status=NotAccessed
  [3] type=Link storage_id=0 ext=None abs=Some("D:\\Work\\Gwjang\\temp\\images\\s1.jpg") rel=Some("") status=NotAccessed
```

- BinData parser (`src/parser/doc_info.rs`) 정상 동작 — Link 타입 3 entry, abs_path 정확 파싱
- 그러나 Picture 의 `external_path` 미설정

### HWPX (hwp3-sample10-hwpx.hwpx)

```
=== DocInfo BinData ===
(없음)
```

- `doc_info.bin_data_list` 가 **완전 비어있음**
- 그러나 content.hpf 의 OPF manifest 는 image 3 entry 보유:

```xml
<opf:item id="image1" href="D:\Work\Gwjang\temp\images\oracle.gif" media-type="image/gif" isEmbeded="0"/>
<opf:item id="image2" href="D:\Work\Gwjang\temp\images\rdb02.gif" media-type="image/gif" isEmbeded="0"/>
<opf:item id="image3" href="D:\Work\Gwjang\temp\images\s1.jpg" media-type="image/jpg" isEmbeded="0"/>
```

### HWP3 (hwp3-sample10.hwp) — 참조

```
=== DocInfo BinData ===
  [1] type=Link storage_id=1 ext=Some("gif") abs=Some("D:\\...\\oracle.gif") rel=Some("D:\\...\\oracle.gif") status=NotAccessed
```

HWP3 파서는 picture record 직접 처리 시 `pic.image_attr.external_path = pic_name` 설정 (Task #741 정합) → `populate_external_images_from_dir` fallback 동작.

## A.2 Picture bin_data_id ↔ BinData index 매핑

### HWP5

- HWP5 spec: Picture 의 `bin_data_id` 는 DocInfo BinData list 의 **1-based index**
- 본 sample: Picture bin_data_id = 1, 2, 3 ↔ BinData[0], BinData[1], BinData[2]
- 매핑 정합 — `bin_data_list.get(bin_data_id - 1)` 으로 접근

### HWPX

- HWPX spec: section.xml `<hp:img binaryItemIDRef="image1"/>` 으로 OPF manifest 의 `<opf:item id="image1">` 참조
- 본 환경 파서 (`src/parser/hwpx/section.rs:1177-1181`): `"image1"` → 숫자 추출 → `bin_data_id = 1`
- 그러나 `doc_info.bin_data_list` 가 비어있어 매핑할 entry 가 없음

## A.3 본질 위치

### HWP5 본질

**`src/parser/control/shape.rs:857`** + **`src/parser/mod.rs`**:
- `parse_picture` 가 `bin_data_id` 만 읽고 `external_path` 미설정
- 정정 위치: `src/parser/mod.rs` 의 `parse_from_bytes` 에 후속 함수 추가 (Picture lookup → BinData Link → external_path 설정)

### HWPX 본질 (2 곳)

**1. `src/parser/hwpx/content.rs:100`** — content.hpf BinData 항목 필터링:
```rust
if href.starts_with("BinData/") || href.contains("/BinData/") {
    info.bin_data_items.push(...);
}
```
→ external linked image (예: `D:\...\oracle.gif`) 가 **필터 아웃**됨. media-type 이 `image/*` 인 모든 item 을 수집해야 함.

또한 `isEmbeded` attribute 미파싱 → Link vs Embedding 구분 불가. PackageItem 에 `is_embedded: bool` 필드 추가 필요.

**2. `src/parser/hwpx/mod.rs:82-89`** — BinData 등록:
```rust
doc_info.bin_data_list.push(BinData {
    data_type: BinDataType::Embedding,  // ← 모두 Embedding 으로 등록
    storage_id: (i + 1) as u16,
    extension: Some(ext),
    ..Default::default()
});
```
→ `isEmbeded="0"` (Link) 인 경우 `BinDataType::Link` + `abs_path` 설정해야 함.

## A.4 정정 후보 선정

### HWP5

**위치**: `src/parser/mod.rs` 의 `parse_from_bytes` 의 `assign_auto_numbers` 다음 위치
**함수**: `populate_link_image_paths(&mut doc)` 신규

Picture (in Control::Picture or ShapeObject::Picture) 의 `bin_data_id` 로 `doc.doc_info.bin_data_list` lookup → `BinDataType::Link` 이면 `pic.image_attr.external_path = bd.abs_path.or(bd.rel_path)` 설정.

### HWPX

**위치 1**: `src/parser/hwpx/content.rs`
- `PackageItem` struct 에 `is_embedded: bool` 추가
- `parse_content_hpf` 에서 `isEmbeded` attribute 파싱
- bin_data_items 수집 조건 변경: `media-type` 이 `image/*` 시작이거나 기존 BinData/ 경로

**위치 2**: `src/parser/hwpx/mod.rs:82-89`
- `item.is_embedded == false` 인 경우 `BinDataType::Link` + `abs_path = item.href` 설정
- 그 외 (`is_embedded == true`) 는 기존대로 Embedding

**위치 3**: 동일한 `populate_link_image_paths` 후속 함수 호출 — 모든 포맷 공통 동작.

## A.5 회귀 영향 분석

| sample | 영향 | 회귀 위험 |
|---|---|---|
| hwp3-sample10-hwp5.hwp (HWP5 Link) | **시각 정합 (목표)** | 양 |
| hwp3-sample10-hwpx.hwpx (HWPX Link) | **시각 정합 (목표)** | 양 |
| hwp3-sample10.hwp (HWP3 외부참조) | 무영향 (HWP3 파서가 이미 처리, Task #741) | 없음 |
| 일반 HWP5 sample (Embedding) | 무영향 (`matches!(Link, _) = false`) | 없음 |
| 일반 HWPX sample (BinData/ 내장) | 무영향 (`is_embedded=true`) | 없음 |

`populate_external_images_from_dir` fallback (main.rs:238) 가 모든 포맷에 자동 동작.

## Stage A 결론

본질 확정:
1. **HWP5**: parser 가 Picture 와 BinData Link 매핑 누락 → `src/parser/mod.rs` 후속 함수 추가
2. **HWPX**: content.hpf parser 가 external image 필터 아웃 + isEmbeded 미파싱 → `src/parser/hwpx/content.rs` + `src/parser/hwpx/mod.rs` 정정

두 본질 정정 후 Task #741 의 generic fallback 이 자동 동작 → 모든 포맷의 외부 참조 image 정합.

## 구현계획서 갱신 필요

Stage B 구현 범위가 HWP5 단일 → HWP5 + HWPX 로 확장. 구현계획서 v2 갱신 필요.

📋 **Stage A 완료. 구현계획서 v2 (HWPX 포함) 갱신 + Stage B 진행 승인 요청드립니다.**
