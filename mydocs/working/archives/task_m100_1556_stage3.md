# 단계 3 완료보고서 — Task #1556

## 목표
합성 parse→serialize→parse roundtrip 테스트 + 대표 실문서 회귀 샘플 추가 (둘 다).

## 변경 사항

### 합성 roundtrip 테스트 (`src/serializer/hwpx/section.rs`)
- `task1556_multipara_field_parse_serialize_parse_roundtrip`:
  다단락 필드(begin=문단0, end=문단1) 합성 section → `parse_hwpx_section` →
  `write_section` → 재`parse_hwpx_section`. end 문단의
  `text`/`char_count`/`char_offsets`/`char_shapes`/`orphan_field_ends` 전부 보존 검증
  (= IR diff=0). 8유닛 소실 없음.

### 실문서 회귀 샘플 (`samples/hwpx/`)
- `field-multipara-clickhere.hwpx` 추가 (서울 열린데이터 공개 행정문서, ≈39KB).
  - 원본: `…고시원 안전시설…건축법령 확인요청 dt2854.hwpx`.
  - 다단락 CLICK_HERE(누름틀 "본문") 필드 — fieldBegin(표 뒤) ~ fieldEnd(말미 "…끝.")
    가 ~18문단 가로지름. 수정 전 `hwpx-roundtrip` diff=1(문단 0.16 cc 38→30).
  - 파일 권한 644 정규화.
- `tests/hwpx_roundtrip_baseline.rs` 의 `collect_samples` 가 신규 샘플 자동 포함
  → `baseline_all_samples_roundtrip` 게이트에 봉인. (XFAIL/EXCLUDED 미등록 = 통과 필수.)

## 검증
- `cargo test --lib task1556`: **5건 통과** (파서2 + 직렬화3, 합성 roundtrip 포함).
- `rhwp hwpx-roundtrip samples/hwpx/field-multipara-clickhere.hwpx` → **diff=0 PASS**.
- `cargo test --test hwpx_roundtrip_baseline` (신규 샘플 자동 포함) **무회귀 통과**.

## 회귀 가드 성격
수정을 되돌리면 `field-multipara-clickhere.hwpx` 의 roundtrip 이 diff=1 로 회귀하여
`baseline_all_samples_roundtrip` 가 실패 → 결함 재유입을 차단.

## 다음 단계
단계 4 — 코퍼스 다건 전수 검증 + 전체 `cargo test` + fmt/clippy + 최종 보고서.
