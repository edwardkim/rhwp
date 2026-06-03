---
타스크: #178 HWPX→HWP IR 매핑 어댑터
단계: Stage 1 — 진단 인프라 + 자동 차이 추출 도구
브랜치: local/task178
작성일: 2026-04-19
선행: mydocs/plans/task_m100_178_impl.md (구현계획서, 승인됨)
---

# Stage 1 단계별 완료 보고서

## 1. 목표 (구현계획서 §2 Stage 1)

매핑이 실패한 영역을 자동 식별하는 도구 + 페이지 폭주 베이스라인 측정.

## 2. 산출물

### 2.1 신규 파일

| 파일 | 역할 |
|---|---|
| [src/document_core/converters/mod.rs](src/document_core/converters/mod.rs) | 어댑터 모듈 노출 |
| [src/document_core/converters/hwpx_to_hwp.rs](src/document_core/converters/hwpx_to_hwp.rs) | 어댑터 진입점 (Stage 1 은 no-op + idempotent 골격) + `AdapterReport` |
| [src/document_core/converters/diagnostics.rs](src/document_core/converters/diagnostics.rs) | `IrFieldDiff` + `DiffSummary` + `diff_hwpx_vs_serializer_assumptions` + `diff_hwpx_vs_hwp` |
| [src/document_core/converters/common_obj_attr_writer.rs](src/document_core/converters/common_obj_attr_writer.rs) | `CommonObjAttr` → ctrl_data 바이트 작성기 (Stage 2 placeholder) |
| [examples/hwpx_hwp_ir_diff.rs](examples/hwpx_hwp_ir_diff.rs) | 진단 CLI |
| [tests/hwpx_to_hwp_adapter.rs](tests/hwpx_to_hwp_adapter.rs) | 통합 테스트 6건 |

### 2.2 수정 파일

| 파일 | 변경 |
|---|---|
| [src/document_core/mod.rs](src/document_core/mod.rs) | `pub mod converters;` 1줄 추가 |

### 2.3 HWP 직렬화기 변경

**0줄 수정** — 본 타스크 정체성 (구현계획서 §1.0.1) 준수.

## 3. 검증 결과

### 3.1 단위 테스트 (6개)

```
test document_core::converters::common_obj_attr_writer::tests::stage1_returns_empty_placeholder ... ok
test document_core::converters::diagnostics::tests::empty_doc_no_diff_items_for_critical_areas ... ok
test document_core::converters::diagnostics::tests::human_report_includes_total ... ok
test document_core::converters::hwpx_to_hwp::tests::stage1_entry_point_returns_default_report ... ok
test document_core::converters::hwpx_to_hwp::tests::stage1_hwp_source_no_op ... ok
test document_core::converters::hwpx_to_hwp::tests::stage1_hwpx_source_runs_adapter ... ok
```

### 3.2 통합 테스트 (6개)

```
test adapter_skips_hwp_source ... ok
test adapter_idempotent_no_op_in_stage_1 ... ok
test baseline_diff_inventory_hwpx_h_01 ... ok
test baseline_page_count_explosion_hwpx_h_01 ... ok
test baseline_page_count_explosion_hwpx_h_02 ... ok
test baseline_page_count_explosion_hwpx_h_03 ... ok
```

### 3.3 회귀 (전체 라이브러리)

```
test result: ok. 881 passed; 0 failed; 1 ignored; 0 measured
```

## 4. 베이스라인 측정 결과

### 4.1 페이지 폭주 (현재, 어댑터 미적용)

| 샘플 | 원본 페이지 | HWP 저장 후 재로드 | 폭주 비율 |
|---|---:|---:|---:|
| hwpx-h-01 | 9 | 200 | 22.2× |
| hwpx-h-02 | 9 | 220 | 24.4× |
| hwpx-h-03 | 9 | 224 | 24.9× |

> 트러블슈팅 문서 (`task178_hwpx_to_hwp_first_attempt_failure.md`) 시점 측정값 (hwpx-h-01: 9→209, hwpx-h-02: 6→155) 과 차이가 있음.
> 사유: 그 사이 #177 (lineseg 보정) 머지로 원본 페이지 수 자체가 변동. **현 측정값이 새 베이스라인**.

### 4.2 진단 도구 검출 결과 (hwpx-h-01)

```
[IR diff summary] total=52
  table.raw_ctrl_data: 26
  table.raw_table_record_attr: 26
```

→ hwpx-h-01 에는 표가 26개 있고, 모두 `raw_ctrl_data` 합성 + `raw_table_record_attr` 재구성이 필요.
→ **Stage 2 의 우선순위 영역 확인** (예상대로 표 ctrl_data 가 핵심).

## 5. 구현계획서 정체성 셀프 체크

- [x] HWP 직렬화기 0줄 수정 — `git diff src/serializer/` 출력 0줄
- [x] 어댑터는 IR 만 만짐 — `convert_hwpx_to_hwp_ir(&mut Document)` 시그니처
- [x] idempotent — `adapter_idempotent_no_op_in_stage_1` 테스트
- [x] HWP 출처 보호 — `adapter_skips_hwp_source` 테스트
- [x] 매핑 명세는 직렬화기 가정 기준 — `diff_hwpx_vs_serializer_assumptions` 함수명·문서주석에 명시

## 6. 다음 단계

Stage 2: `table.raw_ctrl_data` 합성 + `table.attr` 재구성.

진단 도구 출력으로 Stage 2 가 해결할 영역 26건 확인됨. `common_obj_attr_writer.rs` 본격 작성 시작.

## 7. 승인 요청

본 단계 완료 보고서 승인 후 Stage 2 착수.
