# Task #873 최종 결과 보고서

**이슈**: https://github.com/edwardkim/rhwp/issues/873
**브랜치**: `local/task873`
**작성일**: 2026-05-13
**제목**: HWP3 → HWP5/HWPX 변환본의 외부 참조 image 정합

## 1. 본질

### 본질 1 — HWP5 BinDataType::Link → Picture.external_path 미전달

`samples/hwp3-sample10-hwp5.hwp` (한컴이 HWP3 → HWP5 변환) 의 DocInfo BinData 가 `Link` 타입 + `abs_path = "D:\Work\Gwjang\temp\images\rdb02.gif"` 등을 정확히 보유. 그러나 `src/parser/control/shape.rs` 의 `parse_picture` 가 `bin_data_id` 만 읽고 `external_path` 미설정 → Task #741 의 fallback (`populate_external_images_from_dir`) 미동작.

### 본질 2 — HWPX content.hpf parser 의 외부 image 누락 + isEmbeded 미파싱

`samples/hwp3-sample10-hwpx.hwpx` 의 `Contents/content.hpf` (OPF manifest) 가 image 3 entry 보유:
```xml
<opf:item id="image1" href="D:\Work\Gwjang\temp\images\oracle.gif" media-type="image/gif" isEmbeded="0"/>
```

그러나 `src/parser/hwpx/content.rs:100` 가 `BinData/` 폴더 외 항목을 필터 아웃 + `isEmbeded` attribute 미파싱 → `bin_data_list` 가 완전 비어있음.

## 2. 정정 내용

### 2.1 HWP5 정정

`src/parser/mod.rs`:
- 신규 함수 `populate_link_image_paths(&mut Document)`: 모든 Picture 의 `bin_data_id` 로 BinData lookup → Link 타입이면 `external_path = abs_path.or(rel_path)` 설정
- `parse_hwp_with_cfb` + `parse_hwp_with_lenient` 양쪽에 `assign_auto_numbers` 다음 단계로 호출 추가

### 2.2 HWPX 정정

`src/parser/hwpx/content.rs`:
- `PackageItem::is_embedded: bool` 필드 추가
- `parse_content_hpf` 가 `isEmbeded` attribute 파싱 (default: true)
- `bin_data_items` 수집 조건 변경: `media_type` 이 `image/` 시작이거나 `BinData/` 경로
- 신규 test: `test_parse_content_hpf_external_image`

`src/parser/hwpx/mod.rs`:
- `is_embedded == false` → `BinDataType::Link` + `abs_path = item.href` 등록
- `is_embedded == true` → `BinDataType::Embedding` (기존 동작 유지)
- BinData image 로딩 시 `!is_embedded` 항목 skip
- `parse_hwpx` 종료 직전 `super::populate_link_image_paths(&mut doc)` 호출

### 2.3 Task #741 fallback 재사용

기존 `src/model/document.rs:341` 의 `populate_external_images_from_dir` 함수 (Task #741) 가 `external_path` 의 basename 추출 + HWP 파일 같은 dir 에서 매칭하여 자동 load. 본 task 의 변경 후 모든 포맷에 자동 동작.

## 3. 검증 결과

| 검증 항목 | 결과 |
|---|---|
| cargo build --release | ✓ |
| cargo test --release --lib | ✓ **1247 passed** (+1 신규 test, 회귀 0) |
| cargo clippy --release --lib | ✓ 경고 0 |
| hwp3-sample10-hwp5.hwp page 1 시각 정합 | ✓ 3 image (Oracle 로고 + DB 다이어그램 + 사람 그림) |
| hwp3-sample10-hwpx.hwpx page 1 시각 정합 | ✓ 3 image |
| hwp3-sample10.hwp (HWP3 원본) 회귀 | ✓ SVG 출력 동일 (diff 0) |
| hwp3-sample14 page 3 (Task #864) | ✓ 정합 유지 |

## 4. 영향도

| sample 종류 | 영향 |
|---|---|
| 일반 HWP5 (Embedding) | 무영향 (matches!(Link, _) = false) |
| 일반 HWPX (BinData/ 내장) | 무영향 (is_embedded = true → Embedding 유지) |
| **HWP3 → HWP5 변환본 (Link)** | **시각 정합 (목표)** |
| **HWP3 → HWPX 변환본 (isEmbeded="0")** | **시각 정합 (목표)** |
| HWP3 원본 | 무영향 (Task #741 으로 이미 처리) |
| Chart bin_data_id (60000+) | 무영향 (range 분리) |

## 5. CLAUDE.md 규칙 준수

- HWP5 파서 결함 → `src/parser/mod.rs` + `src/parser/control/shape.rs` 영역만 변경
- HWPX 파서 결함 → `src/parser/hwpx/` 영역만 변경
- Renderer / Document IR / 공통 모듈 변경 없음 (Task #741 fallback 재사용)
- HWP3 전용 분기 추가 없음

## 6. 단계별 보고서

- 수행계획서: `mydocs/plans/task_m100_873.md`
- 구현계획서 v1: `mydocs/plans/task_m100_873_impl.md`
- 구현계획서 v2 (HWPX 포함): `mydocs/plans/task_m100_873_impl_v2.md`
- Stage A 본질 진단: `mydocs/working/task_m100_873_stage_a.md`
- Stage B 정정 + 검증: `mydocs/working/task_m100_873_stage_b.md`

## 7. 커밋 메시지 (안)

```
Task #873: HWP5/HWPX 외부 참조 image 정합 — Picture.external_path 전달

본질 1 (HWP5): BinDataType::Link 의 abs_path/rel_path 가 Picture.image_attr.
external_path 에 전달되지 않아 Task #741 fallback (populate_external_images_
from_dir) 미동작. src/parser/mod.rs 에 populate_link_image_paths 후속 함수
추가하여 모든 Picture 의 bin_data_id 로 BinData lookup → Link 인 경우
external_path 설정.

본질 2 (HWPX): content.hpf parser 가 BinData/ 폴더 외 image 항목 (예:
isEmbeded="0" + Windows absolute path) 을 필터 아웃 + isEmbeded attribute
미파싱. src/parser/hwpx/content.rs 의 PackageItem 에 is_embedded 추가 +
image/* media-type 수집. src/parser/hwpx/mod.rs 가 isEmbeded="0" 인 경우
BinDataType::Link + abs_path 설정 + parse_hwpx 종료 시
populate_link_image_paths 호출.

이후 Task #741 의 populate_external_images_from_dir fallback 이 basename
+ same-dir 매칭으로 image 자동 load (한컴 viewer 정합).

검증: cargo test 1247 passed (+1 신규, 회귀 0), clippy 경고 0.
hwp3-sample10-hwp5.hwp / hwp3-sample10-hwpx.hwpx 모두 3 image 정합.

closes #873
```

## 8. 결론

HWP3 → HWP5/HWPX 변환본 의 외부 참조 image (한컴 viewer 의 "외부 file 참조" 케이스) 가 모두 정합. 1247 테스트 회귀 0, clippy 경고 0. CLAUDE.md HWP5/HWPX 파서 규칙 정합 (parser 영역만 변경).

📋 **Task #873 최종 결과 보고서 — 커밋 + 이슈 클로즈 승인 요청드립니다.**
