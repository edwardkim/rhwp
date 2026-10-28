# Task #1552 Stage 4 완료보고서 — 회귀 테스트 + 전수 등급화 + 매뉴얼 + 최종 보고

## 목표
게이트를 `samples/*.hwp` 전수 회귀로 고정하고 문서화 + 최종 보고.

## 변경 사항
- `src/diagnostics/hwp5_roundtrip_batch.rs`: `pub fn baseline_check(bytes) -> Result<(),String>`
  추가 — 게이트 C1~C5 로직의 단일 출처(인메모리). 회귀 테스트가 재사용.
- 신규 `tests/hwp5_roundtrip_baseline.rs`:
  - `baseline_all_samples_roundtrip`(≤3MB) / `baseline_large_samples_roundtrip`(>3MB) — 신규 샘플 자동 포함
  - `xfail_entries_still_fail` — xfail 통과 시 승격 강제
  - `out_of_scope()`: `detect_format != Hwp` (HWP3) + `header.distribution`(배포용) **자동 제외**
  - `XFAIL` 9건(F1 BinData 드롭, 사유 동반)
- 신규 `mydocs/manual/hwp5_roundtrip_baseline.md`
- 신규 `mydocs/report/task_m100_1552_report.md`

## 전수 측정 (`samples/*.hwp` 319건, 27.5s)
| 분류 | 건수 |
|------|----:|
| A (PASS) | 297 |
| B (xfail) F1 BinData 드롭 | 9 |
| 자동 제외 HWP3 | 10 |
| 자동 제외 배포용 | 3 |

## 검증
- `cargo test --test hwp5_roundtrip_baseline`: **3 passed** (18s) — baseline 통과 + xfail 여전히 실패.
- `cargo test --lib hwp5_roundtrip`: 14 passed (단위).
- `cargo build --release` 성공.

## 결론
HWP5 저장 무손실 게이트 완비. F1(serialize_document BinData 드롭) 9건을 xfail 로 봉인,
HWP3·배포용은 자동 제외. 선행 조사의 F2(KTX 페이지 붕괴)는 convert 경로 결함으로 분리 규명.
후속 이슈 후보(F1·F2'·F3)는 최종 보고서에 정리.
