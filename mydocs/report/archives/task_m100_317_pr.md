# PR: Task #317 — HWPX 어댑터 +1쪽 잔존 origin 보강

## 제목

```
Task #317: HWPX 어댑터 +1쪽 잔존 origin 보강 (격리 테스트 3건 재활성화)
```

## 본문

## 배경

#314 (HWPX 어댑터 normalize) 후에도 `hwpx-h-02` 어댑터 라운드트립 결과가 9쪽 → 10쪽으로 잔존하던 +1쪽 차이를 조사하고 보강. #313 의 TypesetEngine 전환 시점에 격리되었던 어댑터 테스트 3건 재활성화.

선행:
- #313 (TypesetEngine main 전환)
- #314 (어댑터 normalize 부분 완료)

## Origin 식별

`src/renderer/typeset.rs:695` 의 `is_tac` 판정이 `table.attr & 0x01` 비트만 보고 `table.common.treat_as_char` 와 비대칭:

| 경로 | `table.attr` | `is_tac` | typeset 분기 |
|------|--------------|----------|--------------|
| DIRECT (HWPX 직접 로드) | `0` | `false` | block (PDF baseline 일치) |
| RELOADED (HWPX→어댑터→HWP→재로드) | `0x002A0211` | `true` | TAC |

HWPX 파서는 `table.attr` 비트를 채우지 않는다. 어댑터의 `pack_common_attr_bits` 가 `common.treat_as_char` 등 enum 으로부터 비트 0 (treat_as_char) 을 합성하면서 RELOADED 만 TAC 분기로 흘려 보냄. block 과 TAC 분기는 host_spacing/outer_margin/host_line_spacing 처리가 달라 같은 표 paragraph 마다 ~2.7px 누적 차이 발생 → 페이지 3 의 표 3개에서 누적 ~+8px → pi=51 (Table 326.7px) 가 가용 공간 부족으로 다음 페이지로 밀림 → +1쪽.

## 변경

### `src/document_core/converters/hwpx_to_hwp.rs::adapt_table`

raw_ctrl_data 합성 직후 attr 영역(offset 0..4)을 0 으로 강제 + `table.attr=0` 보존. DIRECT 와 동일한 block 분기 진입.

```rust
fn adapt_table(table: &mut Table, report: &mut AdapterReport) {
    if table.raw_ctrl_data.is_empty() {
        table.raw_ctrl_data = serialize_common_obj_attr(&table.common);
        report.tables_ctrl_data_synthesized += 1;

        // Task #317: HWPX 출처는 common.attr=0 이 진실. typeset 은 table.attr & 0x01 로
        // is_tac 판정 (common.treat_as_char 아님). attr 합성은 RELOADED 만 TAC 분기로
        // 흘려 보내는 부작용 → 같은 block 분기를 위해 attr=0 강제.
        if table.raw_ctrl_data.len() >= 4 {
            table.raw_ctrl_data[0..4].copy_from_slice(&0u32.to_le_bytes());
        }
    }
    table.attr = 0;
    // (이하 기존 셀 처리)
}
```

기존 "attr 동기화" 블록 (raw_ctrl_data 의 합성 attr → table.attr 복사) 은 본 보강 의도와 정반대이므로 제거.

### 격리 테스트 재활성화 (`tests/hwpx_to_hwp_adapter.rs`)

#313 에서 `#[ignore]` 처리된 3건 제거:
- `stage4_page_count_recovered_hwpx_h_02`
- `stage5_all_three_samples_recover_via_unified_entry_point`
- `stage6_verify_recovered_for_all_three_samples`

### 진단 도구 정리

1단계에서 도입한 도구 회수:
- `src/renderer/typeset.rs` — `RHWP_TYPESET_TRACE` env-gated trace 5블록 제거
- `tests/task317_diag.rs` — 임시 진단 테스트 3건 삭제

## 검증

### 어댑터 격리 테스트 (재활성화)

```
cargo test --test hwpx_to_hwp_adapter
test result: ok. 25 passed; 0 failed; 0 ignored
```

### 4샘플 무회귀 (release dump-pages)

| 샘플 | 기대 | 측정 |
|------|------|------|
| 21_언어_기출_편집가능본 | 15쪽 | 15쪽 ✓ |
| exam_math | 20쪽 | 20쪽 ✓ |
| exam_kor | 24쪽 | 24쪽 ✓ |
| exam_eng | 9쪽 | 9쪽 ✓ |

### 전체 테스트

```
cargo test → 992 lib + 통합 모두 PASS
(issue_301 1건 ignored — #318 별도 처리)
```

## 영향 범위

- HWPX 출처 표만 영향. `table.raw_ctrl_data` 가 이미 채워진 HWP 출처 표는 분기 비진입.
- typeset 은 `table.attr` 만 보고 `common.text_wrap/treat_as_char` 미사용이므로 typeset 결과만 정렬됨.
- 다른 `common.*` 사용 코드 (캡션/외곽선/렌더러) 는 `table.common.*` 를 직접 읽으므로 영향 없음.

## 후속 사안 (별도 sub-issue)

- typeset 의 `is_tac` 판정 일원화: 현재 `table.attr & 0x01` 사용. `common.treat_as_char` 와 일치하지 않을 수 있음. PDF baseline 영향 측정 필요로 별도 sub-issue.
- HWPX 파서가 `common.treat_as_char` 등 enum 으로부터 `table.attr/common.attr` 비트를 채우도록 일원화하면 IR 일관성 확보 가능.

## 단계별 진행

| 단계 | 내용 | 보고서 |
|------|------|--------|
| 1 | paragraph-by-paragraph current_height 추적 (env-gated trace) | `mydocs/working/task_m100_317_stage1.md` |
| 2 | 표 IR 비교 → 1차(outer_margin) 실패 → 2차(attr 비트) 성공 | `mydocs/working/task_m100_317_stage2.md` |
| 3 | 어댑터 attr=0 보강 + 회귀 검증 | `mydocs/working/task_m100_317_stage3.md` |
| 4 | 격리 테스트 재활성화 + 진단 도구 정리 | `mydocs/working/task_m100_317_stage4.md` |

최종 보고서: `mydocs/report/task_m100_317_report.md`

## Test plan

- [x] `cargo test --test hwpx_to_hwp_adapter` 25 passed, 0 ignored
- [x] `cargo test` 전체 PASS
- [x] 4샘플 (21_언어/exam_math/exam_kor/exam_eng) 페이지 수 무변화
- [x] hwpx-h-02 변환 후 9쪽 (PDF 일치)

closes #317
