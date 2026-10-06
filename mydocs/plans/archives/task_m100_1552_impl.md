# Task #1552: HWP5 roundtrip 무손실 게이트 — 구현 계획서

> 수행계획서: `task_m100_1552.md` (승인 완료). 본 문서는 단계별 구현 계획(4단계).
> 미러 대상: `src/diagnostics/hwpx_roundtrip_batch.rs`(577줄) + `tests/hwpx_roundtrip_baseline.rs`.

## 재사용 확정 API (조사 완료)

| API | 위치 | 용도 |
|-----|------|------|
| `parser::parse_document(&[u8]) -> Result<Document>` | 파서 진입 | doc1/doc2/doc3 파싱 |
| `serializer::serialize_document(&Document) -> Result<Vec<u8>>` | `serializer/mod.rs:78` | HWP5 저장(= `export_hwp_native`) |
| `diff_documents(&Document, &Document) -> IrDiff` | `serializer/hwpx/roundtrip.rs:427` | **포맷 무관** IR 뼈대 비교(C1·C5) |
| `CfbReader::{list_bin_data, read_bin_data}` | `parser/cfb_reader.rs` | BinData 스트림 열거·raw 읽기(C2) |
| `decompress_stream(&[u8]) -> Result<Vec<u8>>` | `parser/cfb_reader.rs:640` | BinData raw deflate 해제(C2) |
| `CfbReader::{section_count, list_streams}` | `parser/cfb_reader.rs` | CFB 구조 검사(C4) |
| `DocumentCore::{from_bytes, page_count, serialize_hwp_with_verify}` | `document_core/...` | 페이지수 복원(C3) |

---

## Stage 1 — 모듈 골격 + C1(IR diff) + C5(2-round) + CLI 연결

**목표**: `hwpx-roundtrip`을 미러한 `hwp5-roundtrip` 최소 동작(단일/배치/`-o`/`inventory.tsv`/종료코드).

- 신규 `src/diagnostics/hwp5_roundtrip_batch.rs`:
  - `Options`/`parse_args`(단일 파일 | `--batch <dir>` | `-o <out>`), `collect_hwp5_files`(재귀 `*.hwp`, ViewText/배포용 등은 포함하되 파싱 실패는 상태로 기록)
  - `roundtrip_one`: `parse_document`(doc1) → `serialize_document` → `parse_document`(doc2) → `diff_documents(doc1,doc2)` → 2-round(serialize→doc3, `diff_documents(doc2,doc3)`)
  - `RoundtripRow` + `status()`/`is_hard_fail()` (상태 우선순위: `PARSE_FAIL→SERIALIZE_FAIL→REPARSE_FAIL→IR_DIFF→ROUND2_FAIL→ROUND2_DIFF→PASS`)
  - `write_tsv`/`print_summary`/`rt_output_path`(`{stem}.rt.hwp`)
- `src/diagnostics/mod.rs`: `pub mod hwp5_roundtrip_batch;`
- `main.rs`: `Some("hwp5-roundtrip") => rhwp::diagnostics::hwp5_roundtrip_batch::run(&args[2..])` + `--help` 항목
- 단위 테스트: `parse_args`(단일/배치/거부), `rt_output_path`, blank 샘플 PASS (hwpx 테스트 미러)

**검증**: `rhwp hwp5-roundtrip samples/business_overview.hwp` PASS, `--batch`로 `inventory.tsv` 생성. `cargo test` 회귀 없음.
**산출**: `task_m100_1552_stage1.md` + 소스 커밋.

## Stage 2 — C2 BinData 스트림 보존 검사

**목표**: F1(이미지 드롭) 게이트화.

- `bindata_fingerprint(bytes: &[u8]) -> BTreeMap<해시, count>`:
  - `CfbReader`로 BinData 열거 → 각 스트림 raw 읽기 → `decompress_stream` 시도, 실패 시 raw 사용(압축 플래그 무관 일관)
  - **decompressed 내용**의 멀티셋(내용 해시→개수). 이름(BIN0001 등)은 재명명 가능성 있어 **내용 기준 비교**(hwpx `check_package` 정신과 동일)
- `roundtrip_one`에 orig bytes vs rt bytes fingerprint 비교 → `bindata_lost`(드롭 수)/`bindata_total` 기록
- TSV 컬럼 추가(`bindata_total`, `bindata_lost`), 상태에 `BINDATA_LOSS`(IR_DIFF와 동급 hard-fail) 추가
- 단위 테스트: 이미지 포함 소형 샘플로 보존 PASS, 인위적 드롭 검출

**검증**: KTX(3/3)·interview(1/3)·Worldcup(13/47)이 `BINDATA_LOSS`로 검출, exam_kor(재압축만)은 보존 PASS.
**산출**: `task_m100_1552_stage2.md` + 커밋.

## Stage 3 — C3(페이지수 복원) + C4(CFB 구조)

**목표**: F2(페이지 붕괴) 부분 게이트화 + 구조 회귀 봉인.

- C3: `DocumentCore::from_bytes(orig)`로 `page_before`, rt bytes로 `page_after` 비교(또는 `serialize_hwp_with_verify` 활용). `page_before/page_after`/`page_recovered` 기록. 불일치 시 `PAGE_DIFF`.
  - **한계 명시**: rhwp 자기 일관 기준이라 KTX형(외부 한글에서만 27→1) 일부는 자동 미검출 가능 — 보고서·매뉴얼에 기재, 한글 harness 보조.
- C4: rt CFB 필수 스트림(`FileHeader`,`DocInfo`,`BodyText/Section{0..}`) 존재 + `section_count(rt) == doc.sections.len()` 검사. 불일치 시 `CFB_STRUCT_FAIL`.
- TSV 컬럼 추가, 상태 우선순위에 PAGE_DIFF·CFB_STRUCT_FAIL 편입
- 단위 테스트: 정상 샘플 page_recovered=true·구조 OK

**검증**: 정상 대조군 PASS, page 불일치 샘플 검출(있으면).
**산출**: `task_m100_1552_stage3.md` + 커밋.

## Stage 4 — 회귀 테스트 + 전수 등급화 + 매뉴얼 + 최종 보고

**목표**: 게이트를 `samples/*.hwp` 전수 회귀로 고정 + 문서화.

- `tests/hwp5_roundtrip_baseline.rs`(hwpx baseline 미러):
  - `baseline_all_samples_roundtrip`(전수 재귀, XFAIL/EXCLUDED 제외, 신규 샘플 자동 포함)
  - `xfail_entries_still_fail`(XFAIL 승격 강제), `grade_lists_are_consistent`
  - `--batch samples` 1회 실행 결과로 **현 손실을 사유와 함께 XFAIL 등록**(F1: KTX/interview/Worldcup 등, F2: 발견분). EXCLUDED: 비-HWP5/손상.
  - 대형 샘플 분리(`LARGE`)로 wall time 관리(hwpx 동형)
- `mydocs/manual/hwp5_roundtrip_baseline.md`: 사용법·등급체계·C1~C5·한계(C3 외부 divergence)
- 최종 결과보고서 `mydocs/report/task_m100_1552_report.md` + 후속 이슈 후보(F1/F2/F3 수정) 정리

**검증**: `cargo test --test hwp5_roundtrip_baseline` 통과(XFAIL 정합). `cargo test` 전체 회귀 없음. `cargo clippy` 클린(신규 파일 범위).
**산출**: `task_m100_1552_stage4.md` + `_report.md` + 커밋.

---

## 공통 주의

- HWP3 전용 분기 추가 금지(CLAUDE.md). 게이트는 HWP5 파서/직렬화·CFB만 사용.
- `inventory.tsv` 컬럼은 단계별로 **추가만**(기존 컬럼 의미 불변).
- 한글 OCX 오라클은 게이트 본체 미포함 — `output/poc/fidelity/` harness가 보조.
- 단계마다 완료보고서(`_stage{N}.md`) + 소스 동반 커밋, 승인 후 다음 단계.
- 기능/포맷 변경 미혼합. 신규 파일 위주라 무관 rustfmt diff 미발생.
