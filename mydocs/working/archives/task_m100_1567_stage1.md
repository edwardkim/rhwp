# Task #1567 Stage 1 완료보고서 — 근본원인 확정

## 결과
표 셀 pic 드롭의 3단계 메커니즘 확정:

1. **빈 binaryItemIDRef**: 원본 셀 pic 다수가 `<hc:img binaryItemIDRef="">`(이미지 참조 없는
   placeholder). 실증 36385464: `section0.xml` hp:pic 2개 중 하나가 빈 ref, 하나가 "image1".
2. **파서 매핑 0**: 빈/비수치 ref → `bin_data_id=0`. 파서 패턴 `num.parse().unwrap_or(0)`
   (`src/parser/hwpx/header.rs:1486` 동형). `bin_data_id: u16`(`src/model/image.rs:71`).
3. **serialize 실패 + 드롭**:
   - `write_img`(`src/serializer/hwpx/picture.rs:255`) `resolve_bin_id(0)→None`→`Err`.
   - `section.rs:701`(및 Shape::Picture `1173`) 가 에러를 로그만 찍고 pic 드롭 → IR_DIFF.

## 확정
- `bin_data_id==0` ⟺ 빈 binaryItemIDRef (HWPX bin id 1-based, 0=무참조).
- 따라서 `bin_id==0` 시 `binaryItemIDRef=""` 방출하면 `"" → 0 → ""` roundtrip 보존.
- 비-0 미해결(진짜 BinDataContent 누락)은 별개 — 진단(Err) 유지해야 손실 은폐 방지.

## 수정 지점 (Stage 2)
`src/serializer/hwpx/picture.rs` `write_img:255` — `resolve_bin_id` 실패 시 `bin_id==0` 이면
`binaryItemIDRef=""` 방출, 비-0 은 종전 Err.

## 다음
Stage 2 — 수정 + 대표 케이스 roundtrip diff 해소 검증 + 단위 테스트.
