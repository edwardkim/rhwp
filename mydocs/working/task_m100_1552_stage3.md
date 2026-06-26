# Task #1552 Stage 3 완료보고서 — C3(페이지수 복원) + C4(CFB 구조)

## 목표
F2(페이지 붕괴) 부분 게이트화 + CFB 구조 회귀 봉인.

## 변경 사항 (`hwp5_roundtrip_batch.rs`)
- `page_count_of(bytes) -> Option<u32>`: `DocumentCore::from_bytes` → `page_count()`.
  배치 중 단일 파일 패닉 격리 위해 `catch_unwind`(AssertUnwindSafe). 실패/패닉 시 `None`.
- `cfb_structure_ok(out, expected_sections)`: `CfbReader::open` 후 필수 스트림
  (`/FileHeader`,`/DocInfo`,`/BodyText/Section0`) 존재 + `section_count == IR 섹션 수`.
- `RoundtripRow`: `page_before`/`page_after`/`cfb_struct_ok`/`cfb_problems` 추가.
  - `page_mismatch()`: 양쪽 Some 이고 다를 때만 true(한쪽 None 은 미검출).
  - 상태 `CFB_STRUCT_FAIL`(C4) / `PAGE_DIFF`(C3) 추가, `is_hard_fail` 편입.
- TSV 컬럼 추가(`page_before`,`page_after`,`cfb_struct_ok`,`cfb_problems`), summary 2행, 콘솔 `pg=a→b`.

## 검증
- `cargo build --release` 성공.
- `cargo test --lib hwp5_roundtrip`: **14 passed** (신규 `page_mismatch_only_when_both_present_and_differ`, `cfb_structure_rejects_non_cfb`).
- 스모크:
  - KTX `PASS` (page_before==page_after==27, PAGE_DIFF 없음 — serialize_document 페이지 보존 재확인)
  - interview `BINDATA_LOSS bin_lost=1/3` (유지)
  - exam_kor / business_overview / pic-in-table-01 `PASS`
  - **오탐 없음**: CFB_STRUCT_FAIL·PAGE_DIFF 미발생.

## C3 한계 (명시)
C3 는 rhwp 자기 재로드(`DocumentCore::from_bytes`) 기준이라 **rhwp 가 자기일관**인
경우(예: KTX 가 외부 한글에서만 27→1 붕괴하는 convert 경로 결함)는 자동 미검출된다.
serialize_document 경로의 페이지 변화는 검출하나, 외부 한글-only divergence 는
`output/poc/fidelity/` 한글 harness(T3/T4)가 보조한다.

## 다음 단계
Stage 4 — 회귀 테스트(`tests/hwp5_roundtrip_baseline.rs`) + `samples/*.hwp` 전수 등급화(XFAIL) + 매뉴얼 + 최종 보고서.
