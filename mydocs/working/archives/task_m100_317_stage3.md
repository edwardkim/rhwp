# Task #317 3단계 완료 보고서: 보강 코드 + 회귀 검증

상위: `task_m100_317_impl.md`
선행: `task_m100_317_stage2.md` (origin = `table.attr` bit 0 합성으로 인한 typeset 분기 분기)

## 변경

### `src/document_core/converters/hwpx_to_hwp.rs::adapt_table`

raw_ctrl_data 합성 직후 attr 영역(offset 0..4)을 0으로 강제하고 `table.attr=0` 보존.

```rust
fn adapt_table(table: &mut Table, report: &mut AdapterReport) {
    if table.raw_ctrl_data.is_empty() {
        table.raw_ctrl_data = serialize_common_obj_attr(&table.common);
        report.tables_ctrl_data_synthesized += 1;

        // Task #317: HWPX 출처는 common.attr=0 이 진실. typeset 엔진은
        // table.attr & 0x01 로 is_tac 판정 (common.treat_as_char 아님).
        // attr 합성은 RELOADED 만 TAC 분기로 흘려보내 페이지 누적 차이 (+2.7px/표) 발생.
        if table.raw_ctrl_data.len() >= 4 {
            table.raw_ctrl_data[0..4].copy_from_slice(&0u32.to_le_bytes());
        }
    }
    table.attr = 0;
    // ... (이하 기존 셀 처리)
}
```

기존 "attr 동기화" 블록 제거 (raw_ctrl_data 의 합성 attr 을 table.attr 에 복사하던 로직 — 이번 보강 의도와 정반대).

## 검증

### 어댑터 격리 테스트 (이전 #313 후 격리됨)

```
cargo test --test hwpx_to_hwp_adapter -- --include-ignored
test result: ok. 25 passed; 0 failed; 0 ignored
```

3건 (`stage4_page_count_recovered_hwpx_h_02`, `stage5_all_three_samples_recover_via_unified_entry_point`, `stage6_verify_recovered_for_all_three_samples`) 모두 통과.

### 4샘플 무회귀 (release 빌드 dump-pages)

| 샘플 | 기대 | 측정 |
|------|------|------|
| 21_언어_기출_편집가능본 | 15쪽 | 15쪽 ✓ |
| exam_math | 20쪽 | 20쪽 ✓ |
| exam_kor | 24쪽 | 24쪽 ✓ |
| exam_eng | 9쪽 | 9쪽 ✓ |

### 전체 테스트

```
cargo test → 992 lib + 모든 통합 테스트 PASS (이전과 동일)
```

## 영향 범위

- HWPX 출처 표만 영향. `table.raw_ctrl_data` 가 이미 채워진 HWP 출처 표는 분기 비진입.
- HWPX 파서가 `common.treat_as_char/text_wrap/vert_rel_to` 등 enum 필드를 채우므로, 직렬화 후 HWP 파서가 attr 비트를 0으로 보더라도 enum 필드는 정상 복원되지 않을 가능성 — 단, typeset 은 `table.attr` 만 보고 `common.text_wrap/treat_as_char` 미사용이므로 typeset 결과 영향 없음.
- 다른 `common.*` 사용 코드 (예: 캡션/외곽선/렌더러) 는 `table.common.*` 를 직접 읽으므로 영향 없음.

## 산출

- `src/document_core/converters/hwpx_to_hwp.rs` (수정)
- 본 보고서

## 다음 단계

4단계: `tests/hwpx_to_hwp_adapter.rs` 의 `#[ignore]` 3건 제거 + 진단 도구 정리 (`tests/task317_diag.rs` 삭제, `src/renderer/typeset.rs` 의 `RHWP_TYPESET_TRACE` 제거) + 최종 보고서.
