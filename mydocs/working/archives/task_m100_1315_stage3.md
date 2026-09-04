# Task M100 #1315 — 3단계 완료 보고서

## 단계 목표

샘플 등급화 (A=baseline / B=xfail / C=oracle 부적합) + 전수 통합 테스트 추가.

## 등급표 (samples/hwpx, .hwpx 54개)

### A등급 — baseline pass: 52건

roundtrip + IrDiff 0 + 패키지 검사 + 2-round 안정성 전부 통과. 전체 목록은 `output/poc/task1315/inventory.tsv` 와 일치 (2단계 측정 기준).

대표 분류:

| 묶음 | 샘플 |
|------|------|
| 참조 최소 | ref/ref_empty, ref/ref_text, ref/ref_mixed, ref/ref_table, blank_hwpx |
| 문단/서식 | para-001, para-unit-01, table-text, landscape-001, water-mark |
| 표 | basic-table-01, tb-org-02, tb-img-03 |
| 각주/미주/글상자 | footnote-01, footnote-tbox-01, tbox-v-flow-01 |
| 그림/도형/수식 | ta-pic-001-r, shape-001, math-001, eq-002 |
| 양식 | form-01, form-02, form-002 |
| 실문서(복합) | 보도자료 5종, [2027] 온새미로, aift, exam_kor, exam-kor-1p~4p, k-water-rfp, mel-001, hcar-001, hwpx-h-01~03, hy-001/002 등 |

### B등급 — xfail: 1건

| 샘플 | 사유 |
|------|------|
| `exam_social.hwpx` | serializer: 미등록 `borderFillIDRef 31` — parser→serializer ID 매핑 경계 문제. 별도 이슈 등록 후보 |

### 검사 제외: 1건

| 샘플 | 사유 |
|------|------|
| `hwpx-01.hwpx` | HWP5(OLE CFB)가 `.hwpx` 확장자로 저장된 샘플 — serializer 결함 아님. 샘플 정비 여부는 작업지시자 판단 요청 |

### C등급 — 시각 oracle 부적합 표시: 13건 (A등급의 부분집합)

baseline(구조)은 통과하지만 full visual fidelity oracle 로 쓰면 안 되는 복합 실문서 (이슈 주의사항 반영):
보도자료 5종, [2027] 온새미로 1 본교재, aift, exam_kor, exam-kor-1p~4p, k-water-rfp.

## 구현 내역

`tests/hwpx_roundtrip_baseline.rs` 신규 — 테스트 4건:

| 테스트 | 내용 |
|--------|------|
| `baseline_all_samples_roundtrip` | 전수 스캔(재귀) — XFAIL/EXCLUDED/LARGE 제외 전부 baseline 검사. **신규 샘플 자동 포함** — 통과 못 하면 사유와 함께 XFAIL 등록 강제 |
| `baseline_large_samples_roundtrip` | 대형 3건(exam_kor, aift, k-water-rfp) 분리 — 하네스 병렬 실행으로 wall time 단축 (86s → 56s) |
| `xfail_entries_still_fail` | xfail 이 통과하게 되면 실패 → baseline 승격 강제 (실패 소멸 감지) |
| `grade_lists_are_consistent` | EXCLUDED/ORACLE_UNFIT 실재 + ORACLE_UNFIT ⊂ baseline 가드 (목록 부패 방지) |

baseline 검사 = parse → serialize → 재parse + IrDiff 0 + `check_package` + 2-round IrDiff 0 (배치 CLI와 동일 기준). 샘플은 `include_bytes!` 대신 경로 로드 (계획서 위험 대응).

기존 `tests/hwpx_roundtrip_integration.rs` 무수정 (누적-만-가능 원칙).

## 검증 결과

- `cargo test --test hwpx_roundtrip_baseline` — 통과, 4 passed (debug 56.4s)
- `cargo clippy --tests -- -D warnings` — 통과
- `cargo fmt --check` — 통과

## 판단 요청

1. C등급(ORACLE_UNFIT) 13건 목록이 적절한지 — 시각 판정 권위는 작업지시자에게 있으므로 목록 확정 요청
2. `hwpx-01.hwpx` 샘플 정비(이름 변경/이동) 여부 — 본 타스크 범위 외로 두고 현상 유지 제안

## 다음 단계

4단계 — 대표 샘플 5~8개 roundtrip 산출 + SVG 비교 자료 + rhwp-studio 확인 + 한컴에디터 판정 요청.
