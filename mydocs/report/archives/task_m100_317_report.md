# Task #317 최종 결과 보고서

**제목**: HWPX 어댑터 페이지 수 +1쪽 잔존 origin 조사 및 보강
**브랜치**: `task317`
**상위**: 잔존 사안 (Epic #309 외부 — #314 후속)

## 결과 요약

hwpx-h-02 어댑터 라운드트립 결과 +1쪽(9→10) 차이 해소. 격리되었던 어댑터 테스트 3건 재활성화. 4샘플 무회귀.

## 진짜 origin

`src/renderer/typeset.rs:695` 의 `is_tac` 판정이 `table.attr & 0x01` 비트만 사용하고 `table.common.treat_as_char` 를 무시하는 것이 근본 구조.

- DIRECT (HWPX 직접): HWPX 파서가 attr 비트를 채우지 않음 → `table.attr=0` → `is_tac=false` → block 분기
- RELOADED (HWPX→어댑터→HWP→재로드): 어댑터 `pack_common_attr_bits` 가 비트 0(treat_as_char)을 합성 → `table.attr=0x002A0211` → `is_tac=true` → TAC 분기

block 과 TAC 분기는 host_spacing/outer_margin/host_line_spacing 처리가 달라 같은 표에 대해 paragraph 당 ~2.7px 누적 차이 발생. 페이지 3 의 표 3개에서 누적 ~+8px → pi=51 (Table 326.7px) 가 가용 공간 부족으로 다음 페이지로 밀림 → +1쪽.

## 보강

`src/document_core/converters/hwpx_to_hwp.rs::adapt_table` 에서 raw_ctrl_data 합성 직후 attr 영역(offset 0..4)을 0으로 강제하고 `table.attr=0` 보존. DIRECT 와 동일한 attr=0 으로 같은 block 분기 진입.

기존 "attr 동기화" 블록 (raw_ctrl_data 의 합성 attr → table.attr 복사) 제거.

```rust
fn adapt_table(table: &mut Table, report: &mut AdapterReport) {
    if table.raw_ctrl_data.is_empty() {
        table.raw_ctrl_data = serialize_common_obj_attr(&table.common);
        report.tables_ctrl_data_synthesized += 1;
        // Task #317: HWPX 출처는 common.attr=0 이 진실. typeset 은 table.attr & 0x01 로
        // is_tac 판정하므로 어댑터의 attr 합성이 RELOADED 만 TAC 분기로 흘려 보내는 부작용.
        if table.raw_ctrl_data.len() >= 4 {
            table.raw_ctrl_data[0..4].copy_from_slice(&0u32.to_le_bytes());
        }
    }
    table.attr = 0;
    // ...
}
```

## 단계별 진행

| 단계 | 내용 | 산출 |
|------|------|------|
| 1 | paragraph-by-paragraph current_height 추적 (env-gated trace) | `task_m100_317_stage1.md` |
| 2 | 표 IR 비교 → 1차(outer_margin) 실패 → 2차(attr 비트) 성공 | `task_m100_317_stage2.md` |
| 3 | 어댑터 attr=0 보강 + 회귀 검증 | `task_m100_317_stage3.md` |
| 4 | 격리 테스트 재활성화 + 진단 도구 정리 | `task_m100_317_stage4.md` |

## 검증

### 어댑터 격리 테스트 (재활성화)
```
cargo test --test hwpx_to_hwp_adapter
test result: ok. 25 passed; 0 failed; 0 ignored
```

### 4샘플 무회귀

| 샘플 | 기대 | 측정 |
|------|------|------|
| 21_언어_기출_편집가능본 | 15 | 15 ✓ |
| exam_math | 20 | 20 ✓ |
| exam_kor | 24 | 24 ✓ |
| exam_eng | 9 | 9 ✓ |

### 전체 테스트
```
cargo test
992 lib + 25 어댑터(0 ignored) + 통합 모두 PASS
```

## 변경 파일

- `src/document_core/converters/hwpx_to_hwp.rs` — adapt_table attr=0 보강
- `tests/hwpx_to_hwp_adapter.rs` — `#[ignore]` 3건 제거
- `src/renderer/typeset.rs` — env-gated trace 제거 (1단계 도입분 회수)
- `tests/task317_diag.rs` — 삭제 (1단계 진단 도구 회수)
- `mydocs/plans/task_m100_317.md`, `task_m100_317_impl.md` (수행/구현 계획서)
- `mydocs/working/task_m100_317_stage{1..4}.md`
- `mydocs/report/task_m100_317_report.md` (본 문서)

## 후속 사안 (선택)

- **typeset 의 is_tac 판정 일원화**: 현재 `table.attr & 0x01` 사용. `table.common.treat_as_char` 와 일치하지 않을 수 있음. HWPX 직접 로드 동작이 PDF baseline 과 일치한다는 점 때문에 본 task 에서는 origin 측이 아닌 어댑터 측을 맞추는 방향으로 결론. 향후 typeset 측 통일 검토 시 별도 sub-issue.
- HWPX 파서가 `common.treat_as_char` 등 enum 으로부터 `table.attr/common.attr` 비트를 채우도록 일원화하면 IR 일관성 확보 가능. 단, 이 경우 DIRECT 동작이 변하므로 PDF baseline 영향 측정 필요.

## 완료 조건 점검

- [x] 격리된 어댑터 테스트 3건 통과
- [x] hwpx-h-02 변환 후 9쪽
- [x] cargo test 전체 회귀 0
- [x] 4샘플 (21_언어/exam_*) 무회귀
