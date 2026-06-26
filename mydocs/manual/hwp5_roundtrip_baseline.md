# HWP5 Roundtrip Baseline 가이드 (Task #1552)

`samples/*.hwp` 전수에 대한 HWP5→IR→HWP5 roundtrip **무손실** 검증 체계의 사용·유지보수 매뉴얼.
`hwpx-roundtrip`(Task #1315)의 HWP5 대응 게이트.

## 1. 개요

`serialize_document`(= `export_hwp_native`, HWP5 "저장하기")의 **무손실성**을 회귀 게이트로 고정한다.
검사 항목(C1~C5):

| # | 검사 | 방법 | 잡는 결함 |
|---|------|------|-----------|
| C1 | IR 뼈대 diff | `parse → serialize → 재parse` 후 `diff_documents` == 0 | 구조 손실 |
| C2 | **BinData 보존** | 원본·저장본 CFB의 BinData **decompressed 내용** 멀티셋 동일 | **그림 스트림 드롭(F1)** |
| C3 | 페이지수 복원 | `DocumentCore::from_bytes` 페이지 수 원본==저장본 | 페이지 변형 |
| C4 | CFB 구조 | 필수 스트림(FileHeader/DocInfo/BodyText/Section0) + 섹션 수 = IR | 구조 회귀 |
| C5 | 2-round 안정성 | 저장본 재직렬화→재parse 후 IR diff == 0 | 비결정성 |

> **중요**: 통과 = 구조+BinData+페이지(rhwp 자기 일관) 보존이며 **시각 충실도 보장이 아니다**.
> C3 는 rhwp 자기 재로드 기준이라 **외부 한글에서만** 나타나는 페이지 붕괴(예: `convert`/
> `convert_to_editable` 경로)는 자동 검출하지 못한다. 시각·외부 divergence 판정은
> 작업지시자(한컴에디터)와 `output/poc/fidelity/` 한글 harness(T3 재열림·T4 PDF)가 보조한다.

## 2. 등급 체계

| 등급 | 의미 | 코드 위치 |
|------|------|----------|
| **A (baseline)** | C1~C5 전부 통과. 신규 HWP5 샘플 자동 포함 | `tests/hwp5_roundtrip_baseline.rs` 기본 대상 |
| **B (xfail)** | 식별된 결함으로 baseline 제외. 사유 필수 | `XFAIL` 상수 |
| **자동 제외** | HWP5 아님(HWP3/HWPML) 또는 배포용 문서 — serializer 결함 아님 | `out_of_scope()` (포맷·distribution 감지) |

현황 (2026-06-26, `samples/*.hwp` 319건):
- **A=297, B(xfail)=9**, 자동 제외=13 (HWP3 10 + 배포용 3).
- B(xfail) 9건은 전부 `serialize_document`의 **BinData 그림 스트림 드롭(F1)**:
  `img-start-001`(20/20), `BookReview`(7/10), `Worldcup_FIFA2010_32`(13/47),
  `exam_social`(2/8), `NewYear_s_Day`(2/4), `곡선이있는분산형`(2/3), `pic-crop-01`(2/3),
  `interview`(1/3), `BlogForm_Recipe`(1/3). 후속 이슈에서 serializer 수정 시 승격.

> **자동 제외 근거**: HWP3(`HWP Document File V3.00`)는 별도 포맷이라 HWP5 직렬화 시
> 교차변환(페이지 폭증)된다. 배포용 문서는 `serialize_document` 직접 적용 시
> `DISTRIBUTE_DOC_DATA` 누락으로 재파싱 실패하나, 정상 경로는 `convert_to_editable`
> 선행이므로 게이트 범위 밖이다(별도 이슈로 분리).

## 3. 통합 테스트 (`tests/hwp5_roundtrip_baseline.rs`)

```bash
cargo test --release --test hwp5_roundtrip_baseline
```

| 테스트 | 역할 |
|--------|------|
| `baseline_all_samples_roundtrip` | 소형(≤3MB) 전수 — **신규 샘플 자동 포함** |
| `baseline_large_samples_roundtrip` | 대형(>3MB) 분리 — 하네스 병렬로 wall time 단축 |
| `xfail_entries_still_fail` | xfail 이 통과하면 실패 → baseline 승격 강제 |

### 신규 샘플 추가 시
`samples/` 에 `.hwp` 추가 시 자동으로 baseline 게이트에 포함된다.
- 통과 → 끝 (A등급)
- 실패 → 결함 수정하거나 **사유와 함께** `XFAIL` 등록(사유 없는 등록 금지)
- HWP3/배포용 → `out_of_scope()` 가 자동 제외(목록 불필요)

### xfail 승격 절차
serializer 결함(F1 등) 해소 시 `xfail_entries_still_fail` 가 실패한다.
해당 항목을 `XFAIL` 에서 제거하면 baseline 으로 자동 승격된다.

## 4. 배치 CLI (`rhwp hwp5-roundtrip`)

```bash
rhwp hwp5-roundtrip sample.hwp                          # 단일 파일 검사
rhwp hwp5-roundtrip --batch samples                     # 폴더 전수 (재귀)
rhwp hwp5-roundtrip --batch samples -o output/poc/task1552   # 산출물 지정
```

- 산출물: `{out}/inventory.tsv`(17컬럼) + `{out}/{stem}.rt.hwp`(재조립 파일)
- 상태 우선순위: `PARSE_FAIL → SERIALIZE_FAIL → REPARSE_FAIL → IR_DIFF → BINDATA_LOSS → CFB_STRUCT_FAIL → PAGE_DIFF → ROUND2_FAIL → ROUND2_DIFF → PASS`
- 하드 실패(파싱/직렬화/재파싱/BinData/구조/페이지/2-round) 존재 시 종료 코드 1 (CI 사용 가능)
- `inventory.tsv` 컬럼: sample, status, parse/serialize/reparse_ok, ir_diff_count,
  bindata_total, bindata_lost, page_before, page_after, cfb_struct_ok, round2_diff,
  elapsed_ms, error, ir_diff_summary, cfb_problems, round2_error.

## 5. Known limitations / 후속

| 한계 | 증상 | 후속 |
|------|------|------|
| BinData 그림 스트림 드롭 | `serialize_document` 가 일부 그림 누락 (xfail 9건) | F1 수정 이슈 |
| convert 경로 추가 손실 | `convert_to_editable`/`convert` 경유 시 이미지·페이지 손실(KTX 27→1) — 본 게이트(serialize_document 직접) 범위 밖 | F2' 별도 이슈 |
| C3 외부 divergence | rhwp 자기일관 페이지는 보존이나 외부 한글에서만 붕괴하는 경우 미검출 | 한글 harness 보조 |

## 6. 관련 문서
- 수행/구현 계획: `mydocs/plans/task_m100_1552{,_impl}.md`
- 단계별 보고서: `mydocs/working/task_m100_1552_stage{1..4}.md`
- 최종 보고서: `mydocs/report/task_m100_1552_report.md`
- 선행 조사(한글 4단계 오라클): `output/poc/fidelity/report.md`
