# Task #873 Stage B 정정 구현 + 검증

**작성일**: 2026-05-13
**브랜치**: `local/task873`

## B.1 정정 구현

### 변경 파일 (3)

1. **`src/parser/mod.rs`**:
   - `populate_link_image_paths` 후속 함수 추가 (BinData Link → Picture.external_path)
   - `parse_hwp_with_cfb` + `parse_hwp_with_lenient` 양쪽에서 `assign_auto_numbers` 다음에 호출

2. **`src/parser/hwpx/content.rs`**:
   - `PackageItem::is_embedded: bool` 필드 추가
   - `parse_content_hpf` 가 `isEmbeded` attribute 파싱 (default: true)
   - bin_data_items 수집 조건 변경: `media_type` 이 `image/` 시작이거나 `BinData/` 경로
   - 신규 test: `test_parse_content_hpf_external_image` (외부 절대 경로 + isEmbeded="0" 케이스)

3. **`src/parser/hwpx/mod.rs`**:
   - `is_embedded == false` → `BinDataType::Link` + `abs_path = item.href` 등록
   - `is_embedded == true` → `BinDataType::Embedding` (기존 동작)
   - BinData 이미지 로딩 시 `!is_embedded` 항목 skip (ZIP 내 부재)
   - `parse_hwpx` 종료 직전 `super::populate_link_image_paths(&mut doc)` 호출

## B.2 검증 결과

### 빌드 + 테스트 + clippy

```
cargo build --release: ✓
cargo test --release --lib: 1247 passed (1246 → 1247, +1 신규 test, 회귀 0)
cargo clippy --release --lib: 경고 0
```

### Image 임베드 검증

| 파일 | 변경 전 data URI 수 | 변경 후 | 변화 |
|---|---|---|---|
| hwp3-sample10-hwp5.hwp | 0 | **3** | ✓ |
| hwp3-sample10-hwpx.hwpx | 0 | **3** | ✓ |
| hwp3-sample10.hwp (HWP3 원본) | 3 | 3 | 회귀 0 (SVG 출력 동일) |

### 시각 검증

`hwp3-sample10-hwp5.hwp` page 1 SVG → PNG:
- Oracle 로고 ✓
- Database 다이어그램 (Disaster Tolerance, 24X365 OLTP, Data Warehouse, Web Server, Scalable Client/Server) ✓
- 창원대학교 데이터베이스 연구실 + 사람 그림 (s1.jpg) ✓

`hwp3-sample10-hwpx.hwpx` page 1: 동일 (3 image 모두 정상)

### 회귀 검증

- hwp3-sample14.hwp page 3 ("Cut&Paste 할 영역" caption): Task #864 정합 유지 ✓
- hwp3-sample10.hwp (HWP3 원본): SVG 출력 변경 없음 (diff 0) ✓
- 모든 기존 sample 의 일반 BinData (Embedding/Storage): 변경 없음

## B.3 회귀 영향 범위

| sample 종류 | 영향 |
|---|---|
| 일반 HWP5 (Embedding) | 무영향 (matches!(Link, _) = false) |
| 일반 HWPX (BinData/ 내장) | 무영향 (is_embedded = true → Embedding 유지) |
| HWP3 → HWP5 변환본 (Link) | **시각 정합 (목표)** |
| HWP3 → HWPX 변환본 (isEmbeded="0") | **시각 정합 (목표)** |
| HWP3 원본 | 무영향 (HWP3 파서가 직접 external_path 설정) |
| Chart bin_data_id (60000+) | 무영향 (range 분리) |

## B.4 임시 진단 print 제거

`src/main.rs` 의 Stage A 진단용 BinData dump 출력 제거 완료 — diff 0 영향.

## Stage B 결론

HWP5 + HWPX 변환본 의 외부 참조 image 가 같은 dir 영역 자동 load + 시각 정합. 1247 테스트 회귀 0, clippy 경고 0.

📋 **Stage B 완료. Stage C 종합 보고서 + 커밋 진행 승인 요청드립니다.**
