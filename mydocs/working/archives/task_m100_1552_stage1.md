# Task #1552 Stage 1 완료보고서 — 모듈 골격 + C1(IR diff) + C5(2-round) + CLI 연결

## 목표
`hwpx-roundtrip`을 미러한 `hwp5-roundtrip` 최소 동작(단일/배치/`-o`/`inventory.tsv`/종료코드) 구현.

## 변경 사항
- 신규 `src/diagnostics/hwp5_roundtrip_batch.rs`
  - `parse_document`(doc1) → `serialize_document` → `parse_document`(doc2) → `diff_documents`(C1)
  - 2-round: serialize(doc2)→doc3, `diff_documents(doc2,doc3)` (C5)
  - `RoundtripRow.status()`: `PARSE_FAIL→SERIALIZE_FAIL→REPARSE_FAIL→IR_DIFF→ROUND2_FAIL→ROUND2_DIFF→PASS`
  - 단일/`--batch <dir>`/`-o`, `inventory.tsv`(11컬럼), `print_summary`, `{stem}.rt.hwp` 산출, 하드 실패 시 종료 코드 1
  - `collect_hwp5_files`: `.hwp`만 수집(`.hwpx` 제외)
- `src/diagnostics/mod.rs`: `pub mod hwp5_roundtrip_batch;` 등록
- `src/main.rs`: `Some("hwp5-roundtrip") => ...` 분기 + `--help` 3행 추가

## 재사용 API
`parse_document`, `serialize_document`(=`export_hwp_native`), `diff_documents`(포맷 무관, `serializer/hwpx/roundtrip.rs:427`).

## 검증
- `cargo build --release`: 성공 (기존 bin/lib 동명 note 외 경고 없음)
- `cargo test --release --lib hwp5_roundtrip`: **10 passed; 0 failed**
- 스모크:
  - `business_overview.hwp` → `PASS diff=0 r2=0`
  - `KTX.hwp` → `PASS diff=0 r2=0` (이미지 드롭은 C2/Stage 2에서 검출 예정 — Stage 1 범위 밖, 정상)

## 다음 단계
Stage 2 — C2 BinData 스트림 보존 검사(decompressed 내용 멀티셋)로 F1(이미지 드롭) 게이트화.
